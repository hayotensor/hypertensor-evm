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

  pub fn reward_subnets_v2(block: u32, epoch: u32) -> DispatchResultWithPostInfo {
    // --- Get total rewards for this epoch
    // 1. Epoch emissions
    // 2. Epoch burn
    // 3. Foundation emissions
    let rewards: u128 = Self::get_epoch_emissions(epoch);

    let subnets: Vec<_> = SubnetsData::<T>::iter()
      .filter(|(_, subnet)| subnet.state == SubnetState::Active)
      .collect();

    let total_delegate_stake = TotalDelegateStake::<T>::get();

    let subnet_ids: Vec<u32> = subnets.iter().map(|(id, _)| *id).collect();

    let percentage_factor = Self::percentage_factor_as_u128();

    let stake_weights_normalized: BTreeMap<u32, u128> = Self::calculate_stake_weights(
      &subnet_ids,
      percentage_factor,
      total_delegate_stake,
    );

    let subnet_owner_percentage = SubnetOwnerPercentage::<T>::get();
    let min_attestation_percentage = MinAttestationPercentage::<T>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<T>::get();
    let min_subnet_nodes = MinSubnetNodes::<T>::get();
    let node_attestation_removal_threshold = NodeAttestationRemovalThreshold::<T>::get();
    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<T>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<T>::get();

    for (subnet_id, _) in &subnets {
      let mut attestation_percentage: u128 = 0;

      // --- Get subnet validator submission
      // --- - Run rewards logic
      // --- Otherwise, check if validator exists since they didn't submit incentives consensus
      // --- - Penalize and slash validator if existed
      if let Ok(mut submission) = SubnetConsensusSubmission::<T>::try_get(subnet_id, epoch) {
        // --- Get overall subnet rewards
        let weight: u128 = match stake_weights_normalized.get(&subnet_id) {
          Some(weight) => {
            if weight == &0 {
              continue
            }
            *weight
          },
          None => continue,
        };

        let delegate_stake_rewards_percentage = SubnetDelegateStakeRewardsPercentage::<T>::get(subnet_id);

        let overall_subnet_reward: u128 = Self::percent_mul(rewards, weight);

        // --- Get owner rewards
        let subnet_owner_reward: u128 = Self::percent_mul(overall_subnet_reward, subnet_owner_percentage);

        // --- Get subnet rewards minus owner cut
        let subnet_reward: u128 = overall_subnet_reward.saturating_sub(subnet_owner_reward);

        // --- Get delegators rewards
        let delegate_stake_reward: u128 = Self::percent_mul(subnet_reward, delegate_stake_rewards_percentage);

        // --- Get subnet nodes rewards total
        let subnet_node_reward: u128 = subnet_reward.saturating_sub(delegate_stake_reward);

        // --- Get subnet nodes count to check against attestation count and make sure min nodes are present during time of rewards
        let subnet_nodes: Vec<T::AccountId> = Self::get_classified_hotkeys(*subnet_id, &SubnetNodeClass::Validator, epoch);
        let subnet_node_count = subnet_nodes.len() as u128;

        // --- Ensure nodes are at min requirement to continue rewards operations
        if subnet_node_count < min_subnet_nodes as u128 {
          // We don't give penalties here because they will be given in the next step operation when selecting a new
          // validator
          continue
        }

        let attestations: u128 = submission.attests.len() as u128;
        attestation_percentage = Self::percent_div(attestations, subnet_node_count);

        // Redundant
        // When subnet nodes exit, the consensus data is updated to remove them from it
        if attestation_percentage > percentage_factor {
          attestation_percentage = percentage_factor;
        }
        
        let validator_subnet_node_id: u32 = submission.validator_id;
        let data_len = submission.data.len();
        // let is_submission_empty = (data_len as u32) < min_subnet_nodes;
        // let not_enough_attestations = attestation_percentage < min_attestation_percentage;


        // ────────────────────────────────
        // Case 1: Empty submission
        // ────────────────────────────────
        if (data_len as u32) < min_subnet_nodes {
          // --- Subnet no longer submitting consensus
          //     Increase the penalty count
          SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
          
          // Check if the attestation percentage is below the "vast majority" threshold
          // If validator submits nothing, we require that vast majority to agree with this
          // It can reference the subnet possibly being in a broken stat, requiring maintenance
          // If so, consensus is skipped
          if attestation_percentage < min_vast_majority_attestation_percentage {
            // If the attestation percentage is also below the minimum required threshold, slash the validator
            if attestation_percentage < min_attestation_percentage {
              Self::slash_validator(
                *subnet_id, 
                validator_subnet_node_id, 
                attestation_percentage,
                min_attestation_percentage,
                reputation_decrease_factor,
                block,
                epoch
              );
            }
            // Skip further execution and continue to the next iteration
            continue;
          }
        }

        // ───────────────────────────────────────────────────────
        // Case 2: Submission present, but not enough attestations
        // ───────────────────────────────────────────────────────
        if attestation_percentage < min_attestation_percentage {
          // --- Slash validator and increase penalty score
          Self::slash_validator(
            *subnet_id, 
            validator_subnet_node_id, 
            attestation_percentage,
            min_attestation_percentage,
            reputation_decrease_factor,
            block,
            epoch
          );

          // --- Attestation not successful, move on to next subnet
          continue
        }

        // Data is None, nothing to do
        if data_len == 0 {
          continue
        }

        // ==============================================
        // Subnet is in consensus and proceeds to rewards
        // ==============================================

        // --- Deposit owners rewards
        // `match` to ensure owner not renounced
        match SubnetOwner::<T>::try_get(subnet_id) {
          Ok(coldkey) => {
            let subnet_owner_reward_as_currency = Self::u128_to_balance(subnet_owner_reward);
            if subnet_owner_reward_as_currency.is_some() {
              Self::add_balance_to_coldkey_account(
                &coldkey,
                subnet_owner_reward_as_currency.unwrap()
              );    
            }
          },
          Err(()) => (),
        };

        // --- Get sum of subnet total scores for use of divvying rewards
        let sum = submission.data.iter().fold(0, |acc, x| acc.saturating_add(x.score));

        let max_subnet_node_penalties = MaxSubnetNodePenalties::<T>::get(subnet_id);
        let queue_epochs = QueueClassificationEpochs::<T>::get(subnet_id);
        let included_epochs = IncludedClassificationEpochs::<T>::get(subnet_id);

        let min_stake = SubnetMinStakeBalance::<T>::get(subnet_id);

        for (subnet_node_id, subnet_node) in SubnetNodesData::<T>::iter_prefix(subnet_id) {
          // Redundant
          let hotkey: T::AccountId = match SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id) {
            Ok(hotkey) => hotkey,
            Err(()) => continue,
          };

          // --- If subnet is below the minimum required subnet stake balance, remoe
          // This is only possible if the owner increases the stake balance
          let stake_balance = AccountSubnetStake::<T>::get(&hotkey, subnet_id);
          if stake_balance < min_stake {
            Self::perform_remove_subnet_node(block, *subnet_id, subnet_node_id);
          }

          // Note: Only ``Included`` or above nodes can get emissions
          if subnet_node.classification.node_class == SubnetNodeClass::Queue {
            // --- Automatically upgrade to Included if activated into Queue class
            if subnet_node.classification.start_epoch + queue_epochs > epoch {
              Self::increase_class(*subnet_id, subnet_node_id, epoch);
            }
            continue
          }

          // --- At this point, all nodes are >= `SubnetNodeClass::Included` and can be included in consensus data and receive rewards

          let peer_id: PeerId = subnet_node.peer_id;

          // Find the node in the consensus data
          let subnet_node_data_find = submission.data
            .iter()
            .find(|data| data.peer_id == peer_id);
    
          let penalties = SubnetNodePenalties::<T>::get(subnet_id, subnet_node_id);

          // --- Node is not in the consensus data (>66% agree on consensus data)
          if subnet_node_data_find.is_none() {
            // --- Mutate nodes penalties count if not in consensus
            SubnetNodePenalties::<T>::insert(subnet_id, subnet_node_id, penalties + 1);

            // --- To be removed or increase penalty count, the node consensus threshold must be reached
            // The threshold is a super majority attestation
            if attestation_percentage > node_attestation_removal_threshold {
              // We don't slash nodes for not being in consensus
              // A node can be removed for any reason such as shutting their node down and may not be due to dishonesty
              // If subnet validators want to remove and slash a node, they can use the proposals mechanism

              // --- Ensure maximum sequential removal consensus threshold is reached
              // We make sure the super majority are in agreeance to remove someone
              // TODO: Check the size of subnet and scale it from there
              if penalties + 1 > max_subnet_node_penalties {
                // --- Increase account penalty count
                Self::perform_remove_subnet_node(block, *subnet_id, subnet_node_id);
              }
            }

            continue
          }
          
          // --- At this point, the subnet node is in the consensus data

          // --- Check if node is SubnetNodeClass::Included
          // 
          // By this point, node is validated, update to Validator if they have no penalties
          // Otherwise, `saturate_dec` penalties
          let is_included = subnet_node.classification.node_class == SubnetNodeClass::Included;
          if is_included && penalties == 0 {
            // --- Upgrade to Validator
            if subnet_node.classification.start_epoch + included_epochs > epoch {
              Self::increase_class(*subnet_id, subnet_node_id, epoch);
            }
            continue
          } else if is_included && penalties != 0 {
            // --- Decrease subnet node penalty count by one if in consensus and attested consensus
            SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node_id, |n: &mut u32| n.saturating_dec());
            continue
          }

          // --- At this point, the subnet node is Validator AND included in consensus data

          // --- If subnet node does not attest a super majority attested era, we penalize and skip them
          //
          // TODO: Vote on removal of feature
          //

          // Didn't attest?
          if !submission.attests.contains_key(&subnet_node_id) {
            // Vast majority attested, and it did not
            if attestation_percentage > min_vast_majority_attestation_percentage {
              // --- Penalize on vast majority only
              SubnetNodePenalties::<T>::insert(subnet_id, subnet_node_id, penalties + 1);

              // Skip?
              // continue
            }
          }

          let subnet_node_data: SubnetNodeConsensusData = subnet_node_data_find.unwrap().clone();

          let score = subnet_node_data.score;

          // --- Validators are allowed to submit scores of 0
          // This is useful if a subnet wants to keep a node around but not give them rewards
          // This can be used in scenarios when the max subnet nodes are reached and they don't
          // want to kick them out as a way to have a waitlist.
          //
          // Or
          //
          // If a node is scored on latter epochs, such as if a subnet uses a commit-reveal
          // over multiple epochs and must score them based on the reveal
          if score == 0 {
            continue
          }

          // --- Decrease subnet node penalty count by one if in consensus and attested consensus
          // Don't hit the db unless we have to
          if penalties != 0 {
            SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node_id, |n: &mut u32| n.saturating_dec());
          }

          // --- Calculate score percentage of peer versus sum
          let score_percentage: u128 = Self::percent_div(subnet_node_data.score, sum as u128);

          // --- Calculate score percentage of total subnet generated epoch rewards
          let mut account_reward: u128 = Self::percent_mul(score_percentage, subnet_node_reward);

          // --- Increase reward if validator
          if subnet_node_id == validator_subnet_node_id {
            account_reward += Self::get_validator_reward(attestation_percentage);
            match HotkeyOwner::<T>::try_get(&hotkey) {
              Ok(coldkey) => {
                Self::increase_coldkey_reputation(
                  coldkey,
                  attestation_percentage, 
                  min_attestation_percentage, 
                  reputation_increase_factor,
                  epoch
                );
              },
              Err(()) => continue,
            };
          }
          
          // --- Skip if no rewards to give
          // Unlikely to happen
          if account_reward == 0 {
            continue
          }

          if subnet_node.delegate_reward_rate != 0 {
            // --- Ensure users are staked to subnet node
            let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);
            if total_node_delegated_stake_shares != 0 {
              let node_delegate_reward = Self::percent_mul(account_reward, subnet_node.delegate_reward_rate);
              account_reward = account_reward - node_delegate_reward;
              Self::do_increase_node_delegate_stake(
                *subnet_id,
                subnet_node_id,
                node_delegate_reward,
              );  
            }
          }

          // --- Increase account stake and emit event
          Self::increase_account_stake(
            &hotkey,
            *subnet_id, 
            account_reward,
          );
        }
        // --- Portion of rewards to delegate stakers
        Self::do_increase_delegate_stake(
          *subnet_id,
          delegate_stake_reward,
        );

        // --- Increment down subnet penalty score on successful epochs if result were greater than or equal to the min required nodes
        if data_len as u32 >= min_subnet_nodes {
          SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());
        }
      } else if let Ok(validator_id) = SubnetElectedValidator::<T>::try_get(subnet_id, epoch) {
        // --- If a validator has been chosen that means they are supposed to be submitting consensus data
        // --- If there is no submission but validator chosen, increase penalty on subnet and validator
        // --- Increase the penalty count for the subnet
        // The next validator on the next epoch can increment the penalty score down
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

        // NOTE:
        //  Each subnet increases the penalty score if they don't have the minimum subnet nodes required by the time
        //  the subnet is enabled for emissions. This happens by the blockchain validator before choosing the subnet validator

        // If validator didn't submit anything, then slash
        // Even if a subnet is in a broken state, the chosen validator must submit blank data
        Self::slash_validator(
          *subnet_id, 
          validator_id, 
          0,
          min_attestation_percentage,
          reputation_decrease_factor,
          block,
          epoch
        );
      }
      // TODO: Get benchmark for removing max subnets in one epoch to ensure does not surpass max weights

      Self::deposit_event(
        Event::RewardResult { 
          subnet_id: *subnet_id, 
          attestation_percentage: attestation_percentage, 
        }
      );

      // --- If subnet is past its max penalty count, remove
      let subnet_penalty_count = SubnetPenaltyCount::<T>::get(subnet_id);
      if subnet_penalty_count > max_subnet_penalty_count {
        Self::do_remove_subnet(
          *subnet_id,
          SubnetRemovalReason::MaxPenalties,
        );
      }
    }

    Ok(None.into())
  }

  pub fn emission_step(block: u32, epoch: u32) -> Weight {
    let mut weight = Weight::zero();
    let subnet_emission_weights = match FinalSubnetEmissionWeights::<T>::try_get(epoch) {
      Ok(subnet_weights) => subnet_weights,
      Err(()) => return weight,
    };
    let min_attestation_percentage = MinAttestationPercentage::<T>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<T>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<T>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<T>::get();
    weight = weight.saturating_add(T::DbWeight::get().reads(5));

    let overall_rewards: u128 = Self::get_epoch_emissions(epoch);
    // TODO: Add weights for `get_epoch_emissions`

    for (subnet_id, subnet_weight) in subnet_emission_weights {
      let maybe_consensus_submission_data = Self::precheck_consensus_submission(
        subnet_id, epoch
      );
      if let Some((consensus_submission_data, consensus_submission_weight)) = maybe_consensus_submission_data {
        if let Some((rewards_data, rewards_weight)) = Self::calculate_rewards_v2(
          subnet_id,
          overall_rewards,
          subnet_weight
        ) {
          weight = weight.saturating_add(rewards_weight);
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
        } else {
          SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

          Self::slash_validator(
            subnet_id, 
            consensus_submission_data.validator_subnet_node_id, 
            consensus_submission_data.attestation_ratio,
            min_attestation_percentage,
            reputation_decrease_factor,
            block,
            epoch
          );

          continue
        }
      } else {

      }
    }

    weight
  }

  pub fn handle_subnet_emission_weights(epoch: u32) -> Weight {
    // Get weights
    let (subnet_weights, mut weight): (BTreeMap<u32, u128>, Weight) = Self::calculate_stake_weights_v2(epoch);

    // Store weights
    if !subnet_weights.is_empty() {
      FinalSubnetEmissionWeights::<T>::insert(epoch, subnet_weights);
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
      weight = weight.saturating_add(Weight::from_parts(387_000, 0));
    }

    weight = weight.saturating_add(T::DbWeight::get().reads(total_subnet_reads));
    let mut stake_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();
    let percentage_factor = Self::percentage_factor_as_u128();

    // --- Normalize delegate stake weights from `sqrt`
    for (subnet_id, subnet_weight) in stake_weights {
      let weight_normalized: u128 = (subnet_weight / stake_weight_sum * percentage_factor as f64) as u128;
      stake_weights_normalized.insert(subnet_id, weight_normalized);
      weight = weight.saturating_add(Weight::from_parts(383_000, 0));
    }
    
    (stake_weights_normalized, weight)
  }

  // pub fn generate_rewards_for_subnet(
  //   block: u32, 
  //   epoch: u32, 
  //   subnet_id: u32, 
  //   overall_rewards: u128,
  //   stake_weights_normalized: BTreeMap<u32, u128>
  // ) -> Weight {
  //   let mut weight = Weight::zero();
  //   let node_attestation_removal_threshold = NodeAttestationRemovalThreshold::<T>::get();
  //   let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
  //   let reputation_increase_factor = ReputationIncreaseFactor::<T>::get();
  //   let reputation_decrease_factor = ReputationDecreaseFactor::<T>::get();
  //   weight = weight.saturating_add(T::DbWeight::get().reads(4));

  //   let percentage_factor = Self::percentage_factor_as_u128();
  //   let mut attestation_percentage: u128 = 0;

  //   let subnet_nodes: Vec<SubnetNode<T::AccountId>> = Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);
  //   // weight = weight.saturating_add(T::WeightInfo::get_classified_subnet_nodes());

  //   let subnet_node_count = subnet_nodes.len() as u128;

  //   let maybe_consensus_submission_data = Self::precheck_consensus_submission(
  //     subnet_id, epoch
  //   );
   
  //   if let Some((consensus_submission_data, consensus_submission_weight)) = maybe_consensus_submission_data {
  //     weight = weight.saturating_add(consensus_submission_weight);
  //     if let Some((rewards_data, rewards_weight)) = Self::calculate_rewards(
  //       subnet_id,
  //       overall_rewards,
  //       stake_weights_normalized
  //     ) {
  //       weight = weight.saturating_add(rewards_weight);
  //       let distribute_rewards_weight = Self::distribute_rewards(
  //         subnet_id,
  //         block,
  //         epoch,
  //         consensus_submission_data,
  //         rewards_data,
  //         subnet_nodes,
  //       );
  //       weight = weight.saturating_add(distribute_rewards_weight);
  //     } else {
  //       return weight
  //     }
  //   } else {
  //     return weight
  //   }

  //   weight
  // }

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
    let included_subnet_nodes = submission.included_subnet_nodes;

    // let included_subnet_nodes = submission.included_subnet_nodes;

    // --- Get all qualified possible attestors
    let validators: Vec<SubnetNode<T::AccountId>> = included_subnet_nodes.clone()
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
      included_subnet_nodes: included_subnet_nodes
    };

    Some((consensus_data, weight))
  }

  pub fn calculate_rewards(
    subnet_id: u32,
    overall_rewards: u128,
    stake_weights_normalized: BTreeMap<u32, u128>
  ) -> Option<(RewardsData, Weight)>  {
    let mut weight = Weight::zero();
    let stake_weight: u128 = match stake_weights_normalized.get(&subnet_id) {
      Some(stake_weight) => {
        if stake_weight == &0 {
          return None
        }
        *stake_weight
      },
      None => return None
    };

    let delegate_stake_rewards_percentage = SubnetDelegateStakeRewardsPercentage::<T>::get(subnet_id);
    let subnet_owner_percentage = SubnetOwnerPercentage::<T>::get();
    weight = weight.saturating_add(T::DbWeight::get().reads(2));

    let overall_subnet_reward: u128 = Self::percent_mul(overall_rewards, stake_weight);

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

    Some((rewards_data, weight))
  }

  pub fn calculate_rewards_v2(
    subnet_id: u32,
    overall_rewards: u128,
    emission_weight: u128
  ) -> Option<(RewardsData, Weight)> {
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

    Some((rewards_data, weight))
  }

  pub fn distribute_rewards(
    subnet_id: u32,
    block: u32,
    epoch: u32,
    consensus_submission_data: ConsensusSubmissionData<T::AccountId>, 
    rewards_data: RewardsData, 
    subnet_nodes: Vec<SubnetNode<T::AccountId>>,
  ) -> Weight {
    let mut weight = Weight::zero();

    let min_attestation_percentage = MinAttestationPercentage::<T>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<T>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<T>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<T>::get();
    let queue_epochs = QueueClassificationEpochs::<T>::get(subnet_id);
    let max_subnet_node_penalties = MaxSubnetNodePenalties::<T>::get(subnet_id);
    weight = weight.saturating_add(T::DbWeight::get().reads(6));

    for subnet_node in &subnet_nodes {
      if subnet_node.classification.node_class == SubnetNodeClass::Queue {
        // --- Upgrade to Included if past the queue epochs
        if subnet_node.classification.start_epoch + queue_epochs > epoch {
          Self::increase_class(subnet_id, subnet_node.id, epoch);
          // weight = weight.saturating_add(T::WeightInfo::increase_class());
        }
        continue
      }

      let subnet_node_data_find = consensus_submission_data.data
        .iter()
        .find(|data| data.peer_id == subnet_node.peer_id);

      let penalties = SubnetNodePenalties::<T>::get(subnet_id, subnet_node.id);
      weight = weight.saturating_add(T::DbWeight::get().reads(1));

      if penalties + 1 > max_subnet_node_penalties {
        Self::perform_remove_subnet_node(block, subnet_id, subnet_node.id);
        // 112_050_000
        // weight = weight.saturating_add(T::WeightInfo::perform_remove_subnet_node());
        continue
      }

      if subnet_node_data_find.is_none() {
        // Not included in consensus, increase
        SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node.id, |n: &mut u32| *n += 1);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
        continue
      } else if penalties != 0 {
        // Included in consensus, decrease
        SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node.id, |n: &mut u32| n.saturating_dec());
      }

      // Safely unwrap node_weight, we already confirmed it's not None
      let node_weight = subnet_node_data_find.unwrap().score;

      if node_weight == 0 {
        continue
      }

      // --- Calculate node_weight percentage of peer versus the weighted sum
      let score_percentage: u128 = Self::percent_div(node_weight, consensus_submission_data.weight_sum);

      // --- Calculate node_weight percentage of total subnet generated epoch rewards
      let mut account_reward: u128 = Self::percent_mul(score_percentage, rewards_data.subnet_node_rewards);

      // --- Increase reward if validator
      if subnet_node.id == consensus_submission_data.validator_subnet_node_id {
        account_reward += Self::get_validator_reward(consensus_submission_data.attestation_ratio);
        // Add get_validator_reward (At least 1 read, expects 2)
        weight = weight.saturating_add(T::DbWeight::get().reads(2));
        match HotkeyOwner::<T>::try_get(&subnet_node.hotkey) {
          Ok(coldkey) => {
            Self::increase_coldkey_reputation(
              coldkey,
              consensus_submission_data.attestation_ratio, 
              min_attestation_percentage, 
              reputation_increase_factor,
              epoch
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
    Self::do_increase_delegate_stake(
      subnet_id,
      rewards_data.delegate_stake_rewards,
    );
    // weight = weight.saturating_add(T::WeightInfo::do_increase_delegate_stake());

    weight
  }

  pub fn distribute_rewards_v2(
    subnet_id: u32,
    block: u32,
    epoch: u32,
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
    weight = weight.saturating_add(T::DbWeight::get().reads(2));

    // --- If under minimum attestation ratio, penalize validator, skip rewards
    if consensus_submission_data.attestation_ratio < min_attestation_percentage {
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

        Self::slash_validator(
          subnet_id, 
          consensus_submission_data.validator_subnet_node_id, 
          consensus_submission_data.attestation_ratio,
          min_attestation_percentage,
          reputation_decrease_factor,
          block,
          epoch
        );
        return weight
    }

    // Iterate each node, emit rewards, graduate, or penalize
    for subnet_node in &consensus_submission_data.included_subnet_nodes {
      if subnet_node.classification.node_class == SubnetNodeClass::Queue {
        // --- Upgrade to Included if past the queue epochs
        if subnet_node.classification.start_epoch + queue_epochs > epoch {
          Self::increase_class(subnet_id, subnet_node.id, epoch);
          // weight = weight.saturating_add(T::WeightInfo::increase_class());
        }
        continue
      }

      let subnet_node_data_find = consensus_submission_data.data
        .iter()
        .find(|data| data.peer_id == subnet_node.peer_id);

      let penalties = SubnetNodePenalties::<T>::get(subnet_id, subnet_node.id);
      weight = weight.saturating_add(T::DbWeight::get().reads(1));

      if penalties + 1 > max_subnet_node_penalties {
        Self::perform_remove_subnet_node(block, subnet_id, subnet_node.id);
        // 112_050_000
        // weight = weight.saturating_add(T::WeightInfo::perform_remove_subnet_node());
        continue
      }

      if subnet_node_data_find.is_none() {
        // Not included in consensus, increase
        SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node.id, |n: &mut u32| *n += 1);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
        continue
      } else if penalties != 0 {
        // Is in consensus data, decrease
        SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node.id, |n: &mut u32| n.saturating_dec());
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
      }

      let is_included = subnet_node.classification.node_class == SubnetNodeClass::Included;
      if is_included && penalties == 0 {
        // --- Upgrade to Validator
        if subnet_node.classification.start_epoch + included_epochs > epoch {
          Self::increase_class(subnet_id, subnet_node.id, epoch);
          // weight = weight.saturating_add(T::WeightInfo::increase_class());
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
        // Add get_validator_reward (At least 1 read, expects 2)
        weight = weight.saturating_add(T::DbWeight::get().reads(2));
        match HotkeyOwner::<T>::try_get(&subnet_node.hotkey) {
          Ok(coldkey) => {
            Self::increase_coldkey_reputation(
              coldkey,
              consensus_submission_data.attestation_ratio, 
              min_attestation_percentage, 
              reputation_increase_factor,
              epoch
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