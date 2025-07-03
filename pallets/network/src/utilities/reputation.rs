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

impl<T: Config> Pallet<T> {
  pub fn increase_coldkey_reputation(
    coldkey: T::AccountId,
    attestation_percentage: u128,
    min_attestation_percentage: u128,
    increase_weight_factor: u128, // this is the steepness multiplier
    epoch: u32,
  ) {
    if attestation_percentage < min_attestation_percentage {
      return
    }

    let percentage_factor = Self::percentage_factor_as_u128();
    let mut coldkey_reputation = ColdkeyReputation::<T>::get(&coldkey);
    let current_weight = coldkey_reputation.weight;

    // Stop early if weight is already maxed out
    if current_weight >= percentage_factor {
      return
    }

    // Reward factor decreases as weight increases (to make it harder to max out)
    let reward_factor: u128 = percentage_factor
      .saturating_mul(percentage_factor)
      / current_weight.saturating_add(1);

    // Compute nominal increase
    let nominal_increase: u128 = (attestation_percentage - min_attestation_percentage)
      .saturating_mul(increase_weight_factor)
      .saturating_mul(reward_factor)
      / percentage_factor
      / percentage_factor;

    if nominal_increase == 0 {
        return;
    }

    let new_weight = current_weight.saturating_add(nominal_increase).min(percentage_factor);
    if new_weight == current_weight {
        return;
    }

    // Update fields
    coldkey_reputation.weight = new_weight;
    coldkey_reputation.total_increases += 1;
    coldkey_reputation.last_validator_epoch = epoch;

    if coldkey_reputation.start_epoch == 0 {
      coldkey_reputation.start_epoch = epoch;
    }

    // Update average attestation
    let prev_total = coldkey_reputation
      .total_increases
      .saturating_add(coldkey_reputation.total_decreases)
      .saturating_sub(1) as u128;

    coldkey_reputation.average_attestation = if prev_total == 0 {
      attestation_percentage
    } else {
      (coldkey_reputation.average_attestation * prev_total + attestation_percentage)
        / (prev_total + 1)
    };

    ColdkeyReputation::<T>::insert(&coldkey, coldkey_reputation);
  }

  pub fn decrease_coldkey_reputation(
    coldkey: T::AccountId,
    attestation_percentage: u128,
    min_attestation_percentage: u128,
    decrease_weight_factor: u128, // <- slope/steepness control
    epoch: u32,
  ) {
    if attestation_percentage >= min_attestation_percentage {
      return
    }

    let percentage_factor = Self::percentage_factor_as_u128();
    let mut coldkey_reputation = ColdkeyReputation::<T>::get(&coldkey);
    let current_weight = coldkey_reputation.weight;

    // Remove node / Avoid division by zero
    if current_weight == 0 {
      return
    }

    // Penalty increases as weight increases (same pattern as reward logic)
    let penalty_factor: u128 = percentage_factor
        .saturating_mul(percentage_factor)
        / current_weight.saturating_add(1);

    // Calculate nominal decrease: how much worse than threshold * weight * penalty
    let nominal_decrease: u128 = (min_attestation_percentage - attestation_percentage)
        .saturating_mul(decrease_weight_factor)
        .saturating_mul(penalty_factor)
        / percentage_factor
        / percentage_factor;

    if nominal_decrease == 0 {
      return
    }

    let new_weight = current_weight.saturating_sub(nominal_decrease);
    if new_weight == current_weight {
      return
    }

    coldkey_reputation.weight = new_weight;
    coldkey_reputation.total_decreases += 1;
    coldkey_reputation.last_validator_epoch = epoch;

    if coldkey_reputation.start_epoch == 0 {
      coldkey_reputation.start_epoch = epoch;
    }

    let prev_total = coldkey_reputation
      .total_increases
      .saturating_add(coldkey_reputation.total_decreases)
      .saturating_sub(1) as u128;

    coldkey_reputation.average_attestation = if prev_total == 0 {
      attestation_percentage
    } else {
      (coldkey_reputation.average_attestation * prev_total + attestation_percentage)
        / (prev_total + 1)
    };

    ColdkeyReputation::<T>::insert(&coldkey, coldkey_reputation);
  }
}