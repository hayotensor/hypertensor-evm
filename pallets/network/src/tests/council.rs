use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use crate::{
    BaseNodeBurnAmount, BaseSlashPercentage, BaseValidatorReward, ColdkeyHotkeys,
    DelegateStakeCooldownEpochs, DelegateStakeSubnetRemovalInterval, DelegateStakeWeightFactor,
    Error, HotkeyOverwatchNodeId, HotkeySubnetNodeId, MaxBootnodes, MaxChurnLimit,
    MaxDelegateStakePercentage, MaxIdleClassificationEpochs, MaxIncludedClassificationEpochs,
    MaxMaxRegisteredNodes, MaxMaxSubnetNodePenalties, MaxMinDelegateStakeMultiplier,
    MaxNodeBurnRate, MaxOverwatchNodePenalties, MaxOverwatchNodes, MaxQueueEpochs,
    MaxRewardRateDecrease, MaxSlashAmount, MaxSubnetBootnodeAccess,
    MaxSubnetDelegateStakeRewardsPercentageChange, MaxSubnetMinStake,
    MaxSubnetNodeScorePenaltyThreshold, MaxSubnetNodes, MaxSubnetPauseEpochs,
    MaxSubnetPenaltyCount, MaxSubnetRemovalInterval, MaxSubnets, MaxSwapQueueCallsPerBlock,
    MaxUnbondings, MaximumHooksWeightV2, MinActiveNodeStakeEpochs, MinAttestationPercentage,
    MinChurnLimit, MinDelegateStakeDeposit, MinDelegateStakePercentage,
    MinIdleClassificationEpochs, MinIncludedClassificationEpochs, MinMaxRegisteredNodes,
    MinMaxSubnetNodePenalties, MinNodeBurnRate, MinQueueEpochs, MinRegistrationCost,
    MinSubnetDelegateStakeFactor, MinSubnetMinStake, MinSubnetNodes, MinSubnetRegistrationEpochs,
    MinSubnetRemovalInterval, NetworkMaxStakeBalance, NewRegistrationCostMultiplier,
    NodeDelegateStakeCooldownEpochs, NodeRewardRateUpdatePeriod, OverwatchCommitCutoffPercent,
    OverwatchEpochLengthMultiplier, OverwatchMinAge, OverwatchMinAvgAttestationRatio,
    OverwatchMinDiversificationRatio, OverwatchMinRepScore, OverwatchMinStakeBalance,
    OverwatchNodeBlacklist, OverwatchNodeIdHotkey, OverwatchNodes, RegistrationCostAlpha,
    RegistrationCostDecayBlocks, ReputationDecreaseFactor, ReputationIncreaseFactor,
    SigmoidMidpoint, SigmoidSteepness, StakeCooldownEpochs, SubnetDelegateStakeRewardsUpdatePeriod,
    SubnetDistributionPower, SubnetEnactmentEpochs, SubnetName, SubnetOwnerPercentage,
    SubnetPauseCooldownEpochs, SubnetRegistrationEpochs, SubnetRemovalReason,
    SuperMajorityAttestationRatio, TotalActiveSubnets, TotalOverwatchNodeUids, TotalOverwatchNodes,
    TxPause, TxRateLimit,
};
use frame_support::traits::Currency;
use frame_support::{assert_err, assert_ok};
use sp_std::collections::btree_map::BTreeMap;

#[test]
fn test_do_pause_do_unpause() {
    new_test_ext().execute_with(|| {
        let is_paused = TxPause::<Test>::get();
        assert_eq!(is_paused, false);
        assert_ok!(Network::do_pause());
        assert_eq!(TxPause::<Test>::get(), true);

        assert_ok!(Network::do_unpause());
        assert_eq!(TxPause::<Test>::get(), false);
    })
}

#[test]
fn test_do_set_subnet_owner_percentage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_subnet_owner_percentage(1));
        assert_eq!(SubnetOwnerPercentage::<Test>::get(), 1);

        assert_ok!(Network::do_set_subnet_owner_percentage(2));
        assert_eq!(SubnetOwnerPercentage::<Test>::get(), 2);

        assert_err!(
            Network::do_set_subnet_owner_percentage(Network::percentage_factor_as_u128() + 1),
            Error::<Test>::InvalidPercent
        );
    })
}

