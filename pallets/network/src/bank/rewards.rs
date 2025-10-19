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
        reputation_increase_factor: u128,
        reputation_decrease_factor: u128,
        super_majority_threshold: u128,
    ) {
        let db_weight = T::DbWeight::get();

        let idle_epochs = IdleClassificationEpochs::<T>::get(subnet_id);
        let included_epochs = IncludedClassificationEpochs::<T>::get(subnet_id);
        let max_subnet_node_penalties = MaxSubnetNodePenalties::<T>::get(subnet_id);
        let score_threshold = SubnetNodeScorePenaltyThreshold::<T>::get(subnet_id);
        weight_meter.consume(db_weight.reads(4));

        // --- If under minimum attestation ratio, penalize validator, skip rewards
        if consensus_submission_data.attestation_ratio < min_attestation_percentage {
            // --- Slash validator
            // Slashes stake balance
            // Decreases reputation
            // Increases penalties
            // Possibly removes them if above maximum penalties
            let slash_validator_weight = Self::slash_validator(
                subnet_id,
                consensus_submission_data.validator_subnet_node_id,
                consensus_submission_data.attestation_ratio,
                min_attestation_percentage,
                reputation_decrease_factor,
                current_epoch,
            );
            weight_meter.consume(T::WeightInfo::slash_validator());

            SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
            // SubnetNodePenalties
            weight_meter.consume(db_weight.reads(1) + db_weight.writes(1));
            return;
        }

        //
        // --- We are now in consensus
        //

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

        // Subtract penalties because subnet consensus is successful
        // Only subtract if subnet has >= min subnet nodes
        let current_penalties_count = SubnetPenaltyCount::<T>::get(subnet_id);
        weight_meter.consume(db_weight.reads(1));
        if current_penalties_count > 0
            && consensus_submission_data.data_length >= MinSubnetNodes::<T>::get()
        {
            SubnetPenaltyCount::<T>::insert(subnet_id, current_penalties_count.saturating_sub(1));
            weight_meter.consume(db_weight.writes(1));
        }

        // --- Reward owner
        match SubnetOwner::<T>::try_get(subnet_id) {
            Ok(coldkey) => {
                let subnet_owner_reward_as_currency =
                    Self::u128_to_balance(rewards_data.subnet_owner_reward);
                if subnet_owner_reward_as_currency.is_some() {
                    Self::add_balance_to_coldkey_account(
                        &coldkey,
                        subnet_owner_reward_as_currency.unwrap(),
                    );
                    weight_meter.consume(T::WeightInfo::add_balance_to_coldkey_account());
                }
            }
            Err(()) => (),
        };
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
            let penalties = SubnetNodePenalties::<T>::get(subnet_id, subnet_node.id);
            // SubnetNodePenalties
            weight_meter.consume(db_weight.reads(1));

            // locally tracking of penalties, avoid db reads
            let mut _penalties = penalties;

            if penalties > max_subnet_node_penalties {
                // Remove node if they haven't already been removed
                // Note: We used 0u32 because the node is not in the queue
                if weight_meter.can_consume(T::WeightInfo::perform_remove_subnet_node(0u32)) {
                    Self::perform_remove_subnet_node(subnet_id, subnet_node.id);
                    weight_meter.consume(T::WeightInfo::perform_remove_subnet_node(0u32));
                }

                continue;
            }

            if subnet_node.classification.node_class == SubnetNodeClass::Idle {
                // Idle classified nodes can't be included in consensus data and can't have penalties
                // so we check the class immediately.
                // --- Upgrade to Included if past the queue epochs
                if subnet_node.classification.start_epoch + idle_epochs < current_subnet_epoch {
                    // Increase class if they exist
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

            if subnet_node_data_find.is_none() {
                // Not included in consensus, increase
                _penalties += 1;
                SubnetNodePenalties::<T>::insert(subnet_id, subnet_node.id, _penalties);
                weight_meter.consume(db_weight.writes(1));

                // Break count of consecutive epochs of being included in in-consensus data
                if subnet_node.classification.node_class == SubnetNodeClass::Included {
                    SubnetNodeConsecutiveIncludedEpochs::<T>::insert(subnet_id, subnet_node.id, 0);
                    // SubnetNodeConsecutiveIncludedEpochs
                    weight_meter.consume(db_weight.writes(1));
                }
                continue;
            } else if penalties != 0 {
                // --- Is in consensus data, decrease penalties
                // If the validator submits themselves in the data and passes consensus, this also
                // decreases the validators penalties
                _penalties = _penalties.saturating_sub(1);
                SubnetNodePenalties::<T>::insert(subnet_id, subnet_node.id, _penalties);
                weight_meter.consume(db_weight.writes(1));
            }

            //
            // --- Consensus formed on node
            //

            // Safely unwrap node_score, we already confirmed it's not None
            let node_score = subnet_node_data_find.unwrap().score;

            // --- Calculate node weight percentage of peer versus the weighted sum
            let score_ratio: u128 =
                Self::percent_div(node_score, consensus_submission_data.weight_sum);

            // Increase penalties if under subnets penalty score threshold
            // We don't automatically increase penalties if a node is at ZERO
            // Zero should represent they are not in the subnet
            if score_ratio < score_threshold {
                _penalties += 1;
                SubnetNodePenalties::<T>::insert(subnet_id, subnet_node.id, _penalties);
                weight_meter.consume(db_weight.writes(1));
            }

            if subnet_node.classification.node_class == SubnetNodeClass::Included {
                SubnetNodeConsecutiveIncludedEpochs::<T>::mutate(
                    subnet_id,
                    subnet_node.id,
                    |n: &mut u32| *n += 1,
                );

                // SubnetNodeConsecutiveIncludedEpochs
                weight_meter.consume(db_weight.reads(1) + db_weight.writes(1));

                let consecutive_included_epochs =
                    SubnetNodeConsecutiveIncludedEpochs::<T>::get(subnet_id, subnet_node.id);

                // SubnetNodeConsecutiveIncludedEpochs
                weight_meter.consume(db_weight.reads(1));

                // --- Upgrade to Validator if no penalties and included in weights
                if _penalties == 0 && consecutive_included_epochs >= included_epochs {
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
            // All nodes are at least SubnetNodeClass::Validator from here
            //

            let reward_factor = match consensus_submission_data.attests.get(&subnet_node.id) {
                Some(data) => data.reward_factor,
                None => {
                    // If node didn't attest in super majority, accrue penalty
                    if consensus_submission_data.attestation_ratio >= super_majority_threshold {
                        _penalties += 1;
                        SubnetNodePenalties::<T>::insert(subnet_id, subnet_node.id, _penalties);
                        weight_meter.consume(db_weight.writes(1));
                    }
                    Self::percentage_factor_as_u128()
                }
            };

            if reward_factor == 0 {
                continue;
            }

            if _penalties > max_subnet_node_penalties {
                // Remove node if they haven't already
                if weight_meter.can_consume(T::WeightInfo::perform_remove_subnet_node(0u32)) {
                    Self::perform_remove_subnet_node(subnet_id, subnet_node.id);
                    weight_meter.consume(T::WeightInfo::perform_remove_subnet_node(0u32));
                }

                continue;
            }

            if score_ratio == 0 {
                continue;
            }

            // --- Calculate node_score percentage of total subnet generated epoch rewards
            let mut account_reward: u128 =
                Self::percent_mul(score_ratio, rewards_data.subnet_node_rewards);
            
            // --- Increase reward if validator
            if subnet_node.id == consensus_submission_data.validator_subnet_node_id {
                account_reward += Self::get_validator_reward(
                    consensus_submission_data.attestation_ratio,
                    consensus_submission_data.validator_reward_factor,
                );
                // Add get_validator_reward (At least 1 read, up to 2)
                // MinAttestationPercentage | BaseValidatorReward
                weight_meter.consume(db_weight.reads(2));
                match HotkeyOwner::<T>::try_get(&subnet_node.hotkey) {
                    Ok(coldkey) => {
                        Self::increase_coldkey_reputation(
                            coldkey,
                            consensus_submission_data.attestation_ratio,
                            min_attestation_percentage,
                            reputation_increase_factor,
                            current_epoch,
                        );
                        weight_meter.consume(T::WeightInfo::increase_coldkey_reputation());
                    }
                    Err(()) => (),
                };
                // HotkeyOwner
                weight_meter.consume(db_weight.reads(1));
            }

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
                    // NodeDelegateStakeBalance | TotalNodeDelegateStakeShares
                    //
                    // writes:
                    // TotalNodeDelegateStakeShares | NodeDelegateStakeBalance | TotalNodeDelegateStake
                    weight_meter.consume(db_weight.reads(5) + db_weight.writes(3));

                    node_delegate_stake_rewards.push((subnet_node.id, node_delegate_reward));
                }
            }

            // --- Increase account stake and emit event
            Self::increase_account_stake(&subnet_node.hotkey, subnet_id, account_reward);
            // AccountSubnetStake | TotalSubnetStake | TotalStake
            weight_meter.consume(db_weight.writes(3) + db_weight.reads(3));

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
            weight_meter.consume(db_weight.writes(3) + db_weight.reads(5));
        }

        Self::deposit_event(Event::SubnetRewards {
            subnet_id,
            node_rewards,
            delegate_stake_reward: rewards_data.delegate_stake_rewards,
            node_delegate_stake_rewards,
        });
    }
}
