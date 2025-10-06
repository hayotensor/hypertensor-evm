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
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use frame_support::pallet_prelude::Pays;
use frame_support::pallet_prelude::Weight;

impl<T: Config> Pallet<T> {
    /// Proposes attestation and submits consensus data for a subnet epoch.
    ///
    /// This function allows an elected validator to submit consensus data for their subnet,
    /// including peer scores, queue management decisions, and optional attestation data.
    ///
    /// The validator automatically attests to their own submission.
    ///
    /// # Parameters
    ///
    /// * `subnet_id` - The ID of the subnet for which consensus data is being submitted.
    /// * `hotkey` - The hotkey of the elected validator submitting the consensus data.
    /// * `data` - A vector of consensus data containing scores for each peer in the subnet.
    ///   Duplicates (based on `subnet_node_id`) are automatically removed, and only peers
    ///   with `Included` classification are retained.
    /// * `prioritize_queue_node_id` - Optional node ID from the registration queue to move
    ///   to the front of the queue. The node must exist in the queue or this parameter is ignored.
    /// * `remove_queue_node_id` - Optional node ID from the registration queue to remove.
    ///   The node must exist in the queue and have passed the immunity period, or this
    ///   parameter is ignored.
    /// * `args` - Optional arbitrary arguments for subnet-specific use. This data is not
    ///   used in any onchain logic and is purely for subnet validator coordination.
    ///   This data can be useful within a subnet.
    /// * `attest_data` - Optional arbitrary attestation data. This data is not used in any
    ///   onchain logic but is included as part of the validator's automatic attestation
    ///   to their own consensus submission.
    ///   This data can be useful within a subnet.
    ///
    /// # Behavior
    ///
    /// The function performs the following steps:
    /// 1. Determines the current subnet epoch
    /// 2. Verifies the caller is the elected validator for this epoch
    /// 3. Ensures consensus has not already been submitted for this epoch
    /// 4. Qualifies the consensus data by:
    ///    - Removing duplicates based on peer_id
    ///    - Filtering out non-Included peers
    ///    - Validating scores don't overflow when summed
    /// 5. Validates queue operations (prioritize/remove) if specified
    /// 6. Stores the consensus submission with the validator's auto-attestation
    ///
    /// # Errors
    ///
    /// * `NoElectedValidator` - No validator is elected for the current subnet epoch
    /// * `InvalidValidator` - The caller's hotkey doesn't match the elected validator
    /// * `SubnetRewardsAlreadySubmitted` - Consensus has already been submitted for this epoch
    /// * `ScoreOverflow` - The sum of all scores would overflow u128
    ///
    /// # Returns
    ///
    /// Returns `Ok(Pays::No.into())` on success, indicating the transaction fee is waived.
    pub fn do_propose_attestation(
        subnet_id: u32,
        hotkey: T::AccountId,
        mut data: Vec<SubnetNodeConsensusData>,
        mut prioritize_queue_node_id: Option<u32>,
        mut remove_queue_node_id: Option<u32>,
        args: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
        attest_data: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
    ) -> DispatchResultWithPostInfo {
        // The validator is elected for the next blockchain epoch where rewards will be distributed.
        // Each subnet epoch overlaps with the blockchains epochs, and can submit consensus data for epoch
        // 2 on epoch 1 (if after slot) or 2 (if before slot).
        // If a subnet is on slot 3 of 5 slots, we make sure it can submit on the current blockchains epoch.
        let subnet_epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);

        // --- Ensure current subnet validator by its hotkey
        let validator_id = SubnetElectedValidator::<T>::get(subnet_id, subnet_epoch)
            .ok_or(Error::<T>::NoElectedValidator)?;

