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

    pub fn get_random_number(seed: u32) -> u32 {
        let mut random_number = Self::generate_random_number(seed);

        // Best effort attempt to remove bias from modulus operator.
        let mut i = 1;
        let mut found = false;
        while !found {
            if random_number < u32::MAX {
                found = true;
                break;
            }

            random_number = Self::generate_random_number(i);

            i += 1;
        }

        random_number
    }

    // pub fn get_random_number_v2(seed: u32) -> u32 {
    //     let mut random_number = Self::generate_random_number(seed);

    //     // Best effort attempt to remove bias from modulus operator.
    //     let mut i = 1;
    //     let mut found = false;
    //     while !found {
    //         if random_number < u32::MAX {
    //             found = true;
    //             break;
    //         }

    //         random_number = Self::generate_random_number(i);

    //         i += 1;
    //     }

    //     random_number
    // }

    // If using len() for `max`, avoid overflow by `-1`
    pub fn get_random_number_with_max(mut max: u32, seed: u32) -> u32 {
        if max == 0 {
            return 0;
        }

        let mut random_number = Self::generate_random_number(seed);

        // Best effort attempt to remove bias from modulus operator.
        let mut i = 1;
        let mut found = false;
        while true {
            if random_number < u32::MAX - u32::MAX % max {
                found = true;
                break;
            }

            random_number = Self::generate_random_number(i);

            i += 1;
        }

        random_number % max
    }

    /// Generate a random number from a given seed.
    /// Note that there is potential bias introduced by using modulus operator.
    /// You should call this function with different seed values until the random
    /// number lies within `u32::MAX - u32::MAX % n`.
    /// TODO: deal with randomness freshness
    /// https://github.com/paritytech/substrate/issues/8311
    /// This is not a secure random number generator but serves its purpose for choosing random numbers
    pub fn generate_random_number(seed: u32) -> u32 {
        let (random_seed, _) = T::Randomness::random(&(T::PalletId::get(), seed).encode());
        let random_number = <u32>::decode(&mut random_seed.as_ref())
            .expect("secure hashes should always be bigger than u32; qed");

        random_number
    }

    // pub fn generate_random_number_v2(seed: u32) -> u32 {
    //     let (random_seed, _) = T::Randomness::random(&(T::PalletId::get(), seed).encode());

    //     // Handle case where random_seed might be too short
    //     if random_seed.len() < 4 {
    //         // Fallback: create a u32 from available bytes + padding
    //         let mut bytes = [0u8; 4];
    //         for (i, &byte) in random_seed.iter().take(4).enumerate() {
    //             bytes[i] = byte;
    //         }
    //         return u32::from_le_bytes(bytes);
    //     }

    //     // Try to decode, with fallback
    //     match <u32>::decode(&mut random_seed.as_ref()) {
    //         Ok(number) => number,
    //         Err(_) => {
    //             // Fallback: just take first 4 bytes manually
    //             let bytes = [
    //                 random_seed.get(0).copied().unwrap_or(0),
    //                 random_seed.get(1).copied().unwrap_or(0),
    //                 random_seed.get(2).copied().unwrap_or(0),
    //                 random_seed.get(3).copied().unwrap_or(0),
    //             ];
    //             u32::from_le_bytes(bytes)
    //         }
    //     }
    // }
}
