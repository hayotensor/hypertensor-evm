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
use sp_runtime::Saturating;
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use frame_support::pallet_prelude::Pays;
use libm::sqrt;
use frame_support::pallet_prelude::Weight;

impl<T: Config> Pallet<T> {
  // TODO: Fix this one to use this one instead of below
  // pub fn calculate_stake_weights(
  //   subnet_ids: &[u32],
  //   percentage_factor: u128,
  //   total_delegate_stake: u128,
  // ) -> BTreeMap<u32, u128> {
  //   let mut raw_weights: BTreeMap<u32, f64> = BTreeMap::new();
  //   let mut weight_sum: f64 = 0.0;
  //   let exponent: f64 = 0.5;

  //   if total_delegate_stake == 0 || subnet_ids.is_empty() {
  //       return BTreeMap::new();
  //   }

  //   for subnet_id in subnet_ids {
  //     let stake = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
  //     if stake == 0 {
  //       continue
  //     }

  //     let ratio = stake as f64 / total_delegate_stake as f64;
  //     // let adjusted_weight = Self::pow(ratio, exponent);
  //     let adjusted_weight: f64 = sqrt(ratio);

  //     raw_weights.insert(*subnet_id, adjusted_weight);
  //     weight_sum += adjusted_weight;
  //   }

  //   let mut normalized: BTreeMap<u32, u128> = BTreeMap::new();
  //   let percentage_factor_f64 = percentage_factor as f64;

  //   for (subnet_id, weight) in raw_weights {
  //     let norm = ((weight / weight_sum) * percentage_factor_f64) as u128;
  //     normalized.insert(subnet_id, norm);
  //   }

  //   normalized
  // }

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

  pub fn calculate_stake_weights(
    subnet_ids: &[u32],
    percentage_factor: u128,
    total_delegate_stake: u128,
  ) -> BTreeMap<u32, u128> {
    let mut stake_weights: BTreeMap<&u32, f64> = BTreeMap::new();
    let mut stake_weight_sum: f64 = 0.0;

    for subnet_id in subnet_ids {
      let total_subnet_delegate_stake = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
      // 1. Get all weights in f64
      // *We later use sqrt that uses floats

      let weight: f64 = total_subnet_delegate_stake as f64 / total_delegate_stake as f64;
      let weight_sqrt: f64 = sqrt(weight);

      stake_weights.insert(subnet_id, weight_sqrt);
      stake_weight_sum += weight_sqrt;
    }

    let mut stake_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();
    let percentage_factor = Self::percentage_factor_as_u128();

    // --- Normalize delegate stake weights from `sqrt`
    for (subnet_id, weight) in stake_weights {
      let weight_normalized: u128 = (weight / stake_weight_sum * percentage_factor as f64) as u128;
      stake_weights_normalized.insert(*subnet_id, weight_normalized);
    }
    return stake_weights_normalized
  }

  pub fn emission_step(block: u32, epoch: u32, subnet_id: u32) -> Weight {
    let mut weight = Weight::zero();

    let subnet_emission_weights = match FinalSubnetEmissionWeights::<T>::try_get(epoch) {
      Ok(subnet_weights) => subnet_weights,
      Err(()) => return weight.saturating_add(T::DbWeight::get().reads(1)),
    };
    weight = weight.saturating_add(T::DbWeight::get().reads(1));

    let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);

    if subnet_weight.is_none() {
      return weight
    }

    let maybe_consensus_submission_data = Self::precheck_consensus_submission(
      subnet_id, epoch
    );

    if maybe_consensus_submission_data.is_none() {
      // Penalize subnet
      return weight
    }

    // Safe unwrap
    let (consensus_submission_data, consensus_submission_weight) = maybe_consensus_submission_data.unwrap();

    let (rewards_data, rewards_weight) = Self::calculate_rewards_v2(
      subnet_id,
      subnet_emission_weights.total_issuance,
      *subnet_weight.unwrap()
    );
    weight = weight.saturating_add(rewards_weight);
    let min_attestation_percentage = MinAttestationPercentage::<T>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<T>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<T>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<T>::get();
    weight = weight.saturating_add(T::DbWeight::get().reads(4));

