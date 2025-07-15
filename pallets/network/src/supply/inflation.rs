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
// Enables accounts to delegate stake to subnets for a portion of emissions

use super::*;
use libm::{exp, pow};
use sp_runtime::FixedU128;
use sp_runtime::traits::{Saturating, CheckedDiv, CheckedMul};
use frame_support::pallet_prelude::{One, Zero};
use sp_runtime::FixedPointNumber;

pub struct Inflation {
  /// Initial inflation percentage, from time=0
  pub initial: f64,

  /// Terminal inflation percentage, to time=INF
  pub terminal: f64,

  /// Rate per year, at which inflation is lowered until reaching terminal
  ///  i.e. inflation(year) == MAX(terminal, initial*((1-taper)^year))
  pub taper: f64,

  /// Percentage of total inflation allocated to the foundation
  pub foundation: f64,
  /// Duration of foundation pool inflation, in years
  pub foundation_term: f64,
}

const DEFAULT_INITIAL: f64 = 0.1;
const DEFAULT_TERMINAL: f64 = 0.015;
const DEFAULT_TAPER: f64 = 0.15;
const DEFAULT_FOUNDATION: f64 = 0.05;
const DEFAULT_FOUNDATION_TERM: f64 = 6.0;

impl Default for Inflation {
  fn default() -> Self {
    Self {
      initial: DEFAULT_INITIAL,
      terminal: DEFAULT_TERMINAL,
      taper: DEFAULT_TAPER,
      foundation: DEFAULT_FOUNDATION,
      foundation_term: DEFAULT_FOUNDATION_TERM,
    }
  }
}

impl Inflation {
  pub fn epoch(&self, epoch: u32, epochs_per_year: u32, denominator: u128) -> f64 {
    let years_elapsed = epoch as f64 / epochs_per_year as f64;
    // let rate = self.initial * pow(1.0 - self.terminal, years_elapsed);

    self.total(years_elapsed)

    // Ensure inflation does not go below the minimum taper rate
    // let final_rate = rate.max(self.taper);

    // final_rate
    // final_rate as u128 * denominator
  }

  /// portion of total that goes to validators
  pub fn validator(&self, year: f64) -> f64 {
    self.total(year) - self.foundation(year)
  }

  /// portion of total that goes to foundation
  pub fn foundation(&self, year: f64) -> f64 {
    if year < self.foundation_term {
      self.total(year) * self.foundation
    } else {
      0.0
    }
  }

  /// inflation rate at year
  pub fn total(&self, year: f64) -> f64 {
    // let tapered = self.initial * pow(1.0 - self.taper, year);
    let tapered = self.initial * pow(1.0, year);

    if tapered > self.terminal {
      tapered
    } else {
      self.terminal
    }
  }

  pub fn year_from_epoch(&self, epoch: u32, epochs_per_year: u32) -> f64 {
    epoch as f64 / epochs_per_year as f64
  }

  /// Get the yearly inflation rate
  ///
  /// * called by get_epoch_emissions()
  ///
  /// # Uses 
  /// `initial`: Max interest rate
  /// `terminal`: Min interest rate
  ///
  /// *u: Node utilization ratio
  /// *mid: Sigmoid midpoint
  /// *f: Sigmoid steepness
  pub fn inflation(&self, u: f64, mid: f64, k: f64) -> f64 {
    let c = (u - mid).abs();
    let d = k * c;
    let exp = libm::exp(d);
    let sigmoid = if u > mid {
      1.0/(1.0+exp)
    } else {
      exp/(1.0+exp)
    };

    self.terminal-(self.terminal-self.initial)*sigmoid
  }
}

impl<T: Config> Pallet<T> {
  pub fn get_inflation(node_utilization: f64) -> f64 {
    let mid = Self::get_percent_as_f64(SigmoidMidpoint::<T>::get());
    let k = SigmoidSteepness::<T>::get() as f64;

    let inflation = Inflation::default();

    inflation.inflation(node_utilization, mid, k)
  }

  pub fn get_epoch_inflation_rate(
    node_utilization: f64,
  ) -> f64 {
    let yearly_rate: f64 = Self::get_inflation(node_utilization);
    let epochs_per_year: f64 = T::EpochsPerYear::get() as f64;

    yearly_rate / epochs_per_year
  }

  fn get_subnet_node_utilization() -> f64 {
    let max_subnets: u32 = MaxSubnets::<T>::get();
    let max_nodes: u32 = max_subnets.saturating_mul(MaxSubnetNodes::<T>::get());
    let total_active_nodes: u32 = TotalActiveNodes::<T>::get();

    (total_active_nodes as f64 / max_nodes as f64).clamp(0.0, 1.0)
  }

  pub fn get_epoch_emissions() -> u128 {
    // Get epoch inflation rate
    let epoch_rate: f64 = Self::get_epoch_inflation_rate(
      Self::get_subnet_node_utilization().min(1.0)
    );

    // Placer 
    // TODO: Get total network issuance
    let total_issuance: f64 = 100000000e+18 as f64;

    (total_issuance * epoch_rate) as u128
  }
}