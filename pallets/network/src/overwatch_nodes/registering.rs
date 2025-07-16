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

impl<T: Config> Pallet<T> {
  pub fn do_register_ow(
    origin: T::RuntimeOrigin,
    hotkey: T::AccountId,
    stake_to_be_added: u128,
  ) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    // ⸺ Register fresh hotkey
    ensure!(
      !Self::hotkey_has_owner(hotkey.clone()),
      Error::<T>::HotkeyHasOwner
    );

    let mut hotkeys = ColdkeyHotkeys::<T>::get(&coldkey);
    ensure!(
      !hotkeys.contains(&hotkey),
      Error::<T>::HotkeyAlreadyRegisteredToColdkey
    );

    // Insert coldkey -> hotkeys
    hotkeys.insert(hotkey.clone());
    ColdkeyHotkeys::<T>::insert(&coldkey, hotkeys);

    // ⸺ Ensure qualifies via reputation
    let reputation = ColdkeyReputation::<T>::get(&coldkey);

    // ⸺ Stake
    Self::do_add_overwatch_stake(
      coldkey.clone(), 
      hotkey.clone(),
      stake_to_be_added,
    ).map_err(|e| e)?;

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

    Ok(())
  }

  pub fn do_set_ow_peer_id(
    origin: T::RuntimeOrigin,
    subnet_id: u32,
    overwatch_node_id: u32,
    peer_id: PeerId
  ) -> DispatchResult {
    let key: T::AccountId = ensure_signed(origin)?;

    let subnet = match SubnetsData::<T>::try_get(subnet_id) {
      Ok(subnet) => subnet,
      Err(()) => return Err(Error::<T>::InvalidSubnet.into()),
    };

    ensure!(
      Self::is_overwatch_node_keys_owner(
        overwatch_node_id, 
        key, 
      ),
      Error::<T>::NotKeyOwner
    );
   
    ensure!(
      Self::validate_peer_id(&peer_id),
      Error::<T>::InvalidPeerId
    );

    ensure!(
      Self::is_owner_of_peer_or_ownerless(subnet_id, 0, overwatch_node_id, &peer_id),
      Error::<T>::PeerIdExist
    );

    PeerIdOverwatchNode::<T>::insert(subnet_id, &peer_id, overwatch_node_id);

    // Add or replace PeerID under subnet ID
    OverwatchNodeIndex::<T>::mutate(overwatch_node_id, |map| {
      map.insert(subnet_id, peer_id);
    });

    Ok(())
  }
}