#[test]
fn test_do_set_max_subnets() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_subnets(5));
        assert_eq!(MaxSubnets::<Test>::get(), 5);

        assert_ok!(Network::do_set_max_subnets(6));
        assert_eq!(MaxSubnets::<Test>::get(), 6);

        assert_err!(
            Network::do_set_max_subnets(EpochLength::get()),
            Error::<Test>::InvalidMaxSubnets
        );
    })
}

#[test]
fn test_do_set_max_bootnodes() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_bootnodes(5));
        assert_eq!(MaxBootnodes::<Test>::get(), 5);

        assert_ok!(Network::do_set_max_bootnodes(6));
        assert_eq!(MaxBootnodes::<Test>::get(), 6);

        assert_err!(
            Network::do_set_max_bootnodes(257),
            Error::<Test>::InvalidMaxBootnodes
        );
    })
}

#[test]
fn test_do_set_max_subnet_bootnodes_access() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_subnet_bootnodes_access(5));
        assert_eq!(MaxSubnetBootnodeAccess::<Test>::get(), 5);

        assert_ok!(Network::do_set_max_subnet_bootnodes_access(6));
        assert_eq!(MaxSubnetBootnodeAccess::<Test>::get(), 6);

        assert_err!(
            Network::do_set_max_subnet_bootnodes_access(257),
            Error::<Test>::InvalidMaxSubnetBootnodeAccess
        );
    })
}

#[test]
fn test_do_set_max_subnet_penalty_count() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_subnet_penalty_count(5));
        assert_eq!(MaxSubnetPenaltyCount::<Test>::get(), 5);

        assert_ok!(Network::do_set_max_subnet_penalty_count(6));
        assert_eq!(MaxSubnetPenaltyCount::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_max_pause_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_pause_epochs(5));
        assert_eq!(MaxSubnetPauseEpochs::<Test>::get(), 5);

        assert_ok!(Network::do_set_max_pause_epochs(6));
        assert_eq!(MaxSubnetPauseEpochs::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_min_registration_cost() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_min_registration_cost(5));
        assert_eq!(MinRegistrationCost::<Test>::get(), 5);

        assert_ok!(Network::do_set_min_registration_cost(6));
        assert_eq!(MinRegistrationCost::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_registration_cost_delay_blocks() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_registration_cost_delay_blocks(5));
        assert_eq!(RegistrationCostDecayBlocks::<Test>::get(), 5);

        assert_ok!(Network::do_set_registration_cost_delay_blocks(6));
        assert_eq!(RegistrationCostDecayBlocks::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_registration_cost_alpha() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_registration_cost_alpha(5));
        assert_eq!(RegistrationCostAlpha::<Test>::get(), 5);

        assert_ok!(Network::do_set_registration_cost_alpha(6));
        assert_eq!(RegistrationCostAlpha::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_new_registration_cost_multiplier() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_new_registration_cost_multiplier(5));
        assert_eq!(NewRegistrationCostMultiplier::<Test>::get(), 5);

        assert_ok!(Network::do_set_new_registration_cost_multiplier(6));
        assert_eq!(NewRegistrationCostMultiplier::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_max_min_delegate_stake_multiplier() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_min_delegate_stake_multiplier(5));
        assert_eq!(MaxMinDelegateStakeMultiplier::<Test>::get(), 5);

        assert_ok!(Network::do_set_max_min_delegate_stake_multiplier(6));
        assert_eq!(MaxMinDelegateStakeMultiplier::<Test>::get(), 6);

        assert_err!(
            Network::do_set_max_min_delegate_stake_multiplier(10000000000000000000),
            Error::<Test>::InvalidPercent
        );
    })
}

