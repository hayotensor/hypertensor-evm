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
//
// Handles all slot block steps

use super::*;
use frame_support::pallet_prelude::Weight;

impl<T: Config> Pallet<T> {
    // pub fn calculate_overwatch_rewards(current_epoch: u32) -> Weight {
    //   let mut weight = Weight::zero();
    //   let db_weight = T::DbWeight::get();

    //   let percentage_factor = Self::percentage_factor_as_u128();
    //   let power = Self::get_percent_as_f64(OverwatchDistributionPower::<T>::get());

    //   // {ID, weight}
    //   let mut subnet_weights: BTreeMap<u32,u128> = BTreeMap::new();
    //   // {node_id, score}
    //   let mut node_total_scores: BTreeMap<u32, u128> = BTreeMap::new();
    //   // {node_id, account_id}
    //   let mut node_hotkeys: BTreeMap<u32, T::AccountId> = BTreeMap::new();

    //   let total_stake = TotalOverwatchStake::<T>::get();

    //   // Step 1: Group reveals by subnet
    //   // {node_id, stake_weight}
    //   let mut node_stake_balances: BTreeMap<u32, u128> = BTreeMap::new();
    //   // {subnet_id, (subnet_weight sum, {node_id, subnet_weight})}
    //   let mut subnet_reveals: BTreeMap<u32, (u128, BTreeMap<u32, u128>)> = BTreeMap::new();
    //   for ((subnet_id, overwatch_node_id), subnet_weight) in OverwatchReveals::<T>::iter_prefix((current_epoch,)) {
    //     weight = weight.saturating_add(db_weight.reads(1));
    //     if node_stake_balances.get(&overwatch_node_id).is_none() {
    //       let Some(overwatch_node) = OverwatchNodes::<T>::get(overwatch_node_id) else {
    //         continue
    //       };
    //       let stake_balance = AccountOverwatchStake::<T>::get(overwatch_node.hotkey.clone());
    //       weight = weight.saturating_add(db_weight.reads(1));

    //       let stake_weight = Self::percent_div(stake_balance, total_stake);
    //       node_stake_balances.insert(overwatch_node_id, stake_weight);
    //       node_hotkeys.insert(overwatch_node_id, overwatch_node.hotkey.clone());
    //     }

    //     let entry = subnet_reveals.entry(subnet_id).or_insert((0, BTreeMap::new()));
    //     entry.0 += subnet_weight;                          // sum all weights for this subnet
    //     entry.1.insert(overwatch_node_id, subnet_weight);  // store each node's weight per subnet
    //   }

    //   // Step 2: Iterate each subnet
    //   // - Get subnet weights from nodes
    //   // - Score nodes
    //   for (&subnet_id, (sum_weights, node_weights)) in subnet_reveals.iter() {
    //     // Step 2a: Compute stake fractions with dampening
    //     // {node_id, adj weight}
    //     // Adjusted based on stake weight
    //     let mut adjusted_fractions: BTreeMap<u32, u128> = BTreeMap::new();
    //     let mut total_adjusted = 0_u128;

    //     // Get node stake weight
    //     for (&node_id, _) in node_weights.iter() {
    //       // Get stake weights
    //       let Some(stake_balance) = node_stake_balances.get(&node_id) else {
    //         // Redundant
    //         continue
    //       };

    //       // Get node stake weight
    //       let stake_weight = Self::percent_div(*stake_balance, total_stake);
    //       // Increase the stake weight to later be normalized
    //       let adjusted = Self::pow(Self::get_percent_as_f64(stake_weight), power);
    //       // Adjust back to u128
    //       let adjusted_u128 = Self::get_f64_as_percentage(adjusted);

    //       adjusted_fractions.insert(node_id, adjusted_u128);

    //       // Sum of total adjusted weights for normalizing
    //       total_adjusted += adjusted_u128;
    //     }

    //     // Normalize fractions (stake weights)
    //     for value in adjusted_fractions.values_mut() {
    //       *value = Self::percent_div(*value, total_adjusted);
    //     }
    //     // Step 2b: Compute average weight for subnet
    //     let mut avg_weight = *sum_weights / node_weights.len() as u128;
    //     if avg_weight > percentage_factor {
    //       avg_weight = percentage_factor;
    //     }

    //     // Score subnets
    //     subnet_weights.insert(subnet_id, avg_weight);
    //     // OverwatchSubnetWeights::<T>::insert(subnet_id, avg_weight);
    //     OverwatchSubnetWeights::<T>::insert(current_epoch, subnet_id, avg_weight);
    //     weight = weight.saturating_add(db_weight.writes(1));

    //     // Step 2c: Score nodes and accumulate
    //     for (&node_id, &node_weight) in node_weights.iter() {
    //       let stake_weight = match adjusted_fractions.get(&node_id) {
    //         Some(weight) => weight,
    //         None => continue,
    //       };

    //       let deviation = Self::percent_div((node_weight).abs_diff(avg_weight), avg_weight);
    //       let closeness_score = if deviation >= percentage_factor { 0 } else { percentage_factor - deviation };
    //       let node_final_score = Self::percent_mul(closeness_score, *stake_weight);

    //       // Step 3: Accumulate score
    //       *node_total_scores.entry(node_id).or_insert(0) += node_final_score;
    //     }
    //   }

