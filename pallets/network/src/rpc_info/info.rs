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

impl<T: Config> Pallet<T> {
  pub fn get_subnet_info(subnet_id: u32) -> Option<SubnetInfo<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return None
    }

    let subnet_data = SubnetsData::<T>::get(subnet_id).unwrap();

    let subnet_info = SubnetInfo {
      id: subnet_data.id,
      name: subnet_data.name,
      repo: subnet_data.repo,
      description: subnet_data.description,
      misc: subnet_data.misc,
      state: subnet_data.state,
      start_epoch: subnet_data.start_epoch,
      churn_limit: ChurnLimit::<T>::get(subnet_id),
      min_stake: SubnetMinStakeBalance::<T>::get(subnet_id),
      max_stake: SubnetMaxStakeBalance::<T>::get(subnet_id),
      delegate_stake_percentage: SubnetDelegateStakeRewardsPercentage::<T>::get(subnet_id),
      registration_queue_epochs: RegistrationQueueEpochs::<T>::get(subnet_id),
      activation_grace_epochs: ActivationGraceEpochs::<T>::get(subnet_id),
      queue_classification_epochs: IdleClassificationEpochs::<T>::get(subnet_id),
      included_classification_epochs: IncludedClassificationEpochs::<T>::get(subnet_id),
      max_node_penalties: MaxSubnetNodePenalties::<T>::get(subnet_id),
      initial_coldkeys: SubnetRegistrationInitialColdkeys::<T>::get(subnet_id),
      max_registered_nodes: MaxRegisteredNodes::<T>::get(subnet_id),
      owner: SubnetOwner::<T>::get(subnet_id)?,
      registration_epoch: SubnetRegistrationEpoch::<T>::get(subnet_id)?,
      node_removal_system: SubnetNodeRemovalSystem::<T>::get(subnet_id),
      key_types: SubnetKeyTypes::<T>::get(subnet_id),
      slot_index: SubnetSlot::<T>::get(subnet_id)?,
    };

    Some(subnet_info)
  }

  pub fn get_subnet_nodes(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let epoch: u32 = Self::get_current_epoch_as_u32();
    Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Idle, epoch)
  }

  pub fn get_subnet_nodes_included(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let epoch: u32 = Self::get_current_epoch_as_u32();
    Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch)
  }

  pub fn get_subnet_nodes_validator(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let epoch: u32 = Self::get_current_epoch_as_u32();
    Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Validator, epoch)
  }

  pub fn get_subnet_nodes_info(
    subnet_id: u32,
  ) -> Vec<SubnetNodeInfo<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let epoch: u32 = Self::get_current_epoch_as_u32();
    Self::get_classified_subnet_nodes_info(subnet_id, &SubnetNodeClass::Validator, epoch)
  }

  pub fn get_subnet_node_info(subnet_id: u32, subnet_node_id: u32) -> Option<SubnetNodeInfo<T::AccountId>> {
    let subnet_node = if SubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      SubnetNodesData::<T>::get(subnet_id, subnet_node_id)
    } else if RegisteredSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      RegisteredSubnetNodesData::<T>::get(subnet_id, subnet_node_id)
    } else if DeactivatedSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
      DeactivatedSubnetNodesData::<T>::get(subnet_id, subnet_node_id)
    } else {
      return None
    };

    let coldkey = HotkeyOwner::<T>::get(&subnet_node.hotkey);
    let info = SubnetNodeInfo {
      subnet_node_id: subnet_node_id,
      coldkey: coldkey.clone(),
      hotkey: subnet_node.hotkey.clone(),
      peer_id: subnet_node.peer_id,
      bootstrap_peer_id: subnet_node.bootstrap_peer_id,
      client_peer_id: subnet_node.client_peer_id,
      identity: ColdkeyIdentity::<T>::get(&coldkey),
      classification: subnet_node.classification,
      delegate_reward_rate: subnet_node.delegate_reward_rate,
      last_delegate_reward_rate_update: subnet_node.last_delegate_reward_rate_update,
      a: subnet_node.a,
      b: subnet_node.b,
      c: subnet_node.c,
      stake_balance: AccountSubnetStake::<T>::get(subnet_node.hotkey, subnet_id)
    };

    return Some(info)
  }

  pub fn get_elected_validator_info(subnet_id: u32, epoch: u32) -> Option<SubnetNodeInfo<T::AccountId>> {
    match SubnetElectedValidator::<T>::try_get(subnet_id, epoch) {
      Ok(subnet_node_id) => {
        Self::get_subnet_node_info(subnet_id, subnet_node_id)
      },
      Err(()) => None,
    }
  }

  pub fn get_elected_validator_node(subnet_id: u32, epoch: u32) -> Option<SubnetNode<T::AccountId>> {
    match SubnetElectedValidator::<T>::try_get(subnet_id, epoch) {
      Ok(subnet_node_id) => {
        match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
          Ok(data) => {
            Some(data)
          },
          Err(()) => None,
        }
      },
      Err(()) => None,
    }
  }

  pub fn get_subnet_node_by_params(
    subnet_id: u32,
    a: BoundedVec<u8, DefaultMaxVectorLength>,
  ) -> Option<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return None
    }

    SubnetNodesData::<T>::iter_prefix_values(subnet_id)
      .find(|x| {
        // Find by ``a``, a unique parameter
        x.a == Some(a.clone())
      })
  }

  // id is consensus ID
  pub fn get_consensus_data(
    subnet_id: u32,
    epoch: u32
  ) -> Option<ConsensusData<T::AccountId>> {
    let data = SubnetConsensusSubmission::<T>::get(subnet_id, epoch);
    Some(data?)
  }

  // pub fn get_incentives_data(
  //   subnet_id: u32,
  //   epoch: u32
  // ) -> Option<ConsensusData<T::AccountId>> {
  //   let data = SubnetConsensusSubmission::<T>::get(subnet_id, epoch);
  //   Some(data?)
  // }

  pub fn get_minimum_subnet_nodes(memory_mb: u128) -> u32 {
    MinSubnetNodes::<T>::get()
  }

  pub fn get_minimum_delegate_stake(memory_mb: u128) -> u128 {
    Self::get_min_subnet_delegate_stake_balance()
  }

  pub fn get_subnet_node_stake_by_peer_id(subnet_id: u32, peer_id: PeerId) -> u128 {
    match PeerIdSubnetNodeId::<T>::try_get(subnet_id, &peer_id) {
      Ok(subnet_node_id) => {
        let hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, subnet_node_id).unwrap(); // TODO: error fallback
        AccountSubnetStake::<T>::get(hotkey, subnet_id)
      },
      Err(()) => 0,
    }
  }

  // TODO: Make this only return true is Validator subnet node
  pub fn is_subnet_node_by_peer_id(subnet_id: u32, peer_id: Vec<u8>) -> bool {
    match PeerIdSubnetNodeId::<T>::try_get(subnet_id, PeerId(peer_id)) {
      Ok(_) => true,
      Err(()) => false,
    }
  }

  pub fn is_subnet_node_by_bootstrap_peer_id(subnet_id: u32, peer_id: Vec<u8>) -> bool {
    match BootstrapPeerIdSubnetNodeId::<T>::try_get(subnet_id, PeerId(peer_id)) {
      Ok(_) => true,
      Err(()) => false,
    }
  }

  pub fn are_subnet_nodes_by_peer_id(subnet_id: u32, peer_ids: Vec<Vec<u8>>) -> BTreeMap<Vec<u8>, bool> {
    let mut subnet_nodes: BTreeMap<Vec<u8>, bool> = BTreeMap::new();

    for peer_id in peer_ids.iter() {
      let is = match PeerIdSubnetNodeId::<T>::try_get(subnet_id, PeerId(peer_id.clone())) {
        Ok(_) => true,
        Err(()) => false,
      };
      subnet_nodes.insert(peer_id.clone(), is);
    }

    subnet_nodes
  }

  /// If subnet node exists under unique subnet node parameter ``a``
  pub fn is_subnet_node_by_a(
    subnet_id: u32, 
    a: BoundedVec<u8, DefaultMaxVectorLength>
  ) -> bool {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return false
    }

    match SubnetNodeUniqueParam::<T>::try_get(subnet_id, a) {
      Ok(_) => true,
      Err(()) => false,
    }
  }

  /// Proof-of-stake
  ///
  /// - Returns if the node has a proof of stake
  ///
  /// # Options
  ///
  /// - Can use either a subnet node ID or peer ID, or bootstrap peer ID
  ///
  /// The most secure way to call this function is by peer ID with signatures
  ///
  /// # Requirements
  ///
  /// To use `peer_id` effectively, ensure all communications between nodes in the subnets
  /// are signed and validated.
  ///
  /// # Arguments
  ///
  /// * `subnet_id` - Subnet ID.
  /// * `subnet_node_id` - Subnet node ID
  /// * `peer_id` - Subnet node peer ID
  /// * `require_active` - Require that the subnet node is currently active (not registered or deactivated)
  ///
  pub fn proof_of_stake(
    subnet_id: u32, 
    peer_id: Vec<u8>,
    require_active: bool
  ) -> bool {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return false
    }

    let mut is_staked = false;

    // --- Use peer ID
    is_staked = match PeerIdSubnetNodeId::<T>::try_get(subnet_id, PeerId(peer_id.clone())) {
      Ok(subnet_node_id) => {
        if require_active {
          match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
            Ok(_) => true,
            Err(()) => false
          }
        } else {
          true
        }
      },
      Err(()) => false,
    };

    if is_staked {
      return true
    }

    // --- Use peer ID, check bootstrap peer ID
    is_staked = match BootstrapPeerIdSubnetNodeId::<T>::try_get(subnet_id, PeerId(peer_id.clone())) {
      Ok(subnet_node_id) => {
        if require_active {
          match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
            Ok(_) => true,
            Err(()) => false
          }
        } else {
          true
        }
      },
      Err(()) => false,
    };

    if is_staked {
      return true
    }

    // --- Use peer ID, check client peer ID
    is_staked = match ClientPeerIdSubnetNode::<T>::try_get(subnet_id, PeerId(peer_id.clone())) {
      Ok(subnet_node_id) => {
        if require_active {
          match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
            Ok(_) => true,
            Err(()) => false
          }
        } else {
          true
        }
      },
      Err(()) => false,
    };

    if is_staked {
      return true
    }

    // --- Check overwatch node
    match PeerIdOverwatchNode::<T>::try_get(subnet_id, PeerId(peer_id.clone())) {
      Ok(_) => true,
      Err(()) => false,
    }
  }
  // pub fn proof_of_stake(
  //   subnet_id: u32, 
  //   subnet_node_id: u32,
  //   peer_id: Vec<u8>,
  //   require_active: bool
  // ) -> bool {
  //   if !SubnetsData::<T>::contains_key(subnet_id) {
  //     return false
  //   }

  //   let mut is_staked = false;

  //   // --- Use subnet node ID
  //   if subnet_node_id > 0 {
  //     if require_active {
  //       is_staked = match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
  //         Ok(_) => true,
  //         Err(()) => false
  //       };
  //     } else {
  //       is_staked = match SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id) {
  //         Ok(_) => true,
  //         Err(()) => false
  //       };
  //     }

  //     return is_staked
  //   }

  //   // --- Use peer ID
  //   is_staked = match PeerIdSubnetNodeId::<T>::try_get(subnet_id, PeerId(peer_id.clone())) {
  //     Ok(subnet_node_id) => {
  //       if require_active {
  //         match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
  //           Ok(_) => true,
  //           Err(()) => false
  //         }
  //       } else {
  //         true
  //       }
  //     },
  //     Err(()) => false,
  //   };

  //   if is_staked {
  //     return true
  //   }

  //   // --- Use peer ID, check bootstrap peer ID
  //   is_staked = match BootstrapPeerIdSubnetNodeId::<T>::try_get(subnet_id, PeerId(peer_id.clone())) {
  //     Ok(subnet_node_id) => {
  //       if require_active {
  //         match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
  //           Ok(_) => true,
  //           Err(()) => false
  //         }
  //       } else {
  //         true
  //       }
  //     },
  //     Err(()) => false,
  //   };

  //   if is_staked {
  //     return true
  //   }

  //   // --- Use peer ID, check client peer ID
  //   match ClientPeerIdSubnetNode::<T>::try_get(subnet_id, PeerId(peer_id.clone())) {
  //     Ok(subnet_node_id) => {
  //       if require_active {
  //         match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
  //           Ok(_) => true,
  //           Err(()) => false
  //         }
  //       } else {
  //         true
  //       }
  //     },
  //     Err(()) => false,
  //   }
  // }

  /// Client Proof-of-stake
  ///
  /// Checks if the client peer ID is staked
  ///
  /// - Returns if the node has a proof of stake
  ///
  /// # Options
  ///
  /// - Can use either a subnet node ID or peer ID, or bootstrap peer ID
  ///
  /// The most secure way to call this function is by peer ID with signatures
  ///
  /// # Arguments
  ///
  /// * `subnet_id` - Subnet ID.
  /// * `peer_id` - Subnet node client peer ID
  /// * `require_active` - Require that the subnet node is currently active (not registered or deactivated)
  ///
  pub fn client_proof_of_stake(
    subnet_id: u32, 
    peer_id: Vec<u8>,
    require_active: bool
  ) -> bool {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return false
    }

    match ClientPeerIdSubnetNode::<T>::try_get(subnet_id, PeerId(peer_id)) {
      Ok(_) => {
        if require_active {
          true
        } else {
          true
        }
      },
      Err(()) => false,
    }
  }

}