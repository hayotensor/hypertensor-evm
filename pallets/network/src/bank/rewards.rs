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

impl<T: Config> Pallet<T> {
    pub fn distribute_rewards(
        weight_meter: &mut WeightMeter,
        subnet_id: u32,
        block: u32,
        current_epoch: u32,
        current_subnet_epoch: u32,
        consensus_submission_data: ConsensusSubmissionData<T::AccountId>,
        rewards_data: RewardsData,
        min_attestation_percentage: u128,
        coldkey_reputation_increase_factor: u128,
        coldkey_reputation_decrease_factor: u128,
        super_majority_threshold: u128,
    ) {
        let db_weight = T::DbWeight::get();

        let percentage_factor = Self::percentage_factor_as_u128();
        let min_validator_reputation = MinSubnetNodeReputation::<T>::get(subnet_id);
        let subnet_reputation = SubnetReputation::<T>::get(subnet_id);
        // MinSubnetNodeReputation | SubnetReputation
        weight_meter.consume(db_weight.reads(2));

        // We run this here because any epoch where a validator submits data, whether in consensus
        // or not, we increment the forks `total_epochs`
        let forked_subnet_node_ids: Option<BTreeSet<u32>> =
            EmergencySubnetNodeElectionData::<T>::mutate_exists(subnet_id, |maybe_data| {
                if let Some(data) = maybe_data {
                    weight_meter.consume(db_weight.writes(1));

                    // Increment `total_epochs`
                    data.total_epochs = data.total_epochs.saturating_add(1);

                    Some(data.subnet_node_ids.iter().cloned().collect())
                } else {
                    None
                }
            });
        // EmergencySubnetNodeElectionData
        weight_meter.consume(db_weight.reads(1));

        // --- If under minimum attestation ratio, penalize validator, skip rewards
        if consensus_submission_data.attestation_ratio < min_attestation_percentage {
            // --- Slash validator
            // Slashes stake balance
            // Decreases reputation
            let slash_validator_weight = Self::slash_validator(
                subnet_id,
                consensus_submission_data.validator_subnet_node_id,
                consensus_submission_data.attestation_ratio,
                min_attestation_percentage,
                coldkey_reputation_decrease_factor,
                min_validator_reputation,
                current_epoch,
            );
            weight_meter.consume(T::WeightInfo::slash_validator());

            let new_subnet_reputation = Self::get_decrease_reputation(
                subnet_reputation,
                NotInConsensusSubnetReputationFactor::<T>::get(),
            );
            SubnetReputation::<T>::insert(subnet_id, new_subnet_reputation);
            // NotInConsensusSubnetReputationFactor, SubnetReputation
            weight_meter.consume(db_weight.reads_writes(1, 1));
            Self::deposit_event(Event::SubnetReputationUpdate {
                subnet_id,
                prev_reputation: subnet_reputation,
                new_reputation: new_subnet_reputation,
            });

            let non_consensus_attestor_factor = Self::percent_mul(
                NonConsensusAttestorDecreaseReputationFactor::<T>::get(subnet_id),
                percentage_factor.saturating_sub(Self::percent_div(
                    consensus_submission_data.attestation_ratio,
                    min_attestation_percentage,
                )),
            );
            // NonConsensusAttestorDecreaseReputationFactor
            weight_meter.consume(db_weight.reads(1));

            // --- Decrease reputation of attestors
            for (subnet_node_id, attest_data) in consensus_submission_data.attests {
                let new_reputation = Self::decrease_node_reputation(
                    subnet_id,
                    subnet_node_id,
                    non_consensus_attestor_factor,
                );
                // `decrease_node_reputation`: SubnetNodeReputation (r/w)
                weight_meter.consume(db_weight.reads_writes(1, 1));
                if new_reputation < min_validator_reputation {
                    // Remove node if below minimum threshold
                    if weight_meter.can_consume(T::WeightInfo::perform_remove_subnet_node(0u32)) {
                        Self::perform_remove_subnet_node(subnet_id, subnet_node_id);
                        weight_meter.consume(T::WeightInfo::perform_remove_subnet_node(0u32));
                    }
                    continue;
                }
            }
            return;
        } else if let Some(hotkey) = SubnetNodeIdHotkey::<T>::get(
            subnet_id,
            consensus_submission_data.validator_subnet_node_id,
        ) {
            //
            // In consensus
            //

            // Increase validators stake

            // SubnetNodeIdHotkey
            weight_meter.consume(db_weight.reads(1));

            // --- Increase validator reward
            let validator_reward = Self::get_validator_reward(
                consensus_submission_data.attestation_ratio,
                consensus_submission_data.validator_reward_factor,
            );
            // Add get_validator_reward (At least 1 read, up to 2)
            // MinAttestationPercentage | BaseValidatorReward
            weight_meter.consume(db_weight.reads(2));

            if let Ok(coldkey) = HotkeyOwner::<T>::try_get(&hotkey) {
                Self::increase_coldkey_reputation(
                    coldkey,
                    consensus_submission_data.attestation_ratio,
                    min_attestation_percentage,
                    coldkey_reputation_increase_factor,
                    current_epoch,
                );
                weight_meter.consume(T::WeightInfo::increase_coldkey_reputation());
            }

            // HotkeyOwner
            weight_meter.consume(db_weight.reads(1));

            Self::increase_account_stake(&hotkey, subnet_id, validator_reward);
        } else {
            // Validator left subnet

            // We read SubnetNodeIdHotkey
            weight_meter.consume(db_weight.reads(1));
        }

        //
        // --- We are now in consensus
        //

        let idle_epochs = IdleClassificationEpochs::<T>::get(subnet_id);
        let included_epochs = IncludedClassificationEpochs::<T>::get(subnet_id);
        let weight_threshold = SubnetNodeMinWeightDecreaseReputationThreshold::<T>::get(subnet_id);
        let absent_factor = AbsentDecreaseReputationFactor::<T>::get(subnet_id);
        let included_factor = IncludedIncreaseReputationFactor::<T>::get(subnet_id);
        let min_weight_factor = BelowMinWeightDecreaseReputationFactor::<T>::get(subnet_id);
        let non_attestor_factor = NonAttestorDecreaseReputationFactor::<T>::get(subnet_id);
        weight_meter.consume(db_weight.reads(7));

        // Super majority, update queue to prioritize node ID that subnet form a consensus to cut the line
        // and or update queue to remove a node ID the subnet forms a consensus to be removed (if passed immunity period)
        if consensus_submission_data.attestation_ratio >= super_majority_threshold {
            let mut queue = SubnetNodeQueue::<T>::get(subnet_id);

            // Handle prioritize node - move to front
            if let Some(prioritize_queue_node_id) =
                consensus_submission_data.prioritize_queue_node_id
            {
                weight_meter.consume(db_weight.reads(1));

                if let Some(index) = queue
                    .iter()
                    .position(|node| node.id == prioritize_queue_node_id)
                {
                    let node = queue.remove(index); // Remove from current position
                    queue.insert(0, node); // Insert at front (index 0)

                    // Add computational weight for vector operations
                    weight_meter.consume(Weight::from_parts(
                        queue.len() as u64 * 100, // Linear cost based on queue size
                        0,
                    ));

                    SubnetNodeQueue::<T>::insert(subnet_id, &queue);
                    weight_meter.consume(db_weight.writes(1));

                    Self::deposit_event(Event::QueuedNodePrioritized {
                        subnet_id,
                        subnet_node_id: prioritize_queue_node_id,
                    });
                }
            }

            // Handle remove node - remove from queue entirely
            // These are not yet activated nodes so this does not impact the emissions distribution
            if let Some(remove_queue_node_id) = consensus_submission_data.remove_queue_node_id {
                // `perform_remove_subnet_node` handles SubnetNodeQueue retain
                if weight_meter
                    .can_consume(T::WeightInfo::perform_remove_subnet_node(queue.len() as u32))
                {
                    Self::perform_remove_subnet_node(subnet_id, remove_queue_node_id);
                    weight_meter
                        .consume(T::WeightInfo::perform_remove_subnet_node(queue.len() as u32));

                    Self::deposit_event(Event::QueuedNodeRemoved {
                        subnet_id,
                        subnet_node_id: remove_queue_node_id,
                    });
                }
            }
        }

        // Increase reputation because subnet consensus is successful
        // Only increase if subnet has >= min subnet nodes

        // MinSubnetNodes
        weight_meter.consume(db_weight.reads(1));

        if subnet_reputation != percentage_factor
            && consensus_submission_data.data_length >= MinSubnetNodes::<T>::get()
        {
            let new_subnet_reputation = Self::get_increase_reputation(
                subnet_reputation,
                InConsensusSubnetReputationFactor::<T>::get(),
            );
            SubnetReputation::<T>::insert(subnet_id, new_subnet_reputation);
            weight_meter.consume(db_weight.reads_writes(2, 1));
        }

        // --- Reward owner
        if let Ok(owner) = SubnetOwner::<T>::try_get(subnet_id) {
            if let Some(balance) = Self::u128_to_balance(rewards_data.subnet_owner_reward) {
                Self::add_balance_to_coldkey_account(&owner, balance);
                weight_meter.consume(T::WeightInfo::add_balance_to_coldkey_account());
            }
        }

        // SubnetOwner
        weight_meter.consume(db_weight.reads(1));

        // Loop iteration overhead
        weight_meter.consume(Weight::from_parts(
            1_000 * consensus_submission_data.subnet_nodes.len() as u64,
            0,
        ));

        let mut node_rewards: Vec<(u32, u128)> = Vec::new();
        let mut node_delegate_stake_rewards: Vec<(u32, u128)> = Vec::new();

        // Iterate each node, emit rewards, graduate, or penalize
        for subnet_node in &consensus_submission_data.subnet_nodes {
            let mut reputation = SubnetNodeReputation::<T>::get(subnet_id, subnet_node.id);

            // SubnetNodeReputation
            weight_meter.consume(db_weight.reads(1));

            if reputation < min_validator_reputation {
                // Remove node if they haven't already been removed
                // Note: We used 0u32 because the node is not in the queue
                if weight_meter.can_consume(T::WeightInfo::perform_remove_subnet_node(0u32)) {
                    Self::perform_remove_subnet_node(subnet_id, subnet_node.id);
                    weight_meter.consume(T::WeightInfo::perform_remove_subnet_node(0u32));
                }

                continue;
            }

            // If node is Idle class and subnet is not temporarily forked, upgrade to Included class
            if subnet_node.classification.node_class == SubnetNodeClass::Idle
                && forked_subnet_node_ids.is_none()
            {
                SubnetNodeIdleConsecutiveEpochs::<T>::mutate(
                    subnet_id,
                    subnet_node.id,
                    |n: &mut u32| *n += 1,
                );

                let node_idle_epochs =
                    SubnetNodeIdleConsecutiveEpochs::<T>::get(subnet_id, subnet_node.id);
                weight_meter.consume(db_weight.reads_writes(1, 1));

                // Idle classified nodes can't be included in consensus data and can't have a used reputation
                // so we check the class immediately.
                // --- Upgrade to Included if past the queue epochs
                if node_idle_epochs >= idle_epochs {
                    Self::graduate_class(subnet_id, subnet_node.id, current_subnet_epoch);
                    weight_meter.consume(T::WeightInfo::graduate_class());
                }
                continue;
            }

            //
            // All nodes are at least SubnetNodeClass::Included from here
            //

            let subnet_node_data_find = consensus_submission_data
                .data
                .iter()
                .find(|data| data.subnet_node_id == subnet_node.id);

            // Handle case where node is found in consensus data
            let subnet_node_data = if let Some(data) = subnet_node_data_find {
                // --- Is in consensus data, increase reputation if not at max
                if reputation != percentage_factor {
                    // If the validator submits themselves in the data and passes consensus, this also
                    // increases the validators reputation
                    reputation = Self::increase_and_return_node_reputation(
                        subnet_id,
                        subnet_node.id,
                        reputation,
                        included_factor,
                    );

                    // `increase_and_return_node_reputation`: SubnetNodeReputation (w)
                    weight_meter.consume(db_weight.writes(1));
                }
                data
            } else {
                // Not included in consensus, decrease reputation
                reputation = Self::decrease_and_return_node_reputation(
                    subnet_id,
                    subnet_node.id,
                    reputation,
                    absent_factor,
                );
                // `decrease_and_return_node_reputation`: SubnetNodeReputation (w)
                weight_meter.consume(db_weight.writes(1));

                // Break count of consecutive epochs of being included in in-consensus data
                if subnet_node.classification.node_class == SubnetNodeClass::Included {
                    SubnetNodeConsecutiveIncludedEpochs::<T>::insert(subnet_id, subnet_node.id, 0);
                    // SubnetNodeConsecutiveIncludedEpochs
                    weight_meter.consume(db_weight.writes(1));
                }
                continue;
            };

            //
            // --- Consensus formed on node
            //

            let node_score = subnet_node_data.score;

            // --- Calculate node weight percentage of peer versus the weighted sum
            let node_weight: u128 =
                Self::percent_div(node_score, consensus_submission_data.weight_sum);

            // Decrease reputation if under subnets weight threshold
            // We don't automatically decrease reputation if a node is at ZERO
            // This is an optional feature for subnets
            if node_weight < weight_threshold {
                reputation = Self::decrease_and_return_node_reputation(
                    subnet_id,
                    subnet_node.id,
                    reputation,
                    min_weight_factor,
                );
                // `decrease_and_return_node_reputation`: SubnetNodeReputation (w)
                weight_meter.consume(db_weight.writes(1));
            }

            // If node is Included class and subnet is not temporarily forked, upgrade to Validator class
            if subnet_node.classification.node_class == SubnetNodeClass::Included
                && forked_subnet_node_ids.is_none()
            {
                SubnetNodeConsecutiveIncludedEpochs::<T>::mutate(
                    subnet_id,
                    subnet_node.id,
                    |n: &mut u32| *n += 1,
                );

                // SubnetNodeConsecutiveIncludedEpochs
                weight_meter.consume(db_weight.reads_writes(1, 1));

                let consecutive_included_epochs =
                    SubnetNodeConsecutiveIncludedEpochs::<T>::get(subnet_id, subnet_node.id);

                // SubnetNodeConsecutiveIncludedEpochs
                weight_meter.consume(db_weight.reads(1));

                // --- Upgrade to Validator if at percentage_factor reputation and included in weights
                if reputation >= percentage_factor && consecutive_included_epochs >= included_epochs
                {
                    if Self::graduate_class(subnet_id, subnet_node.id, current_subnet_epoch) {
                        // --- Insert into election slot
                        Self::insert_node_into_election_slot(subnet_id, subnet_node.id);
                        weight_meter.consume(T::WeightInfo::insert_node_into_election_slot());

                        // reset
                        SubnetNodeConsecutiveIncludedEpochs::<T>::remove(subnet_id, subnet_node.id);
                        weight_meter.consume(db_weight.writes(1));
                    }
                }

                // SubnetNodeClass::Included does not get rewards yet, they must pass the gauntlet
                continue;
            }

            //
            // All nodes are at least SubnetNodeClass::Validator from here and in consensus data
            //

            // Get the nodes reward factor
            let reward_factor = if let Some(forked_node_ids) = &forked_subnet_node_ids {
                if forked_node_ids.get(&subnet_node.id).is_some() {
                    // If one of the temporary fork nodes
                    match consensus_submission_data.attests.get(&subnet_node.id) {
                        Some(data) => data.reward_factor,
                        None => {
                            // If node didn't attest in super majority, decrease reputation
                            // We can likely assume the validator is offline in the current epoch and
                            // failed to attest. The `non_attestor_factor` is suggested to the be lowest
                            // decreasing factor of all node reputation factors.
                            if consensus_submission_data.attestation_ratio
                                >= super_majority_threshold
                            {
                                reputation = Self::decrease_and_return_node_reputation(
                                    subnet_id,
                                    subnet_node.id,
                                    reputation,
                                    non_attestor_factor,
                                );
                                // `decrease_and_return_node_reputation`: SubnetNodeReputation (w)
                                weight_meter.consume(db_weight.writes(1));
                            }
                            percentage_factor
                        }
                    }
                } else {
                    percentage_factor
                }
            } else if let Some(data) = consensus_submission_data.attests.get(&subnet_node.id) {
                // Subnet is not forked and node attested
                data.reward_factor
            } else {
                // Node not attested, decrease reputation, return 1.0 reward factor
                if consensus_submission_data.attestation_ratio >= super_majority_threshold {
                    reputation = Self::decrease_and_return_node_reputation(
                        subnet_id,
                        subnet_node.id,
                        reputation,
                        non_attestor_factor,
                    );
                    // `decrease_and_return_node_reputation`: SubnetNodeReputation (w)
                    weight_meter.consume(db_weight.writes(1));
                }

                percentage_factor
            };

            if reward_factor == 0 {
                continue;
            }

            if reputation < min_validator_reputation {
                // Remove node if they haven't already
                if weight_meter.can_consume(T::WeightInfo::perform_remove_subnet_node(0u32)) {
                    Self::perform_remove_subnet_node(subnet_id, subnet_node.id);
                    weight_meter.consume(T::WeightInfo::perform_remove_subnet_node(0u32));
                }

                continue;
            }

            if node_weight == 0 {
                continue;
            }

            // --- Calculate node_score percentage of total subnet generated epoch rewards
            let mut account_reward: u128 =
                Self::percent_mul(node_weight, rewards_data.subnet_node_rewards);

            account_reward = Self::percent_mul(account_reward, reward_factor);

            // --- Skip if no rewards to give
            // Unlikely to happen
            if account_reward == 0 {
                continue;
            }

            if subnet_node.delegate_reward_rate != 0 {
                // --- Ensure users are staked to subnet node
                let total_node_delegated_stake_shares =
                    TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node.id);
                // TotalNodeDelegateStakeShares
                weight_meter.consume(db_weight.reads(1));
                if total_node_delegated_stake_shares != 0 {
                    let node_delegate_reward =
                        Self::percent_mul(account_reward, subnet_node.delegate_reward_rate);
                    account_reward = account_reward.saturating_sub(node_delegate_reward);
                    Self::do_increase_node_delegate_stake(
                        subnet_id,
                        subnet_node.id,
                        node_delegate_reward,
                    );
                    // reads:
                    // TotalNodeDelegateStakeBalance | TotalNodeDelegateStakeShares
                    //
                    // writes:
                    // TotalNodeDelegateStakeShares | TotalNodeDelegateStakeBalance | TotalNodeDelegateStake
                    weight_meter.consume(db_weight.reads_writes(5, 3));

                    node_delegate_stake_rewards.push((subnet_node.id, node_delegate_reward));
                }
            }

            // --- Increase account stake and emit event
            Self::increase_account_stake(&subnet_node.hotkey, subnet_id, account_reward);
            // AccountSubnetStake | TotalSubnetStake | TotalStake
            weight_meter.consume(db_weight.reads_writes(3, 3));

            node_rewards.push((subnet_node.id, account_reward));
        }

        // --- Increase the delegate stake pool balance
        if rewards_data.delegate_stake_rewards != 0 {
            Self::do_increase_delegate_stake(subnet_id, rewards_data.delegate_stake_rewards);
            // reads::
            // TotalSubnetDelegateStakeShares | TotalSubnetDelegateStakeBalance | TotalDelegateStake
            //
            // writes::
            // TotalSubnetDelegateStakeBalance | | TotalSubnetDelegateStakeShares|
            // TotalSubnetDelegateStakeShares| TotalSubnetDelegateStakeBalance| TotalDelegateStake
            weight_meter.consume(db_weight.reads_writes(3, 5));
        }

        Self::deposit_event(Event::SubnetRewards {
            subnet_id,
            node_rewards,
            delegate_stake_reward: rewards_data.delegate_stake_rewards,
            node_delegate_stake_rewards,
        });
    }

    pub fn rewards_fork() {}
}