    //   // Step 4: Normalize node scores
    //   let total_final_score: u128 = node_total_scores.values().sum();
    //   for (node_id, score) in node_total_scores.iter() {
    //     let node_final_score = Self::percent_div(*score, total_final_score);
    //     OverwatchNodeWeights::<T>::insert(current_epoch, node_id, node_final_score);
    //     weight = weight.saturating_add(db_weight.writes(1));
    //   }

    //   // Step 5: Reward nodes
    //   let ow_emissions = T::OverwatchEpochEmissions::get();

    //   for (node_id, ow_weight) in OverwatchNodeWeights::<T>::iter_prefix(current_epoch) {
    //     let hotkey = match node_hotkeys.get(&node_id) {
    //       Some(hotkey) => hotkey,
    //       None => continue,
    //     };

    //     let amount = Self::percent_mul(ow_weight, ow_emissions);
    //     if amount == 0 {
    //       continue
    //     }
    //     Self::increase_account_overwatch_stake(
    //       hotkey,
    //       amount,
    //     );
    //     // weight = weight.saturating_add(T::WeightInfo::increase_account_overwatch_stake());
    //   }

    //   // (subnet_weights, weight)
    //   weight
    // }

    // // Returns subnet weights, node scores, and db weight
    // pub fn calculate_overwatch_rewards_v2(current_epoch: u32) -> (BTreeMap<u32,u128>, BTreeMap<u32, u128>, Weight) {
    //   let mut weight = Weight::zero();
    //   let db_weight = T::DbWeight::get();

    //   let percentage_factor = Self::percentage_factor_as_u128();
    //   let power = Self::get_percent_as_f64(OverwatchDistributionPower::<T>::get());

    //   // {ID, weight}
    //   let mut subnet_weights: BTreeMap<u32,u128> = BTreeMap::new();
    //   // {node_id, score}
    //   let mut node_total_scores: BTreeMap<u32, u128> = BTreeMap::new();
    //   // {node_id, account_id}
    //   let mut node_hotkeys: BTreeMap<u32, T::AccountId> = BTreeMap::new();

    //   let total_stake = TotalOverwatchStake::<T>::get();

    //   // Step 1: Group reveals by subnet
    //   // {node_id, stake_weight}
    //   let mut node_stake_balances: BTreeMap<u32, u128> = BTreeMap::new();
    //   // {subnet_id, (subnet_weight sum, {node_id, subnet_weight})}
    //   let mut subnet_reveals: BTreeMap<u32, (u128, BTreeMap<u32, u128>)> = BTreeMap::new();
    //   for ((subnet_id, overwatch_node_id), subnet_weight) in OverwatchReveals::<T>::iter_prefix((current_epoch,)) {
    //     weight = weight.saturating_add(db_weight.reads(1));
    //     if node_stake_balances.get(&overwatch_node_id).is_none() {
    //       let Some(overwatch_node) = OverwatchNodes::<T>::get(overwatch_node_id) else {
    //         continue
    //       };
    //       let stake_balance = AccountOverwatchStake::<T>::get(overwatch_node.hotkey.clone());
    //       weight = weight.saturating_add(db_weight.reads(1));

    //       let stake_weight = Self::percent_div(stake_balance, total_stake);
    //       node_stake_balances.insert(overwatch_node_id, stake_weight);
    //       node_hotkeys.insert(overwatch_node_id, overwatch_node.hotkey.clone());
    //     }

    //     let entry = subnet_reveals.entry(subnet_id).or_insert((0, BTreeMap::new()));
    //     entry.0 += subnet_weight;                          // sum all weights for this subnet
    //     entry.1.insert(overwatch_node_id, subnet_weight);  // store each node's weight per subnet
    //   }

    //   // Step 2: Iterate each subnet
    //   // - Get subnet weights from nodes
    //   // - Score nodes
    //   for (&subnet_id, (sum_weights, node_weights)) in subnet_reveals.iter() {
    //     // Step 2a: Compute stake fractions with dampening
    //     // {node_id, adj weight}
    //     // Adjusted based on stake weight
    //     let mut adjusted_fractions: BTreeMap<u32, u128> = BTreeMap::new();
    //     let mut total_adjusted = 0_u128;

    //     // Get node stake weight
    //     for (&node_id, _) in node_weights.iter() {
    //       // Get stake weights
    //       let Some(stake_balance) = node_stake_balances.get(&node_id) else {
    //         // Redundant
    //         continue
    //       };

    //       // Get node stake weight
    //       let stake_weight = Self::percent_div(*stake_balance, total_stake);
    //       // Increase the stake weight to later be normalized
    //       let adjusted = Self::pow(Self::get_percent_as_f64(stake_weight), power);
    //       // Adjust back to u128
    //       let adjusted_u128 = Self::get_f64_as_percentage(adjusted);

    //       adjusted_fractions.insert(node_id, adjusted_u128);

    //       // Sum of total adjusted weights for normalizing
    //       total_adjusted += adjusted_u128;
    //     }

    //     // Normalize fractions (stake weights)
    //     for value in adjusted_fractions.values_mut() {
    //       *value = Self::percent_div(*value, total_adjusted);
    //     }
    //     // Step 2b: Compute average weight for subnet
    //     let mut avg_weight = *sum_weights / node_weights.len() as u128;
    //     if avg_weight > percentage_factor {
    //       avg_weight = percentage_factor;
    //     }