#[test]
fn test_do_set_min_churn_limit() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_churn_limits(5, 6));
        assert_eq!(MinChurnLimit::<Test>::get(), 5);
        assert_eq!(MaxChurnLimit::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_min_queue_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_queue_epochs(5, 6));
        assert_eq!(MinQueueEpochs::<Test>::get(), 5);
        assert_eq!(MaxQueueEpochs::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_min_idle_classification_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_min_idle_classification_epochs(5));
        assert_eq!(MinIdleClassificationEpochs::<Test>::get(), 5);

        assert_ok!(Network::do_set_min_idle_classification_epochs(6));
        assert_eq!(MinIdleClassificationEpochs::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_max_idle_classification_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_idle_classification_epochs(5));
        assert_eq!(MaxIdleClassificationEpochs::<Test>::get(), 5);

        assert_ok!(Network::do_set_max_idle_classification_epochs(6));
        assert_eq!(MaxIdleClassificationEpochs::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_subnet_activation_enactment_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_subnet_activation_enactment_epochs(5));
        assert_eq!(SubnetEnactmentEpochs::<Test>::get(), 5);

        assert_ok!(Network::do_set_subnet_activation_enactment_epochs(6));
        assert_eq!(SubnetEnactmentEpochs::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_included_classification_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_included_classification_epochs(5, 6));
        assert_eq!(MinIncludedClassificationEpochs::<Test>::get(), 5);
        assert_eq!(MaxIncludedClassificationEpochs::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_max_subnet_node_penalties() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_subnet_node_penalties(5, 6));
        assert_eq!(MinMaxSubnetNodePenalties::<Test>::get(), 5);
        assert_eq!(MaxMaxSubnetNodePenalties::<Test>::get(), 6);

    })
}

#[test]
fn test_do_set_subnet_min_stakes() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_subnet_min_stakes(5, 6));
        assert_eq!(MinSubnetMinStake::<Test>::get(), 5);
        assert_eq!(MaxSubnetMinStake::<Test>::get(), 6);

    })
}

#[test]
fn test_do_set_delegate_stake_percentages() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_delegate_stake_percentages(5, 6));
        assert_eq!(MinDelegateStakePercentage::<Test>::get(), 5);
        assert_eq!(MaxDelegateStakePercentage::<Test>::get(), 6);

    })
}

#[test]
fn test_do_set_max_registered_nodes() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_registered_nodes(5, 6));
        assert_eq!(MinMaxRegisteredNodes::<Test>::get(), 5);
        assert_eq!(MaxMaxRegisteredNodes::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_max_subnet_delegate_stake_rewards_percentage_change() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_subnet_delegate_stake_rewards_percentage_change(5));
        assert_eq!(
            MaxSubnetDelegateStakeRewardsPercentageChange::<Test>::get(),
            5
        );

        assert_ok!(Network::do_set_max_subnet_delegate_stake_rewards_percentage_change(6));
        assert_eq!(
            MaxSubnetDelegateStakeRewardsPercentageChange::<Test>::get(),
            6
        );
    })
}

#[test]
fn test_do_set_subnet_delegate_stake_rewards_update_period() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_subnet_delegate_stake_rewards_update_period(
            5
        ));
        assert_eq!(SubnetDelegateStakeRewardsUpdatePeriod::<Test>::get(), 5);

        assert_ok!(Network::do_set_subnet_delegate_stake_rewards_update_period(
            6
        ));
        assert_eq!(SubnetDelegateStakeRewardsUpdatePeriod::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_min_attestation_percentage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_min_attestation_percentage(
            600000000000000000
        ));
        assert_eq!(MinAttestationPercentage::<Test>::get(), 600000000000000000);

        assert_ok!(Network::do_set_min_attestation_percentage(
            900000000000000000
        ));
        assert_eq!(MinAttestationPercentage::<Test>::get(), 900000000000000000);

        assert_err!(
            Network::do_set_min_attestation_percentage(1000000000000000000000),
            Error::<Test>::InvalidPercent
        );
    })
}

