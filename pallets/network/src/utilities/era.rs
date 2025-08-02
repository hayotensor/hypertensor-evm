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
use frame_system::pallet_prelude::BlockNumberFor;
use frame_support::pallet_prelude::Weight;

impl<T: Config> Pallet<T> {
  pub fn get_current_block_as_u64() -> u64 {
    TryInto::try_into(<frame_system::Pallet<T>>::block_number())
      .ok()
      .expect("blockchain will not exceed 2^64 blocks; QED.")
  }

  pub fn convert_block_as_u64(block: BlockNumberFor<T>) -> u64 {
    TryInto::try_into(block)
      .ok()
      .expect("blockchain will not exceed 2^64 blocks; QED.")
  }
  
  pub fn get_current_block_as_u32() -> u32 {
    TryInto::try_into(<frame_system::Pallet<T>>::block_number())
      .ok()
      .expect("blockchain will not exceed 2^32 blocks; QED.")
  }

  pub fn convert_block_as_u32(block: BlockNumberFor<T>) -> u32 {
    TryInto::try_into(block)
      .ok()
      .expect("blockchain will not exceed 2^32 blocks; QED.")
  }

  pub fn get_current_epoch_as_u32() -> u32 {
    let current_block = Self::get_current_block_as_u32();
    let epoch_length: u32 = T::EpochLength::get();
    current_block.saturating_div(epoch_length)
  }

  pub fn get_current_epoch_with_block_as_u32(current_block: u32) -> u32 {
    let epoch_length: u32 = T::EpochLength::get();
    current_block.saturating_div(epoch_length)
  }

  pub fn get_current_subnet_epoch_as_u32(subnet_id: u32) -> u32 {
    let epoch_length = T::EpochLength::get();
    let subnet_slot = match SubnetSlot::<T>::try_get(subnet_id) {
      Ok(slot) => slot,
      Err(_) => 0,
    };
    if subnet_slot == 0 {
      return 0;
    }

    let current_block = Self::get_current_block_as_u32();
    
    if current_block < subnet_slot {
      return 0;
    }

    // Example: 150 = 200-50
    let offset_block = current_block - subnet_slot;

    // Example: 150 = 150 / 100
    offset_block / epoch_length    
  }

