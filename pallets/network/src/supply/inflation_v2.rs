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

pub struct InflationV2 {
    /// Initial maximum inflation percentage, from time=0
    pub initial_max: f64,

    /// Initial minimum inflation percentage, from time=0
    pub initial_min: f64,

    /// Percentage of total inflation allocated to the foundation
    pub foundation: f64,
    /// Duration of foundation pool inflation, in years
    pub foundation_term: f64,
}

const DEFAULT_INITIAL_MAX: f64 = 100000000000000000000000.0; // 100,000
const DEFAULT_INITIAL_MIN: f64 = 75000000000000000000000.0; // 75,000
const DEFAULT_FOUNDATION: f64 = 0.2;
const DEFAULT_FOUNDATION_TERM: f64 = 7.0;

impl Default for InflationV2 {
    fn default() -> Self {
        Self {
            initial_max: DEFAULT_INITIAL_MAX,
            initial_min: DEFAULT_INITIAL_MIN,
            foundation: DEFAULT_FOUNDATION,
            foundation_term: DEFAULT_FOUNDATION_TERM,
        }
    }
}

impl InflationV2 {
    /// Get the yearly inflation rate
    ///
    /// * called by get_epoch_emissions()
    ///
    /// # Uses
    /// `initial_max`: Max interest rate
    /// `initial_min`: Min interest rate
    ///
    /// *x: Node utilization ratio
    /// *mid: Sigmoid midpoint
    /// *f: Sigmoid steepness
    /// *sigmoid_fn: Sigmoid function
    pub fn inflation_v2<F>(&self, x: f64, mid: f64, k: f64, sigmoid_fn: F) -> f64
    where
        F: Fn(f64, f64, f64) -> f64,
    {
        let max = self.initial_max;
        let min = self.initial_min;

        log::error!("max               {:?}", max);
        log::error!("min               {:?}", min);

        log::error!("min + (max - min) {:?}", min + (max - min));
        log::error!("sigmoid_fn        {:?}", sigmoid_fn(x, mid, k));

        min + (max - min) * sigmoid_fn(x, mid, k)
    }
}

impl<T: Config> Pallet<T> {
    pub fn get_inflation_v2(node_utilization: f64) -> f64 {
        let mid = Self::get_percent_as_f64(InflationSigmoidMidpoint::<T>::get());
        let k = InflationSigmoidSteepness::<T>::get() as f64;

        let inflation = InflationV2::default();

        inflation.inflation_v2(node_utilization, mid, k, Self::sigmoid_decreasing_v2)
    }

    fn get_subnet_node_utilization_v2() -> f64 {
        let max_subnets: u32 = MaxSubnets::<T>::get();
        let max_nodes: u32 = max_subnets.saturating_mul(MaxSubnetNodes::<T>::get());
        let total_active_nodes: u32 = TotalActiveNodes::<T>::get();

        (total_active_nodes as f64 / max_nodes as f64).clamp(0.0, 1.0)
    }

    pub fn get_epoch_emissions_v2() -> (u128, u128) {
        let node_utilization = Self::get_subnet_node_utilization_v2().min(1.0);
        let emissions = Self::get_inflation_v2(node_utilization);
        log::error!("emissions                   {:?}", emissions);

        let (validator_emissions, foundation_emissions) = {
            let inflation = InflationV2::default();
            (
                emissions - emissions * inflation.foundation,
                emissions * inflation.foundation,
            )
        };

        let epochs_per_year: f64 = T::EpochsPerYear::get() as f64;

        (
            (validator_emissions / epochs_per_year) as u128,
            (foundation_emissions / epochs_per_year) as u128,
        )
    }
}
