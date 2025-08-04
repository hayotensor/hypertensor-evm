// Copyright (C) Hypertensor.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use frame_support::pallet_prelude::DispatchError;
use sp_runtime::{FixedU128, FixedPointNumber};
use frame_support::pallet_prelude::One;

impl<T: Config> Pallet<T> {
  pub fn assign_subnet_slot(subnet_id: u32) -> Result<u32, DispatchError> {
    let max_slots = T::EpochLength::get();
    // Max slots must always be > 2

    // Get currently assigned slots
    let mut assigned_slots = AssignedSlots::<T>::get();

    ensure!(
      (assigned_slots.len() as u32) < max_slots - 2,
      Error::<T>::NoAvailableSlots
    );

    // Find first free slot [2..max_slots)
    // Slot 1: Electing validators
    // Slot 2: Generating weights
    let free_slot = (2..max_slots)
      .find(|slot| !assigned_slots.contains(slot))
      .ok_or(Error::<T>::NoAvailableSlots)?;

    // Update assigned slots set
    assigned_slots.insert(free_slot);
    AssignedSlots::<T>::put(assigned_slots);

    // Assign
    SubnetSlot::<T>::insert(subnet_id, free_slot);
    SlotAssignment::<T>::insert(free_slot, subnet_id);

    Ok(free_slot)
  }

  /// Remove a subnet from its slot
  pub fn free_slot_of_subnet(subnet_id: u32) {
    let assigned_slots = AssignedSlots::<T>::get();

    if let Some(slot) = SubnetSlot::<T>::take(subnet_id) {
      SlotAssignment::<T>::remove(slot);

      let mut assigned_slots = assigned_slots;
      assigned_slots.remove(&slot);
      AssignedSlots::<T>::put(assigned_slots);
    }
  }

  pub fn get_min_subnet_nodes(base_node_memory: u128, memory_mb: u128) -> u32 {
    0
  }

  pub fn get_target_subnet_nodes(min_subnet_nodes: u32) -> u32 {
    min_subnet_nodes
  }

  /// Calculates the current subnet registration fee based on a linear decay model.
  ///
  /// The registration cost starts at a maximum value (`MaxSubnetRegistrationFee`)
  /// and linearly decreases to a minimum (`MinSubnetRegistrationFee`) over a fixed
  /// interval of epochs (`SubnetRegistrationInterval`). After the interval expires,
  /// the cost remains at the minimum fee.
  ///
  /// # Arguments
  ///
  /// * `current_epoch` - The current epoch at which registration is being attempted.
  ///
  /// # Returns
  ///
  /// * `u128` - The computed registration fee in token units.
  ///
  /// # Behavior
  ///
  /// - If no registration has ever occurred (`LastSubnetRegistrationEpoch == 0`),
  ///   the cost starts from `MaxSubnetRegistrationFee`.
  /// - The cost decreases linearly from max to min across the interval.
  /// - If the `current_epoch` is past the interval, the minimum fee is returned.
  pub fn registration_cost(current_epoch: u32) -> u128 {
    let last_registration_epoch = LastSubnetRegistrationEpoch::<T>::get();
    let fee_min: u128 = MinSubnetRegistrationFee::<T>::get();
    let fee_max: u128 = MaxSubnetRegistrationFee::<T>::get();
    let period: u32 = SubnetRegistrationInterval::<T>::get();

    // Determine the start of the current fee period
    let start_epoch = if last_registration_epoch == 0 {
      0
    } else {
      last_registration_epoch
    };

    let end_epoch = start_epoch + period;

    // If current epoch is after end of the period, return min fee
    if current_epoch >= end_epoch {
      return fee_min;
    }

    // How far into the period we are
    let cycle_epoch = current_epoch.saturating_sub(start_epoch);

    // Decrease per epoch
    let total_decrease = fee_max.saturating_sub(fee_min);
    let decrease_per_epoch = total_decrease.saturating_div(period as u128);

    // Linear decrease
    let fee = fee_max.saturating_sub(decrease_per_epoch.saturating_mul(cycle_epoch as u128));

    fee.max(fee_min)
  }

  pub fn can_subnet_register(current_epoch: u32) -> bool {
    current_epoch >= Self::get_next_registration_epoch(current_epoch)
  }

  pub fn get_next_registration_epoch(current_epoch: u32) -> u32 {
    let last_registration_epoch: u32 = LastSubnetRegistrationEpoch::<T>::get();
    let interval: u32 = SubnetRegistrationInterval::<T>::get();

    // If no registration has happened yet, return current_interval-aligned epoch
    if last_registration_epoch == 0 {
        return current_epoch - (current_epoch % interval);
    }

    // Otherwise, calculate next registration interval after last one
    ((last_registration_epoch / interval) + 1) * interval
  }

  // /// Get the next registration epoch based on an epoch
  // pub fn get_next_registration_epoch(current_epoch: u32) -> u32 {
  //   let last_registration_epoch: u32 = LastSubnetRegistrationEpoch::<T>::get();
  //   // --- Handle genesis
  //   if last_registration_epoch == 0 {
  //     return 0
  //   }