    //     // Score subnets
    //     subnet_weights.insert(subnet_id, avg_weight);
    //     // OverwatchSubnetWeights::<T>::insert(subnet_id, avg_weight);
    //     OverwatchSubnetWeights::<T>::insert(current_epoch, subnet_id, avg_weight);
    //     weight = weight.saturating_add(db_weight.writes(1));

    //     // Step 2c: Score nodes and accumulate
    //     for (&node_id, &node_weight) in node_weights.iter() {
    //       let stake_weight = match adjusted_fractions.get(&node_id) {
    //         Some(weight) => weight,
    //         None => continue,
    //       };

    //       let deviation = Self::percent_div((node_weight).abs_diff(avg_weight), avg_weight);
    //       let closeness_score = if deviation >= percentage_factor { 0 } else { percentage_factor - deviation };
    //       let node_final_score = Self::percent_mul(closeness_score, *stake_weight);

    //       // Step 3: Accumulate score
    //       *node_total_scores.entry(node_id).or_insert(0) += node_final_score;
    //     }
    //   }

    //   // 4-5
    //   // Step 4: Normalize node scores
    //   let total_final_score: u128 = node_total_scores.values().sum();
    //   // for (node_id, score) in node_total_scores.iter() {
    //   //   let node_final_score = Self::percent_div(*score, total_final_score);
    //   //   OverwatchNodeWeights::<T>::insert(current_epoch, node_id, node_final_score);
    //   //   weight = weight.saturating_add(db_weight.writes(1));
    //   // }

    //   // Step 5: Reward nodes
    //   let ow_emissions = T::OverwatchEpochEmissions::get();

    //   for (node_id, score) in node_total_scores.iter_mut() {
    //     let node_final_score = Self::percent_div(*score, total_final_score);
    //     *score = node_final_score;

    //     let hotkey = match node_hotkeys.get(&node_id) {
    //       Some(hotkey) => hotkey,
    //       None => continue,
    //     };

    //     let amount = Self::percent_mul(node_final_score, ow_emissions);
    //     if amount == 0 {
    //       continue
    //     }
    //     Self::increase_account_overwatch_stake(
    //       hotkey,
    //       amount,
    //     );
    //     // weight = weight.saturating_add(T::WeightInfo::increase_account_overwatch_stake());

    //   }

    //   // TODO: Emit

    //   (subnet_weights, node_total_scores, weight)
    // }

