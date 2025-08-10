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
use sp_core::U256;

impl<T: Config> Pallet<T> {
  pub fn insert_node_into_election_slot(subnet_id: u32, subnet_node_id: u32) -> bool {
    SubnetNodeElectionSlots::<T>::try_mutate(subnet_id, |slot_list| -> Result<bool, ()> {
      if !slot_list.contains(&subnet_node_id) {
        let idx = slot_list.len() as u32;
        slot_list.try_push(subnet_node_id).map_err(|_| ())?;
        NodeSlotIndex::<T>::insert(subnet_id, subnet_node_id, idx);
        TotalSubnetElectableNodes::<T>::mutate(subnet_id, |mut n| n.saturating_inc());
        TotalElectableNodes::<T>::mutate(|mut n| n.saturating_inc());
        Ok(true)
      } else {
        Ok(false)
      }
    }).unwrap_or(false)
  }

  pub fn remove_node_from_election_slot(subnet_id: u32, subnet_node_id: u32) -> bool {
    SubnetNodeElectionSlots::<T>::try_mutate(subnet_id, |slot_list| -> Result<bool, ()> {
      if let Some(pos) = slot_list.iter().position(|id| *id == subnet_node_id) {
        let last_idx = slot_list.len() - 1;
        slot_list.swap_remove(pos);

        if pos != last_idx {
          let moved_node_id = slot_list[pos];
          NodeSlotIndex::<T>::insert(subnet_id, moved_node_id, pos as u32);
        }

        NodeSlotIndex::<T>::remove(subnet_id, subnet_node_id);
        TotalSubnetElectableNodes::<T>::mutate(subnet_id, |mut n| n.saturating_dec());
        TotalElectableNodes::<T>::mutate(|mut n| n.saturating_dec());
        Ok(true)
      } else {
        Ok(false)
      }
    })
    .unwrap_or(false)
  }