  //   let interval: u32 = SubnetRegistrationInterval::<T>::get();

  //   let offset = current_epoch % interval;


  //   last_registration_epoch.saturating_add(
  //     interval.saturating_sub(last_registration_epoch % interval)
  //   )
  // }

  // Check if the subnet state is registered, and if it's in the registration period
  pub fn is_subnet_registering(subnet_id: u32, state: SubnetState, epoch: u32) -> bool {
    let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
    let is_registering: bool = state == SubnetState::Registered;

    match SubnetRegistrationEpoch::<T>::try_get(subnet_id) {
      Ok(registered_epoch) => {
        let max_registration_epoch = registered_epoch.saturating_add(subnet_registration_epochs);
        if is_registering && epoch <= max_registration_epoch {
          return true
        }
        false
      },
      Err(()) => false,
    }
  }

  // Check if the subnet state is registered, and if it's in the enactment period
  pub fn is_subnet_in_enactment(subnet_id: u32, state: SubnetState, epoch: u32) -> bool {
    let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
    let subnet_activation_enactment_epochs = SubnetActivationEnactmentEpochs::<T>::get();

    let is_registering: bool = state == SubnetState::Registered;

    match SubnetRegistrationEpoch::<T>::try_get(subnet_id) {
      Ok(registered_epoch) => {
        let max_registration_epoch = registered_epoch.saturating_add(subnet_registration_epochs);
        let max_enactment_epoch = max_registration_epoch.saturating_add(subnet_activation_enactment_epochs);
    
        if is_registering && epoch <= max_registration_epoch {
          return false
        } else if is_registering && epoch <= max_enactment_epoch {
          return true
        }
        false
      },
      Err(()) => false,
    }
    // let registered_epoch: u32 = subnet_data.registered;
    // let max_registration_epoch = registered_epoch.saturating_add(subnet_registration_epochs);
    // let max_enactment_epoch = max_registration_epoch.saturating_add(subnet_activation_enactment_epochs);

    // if is_registering && epoch <= max_registration_epoch {
    //   return false
    // } else if is_registering && epoch <= max_enactment_epoch {
    //   return true
    // }
    // false
  }

  pub fn is_subnet_active(subnet_id: u32) -> Option<bool> {
    match SubnetsData::<T>::try_get(subnet_id) {
      Ok(subnet) => Some(subnet.state == SubnetState::Active),
      Err(()) => None,
    }
  }

  pub fn is_subnet_paused(subnet_id: u32) -> Option<bool> {
    match SubnetsData::<T>::try_get(subnet_id) {
      Ok(subnet) => Some(subnet.state == SubnetState::Paused),
      Err(()) => None,
    }
  }

  pub fn is_subnet_owner(account_id: &T::AccountId, subnet_id: u32) -> Option<bool> {
    match SubnetOwner::<T>::try_get(subnet_id) {
      Ok(owner) => Some(&owner == account_id),
      Err(()) => None,
    }
  }

  // pub fn get_current_registration_cost() -> u128 {
  //   let last_registration_cost = LastRegistrationCost::<T>::get();
  //   let min_price = MinRegistrationCost::<T>::get();
  //   let last_updated = LastRegistrationBlock::<T>::get();
  //   let decay_blocks = RegistrationCostDecayBlocks::<T>::get();
  //   log::error!("last_registration_cost {:?}", last_registration_cost);
  //   log::error!("min_price              {:?}", min_price);
  //   log::error!("last_updated           {:?}", last_updated);
  //   log::error!("decay_blocks           {:?}", decay_blocks);

  //   let current_block = Self::get_current_block_as_u32();
  //   let delta_blocks = current_block.saturating_sub(last_updated);
  //   log::error!("current_block          {:?}", current_block);
  //   log::error!("delta_blocks           {:?}", delta_blocks);

  //   if decay_blocks == 0 || last_registration_cost <= min_price {
  //     return last_registration_cost.max(min_price);
  //   }

  //   let diff = last_registration_cost.saturating_sub(min_price);
  //   log::error!("diff                   {:?}", diff);

  //   // Calculate decay exponent: delta_blocks / decay_blocks
  //   // This is in fixed point
  //   let ratio = FixedU128::saturating_from_rational(delta_blocks as u128, decay_blocks as u128);
  //   log::error!("ratio                  {:?}", ratio);

  //   // Approximate e^(-ratio) using pow-based approximation
  //   // For small x, (1 - x / n)^n ~ e^-x. We'll pick n = 64 for reasonable accuracy.
  //   let n: u128 = 64;
  //   // Divide ratio by integer n:
  //   let n_fixed = FixedU128::from_inner(n * 1000000000000000000);
  //   log::error!("n_fixed                {:?}", n_fixed);

  //   let base_factor = match ratio.const_checked_div(n_fixed) {
  //     Some(v) => v,
  //     None => FixedU128::one()
  //   };
  //   log::error!("base_factor            {:?}", base_factor);

  //   let base = FixedU128::one().saturating_sub(base_factor);
    