#[test]
fn test_do_set_super_majority_attestation_ratio() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_super_majority_attestation_ratio(
            60000000000
        ));
        assert_eq!(
            SuperMajorityAttestationRatio::<Test>::get(),
            60000000000
        );

        assert_ok!(Network::do_set_super_majority_attestation_ratio(
            90000000000
        ));
        assert_eq!(
            SuperMajorityAttestationRatio::<Test>::get(),
            90000000000
        );

        assert_err!(
            Network::do_set_super_majority_attestation_ratio(1000000000000000000000),
            Error::<Test>::InvalidPercent
        );
    })
}

#[test]
fn test_do_set_base_validator_reward() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_base_validator_reward(6));
        assert_eq!(BaseValidatorReward::<Test>::get(), 6);

        assert_ok!(Network::do_set_base_validator_reward(5));
        assert_eq!(BaseValidatorReward::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_base_slash_percentage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_base_slash_percentage(6));
        assert_eq!(BaseSlashPercentage::<Test>::get(), 6);

        assert_ok!(Network::do_set_base_slash_percentage(5));
        assert_eq!(BaseSlashPercentage::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_max_slash_amount() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_slash_amount(6));
        assert_eq!(MaxSlashAmount::<Test>::get(), 6);

        assert_ok!(Network::do_set_max_slash_amount(5));
        assert_eq!(MaxSlashAmount::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_reputation_increase_factor() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_reputation_increase_factor(6));
        assert_eq!(ReputationIncreaseFactor::<Test>::get(), 6);

        assert_ok!(Network::do_set_reputation_increase_factor(5));
        assert_eq!(ReputationIncreaseFactor::<Test>::get(), 5);

        assert_err!(
            Network::do_set_reputation_increase_factor(1000000000000000000000),
            Error::<Test>::InvalidPercent
        );
    })
}

#[test]
fn test_do_set_reputation_decrease_factor() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_reputation_decrease_factor(6));
        assert_eq!(ReputationDecreaseFactor::<Test>::get(), 6);

        assert_ok!(Network::do_set_reputation_decrease_factor(5));
        assert_eq!(ReputationDecreaseFactor::<Test>::get(), 5);

        assert_err!(
            Network::do_set_reputation_decrease_factor(1000000000000000000000),
            Error::<Test>::InvalidPercent
        );
    })
}

#[test]
fn test_do_set_network_max_stake_balance() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_network_max_stake_balance(6));
        assert_eq!(NetworkMaxStakeBalance::<Test>::get(), 6);

        assert_ok!(Network::do_set_network_max_stake_balance(5));
        assert_eq!(NetworkMaxStakeBalance::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_min_delegate_stake_deposit() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_min_delegate_stake_deposit(6000));
        assert_eq!(MinDelegateStakeDeposit::<Test>::get(), 6000);

        assert_ok!(Network::do_set_min_delegate_stake_deposit(5000));
        assert_eq!(MinDelegateStakeDeposit::<Test>::get(), 5000);

        assert_err!(
            Network::do_set_min_delegate_stake_deposit(1),
            Error::<Test>::InvalidMinDelegateStakeDeposit
        );
    })
}

