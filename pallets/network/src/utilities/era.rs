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

  pub fn do_epoch_preliminaries(block: u32, epoch: u32) {
    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
    let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
    let subnet_activation_enactment_epochs = SubnetActivationEnactmentEpochs::<T>::get();
    let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance();

    let subnets: Vec<_> = SubnetsData::<T>::iter().collect();
    let total_subnets: u32 = subnets.len() as u32;
    let excess_subnets: bool = total_subnets > MaxSubnets::<T>::get();
    let mut subnet_delegate_stake: Vec<(u32, u128)> = Vec::new();
    let min_subnet_nodes = MinSubnetNodes::<T>::get();

    for (subnet_id, data) in subnets {
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

      let is_registering = data.state == SubnetState::Registered;
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
              let subnet_node_ids: Vec<u32> = Self::get_classified_subnet_node_ids(subnet_id, &SubnetNodeClass::Validator, epoch);
              let subnet_nodes_count = subnet_node_ids.len();  
              if (subnet_nodes_count as u32) < min_subnet_nodes {
                Self::do_remove_subnet(
                  subnet_id,
                  SubnetRemovalReason::MinSubnetNodes,
                );
              }
              continue
            } else if is_registering && epoch > max_enactment_epoch {
              // --- Out of Enactment Period
              // If out of enactment period and not activated, remove subnet
              Self::do_remove_subnet(
                subnet_id,
                SubnetRemovalReason::EnactmentPeriod,
              );
              continue
            }
          },
          Err(()) => (),
        };  
      }

      if data.start_epoch > epoch {
        continue
      }

      // --- All subnets are now activated and passed the registration period
      // Must have:
      //  - Minimum nodes (increases penalties if less than - later removed if over max penalties)
      //  - Minimum delegate stake balance (remove subnet if less than)
			let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

      // --- Ensure min delegate stake balance is met
      if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
        Self::do_remove_subnet(
          subnet_id,
          SubnetRemovalReason::MinSubnetDelegateStake,
        );
        continue
      }

      // --- Get all possible validators
      let subnet_node_ids: Vec<u32> = Self::get_classified_subnet_node_ids(subnet_id, &SubnetNodeClass::Validator, epoch);
      let subnet_nodes_count = subnet_node_ids.len();
      
      // --- Ensure min nodes are active
      // Only choose validator if min nodes are present
      // The ``SubnetPenaltyCount`` when surpassed doesn't penalize anyone, only removes the subnet from the chain
      if (subnet_nodes_count as u32) < min_subnet_nodes {
        // Nodes may be deactivated so we don't remove the subnet here, but increase penalties instead
        // Note: Subnets decrease its number of penalties for each successful epoch
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
      }

      // --- Check penalties and remove subnet is threshold is breached
      let penalties = SubnetPenaltyCount::<T>::get(subnet_id);
      if penalties > max_subnet_penalty_count {
        Self::do_remove_subnet(
          subnet_id,
          SubnetRemovalReason::MaxPenalties,
        );
        continue
      }

      if excess_subnets {
        subnet_delegate_stake.push((subnet_id, subnet_delegate_stake_balance));
      }

      Self::choose_validator(
        block,
        subnet_id,
        subnet_node_ids,
        min_subnet_nodes,
        epoch,
      );
    }

    // --- If over max subnets, remove the subnet with the lowest delegate stake
    if excess_subnets {
      subnet_delegate_stake.sort_by_key(|&(_, value)| value);
      Self::do_remove_subnet(
        subnet_delegate_stake[0].0.clone(),
        SubnetRemovalReason::MaxSubnets,
      );
    }

    // --- TODO: Push subnet_ids and subnet_nodes into mapping and choose validator after possible removal of subnet
    // Avoid randomization if there are max subnets
  }

  // pub fn do_epoch_preliminaries(block: u32, epoch: u32, epoch_length: u32) {
  //   let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
  //   let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
  //   let subnet_activation_enactment_epochs = SubnetActivationEnactmentEpochs::<T>::get();
  //   let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance();

  //   let subnets: Vec<_> = SubnetsData::<T>::iter().collect();
  //   let total_subnets: u32 = subnets.len() as u32;
  //   let excess_subnets: bool = total_subnets > MaxSubnets::<T>::get();
  //   let mut subnet_delegate_stake: Vec<(u32, u128)> = Vec::new();
  //   let min_subnet_nodes = MinSubnetNodes::<T>::get();

  //   for (subnet_id, data) in subnets {
  //     // ==========================
  //     // # Logic
  //     //
  //     // *Registration Period:
  //     //  - Can exist no matter what
  //     //
  //     // *Enactment Period:
  //     //  - Must have min nodes.
  //     //  *We don't check on min delegate stake balance here.
  //     //  - We allow being under min delegate stake to allow delegate stake conditions to be met before
  //     //    the end of the enactment period.
  //     //
  //     // *Out of Enactment Period:
  //     //  - Remove if not activated.
  //     //
  //     // ==========================

  //     let is_registering = data.state == SubnetState::Registered;
  //     if is_registering {
  //       match SubnetRegistrationEpoch::<T>::try_get(subnet_id) {
  //         Ok(registered_epoch) => {
            
  //           let max_registration_epoch = registered_epoch.saturating_add(subnet_registration_epochs);
  //           let max_enactment_epoch = max_registration_epoch.saturating_add(subnet_activation_enactment_epochs);

  //           if is_registering && epoch <= max_registration_epoch {
  //             // --- Registration Period
  //             // If in registration period, do nothing
  //             continue
  //           } else if is_registering && epoch <= max_enactment_epoch {
  //             // --- Enactment Period
  //             // If in enactment period, ensure min nodes
  //             let subnet_node_ids: Vec<u32> = Self::get_classified_subnet_node_ids(subnet_id, &SubnetNodeClass::Validator, epoch);
  //             let subnet_nodes_count = subnet_node_ids.len();  
  //             if (subnet_nodes_count as u32) < min_subnet_nodes {
  //               Self::do_remove_subnet(
  //                 subnet_id,
  //                 SubnetRemovalReason::MinSubnetNodes,
  //               );
  //             }
  //             continue
  //           } else if is_registering && epoch > max_enactment_epoch {
  //             // --- Out of Enactment Period
  //             // If out of enactment period and not activated, remove subnet
  //             Self::do_remove_subnet(
  //               subnet_id,
  //               SubnetRemovalReason::EnactmentPeriod,
  //             );
  //             continue
  //           }
  //         },
  //         Err(()) => (),
  //       };  
  //     }

  //     // --- All subnets are now activated and passed the registration period
  //     // Must have:
  //     //  - Minimum nodes (increases penalties if less than - later removed if over max penalties)
  //     //  - Minimum delegate stake balance (remove subnet if less than)
	// 		let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

  //     // --- Ensure min delegate stake balance is met
  //     if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
  //       Self::do_remove_subnet(
  //         subnet_id,
  //         SubnetRemovalReason::MinSubnetDelegateStake,
  //       );
  //       continue
  //     }

  //     // --- Get all possible validators
  //     let subnet_node_ids: Vec<u32> = Self::get_classified_subnet_node_ids(subnet_id, &SubnetNodeClass::Validator, epoch);
  //     let subnet_nodes_count = subnet_node_ids.len();
      
  //     // --- Ensure min nodes are active
  //     // Only choose validator if min nodes are present
  //     // The ``SubnetPenaltyCount`` when surpassed doesn't penalize anyone, only removes the subnet from the chain
  //     if (subnet_nodes_count as u32) < min_subnet_nodes {
  //       SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
  //     }

  //     // --- Check penalties and remove subnet is threshold is breached
  //     let penalties = SubnetPenaltyCount::<T>::get(subnet_id);
  //     if penalties > max_subnet_penalty_count {
  //       Self::do_remove_subnet(
  //         subnet_id,
  //         SubnetRemovalReason::MaxPenalties,
  //       );
  //       continue
  //     }

  //     if excess_subnets {
  //       subnet_delegate_stake.push((subnet_id, subnet_delegate_stake_balance));
  //     }

  //     Self::choose_validator(
  //       block,
  //       subnet_id,
  //       subnet_node_ids,
  //       min_subnet_nodes,
  //       epoch,
  //     );
  //   }

  //   // --- If over max subnets, remove the subnet with the lowest delegate stake
  //   if excess_subnets {
  //     subnet_delegate_stake.sort_by_key(|&(_, value)| value);
  //     Self::do_remove_subnet(
  //       subnet_delegate_stake[0].0.clone(),
  //       SubnetRemovalReason::MaxSubnets,
  //     );
  //   }

  //   // --- TODO: Push subnet_ids and subnet_nodes into mapping and choose validator after possible removal of subnet
  //   // Avoid randomization if there are max subnets
  // }
}