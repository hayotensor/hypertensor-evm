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
use sp_runtime::traits::Saturating;

pub struct Inflation {
    /// Initial maximum inflation percentage, from time=0
    pub initial_max: f64,

    /// Initial minimum inflation percentage, from time=0
    pub initial_min: f64,

    /// Terminal inflation percentage, to time=INF
    pub terminal: f64,

    /// Rate per year, at which inflation is lowered until reaching terminal
    ///  i.e. inflation(year) == MAX(terminal, initial_max*((1-taper)^year))
    pub taper: f64,

    /// Percentage of total inflation allocated to the foundation
    pub foundation: f64,
    /// Duration of foundation pool inflation, in years
    pub foundation_term: f64,
}

const DEFAULT_INITIAL_MAX: f64 = 0.1;
const DEFAULT_INITIAL_MIN: f64 = 0.045;
const DEFAULT_TERMINAL: f64 = 0.015;
const DEFAULT_TAPER: f64 = 0.0369;
const DEFAULT_FOUNDATION: f64 = 0.2;
const DEFAULT_FOUNDATION_TERM: f64 = 7.0;

impl Default for Inflation {
    fn default() -> Self {
        Self {
            initial_max: DEFAULT_INITIAL_MAX,
            initial_min: DEFAULT_INITIAL_MIN,
            terminal: DEFAULT_TERMINAL,
            taper: DEFAULT_TAPER,
            foundation: DEFAULT_FOUNDATION,
            foundation_term: DEFAULT_FOUNDATION_TERM,
        }
    }
}

impl Inflation {
    /// portion of total that goes to validators
    pub fn validator(&self, u: f64, mid: f64, k: f64, year: f64) -> f64 {
        self.total(u, mid, k, year) - self.foundation(u, mid, k, year)
    }

    /// portion of total that goes to foundation
    pub fn foundation(&self, u: f64, mid: f64, k: f64, year: f64) -> f64 {
        if year < self.foundation_term {
            self.total(u, mid, k, year) * self.foundation
        } else {
            0.0
        }
    }

    /// inflation rate at year
    pub fn total(&self, u: f64, mid: f64, k: f64, year: f64) -> f64 {
        let c = (u - mid).abs();
        let d = k * c;
        let exp = exp(d);
        let sigmoid = if u > mid {
            1.0 / (1.0 + exp)
        } else {
            exp / (1.0 + exp)
        };

        let max = self.current_max_rate(year);

        self.terminal + (max - self.terminal) * sigmoid
    }

    pub fn year_from_epoch(&self, epoch: u32, epochs_per_year: u32) -> f64 {
        epoch as f64 / epochs_per_year as f64
    }

    pub fn current_max_rate(&self, year: f64) -> f64 {
        let tapered = self.initial_max * pow(1.0 - self.taper, year);

        if tapered > self.terminal {
            tapered
        } else {
            self.terminal
        }
    }

    pub fn current_min_rate(&self, year: f64) -> f64 {
        let tapered = self.initial_min * pow(1.0 - self.taper, year);

        if tapered > self.terminal {
            tapered
        } else {
            self.terminal
        }
    }

    /// Get the yearly inflation rate
    ///
    /// * called by get_epoch_emissions()
    ///
    /// # Uses
    /// `initial_max`: Max interest rate
    /// `initial_min`: Min interest rate
    /// `terminal`: Min interest rate
    ///
    /// *u: Node utilization ratio
    /// *mid: Sigmoid midpoint
    /// *f: Sigmoid steepness
    pub fn inflation(&self, u: f64, mid: f64, k: f64, year: f64) -> f64 {
        let c = (u - mid).abs();
        let d = k * c;
        let exp = exp(d);
        let sigmoid = if u > mid {
            1.0 / (1.0 + exp)
        } else {
            exp / (1.0 + exp)
        };

        let max = self.current_max_rate(year);

        if max == self.terminal {
            return max;
        }

        let min = self.current_min_rate(year);

        min + (max - min) * sigmoid
    }
}

impl<T: Config> Pallet<T> {
    pub fn get_inflation(node_utilization: f64, year: f64) -> f64 {
        let mid = Self::get_percent_as_f64(SigmoidMidpoint::<T>::get());
        let k = SigmoidSteepness::<T>::get() as f64;

        let inflation = Inflation::default();

        inflation.inflation(node_utilization, mid, k, year)
    }

    pub fn get_epoch_inflation_rate(epoch: u32, node_utilization: f64) -> f64 {
        let epochs_per_year: f64 = T::EpochsPerYear::get() as f64;
        let year: f64 = epoch as f64 / epochs_per_year;
        let yearly_rate: f64 = Self::get_inflation(node_utilization, year);

        yearly_rate / epochs_per_year
    }

    fn get_subnet_node_utilization() -> f64 {
        let max_subnets: u32 = MaxSubnets::<T>::get();
        let max_nodes: u32 = max_subnets.saturating_mul(MaxSubnetNodes::<T>::get());
        let total_active_nodes: u32 = TotalActiveNodes::<T>::get();

        (total_active_nodes as f64 / max_nodes as f64).clamp(0.0, 1.0)
    }

    /// Get the epochs emissions for validators and foundation
    ///
    /// # Arguments
    ///
    /// * `epoch` - Current epoch
    ///
    /// # Returns
    ///
    /// *(validator emissions, foundation emissions)
    ///
    pub fn get_epoch_emissions_v2(epoch: u32) -> (u128, u128) {
        let mid = Self::get_percent_as_f64(SigmoidMidpoint::<T>::get());
        let k = SigmoidSteepness::<T>::get() as f64;

        let epochs_per_year: f64 = T::EpochsPerYear::get() as f64;
        let year: f64 = epoch as f64 / epochs_per_year;
        let node_utilization = Self::get_subnet_node_utilization().min(1.0);

        let (validator_rate, foundation_rate) = {
            let inflation = Inflation::default();
            (
                (inflation).validator(node_utilization, mid, k, year),
                (inflation).foundation(node_utilization, mid, k, year),
            )
        };

        let total_issuance: f64 = 100000000e+18 as f64;
        let validator_epoch_rate = validator_rate / epochs_per_year;
        let foundation_epoch_rate = foundation_rate / epochs_per_year;

        (
            (total_issuance * validator_epoch_rate) as u128,
            (total_issuance * validator_epoch_rate) as u128,
        )
    }
}
