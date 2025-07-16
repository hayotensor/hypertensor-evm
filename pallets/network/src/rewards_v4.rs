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
use libm::sqrt;
use frame_support::pallet_prelude::Weight;

impl<T: Config> Pallet<T> {
  pub fn distribute_rewards(
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

    let queue_epochs = IdleClassificationEpochs::<T>::get(subnet_id);
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

    //
    // --- We are now in consensus
    //

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

      if subnet_node.classification.node_class == SubnetNodeClass::Idle {
        // Idle classified nodes can't be included in consensus data and can't have penalties
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
        .find(|data| data.subnet_node_id == subnet_node.id);

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