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
        let current_overwatch_epoch = current_block.saturating_div(overwatch_epoch_length);
        let cutoff_percentage = OverwatchCommitCutoffPercent::<T>::get();
        let block_increase_cutoff =
            Self::percent_mul(overwatch_epoch_length as u128, cutoff_percentage);
        // start_block + cutoff blocks
        let epoch_cutoff_block =
            overwatch_epoch_length * current_overwatch_epoch + block_increase_cutoff as u32;
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
        let offset_block = current_block.saturating_sub(subnet_slot);

        // Example: 1 = 150 / 100
        offset_block.saturating_div(epoch_length)
    }

    pub fn get_subnet_epoch_progression(subnet_id: u32) -> u128 {
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

        let offset_block = current_block.saturating_sub(subnet_slot);
        let blocks_into_epoch = offset_block % epoch_length;
        Self::percent_div(blocks_into_epoch as u128, epoch_length as u128)
    }

    /// Returns true if < last subnet epoch block
    pub fn can_propose_or_attest_attestation(subnet_id: u32) -> bool {
        let epoch_length = T::EpochLength::get();
        let subnet_slot = match SubnetSlot::<T>::try_get(subnet_id) {
            Ok(slot) => slot,
            Err(_) => 0,
        };
        if subnet_slot == 0 {
            return false;
        }

        let current_block = Self::get_current_block_as_u32();

        if current_block < subnet_slot {
            return false;
        }

        let offset_block = current_block.saturating_sub(subnet_slot);

        let current_subnet_epoch = offset_block.saturating_div(epoch_length);

        // Get last subnet epoch block, start of next epoch
        let last_epoch_block = subnet_slot + (current_subnet_epoch + 1) * epoch_length;

        // Check if we are at the last block
        current_block < last_epoch_block
    }

    pub fn attestor_subnet_epoch_data(
        subnet_id: u32,
        block_proposed: u32,
    ) -> Option<SubnetEpochData> {
        let epoch_length = T::EpochLength::get();
        let subnet_slot = match SubnetSlot::<T>::try_get(subnet_id) {
            Ok(slot) => slot,
            Err(_) => 0,
        };
        if subnet_slot == 0 {
            return None;
        }

        let current_block = Self::get_current_block_as_u32();

        if current_block < block_proposed {
            return None; // can't attest before proposal
        }

        // The validator's epoch offset at submission
        let proposed_offset = block_proposed.saturating_sub(subnet_slot);
        let subnet_epoch = proposed_offset.saturating_div(epoch_length);

        // Blocks from submission to current block
        let blocks_since_submission = current_block.saturating_sub(block_proposed);

        // Remaining blocks in this epoch
        let blocks_into_epoch = proposed_offset % epoch_length;
        let remaining_blocks_in_epoch = epoch_length.saturating_sub(blocks_into_epoch);

        // How far from submission to epoch end (percentage)
        // If current block > epoch end, clamp to 100%
        let progress_from_submission = if blocks_since_submission >= remaining_blocks_in_epoch {
            Self::percentage_factor_as_u128()
        } else {
            Self::percent_div(
                blocks_since_submission as u128,
                remaining_blocks_in_epoch as u128,
            )
        };

        Some(SubnetEpochData {
            subnet_epoch,
            subnet_epoch_progression: progress_from_submission,
        })
    }

    pub fn get_current_subnet_epoch_data(subnet_id: u32) -> Option<SubnetEpochData> {
        let epoch_length = T::EpochLength::get();
        let subnet_slot = match SubnetSlot::<T>::try_get(subnet_id) {
            Ok(slot) => slot,
            Err(_) => 0,
        };
        if subnet_slot == 0 {
            return None;
        }

        let current_block = Self::get_current_block_as_u32();

        if current_block < subnet_slot {
            return None;
        }

        // Example: 150 = 200-50
        let offset_block = current_block.saturating_sub(subnet_slot);

        // Example: 1 = 150 / 100
        let subnet_epoch = offset_block.saturating_div(epoch_length);

        let blocks_into_epoch = offset_block % epoch_length;
        let subnet_epoch_progression =
            Self::percent_div(blocks_into_epoch as u128, epoch_length as u128);

        Some(SubnetEpochData {
            subnet_epoch,
            subnet_epoch_progression,
        })
    }

    /// Performs preliminary subnet checks and maintenance at the start of each epoch.
    ///
    /// This function iterates over all registered subnets and enforces several rules:
    ///
    /// - Subnets in the **registration period** are allowed to exist without reputation decrease.
    /// - Subnets in the **enactment period** must meet minimum active node counts or get removed.
    /// - Subnets **out of enactment period** but not activated are removed.
    /// - Subnets in the **paused state** are penalized if they exceed allowed pause duration, potentially leading to removal.
    /// - Activated subnets are checked to ensure they meet minimum delegate stake requirements; otherwise they are removed.
    /// - Activated subnets with insufficient active nodes decrease reputation.  
    /// - Subnets exceeding the minimum reputation are removed.
    /// - If the total number of subnets exceeds the configured maximum, the subnet with the lowest delegate stake is removed.
    ///
    /// Reputations are global and can be increased or decreased by other runtime logic as well, so this function enforces removal
    /// conditions based on the current reputation regardless of its origin.
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
    pub fn do_epoch_preliminaries(weight_meter: &mut WeightMeter, block: u32, epoch: u32) {
        let db_weight = T::DbWeight::get();

        let min_reputation = MinSubnetReputation::<T>::get();
        let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
        let subnet_enactment_epochs = SubnetEnactmentEpochs::<T>::get();
        let min_subnet_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_pause_epochs = MaxSubnetPauseEpochs::<T>::get();
        let dstake_epoch_interval = DelegateStakeSubnetRemovalInterval::<T>::get();
        let prev_activation_epoch = PrevSubnetActivationEpoch::<T>::get();
        let is_removal_epoch: bool = epoch % MaxSubnetRemovalInterval::<T>::get() == 0;
        let can_remove: bool =
            epoch >= prev_activation_epoch + MinSubnetRemovalInterval::<T>::get();
        let dstake_epoch_interval_can_remove: bool = epoch % dstake_epoch_interval == 0;

        let subnets: Vec<_> = SubnetsData::<T>::iter().collect();
        let total_subnets: u32 = subnets.len() as u32;

        weight_meter.consume(db_weight.reads((10 + total_subnets).into()));

        let excess_subnets: bool = total_subnets > max_subnets;
        let mut subnet_delegate_stake: Vec<(u32, u128)> = Vec::new();

        if excess_subnets {
            subnet_delegate_stake.reserve(total_subnets as usize);
            // --- Get expected weight for `subnet_delegate_stake`
            weight_meter.consume(
                db_weight.reads(total_subnets as u64)
                    + Weight::from_parts(5_000 * total_subnets as u64, 0),
            );
        }

        // Main loop computational overhead
        weight_meter.consume(Weight::from_parts(1_000 * total_subnets as u64, 0));

        for (subnet_id, data) in &subnets {
            // --- Registration logic
            if data.state == SubnetState::Registered {
                // SubnetRegistrationEpoch
                weight_meter.consume(db_weight.reads(1));
                if let Ok(registered_epoch) = SubnetRegistrationEpoch::<T>::try_get(subnet_id) {
                    // --- Do the registration and enactment period math manually instead of using helper functions to avoid duplicate lookups
                    let max_registration_epoch =
                        registered_epoch.saturating_add(subnet_registration_epochs);
                    let max_enactment_epoch =
                        max_registration_epoch.saturating_add(subnet_enactment_epochs);

                    if epoch <= max_registration_epoch {
                        // --- Registration Period: do nothing
                        // We wait for the owner to activate the subnet to ensure the subnet is ready to begin
                        continue;
                    }

                    if epoch <= max_enactment_epoch {
                        // --- Enactment Period
                        // - Check min nodes
                        // We don't check delegate stake here because users can continue to stake in this phase
                        let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
                        weight_meter.consume(db_weight.reads(1));

                        if active_nodes < min_subnet_nodes {
                            Self::try_do_remove_subnet(
                                weight_meter,
                                *subnet_id,
                                SubnetRemovalReason::MinSubnetNodes,
                            );
                        }
                        continue;
                    }

                    // --- Out of Enactment Period: not activated → remove
                    Self::try_do_remove_subnet(
                        weight_meter,
                        *subnet_id,
                        SubnetRemovalReason::EnactmentPeriod,
                    );
                    continue;
                }
                continue;
            }

            // --- Pause logic
            if data.state == SubnetState::Paused {
                if data.start_epoch + max_pause_epochs < epoch {
                    let subnet_reputation = SubnetReputation::<T>::get(subnet_id);
                    let new_subnet_reputation = Self::get_decrease_reputation(
                        subnet_reputation,
                        MaxPauseEpochsSubnetReputationFactor::<T>::get(),
                    );
                    SubnetReputation::<T>::insert(subnet_id, new_subnet_reputation);
                    weight_meter.consume(db_weight.reads_writes(2, 1));

                    if new_subnet_reputation < min_reputation {
                        // --- Remove
                        Self::try_do_remove_subnet(
                            weight_meter,
                            *subnet_id,
                            SubnetRemovalReason::PauseExpired,
                        );
                        continue;
                    }
                }
                continue;
            }

            // Ignore if not started yet
            if data.start_epoch > epoch {
                continue;
            }

            // --- Activated subnet checks and conditionals
            let min_subnet_delegate_stake_balance =
                Self::get_min_subnet_delegate_stake_balance(*subnet_id);
            weight_meter.consume(T::WeightInfo::get_min_subnet_delegate_stake_balance());

            let subnet_delegate_stake_balance =
                TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
            weight_meter.consume(db_weight.reads(1));

            // Remove if below delegate stake requirement
            if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance
                && dstake_epoch_interval_can_remove
            {
                Self::try_do_remove_subnet(
                    weight_meter,
                    *subnet_id,
                    SubnetRemovalReason::MinSubnetDelegateStake,
                );
                continue;
            }

            // Check min nodes (we don't kick active subnet for this to give them time to recoup)
            // We decrease reputation only
            // A subnet can have n-1 min electable nodes, we'll allow them to get more nodes until
            // they read the min nodes count
            let electable_nodes = TotalSubnetElectableNodes::<T>::get(subnet_id);

            if electable_nodes < min_subnet_nodes {
                let subnet_reputation = SubnetReputation::<T>::get(subnet_id);
                let new_subnet_reputation = Self::get_decrease_reputation(
                    subnet_reputation,
                    LessThanMinNodesSubnetReputationFactor::<T>::get(),
                );
                SubnetReputation::<T>::insert(subnet_id, new_subnet_reputation);
                weight_meter.consume(db_weight.reads_writes(2, 1));
            }

            let subnet_reputation = SubnetReputation::<T>::get(subnet_id);
            // TotalSubnetElectableNodes | SubnetReputation
            weight_meter.consume(db_weight.reads(2));

            if subnet_reputation < min_reputation {
                // --- Remove
                Self::try_do_remove_subnet(
                    weight_meter,
                    *subnet_id,
                    SubnetRemovalReason::MinReputation,
                );
                continue;
            }

            // Store delegate stake for possible excess removal
            if excess_subnets && is_removal_epoch && can_remove {
                subnet_delegate_stake.push((*subnet_id, subnet_delegate_stake_balance));
            }
        }

        // --- Excess subnet removal
        // We allow max+1 subnets to exist in the economy and every `x` epochs remove one
        // based on the delegate stake balance
        if excess_subnets && !subnet_delegate_stake.is_empty() && is_removal_epoch && can_remove {
            subnet_delegate_stake.sort_by_key(|&(_, value)| value);

            // Account for sorting cost (O(n log n))
            let sort_items = subnet_delegate_stake.len() as u64;
            let sort_weight = Weight::from_parts(
                sort_items * sort_items.ilog2() as u64 * 100, // Approximate O(n log n)
                0,
            );
            weight_meter.consume(sort_weight);

            let subnet_id = subnet_delegate_stake[0].0.clone();
            Self::try_do_remove_subnet(weight_meter, subnet_id, SubnetRemovalReason::MaxSubnets);
        }
    }

    pub fn elect_validator(subnet_id: u32, subnet_epoch: u32, block: u32) {
        // Redundant
        // If validator already chosen, then return
        if SubnetElectedValidator::<T>::contains_key(subnet_id, subnet_epoch) {
            return;
        }

        // Check for emergency validators
        let slot_list = if let Some(emergency_validator_data) =
            EmergencySubnetNodeElectionData::<T>::get(subnet_id)
        {
            if emergency_validator_data.total_epochs
                > emergency_validator_data.target_emergency_validators_epochs
                || subnet_epoch > emergency_validator_data.max_emergency_validators_epoch
            {
                // Temporary emergency validators is complete, remove and return default election slots
                EmergencySubnetNodeElectionData::<T>::remove(subnet_id);
                SubnetNodeElectionSlots::<T>::get(subnet_id)
            } else {
                emergency_validator_data.subnet_node_ids
            }
        } else {
            SubnetNodeElectionSlots::<T>::get(subnet_id)
        };

        if slot_list.is_empty() {
            return;
        }

        let idx = Self::get_random_number(block, slot_list.len() as u32) as usize;

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