  pub fn do_epoch_preliminaries(block: u32, epoch: u32) -> Weight {
    let mut weight = Weight::zero();
    let db_weight = T::DbWeight::get();

    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
    let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
    let subnet_activation_enactment_epochs = SubnetActivationEnactmentEpochs::<T>::get();
    let min_subnet_nodes = MinSubnetNodes::<T>::get();
    let max_subnets = MaxSubnets::<T>::get();
    let max_pause_epochs = MaxSubnetPauseEpochs::<T>::get();

    weight = weight.saturating_add(db_weight.reads(5));

    let subnets: Vec<_> = SubnetsData::<T>::iter().collect();
    let total_subnets: u32 = subnets.len() as u32;
    weight = weight.saturating_add(db_weight.reads(total_subnets.into()));
    
    let excess_subnets: bool = total_subnets > max_subnets;
    let mut subnet_delegate_stake: Vec<(u32, u128)> = Vec::new();

    for (subnet_id, data) in &subnets {
      // ==========================
      // # Logic
      //
      // *Registration Period:
      //  - Can exist no matter what
      //
      // *Enactment Period:
      //  - Must have min nodes.
      //  *We don't check on min delegate stake balance here.
      //  - We allow being under min delegate stake to allow delegate stake conditions to be met before
      //    the end of the enactment period.
      //
      // *Out of Enactment Period:
      //  - Remove if not activated.
      //
      // ==========================

      let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance_v2(*subnet_id);

      let is_registering = data.state == SubnetState::Registered;
      let is_paused = data.state == SubnetState::Paused;
      if is_registering {
        match SubnetRegistrationEpoch::<T>::try_get(subnet_id) {
          Ok(registered_epoch) => {
            
            let max_registration_epoch = registered_epoch.saturating_add(subnet_registration_epochs);
            let max_enactment_epoch = max_registration_epoch.saturating_add(subnet_activation_enactment_epochs);

            if is_registering && epoch <= max_registration_epoch {
              // --- Registration Period
              // If in registration period, do nothing
              continue
            } else if is_registering && epoch <= max_enactment_epoch {
              // --- Enactment Period
              // If in enactment period, ensure min nodes
              // Otherwise continue
              let active_subnet_nodes_count = TotalActiveSubnetNodes::<T>::get(subnet_id);
              weight = weight.saturating_add(db_weight.reads(1));
              if active_subnet_nodes_count < min_subnet_nodes {
                Self::do_remove_subnet(
                  *subnet_id,
                  SubnetRemovalReason::MinSubnetNodes,
                );
                // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
              }
              continue
            } else if is_registering && epoch > max_enactment_epoch {
              // --- Out of Enactment Period
              // If out of enactment period and not activated, remove subnet
              Self::do_remove_subnet(
                *subnet_id,
                SubnetRemovalReason::EnactmentPeriod,
              );
              // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
              continue
            }
          },
          Err(()) => (),
        };  
      }

      if is_paused {
        if data.start_epoch + max_pause_epochs < epoch {
          // Automatic Expiry / Force Unpause
          // - Increase penalty count
          // - Check if should remove
          // - Force unpause

          SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
          weight = weight.saturating_add(db_weight.writes(1));

          let penalties = SubnetPenaltyCount::<T>::get(subnet_id);
          weight = weight.saturating_add(db_weight.reads(1));

          if penalties > max_subnet_penalty_count {
            // --- Remove
            Self::do_remove_subnet(
              *subnet_id,
              SubnetRemovalReason::PauseExpired,
            );
            // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
            continue
          } 
          // else {
          //   SubnetsData::<T>::mutate(subnet_id, |maybe_data| {
          //     if let Some(data) = maybe_data {
          //       data.state = SubnetState::Active;
          //       data.start_epoch = epoch + 1;
          //     }
          //   });
          //   continue
          // }
        }
      }

      if data.start_epoch > epoch {
        continue
      }

      // --- All subnets are now activated and passed the registration period
      // Must have:
      //  - Minimum nodes (increases penalties if less than - later removed if over max penalties)
      //  - Minimum delegate stake balance (remove subnet if less than)
			let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
      weight = weight.saturating_add(db_weight.reads(1));

      // --- Ensure min delegate stake balance is met
      if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
        Self::do_remove_subnet(
          *subnet_id,
          SubnetRemovalReason::MinSubnetDelegateStake,
        );
        // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
        continue
      }

      let active_subnet_nodes_count = TotalActiveSubnetNodes::<T>::get(subnet_id);
      weight = weight.saturating_add(db_weight.reads(1));

      // --- Ensure min nodes are active
      // Only choose validator if min nodes are present
      // The ``SubnetPenaltyCount`` when surpassed doesn't penalize anyone, only removes the subnet from the chain
      if active_subnet_nodes_count < min_subnet_nodes {
        // Nodes may be deactivated so we don't remove the subnet here, but increase penalties instead
        // Note: Subnets decrease its number of penalties for each successful epoch
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
        weight = weight.saturating_add(db_weight.writes(1));
      }

      // --- Check penalties and remove subnet is threshold is breached
      let penalties = SubnetPenaltyCount::<T>::get(subnet_id);
      weight = weight.saturating_add(db_weight.reads(1));
      if penalties > max_subnet_penalty_count {
        Self::do_remove_subnet(
          *subnet_id,
          SubnetRemovalReason::MaxPenalties,
        );
        // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
        continue
      }

      if excess_subnets {
        subnet_delegate_stake.push((*subnet_id, subnet_delegate_stake_balance));
      }
    }

    // --- If over max subnets, remove the subnet with the lowest delegate stake
    if excess_subnets {
      subnet_delegate_stake.sort_by_key(|&(_, value)| value);
      Self::do_remove_subnet(
        subnet_delegate_stake[0].0.clone(),
        SubnetRemovalReason::MaxSubnets,
      );
      // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
    }

    weight
  }

  pub fn elect_validator_v3(
    subnet_id: u32,
    epoch: u32,
    block: u32
  ) {
    // Redundant
    // If validator already chosen, then return
    if let Ok(validator_id) = SubnetElectedValidator::<T>::try_get(subnet_id, epoch) {
      return
    }

    let slot_list = SubnetNodeElectionSlots::<T>::get(subnet_id);

    if slot_list.is_empty() {
      return
    }

    let random_number = Self::get_random_number(block);

    let idx = (random_number as usize) % slot_list.len();

    let subnet_node_id = slot_list.get(idx).cloned();

    if subnet_node_id.is_some() {
      // --- Insert validator for next epoch
      SubnetElectedValidator::<T>::insert(subnet_id, epoch, subnet_node_id.unwrap());
    }
  }
}