    // Returns subnet weights, node scores, and db weight
    pub fn calculate_overwatch_rewards_v3() -> Weight {
        let mut weight = Weight::zero();
        let db_weight = T::DbWeight::get();

        let current_overwatch_epoch = Self::get_current_overwatch_epoch_as_u32();

        let percentage_factor = Self::percentage_factor_as_u128();

        // {ID, weight}
        // let mut subnet_weights: BTreeMap<u32,u128> = BTreeMap::new();
        // {node_id, score}
        let mut node_total_scores: BTreeMap<u32, u128> = BTreeMap::new();
        // {node_id, account_id}
        let mut node_hotkeys: BTreeMap<u32, T::AccountId> = BTreeMap::new();

        let total_stake = TotalOverwatchStake::<T>::get();

        // Step 1: Group reveals by subnet
        // {node_id, stake_weight}
        let mut node_stake_balances: BTreeMap<u32, u128> = BTreeMap::new();
        // {subnet_id, (subnet_weight sum, {node_id, subnet_weight})}
        let mut subnet_reveals: BTreeMap<u32, (u128, BTreeMap<u32, u128>)> = BTreeMap::new();
        for ((subnet_id, overwatch_node_id), subnet_weight) in
            OverwatchReveals::<T>::iter_prefix((current_overwatch_epoch.saturating_sub(1),))
        {
            weight = weight.saturating_add(db_weight.reads(1));
            if node_stake_balances.get(&overwatch_node_id).is_none() {
                let Some(overwatch_node) = OverwatchNodes::<T>::get(overwatch_node_id) else {
                    continue;
                };
                let stake_balance = AccountOverwatchStake::<T>::get(overwatch_node.hotkey.clone());
                weight = weight.saturating_add(db_weight.reads(1));

                let stake_weight = Self::percent_div(stake_balance, total_stake);
                node_stake_balances.insert(overwatch_node_id, stake_weight);
                node_hotkeys.insert(overwatch_node_id, overwatch_node.hotkey.clone());
            }

            let entry = subnet_reveals
                .entry(subnet_id)
                .or_insert((0, BTreeMap::new()));
            entry.0 += subnet_weight; // sum all weights for this subnet
            entry.1.insert(overwatch_node_id, subnet_weight); // store each node's weight per subnet (subnet weight the overwatch submitted)
        }

        // Step 2: Iterate each subnet
        // - Get subnet weights from nodes
        // - Score nodes
        for (&subnet_id, (sum_weights, node_weights)) in subnet_reveals.iter() {
            // Step 2a: Compute stake fractions with dampening
            // {node_id, adj weight}
            // Adjusted based on stake weight
            let mut adjusted_fractions: BTreeMap<u32, u128> = BTreeMap::new();
            let mut total_adjusted = 0_u128;

            // Get node stake weight
            for (&node_id, subnet_weight) in node_weights.iter() {
                // Get stake weights
                let Some(stake_weight) = node_stake_balances.get(&node_id) else {
                    // Redundant
                    continue;
                };

                let stake_weight_adj_subnet_weight =
                    Self::percent_mul(*subnet_weight, *stake_weight);

                adjusted_fractions.insert(node_id, stake_weight_adj_subnet_weight);

                // Sum of total adjusted subnet weights for normalizing
                total_adjusted += stake_weight_adj_subnet_weight;
            }

            // Normalize fractions (stake weights)
            for value in adjusted_fractions.values_mut() {
                *value = Self::percent_div(*value, total_adjusted);
            }

            // Score subnets
            // OverwatchSubnetWeights::<T>::insert(current_overwatch_epoch, subnet_id, avg_weight);
            OverwatchSubnetWeights::<T>::insert(
                current_overwatch_epoch.saturating_sub(1),
                subnet_id,
                total_adjusted,
            );
            weight = weight.saturating_add(db_weight.writes(1));

            // Step 2c: Score nodes and accumulate
            for (&node_id, &subnet_weight) in node_weights.iter() {
                // Get the deviation from the resulting score.
                // We check the abs diff since the submitted weights can only be between 0.0-1.0
                let deviation = subnet_weight.abs_diff(total_adjusted);

                let closeness_score = if deviation >= percentage_factor {
                    0
                } else {
                    percentage_factor - deviation
                };

                let node_final_score = Self::percent_mul(closeness_score, total_adjusted);

                // Step 3: Accumulate score
                *node_total_scores.entry(node_id).or_insert(0) += node_final_score;
            }
        }

        // 4-5
        // Step 4: Normalize node scores
        let total_final_score: u128 = node_total_scores.values().sum();

        // Step 5: Reward nodes
        let ow_emissions = T::OverwatchEpochEmissions::get();

        for (node_id, score) in node_total_scores.iter() {
            let node_final_score = Self::percent_div(*score, total_final_score);

            // For data purposes only
            OverwatchNodeWeights::<T>::insert(
                current_overwatch_epoch.saturating_sub(1),
                node_id,
                node_final_score,
            );
            weight = weight.saturating_add(db_weight.writes(1));

            let hotkey = match node_hotkeys.get(&node_id) {
                Some(hotkey) => hotkey,
                None => continue,
            };

            let amount = Self::percent_mul(node_final_score, ow_emissions);
            if amount == 0 {
                continue;
            }
            Self::increase_account_overwatch_stake(hotkey, amount);
            // weight = weight.saturating_add(T::WeightInfo::increase_account_overwatch_stake());
        }

        // TODO: Emit
        // log::error!("calculate_overwatch_rewards_v3 done");

        weight
    }

    pub fn emission_step(
        block: u32,
        current_epoch: u32,
        current_subnet_epoch: u32,
        subnet_id: u32,
    ) -> Weight {
        let mut weight = Weight::zero();
        let db_weight = T::DbWeight::get();

        // Get all subnet weights calculated at the start of the blockchains epoch
        if let Ok(subnet_emission_weights) = FinalSubnetEmissionWeights::<T>::try_get(current_epoch)
        {
            weight = weight.saturating_add(db_weight.reads(1));

            // Get weight of subnet_id from calculated weights
            if let Some(&subnet_weight) = subnet_emission_weights.weights.get(&subnet_id) {
                // Get elected consensus submission from the previous epoch
                if let Some((consensus_submission_data, consensus_submission_weight)) =
                    Self::precheck_subnet_consensus_submission(subnet_id, current_subnet_epoch - 1)
                {
                    // Accumulate weight from precheck
                    weight = weight.saturating_add(consensus_submission_weight);

                    // Calculate rewards
                    let (rewards_data, rewards_weight) = Self::calculate_rewards(
                        subnet_id,
                        subnet_emission_weights.validator_emissions,
                        subnet_weight,
                    );
                    weight = weight.saturating_add(rewards_weight);

                    // Read constants
                    let min_attestation = MinAttestationPercentage::<T>::get();
                    let rep_increase = ReputationIncreaseFactor::<T>::get();
                    let rep_decrease = ReputationDecreaseFactor::<T>::get();
                    let vast_majority = MinVastMajorityAttestationPercentage::<T>::get();
                    weight = weight.saturating_add(db_weight.reads(4));

                    // Distribute rewards
                    let dist_weight = Self::distribute_rewards(
                        subnet_id,
                        block,
                        current_epoch,
                        current_subnet_epoch, // used for graduating nodes
                        consensus_submission_data,
                        rewards_data,
                        min_attestation,
                        rep_increase,
                        rep_decrease,
                        vast_majority,
                    );
                    weight = weight.saturating_add(dist_weight);
                } else {
                    // Validator didn't submit consensus
                    weight = weight.saturating_add(db_weight.reads(1));
                }

                // --- Elect new validator for the current epoch
                // The current epoch is the start of the subnets epoch
                // We only elect if the subnet has weights, otherwise it isn't active yet
                // See `calculate_subnet_weights`
                Self::elect_validator_v3(subnet_id, current_subnet_epoch, block);
                // weight = weight.saturating_add(T::WeightInfo::elect_validator_v3());

                // After election, we activate nodes in the queue

                let subnet_node_queue_epochs = SubnetNodeQueueEpochs::<T>::get(subnet_id);
                let max_nodes = MaxSubnetNodes::<T>::get();
                let total_active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
                let churn_limit = ChurnLimit::<T>::get(subnet_id);
                let take = if max_nodes.saturating_sub(total_active_nodes) < churn_limit {
                    max_nodes.saturating_sub(total_active_nodes)
                } else {
                    churn_limit
                };
                weight = weight.saturating_add(db_weight.reads(4));

                let mut queue = SubnetNodeQueue::<T>::get(subnet_id);
                if queue.len() != 0 && take != 0 {
                    let mut activated_nodes = 0;
                    // Activate any nodes in the unpause queue
                    for subnet_node in queue.iter().take(take as usize) {
                        log::error!("subnet_node.classification.start_epoch {:?}", subnet_node.classification.start_epoch);
                        if subnet_node.classification.start_epoch + subnet_node_queue_epochs
                            < current_subnet_epoch
                        {
                            weight = weight.saturating_add(Self::do_activate_subnet_node_v2(
                                subnet_id,
                                subnet_node.clone(),
                                current_subnet_epoch,
                            ));
                            activated_nodes += 1;
                        } else {
                            // Nodes are in order, break early if first one isn't qualified yet
                            break;
                        }
                    }

                    // Remove the processed nodes from the front
                    queue.drain(0..activated_nodes);

                    // Update storage
                    SubnetNodeQueue::<T>::set(subnet_id, queue);
                    weight = weight.saturating_add(db_weight.writes(1));
                }
            }
        } else {
            // Count DB read even if subnet_emission_weights is missing
            weight = weight.saturating_add(db_weight.reads(1));
        }

        weight
    }

