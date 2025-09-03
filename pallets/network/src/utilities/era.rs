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
use frame_support::pallet_prelude::Weight;
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

    pub fn get_current_overwatch_epoch_as_u32() -> u32 {
        let current_block = Self::get_current_block_as_u32();
        let epoch_length: u32 = T::EpochLength::get();
        let multiplier: u32 = OverwatchEpochLengthMultiplier::<T>::get();
        current_block.saturating_div(epoch_length.saturating_mul(multiplier))
    }

    pub fn in_overwatch_commit_period() -> bool {
        let current_block = Self::get_current_block_as_u32();
        let epoch_length: u32 = T::EpochLength::get();
        let multiplier: u32 = OverwatchEpochLengthMultiplier::<T>::get();
        let overwatch_epoch_length = epoch_length.saturating_mul(multiplier);
        let current_epoch = current_block.saturating_div(overwatch_epoch_length);
        let cutoff_percentage = OverwatchCommitCutoffPercent::<T>::get();
        let block_increase_cutoff =
            Self::percent_mul(overwatch_epoch_length as u128, cutoff_percentage);
        // start_block + cutoff blocks
        let epoch_cutoff_block =
            overwatch_epoch_length * current_epoch + block_increase_cutoff as u32;
        current_block < epoch_cutoff_block
    }

    /// Return epoch, overwatch epoch
    pub fn get_current_epochs_as_u32() -> (u32, u32) {
        let current_block = Self::get_current_block_as_u32();
        let epoch_length: u32 = T::EpochLength::get();
        let multiplier: u32 = OverwatchEpochLengthMultiplier::<T>::get();
        let epoch = current_block.saturating_div(epoch_length);
        (
            epoch,
            current_block.saturating_div(epoch_length.saturating_mul(multiplier)),
        )
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

    /// Performs preliminary subnet checks and maintenance at the start of each epoch.
    ///
    /// This function iterates over all registered subnets and enforces several rules:
    ///
    /// - Subnets in the **registration period** are allowed to exist without penalty.
    /// - Subnets in the **enactment period** must meet minimum active node counts or get removed.
    /// - Subnets **out of enactment period** but not activated are removed.
    /// - Subnets in the **paused state** are penalized if they exceed allowed pause duration, potentially leading to removal.
    /// - Activated subnets are checked to ensure they meet minimum delegate stake requirements; otherwise they are removed.
    /// - Activated subnets with insufficient active nodes accumulate penalties.  
    /// - Subnets exceeding the maximum penalty count are removed.
    /// - If the total number of subnets exceeds the configured maximum, the subnet with the lowest delegate stake is removed.
    ///
    /// Penalties are global and can be increased by other runtime logic as well, so this function enforces removal
    /// conditions based on the current penalty count regardless of its origin.
    ///
    /// # Arguments
    ///
    /// * `block` - The current block number (not used directly in the current implementation but reserved for weight calculations).
    /// * `epoch` - The current epoch number.
    ///
    /// # Returns
    ///
    /// The accumulated weight consumed by database reads and writes during the operation.
    ///
    /// # Notes
    ///
    /// - The function uses storage reads and writes extensively; weights are accumulated accordingly.
    /// - Subnet removal triggers are delegated to `do_remove_subnet`.
    ///
    pub fn do_epoch_preliminaries(block: u32, epoch: u32) -> Weight {
        let mut weight = Weight::zero();
        let db_weight = T::DbWeight::get();

        let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
        let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
        let subnet_enactment_epochs = SubnetActivationEnactmentEpochs::<T>::get();
        let min_subnet_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_pause_epochs = MaxSubnetPauseEpochs::<T>::get();

        weight = weight.saturating_add(db_weight.reads(6));

        let subnets: Vec<_> = SubnetsData::<T>::iter().collect();
        let total_subnets: u32 = subnets.len() as u32;
        weight = weight.saturating_add(db_weight.reads(total_subnets.into()));

        let excess_subnets: bool = total_subnets > max_subnets;
        let mut subnet_delegate_stake: Vec<(u32, u128)> = Vec::new();

        for (subnet_id, data) in &subnets {
            // --- Registration logic
            if data.state == SubnetState::Registered {
                if let Ok(registered_epoch) = SubnetRegistrationEpoch::<T>::try_get(subnet_id) {
                    // --- Do the registration and enactment period math manually instead of using helper functions to avoid duplicate lookups
                    let max_registration_epoch =
                        registered_epoch.saturating_add(subnet_registration_epochs);
                    let max_enactment_epoch =
                        max_registration_epoch.saturating_add(subnet_enactment_epochs);

                    if epoch <= max_registration_epoch {
                        // --- Registration Period: do nothing
                        continue;
                    }

                    if epoch <= max_enactment_epoch {
                        // --- Enactment Period: check min nodes
                        let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
                        weight = weight.saturating_add(db_weight.reads(1));

                        if active_nodes < min_subnet_nodes {
                            Self::do_remove_subnet(*subnet_id, SubnetRemovalReason::MinSubnetNodes);
                            // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
                        }
                        continue;
                    }

                    // --- Out of Enactment Period: not activated → remove
                    Self::do_remove_subnet(*subnet_id, SubnetRemovalReason::EnactmentPeriod);
                    // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
                    continue;
                }
                continue;
            }

            // --- Pause logic
            if data.state == SubnetState::Paused {
                if data.start_epoch + max_pause_epochs < epoch {
                    SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
                    weight = weight.saturating_add(db_weight.writes(1));

                    let penalties = SubnetPenaltyCount::<T>::get(subnet_id);
                    weight = weight.saturating_add(db_weight.reads(1));

                    if penalties > max_subnet_penalty_count {
                        // --- Remove
                        Self::do_remove_subnet(*subnet_id, SubnetRemovalReason::PauseExpired);
                        // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
                        continue;
                    }
                }
                continue;
            }

            // Ignore if not started yet
            if data.start_epoch > epoch {
                continue;
            }

            // --- Activated subnet checks
            let min_subnet_delegate_stake_balance =
                Self::get_min_subnet_delegate_stake_balance_v2(*subnet_id);
            let subnet_delegate_stake_balance =
                TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
            weight = weight.saturating_add(db_weight.reads(1));

            // Remove if below delegate stake requirement
            if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
                Self::do_remove_subnet(*subnet_id, SubnetRemovalReason::MinSubnetDelegateStake);
                // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
                continue;
            }

            // Check min nodes (we don't kick active subnet for this to give them time to recoup)
            let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
            weight = weight.saturating_add(db_weight.reads(1));

            if active_nodes < min_subnet_nodes {
                SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
                weight = weight.saturating_add(db_weight.writes(1));
            }

            let penalties = SubnetPenaltyCount::<T>::get(subnet_id);
            weight = weight.saturating_add(db_weight.reads(1));
            if penalties > max_subnet_penalty_count {
                Self::do_remove_subnet(*subnet_id, SubnetRemovalReason::MaxPenalties);
                // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
                continue;
            }

            // Store delegate stake for possible excess removal
            if excess_subnets {
                subnet_delegate_stake.push((*subnet_id, subnet_delegate_stake_balance));
            }
        }

        // --- Excess subnet removal
        if excess_subnets && !subnet_delegate_stake.is_empty() {
            subnet_delegate_stake.sort_by_key(|&(_, value)| value);
            Self::do_remove_subnet(
                subnet_delegate_stake[0].0.clone(),
                SubnetRemovalReason::MaxSubnets,
            );
            // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
        }

        weight
    }

    // pub fn do_epoch_preliminaries_v2(block: u32, epoch: u32) -> Weight {
    //   let mut weight = Weight::zero();
    //   let db_weight = T::DbWeight::get();

    //   let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
    //   let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
    //   let subnet_enactment_epochs = SubnetActivationEnactmentEpochs::<T>::get();
    //   let min_subnet_nodes = MinSubnetNodes::<T>::get();
    //   let max_subnets = MaxSubnets::<T>::get();
    //   let max_pause_epochs = MaxSubnetPauseEpochs::<T>::get();

    //   weight = weight.saturating_add(db_weight.reads(6));

    //   let subnets: Vec<_> = SubnetsData::<T>::iter().collect();
    //   let total_subnets: u32 = subnets.len() as u32;
    //   weight = weight.saturating_add(db_weight.reads(total_subnets.into()));

    //   let excess_subnets: bool = total_subnets > max_subnets;
    //   let mut subnet_delegate_stake: Vec<(u32, u128)> = Vec::new();

    //   for (subnet_id, data) in &subnets {
    //     // --- Registration logic
    //     if data.state == SubnetState::Registered {
    //       if let Ok(registered_epoch) = SubnetRegistrationEpoch::<T>::try_get(subnet_id) {
    //         let max_registration_epoch = registered_epoch.saturating_add(subnet_registration_epochs);
    //         let max_enactment_epoch = max_registration_epoch.saturating_add(subnet_enactment_epochs);

    //         if epoch <= max_registration_epoch {
    //           // --- Registration Period: do nothing
    //           continue
    //         }

    //         if epoch <= max_enactment_epoch {
    //           // --- Enactment Period: check min nodes
    //           let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
    //           weight = weight.saturating_add(db_weight.reads(1));

    //           if active_nodes < min_subnet_nodes {
    //             Self::do_remove_subnet(*subnet_id, SubnetRemovalReason::MinSubnetNodes);
    //             // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
    //           }
    //           continue
    //         }

    //         // --- Out of Enactment Period: not activated → remove
    //         Self::do_remove_subnet(*subnet_id, SubnetRemovalReason::EnactmentPeriod);
    //         // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
    //         continue
    //       }
    //       continue
    //     }

    //     // --- Pause logic
    //     if data.state == SubnetState::Paused {
    //       if data.start_epoch + max_pause_epochs < epoch {
    //         SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
    //         weight = weight.saturating_add(db_weight.writes(1));

    //         let penalties = SubnetPenaltyCount::<T>::get(subnet_id);
    //         weight = weight.saturating_add(db_weight.reads(1));

    //         if penalties > max_subnet_penalty_count {
    //           // --- Remove (If paused, assume it's because of pause expiration)
    //           Self::do_remove_subnet(*subnet_id, SubnetRemovalReason::PauseExpired);
    //           // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
    //           continue
    //         }
    //       }
    //       continue
    //     }

    //     // Ignore if not started yet (after pause check because pause start_epoch is the epoch is paused)
    //     if data.start_epoch > epoch {
    //       continue
    //     }

    //     let (can_subnet_be_active, reason) = Self::can_subnet_be_active(*subnet_id);

    //     if !can_subnet_be_active {
    //       let reason_unwrapped = reason.unwrap();
    //       if reason_unwrapped == SubnetRemovalReason::MaxPenalties {
    //         weight = weight.saturating_add(db_weight.writes(2));
    //       } else if reason_unwrapped == SubnetRemovalReason::MinSubnetNodes {
    //         weight = weight.saturating_add(db_weight.writes(4));
    //       } else if reason_unwrapped == SubnetRemovalReason::MinSubnetDelegateStake {
    //         weight = weight.saturating_add(db_weight.writes(9));
    //       }
    //       Self::do_remove_subnet(*subnet_id, reason_unwrapped);
    //       continue
    //     }

    //     // Store delegate stake for possible excess removal
    //     if excess_subnets {
    //       let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
    //       weight = weight.saturating_add(db_weight.writes(1));
    //       subnet_delegate_stake.push((*subnet_id, subnet_delegate_stake_balance));
    //     }
    //   }

    //   // --- Excess subnet removal
    //   if excess_subnets && !subnet_delegate_stake.is_empty() {
    //     subnet_delegate_stake.sort_by_key(|&(_, value)| value);
    //     Self::do_remove_subnet(subnet_delegate_stake[0].0.clone(), SubnetRemovalReason::MaxSubnets);
    //     // weight = weight.saturating_add(T::WeightInfo::do_remove_subnet());
    //   }

    //   weight
    // }

    pub fn elect_validator_v3(subnet_id: u32, subnet_epoch: u32, block: u32) {
        // Redundant
        // If validator already chosen, then return
        if let Ok(validator_id) = SubnetElectedValidator::<T>::try_get(subnet_id, subnet_epoch) {
            return;
        }

        let slot_list = SubnetNodeElectionSlots::<T>::get(subnet_id);

        if slot_list.is_empty() {
            return;
        }

        let random_number = Self::get_random_number(block);

        let idx = (random_number as usize) % slot_list.len();

        let subnet_node_id = slot_list.get(idx).cloned();

        if subnet_node_id.is_some() {
            // --- Insert validator for next epoch
            SubnetElectedValidator::<T>::insert(subnet_id, subnet_epoch, subnet_node_id.unwrap());
        }
    }

    fn get_last_overwatch_epoch(current_epoch: u32, submit_interval: u32) -> u32 {
        current_epoch - (current_epoch % submit_interval)
    }
}
