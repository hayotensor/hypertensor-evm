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
    name: BoundedVec<u8, DefaultMaxUrlLength>,
    url: BoundedVec<u8, DefaultMaxUrlLength>,
    image: BoundedVec<u8, DefaultMaxUrlLength>,
    discord: BoundedVec<u8, DefaultMaxSocialIdLength>,
    x: BoundedVec<u8, DefaultMaxSocialIdLength>,
    telegram: BoundedVec<u8, DefaultMaxSocialIdLength>,
    github: BoundedVec<u8, DefaultMaxUrlLength>,
    hugging_face: BoundedVec<u8, DefaultMaxUrlLength>,
    description: BoundedVec<u8, DefaultMaxVectorLength>,
    misc: BoundedVec<u8, DefaultMaxVectorLength>,
  ) -> DispatchResult {
    // --- Ensure is or has had a subnet node
    // This will not completely stop non-subnet-node users from registering identities but prevents it
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

  pub fn do_remove_identity(
    coldkey: T::AccountId,
  ) -> DispatchResult {
    ColdkeyIdentity::<T>::remove(&coldkey);
    Ok(())
  }
}