#[test]
fn test_do_set_node_reward_rate_update_period() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_node_reward_rate_update_period(6));
        assert_eq!(NodeRewardRateUpdatePeriod::<Test>::get(), 6);

        assert_ok!(Network::do_set_node_reward_rate_update_period(5));
        assert_eq!(NodeRewardRateUpdatePeriod::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_max_reward_rate_decrease() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_reward_rate_decrease(6));
        assert_eq!(MaxRewardRateDecrease::<Test>::get(), 6);

        assert_ok!(Network::do_set_max_reward_rate_decrease(5));
        assert_eq!(MaxRewardRateDecrease::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_subnet_distribution_power() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_subnet_distribution_power(6));
        assert_eq!(SubnetDistributionPower::<Test>::get(), 6);

        assert_ok!(Network::do_set_subnet_distribution_power(5));
        assert_eq!(SubnetDistributionPower::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_delegate_stake_weight_factor() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_delegate_stake_weight_factor(6));
        assert_eq!(DelegateStakeWeightFactor::<Test>::get(), 6);

        assert_ok!(Network::do_set_delegate_stake_weight_factor(5));
        assert_eq!(DelegateStakeWeightFactor::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_sigmoid_steepness() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_sigmoid_steepness(6));
        assert_eq!(SigmoidSteepness::<Test>::get(), 6);

        assert_ok!(Network::do_set_sigmoid_steepness(5));
        assert_eq!(SigmoidSteepness::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_max_overwatch_nodes() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_overwatch_nodes(6));
        assert_eq!(MaxOverwatchNodes::<Test>::get(), 6);

        assert_ok!(Network::do_set_max_overwatch_nodes(5));
        assert_eq!(MaxOverwatchNodes::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_overwatch_epoch_length_multiplier() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_overwatch_epoch_length_multiplier(6));
        assert_eq!(OverwatchEpochLengthMultiplier::<Test>::get(), 6);

        assert_ok!(Network::do_set_overwatch_epoch_length_multiplier(5));
        assert_eq!(OverwatchEpochLengthMultiplier::<Test>::get(), 5);

        assert_err!(
            Network::do_set_overwatch_epoch_length_multiplier(0),
            Error::<Test>::InvalidOverwatchEpochLengthMultiplier
        );
    })
}

#[test]
fn test_do_set_overwatch_commit_cutoff_percent() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_overwatch_commit_cutoff_percent(6));
        assert_eq!(OverwatchCommitCutoffPercent::<Test>::get(), 6);

        assert_ok!(Network::do_set_overwatch_commit_cutoff_percent(5));
        assert_eq!(OverwatchCommitCutoffPercent::<Test>::get(), 5);

        assert_err!(
            Network::do_set_overwatch_commit_cutoff_percent(950000000000000001),
            Error::<Test>::InvalidPercent
        );
    })
}

#[test]
fn test_do_set_max_overwatch_node_penalties() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_overwatch_node_penalties(6));
        assert_eq!(MaxOverwatchNodePenalties::<Test>::get(), 6);

        assert_ok!(Network::do_set_max_overwatch_node_penalties(5));
        assert_eq!(MaxOverwatchNodePenalties::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_overwatch_min_diversification_ratio() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_overwatch_min_diversification_ratio(6));
        assert_eq!(OverwatchMinDiversificationRatio::<Test>::get(), 6);

        assert_ok!(Network::do_set_overwatch_min_diversification_ratio(5));
        assert_eq!(OverwatchMinDiversificationRatio::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_overwatch_min_rep_score() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_overwatch_min_rep_score(6));
        assert_eq!(OverwatchMinRepScore::<Test>::get(), 6);

        assert_ok!(Network::do_set_overwatch_min_rep_score(5));
        assert_eq!(OverwatchMinRepScore::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_overwatch_min_avg_attestation_ratio() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_overwatch_min_avg_attestation_ratio(6));
        assert_eq!(OverwatchMinAvgAttestationRatio::<Test>::get(), 6);

        assert_ok!(Network::do_set_overwatch_min_avg_attestation_ratio(5));
        assert_eq!(OverwatchMinAvgAttestationRatio::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_overwatch_min_age() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_overwatch_min_age(6));
        assert_eq!(OverwatchMinAge::<Test>::get(), 6);

        assert_ok!(Network::do_set_overwatch_min_age(5));
        assert_eq!(OverwatchMinAge::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_overwatch_min_stake_balance() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_overwatch_min_stake_balance(6));
        assert_eq!(OverwatchMinStakeBalance::<Test>::get(), 6);

        assert_ok!(Network::do_set_overwatch_min_stake_balance(5));
        assert_eq!(OverwatchMinStakeBalance::<Test>::get(), 5);
    })
}

#[test]
fn test_do_collective_set_coldkey_overwatch_node_eligibility() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            Network::do_collective_set_coldkey_overwatch_node_eligibility(account(99), true)
        );
        let blacklist = OverwatchNodeBlacklist::<Test>::get(account(99));
        assert!(blacklist);

        assert_ok!(
            Network::do_collective_set_coldkey_overwatch_node_eligibility(account(99), false)
        );
        let blacklist = OverwatchNodeBlacklist::<Test>::get(account(99));
        assert!(!blacklist);
    })
}