    /// Calculate and store emissions distribution
    ///
    pub fn handle_subnet_emission_weights(epoch: u32) -> Weight {
        // Get weights
        // - Takes in general epoch
        let (subnet_weights, mut weight): (BTreeMap<u32, u128>, Weight) =
            Self::calculate_subnet_weights_v3(epoch);

        // Store weights and handle foundation
        if !subnet_weights.is_empty() {
            let (validator_emissions, foundation_emissions) = Self::get_epoch_emissions_v2(epoch);
            let foundation_emissions_as_balance = Self::u128_to_balance(foundation_emissions);
            if foundation_emissions_as_balance.is_some() {
                Self::add_balance_to_treasury(foundation_emissions_as_balance.unwrap());
                // weight = weight.saturating_add(T::WeightInfo::add_balance_to_treasury());
            }
            let data = DistributionData {
                validator_emissions: validator_emissions,
                weights: subnet_weights,
            };
            FinalSubnetEmissionWeights::<T>::insert(epoch, data);
            weight = weight.saturating_add(T::DbWeight::get().writes(1));
        }

        weight
    }

    // /// Get overall subnet weights based on delegate stake
    // pub fn calculate_subnet_weights(epoch: u32) -> (BTreeMap<u32, u128>, Weight) {
    //   let mut weight = Weight::zero();
    //   let db_weight = T::DbWeight::get();

    //   let total_delegate_stake = TotalDelegateStake::<T>::get();
    //   weight = weight.saturating_add(db_weight.reads(1));

    //   let mut stake_weights: BTreeMap<u32, f64> = BTreeMap::new();
    //   let mut stake_weight_sum: f64 = 0.0;
    //   let mut total_subnet_reads = 0u64;

    //   let (overwatch_subnet_weights, _, overwatch_block_weight) = Self::calculate_overwatch_rewards_v2(
    //     Self::get_current_overwatch_epoch_as_u32()
    //   );
    //   weight = weight.saturating_add(overwatch_block_weight);

    //   for (subnet_id, data) in SubnetsData::<T>::iter() {
    //     total_subnet_reads += 1;
    //     if data.start_epoch > epoch && data.state != SubnetState::Active {
    //       continue
    //     }

    //     let total_subnet_delegate_stake = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
    //     weight = weight.saturating_add(db_weight.reads(1));
    //     // 1. Get all weights in f64

    //     let subnet_weight: f64 = total_subnet_delegate_stake as f64 / total_delegate_stake as f64;
    //     let adjusted_subnet_weight: f64 = Self::pow(subnet_weight, 0.5);

    //     let overwatch_subnet_weight = match overwatch_subnet_weights.get(&subnet_id) {
    //       Some(weight) => Self::get_percent_as_f64(*weight),
    //       None => 1.0,
    //     };
    //     let final_adjusted_subnet_weight: f64 = adjusted_subnet_weight * adjusted_subnet_weight;

    //     // stake_weights.insert(subnet_id, adjusted_subnet_weight);
    //     // stake_weight_sum += adjusted_subnet_weight;
    //     stake_weights.insert(subnet_id, final_adjusted_subnet_weight);
    //     stake_weight_sum += final_adjusted_subnet_weight;
    //     weight = weight.saturating_add(Weight::from_parts(400_000, 0));
    //   }

