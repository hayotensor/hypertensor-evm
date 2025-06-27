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
  pub fn do_register_identity(
    coldkey: T::AccountId,
    hotkey: T::AccountId,
    name: Vec<u8>,
    url: Vec<u8>,
    image: Vec<u8>,
    discord: Vec<u8>,
    x: Vec<u8>,
    telegram: Vec<u8>,
    github: Vec<u8>,
    hugging_face: Vec<u8>,
    description: Vec<u8>,
    misc: Vec<u8>,
  ) -> DispatchResult {
    ensure!(
      HotkeyOwner::<T>::get(&hotkey) == coldkey,
      Error::<T>::NotKeyOwner
    );

    let coldkey_identity = ColdkeyIdentityData {
      name,
      url,
      image,
      discord,
      x,
      telegram,
      github,
      hugging_face,
      description,
      misc,
    };

    ColdkeyIdentity::<T>::insert(&coldkey, &coldkey_identity);

    Ok(())
  }
}