#[test]
fn test_do_set_min_active_node_stake_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_min_active_node_stake_epochs(6));
        assert_eq!(MinActiveNodeStakeEpochs::<Test>::get(), 6);

        assert_ok!(Network::do_set_min_active_node_stake_epochs(5));
        assert_eq!(MinActiveNodeStakeEpochs::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_delegate_stake_cooldown_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_delegate_stake_cooldown_epochs(6));
        assert_eq!(DelegateStakeCooldownEpochs::<Test>::get(), 6);

        assert_ok!(Network::do_set_delegate_stake_cooldown_epochs(5));
        assert_eq!(DelegateStakeCooldownEpochs::<Test>::get(), 5);

        assert_err!(
            Network::do_set_delegate_stake_cooldown_epochs(0),
            Error::<Test>::InvalidDelegateStakeCooldownEpochs
        );
    })
}

#[test]
fn test_do_set_node_delegate_stake_cooldown_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_node_delegate_stake_cooldown_epochs(6));
        assert_eq!(NodeDelegateStakeCooldownEpochs::<Test>::get(), 6);

        assert_ok!(Network::do_set_node_delegate_stake_cooldown_epochs(5));
        assert_eq!(NodeDelegateStakeCooldownEpochs::<Test>::get(), 5);

        assert_err!(
            Network::do_set_node_delegate_stake_cooldown_epochs(0),
            Error::<Test>::InvalidNodeDelegateStakeCooldownEpochs
        );
    })
}

#[test]
fn test_do_set_min_stake_cooldown_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_min_stake_cooldown_epochs(6));
        assert_eq!(StakeCooldownEpochs::<Test>::get(), 6);

        assert_ok!(Network::do_set_min_stake_cooldown_epochs(5));
        assert_eq!(StakeCooldownEpochs::<Test>::get(), 5);

        assert_err!(
            Network::do_set_min_stake_cooldown_epochs(0),
            Error::<Test>::InvalidStakeCooldownEpochs
        );
    })
}

#[test]
fn test_do_set_max_unbondings() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_unbondings(6));
        assert_eq!(MaxUnbondings::<Test>::get(), 6);

        assert_ok!(Network::do_set_max_unbondings(5));
        assert_eq!(MaxUnbondings::<Test>::get(), 5);

        assert_err!(
            Network::do_set_max_unbondings(257),
            Error::<Test>::InvalidMaxUnbondings
        );
    })
}

// Note: The BlockWeights::get().max_block calculations differ from runtime
// therefor we only test that it works and don't verify the values
#[test]
fn test_do_set_maximum_hooks_weight() {
    new_test_ext().execute_with(|| {
        let _ = env_logger::builder().is_test(true).try_init();
        log::error!(
            "current block weight {:?}",
            MaximumHooksWeightV2::<Test>::get()
        );

        let new_value = 10;
        // let expected_value = 
        //     sp_runtime::Perbill::from_percent(new_value) * BlockWeights::get().max_block;
        // log::error!(
        //     "expected_value {:?}",
        //     expected_value
        // );

        assert_ok!(Network::do_set_maximum_hooks_weight(new_value));
        // assert_eq!(MaximumHooksWeightV2::<Test>::get(), expected_value);
        log::error!(
            "current block weight {:?}",
            MaximumHooksWeightV2::<Test>::get()
        );

        let new_value = 5;
        // let expected_value =
        //     sp_runtime::Perbill::from_percent(new_value) * BlockWeights::get().max_block;
        // log::error!(
        //     "expected_value {:?}",
        //     expected_value
        // );

        assert_ok!(Network::do_set_maximum_hooks_weight(new_value));
        // assert_eq!(MaximumHooksWeightV2::<Test>::get(), expected_value);
        log::error!(
            "current block weight {:?}",
            MaximumHooksWeightV2::<Test>::get()
        );

        // =======
        // uncomment `assert!(false)` to see matching values
        // =======
        // assert!(false);

        assert_err!(
            Network::do_set_maximum_hooks_weight(0),
            Error::<Test>::InvalidPerbillPercent
        );

        assert_err!(
            Network::do_set_maximum_hooks_weight(101),
            Error::<Test>::InvalidPerbillPercent
        );
    })
}

