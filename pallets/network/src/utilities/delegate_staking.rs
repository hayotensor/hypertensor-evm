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
    /// The minimum delegate stake balance for a subnet to stay live
    /// Get total required subnet nodes based on total nodes
    ///
    /// We use electable node count to ensure users can spam registered
    /// node slots or spam active node slots without being in-consensus
    /// to being an actual node
    pub fn get_min_subnet_delegate_stake_balance_v2(subnet_id: u32) -> u128 {
        let total_network_issuance = Self::get_total_network_issuance();
        let factor: u128 = MinSubnetDelegateStakeFactor::<T>::get(); // 0.1%
        let base_min = Self::percent_mul(total_network_issuance, factor);

        let electable_node_count = SubnetNodeElectionSlots::<T>::get(subnet_id).len() as u32;
        let multiplier = Self::get_subnet_min_delegate_staking_multiplier(electable_node_count);

        Self::percent_mul(base_min, multiplier)
    }

    /// Get the subnet's minimum delegate stake multiplier based on the current electable nodes count.
    ///
    /// Returns a multiplier that scales linearly between:
    /// - `min_multiplier` (100%) when `electable_node_count <= MinSubnetNodes`
    /// - `max_multiplier` (MaxMinDelegateStakeMultiplier) when `electable_node_count >= MaxSubnetNodes`
    ///
    /// For counts between min and max nodes, the multiplier is interpolated proportionally.
    ///
    /// # Example
    /// If MinSubnetNodes = 10, MaxSubnetNodes = 100, and MaxMinDelegateStakeMultiplier = 500%:
    /// - At 10 nodes: returns 100%
    /// - At 55 nodes (midpoint): returns ~300%
    /// - At 100+ nodes: returns 500%
    /// The return value is then applied to the base minimum delegate stake balance
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

        let node_delta = electable_node_count.saturating_sub(min_nodes);
        let range = max_nodes.saturating_sub(min_nodes);

        let ratio = Self::percent_div(node_delta as u128, range as u128);
        let delta = max_multiplier.saturating_sub(min_multiplier);

        min_multiplier.saturating_add(Self::percent_mul(ratio, delta))
    }
}
