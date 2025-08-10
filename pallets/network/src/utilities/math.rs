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
  pub const PERCENTAGE_FACTOR: U256 = U256([0xde0b6b3a7640000, 0x0, 0x0, 0x0]);
  pub const HALF_PERCENT: U256 = U256([0x06f05b59d3b20000, 0x0, 0x0, 0x0]);

  /// Inspired by Aave PercentageMath

  /// `x` is value
  /// `y` is percentage
  /// Rounds down to the nearest 10th decimal
  pub fn percent_mul(x: u128, y: u128) -> u128 {
    if x == 0 || y == 0 {
      return 0
    }
    
    let x = U256::from(x);
    let y = U256::from(y);

    if x > ((U256::MAX - Self::HALF_PERCENT) / y) {
      return 0
    }

    // x * y / 100.0
    let result = x * y / Self::PERCENTAGE_FACTOR;

    if result > U256::from(u128::MAX) {
      // return 0
      return u128::MAX
    }

    result.try_into().unwrap_or(u128::MAX)
  }

  /// `x` is value
  /// `y` is percentage
  /// Rounds down to the nearest 10th decimal
  pub fn percent_div(x: u128, y: u128) -> u128 {
    if x == 0 || y == 0 {
      return 0
    }
    
    let x = U256::from(x);
    let y = U256::from(y);

    // x * 100.0 / y
    let result = x * Self::PERCENTAGE_FACTOR / y;

    // if result > U256::from(u128::MAX) {
    //   return 0;
    // }

    result.try_into().unwrap_or(u128::MAX)
  }

  pub fn percentage_factor_as_u128() -> u128 {
    1_000_000_000_000_000_000
  }

  /// Get percentage in decimal format that uses `PERCENTAGE_FACTOR` as f64
  pub fn get_percent_as_f64(v: u128) -> f64 {
    v as f64 / Self::percentage_factor_as_u128() as f64
  }

  pub fn get_f64_as_percentage(v: f64) -> u128 {
    (v * Self::percentage_factor_as_u128() as f64) as u128
  }

  pub fn pow(x: f64, exp: f64) -> f64 {
    pow(x, exp)
  }

  pub fn checked_mul_div(x: U256, y: U256, z: U256) -> Option<U256> {
    if z.is_zero() {
      return None;
    }
    x.checked_mul(y)?.checked_div(z)
  }
}