#[test]
fn test_do_set_base_node_burn_amount() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_base_node_burn_amount(6));
        assert_eq!(BaseNodeBurnAmount::<Test>::get(), 6);

        assert_ok!(Network::do_set_base_node_burn_amount(5));
        assert_eq!(BaseNodeBurnAmount::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_min_node_burn_rate() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_node_burn_rates(5, 6));
        assert_eq!(MinNodeBurnRate::<Test>::get(), 5);
        assert_eq!(MaxNodeBurnRate::<Test>::get(), 6);


    })
}

#[test]
fn test_do_set_delegate_stake_subnet_removal_interval() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_delegate_stake_subnet_removal_interval(6));
        assert_eq!(DelegateStakeSubnetRemovalInterval::<Test>::get(), 6);

        assert_ok!(Network::do_set_delegate_stake_subnet_removal_interval(5));
        assert_eq!(DelegateStakeSubnetRemovalInterval::<Test>::get(), 5);

        assert_err!(
            Network::do_set_delegate_stake_subnet_removal_interval(0),
            Error::<Test>::InvalidDelegateStakeSubnetRemovalInterval
        );
    })
}

#[test]
fn test_do_set_min_subnet_removal_interval() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_subnet_removal_intervals(5, 6));
        assert_eq!(MinSubnetRemovalInterval::<Test>::get(), 5);
        assert_eq!(MaxSubnetRemovalInterval::<Test>::get(), 6);

    })
}

#[test]
fn test_do_set_subnet_pause_cooldown_epochs() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_subnet_pause_cooldown_epochs(6));
        assert_eq!(SubnetPauseCooldownEpochs::<Test>::get(), 6);

        assert_ok!(Network::do_set_subnet_pause_cooldown_epochs(5));
        assert_eq!(SubnetPauseCooldownEpochs::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_max_swap_queue_calls_per_block() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_swap_queue_calls_per_block(6));
        assert_eq!(MaxSwapQueueCallsPerBlock::<Test>::get(), 6);

        assert_ok!(Network::do_set_max_swap_queue_calls_per_block(5));
        assert_eq!(MaxSwapQueueCallsPerBlock::<Test>::get(), 5);
    })
}

#[test]
fn test_do_set_min_max_subnet_node() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_min_max_subnet_node(5, 6));
        assert_eq!(MinSubnetNodes::<Test>::get(), 5);
        assert_eq!(MaxSubnetNodes::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_tx_rate_limit() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_tx_rate_limit(5));
        assert_eq!(TxRateLimit::<Test>::get(), 5);

        assert_ok!(Network::do_set_tx_rate_limit(6));
        assert_eq!(TxRateLimit::<Test>::get(), 6);
    })
}

#[test]
fn test_do_set_min_subnet_delegate_stake_factor() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_min_subnet_delegate_stake_factor(5));
        assert_eq!(MinSubnetDelegateStakeFactor::<Test>::get(), 5);

        assert_err!(
            Network::do_set_min_subnet_delegate_stake_factor(
                Network::percentage_factor_as_u128() + 1
            ),
            Error::<Test>::InvalidPercent
        );
    })
}

#[test]
fn test_do_collective_remove_subnet() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        assert_ok!(Network::do_collective_remove_subnet(subnet_id));

        assert_eq!(
            *network_events().last().unwrap(),
            Event::SubnetDeactivated {
                subnet_id: subnet_id,
                reason: SubnetRemovalReason::Council
            }
        );
    })
}

