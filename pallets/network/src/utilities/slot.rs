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

  // pub fn calculate_overwatch_rewards(current_epoch: u32) -> BTreeMap<u32, f64> {
  //   let power = 0.5;
  //   // node_id -> score
  //   let mut node_total_scores: BTreeMap<u32, f64> = BTreeMap::new();

  //   // Step 1: Group reveals by subnet
  //   let mut subnet_reveals: BTreeMap<u32, BTreeMap<u32, u128>> = BTreeMap::new();
  //   for ((subnet_id, node_id), weight) in OverwatchReveals::<T>::iter_prefix((current_epoch,)) {
  //     subnet_reveals.entry(subnet_id).or_default().insert(node_id, weight);
  //   }

  //   let total_stake = TotalOverwatchStake::<T>::get();

  //   // Step 2: Iterate each subnet
  //   for (subnet_id, node_weights) in subnet_reveals.iter() {
  //     // Step 2a: Compute stake fractions with dampening
  //     let mut adjusted_fractions = BTreeMap::new();
  //     let mut total_adjusted = 0f64;

  //     // Get node stake weight
  //     for (&node_id, _) in node_weights.iter() {
  //       // Get stake weights
  //       let Some(overwatch_node) = OverwatchNodes::<T>::get(node_id) else {
  //         continue
  //       };
  //       let stake = AccountOverwatchStake::<T>::get(overwatch_node.hotkey);
  //       let fraction = stake as f64 / total_stake as f64;
  //       let adjusted = Self::pow(fraction, power);
  //       adjusted_fractions.insert(node_id, adjusted);

  //       log::error!("node_id  {:?}", node_id);
  //       log::error!("fraction {:?}", fraction);
  //       log::error!("adjusted {:?}", adjusted);
  //       log::error!(" ");

  //       total_adjusted += adjusted;
  //     }

  //     // Normalize fractions (stake weights)
  //     for value in adjusted_fractions.values_mut() {
  //       // let test = *value /= total_adjusted;
  //       // log::error!("adjusted_fractions {:?}", test);
  //       // log::error!(" ");
  //       *value /= total_adjusted;
  //     }

  //     // Step 2b: Compute average weight for subnet
  //     let sum_weights: u128 = node_weights.values().sum();
  //     let avg_weight = sum_weights as f64 / node_weights.len() as f64;

  //     // Step 2c: Score nodes and accumulate
  //     for (&node_id, &weight) in node_weights.iter() {
  //       let stake_weight = *adjusted_fractions.get(&node_id).unwrap_or(&0.0);
  //       if stake_weight == 0.0 {
  //         log::error!("NO STAKE");
  //         continue
  //       }
  //       let deviation = ((weight as f64 - avg_weight).abs()) / avg_weight;
  //       let closeness_score = if deviation >= 1.0 { 0.0 } else { 1.0 - deviation };
        
  //       let final_score = closeness_score * stake_weight;

  //       log::error!("deviation {:?}", deviation);
  //       log::error!("closeness_score {:?}", closeness_score);
  //       log::error!("stake_weight {:?}", stake_weight);
  //       log::error!("final_score {:?}", final_score);
  //       log::error!(" ");

  //       // Step 3: Accumulate score
  //       *node_total_scores.entry(node_id).or_insert(0.0) += final_score;
  //     }
  //   }

  //   // Step 4: Normalize total scores
  //   let total_final_score: f64 = node_total_scores.values().sum();
  //   log::error!("total_final_score {:?}", total_final_score);
  //   for (node_id, score) in node_total_scores.iter_mut() {
  //     *score /= total_final_score;
  //   }

  //   node_total_scores
  // }

  // pub fn calculate_overwatch_rewards(current_epoch: u32) -> BTreeMap<u32, u128> {
  //   let mut subnet_weights: BTreeMap<u32,u128> = BTreeMap::new();
  //   let percentage_factor = Self::percentage_factor_as_u128();
  //   let power = 0.5;

  //   let mut node_total_scores: BTreeMap<u32, u128> = BTreeMap::new();

  //   let total_stake = TotalOverwatchStake::<T>::get();

  //   // Step 1: Group reveals by subnet
  //   let mut subnet_reveals: BTreeMap<u32, (u128, BTreeMap<u32, u128>)> = BTreeMap::new();
  //   for ((subnet_id, overwatch_node_id), weight) in OverwatchReveals::<T>::iter_prefix((current_epoch,)) {
  //     let Some(overwatch_node) = OverwatchNodes::<T>::get(overwatch_node_id) else {
  //       continue
  //     };
  //     let stake_balance = AccountOverwatchStake::<T>::get(overwatch_node.hotkey);

  //     let stake_weight = Self::percent_div(stake_balance, total_stake);

  //     let entry = subnet_reveals.entry(subnet_id).or_insert((0, BTreeMap::new()));
  //     entry.0 += weight;                          // sum all weights for this subnet
  //     entry.1.insert(overwatch_node_id, weight);  // store each node's weight per subnet
  //   }


  //   // Step 2: Iterate each subnet
  //   for (&subnet_id, (sum_weights, node_weights)) in subnet_reveals.iter() {
  //     // Step 2a: Compute stake fractions with dampening
  //     let mut adjusted_fractions = BTreeMap::new();
  //     let mut total_adjusted = 0_u128;

  //     // Get node stake weight
  //     for (&node_id, _) in node_weights.iter() {
  //       // Get stake weights
  //       let Some(overwatch_node) = OverwatchNodes::<T>::get(node_id) else {
  //         continue
  //       };
  //       let stake = AccountOverwatchStake::<T>::get(overwatch_node.hotkey);
  //       // Get node stake weight
  //       let fraction = Self::percent_div(stake, total_stake);
  //       // Increase the stake weight to later be normalized
  //       let adjusted = Self::pow(Self::get_percent_as_f64(fraction), power);
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
  //     let avg_weight = *sum_weights / node_weights.len() as u128;

  //     // Step 2c: Score nodes and accumulate
  //     for (&node_id, &weight) in node_weights.iter() {
  //       let stake_weight = *adjusted_fractions.get(&node_id).unwrap_or(&0);
  //       if stake_weight == 0 {
  //         continue
  //       }
  //       let deviation = Self::percent_div(
  //         (weight).abs_diff(avg_weight),
  //         avg_weight
  //       );
  //       let closeness_score = if deviation >= percentage_factor { 0 } else { percentage_factor - deviation };
  //       let final_score = Self::percent_mul(closeness_score, stake_weight);

  //       // Step 3: Accumulate score
  //       *node_total_scores.entry(node_id).or_insert(0) += final_score;
  //     }

  //     // Score subnets
  //   }
  //   // TODO: Store subnet overwatch weights (subnet scores from ow nodes)

  //   // Step 4: Normalize node total scores
  //   let total_final_score: u128 = node_total_scores.values().sum();

  //   for (node_id, score) in node_total_scores.iter_mut() {
  //     *score = Self::percent_div(*score, total_final_score);
  //   }
  //   // // TODO: Store overwatch node weights (ow node scores)

  //   node_total_scores
  // }

  pub fn calculate_overwatch_rewards(current_epoch: u32) -> Weight {
    let mut weight = Weight::zero();
    let mut subnet_weights: BTreeMap<u32,u128> = BTreeMap::new();
    let percentage_factor = Self::percentage_factor_as_u128();
    let power = 0.5;

    let mut node_total_scores: BTreeMap<u32, u128> = BTreeMap::new();

    let total_stake = TotalOverwatchStake::<T>::get();

    // Step 1: Group reveals by subnet
    let mut subnet_reveals: BTreeMap<u32, (u128, BTreeMap<u32, u128>)> = BTreeMap::new();
    for ((subnet_id, overwatch_node_id), weight) in OverwatchReveals::<T>::iter_prefix((current_epoch,)) {
      let Some(overwatch_node) = OverwatchNodes::<T>::get(overwatch_node_id) else {
        continue
      };
      let stake_balance = AccountOverwatchStake::<T>::get(overwatch_node.hotkey);

      let stake_weight = Self::percent_div(stake_balance, total_stake);

      let entry = subnet_reveals.entry(subnet_id).or_insert((0, BTreeMap::new()));
      entry.0 += weight;                          // sum all weights for this subnet
      entry.1.insert(overwatch_node_id, weight);  // store each node's weight per subnet
    }


    // Step 2: Iterate each subnet
    for (&subnet_id, (sum_weights, node_weights)) in subnet_reveals.iter() {
      // Step 2a: Compute stake fractions with dampening
      let mut adjusted_fractions = BTreeMap::new();
      let mut total_adjusted = 0_u128;

      // Get node stake weight
      for (&node_id, _) in node_weights.iter() {
        // Get stake weights
        let Some(overwatch_node) = OverwatchNodes::<T>::get(node_id) else {
          continue
        };
        let stake = AccountOverwatchStake::<T>::get(overwatch_node.hotkey);
        // Get node stake weight
        let fraction = Self::percent_div(stake, total_stake);
        // Increase the stake weight to later be normalized
        let adjusted = Self::pow(Self::get_percent_as_f64(fraction), power);
        // Adjust back to u128
        let adjusted_u128 = Self::get_f64_as_percentage(adjusted);

        adjusted_fractions.insert(node_id, adjusted_u128);

        // Sum of total adjusted weights for normalizing
        total_adjusted += adjusted_u128;
      }

      // Normalize fractions (stake weights)
      for value in adjusted_fractions.values_mut() {
        *value = Self::percent_div(*value, total_adjusted);
      }

      // Step 2b: Compute average weight for subnet
      let mut avg_weight = *sum_weights / node_weights.len() as u128;
      if avg_weight > percentage_factor {
        avg_weight = percentage_factor;
      }
      // Score subnets
      OverwatchSubnetWeights::<T>::insert(subnet_id, avg_weight);

      // Step 2c: Score nodes and accumulate
      for (&node_id, &weight) in node_weights.iter() {
        let stake_weight = *adjusted_fractions.get(&node_id).unwrap_or(&0);
        if stake_weight == 0 {
          continue
        }
        let deviation = Self::percent_div(
          (weight).abs_diff(avg_weight),
          avg_weight
        );
        let closeness_score = if deviation >= percentage_factor { 0 } else { percentage_factor - deviation };
        let final_score = Self::percent_mul(closeness_score, stake_weight);

        // Step 3: Accumulate score
        *node_total_scores.entry(node_id).or_insert(0) += final_score;
      }
    }

    // Step 4: Normalize node total scores
    let total_final_score: u128 = node_total_scores.values().sum();

    for (node_id, score) in node_total_scores.iter() {
      let final_score = Self::percent_div(*score, total_final_score);
      OverwatchNodeWeights::<T>::insert(current_epoch, node_id, final_score);
    }

    weight
  }

  // Calculate weights from overwatch reveals
  pub fn calculate_overwatch_weights(epoch: u32) -> Option<u128> {

    let total_stake_balance = TotalOverwatchStake::<T>::get();

    // subnet_id -> (sum_weights, map of overwatch_node_id -> weight)
    let mut subnet_scores: BTreeMap<u32, (u128, BTreeMap<u32, u128>)> = BTreeMap::new();
    for ((subnet_id, overwatch_node_id), weight) in OverwatchReveals::<T>::iter_prefix((epoch,)) {
      let Some(overwatch_node) = OverwatchNodes::<T>::get(overwatch_node_id) else {
        continue
      };
      let stake_balance = AccountOverwatchStake::<T>::get(overwatch_node.hotkey);

      let stake_weight: f64 = stake_balance as f64 / total_stake_balance as f64;
      // let subnet_weight_pow: f64 = Self::pow(stake_weight, 0.5);

      let entry = subnet_scores.entry(subnet_id).or_insert((0, BTreeMap::new()));
      entry.0 += weight;                          // sum all weights for this subnet
      entry.1.insert(overwatch_node_id, weight);  // store each node's weight per subnet
    }

    let mut averages: BTreeMap<u32,u128> = BTreeMap::new();

    for (&subnet_id, (sum_weights, node_weights)) in subnet_scores.iter() {
      let count = node_weights.len() as u128;
      let average = if count > 0 {
        *sum_weights / count
      } else {
        0
      };
      averages.insert(subnet_id, average);
    }


    Some(1)
  }

  // Version with total nodes included
  pub fn calculate_emission_weights(
    subnet_ids: &[u32],
    percentage_factor: u128,
    total_delegate_stake: u128,
    alpha: u128,
  ) -> BTreeMap<u32, u128> {
    let mut weights: BTreeMap<u32, f64> = BTreeMap::new();
    let mut weight_sum: f64 = 0.0;
    let alpha: f64 = Self::get_percent_as_f64(alpha);

    // We avoid division by Zero
    // Subnets must always have at least 1 node to be live in the first place to generate incentives
    // because they are iterated later
    let total_active_nodes = TotalActiveNodes::<T>::get().max(1);

    for subnet_id in subnet_ids {
      let stake = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id) as f64;
      let nodes = TotalActiveSubnetNodes::<T>::get(subnet_id) as f64;

      let stake_ratio = stake / total_delegate_stake as f64;
      let node_ratio = nodes / total_active_nodes.max(1) as f64;

      let weight = Self::pow(stake_ratio + node_ratio, alpha);

      let combined_weight = weight;

      weights.insert(*subnet_id, combined_weight);

      weight_sum += combined_weight;
    }

    let mut stake_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();

    for (subnet_id, weight) in weights {
      let normalized_test = weight / weight_sum;
      let normalized = (weight / weight_sum * percentage_factor as f64) as u128;
      stake_weights_normalized.insert(subnet_id, normalized);
    }

    stake_weights_normalized
  }

  pub fn emission_step(block: u32, current_epoch: u32, subnet_id: u32) -> Weight {
    let mut weight = Weight::zero();
    let db_weight = T::DbWeight::get();

    // Get all subnet weights calculated at the start of the blockchains epoch
    if let Ok(subnet_emission_weights) = FinalSubnetEmissionWeights::<T>::try_get(current_epoch) {
      weight = weight.saturating_add(db_weight.reads(1));

      // Get weight of subnet_id
      if let Some(&subnet_weight) = subnet_emission_weights.weights.get(&subnet_id) {
        // Get elected consensus submission
        if let Some((consensus_submission_data, consensus_submission_weight)) =
          Self::precheck_consensus_submission(subnet_id, current_epoch - 1)
        {
          // Accumulate weight from precheck
          weight = weight.saturating_add(consensus_submission_weight);

          // Calculate rewards
          let (rewards_data, rewards_weight) = Self::calculate_rewards_v2(
            subnet_id,
            subnet_emission_weights.total_issuance,
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
            current_epoch, // used for graduating nodes
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
          weight = weight.saturating_add(T::DbWeight::get().reads(1));
        }

        // --- Elect new validator for the current epoch
        // The current epoch is the start of the subnets epoch
        // We only elect if the subnet has weights, otherwise it isn't activate yet
        // See `calculate_subnet_weights`
        Self::elect_validator_v3(
          subnet_id,
          current_epoch,
          block
        );

        // weight = weight.saturating_add(T::WeightInfo::elect_validator_v3());
      }
    } else {
      // Count DB read even if subnet_emission_weights is missing
      weight = weight.saturating_add(db_weight.reads(1));
    }

    weight
  }

  pub fn handle_subnet_emission_weights(epoch: u32) -> Weight {
    // Get weights
    let (subnet_weights, mut weight): (BTreeMap<u32, u128>, Weight) = Self::calculate_subnet_weights(epoch);

    // Store weights
    if !subnet_weights.is_empty() {
      let data = DistributionData {
        total_issuance: Self::get_epoch_emissions(),
        weights: subnet_weights
      };
      // weight = weight.saturating_add(T::WeightInfo::get_epoch_emissions());
      FinalSubnetEmissionWeights::<T>::insert(epoch, data);
      weight = weight.saturating_add(T::DbWeight::get().writes(1));
    }

    weight
  }

  pub fn calculate_subnet_weights(epoch: u32) 
    -> (BTreeMap<u32, u128>, Weight)
  {
    let mut weight = Weight::zero();
    
    let total_delegate_stake = TotalDelegateStake::<T>::get();
    weight = weight.saturating_add(T::DbWeight::get().reads(1));

    let mut stake_weights: BTreeMap<u32, f64> = BTreeMap::new();
    let mut stake_weight_sum: f64 = 0.0;
    let mut total_subnet_reads = 0u64;

    for (subnet_id, data) in SubnetsData::<T>::iter() {
      total_subnet_reads += 1;
      if data.start_epoch > epoch && data.state != SubnetState::Active {
        continue
      }

      let total_subnet_delegate_stake = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
      weight = weight.saturating_add(T::DbWeight::get().reads(1));
      // 1. Get all weights in f64

      let subnet_weight: f64 = total_subnet_delegate_stake as f64 / total_delegate_stake as f64;
      let subnet_weight_pow: f64 = Self::pow(subnet_weight, 0.5);

      stake_weights.insert(subnet_id, subnet_weight_pow);
      stake_weight_sum += subnet_weight_pow;
      weight = weight.saturating_add(Weight::from_parts(400_000, 0));
    }

    weight = weight.saturating_add(T::DbWeight::get().reads(total_subnet_reads));
    let mut subnet_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();
    let percentage_factor = Self::percentage_factor_as_u128();

    // --- Normalize delegate stake weights from `pow`
    for (subnet_id, subnet_weight) in stake_weights {
      let weight_normalized: u128 = (subnet_weight / stake_weight_sum * percentage_factor as f64) as u128;
      subnet_weights_normalized.insert(subnet_id, weight_normalized);
      weight = weight.saturating_add(Weight::from_parts(400_000, 0));
    }
    
    (subnet_weights_normalized, weight)
  }

  pub fn precheck_consensus_submission(
    subnet_id: u32,
    epoch: u32
  ) -> Option<(ConsensusSubmissionData<T::AccountId>, Weight)> { 
    let mut weight = Weight::zero();
    let submission = match SubnetConsensusSubmission::<T>::try_get(subnet_id, epoch) {
      Ok(submission) => submission,
      Err(()) => return None,
    };
    weight = weight.saturating_add(T::DbWeight::get().reads(1));

    let attestations: u128 = submission.attests.len() as u128;
    let subnet_nodes = submission.subnet_nodes;

    // --- Get all qualified possible attestors
    let validators: Vec<SubnetNode<T::AccountId>> = subnet_nodes.clone()
      .into_iter().filter(
        |subnet_node| subnet_node.has_classification(&SubnetNodeClass::Validator, epoch)
      ).collect();

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
    let weight_sum = submission.data.iter().fold(0, |acc, x| acc.saturating_add(x.score));

    let consensus_data = ConsensusSubmissionData {
      validator_subnet_node_id: submission.validator_id,
      attestation_ratio: attestation_ratio,
      weight_sum: weight_sum,
      data_length: data_length,
      data: submission.data,
      subnet_nodes: subnet_nodes
    };

    Some((consensus_data, weight))
  }

  pub fn calculate_rewards_v2(
    subnet_id: u32,
    overall_rewards: u128,
    emission_weight: u128
  ) -> (RewardsData, Weight) {
    let mut weight = Weight::zero();

    let delegate_stake_rewards_percentage = SubnetDelegateStakeRewardsPercentage::<T>::get(subnet_id);
    let subnet_owner_percentage = SubnetOwnerPercentage::<T>::get();
    weight = weight.saturating_add(T::DbWeight::get().reads(2));

    let overall_subnet_reward: u128 = Self::percent_mul(overall_rewards, emission_weight);

    // --- Get owner rewards
    let subnet_owner_reward: u128 = Self::percent_mul(overall_subnet_reward, subnet_owner_percentage);

    // --- Get subnet rewards minus owner cut
    let subnet_rewards: u128 = overall_subnet_reward.saturating_sub(subnet_owner_reward);

    // --- Get delegators rewards
    let delegate_stake_rewards: u128 = Self::percent_mul(subnet_rewards, delegate_stake_rewards_percentage);

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