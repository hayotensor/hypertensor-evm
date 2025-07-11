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
        // See `calculate_stake_weights`
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
    let (subnet_weights, mut weight): (BTreeMap<u32, u128>, Weight) = Self::calculate_stake_weights(epoch);

    // Store weights
    if !subnet_weights.is_empty() {
      let data = DistributionData {
        total_issuance: Self::get_epoch_emissions(epoch),
        weights: subnet_weights
      };
      // weight = weight.saturating_add(T::WeightInfo::get_epoch_emissions());
      FinalSubnetEmissionWeights::<T>::insert(epoch, data);
      weight = weight.saturating_add(T::DbWeight::get().writes(1));
    }

    weight
  }

  pub fn calculate_stake_weights(epoch: u32) 
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
    let mut stake_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();
    let percentage_factor = Self::percentage_factor_as_u128();

    // --- Normalize delegate stake weights from `pow`
    for (subnet_id, subnet_weight) in stake_weights {
      let weight_normalized: u128 = (subnet_weight / stake_weight_sum * percentage_factor as f64) as u128;
      stake_weights_normalized.insert(subnet_id, weight_normalized);
      weight = weight.saturating_add(Weight::from_parts(400_000, 0));
    }
    
    (stake_weights_normalized, weight)
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