#[test]
fn test_do_collective_remove_subnet_node() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
        let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        assert_ok!(Network::do_collective_remove_subnet_node(
            subnet_id,
            subnet_node_id
        ));

        assert_eq!(
            *network_events().last().unwrap(),
            Event::SubnetNodeRemoved {
                subnet_id: subnet_id,
                subnet_node_id: subnet_node_id
            }
        );
    })
}

#[test]
fn test_do_collective_remove_overwatch_node() {
    new_test_ext().execute_with(|| {
        let coldkey = account(1);
        let hotkey = account(2);
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let _ = Balances::deposit_creating(&coldkey.clone(), stake_amount + 500);

        make_overwatch_qualified(1);

        let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
        let uids = TotalOverwatchNodeUids::<Test>::get();
        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(!hotkeys.contains(&hotkey.clone()));

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            stake_amount,
        ));

        let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();
        assert_ok!(Network::do_collective_remove_overwatch_node(uid));

        assert_eq!(OverwatchNodes::<Test>::try_get(uid), Err(()));
        assert_eq!(OverwatchNodeIdHotkey::<Test>::try_get(uid), Err(()));
    })
}

#[test]
fn test_do_set_sigmoid_midpoint() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_sigmoid_midpoint(100));
        assert_eq!(SigmoidMidpoint::<Test>::get(), 100);

        assert_ok!(Network::do_set_sigmoid_midpoint(200));
        assert_eq!(SigmoidMidpoint::<Test>::get(), 200);

        assert_err!(
            Network::do_set_sigmoid_midpoint(Network::percentage_factor_as_u128() + 1),
            Error::<Test>::InvalidPercent
        );
    })
}

#[test]
fn test_do_set_min_subnet_registration_epochs() {
    new_test_ext().execute_with(|| {
        let registration_epochs = SubnetRegistrationEpochs::<Test>::get();

        assert_ok!(Network::do_set_min_subnet_registration_epochs(
            registration_epochs - 1
        ));
        assert_eq!(
            MinSubnetRegistrationEpochs::<Test>::get(),
            registration_epochs - 1
        );

        assert_ok!(Network::do_set_min_subnet_registration_epochs(
            registration_epochs - 2
        ));
        assert_eq!(
            MinSubnetRegistrationEpochs::<Test>::get(),
            registration_epochs - 2
        );

        assert_err!(
            Network::do_set_min_subnet_registration_epochs(registration_epochs + 1),
            Error::<Test>::InvalidMinSubnetRegistrationEpochs
        );
    })
}

#[test]
fn test_do_set_subnet_registration_epochs() {
    new_test_ext().execute_with(|| {
        let min_registration_epochs = MinSubnetRegistrationEpochs::<Test>::get();

        assert_ok!(Network::do_set_subnet_registration_epochs(
            min_registration_epochs + 1
        ));
        assert_eq!(
            SubnetRegistrationEpochs::<Test>::get(),
            min_registration_epochs + 1
        );

        assert_ok!(Network::do_set_subnet_registration_epochs(
            min_registration_epochs + 11
        ));
        assert_eq!(
            SubnetRegistrationEpochs::<Test>::get(),
            min_registration_epochs + 11
        );

        assert_err!(
            Network::do_set_subnet_registration_epochs(min_registration_epochs - 1),
            Error::<Test>::InvalidSubnetRegistrationEpochs
        );
    })
}

#[test]
fn test_do_set_max_subnet_node_score_penalty_threshold() {
    new_test_ext().execute_with(|| {
        let min_registration_epochs = MinSubnetRegistrationEpochs::<Test>::get();

        assert_ok!(Network::do_set_max_subnet_node_score_penalty_threshold(1));
        assert_eq!(MaxSubnetNodeScorePenaltyThreshold::<Test>::get(), 1);

        assert_ok!(Network::do_set_max_subnet_node_score_penalty_threshold(2));
        assert_eq!(MaxSubnetNodeScorePenaltyThreshold::<Test>::get(), 2);

        assert_err!(
            Network::do_set_max_subnet_node_score_penalty_threshold(
                (Network::percentage_factor_as_u128() + 1)
            ),
            Error::<Test>::InvalidPercent
        );
    })
}
