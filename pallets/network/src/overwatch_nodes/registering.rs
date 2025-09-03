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
    pub fn do_register_overwatch_node(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        stake_to_be_added: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::get_current_overwatch_epoch_as_u32() > 0,
            Error::<T>::OverwatchEpochIsZero
        );

        let total_overwatch_nodes = TotalOverwatchNodes::<T>::get();

        ensure!(
            total_overwatch_nodes < MaxOverwatchNodes::<T>::get(),
            Error::<T>::MaxOverwatchNodes
        );

        ensure!(&hotkey != &coldkey, Error::<T>::ColdkeyMatchesHotkey);

        // ⸺ Register fresh hotkey
        ensure!(
            !Self::hotkey_has_owner(hotkey.clone()),
            Error::<T>::HotkeyHasOwner
        );

        let mut hotkeys = ColdkeyHotkeys::<T>::get(&coldkey);
        // Redundant
        ensure!(
            !hotkeys.contains(&hotkey),
            Error::<T>::HotkeyAlreadyRegisteredToColdkey
        );

        // Insert coldkey -> hotkeys
        hotkeys.insert(hotkey.clone());
        ColdkeyHotkeys::<T>::insert(&coldkey, hotkeys);

        // ⸺ Ensure qualifies via reputation
        let reputation = ColdkeyReputation::<T>::get(&coldkey);

        ensure!(
            Self::is_overwatch_node_qualified(&coldkey),
            Error::<T>::ColdkeyNotOverwatchQualified
        );

        // ⸺ Stake
        Self::do_add_overwatch_stake(coldkey.clone(), hotkey.clone(), stake_to_be_added)
            .map_err(|e| e)?;

        // ⸺ Register
        TotalOverwatchNodeUids::<T>::mutate(|n: &mut u32| *n += 1);
        let current_uid = TotalOverwatchNodeUids::<T>::get();

        HotkeyOwner::<T>::insert(&hotkey, &coldkey);
        HotkeyOverwatchNodeId::<T>::insert(&hotkey, current_uid);

        let overwatch_node: OverwatchNode<T::AccountId> = OverwatchNode {
            id: current_uid,
            hotkey: hotkey.clone(),
        };

        OverwatchNodeIdHotkey::<T>::insert(current_uid, hotkey.clone());
        OverwatchNodes::<T>::insert(current_uid, overwatch_node);

        TotalOverwatchNodes::<T>::mutate(|n: &mut u32| *n += 1);

        Ok(())
    }

    pub fn do_set_overwatch_node_peer_id(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        overwatch_node_id: u32,
        peer_id: PeerId,
    ) -> DispatchResult {
        let key: T::AccountId = ensure_signed(origin)?;

        let subnet = match SubnetsData::<T>::try_get(subnet_id) {
            Ok(subnet) => subnet,
            Err(()) => return Err(Error::<T>::InvalidSubnetId.into()),
        };

        ensure!(
            Self::is_overwatch_node_keys_owner(overwatch_node_id, key),
            Error::<T>::NotKeyOwner
        );

        ensure!(Self::validate_peer_id(&peer_id), Error::<T>::InvalidPeerId);

        // Ensure no one owns the peer Id and we don't already own it
        ensure!(
            Self::is_owner_of_peer_or_ownerless(subnet_id, 0, 0, &peer_id),
            Error::<T>::PeerIdExist
        );

        PeerIdOverwatchNode::<T>::insert(subnet_id, &peer_id, overwatch_node_id);

        // Add or replace PeerID under subnet ID
        OverwatchNodeIndex::<T>::mutate(overwatch_node_id, |map| {
            map.insert(subnet_id, peer_id);
        });

        Ok(())
    }

    pub fn is_overwatch_node_qualified(coldkey: &T::AccountId) -> bool {
        let reputation = match ColdkeyReputation::<T>::try_get(coldkey) {
            Ok(value) => value,
            Err(_) => return false,
        };
        let min_diversification_ratio = OverwatchMinDiversificationRatio::<T>::get();
        let min_score = OverwatchMinRepScore::<T>::get();
        let min_avg_attestation = OverwatchMinAvgAttestationRatio::<T>::get();
        let min_age = OverwatchMinAge::<T>::get();

        let current_epoch = Self::get_current_epoch_as_u32();

        // - No one can be an Overwatch Node yet
        if current_epoch <= min_age {
            // log::error!("current_epoch <= min_age");
            return false;
        }

        let age = current_epoch.saturating_sub(reputation.start_epoch);

        if age < min_age {
            // log::error!("age < min_age");
            return false;
        }

        if reputation.score < min_score {
            // log::error!("score < min_score");
            return false;
        }

        Self::clean_coldkey_subnet_nodes(coldkey.clone());

        // Get number of nodes under coldkey
        let mut active_unique_node_count = 0;
        ColdkeySubnetNodes::<T>::mutate(coldkey, |colkey_map| {
            for (subnet_id, nodes) in colkey_map.iter_mut() {
                let node_ids: Vec<u32> = nodes.iter().copied().collect();
                // log::error!("subnet_id {:?}", subnet_id);
                // log::error!("node_ids {:?}", node_ids);

                // Process each node_id one by one
                for node_id in node_ids {
                    if !Self::get_active_subnet_node(*subnet_id, node_id).is_none() {
                        // log::error!("get_active_subnet_node");
                        active_unique_node_count += 1;
                        // `break` to next subnet
                        break;
                    }
                }
            }
        });

        // log::error!("active_unique_node_count       {:?}", active_unique_node_count);
        // log::error!("TotalActiveSubnets::<T>::get() {:?}", TotalActiveSubnets::<T>::get());

        let diversification = match active_unique_node_count >= TotalActiveSubnets::<T>::get() {
            true => Self::percentage_factor_as_u128(),
            false => Self::percent_div(
                active_unique_node_count as u128,
                TotalActiveSubnets::<T>::get() as u128,
            ),
        };
        // log::error!("diversification                {:?}", diversification);

        if diversification < min_diversification_ratio {
            // log::error!("active_unique_node_count       {:?}", active_unique_node_count);
            // log::error!("diversification < min_diversification_ratio");
            return false;
        }

        if reputation.average_attestation < min_avg_attestation {
            // log::error!("average_attestation < min_avg_attestation");
            return false;
        }

        true
    }
}