    //   weight = weight.saturating_add(db_weight.reads(total_subnet_reads));
    //   let mut subnet_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();
    //   let percentage_factor = Self::percentage_factor_as_u128();

    //   // --- Normalize delegate stake weights from power
    //   for (subnet_id, subnet_weight) in stake_weights {
    //     let weight_normalized: u128 = (subnet_weight / stake_weight_sum * percentage_factor as f64) as u128;
    //     subnet_weights_normalized.insert(subnet_id, weight_normalized);
    //     weight = weight.saturating_add(Weight::from_parts(400_000, 0));
    //   }

    //   //
    //   // Weight calc complete
    //   //

    //   // --- From the resulting weights, adjust them based on overwatch weights

    //   (subnet_weights_normalized, weight)
    // }

    // pub fn calculate_subnet_weights_v2(epoch: u32) -> (BTreeMap<u32, u128>, Weight) {
    //   let mut weight = Weight::zero();
    //   let db_weight = T::DbWeight::get();

    //   let total_delegate_stake = TotalDelegateStake::<T>::get();
    //   weight = weight.saturating_add(db_weight.reads(1));

    //   // {subnet_id, weight}
    //   let mut stake_weights: BTreeMap<u32, f64> = BTreeMap::new();
    //   // {subnet_id, count}
    //   let mut stake_weight_sum: f64 = 0.0;
    //   let total_electable_nodes: f64 = TotalElectableNodes::<T>::get() as f64;
    //   let mut total_subnet_reads = 0u64;

    //   let current_overwatch_epoch = Self::get_current_overwatch_epoch_as_u32();

    //   let (overwatch_subnet_weights, _, overwatch_block_weight) = Self::calculate_overwatch_rewards_v2(
    //     current_overwatch_epoch
    //   );
    //   weight = weight.saturating_add(overwatch_block_weight);

    //   let dstake_factor = Self::get_percent_as_f64(DelegateStakeWeightFactor::<T>::get());
    //   let node_factor = 1.0 - dstake_factor;

    //   for (subnet_id, data) in SubnetsData::<T>::iter() {
    //     total_subnet_reads += 1;
    //     if data.start_epoch > epoch && data.state != SubnetState::Active {
    //       continue
    //     }

    //     let total_subnet_delegate_stake = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
    //     weight = weight.saturating_add(db_weight.reads(1));

    //     // - Get delegate stake weight in f64
    //     let subnet_dstake_weight: f64 = (total_subnet_delegate_stake as f64 / total_delegate_stake as f64).clamp(0.0, 1.0);

    //     // - Get node count weight in f64
    //     let electable_nodes_count = TotalSubnetElectableNodes::<T>::get(subnet_id);
    //     weight = weight.saturating_add(db_weight.reads(1));
    //     let subnet_nodes_weight = electable_nodes_count as f64 / total_electable_nodes;

    //     let overwatch_subnet_weight = match overwatch_subnet_weights.get(&subnet_id) {
    //       Some(weight) => Self::get_percent_as_f64(*weight).min(1.0),
    //       None => 1.0,
    //     };

    //     // - Get combined weight (stake + node count) * overwatchers weight
    //     let subnet_weight = (
    //       (
    //         subnet_dstake_weight * dstake_factor +
    //         subnet_nodes_weight * node_factor
    //       ) *
    //       overwatch_subnet_weight
    //     ).clamp(0.0, 1.0);

    //     // - Adj weight (to later be normalized)
    //     let adj_subnet_weight: f64 = Self::pow(subnet_weight, 0.5);

    //     stake_weights.insert(subnet_id, adj_subnet_weight);
    //     stake_weight_sum += adj_subnet_weight;
    //     weight = weight.saturating_add(Weight::from_parts(400_000, 0));
    //   }

    //   weight = weight.saturating_add(db_weight.reads(total_subnet_reads));
    //   let mut subnet_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();
    //   let percentage_factor = Self::percentage_factor_as_u128();

    //   // --- Normalize delegate stake weights from power
    //   for (subnet_id, subnet_dstake_weight) in stake_weights {
    //     let weight_normalized: u128 = (subnet_dstake_weight / stake_weight_sum * percentage_factor as f64) as u128;
    //     // overwatch_subnet_weights.get(&subnet_id);
    //     let ow_subnet_weight = OverwatchSubnetWeights::<T>::get(current_overwatch_epoch, subnet_id);
    //     subnet_weights_normalized.insert(subnet_id, weight_normalized);
    //     weight = weight.saturating_add(Weight::from_parts(400_000, 0));
    //   }

    //   //
    //   // Weight calc complete
    //   //

    //   (subnet_weights_normalized, weight)
    // }

