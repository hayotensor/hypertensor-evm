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
            value <= T::EpochLength::get() - 3,
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
        // Account for the first 3 block steps in an epoch
        // Do not go over epoch length - 3 to ensure each subnet has a slot in each epoch
        ensure!(value <= 256, Error::<T>::InvalidMaxSubnetBootnodeAccess);

        MaxSubnetBootnodeAccess::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetBootnodeAccess(value));

        Ok(())
    }

    pub fn do_set_max_subnet_penalty_count(value: u32) -> DispatchResult {
        MaxSubnetPenaltyCount::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetPenaltyCount(value));

        Ok(())
    }

    pub fn do_set_max_pause_epochs(value: u32) -> DispatchResult {
        MaxSubnetPauseEpochs::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetPauseEpochs(value));

        Ok(())
    }

    // pub fn do_set_min_subnet_registration_fee(value: u128) -> DispatchResult {
    //     MinSubnetRegistrationFee::<T>::set(value);

    //     Self::deposit_event(Event::SetMinSubnetRegistrationFee(value));

    //     Ok(())
    // }

    // pub fn do_set_max_subnet_registration_fee(value: u128) -> DispatchResult {
    //     MaxSubnetRegistrationFee::<T>::set(value);

    //     Self::deposit_event(Event::SetMaxSubnetRegistrationFee(value));

    //     Ok(())
    // }

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
        MaxMinDelegateStakeMultiplier::<T>::set(value);

        Self::deposit_event(Event::SetMaxMinDelegateStakeMultiplier(value));

        Ok(())
    }

    pub fn do_set_min_churn_limit(value: u32) -> DispatchResult {
        MinChurnLimit::<T>::set(value);

        Self::deposit_event(Event::SetMinChurnLimit(value));

        Ok(())
    }

    pub fn do_set_max_churn_limit(value: u32) -> DispatchResult {
        MaxChurnLimit::<T>::set(value);

        Self::deposit_event(Event::SetMaxChurnLimit(value));

        Ok(())
    }

    pub fn do_set_min_queue_epochs(value: u32) -> DispatchResult {
        MinQueueEpochs::<T>::set(value);

        Self::deposit_event(Event::SetMinQueueEpochs(value));

        Ok(())
    }

    pub fn do_set_max_queue_epochs(value: u32) -> DispatchResult {
        MaxQueueEpochs::<T>::set(value);

        Self::deposit_event(Event::SetMaxQueueEpochs(value));

        Ok(())
    }

    // pub fn do_set_min_activation_grace_epochs(value: u32) -> DispatchResult {
    //     MinActivationGraceEpochs::<T>::set(value);

    //     Self::deposit_event(Event::SetMinActivationGraceEpochs(value));

    //     Ok(())
    // }

    // pub fn do_set_max_activation_grace_epochs(value: u32) -> DispatchResult {
    //     MaxActivationGraceEpochs::<T>::set(value);

    //     Self::deposit_event(Event::SetMaxActivationGraceEpochs(value));

    //     Ok(())
    // }

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

    pub fn do_set_min_included_classification_epochs(value: u32) -> DispatchResult {
        MinIncludedClassificationEpochs::<T>::set(value);

        Self::deposit_event(Event::SetMinIncludedClassificationEpochs(value));

        Ok(())
    }

    pub fn do_set_max_included_classification_epochs(value: u32) -> DispatchResult {
        MaxIncludedClassificationEpochs::<T>::set(value);

        Self::deposit_event(Event::SetMaxIncludedClassificationEpochs(value));

        Ok(())
    }

    pub fn do_set_min_max_subnet_node_penalties(value: u32) -> DispatchResult {
        MinMaxSubnetNodePenalties::<T>::set(value);

        Self::deposit_event(Event::SetMinMaxSubnetNodePenalties(value));

        Ok(())
    }

    pub fn do_set_max_max_subnet_node_penalties(value: u32) -> DispatchResult {
        MaxMaxSubnetNodePenalties::<T>::set(value);

        Self::deposit_event(Event::SetMaxMaxSubnetNodePenalties(value));

        Ok(())
    }

    pub fn do_set_min_subnet_min_stake(value: u128) -> DispatchResult {
        MinSubnetMinStake::<T>::set(value);

        Self::deposit_event(Event::SetMinSubnetMinStake(value));

        Ok(())
    }

    pub fn do_set_max_subnet_min_stake(value: u128) -> DispatchResult {
        MaxSubnetMinStake::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetMinStake(value));

        Ok(())
    }

    pub fn do_set_min_subnet_max_stake(value: u128) -> DispatchResult {
        MinSubnetMaxStake::<T>::set(value);

        Self::deposit_event(Event::SetMinSubnetMaxStake(value));

        Ok(())
    }

    pub fn do_set_max_subnet_max_stake(value: u128) -> DispatchResult {
        MaxSubnetMaxStake::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetMaxStake(value));

        Ok(())
    }

    pub fn do_set_min_delegate_stake_percentage(value: u128) -> DispatchResult {
        MinDelegateStakePercentage::<T>::set(value);

        Self::deposit_event(Event::SetMinDelegateStakePercentage(value));

        Ok(())
    }

    pub fn do_set_max_delegate_stake_percentage(value: u128) -> DispatchResult {
        MaxDelegateStakePercentage::<T>::set(value);

        Self::deposit_event(Event::SetMaxDelegateStakePercentage(value));

        Ok(())
    }

    pub fn do_set_min_max_registered_nodes(value: u32) -> DispatchResult {
        MinMaxRegisteredNodes::<T>::set(value);

        Self::deposit_event(Event::SetMinMaxRegisteredNodes(value));

        Ok(())
    }

    pub fn do_set_max_max_registered_nodes(value: u32) -> DispatchResult {
        MaxMaxRegisteredNodes::<T>::set(value);

        Self::deposit_event(Event::SetMaxMaxRegisteredNodes(value));

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
            value <= Self::percentage_factor_as_u128(),
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

    pub fn do_set_network_min_stake_balance(value: u128) -> DispatchResult {
        NetworkMinStakeBalance::<T>::set(value);

        Self::deposit_event(Event::SetNetworkMinStakeBalance(value));

        Ok(())
    }

    pub fn do_set_network_max_stake_balance(value: u128) -> DispatchResult {
        NetworkMaxStakeBalance::<T>::set(value);

        Self::deposit_event(Event::SetNetworkMaxStakeBalance(value));

        Ok(())
    }

    pub fn do_set_min_active_nodes_stake_epochs(value: u32) -> DispatchResult {
        MinActiveNodeStakeEpochs::<T>::set(value);

        Self::deposit_event(Event::SetMinActiveNodeStakeEpochs(value));

        Ok(())
    }

    pub fn do_set_min_delegate_stake_deposit(value: u128) -> DispatchResult {
        ensure!(value >= 1000, Error::<T>::InvalidMinDelegateStakeDeposit);

        MinDelegateStakeDeposit::<T>::set(value);

        Self::deposit_event(Event::SetMinDelegateStakeDeposit(value));

        Ok(())
    }

    pub fn do_set_reward_rate_update_period(value: u32) -> DispatchResult {
        RewardRateUpdatePeriod::<T>::set(value);

        Self::deposit_event(Event::SetRewardRateUpdatePeriod(value));

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
        DelegateStakeWeightFactor::<T>::set(value);

        Self::deposit_event(Event::SetDelegateStakeWeightFactor(value));

        Ok(())
    }

    pub fn do_set_sigmoid_steepness(value: u128) -> DispatchResult {
        SigmoidSteepness::<T>::set(value);

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

    pub fn do_set_min_subnet_nodes(value: u32) -> DispatchResult {
        ensure!(
            value > 0 && value < MaxSubnetNodes::<T>::get(),
            Error::<T>::InvalidMinSubnetNodes
        );

        MinSubnetNodes::<T>::set(value);
        Self::deposit_event(Event::SetMinSubnetNodes(value));
        Ok(())
    }

    pub fn do_set_max_subnet_nodes(value: u32) -> DispatchResult {
        ensure!(
            value > MinSubnetNodes::<T>::get(),
            Error::<T>::InvalidMaxSubnetNodes
        );

        MaxSubnetNodes::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetNodes(value));

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
        OverwatchNodeBlacklist::<T>::insert(&coldkey, true);

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

        SigmoidMidpoint::<T>::put(value);

        Self::deposit_event(Event::SetSigmoidMidpoint(value));

        Ok(())
    }
    pub fn do_set_maximum_hooks_weight(value: u32) -> DispatchResult {
        ensure!(
            value > 100 && value <= 100,
            Error::<T>::InvalidPerbillPercent
        );

        MaximumHooksWeightV2::<T>::put(
            sp_runtime::Perbill::from_percent(value) * T::BlockWeights::get().max_block,
        );

        Self::deposit_event(Event::SetMaximumHooksWeight(value));

        Ok(())
    }
    pub fn do_set_base_node_burn_amount(value: u128) -> DispatchResult {
        BaseNodeBurnAmount::<T>::put(value);

        Self::deposit_event(Event::SetBaseNodeBurnAmount(value));

        Ok(())
    }
    pub fn do_set_min_node_burn_rate(value: u128) -> DispatchResult {
        let max_rate = MaxNodeBurnRate::<T>::get();
        ensure!(max_rate > value, Error::<T>::InvalidMinNodeBurnRate);

        MinNodeBurnRate::<T>::put(value);

        Self::deposit_event(Event::SetMinNodeBurnRate(value));

        Ok(())
    }
    pub fn do_set_max_node_burn_rate(value: u128) -> DispatchResult {
        let min_rate = MinNodeBurnRate::<T>::get();
        ensure!(min_rate < value, Error::<T>::InvalidMaxNodeBurnRate);

        MaxNodeBurnRate::<T>::put(value);

        Self::deposit_event(Event::SetMaxNodeBurnRate(value));

        Ok(())
    }
}
