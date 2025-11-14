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
//

use super::*;

impl<T: Config> Pallet<T> {
    pub fn hotkey_has_owner(hotkey: T::AccountId) -> bool {
        HotkeyOwner::<T>::contains_key(&hotkey)
    }

    /// Check if the hotkeys owner
    pub fn is_hotkey_owner(hotkey: &T::AccountId, coldkey: &T::AccountId) -> bool {
        match HotkeyOwner::<T>::try_get(&hotkey) {
            Ok(owner) => coldkey == &owner,
            Err(()) => false,
        }
    }

    // Loosely validates Node ID
    pub fn validate_peer_id(peer_id: &PeerId) -> bool {
        let peer_id_0 = &peer_id.0;
        let len = peer_id_0.len();

        // Length must be between 32 and 128 bytes
        if len < 32 || len > 128 {
            return false;
        }

        let first = peer_id_0[0];
        let second = peer_id_0[1];

        match (first, second) {
            // (ed25519, using the "identity" multihash) encoded as a raw base58btc multihash
            // '1' → base58btc identity multihash (ed25519)
            (49, _) => true,
            // (sha256) encoded as a raw base58btc multihash - 'Qm' → SHA256 base58 multihash
            (81, 109) => true,
            // (sha256) encoded as a CID - 'f', 'b', 'z', 'm' → CID/base multibase prefixes
            (102, _) | (98, _) | (122, _) | (109, _) => true,
            _ => false,
        }
    }
}