  /// Removes a subnet node, can be registering, active, or deactive
  pub fn perform_remove_subnet_node(subnet_id: u32, subnet_node_id: u32) {
    let mut is_active = false;
    let subnet_node = if SubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      is_active = true;
      SubnetNodesData::<T>::take(subnet_id, subnet_node_id)
    } else if RegisteredSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      RegisteredSubnetNodesData::<T>::take(subnet_id, subnet_node_id)
    } else if DeactivatedSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      DeactivatedSubnetNodesData::<T>::take(subnet_id, subnet_node_id)
    } else {
      return
    };

    let hotkey = subnet_node.hotkey;
    let peer_id = subnet_node.peer_id;

    if subnet_node.a.is_some() {
      SubnetNodeUniqueParam::<T>::remove(subnet_id, subnet_node.a.unwrap())
    }

    // Remove all subnet node elements
    PeerIdSubnetNodeId::<T>::remove(subnet_id, &peer_id);
    BootstrapPeerIdSubnetNodeId::<T>::remove(subnet_id, subnet_node.bootnode_peer_id);
    HotkeySubnetNodeId::<T>::remove(subnet_id, &hotkey);
    SubnetNodeIdHotkey::<T>::remove(subnet_id, subnet_node_id);

    // Remove subnet ID from set
    match HotkeyOwner::<T>::try_get(&hotkey) {
      Ok(coldkey) => {
        ColdkeySubnets::<T>::mutate(&coldkey, |subnets| {
          subnets.remove(&subnet_id);
        });
      },
      Err(()) => ()
    }

    // We don't remove the HotkeyOwner so the user can remove stake with coldkey

    // Update total subnet peers by subtracting  1
    TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());
    TotalNodes::<T>::mutate(|n: &mut u32| n.saturating_dec());

    if subnet_node.classification.node_class == SubnetNodeClass::Validator {
      // --- Try removing node from election slots (only happens if Validator classification)
      Self::remove_node_from_election_slot(subnet_id, subnet_node_id);
    }

    if is_active {
      TotalActiveSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());
      TotalActiveNodes::<T>::mutate(|n: &mut u32| n.saturating_dec());
      match HotkeyOwner::<T>::try_get(&hotkey) {
        Ok(coldkey) => {
          ColdkeyReputation::<T>::mutate(&coldkey, |rep| {
            rep.total_active_nodes = rep.total_active_nodes.saturating_sub(1);
          });
        },
        Err(()) => ()
      }
    }

    Self::deposit_event(Event::SubnetNodeRemoved { subnet_id: subnet_id, subnet_node_id: subnet_node_id });
  }

  pub fn get_subnet_node(subnet_id: u32, subnet_node_id: u32) -> Option<SubnetNode<T::AccountId>> {
    if SubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      Some(SubnetNodesData::<T>::get(subnet_id, subnet_node_id))
    } else if RegisteredSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      Some(RegisteredSubnetNodesData::<T>::get(subnet_id, subnet_node_id))
    } else if DeactivatedSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      Some(DeactivatedSubnetNodesData::<T>::get(subnet_id, subnet_node_id))
    } else {
      None
    }
  }

  /// Get any subnet node that has been activated (not including registered nodes)
  pub fn get_activated_subnet_node(subnet_id: u32, subnet_node_id: u32) -> Option<SubnetNode<T::AccountId>> {
    if SubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      Some(SubnetNodesData::<T>::get(subnet_id, subnet_node_id))
    } else if DeactivatedSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      Some(DeactivatedSubnetNodesData::<T>::get(subnet_id, subnet_node_id))
    } else {
      None
    }
  }

  /// Get any subnet node that has been activated (not including registered nodes)
  pub fn update_subnet_node_hotkey(subnet_id: u32, subnet_node_id: u32, new_hotkey: T::AccountId) {
    if SubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      SubnetNodesData::<T>::try_mutate_exists(
        subnet_id,
        subnet_node_id,
        |maybe_params| -> DispatchResult {
          let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetNodeId)?;
          params.hotkey = new_hotkey.clone();
          Ok(())
        }
      );
    } else if DeactivatedSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      DeactivatedSubnetNodesData::<T>::try_mutate_exists(
        subnet_id,
        subnet_node_id,
        |maybe_params| -> DispatchResult {
          let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetNodeId)?;
          params.hotkey = new_hotkey.clone();
          Ok(())
        }
      );
    } else if RegisteredSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      RegisteredSubnetNodesData::<T>::try_mutate_exists(
        subnet_id,
        subnet_node_id,
        |maybe_params| -> DispatchResult {
          let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetNodeId)?;
          params.hotkey = new_hotkey.clone();
          Ok(())
        }
      );
    }
  }

  pub fn get_classified_subnet_node_ids<C>(
    subnet_id: u32,
    classification: &SubnetNodeClass,
    subnet_epoch: u32,
  ) -> C
    where
      C: FromIterator<u32>,
  {
    SubnetNodesData::<T>::iter_prefix(subnet_id)
      .filter(|(_, subnet_node)| subnet_node.has_classification(classification, subnet_epoch))
      .map(|(subnet_node_id, _)| subnet_node_id)
      .collect()
  }
  
  /// Get subnet nodes by classification
  pub fn get_classified_subnet_nodes(subnet_id: u32, classification: &SubnetNodeClass, subnet_epoch: u32) -> Vec<SubnetNode<T::AccountId>> {
    SubnetNodesData::<T>::iter_prefix_values(subnet_id)
      .filter(|subnet_node| subnet_node.has_classification(classification, subnet_epoch))
      .collect()
  }

  pub fn get_classified_subnet_nodes_map(
    subnet_id: u32,
    classification: &SubnetNodeClass,
    subnet_epoch: u32,
  ) -> BTreeMap<u32, SubnetNode<T::AccountId>> {
    SubnetNodesData::<T>::iter_prefix(subnet_id)
      .filter_map(|(subnet_node_id, subnet_node)| {
        if subnet_node.has_classification(classification, subnet_epoch) {
          Some((subnet_node_id, subnet_node))
        } else {
          None
        }
      })
      .collect()
  }

  pub fn get_classified_subnet_nodes_info(subnet_id: u32, classification: &SubnetNodeClass, subnet_epoch: u32) -> Vec<SubnetNodeInfo<T::AccountId>> {
    SubnetNodesData::<T>::iter_prefix(subnet_id)
      .filter(|(subnet_node_id, subnet_node)| subnet_node.has_classification(classification, subnet_epoch))
      .map(|(subnet_node_id, subnet_node)| {
        let coldkey = HotkeyOwner::<T>::get(&subnet_node.hotkey);
        SubnetNodeInfo {
          subnet_node_id: subnet_node_id,
          coldkey: coldkey.clone(),
          hotkey: subnet_node.hotkey.clone(),
          peer_id: subnet_node.peer_id,
          bootnode_peer_id: subnet_node.bootnode_peer_id,
          client_peer_id: subnet_node.client_peer_id,
          bootnode: subnet_node.bootnode,
          identity: ColdkeyIdentity::<T>::get(&coldkey),
          classification: subnet_node.classification,
          delegate_reward_rate: subnet_node.delegate_reward_rate,
          last_delegate_reward_rate_update: subnet_node.last_delegate_reward_rate_update,
          a: subnet_node.a,
          b: subnet_node.b,
          c: subnet_node.c,
          stake_balance: AccountSubnetStake::<T>::get(subnet_node.hotkey, subnet_id),
          node_delegate_stake_balance: NodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id),
          penalties: SubnetNodePenalties::<T>::get(subnet_id, subnet_node_id),
          reputation: ColdkeyReputation::<T>::get(coldkey.clone()),
        }
      })
      .collect()
  }

  // Get subnet node ``hotkeys`` by classification
  pub fn get_classified_hotkeys<C>(
    subnet_id: u32,
    classification: &SubnetNodeClass,
    subnet_epoch: u32,
  ) -> C
    where
      C: FromIterator<T::AccountId>,
  {
    SubnetNodesData::<T>::iter_prefix(subnet_id)
      .filter(|(_, subnet_node)| subnet_node.has_classification(classification, subnet_epoch))
      .map(|(_, subnet_node)| subnet_node.hotkey)
      .collect()
  }

  /// Is hotkey or coldkey owner for functions that allow both
  pub fn get_subnet_node_hotkey_coldkey(
    subnet_id: u32, 
    subnet_node_id: u32, 
  ) -> Option<(T::AccountId, T::AccountId)> {
    let hotkey = SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id).ok()?;
    let coldkey = HotkeyOwner::<T>::try_get(&hotkey).ok()?;

    Some((hotkey, coldkey))
  }

  pub fn is_subnet_node_keys_owner(
    subnet_id: u32, 
    subnet_node_id: u32, 
    key: T::AccountId, 
  ) -> bool {
    let (hotkey, coldkey) = match Self::get_subnet_node_hotkey_coldkey(subnet_id, subnet_node_id) {
      Some((hotkey, coldkey)) => {
        (hotkey, coldkey)
      }
      None => {
        return false
      }
    };

    key == hotkey || key == coldkey
  }

  pub fn is_subnet_node_coldkey(
    subnet_id: u32, 
    subnet_node_id: u32, 
    coldkey: T::AccountId, 
  ) -> bool {
    let hotkey = match SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id) {
      Ok(hotkey) => hotkey,
      Err(()) => return false
    };
    match HotkeyOwner::<T>::try_get(hotkey) {
      Ok(subnet_node_coldkey) => return subnet_node_coldkey == coldkey,
      Err(()) => return false
    }
  }

  pub fn is_validator(subnet_id: u32, subnet_node_id: u32, subnet_epoch: u32) -> bool {
    match SubnetElectedValidator::<T>::try_get(subnet_id, subnet_epoch) {
      Ok(validator_subnet_node_id) => {
        let mut is_validator = false;
        if subnet_node_id == validator_subnet_node_id {
          is_validator = true
        }
        is_validator
      },
      Err(()) => false,
    }
  }

  pub fn graduate_class(
    subnet_id: u32, 
    subnet_node_id: u32, 
    start_epoch: u32,
  ) -> bool {
    SubnetNodesData::<T>::try_mutate_exists(
      subnet_id,
      subnet_node_id,
      |maybe_node_data| -> Result<bool, ()> {
        if let Some(node_data) = maybe_node_data {
          node_data.classification = SubnetNodeClassification {
            node_class: node_data.classification.node_class.next(),
            start_epoch,
          };
          Ok(true)
        } else {
          Ok(false)
        }
      },
    ).unwrap_or(false)
  }

  /// Check if subnet node is owner of a peer ID
  /// Main, bootnode, and client peer IDs must be unique so we check all of them to ensure
  /// that no one else owns them
  /// Returns True is no owner or the peer ID is ownerless and available
  pub fn is_owner_of_peer_or_ownerless(
    subnet_id: u32,
    subnet_node_id: u32,
    overwatch_node_id: u32,
    peer_id: &PeerId
  ) -> bool {
    let mut is_peer_owner_or_ownerless = match PeerIdSubnetNodeId::<T>::try_get(subnet_id, peer_id) {
      Ok(peer_subnet_node_id) => {
        if peer_subnet_node_id == subnet_node_id {
          return true
        }
        false
      },
      Err(()) => true,
    };

    is_peer_owner_or_ownerless = is_peer_owner_or_ownerless && match BootstrapPeerIdSubnetNodeId::<T>::try_get(subnet_id, peer_id) {
      Ok(bootnode_subnet_node_id) => {
        if bootnode_subnet_node_id == subnet_node_id {
          return true
        }
        false
      },
      Err(()) => true,
    };

    is_peer_owner_or_ownerless = is_peer_owner_or_ownerless && match ClientPeerIdSubnetNode::<T>::try_get(subnet_id, peer_id) {
      Ok(client_subnet_node_id) => {
        if client_subnet_node_id == subnet_node_id {
          return true
        }
        false
      },
      Err(()) => true,
    };

    is_peer_owner_or_ownerless && match PeerIdOverwatchNode::<T>::try_get(subnet_id, peer_id) {
      Ok(peer_overwatch_node_id) => {
        if peer_overwatch_node_id == overwatch_node_id {
          return true
        }
        false
      },
      Err(()) => true,
    }
  }

  pub fn calculate_max_activation_epoch(subnet_id: u32) -> u32 {
    let prev_registration_epoch = 10;
    0
  }

  /// Get the subnets minimum delegate stake multipler based on the current electable nodes count
  pub fn get_subnet_min_delegate_staking_multiplier(electable_node_count: u32) -> u128 {
    let min_nodes = MinSubnetNodes::<T>::get();
    let max_nodes = MaxSubnetNodes::<T>::get();
    let min_multiplier = Self::percentage_factor_as_u128(); // 100%
    let max_multiplier = MaxMinDelegateStakeMultiplier::<T>::get();

    if electable_node_count <= min_nodes {
      return min_multiplier;
    } else if electable_node_count >= max_nodes {
      return max_multiplier;
    }

    let node_delta = electable_node_count - min_nodes;
    let range = max_nodes - min_nodes;

    let ratio = Self::percent_div(node_delta as u128, range as u128);
    let delta = max_multiplier.saturating_sub(min_multiplier);

    min_multiplier.saturating_add(Self::percent_mul(ratio, delta))
  }

  
  pub fn get_lowest_stake_balance_node(subnet_id: u32, hotkey: &T::AccountId) -> Option<u32> {
    // Get calling nodes stake balance
    let activating_node_stake_balance = AccountSubnetStake::<T>::get(&hotkey, subnet_id);
    let percentage_delta = NodeRemovalStakePercentageDelta::<T>::get(subnet_id);
    let removal_stake_balance = activating_node_stake_balance.saturating_sub(
      Self::percent_mul(activating_node_stake_balance, percentage_delta)
    );
    let mut candidates: Vec<(u32, u128, u32)> = Vec::new(); // (uid, stake, start_epoch)

    for (uid, node) in SubnetNodesData::<T>::iter_prefix(subnet_id) {
      let node_hotkey = node.hotkey.clone();
      let stake = AccountSubnetStake::<T>::get(&node_hotkey, subnet_id);
      if stake >= removal_stake_balance {
        continue
      }
      let start_epoch = node.classification.start_epoch;
      candidates.push((uid, stake, start_epoch));
    }

    if candidates.is_empty() {
      return None
    }

    candidates.sort_by(|a, b| {
      // Sort by stake ascending, then start_epoch descending
      a.1.cmp(&b.1).then(b.2.cmp(&a.2))
    });

    candidates.first().map(|(uid, _, _)| *uid)
  }

  pub fn get_lowest_reputation_node(subnet_id: u32, coldkey: &T::AccountId) -> Option<u32> {
    // Get calling nodes reputation element
    let coldkey_reputation = ColdkeyReputation::<T>::get(coldkey);
    let percentage_delta = NodeRemovalReputationScorePercentageDelta::<T>::get(subnet_id);
    let min_score = NodeRemovalReputationScoreMin::<T>::get(subnet_id);
    let rep_score = coldkey_reputation.score;
    let removal_rep_score = rep_score.saturating_sub(
      Self::percent_mul(rep_score, percentage_delta)
    );

    let immunity_epochs = NodeRemovalImmunityEpochs::<T>::get(subnet_id);

    let mut candidates: Vec<(u32, u128, u32)> = Vec::new(); // (uid, score, start_epoch)

    for (uid, node) in SubnetNodesData::<T>::iter_prefix(subnet_id) {
      let node_hotkey = node.hotkey.clone();
      let reputation = ColdkeyReputation::<T>::get(coldkey);
      if reputation.score >= removal_rep_score || reputation.score >= min_score {
        continue
      }
      let start_epoch = node.classification.start_epoch;
      candidates.push((uid, reputation.score, start_epoch));
    }

    if candidates.is_empty() {
      return None
    }

    candidates.sort_by(|a, b| {
      // Sort by reputation.score ascending, then start_epoch descending
      a.1.cmp(&b.1).then(b.2.cmp(&a.2))
    });

    candidates.first().map(|(uid, _, _)| *uid)
  }

  pub fn get_removing_node(
    subnet_id: u32, 
    coldkey: &T::AccountId, 
    hotkey: &T::AccountId, 
    subnet_node: &SubnetNode<T::AccountId>
  ) -> Option<u32> {
    let policy = match NodeRemovalSystemV2::<T>::get(subnet_id) {
      Some(policy) => policy,
      None => return None
    };

    let activating_coldkey_reputation = ColdkeyReputation::<T>::get(coldkey);
    let activating_node_dstake_balance = NodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node.id);
    let activating_stake_balance = AccountSubnetStake::<T>::get(hotkey, subnet_id);

    let mut candidates: Vec<(u32, u128, u32)> = Vec::new(); // (uid, score, start_epoch)

    for (uid, node) in SubnetNodesData::<T>::iter_prefix(subnet_id) {
      // Redundant, activating node is in RegisteredSubnetNodesData
      if uid == subnet_node.id {
        continue
      }

      let node_hotkey = node.hotkey.clone();
      let node_coldkey = HotkeyOwner::<T>::get(&node_hotkey);
      let proposing_stake = AccountSubnetStake::<T>::get(&node_hotkey, subnet_id);
      let proposing_dstake = NodeDelegateStakeBalance::<T>::get(subnet_id, node.id);
      let proposing_reputation = ColdkeyReputation::<T>::get(&node_coldkey);
      let proposing_score = proposing_reputation.score;

      if !Self::evaluate_logic_expr(
        &policy.logic,
        activating_coldkey_reputation.score,
        activating_coldkey_reputation.average_attestation,
        subnet_node.delegate_reward_rate,
        activating_node_dstake_balance,
        activating_stake_balance,
        proposing_score,
        proposing_reputation.average_attestation,
        node.delegate_reward_rate,
        proposing_dstake,
        proposing_stake,
      ) {
        continue
      }
      let start_epoch = node.classification.start_epoch;
      candidates.push((uid, proposing_score, start_epoch));
    }

    if candidates.is_empty() {
      return None
    }

    candidates.sort_by(|a, b| {
      // Sort by reputation.score ascending, then start_epoch descending
      a.1.cmp(&b.1).then(b.2.cmp(&a.2))
    });

    candidates.first().map(|(uid, _, _)| *uid)
  }

  pub fn evaluate_logic_expr(
    expr: &LogicExpr,
    activating_score: u128,
    activating_avg_attestation: u128,
    activating_dstake_rate: u128,
    activating_dstake_balance: u128,
    activating_stake_balance: u128,
    proposing_score: u128,
    proposing_avg_attestation: u128,
    proposing_dstake_rate: u128,
    proposing_dstake_balance: u128,
    proposing_stake_balance: u128,
  ) -> bool {
    match expr {
      LogicExpr::And(left, right) => {
        Self::evaluate_logic_expr(
          left, 
          activating_score,
          activating_avg_attestation,
          activating_dstake_rate,
          activating_dstake_balance,
          activating_stake_balance,
          proposing_score,
          proposing_avg_attestation,
          proposing_dstake_rate,
          proposing_dstake_balance,
          proposing_stake_balance
        )
          && Self::evaluate_logic_expr(
              right, 
              activating_score,
              activating_avg_attestation,
              activating_dstake_rate,
              activating_dstake_balance,
              activating_stake_balance,
              proposing_score,
              proposing_avg_attestation,
              proposing_dstake_rate,
              proposing_dstake_balance,
              proposing_stake_balance
            )
      }
      LogicExpr::Or(left, right) => {
        Self::evaluate_logic_expr(
          left, 
          activating_score,
          activating_avg_attestation,
          activating_dstake_rate,
          activating_dstake_balance,
          activating_stake_balance,
          proposing_score,
          proposing_avg_attestation,
          proposing_dstake_rate,
          proposing_dstake_balance,
          proposing_stake_balance
        )
          || Self::evaluate_logic_expr(
              right, 
              activating_score,
              activating_avg_attestation,
              activating_dstake_rate,
              activating_dstake_balance,
              activating_stake_balance,
              proposing_score,
              proposing_avg_attestation,
              proposing_dstake_rate,
              proposing_dstake_balance,
              proposing_stake_balance
            )
      }
      LogicExpr::Xor(left, right) => {
        Self::evaluate_logic_expr(
          left, 
          activating_score,
          activating_avg_attestation,
          activating_dstake_rate,
          activating_dstake_balance,
          activating_stake_balance,
          proposing_score,
          proposing_avg_attestation,
          proposing_dstake_rate,
          proposing_dstake_balance,
          proposing_stake_balance
        )
          ^ Self::evaluate_logic_expr(
              right, 
              activating_score,
              activating_avg_attestation,
              activating_dstake_rate,
              activating_dstake_balance,
              activating_stake_balance,
              proposing_score,
              proposing_avg_attestation,
              proposing_dstake_rate,
              proposing_dstake_balance,
              proposing_stake_balance
            )
      }
      LogicExpr::Not(inner) => {
        !Self::evaluate_logic_expr(
          inner, 
          activating_score,
          activating_avg_attestation,
          activating_dstake_rate,
          activating_dstake_balance,
          activating_stake_balance,
          proposing_score,
          proposing_avg_attestation,
          proposing_dstake_rate,
          proposing_dstake_balance,
          proposing_stake_balance
        )
      }
      LogicExpr::Condition(cond) => match cond {
        // hard
        NodeRemovalConditionType::HardBelowScore(v) => {
          proposing_score < *v
        },
        NodeRemovalConditionType::HardBelowAverageAttestation(v) => {
          proposing_avg_attestation < *v
        },
        NodeRemovalConditionType::HardBelowNodeDelegateStakeRate(v) => {
          proposing_dstake_rate < *v
        },

        // delta

        // If node is under the activating nodes score delta value
        NodeRemovalConditionType::DeltaBelowScore(v) => {
          proposing_score < activating_score.saturating_sub(Self::percent_mul(activating_score, *v))
        },
        NodeRemovalConditionType::DeltaBelowAverageAttestation(v) => {
          proposing_avg_attestation < activating_avg_attestation.saturating_sub(Self::percent_mul(activating_avg_attestation, *v))
        },
        NodeRemovalConditionType::DeltaBelowNodeDelegateStakeRate(v) => {
          proposing_dstake_rate < activating_dstake_rate.saturating_sub(Self::percent_mul(activating_dstake_rate, *v))
        },
        NodeRemovalConditionType::DeltaBelowNodeDelegateStakeBalance(v) => {
          proposing_dstake_balance < activating_dstake_balance.saturating_sub(Self::percent_mul(activating_dstake_balance, *v))
        },
        NodeRemovalConditionType::DeltaBelowStakeBalance(v) => {
          proposing_stake_balance < activating_stake_balance.saturating_sub(Self::percent_mul(activating_stake_balance, *v))
        },
      },
    }
  }
}