        // --- If hotkey is hotkey, ensure it matches validator, otherwise if coldkey -> get hotkey
        // If the epoch is 0, this will break
        ensure!(
            SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id) == Some(hotkey.clone()),
            Error::<T>::InvalidValidator
        );

        // - Note: we don't check stake balance here

        // --- Ensure not submitted already
        ensure!(
            !SubnetConsensusSubmission::<T>::contains_key(subnet_id, subnet_epoch),
            Error::<T>::SubnetRewardsAlreadySubmitted
        );

        //
        // --- Qualify the data
        //

        // Remove duplicates based on peer_id
        data.dedup_by(|a, b| a.subnet_node_id == b.subnet_node_id);

        // Remove queue classified entries
        // Each peer must have an inclusion classification at minimum
        data.retain(
            |x| match SubnetNodesData::<T>::try_get(subnet_id, x.subnet_node_id) {
                Ok(subnet_node) => {
                    subnet_node.has_classification(&SubnetNodeClass::Included, subnet_epoch)
                }
                Err(()) => false,
            },
        );

        // --- Ensure overflow sum fails
        data.iter().try_fold(0u128, |acc, node| {
            acc.checked_add(node.score).ok_or(Error::<T>::ScoreOverflow)
        })?;

        let block: u32 = Self::get_current_block_as_u32();

        // --- Validator auto-attests the epoch
        // let attests: BTreeMap<u32, (u32, Option<BoundedVec<u8, DefaultValidatorArgsLimit>>)> =
        //     BTreeMap::from([(validator_id, (block, attest_data))]);
        let attests: BTreeMap<u32, AttestEntry> = BTreeMap::from([(
            validator_id,
            AttestEntry {
                block: block,
                data: attest_data,
            },
        )]);

        // --- Get all (activated) Idle + consensus-eligible nodes
        // We get this here instead of in the rewards distribution to handle block weight more efficiently
        let subnet_nodes: Vec<SubnetNode<T::AccountId>> = Self::get_active_classified_subnet_nodes(
            subnet_id,
            &SubnetNodeClass::Idle,
            subnet_epoch,
        );
        let subnet_nodes_count = subnet_nodes.len();

        if prioritize_queue_node_id.is_some() || remove_queue_node_id.is_some() {
            let queue = SubnetNodeQueue::<T>::get(subnet_id);
            let immunity_epochs = QueueImmunityEpochs::<T>::get(subnet_id); // Move outside loop

            let mut prioritize_exists = prioritize_queue_node_id.is_none();
            let mut remove_allowed = remove_queue_node_id.is_none(); // Rename for clarity

            // Single pass through the queue to check both nodes
            for node in &queue {
                if let Some(node_id) = prioritize_queue_node_id {
                    if node.id == node_id {
                        prioritize_exists = true;
                    }
                }

                if let Some(node_id) = remove_queue_node_id {
                    if node.id == node_id {
                        // Node exists AND has passed immunity period
                        remove_allowed =
                            node.classification.start_epoch + immunity_epochs <= subnet_epoch;
                    }
                }

                if prioritize_exists && (remove_queue_node_id.is_none() || remove_allowed) {
                    break;
                }
            }

            // Update parameters based on checks
            if !prioritize_exists {
                prioritize_queue_node_id = None;
            }

            if !remove_allowed {
                remove_queue_node_id = None;
            }
        }

        let consensus_data: ConsensusData<T::AccountId> = ConsensusData {
            validator_id: validator_id,
            attests: attests,
            subnet_nodes: subnet_nodes,
            prioritize_queue_node_id: prioritize_queue_node_id,
            remove_queue_node_id: remove_queue_node_id,
            data: data,
            args: args,
        };

        SubnetConsensusSubmission::<T>::insert(subnet_id, subnet_epoch, consensus_data);

        Self::deposit_event(Event::ValidatorSubmission {
            subnet_id: subnet_id,
            account_id: hotkey,
            epoch: subnet_epoch,
        });

        Ok(Pays::No.into())
    }

    /// Attest validator subnet rewards data
    // Nodes must attest data to receive rewards
    pub fn do_attest(
        subnet_id: u32,
        hotkey: T::AccountId,
        data: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
    ) -> DispatchResultWithPostInfo {
        let subnet_epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);

        // --- Ensure subnet node exists under hotkey
        let subnet_node_id = match HotkeySubnetNodeId::<T>::try_get(subnet_id, &hotkey) {
            Ok(subnet_node_id) => subnet_node_id,
            Err(()) => return Err(Error::<T>::InvalidHotkeySubnetNodeId.into()),
        };

        // --- Ensure node classified to attest
        match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
            Ok(subnet_node) => {
                subnet_node.has_classification(&SubnetNodeClass::Validator, subnet_epoch)
            }
            Err(()) => return Err(Error::<T>::InvalidSubnetNodeId.into()),
        };

        // - Note: we don't check stake balance here

        let block: u32 = Self::get_current_block_as_u32();

        SubnetConsensusSubmission::<T>::try_mutate_exists(
            subnet_id,
            subnet_epoch,
            |maybe_params| -> DispatchResult {
                let params = maybe_params
                    .as_mut()
                    .ok_or(Error::<T>::InvalidSubnetConsensusSubmission)?;

                // Reduntantly check they are in the list
                // See `do_propose_attestation`
                // We check they are SubnetNodeClass::Validator above so we only
                // check they are in the list here
                let subnet_nodes = &mut params.subnet_nodes;
                ensure!(
                    subnet_nodes.iter().any(|node| node.id == subnet_node_id),
                    Error::<T>::InvalidSubnetNodeId
                );

                let mut attests = &mut params.attests;

                ensure!(
                    // attests.insert(subnet_node_id, (block, data)) == None,
                    attests.insert(subnet_node_id, AttestEntry { block, data }) == None,
                    Error::<T>::AlreadyAttested
                );

                params.attests = attests.clone();
                Ok(())
            },
        )?;

        Self::deposit_event(Event::Attestation {
            subnet_id: subnet_id,
            subnet_node_id: subnet_node_id,
            epoch: subnet_epoch,
        });

        Ok(Pays::No.into())
    }

    /// Return the validators reward that submitted data on the previous epoch
    // The attestation percentage must be greater than the MinAttestationPercentage
    pub fn get_validator_reward(attestation_percentage: u128) -> u128 {
        if MinAttestationPercentage::<T>::get() > attestation_percentage {
            return 0;
        }
        Self::percent_mul(BaseValidatorReward::<T>::get(), attestation_percentage)
    }

    /// Slash subnet validator node
    ///
    /// # Arguments
    ///
    /// * `subnet_id` - Subnet ID
    /// * `subnet_node_id` - Subnet node ID
    /// * `attestation_percentage` - The attestation ratio of the validator nodes consensus
    /// * `min_attestation_percentage` - Blockchains minimum attestation percentage (66%)
    /// * `reputation_decrease_factor`: `ReputationDecreaseFactor`
    /// * `epoch`: The blockchains general epoch
    pub fn slash_validator(
        subnet_id: u32,
        subnet_node_id: u32,
        attestation_percentage: u128,
        min_attestation_percentage: u128,
        reputation_decrease_factor: u128,
        epoch: u32,
    ) -> Weight {
        let mut weight = Weight::zero();
        let db_weight = T::DbWeight::get();

        // Redundant
        if attestation_percentage >= min_attestation_percentage {
            return weight;
        }

        // We never ensure balance is above 0 because any hotkey chosen must have the target stake
        // balance at a minimum
        //
        // Redundantly use try_get (elected validators can't exit)
        let hotkey = match SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id) {
            Ok(hotkey) => hotkey,
            // If they exited, ignore slash and return
            Err(()) => return weight.saturating_add(db_weight.reads(1)),
        };

        weight = weight.saturating_add(db_weight.reads(1));

        match HotkeyOwner::<T>::try_get(&hotkey) {
            Ok(coldkey) => {
                Self::decrease_coldkey_reputation(
                    coldkey,
                    attestation_percentage,
                    min_attestation_percentage,
                    reputation_decrease_factor,
                    epoch,
                );
            }
            Err(()) => (),
        };

        // --- Get stake balance. This is safe, uses Default value
        // This could be greater than the target stake balance
        let account_subnet_stake: u128 = AccountSubnetStake::<T>::get(&hotkey, subnet_id);

        // --- Get slash amount up to max slash
        // --- Base slash amount
        // stake balance * BaseSlashPercentage
        let base_slash: u128 =
            Self::percent_mul(account_subnet_stake, BaseSlashPercentage::<T>::get());

        // --- Get percent difference between attestation ratio and min attestation ratio
        // 1.0 - attestation ratio / min attestation ratio
        let attestation_delta = Self::percentage_factor_as_u128().saturating_sub(
            Self::percent_div(attestation_percentage, min_attestation_percentage),
        );

        // --- Update slash amount based on delta
        // base_slash * attestation_delta
        let mut slash_amount = Self::percent_mul(base_slash, attestation_delta);

        // --- Update slash amount up to max slash
        let max_slash: u128 = MaxSlashAmount::<T>::get();
        weight = weight.saturating_add(db_weight.reads(4));

        if slash_amount > max_slash {
            slash_amount = max_slash
        }

        if slash_amount > 0 {
            // --- Decrease account stake
            Self::decrease_account_stake(&hotkey, subnet_id, slash_amount);

            // AccountSubnetStake | TotalSubnetStake | TotalStake
            weight = weight.saturating_add(db_weight.writes(3));
            weight = weight.saturating_add(db_weight.reads(3));
        }

        // --- Increase validator penalty count
        let penalties = SubnetNodePenalties::<T>::get(subnet_id, subnet_node_id);
        weight = weight.saturating_add(db_weight.reads(1));
        SubnetNodePenalties::<T>::insert(subnet_id, subnet_node_id, penalties + 1);
        weight = weight.saturating_add(db_weight.writes(1));

        // --- Ensure maximum sequential removal consensus threshold is reached
        if penalties + 1 > MaxSubnetNodePenalties::<T>::get(subnet_id) {
            // --- Increase account penalty count
            Self::perform_remove_subnet_node(subnet_id, subnet_node_id);
            // weight = weight.saturating_add(T::WeightInfo::perform_remove_subnet_node());
        }

        Self::deposit_event(Event::Slashing {
            subnet_id: subnet_id,
            account_id: hotkey,
            amount: slash_amount,
        });

        weight
    }
}
