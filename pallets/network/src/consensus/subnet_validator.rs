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
  /// Submit subnet scores per subnet node
  /// Validator of the epoch receives rewards when attestation passes consensus
  pub fn do_validate(
    subnet_id: u32, 
    hotkey: T::AccountId,
    mut data: Vec<SubnetNodeConsensusData>,
    args: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
  ) -> DispatchResultWithPostInfo {
    // The validator is elected for the next blockchain epoch where rewards will be distributed.
    // Each subnet epoch overlaps with the blockchains epochs, and can submit consensus data for epoch
    // 2 on epoch 1 (if after slot) or 2 (if before slot).
    // If a subnet is on slot 3 of 5 slots, we make sure it can submit on the current blockchains epoch.
    // We use n+1 on the slots offset epoch
    let subnet_epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);

    // --- Ensure current subnet validator by its hotkey
    let validator_id = SubnetElectedValidator::<T>::get(subnet_id, subnet_epoch).ok_or(Error::<T>::InvalidValidator)?;

    // --- If hotkey is hotkey, ensure it matches validator, otherwise if coldkey -> get hotkey
    // If the epoch is 0, this will break
    ensure!(
      SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id) == Some(hotkey.clone()),
      Error::<T>::InvalidValidator
    );

    ensure!(
      AccountSubnetStake::<T>::get(&hotkey, subnet_id) >= SubnetMinStakeBalance::<T>::get(subnet_id),
      Error::<T>::MinStakeNotReached
    );

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
    data.retain(|x| {
      match SubnetNodesData::<T>::try_get(
        subnet_id, 
        x.subnet_node_id
      ) {
        Ok(subnet_node) => subnet_node.has_classification(&SubnetNodeClass::Included, subnet_epoch),
        Err(()) => false,
      }
    });

    // --- Ensure overflow sum fails
    data.iter().try_fold(0u128, |acc, node| {
      acc.checked_add(node.score).ok_or(Error::<T>::ScoreOverflow)
    })?;
    
    let block: u32 = Self::get_current_block_as_u32();

    // --- Validator auto-attests the epoch
    let attests: BTreeMap<u32, u32> = BTreeMap::from([(validator_id, block)]);

    // --- Get all (activated) Idle + consensus-eligible nodes
    // We get this here instead of in the rewards distribution to handle block weight more efficiently
    let subnet_nodes: Vec<SubnetNode<T::AccountId>> = Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Idle, subnet_epoch);
    let subnet_nodes_count = subnet_nodes.len();

    let consensus_data: ConsensusData<T::AccountId> = ConsensusData {
      validator_id: validator_id,
      attests: attests,
      subnet_nodes: subnet_nodes,
      data: data,
      args: args,
    };

    SubnetConsensusSubmission::<T>::insert(subnet_id, subnet_epoch, consensus_data);
  
    Self::deposit_event(
      Event::ValidatorSubmission { 
        subnet_id: subnet_id, 
        account_id: hotkey, 
        epoch: subnet_epoch,
      }
    );

    Ok(Pays::No.into())
  }

  /// Attest validator subnet rewards data
  // Nodes must attest data to receive rewards
  pub fn do_attest(
    subnet_id: u32, 
    hotkey: T::AccountId,
  ) -> DispatchResultWithPostInfo {
    let subnet_epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);

    // --- Ensure subnet node exists under hotkey
    let subnet_node_id = match HotkeySubnetNodeId::<T>::try_get(
      subnet_id, 
      &hotkey
    ) {
      Ok(subnet_node_id) => subnet_node_id,
      Err(()) => return Err(Error::<T>::SubnetNodeNotExist.into()),
    };

    // --- Ensure node classified to attest
    match SubnetNodesData::<T>::try_get(
      subnet_id, 
      subnet_node_id
    ) {
      Ok(subnet_node) => subnet_node.has_classification(&SubnetNodeClass::Validator, subnet_epoch),
      Err(()) => return Err(Error::<T>::SubnetNodeNotExist.into()),
    };

    ensure!(
      AccountSubnetStake::<T>::get(&hotkey, subnet_id) >= SubnetMinStakeBalance::<T>::get(subnet_id),
      Error::<T>::MinStakeNotReached
    );

    let block: u32 = Self::get_current_block_as_u32();

    SubnetConsensusSubmission::<T>::try_mutate_exists(
      subnet_id,
      subnet_epoch,
      |maybe_params| -> DispatchResult {
        let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetConsensusSubmission)?;

        // --- Double check in inclusion list
        // Redundant
        // let included_subnet_nodes = &params.included_subnet_nodes;
        // ensure!(included_subnet_nodes.get(&subnet_node_id), Error::<T>::AlreadyAttested);

        let mut attests = &mut params.attests;

        ensure!(attests.insert(subnet_node_id, block) == None, Error::<T>::AlreadyAttested);

        params.attests = attests.clone();
        Ok(())
      }
    )?;

    Self::deposit_event(
      Event::Attestation { 
        subnet_id: subnet_id, 
        account_id: hotkey, 
        epoch: subnet_epoch,
      }
    );

    Ok(Pays::No.into())
  }

  // pub fn elect_validator(
  //   block: u32,
  //   subnet_id: u32,
  //   subnet_node_ids: Vec<u32>,
  //   min_subnet_nodes: u32,
  //   epoch: u32,
  // ) {
  //   // TODO: Make sure this is only called if subnet is activated and on the following epoch
    
  //   // Redundant
  //   // If validator already chosen, then return
  //   if let Ok(validator_id) = SubnetElectedValidator::<T>::try_get(subnet_id, epoch) {
  //     return
  //   }

  //   let subnet_nodes_len = subnet_node_ids.len();
    
  //   // --- Ensure min subnet peers that are submittable are at least the minimum required
  //   // --- Consensus cannot begin until this minimum is reached
  //   // --- If not min subnet peers count then accountant isn't needed
  //   if (subnet_nodes_len as u32) < min_subnet_nodes {
  //     return
  //   }

  //   // --- n-1 to get 0 index in the randomization
  //   let rand_index = Self::get_random_number_with_max(subnet_nodes_len as u32, block as u32);

  //   // --- Choose random accountant from eligible accounts
  //   let validator: &u32 = &subnet_node_ids[rand_index as usize];

  //   // --- Insert validator for next epoch
  //   SubnetElectedValidator::<T>::insert(subnet_id, epoch, validator);
  // }

  pub fn elect_validator_v2(
    subnet_id: u32,
    epoch: u32,
    random_number: u32
  ) {
    // Redundant
    // If validator already chosen, then return
    if let Ok(validator_id) = SubnetElectedValidator::<T>::try_get(subnet_id, epoch) {
      return
    }

    let slot_list = SubnetNodeElectionSlots::<T>::get(subnet_id);

    if slot_list.is_empty() {
      return
    }

    let idx = (random_number as usize) % slot_list.len();

    let subnet_node_id = slot_list.get(idx).cloned();

    if subnet_node_id.is_some() {
      // --- Insert validator for next epoch
      SubnetElectedValidator::<T>::insert(subnet_id, epoch, subnet_node_id.unwrap());
    }
  }

  /// Return the validators reward that submitted data on the previous epoch
  // The attestation percentage must be greater than the MinAttestationPercentage
  pub fn get_validator_reward(
    attestation_percentage: u128,
  ) -> u128 {
    if MinAttestationPercentage::<T>::get() > attestation_percentage {
      return 0
    }
    Self::percent_mul(BaseValidatorReward::<T>::get(), attestation_percentage)
  }

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
      return weight
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
          epoch
        );
      },
      Err(()) => (),
    };

    // --- Get stake balance. This is safe, uses Default value
    // This could be greater than the target stake balance
    let account_subnet_stake: u128 = AccountSubnetStake::<T>::get(&hotkey, subnet_id);

    // --- Get slash amount up to max slash
    // --- Base slash amount
    // stake balance * BaseSlashPercentage
    let base_slash: u128 = Self::percent_mul(account_subnet_stake, BaseSlashPercentage::<T>::get());

    // --- Get percent difference between attestation ratio and min attestation ratio
    // 1.0 - attestation ratio / min attestation ratio
    let attestation_delta = Self::percentage_factor_as_u128().saturating_sub(
      Self::percent_div(
        attestation_percentage, 
        min_attestation_percentage
      )
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
      Self::decrease_account_stake(
        &hotkey,
        subnet_id, 
        slash_amount,
      );
      // weight = weight.saturating_add(T::WeightInfo::decrease_account_stake());
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

    Self::deposit_event(
      Event::Slashing { 
        subnet_id: subnet_id, 
        account_id: hotkey, 
        amount: slash_amount,
      }
    );

    weight
  }
}