    let distribute_rewards_weight = Self::distribute_rewards_v2(
      subnet_id,
      block,
      epoch,
      consensus_submission_data,
      rewards_data,
      min_attestation_percentage,
      reputation_increase_factor,
      reputation_decrease_factor,
      min_vast_majority_attestation_percentage,
    );
    weight = weight.saturating_add(distribute_rewards_weight);
    weight
  }

  pub fn emission_step_v2(block: u32, current_epoch: u32, subnet_id: u32) -> Weight {
    let mut weight = Weight::zero();
    let db_weight = T::DbWeight::get();

    // Optional: reward distribution path
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
          let dist_weight = Self::distribute_rewards_v2(
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
        // See `calculate_stake_weights_v2`
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
    let (subnet_weights, mut weight): (BTreeMap<u32, u128>, Weight) = Self::calculate_stake_weights_v2(epoch);

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

  pub fn calculate_stake_weights_v2(epoch: u32) 
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
      // *We later use sqrt that uses floats

      let subnet_weight: f64 = total_subnet_delegate_stake as f64 / total_delegate_stake as f64;
      let subnet_weight_pow: f64 = Self::pow(subnet_weight, 0.5);

      stake_weights.insert(subnet_id, subnet_weight_pow);
      stake_weight_sum += subnet_weight_pow;
      weight = weight.saturating_add(Weight::from_parts(400_000, 0));
    }

    weight = weight.saturating_add(T::DbWeight::get().reads(total_subnet_reads));
    let mut stake_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();
    let percentage_factor = Self::percentage_factor_as_u128();

    // --- Normalize delegate stake weights from `sqrt`
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

  pub fn distribute_rewards_v2(
    subnet_id: u32,
    block: u32,
    current_epoch: u32,
    consensus_submission_data: ConsensusSubmissionData<T::AccountId>, 
    rewards_data: RewardsData, 
    min_attestation_percentage: u128,
    reputation_increase_factor: u128,
    reputation_decrease_factor: u128,
    min_vast_majority_attestation_percentage: u128
  ) -> Weight {
    let mut weight = Weight::zero();

    let queue_epochs = QueueClassificationEpochs::<T>::get(subnet_id);
    let included_epochs = IncludedClassificationEpochs::<T>::get(subnet_id);
    let max_subnet_node_penalties = MaxSubnetNodePenalties::<T>::get(subnet_id);
    weight = weight.saturating_add(T::DbWeight::get().reads(3));

    // --- If under minimum attestation ratio, penalize validator, skip rewards
    if consensus_submission_data.attestation_ratio < min_attestation_percentage {
      SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

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
        current_epoch
      );
      return weight.saturating_add(slash_validator_weight);
    }

    // --- Reward owner
    match SubnetOwner::<T>::try_get(subnet_id) {
      Ok(coldkey) => {
        let subnet_owner_reward_as_currency = Self::u128_to_balance(rewards_data.subnet_owner_reward);
        if subnet_owner_reward_as_currency.is_some() {
          // Add balance to coldkey account
          // An owner may not have a subnet node
          Self::add_balance_to_coldkey_account(
            &coldkey,
            subnet_owner_reward_as_currency.unwrap()
          );
          // weight = weight.saturating_add(T::WeightInfo::add_balance_to_coldkey_account());
        }
      },
      Err(()) => (),
    };
    weight = weight.saturating_add(T::DbWeight::get().reads(1));

    // Iterate each node, emit rewards, graduate, or penalize
    for subnet_node in &consensus_submission_data.subnet_nodes {
      let penalties = SubnetNodePenalties::<T>::get(subnet_id, subnet_node.id);
      weight = weight.saturating_add(T::DbWeight::get().reads(1));

      if penalties + 1 > max_subnet_node_penalties {
        // Remove node if they haven't already
        Self::perform_remove_subnet_node(subnet_id, subnet_node.id);
        // 112_050_000
        // weight = weight.saturating_add(T::WeightInfo::perform_remove_subnet_node());
        continue
      }

      if subnet_node.classification.node_class == SubnetNodeClass::Queue {
        // Queue classified nodes can't be included in consensus data and can't have penalties
        // so we check the class immediately.
        // --- Upgrade to Included if past the queue epochs
        if subnet_node.classification.start_epoch + queue_epochs > current_epoch {
          // Increase class if they exist
          Self::graduate_class(subnet_id, subnet_node.id, current_epoch);
          // weight = weight.saturating_add(T::WeightInfo::graduate_class());
        }
        continue
      }

      let subnet_node_data_find = consensus_submission_data.data
        .iter()
        .find(|data| data.peer_id == subnet_node.peer_id);

      if subnet_node_data_find.is_none() {
        // Not included in consensus, increase
        SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node.id, |n: &mut u32| *n += 1);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
        continue
      } else if penalties != 0 {
        // Is in consensus data, decrease
        // If the validator submits themselves in the data and is successfully attested, this also
        // decreases the validators penalties
        SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node.id, |n: &mut u32| n.saturating_dec());
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
      }

      if subnet_node.classification.node_class == SubnetNodeClass::Included {
        // --- Upgrade to Validator if no penalties
        if penalties == 0 && subnet_node.classification.start_epoch + included_epochs > current_epoch {
          if Self::graduate_class(subnet_id, subnet_node.id, current_epoch) {
            // --- Insert into election slot
            Self::insert_node_into_election_slot(subnet_id, subnet_node.id);
            // weight = weight.saturating_add(T::WeightInfo::insert_node_into_election_slot());
          }
        }
        continue
      }

      // Safely unwrap node_weight, we already confirmed it's not None
      let node_weight = subnet_node_data_find.unwrap().score;

      if node_weight == 0 {
        continue
      }

      // --- Calculate node_weight percentage of peer versus the weighted sum
      let score_ratio: u128 = Self::percent_div(node_weight, consensus_submission_data.weight_sum);

      // --- Calculate node_weight percentage of total subnet generated epoch rewards
      let mut account_reward: u128 = Self::percent_mul(score_ratio, rewards_data.subnet_node_rewards);

      // --- Increase reward if validator
      if subnet_node.id == consensus_submission_data.validator_subnet_node_id {
        account_reward += Self::get_validator_reward(consensus_submission_data.attestation_ratio);
        // Add get_validator_reward (At least 1 read, up to 2)
        weight = weight.saturating_add(T::DbWeight::get().reads(2));
        match HotkeyOwner::<T>::try_get(&subnet_node.hotkey) {
          Ok(coldkey) => {
            Self::increase_coldkey_reputation(
              coldkey,
              consensus_submission_data.attestation_ratio, 
              min_attestation_percentage, 
              reputation_increase_factor,
              current_epoch
            );
            // weight = weight.saturating_add(T::WeightInfo::increase_coldkey_reputation());
          },
          Err(()) => (),
        };
        // Add HotkeyOwner read
        weight = weight.saturating_add(T::DbWeight::get().reads(1));
      }
      
      // --- Skip if no rewards to give
      // Unlikely to happen
      if account_reward == 0 {
        continue
      }
      if subnet_node.delegate_reward_rate != 0 {
        // --- Ensure users are staked to subnet node
        let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node.id);
        weight = weight.saturating_add(T::DbWeight::get().reads(1));
        if total_node_delegated_stake_shares != 0 {
          let node_delegate_reward = Self::percent_mul(account_reward, subnet_node.delegate_reward_rate);
          account_reward = account_reward - node_delegate_reward;
          Self::do_increase_node_delegate_stake(
            subnet_id,
            subnet_node.id,
            node_delegate_reward,
          );
          // weight = weight.saturating_add(T::WeightInfo::do_increase_node_delegate_stake());
        }
      }

      // --- Increase account stake and emit event
      Self::increase_account_stake(
        &subnet_node.hotkey,
        subnet_id, 
        account_reward,
      );
      // weight = weight.saturating_add(T::WeightInfo::increase_account_stake());
    }
    // --- Increase the delegate stake pool balance
    Self::do_increase_delegate_stake(
      subnet_id,
      rewards_data.delegate_stake_rewards,
    );
    // weight = weight.saturating_add(T::WeightInfo::do_increase_delegate_stake());

    weight
  }

}