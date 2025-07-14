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
use libm::pow;
use sp_core::U256;

impl<T: Config> Pallet<T> {
  pub fn hotkey_has_owner(hotkey: T::AccountId) -> bool {
    match HotkeyOwner::<T>::try_get(&hotkey) {
      Ok(_) => true,
      Err(()) => false,
    }
  }
}