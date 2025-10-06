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
    pub fn get_random_number_v2(seed: u32) -> u32 {
        let mut random_number = Self::generate_random_number(seed);
        let mut i = 1;
        const MAX_ATTEMPTS: u32 = 10; // Prevent infinite loop

        while random_number == u32::MAX && i < MAX_ATTEMPTS {
            random_number = Self::generate_random_number(seed.wrapping_add(i));
            i += 1;
        }

        random_number
    }

    pub fn get_random_number(seed: u32, total: u32) -> u32 {
        let random_number = Self::generate_random_number(seed);
        random_number % total
    }

    /// Generate a random number from a given seed.
    /// Note that there is potential bias introduced by using modulus operator.
    /// You should call this function with different seed values until the random
    /// number lies within `u32::MAX - u32::MAX % n`.
    /// TODO: deal with randomness freshness
    /// https://github.com/paritytech/substrate/issues/8311
    /// This is not a secure random number generator but serves its purpose for choosing random numbers
    pub fn generate_random_number_old(seed: u32) -> u32 {
        let (random_seed, _) = T::Randomness::random(&(T::PalletId::get(), seed).encode());
        let random_number = <u32>::decode(&mut random_seed.as_ref())
            .expect("secure hashes should always be bigger than u32; qed");

        random_number
    }

    pub fn generate_random_number(seed: u32) -> u32 {
        let (random_seed, _) = T::Randomness::random(&(T::PalletId::get(), seed).encode());

        // Take first 4 bytes and interpret as u32
        let bytes = random_seed.as_ref();
        let mut array = [0u8; 4];
        array.copy_from_slice(&bytes[0..4]);

        u32::from_le_bytes(array)
    }
}