  //   // decay_factor = base ^ n
  //   let decay_factor = base.saturating_pow(n as usize);

  //   log::error!("base                   {:?}", base);
  //   log::error!("decay_factor           {:?}", decay_factor);

  //   let decayed = min_price.saturating_add(
  //     decay_factor.saturating_mul_int(diff)
  //   );
  //   log::error!("decayed                {:?}", decayed);
  //   log::error!("decayed                {:?}", decayed.max(min_price));

  //   decayed.max(min_price)
  // }

  // pub fn get_current_registration_cost() -> u128 {
  //   let last_registration_cost = LastRegistrationCost::<T>::get();
  //   let min_price = MinRegistrationCost::<T>::get();
  //   let last_updated = LastRegistrationBlock::<T>::get();
  //   let decay_blocks = RegistrationCostDecayBlocks::<T>::get();
  //   log::error!("decay_blocks           {:?}", decay_blocks);

  //   let current_block = Self::get_current_block_as_u32();
  //   let delta_blocks = current_block.saturating_sub(last_updated);
  //   log::error!("current_block          {:?}", current_block);
  //   log::error!("delta_blocks           {:?}", delta_blocks);

  //   // Already at min or no decay period
  //   if decay_blocks == 0 || last_registration_cost <= min_price {
  //     return last_registration_cost.max(min_price);
  //   }

  //   let diff = last_registration_cost.saturating_sub(min_price);
  //   log::error!("diff                   {:?}", diff);

  //   // Linear decay: factor = 1 - delta / decay_blocks
  //   let factor = decay_blocks.saturating_sub(delta_blocks) as u128;
  //   let decayed = min_price.saturating_add(diff.saturating_mul(factor) / decay_blocks as u128);
  //   log::error!("factor                 {:?}", factor);
  //   log::error!("decayed                {:?}", decayed);

  //   decayed.max(min_price)
  // }

  pub fn get_current_registration_cost(block: u32) -> u128 {
    let last_registration_cost = LastRegistrationCost::<T>::get();
    let min_price = MinRegistrationCost::<T>::get();
    let last_updated = LastRegistrationBlock::<T>::get();
    let decay_blocks = RegistrationCostDecayBlocks::<T>::get();
    let alpha = RegistrationCostAlpha::<T>::get();

    let delta_blocks = block.saturating_sub(last_updated);

    // Already at min or no decay period
    if decay_blocks == 0 || last_registration_cost <= min_price {
      return last_registration_cost.max(min_price);
    }

    // Fully decayed: exactly min price
    if delta_blocks >= decay_blocks {
      return min_price;
    }

    let diff = last_registration_cost.saturating_sub(min_price);

    let remaining_frac = Self::percent_div((decay_blocks - delta_blocks) as u128, decay_blocks as u128);

    // Apply concave exponential: exponent α < 1
    // e.g., α = 0.5 = sqrt (concave)
    // concave factor = remaining_frac ^ alpha
    let concave_factor = Self::pow(Self::get_percent_as_f64(remaining_frac), Self::get_percent_as_f64(alpha));

    // price = min_price + diff * concave_factor
    let decayed = min_price.saturating_add(
      (diff as f64 * concave_factor) as u128
    );

    decayed.max(min_price)
  }

  pub fn update_last_registration_cost(current_cost: u128, block: u32) {
    let new_cost = Self::percent_mul(current_cost, NewRegistrationCostMultiplier::<T>::get());
    LastRegistrationCost::<T>::put(new_cost);
    LastRegistrationBlock::<T>::put(block);
  }

  /// Update bootnode set
  ///
  /// Allows accessible (set by owner) to set the official bootnodes
  ///
  /// These are used for new nodes and overwatch nodes
  ///
  /// * Note: Each subnet node can have a bootnode and Overwatchers will check those as well
  ///
  /// subnet_id: Subnet ID of bootnode set
  /// add: Bootnodes to add to set
  /// remove: Bootnodes to remove from set
  pub fn do_update_bootnodes(
    origin: T::RuntimeOrigin, 
    subnet_id: u32, 
    add: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>, 
    remove: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>
  ) -> DispatchResult {
    let account_id: T::AccountId = ensure_signed(origin)?;

    ensure!(SubnetsData::<T>::contains_key(subnet_id), Error::<T>::InvalidSubnetId);

    ensure!(SubnetBootnodeAccess::<T>::get(subnet_id).contains(&account_id), Error::<T>::InvalidAccess);

    let max_bootnodes = MaxBootnodes::<T>::get();
    
    SubnetBootnodes::<T>::try_mutate(subnet_id, |bootnodes| -> DispatchResult {
      for item in remove.iter() {
        bootnodes.remove(item);
      }

      for item in add.iter() {
        // Check in the for loop for length in case some inserts are false
        ensure!(
          bootnodes.len() < max_bootnodes as usize,
          Error::<T>::TooManyBootnodes
        );
        bootnodes.insert(item.clone());
      }

      Ok(())
    })?;

    Self::deposit_event(Event::BootnodesUpdated {
      subnet_id,
      added: add,
      removed: remove,
    });

    Ok(())
  }
}