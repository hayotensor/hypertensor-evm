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
    /// Increase coldkey reptuation
    ///
    /// # Arguments
    ///
    /// * `coldkey` - Nodes coldkey
    /// * `attestation_percentage` - The attestation ratio of the validator nodes consensus
    /// * `min_attestation_percentage` - Blockchains minimum attestation percentage (66%)
    /// * `decrease_weight_factor` - `ColdkeyReputationIncreaseFactor`.
    /// * `epoch`: The blockchains general epoch
    pub fn increase_coldkey_reputation(
        coldkey: T::AccountId,
        attestation_percentage: u128,
        min_attestation_percentage: u128,
        increase_weight_factor: u128,
        epoch: u32,
    ) {
        if !ColdkeyReputation::<T>::contains_key(&coldkey) {
            return;
        }

        if attestation_percentage < min_attestation_percentage {
            return;
        }

        let percentage_factor = Self::percentage_factor_as_u128();
        let mut coldkey_reputation = ColdkeyReputation::<T>::get(&coldkey);
        let current_score = coldkey_reputation.score;

        // Stop early if weight is already maxed out
        if current_score >= percentage_factor {
            return;
        }

        // Reward factor decreases as weight increases (to make it harder to max out)
        // 1999999999999999996 = 1e18 * 1e18 / (.5e18 + 1)
        let reward_factor: u128 = percentage_factor
            .saturating_mul(percentage_factor)
            .saturating_div(current_score.saturating_add(1));

        // Compute nominal increase
        // Examples
        // 339999999999999999 = (1e18 - .66e18) * .5e18 * 1999999999999999996 / 1e18 / 1e18
        // 39999999999999999 = (.7e18 - .66e18) * .5e18 * 1999999999999999996 / 1e18 / 1e18
        let nominal_increase: u128 = (attestation_percentage - min_attestation_percentage)
            .saturating_mul(increase_weight_factor)
            .saturating_mul(reward_factor)
            .saturating_div(percentage_factor)
            .saturating_div(percentage_factor);

        if nominal_increase == 0 {
            return;
        }

        let new_weight = current_score
            .saturating_add(nominal_increase)
            .min(percentage_factor);
        if new_weight == current_score {
            return;
        }

        // Update fields
        coldkey_reputation.score = new_weight;
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
            (coldkey_reputation
                .average_attestation
                .saturating_mul(prev_total)
                .saturating_add(attestation_percentage))
            .saturating_div(prev_total + 1)
        };

        ColdkeyReputation::<T>::insert(&coldkey, coldkey_reputation);
    }

    /// Decrease coldkey reptuation
    ///
    /// # Arguments
    ///
    /// * `coldkey` - Nodes coldkey
    /// * `attestation_percentage` - The attestation ratio of the validator nodes consensus
    /// * `min_attestation_percentage` - Blockchains minimum attestation percentage (66%)
    /// * `decrease_weight_factor` - `ColdkeyReputationDecreaseFactor`.
    /// * `epoch`: The blockchains general epoch
    pub fn decrease_coldkey_reputation(
        coldkey: T::AccountId,
        attestation_percentage: u128,
        min_attestation_percentage: u128,
        decrease_weight_factor: u128, // <- slope/steepness control
        epoch: u32,
    ) {
        if !ColdkeyReputation::<T>::contains_key(&coldkey) {
            return;
        }

        if attestation_percentage >= min_attestation_percentage {
            return;
        }

        let percentage_factor = Self::percentage_factor_as_u128();
        // Safe get, has Default value
        let mut coldkey_reputation = ColdkeyReputation::<T>::get(&coldkey);
        let current_score = coldkey_reputation.score;

        // Remove node / Avoid division by zero
        if current_score == 0 {
            return;
        }

        // Penalty increases as score increases (same pattern as reward logic)
        let penalty_factor: u128 = percentage_factor
            .saturating_mul(percentage_factor)
            .saturating_div(current_score.saturating_add(1));

        // Calculate nominal decrease: how much worse than threshold * score * penalty
        let nominal_decrease: u128 = (min_attestation_percentage - attestation_percentage)
            .saturating_mul(decrease_weight_factor)
            .saturating_mul(penalty_factor)
            .saturating_div(percentage_factor)
            .saturating_div(percentage_factor);

        if nominal_decrease == 0 {
            return;
        }

        let new_weight = current_score.saturating_sub(nominal_decrease);
        if new_weight == current_score {
            return;
        }

        coldkey_reputation.score = new_weight;
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
            (coldkey_reputation
                .average_attestation
                .saturating_mul(prev_total)
                .saturating_add(attestation_percentage))
            .saturating_div(prev_total + 1)
        };

        ColdkeyReputation::<T>::insert(&coldkey, coldkey_reputation);
    }

    pub fn increase_node_reputation(subnet_id: u32, subnet_node_id: u32, factor: u128) {
        let mut reputation = SubnetNodeReputation::<T>::get(subnet_id, subnet_node_id);
        reputation = Self::get_increase_reputation(reputation, factor);
        SubnetNodeReputation::<T>::insert(subnet_id, subnet_node_id, reputation);
    }

    pub fn increase_and_return_node_reputation(
        subnet_id: u32,
        subnet_node_id: u32,
        current_reputation: u128,
        factor: u128,
    ) -> u128 {
        let new_reputation = Self::get_increase_reputation(current_reputation, factor);
        SubnetNodeReputation::<T>::insert(subnet_id, subnet_node_id, new_reputation);
        Self::deposit_event(Event::NodeReputationUpdate {
            subnet_id,
            subnet_node_id,
            prev_reputation: current_reputation,
            new_reputation,
        });

        new_reputation
    }

    pub fn get_increase_reputation(prev_reputation: u128, factor: u128) -> u128 {
        let one = Self::percentage_factor_as_u128();
        let factor = factor.min(one);
        let delta = Self::percent_mul(one.saturating_sub(prev_reputation), factor);
        prev_reputation.saturating_add(delta).min(one)
    }

    pub fn decrease_node_reputation(subnet_id: u32, subnet_node_id: u32, factor: u128) -> u128 {
        Self::decrease_and_return_node_reputation(
            subnet_id,
            subnet_node_id,
            SubnetNodeReputation::<T>::get(subnet_id, subnet_node_id),
            factor,
        )
    }

    /// Decrease from submitted node reputation and return new reputation
    /// This function is used to track reputations locally to lessen db reads
    pub fn decrease_and_return_node_reputation(
        subnet_id: u32,
        subnet_node_id: u32,
        current_reputation: u128,
        factor: u128,
    ) -> u128 {
        let new_reputation = Self::get_decrease_reputation(current_reputation, factor);
        SubnetNodeReputation::<T>::insert(subnet_id, subnet_node_id, new_reputation);
        Self::deposit_event(Event::NodeReputationUpdate {
            subnet_id,
            subnet_node_id,
            prev_reputation: current_reputation,
            new_reputation,
        });

        new_reputation
    }

    pub fn get_decrease_reputation(prev_reputation: u128, factor: u128) -> u128 {
        let one = Self::percentage_factor_as_u128();
        let factor = factor.min(one);
        let delta = Self::percent_mul(prev_reputation, factor);
        prev_reputation.saturating_sub(delta)
    }
}
