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
    pub fn do_register_or_update_identity(
        coldkey: T::AccountId,
        hotkey: T::AccountId,
        name: BoundedVec<u8, DefaultMaxVectorLength>,
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
        // Accounts that have never registered a subnet node will not have a HotkeyOwner stored
        ensure!(
            HotkeyOwner::<T>::get(&hotkey) == coldkey,
            Error::<T>::NotKeyOwner
        );

        ensure!(!name.is_empty(), Error::<T>::IdentityFieldEmpty);
        ensure!(!url.is_empty(), Error::<T>::IdentityFieldEmpty);
        ensure!(!image.is_empty(), Error::<T>::IdentityFieldEmpty);
        ensure!(!discord.is_empty(), Error::<T>::IdentityFieldEmpty);
        ensure!(!x.is_empty(), Error::<T>::IdentityFieldEmpty);
        ensure!(!telegram.is_empty(), Error::<T>::IdentityFieldEmpty);
        ensure!(!github.is_empty(), Error::<T>::IdentityFieldEmpty);
        ensure!(!hugging_face.is_empty(), Error::<T>::IdentityFieldEmpty);
        ensure!(!description.is_empty(), Error::<T>::IdentityFieldEmpty);
        ensure!(!misc.is_empty(), Error::<T>::IdentityFieldEmpty);

        if let Ok(owner) = ColdkeyIdentityNameOwner::<T>::try_get(name.clone()) {
            ensure!(owner == coldkey.clone(), Error::<T>::IdentityTaken);
        }

        // Remove previous name to ensure they can't own multiple names
        if let Ok(coldkey_identity) = ColdkeyIdentity::<T>::try_get(&coldkey) {
            ColdkeyIdentityNameOwner::<T>::remove(coldkey_identity.name);
        }

        let coldkey_identity = ColdkeyIdentityData {
            name: name.clone(),
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

        ColdkeyIdentityNameOwner::<T>::insert(name.clone(), &coldkey);
        ColdkeyIdentity::<T>::insert(&coldkey, &coldkey_identity);

        Self::deposit_event(Event::IdentityRegistered {
            coldkey: coldkey,
            identity: coldkey_identity,
        });

        Ok(())
    }

    pub fn do_remove_identity(coldkey: T::AccountId) -> DispatchResult {
        let coldkey_identity = ColdkeyIdentity::<T>::take(&coldkey);
        ColdkeyIdentityNameOwner::<T>::remove(coldkey_identity.clone().name);

        Self::deposit_event(Event::IdentityRemoved {
            coldkey: coldkey,
            identity: coldkey_identity.clone(),
        });

        Ok(())
    }
}