    /// Calculate emissions distribution weights
    ///
    /// # Based On
    /// - Delegate stake weight
    /// - Node count weight
    /// - Overwatch weight
    ///
    /// This calculates the distribution of emissions to each subnet
    ///
    pub fn calculate_subnet_weights_v3(epoch: u32) -> (BTreeMap<u32, u128>, Weight) {
        let mut weight = Weight::zero();
        let db_weight = T::DbWeight::get();

        let subnet_distribution_power =
            Self::get_percent_as_f64(SubnetDistributionPower::<T>::get());
        let total_delegate_stake = TotalDelegateStake::<T>::get();

        // {subnet_id, weight}
        let mut subnet_weights: BTreeMap<u32, f64> = BTreeMap::new();
        // {subnet_id, count}
        let mut subnet_weight_sum: f64 = 0.0;
        let total_electable_nodes: f64 = TotalElectableNodes::<T>::get() as f64;
        let mut total_subnet_reads = 0u64;

        let dstake_factor = Self::get_percent_as_f64(DelegateStakeWeightFactor::<T>::get());
        let node_factor = 1.0 - dstake_factor;

        weight = weight.saturating_add(db_weight.reads(4));

        let current_overwatch_epoch = Self::get_current_overwatch_epoch_as_u32();

        for (subnet_id, data) in SubnetsData::<T>::iter() {
            total_subnet_reads += 1;
            // - Must be active to calculate rewards distribution
            if data.start_epoch > epoch && data.state != SubnetState::Active {
                continue;
            }

            let total_subnet_delegate_stake = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
            weight = weight.saturating_add(db_weight.reads(1));

            // - Get delegate stake weight in f64
            let subnet_dstake_weight: f64 =
                (total_subnet_delegate_stake as f64 / total_delegate_stake as f64).clamp(0.0, 1.0);

            // - Get node count weight in f64
            let electable_nodes_count = TotalSubnetElectableNodes::<T>::get(subnet_id);
            weight = weight.saturating_add(db_weight.reads(1));
            let subnet_nodes_weight = electable_nodes_count as f64 / total_electable_nodes;

            // - Get Overwatch weight in f64
            let overwatch_subnet_weight = match OverwatchSubnetWeights::<T>::try_get(
                current_overwatch_epoch.saturating_sub(1),
                subnet_id,
            ) {
                Ok(weight) => Self::get_percent_as_f64(weight).min(1.0),
                Err(()) => 1.0,
            };

            weight = weight.saturating_add(db_weight.reads(1));

            // - Get combined weight (stake + node count) * overwatchers weight
            let subnet_weight = ((subnet_dstake_weight * dstake_factor
                + subnet_nodes_weight * node_factor)
                * overwatch_subnet_weight)
                .clamp(0.0, 1.0);

            // - Adj weight (to later be normalized)
            let adj_subnet_weight: f64 = Self::pow(subnet_weight, subnet_distribution_power);

            subnet_weights.insert(subnet_id, adj_subnet_weight);
            subnet_weight_sum += adj_subnet_weight;
            weight = weight.saturating_add(Weight::from_parts(400_000, 0));
        }

        weight = weight.saturating_add(db_weight.reads(total_subnet_reads));
        let mut subnet_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();
        let percentage_factor = Self::percentage_factor_as_u128();

        // --- Normalize delegate stake weights from power
        for (subnet_id, subnet_weight) in subnet_weights {
            let weight_normalized: u128 =
                (subnet_weight / subnet_weight_sum * percentage_factor as f64) as u128;
            subnet_weights_normalized.insert(subnet_id, weight_normalized);
            weight = weight.saturating_add(Weight::from_parts(400_000, 0));
        }

        //
        // Weight calc complete
        //

        (subnet_weights_normalized, weight)
    }

    // // No decimals
    // pub fn calculate_subnet_weights_v4(epoch: u32) -> (BTreeMap<u32, u128>, Weight) {
    //     let mut weight = Weight::zero();
    //     let db_weight = T::DbWeight::get();

    //     let subnet_distribution_power =
    //         Self::get_percent_as_f64(SubnetDistributionPower::<T>::get());
    //     let total_delegate_stake = TotalDelegateStake::<T>::get();

    //     // {subnet_id, weight}
    //     let mut subnet_weights: BTreeMap<u32, f64> = BTreeMap::new();
    //     // {subnet_id, count}
    //     let mut subnet_weight_sum: u128 = 0;
    //     let total_electable_nodes: f64 = TotalElectableNodes::<T>::get() as f64;
    //     let mut total_subnet_reads = 0u64;

    //     let dstake_factor = Self::get_percent_as_f64(DelegateStakeWeightFactor::<T>::get());
    //     let node_factor = 1.0 - dstake_factor;

    //     weight = weight.saturating_add(db_weight.reads(4));

    //     let current_overwatch_epoch = Self::get_current_overwatch_epoch_as_u32();

    //     for (subnet_id, data) in SubnetsData::<T>::iter() {
    //         total_subnet_reads += 1;
    //         // - Must be active to calculate rewards distribution
    //         if data.start_epoch > epoch && data.state != SubnetState::Active {
    //             continue;
    //         }

    //         let total_subnet_delegate_stake = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
    //         weight = weight.saturating_add(db_weight.reads(1));

    //         // - Get delegate stake weight in f64
    //         let subnet_dstake_weight: f64 =
    //             (total_subnet_delegate_stake as f64 / total_delegate_stake as f64).clamp(0.0, 1.0);

    //         // - Get node count weight in f64
    //         let electable_nodes_count = TotalSubnetElectableNodes::<T>::get(subnet_id);
    //         weight = weight.saturating_add(db_weight.reads(1));
    //         let subnet_nodes_weight = electable_nodes_count as f64 / total_electable_nodes;

