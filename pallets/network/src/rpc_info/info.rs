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
        let subnet_data = SubnetsData::<T>::try_get(subnet_id).ok()?;

        Some(SubnetInfo {
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
            subnet_node_queue_epochs: SubnetNodeQueueEpochs::<T>::get(subnet_id),
            activation_grace_epochs: ActivationGraceEpochs::<T>::get(subnet_id),
            queue_classification_epochs: IdleClassificationEpochs::<T>::get(subnet_id),
            included_classification_epochs: IncludedClassificationEpochs::<T>::get(subnet_id),
            max_node_penalties: MaxSubnetNodePenalties::<T>::get(subnet_id),
            max_registered_nodes: MaxRegisteredNodes::<T>::get(subnet_id),
            key_types: SubnetKeyTypes::<T>::get(subnet_id),
            penalty_count: SubnetPenaltyCount::<T>::get(subnet_id),
            total_nodes: TotalSubnetNodes::<T>::get(subnet_id),
            total_active_nodes: TotalActiveSubnetNodes::<T>::get(subnet_id),
            total_electable_nodes: TotalSubnetElectableNodes::<T>::get(subnet_id),
            initial_coldkeys: SubnetRegistrationInitialColdkeys::<T>::get(subnet_id),
            owner: SubnetOwner::<T>::get(subnet_id),
            registration_epoch: SubnetRegistrationEpoch::<T>::get(subnet_id),
            // node_removal_system: NodeRemovalSystemV2::<T>::get(subnet_id),
            slot_index: SubnetSlot::<T>::get(subnet_id),
        })
    }

    pub fn get_subnet_data(subnet_id: u32) -> Option<SubnetData> {
        Some(SubnetsData::<T>::try_get(subnet_id).ok()?)
    }

    pub fn get_all_subnets_info() -> Vec<SubnetInfo<T::AccountId>> {
        let mut infos: Vec<SubnetInfo<T::AccountId>> = Vec::new();

        for (subnet_id, subnet_data) in SubnetsData::<T>::iter() {
            infos.push(SubnetInfo {
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
                delegate_stake_percentage: SubnetDelegateStakeRewardsPercentage::<T>::get(
                    subnet_id,
                ),
                subnet_node_queue_epochs: SubnetNodeQueueEpochs::<T>::get(subnet_id),
                activation_grace_epochs: ActivationGraceEpochs::<T>::get(subnet_id),
                queue_classification_epochs: IdleClassificationEpochs::<T>::get(subnet_id),
                included_classification_epochs: IncludedClassificationEpochs::<T>::get(subnet_id),
                max_node_penalties: MaxSubnetNodePenalties::<T>::get(subnet_id),
                initial_coldkeys: SubnetRegistrationInitialColdkeys::<T>::get(subnet_id),
                max_registered_nodes: MaxRegisteredNodes::<T>::get(subnet_id),
                owner: SubnetOwner::<T>::get(subnet_id),
                registration_epoch: SubnetRegistrationEpoch::<T>::get(subnet_id),
                // node_removal_system: NodeRemovalSystemV2::<T>::get(subnet_id),
                key_types: SubnetKeyTypes::<T>::get(subnet_id),
                slot_index: SubnetSlot::<T>::get(subnet_id),
                penalty_count: SubnetPenaltyCount::<T>::get(subnet_id),
                total_nodes: TotalSubnetNodes::<T>::get(subnet_id),
                total_active_nodes: TotalActiveSubnetNodes::<T>::get(subnet_id),
                total_electable_nodes: TotalSubnetElectableNodes::<T>::get(subnet_id),
            })
        }

        infos
    }

    pub fn get_subnet_nodes(subnet_id: u32) -> Vec<SubnetNode<T::AccountId>> {
        if !SubnetsData::<T>::contains_key(subnet_id) {
            return Vec::new();
        }
        let subnet_epoch: u32 = Self::get_current_subnet_epoch_as_u32(subnet_id);
        Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Idle, subnet_epoch)
    }

    pub fn get_min_class_subnet_nodes(
        subnet_id: u32,
        subnet_epoch: u32,
        min_class: u8,
    ) -> Vec<SubnetNode<T::AccountId>> {
        if !SubnetsData::<T>::contains_key(subnet_id) {
            return Vec::new();
        }
        let min_class = SubnetNodeClass::from_repr(min_class.into());
        if min_class.is_none() {
            return Vec::new();
        }

        Self::get_classified_subnet_nodes(subnet_id, &min_class.unwrap(), subnet_epoch)
    }

    pub fn get_subnet_nodes_included(subnet_id: u32) -> Vec<SubnetNode<T::AccountId>> {
        if !SubnetsData::<T>::contains_key(subnet_id) {
            return Vec::new();
        }
        let subnet_epoch: u32 = Self::get_current_subnet_epoch_as_u32(subnet_id);
        Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, subnet_epoch)
    }

    pub fn get_subnet_nodes_validator(subnet_id: u32) -> Vec<SubnetNode<T::AccountId>> {
        if !SubnetsData::<T>::contains_key(subnet_id) {
            return Vec::new();
        }
        let subnet_epoch: u32 = Self::get_current_subnet_epoch_as_u32(subnet_id);
        Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Validator, subnet_epoch)
    }

    pub fn get_subnet_node_info(
        subnet_id: u32,
        subnet_node_id: u32,
    ) -> Option<SubnetNodeInfo<T::AccountId>> {
        let subnet_node = if SubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
            SubnetNodesData::<T>::get(subnet_id, subnet_node_id)
        } else if RegisteredSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
            RegisteredSubnetNodesData::<T>::get(subnet_id, subnet_node_id)
        // } else if PausedSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id) {
        //     PausedSubnetNodesData::<T>::get(subnet_id, subnet_node_id)
        } else {
            return None;
        };

        let coldkey = HotkeyOwner::<T>::get(&subnet_node.hotkey);
        let info = SubnetNodeInfo {
            subnet_id: subnet_id,
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
            unique: subnet_node.unique,
            non_unique: subnet_node.non_unique,
            stake_balance: AccountSubnetStake::<T>::get(subnet_node.hotkey, subnet_id),
            node_delegate_stake_balance: NodeDelegateStakeBalance::<T>::get(
                subnet_id,
                subnet_node_id,
            ),
            penalties: SubnetNodePenalties::<T>::get(subnet_id, subnet_node_id),
            reputation: ColdkeyReputation::<T>::get(coldkey.clone()),
        };

        return Some(info);
    }

    pub fn get_subnet_nodes_info(
        subnet_id: u32,
    ) -> Vec<SubnetNodeInfo<T::AccountId>> {
        let mut infos: Vec<SubnetNodeInfo<T::AccountId>> = Vec::new();

        for (_, subnet_node_id) in HotkeySubnetNodeId::<T>::iter_prefix(subnet_id) {
            if let Some(subnet_node_info) = Self::get_subnet_node_info(
                subnet_id,
                subnet_node_id,
            ) {
                infos.push(subnet_node_info);
            }
        }

        infos
    }

    pub fn get_all_subnet_nodes_info() -> Vec<SubnetNodeInfo<T::AccountId>> {
        let mut infos: Vec<SubnetNodeInfo<T::AccountId>> = Vec::new();

        for (subnet_id, subnet_data) in SubnetsData::<T>::iter() {
            for (_, subnet_node_id) in HotkeySubnetNodeId::<T>::iter_prefix(subnet_id) {
                if let Some(subnet_node_info) = Self::get_subnet_node_info(
                    subnet_id,
                    subnet_node_id,
                ) {
                    infos.push(subnet_node_info);
                }
            }
        }

        infos
    }

    pub fn get_elected_validator_info(
        subnet_id: u32,
        subnet_epoch: u32,
    ) -> Option<SubnetNodeInfo<T::AccountId>> {
        match SubnetElectedValidator::<T>::try_get(subnet_id, subnet_epoch) {
            Ok(subnet_node_id) => Self::get_subnet_node_info(subnet_id, subnet_node_id),
            Err(()) => None,
        }
    }

    pub fn get_elected_validator_node(
        subnet_id: u32,
        subnet_epoch: u32,
    ) -> Option<SubnetNode<T::AccountId>> {
        match SubnetElectedValidator::<T>::try_get(subnet_id, subnet_epoch) {
            Ok(subnet_node_id) => match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
                Ok(data) => Some(data),
                Err(()) => None,
            },
            Err(()) => None,
        }
    }

    pub fn get_subnet_node_by_params(
        subnet_id: u32,
        unique: BoundedVec<u8, DefaultMaxVectorLength>,
    ) -> Option<SubnetNode<T::AccountId>> {
        if !SubnetsData::<T>::contains_key(subnet_id) {
            return None;
        }

        SubnetNodesData::<T>::iter_prefix_values(subnet_id).find(|x| {
            // Find by ``unique``, a unique parameter
            x.unique == Some(unique.clone())
        })
    }

    pub fn get_consensus_data(
        subnet_id: u32,
        subnet_epoch: u32,
    ) -> Option<ConsensusData<T::AccountId>> {
        let data = SubnetConsensusSubmission::<T>::get(subnet_id, subnet_epoch);
        Some(data?)
    }

    pub fn get_subnet_node_stake_by_peer_id(subnet_id: u32, peer_id: PeerId) -> u128 {
        match PeerIdSubnetNodeId::<T>::try_get(subnet_id, &peer_id) {
            Ok(subnet_node_id) => {
                if let Some(hotkey) = SubnetNodeIdHotkey::<T>::get(subnet_id, subnet_node_id) {
                    return AccountSubnetStake::<T>::get(hotkey, subnet_id);
                } else {
                    return 0;
                }
            }
            Err(()) => 0,
        }
    }

    pub fn is_subnet_node_by_peer_id(subnet_id: u32, peer_id: Vec<u8>) -> bool {
        match PeerIdSubnetNodeId::<T>::try_get(subnet_id, PeerId(peer_id)) {
            Ok(_) => true,
            Err(()) => false,
        }
    }

    pub fn is_subnet_node_by_bootnode_peer_id(subnet_id: u32, peer_id: Vec<u8>) -> bool {
        match BootnodePeerIdSubnetNodeId::<T>::try_get(subnet_id, PeerId(peer_id)) {
            Ok(_) => true,
            Err(()) => false,
        }
    }

    /// If subnet node exists under unique subnet node parameter ``unique``
    pub fn is_subnet_node_by_unique(
        subnet_id: u32,
        unique: BoundedVec<u8, DefaultMaxVectorLength>,
    ) -> bool {
        if !SubnetsData::<T>::contains_key(subnet_id) {
            return false;
        }

        match SubnetNodeUniqueParam::<T>::try_get(subnet_id, unique) {
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
    /// - Can use either a subnet node ID or peer ID, or bootnode peer ID
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
    /// * `min_class` - Minimum required class
    ///     * A subnet may likely require Registered or Idle to enter subnet
    ///
    pub fn proof_of_stake(subnet_id: u32, peer_id: Vec<u8>, min_class: u8) -> bool {
        if !SubnetsData::<T>::contains_key(subnet_id) {
            return false;
        }

        let class = SubnetNodeClass::from_repr(min_class.into());
        if class.is_none() {
            return false;
        }
        let min_stake = SubnetMinStakeBalance::<T>::get(subnet_id);
        let current_subnet_epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);
        let peer_id = PeerId(peer_id);

        // Helper closure to check a peer_id lookup mapping
        let check_mapping = |mapping: fn(u32, PeerId) -> Result<u32, ()>| -> bool {
            mapping(subnet_id, peer_id.clone())
                .ok()
                .and_then(|subnet_node_id| {
                    SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id).ok()
                })
                .map(|subnet_node| {
                    subnet_node.has_classification(&class.unwrap(), current_subnet_epoch)
                        && AccountSubnetStake::<T>::get(subnet_node.hotkey, subnet_id) >= min_stake
                })
                .unwrap_or(false)
        };

        // Check the three possible peer-id → subnet-node mappings
        if check_mapping(PeerIdSubnetNodeId::<T>::try_get)
            || check_mapping(BootnodePeerIdSubnetNodeId::<T>::try_get)
            || check_mapping(ClientPeerIdSubnetNodeId::<T>::try_get)
        {
            return true;
        }

        // Finally, check overwatch node
        PeerIdOverwatchNode::<T>::try_get(subnet_id, peer_id).is_ok()
    }

    /// Client Proof-of-stake
    ///
    /// Checks if the client peer ID is staked
    ///
    /// - Returns if the node has a proof of stake
    ///
    /// # Options
    ///
    /// - Can use either a subnet node ID or peer ID, or bootnode peer ID
    ///
    /// The most secure way to call this function is by peer ID with signatures
    ///
    /// # Arguments
    ///
    /// * `subnet_id` - Subnet ID.
    /// * `peer_id` - Subnet node client peer ID
    /// * `require_active` - Require that the subnet node is currently active (not registered or deactivated)
    ///
    pub fn client_proof_of_stake(subnet_id: u32, peer_id: Vec<u8>, require_active: bool) -> bool {
        if !SubnetsData::<T>::contains_key(subnet_id) {
            return false;
        }

        match ClientPeerIdSubnetNodeId::<T>::try_get(subnet_id, PeerId(peer_id)) {
            Ok(_) => {
                if require_active {
                    true
                } else {
                    true
                }
            }
            Err(()) => false,
        }
    }

    pub fn get_bootnodes(subnet_id: u32) -> BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>> {
        let mut bootnodes: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>> =
            SubnetBootnodes::<T>::get(subnet_id);

        bootnodes.extend(
            SubnetNodesData::<T>::iter_prefix(subnet_id).filter_map(|(_, node)| node.bootnode),
        );

        bootnodes
    }

    pub fn get_coldkey_subnet_nodes_info(coldkey: T::AccountId) -> Vec<SubnetNodeInfo<T::AccountId>> {
        ColdkeyHotkeys::<T>::get(coldkey.clone())
            .iter()
            .filter_map(|hotkey| {
                HotkeySubnetId::<T>::get(hotkey)
                    .and_then(|subnet_id| {
                        HotkeySubnetNodeId::<T>::get(subnet_id, hotkey)
                            .and_then(|subnet_node_id| {
                                Self::get_subnet_node_info(subnet_id, subnet_node_id)
                            })
                    })
            })
            .collect()
    }

    pub fn get_coldkey_stakes2(coldkey: T::AccountId) -> Vec<SubnetNodeStakeInfo<T::AccountId>> {
        let mut coldkey_stake: Vec<SubnetNodeStakeInfo<T::AccountId>> = Vec::new();

        for hotkey in ColdkeyHotkeys::<T>::get(coldkey.clone()).iter() {
            if let Some(subnet_id) = HotkeySubnetId::<T>::get(hotkey) {
                coldkey_stake.push(SubnetNodeStakeInfo {
                    subnet_id: Some(subnet_id),
                    subnet_node_id: HotkeySubnetNodeId::<T>::get(subnet_id, hotkey),
                    hotkey: hotkey.clone(),
                    balance: AccountSubnetStake::<T>::get(hotkey, subnet_id),
                })
            }
        }

        coldkey_stake
    }

    pub fn get_coldkey_stakes(coldkey: T::AccountId) -> Vec<SubnetNodeStakeInfo<T::AccountId>> {
        let mut coldkey_stake: Vec<SubnetNodeStakeInfo<T::AccountId>> = Vec::new();

        for (subnet_id, nodes) in ColdkeySubnetNodes::<T>::get(&coldkey).iter() {
            for subnet_node_id in nodes {
                let hotkey: T::AccountId =
                    match SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id) {
                        Ok(hotkey) => hotkey,
                        Err(()) => continue,
                    };

                coldkey_stake.push(SubnetNodeStakeInfo {
                    subnet_id: Some(*subnet_id),
                    subnet_node_id: Some(*subnet_node_id),
                    hotkey: hotkey.clone(),
                    balance: AccountSubnetStake::<T>::get(&hotkey, subnet_id),
                })
            }
        }

        coldkey_stake
    }

    pub fn get_delegate_stakes(account_id: T::AccountId) -> Vec<DelegateStakeInfo> {
        let mut delegate_stake: Vec<DelegateStakeInfo> = Vec::new();

        for (subnet_id, shares) in AccountSubnetDelegateStakeShares::<T>::iter_prefix(&account_id) {
            let balance = Self::convert_to_balance(
                shares,
                TotalSubnetDelegateStakeShares::<T>::get(subnet_id),
                TotalSubnetDelegateStakeBalance::<T>::get(subnet_id),
            );

            delegate_stake.push(DelegateStakeInfo {
                subnet_id,
                shares,
                balance,
            })
        }

        delegate_stake
    }

    pub fn get_node_delegate_stakes(account_id: T::AccountId) -> Vec<NodeDelegateStakeInfo> {
        let mut node_delegate_stake: Vec<NodeDelegateStakeInfo> = Vec::new();

        for ((subnet_id, subnet_node_id), shares) in
            AccountNodeDelegateStakeShares::<T>::iter_prefix((&account_id,))
        {
            let balance = Self::convert_to_balance(
                shares,
                TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id),
                NodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id),
            );

            node_delegate_stake.push(NodeDelegateStakeInfo {
                subnet_id,
                subnet_node_id,
                shares,
                balance,
            })
        }
        node_delegate_stake
    }

    pub fn get_overwatch_commits_for_epoch_and_node(
        epoch: u32,
        overwatch_node_id: u32,
    ) -> Vec<(u32, T::Hash)> {  // Returns (subnet_id, commit_hash) pairs
        OverwatchCommits::<T>::iter_prefix((epoch, overwatch_node_id,))
            .collect()
    }

    pub fn get_overwatch_reveals_for_epoch_and_node(
        epoch: u32,
        overwatch_node_id: u32,
    ) -> Vec<(u32, u128)> {  // Returns (subnet_id, commit_hash) pairs
        OverwatchReveals::<T>::iter_prefix((epoch, overwatch_node_id,))
            .collect()
    }
}
