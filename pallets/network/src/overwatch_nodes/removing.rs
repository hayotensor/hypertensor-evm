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
  pub fn do_remove_ow(
    overwatch_node_id: u32,
  ) -> DispatchResult {
    let overwatch_node = match OverwatchNodes::<T>::try_get(overwatch_node_id) {
      Ok(overwatch_node) => overwatch_node,
      Err(()) => return Err(Error::<T>::InvalidOverwatchNode.into()),
    };

    OverwatchNodes::<T>::remove(overwatch_node_id);
    if let Some(hotkey) = OverwatchNodeIdHotkey::<T>::take(overwatch_node_id) {
      HotkeyOverwatchNodeId::<T>::remove(&hotkey);
    };

    // Remove all peer IDs in all subnets
    let map = OverwatchNodeIndex::<T>::take(overwatch_node_id);
    for (subnet_id, peer_id) in map {
      PeerIdOverwatchNode::<T>::remove(subnet_id, peer_id);
    }

    Ok(())
  }
}