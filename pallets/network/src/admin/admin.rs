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
use frame_support::pallet_prelude::DispatchResultWithPostInfo;

impl<T: Config> Pallet<T> {
    pub fn do_pause() -> DispatchResult {
        TxPause::<T>::put(true);
        Self::deposit_event(Event::SetTxPause());
        Ok(())
    }
    pub fn do_unpause() -> DispatchResult {
        TxPause::<T>::put(false);
        Self::deposit_event(Event::SetTxUnpause());
        Ok(())
    }
    pub fn do_set_subnet_owner_percentage(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        SubnetOwnerPercentage::<T>::put(value);

        Self::deposit_event(Event::SetSubnetOwnerPercentage(value));

        Ok(())
    }
    pub fn do_set_max_subnets(value: u32) -> DispatchResult {
        // Account for the first 3 block steps in an epoch
        // Do not go over epoch length - 3 to ensure each subnet has a slot in each epoch
        ensure!(
            value <= T::EpochLength::get() - T::DesignatedEpochSlots::get(),
            Error::<T>::InvalidMaxSubnets
        );

        MaxSubnets::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnets(value));

        Ok(())
    }
    pub fn do_set_max_bootnodes(value: u32) -> DispatchResult {
        ensure!(value <= 256, Error::<T>::InvalidMaxBootnodes);

        MaxBootnodes::<T>::set(value);

        Self::deposit_event(Event::SetMaxBootnodes(value));

        Ok(())
    }
    pub fn do_set_max_subnet_bootnodes_access(value: u32) -> DispatchResult {
        ensure!(value <= 256, Error::<T>::InvalidMaxSubnetBootnodeAccess);

        MaxSubnetBootnodeAccess::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetBootnodeAccess(value));

        Ok(())
    }
    pub fn do_set_max_subnet_penalty_count(value: u32) -> DispatchResult {
        ensure!(value > 0, Error::<T>::InvalidMaxSubnetPenaltyCount);

        MaxSubnetPenaltyCount::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetPenaltyCount(value));

        Ok(())
    }
    pub fn do_set_max_pause_epochs(value: u32) -> DispatchResult {
        ensure!(value > 0, Error::<T>::InvalidMaxSubnetPauseEpochs);

        MaxSubnetPauseEpochs::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetPauseEpochs(value));

        Ok(())
    }
    pub fn do_set_min_registration_cost(value: u128) -> DispatchResult {
        MinRegistrationCost::<T>::set(value);

        Self::deposit_event(Event::SetMinRegistrationCost(value));

        Ok(())
    }
    pub fn do_set_registration_cost_delay_blocks(value: u32) -> DispatchResult {
        RegistrationCostDecayBlocks::<T>::set(value);

        Self::deposit_event(Event::SetRegistrationCostDecayBlocks(value));

        Ok(())
    }
    pub fn do_set_registration_cost_alpha(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );
        RegistrationCostAlpha::<T>::set(value);

        Self::deposit_event(Event::SetRegistrationCostAlpha(value));

        Ok(())
    }
    pub fn do_set_new_registration_cost_multiplier(value: u128) -> DispatchResult {
        NewRegistrationCostMultiplier::<T>::set(value);

        Self::deposit_event(Event::SetNewRegistrationCostMultiplier(value));

        Ok(())
    }
    pub fn do_set_max_min_delegate_stake_multiplier(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );
        MaxMinDelegateStakeMultiplier::<T>::set(value);

        Self::deposit_event(Event::SetMaxMinDelegateStakeMultiplier(value));

        Ok(())
    }

    pub fn do_set_churn_limits(min: u32, max: u32) -> DispatchResult {
        ensure!(
            min < max,
            Error::<T>::InvalidValues
        );

        MinChurnLimit::<T>::set(min);
        MaxChurnLimit::<T>::set(max);

        Self::deposit_event(Event::SetMinChurnLimit(min));
        Self::deposit_event(Event::SetMaxChurnLimit(max));

        Ok(())
    }

    pub fn do_set_queue_epochs(min: u32, max: u32) -> DispatchResult {
        ensure!(
            min < max,
            Error::<T>::InvalidValues
        );

        MinQueueEpochs::<T>::set(min);
        MaxQueueEpochs::<T>::set(max);

        Self::deposit_event(Event::SetMinQueueEpochs(min));
        Self::deposit_event(Event::SetMaxQueueEpochs(max));

        Ok(())
    }
    pub fn do_set_min_idle_classification_epochs(value: u32) -> DispatchResult {
        MinIdleClassificationEpochs::<T>::set(value);

        Self::deposit_event(Event::SetMinIdleClassificationEpochs(value));

        Ok(())
    }
    pub fn do_set_max_idle_classification_epochs(value: u32) -> DispatchResult {
        MaxIdleClassificationEpochs::<T>::set(value);

        Self::deposit_event(Event::SetMaxIdleClassificationEpochs(value));

        Ok(())
    }
    pub fn do_set_subnet_activation_enactment_epochs(value: u32) -> DispatchResult {
        SubnetEnactmentEpochs::<T>::set(value);

        Self::deposit_event(Event::SetSubnetEnactmentEpochs(value));

        Ok(())
    }
    pub fn do_set_included_classification_epochs(min: u32, max: u32) -> DispatchResult {
        ensure!(
            min < max,
            Error::<T>::InvalidValues
        );

        MinIncludedClassificationEpochs::<T>::set(min);
        MaxIncludedClassificationEpochs::<T>::set(max);

        Self::deposit_event(Event::SetMinIncludedClassificationEpochs(min));
        Self::deposit_event(Event::SetMaxIncludedClassificationEpochs(max));

        Ok(())
    }
    pub fn do_set_max_subnet_node_penalties(min: u32, max: u32) -> DispatchResult {
        ensure!(
            min < max,
            Error::<T>::InvalidValues
        );


        MinMaxSubnetNodePenalties::<T>::set(min);
        MaxMaxSubnetNodePenalties::<T>::set(max);

        Self::deposit_event(Event::SetMinMaxSubnetNodePenalties(min));
        Self::deposit_event(Event::SetMaxMaxSubnetNodePenalties(max));

        Ok(())
    }
    pub fn do_set_subnet_min_stakes(min: u128, max: u128) -> DispatchResult {
        ensure!(
            min < max,
            Error::<T>::InvalidValues
        );

        MinSubnetMinStake::<T>::set(min);
        MaxSubnetMinStake::<T>::set(max);

        Self::deposit_event(Event::SetMinSubnetMinStake(min));
        Self::deposit_event(Event::SetMaxSubnetMinStake(max));

        Ok(())
    }
    pub fn do_set_delegate_stake_percentages(min: u128, max: u128) -> DispatchResult {
        ensure!(
            min < max,
            Error::<T>::InvalidValues
        );

        MinDelegateStakePercentage::<T>::set(min);
        MaxDelegateStakePercentage::<T>::set(max);

        Self::deposit_event(Event::SetMinDelegateStakePercentage(min));
        Self::deposit_event(Event::SetMaxDelegateStakePercentage(max));

        Ok(())
    }
    pub fn do_set_max_registered_nodes(min: u32, max: u32) -> DispatchResult {
        ensure!(
            min < max,
            Error::<T>::InvalidValues
        );

        MinMaxRegisteredNodes::<T>::set(min);
        MaxMaxRegisteredNodes::<T>::set(max);

        Self::deposit_event(Event::SetMinMaxRegisteredNodes(min));
        Self::deposit_event(Event::SetMaxMaxRegisteredNodes(max));

        Ok(())
    }
    pub fn do_set_max_subnet_delegate_stake_rewards_percentage_change(
        value: u128,
    ) -> DispatchResult {
        MaxSubnetDelegateStakeRewardsPercentageChange::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetDelegateStakeRewardsPercentageChange(
            value,
        ));

        Ok(())
    }
    pub fn do_set_subnet_delegate_stake_rewards_update_period(value: u32) -> DispatchResult {
        SubnetDelegateStakeRewardsUpdatePeriod::<T>::set(value);

        Self::deposit_event(Event::SetSubnetDelegateStakeRewardsUpdatePeriod(value));

        Ok(())
    }
    pub fn do_set_min_attestation_percentage(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128()
                && value > Self::percentage_factor_as_u128() / 2,
            Error::<T>::InvalidPercent
        );

        MinAttestationPercentage::<T>::set(value);

        Self::deposit_event(Event::SetMinAttestationPercentage(value));

        Ok(())
    }
    pub fn do_set_super_majority_attestation_ratio(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ensure!(
            value <= MinAttestationPercentage::<T>::get(),
            Error::<T>::InvalidSuperMajorityAttestationRatio
        );

        SuperMajorityAttestationRatio::<T>::set(value);

        Self::deposit_event(Event::SetSuperMajorityAttestationRatio(value));

        Ok(())
    }
    pub fn do_set_base_validator_reward(value: u128) -> DispatchResult {
        BaseValidatorReward::<T>::set(value);

        Self::deposit_event(Event::SetBaseValidatorReward(value));

        Ok(())
    }
    pub fn do_set_base_slash_percentage(value: u128) -> DispatchResult {
        BaseSlashPercentage::<T>::set(value);

        Self::deposit_event(Event::SetBaseSlashPercentage(value));

        Ok(())
    }
    pub fn do_set_max_slash_amount(value: u128) -> DispatchResult {
        MaxSlashAmount::<T>::set(value);

        Self::deposit_event(Event::SetMaxSlashAmount(value));

        Ok(())
    }
    pub fn do_set_reputation_increase_factor(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ReputationIncreaseFactor::<T>::set(value);

        Self::deposit_event(Event::SetReputationIncreaseFactor(value));

        Ok(())
    }
    pub fn do_set_reputation_decrease_factor(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ReputationDecreaseFactor::<T>::set(value);

        Self::deposit_event(Event::SetReputationDecreaseFactor(value));

        Ok(())
    }
    pub fn do_set_network_max_stake_balance(value: u128) -> DispatchResult {
        NetworkMaxStakeBalance::<T>::set(value);

        Self::deposit_event(Event::SetNetworkMaxStakeBalance(value));

        Ok(())
    }
    pub fn do_set_min_delegate_stake_deposit(value: u128) -> DispatchResult {
        ensure!(value >= 1000, Error::<T>::InvalidMinDelegateStakeDeposit);

        MinDelegateStakeDeposit::<T>::set(value);

        Self::deposit_event(Event::SetMinDelegateStakeDeposit(value));

        Ok(())
    }
    pub fn do_set_node_reward_rate_update_period(value: u32) -> DispatchResult {
        NodeRewardRateUpdatePeriod::<T>::set(value);

        Self::deposit_event(Event::SetNodeRewardRateUpdatePeriod(value));

        Ok(())
    }
    pub fn do_set_max_reward_rate_decrease(value: u128) -> DispatchResult {
        MaxRewardRateDecrease::<T>::set(value);

        Self::deposit_event(Event::SetMaxRewardRateDecrease(value));

        Ok(())
    }
    pub fn do_set_subnet_distribution_power(value: u128) -> DispatchResult {
        SubnetDistributionPower::<T>::set(value);

        Self::deposit_event(Event::SetSubnetDistributionPower(value));

        Ok(())
    }
    pub fn do_set_delegate_stake_weight_factor(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        DelegateStakeWeightFactor::<T>::set(value);

        Self::deposit_event(Event::SetDelegateStakeWeightFactor(value));

        Ok(())
    }
    pub fn do_set_sigmoid_steepness(value: u128) -> DispatchResult {
        InflationSigmoidSteepness::<T>::set(value);

        Self::deposit_event(Event::SetSigmoidSteepness(value));

        Ok(())
    }
    pub fn do_set_max_overwatch_nodes(value: u32) -> DispatchResult {
        MaxOverwatchNodes::<T>::set(value);

        Self::deposit_event(Event::SetMaxOverwatchNodes(value));

        Ok(())
    }
    pub fn do_set_overwatch_epoch_length_multiplier(value: u32) -> DispatchResult {
        // Ensure always at least  `1` to avoid modulo operator errors in `on_initialize`
        ensure!(value > 0, Error::<T>::InvalidOverwatchEpochLengthMultiplier);

        OverwatchEpochLengthMultiplier::<T>::set(value);

        Self::deposit_event(Event::SetOverwatchEpochLengthMultiplier(value));

        Ok(())
    }
    pub fn do_set_overwatch_commit_cutoff_percent(value: u128) -> DispatchResult {
        ensure!(
            value <= 950000000000000000, // 95%
            Error::<T>::InvalidPercent
        );

        OverwatchCommitCutoffPercent::<T>::set(value);

        Self::deposit_event(Event::SetOverwatchCommitCutoffPercent(value));

        Ok(())
    }
    pub fn do_set_max_overwatch_node_penalties(value: u32) -> DispatchResult {
        MaxOverwatchNodePenalties::<T>::set(value);

        Self::deposit_event(Event::SetMaxOverwatchNodePenalties(value));

        Ok(())
    }
    pub fn do_set_overwatch_min_diversification_ratio(value: u128) -> DispatchResult {
        OverwatchMinDiversificationRatio::<T>::set(value);

        Self::deposit_event(Event::SetOverwatchMinDiversificationRatio(value));

        Ok(())
    }
    pub fn do_set_overwatch_min_rep_score(value: u128) -> DispatchResult {
        OverwatchMinRepScore::<T>::set(value);

        Self::deposit_event(Event::SetOverwatchMinRepScore(value));

        Ok(())
    }
    pub fn do_set_overwatch_min_avg_attestation_ratio(value: u128) -> DispatchResult {
        OverwatchMinAvgAttestationRatio::<T>::set(value);

        Self::deposit_event(Event::SetOverwatchMinAvgAttestationRatio(value));

        Ok(())
    }
    pub fn do_set_overwatch_min_age(value: u32) -> DispatchResult {
        OverwatchMinAge::<T>::set(value);

        Self::deposit_event(Event::SetOverwatchMinAge(value));

        Ok(())
    }
    pub fn do_set_overwatch_min_stake_balance(value: u128) -> DispatchResult {
        OverwatchMinStakeBalance::<T>::set(value);

        Self::deposit_event(Event::SetOverwatchMinStakeBalance(value));

        Ok(())
    }

    pub fn do_set_min_max_subnet_node(min: u32, max: u32) -> DispatchResult {
        ensure!(
            min < max && min > 0,
            Error::<T>::InvalidValues
        );

        MinSubnetNodes::<T>::set(min);
        MaxSubnetNodes::<T>::set(max);

        Self::deposit_event(Event::SetMinSubnetNodes(min));
        Self::deposit_event(Event::SetMaxSubnetNodes(max));

        Ok(())
    }
    pub fn do_set_tx_rate_limit(value: u32) -> DispatchResult {
        TxRateLimit::<T>::set(value);

        Self::deposit_event(Event::SetTxRateLimit(value));

        Ok(())
    }
    pub fn do_set_min_subnet_delegate_stake_factor(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        MinSubnetDelegateStakeFactor::<T>::set(value);

        Self::deposit_event(Event::SetMinSubnetDelegateStakeFactor(value));

        Ok(())
    }
    pub fn do_collective_remove_subnet(subnet_id: u32) -> DispatchResultWithPostInfo {
        let weight = Self::do_remove_subnet(subnet_id, SubnetRemovalReason::Council);
        Ok(Some(weight).into())
    }
    pub fn do_collective_remove_subnet_node(subnet_id: u32, subnet_node_id: u32) -> DispatchResult {
        Self::deposit_event(Event::CollectiveRemoveSubnetNode(subnet_id, subnet_node_id));
        Self::do_remove_subnet_node(subnet_id, subnet_node_id)
    }
    pub fn do_collective_remove_overwatch_node(overwatch_node_id: u32) -> DispatchResult {
        Self::perform_remove_overwatch_node(overwatch_node_id);
        Self::deposit_event(Event::CollectiveRemoveOverwatchNode(overwatch_node_id));
        Ok(())
    }
    /// Temporary solution until network maturity
    pub fn do_collective_set_coldkey_overwatch_node_eligibility(
        coldkey: T::AccountId,
        value: bool,
    ) -> DispatchResult {
        OverwatchNodeBlacklist::<T>::insert(&coldkey, value);

        Self::deposit_event(Event::OverwatchNodeBlacklist(coldkey.clone(), value));

        Ok(())
    }
    pub fn do_set_min_subnet_registration_epochs(value: u32) -> DispatchResult {
        let registration_epochs = SubnetRegistrationEpochs::<T>::get();
        // Must be less than the registration period itself
        ensure!(
            value < registration_epochs,
            Error::<T>::InvalidMinSubnetRegistrationEpochs
        );

        MinSubnetRegistrationEpochs::<T>::put(value);

        Self::deposit_event(Event::SetMinSubnetRegistrationEpochs(value));

        Ok(())
    }
    pub fn do_set_subnet_registration_epochs(value: u32) -> DispatchResult {
        let min_registration_epochs = MinSubnetRegistrationEpochs::<T>::get();
        ensure!(
            value > min_registration_epochs,
            Error::<T>::InvalidSubnetRegistrationEpochs
        );
        SubnetRegistrationEpochs::<T>::put(value);

        Self::deposit_event(Event::SetSubnetRegistrationEpochs(value));

        Ok(())
    }
    pub fn do_set_min_active_node_stake_epochs(value: u32) -> DispatchResult {
        MinActiveNodeStakeEpochs::<T>::put(value);

        Self::deposit_event(Event::SetMinActiveNodeStakeEpochs(value));

        Ok(())
    }

    pub fn do_set_delegate_stake_cooldown_epochs(value: u32) -> DispatchResult {
        ensure!(value > 0, Error::<T>::InvalidDelegateStakeCooldownEpochs);

        DelegateStakeCooldownEpochs::<T>::set(value);

        Self::deposit_event(Event::SetDelegateStakeCooldownEpochs(value));

        Ok(())
    }
    pub fn do_set_node_delegate_stake_cooldown_epochs(value: u32) -> DispatchResult {
        ensure!(
            value > 0,
            Error::<T>::InvalidNodeDelegateStakeCooldownEpochs
        );

        NodeDelegateStakeCooldownEpochs::<T>::set(value);

        Self::deposit_event(Event::SetNodeDelegateStakeCooldownEpochs(value));

        Ok(())
    }
    pub fn do_set_min_stake_cooldown_epochs(value: u32) -> DispatchResult {
        ensure!(value > 0, Error::<T>::InvalidStakeCooldownEpochs);

        StakeCooldownEpochs::<T>::set(value);

        Self::deposit_event(Event::SetStakeCooldownEpochs(value));

        Ok(())
    }
    pub fn do_set_max_unbondings(value: u32) -> DispatchResult {
        ensure!(value <= 256, Error::<T>::InvalidMaxUnbondings);

        MaxUnbondings::<T>::set(value);

        Self::deposit_event(Event::SetMaxUnbondings(value));

        Ok(())
    }
    pub fn do_set_sigmoid_midpoint(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        InflationSigmoidMidpoint::<T>::put(value);

        Self::deposit_event(Event::SetSigmoidMidpoint(value));

        Ok(())
    }
    pub fn do_set_maximum_hooks_weight(value: u32) -> DispatchResult {
        ensure!(value > 0 && value <= 100, Error::<T>::InvalidPerbillPercent);

        let new_value = sp_runtime::Perbill::from_percent(value) * T::BlockWeights::get().max_block;

        MaximumHooksWeightV2::<T>::put(new_value);

        Self::deposit_event(Event::SetMaximumHooksWeight(value));

        Ok(())
    }
    pub fn do_set_base_node_burn_amount(value: u128) -> DispatchResult {
        BaseNodeBurnAmount::<T>::put(value);

        Self::deposit_event(Event::SetBaseNodeBurnAmount(value));

        Ok(())
    }
    pub fn do_set_node_burn_rates(min: u128, max: u128) -> DispatchResult {
        ensure!(
            min < max && min > 0,
            Error::<T>::InvalidValues
        );

        MinNodeBurnRate::<T>::put(min);
        MaxNodeBurnRate::<T>::put(max);

        Self::deposit_event(Event::SetMinNodeBurnRate(min));
        Self::deposit_event(Event::SetMaxNodeBurnRate(max));

        Ok(())
    }
    pub fn do_set_delegate_stake_subnet_removal_interval(value: u32) -> DispatchResult {
        ensure!(
            value > 0,
            Error::<T>::InvalidDelegateStakeSubnetRemovalInterval
        );

        DelegateStakeSubnetRemovalInterval::<T>::put(value);

        Self::deposit_event(Event::SetDelegateStakeSubnetRemovalInterval(value));

        Ok(())
    }
    pub fn do_set_subnet_removal_intervals(min: u32, max: u32) -> DispatchResult {
        ensure!(
            min < max,
            Error::<T>::InvalidValues
        );

        MinSubnetRemovalInterval::<T>::put(min);
        MaxSubnetRemovalInterval::<T>::put(max);

        Self::deposit_event(Event::SetMinSubnetRemovalInterval(min));
        Self::deposit_event(Event::SetMaxSubnetRemovalInterval(max));

        Ok(())
    }
    pub fn do_set_subnet_pause_cooldown_epochs(value: u32) -> DispatchResult {
        SubnetPauseCooldownEpochs::<T>::put(value);

        Self::deposit_event(Event::SetSubnetPauseCooldownEpochs(value));

        Ok(())
    }
    pub fn do_set_max_swap_queue_calls_per_block(value: u32) -> DispatchResult {
        MaxSwapQueueCallsPerBlock::<T>::put(value);

        Self::deposit_event(Event::SetMaxSwapQueueCallsPerBlock(value));

        Ok(())
    }
    pub fn do_set_max_subnet_node_score_penalty_threshold(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        MaxSubnetNodeScorePenaltyThreshold::<T>::put(value);

        Self::deposit_event(Event::SetMaxSubnetNodeScorePenaltyThreshold(value));

        Ok(())
    }
    pub fn do_set_validator_reward_k(value: u128) -> DispatchResult {
        ensure!(
            value > 0,
            Error::<T>::InvalidValidatorRewardK
        );

        ValidatorRewardK::<T>::put(value);

        Self::deposit_event(Event::SetValidatorRewardK(value));

        Ok(())
    }
    pub fn do_set_validator_reward_midpoint(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ValidatorRewardMidpoint::<T>::put(value);

        Self::deposit_event(Event::SetValidatorRewardMidpoint(value));

        Ok(())
    }
}