    //         // - Get Overwatch weight in f64
    //         let overwatch_subnet_weight = match OverwatchSubnetWeights::<T>::try_get(
    //             current_overwatch_epoch.saturating_sub(1),
    //             subnet_id,
    //         ) {
    //             Ok(weight) => Self::get_percent_as_f64(weight).min(1.0),
    //             Err(()) => 1.0,
    //         };

    //         weight = weight.saturating_add(db_weight.reads(1));

    //         // - Get combined weight (stake + node count) * overwatchers weight
    //         let subnet_weight = ((subnet_dstake_weight * dstake_factor
    //             + subnet_nodes_weight * node_factor)
    //             * overwatch_subnet_weight)
    //             .clamp(0.0, 1.0);

    //         // - Adj weight (to later be normalized)
    //         let adj_subnet_weight: f64 = Self::pow(subnet_weight, subnet_distribution_power);

    //         subnet_weights.insert(subnet_id, adj_subnet_weight);
    //         subnet_weight_sum += adj_subnet_weight;
    //         weight = weight.saturating_add(Weight::from_parts(400_000, 0));
    //     }

    //     weight = weight.saturating_add(db_weight.reads(total_subnet_reads));
    //     let mut subnet_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();
    //     let percentage_factor = Self::percentage_factor_as_u128();

    //     // --- Normalize delegate stake weights from power
    //     for (subnet_id, subnet_weight) in subnet_weights {
    //         let weight_normalized: u128 = Self::percent_div(subnet_weight, subnet_weight_sum);
    //         subnet_weights_normalized.insert(subnet_id, weight_normalized);
    //         weight = weight.saturating_add(Weight::from_parts(400_000, 0));
    //     }

    //     //
    //     // Weight calc complete
    //     //

    //     (subnet_weights_normalized, weight)
    // }

    /// Compile consensus data from the previous epochs attestation proposals
    ///
    pub fn precheck_subnet_consensus_submission(
        subnet_id: u32,
        subnet_epoch: u32,
    ) -> Option<(ConsensusSubmissionData<T::AccountId>, Weight)> {
        let mut weight = Weight::zero();
        let submission = match SubnetConsensusSubmission::<T>::try_get(subnet_id, subnet_epoch) {
            Ok(submission) => submission,
            Err(()) => {
                SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
                return None;
            }
        };
        weight = weight.saturating_add(T::DbWeight::get().reads(1));

        let attestations: u128 = submission.attests.len() as u128;
        let subnet_nodes = submission.subnet_nodes;

        // --- Get all qualified possible attestors
        // We take the subnet nodes generated from the validators `propose_attestation` call
        let validators: Vec<SubnetNode<T::AccountId>> = subnet_nodes
            .clone()
            .into_iter()
            .filter(|subnet_node| {
                subnet_node.has_classification(&SubnetNodeClass::Validator, subnet_epoch)
            })
            .collect();

        let mut attestation_ratio = Self::percent_div(attestations, validators.len() as u128);

        // Redundant
        // When subnet nodes exit, the consensus data is updated to remove them from it
        let percentage_factor = Self::percentage_factor_as_u128();
        if attestation_ratio > percentage_factor {
            attestation_ratio = percentage_factor;
        }

        let validator_subnet_node_id: u32 = submission.validator_id;
        let data_length = submission.data.len() as u32;

        // --- Get sum of subnet total scores for use of divvying rewards
        let weight_sum = submission
            .data
            .iter()
            .fold(0, |acc, x| acc.saturating_add(x.score));

        let consensus_data = ConsensusSubmissionData {
            validator_subnet_node_id: submission.validator_id,
            attestation_ratio: attestation_ratio,
            weight_sum: weight_sum,
            data_length: data_length,
            data: submission.data,
            attests: submission.attests,
            subnet_nodes: subnet_nodes,
        };

        Some((consensus_data, weight))
    }

    /// Calculate the subnets rewards and how they are distributed throughout the subnet
    ///
    pub fn calculate_rewards(
        subnet_id: u32,
        overall_rewards: u128,
        emission_weight: u128,
    ) -> (RewardsData, Weight) {
        let mut weight = Weight::zero();

        let delegate_stake_rewards_percentage =
            SubnetDelegateStakeRewardsPercentage::<T>::get(subnet_id);
        let subnet_owner_percentage = SubnetOwnerPercentage::<T>::get();
        weight = weight.saturating_add(T::DbWeight::get().reads(2));

        let overall_subnet_reward: u128 = Self::percent_mul(overall_rewards, emission_weight);

        // --- Get owner rewards
        let subnet_owner_reward: u128 =
            Self::percent_mul(overall_subnet_reward, subnet_owner_percentage);

        // --- Get subnet rewards minus owner cut
        let subnet_rewards: u128 = overall_subnet_reward.saturating_sub(subnet_owner_reward);

        // --- Get delegators rewards
        let delegate_stake_rewards: u128 =
            Self::percent_mul(subnet_rewards, delegate_stake_rewards_percentage);

        // --- Get subnet nodes rewards total
        let subnet_node_rewards: u128 = subnet_rewards.saturating_sub(delegate_stake_rewards);

        let rewards_data = RewardsData {
            overall_subnet_reward,
            subnet_owner_reward,
            subnet_rewards,
            delegate_stake_rewards,
            subnet_node_rewards,
        };

        (rewards_data, weight)
    }
}
