use super::mock::*;
use super::test_utils::*;
use crate::Event;
use crate::{
    AttestorMinRewardFactor, AttestorRewardExponent, BaseNodeBurnAmount, BaseSlashPercentage,
    BaseValidatorReward, ColdkeyReputationDecreaseFactor, ColdkeyReputationIncreaseFactor,
    DelegateStakeCooldownEpochs, DelegateStakeSubnetRemovalInterval, DelegateStakeWeightFactor,
    Error, InConsensusSubnetReputationFactor, InflationSigmoidMidpoint, InflationSigmoidSteepness,
    LessThanMinNodesSubnetReputationFactor, MaxBootnodes, MaxChurnLimit, MaxChurnLimitMultiplier,
    MaxDelegateStakePercentage, MaxEmergencySubnetNodes, MaxEmergencyValidatorEpochsMultiplier,
    MaxIdleClassificationEpochs, MaxIncludedClassificationEpochs, MaxMaxRegisteredNodes,
    MaxMinDelegateStakeMultiplier, MaxMinSubnetNodeReputation, MaxNodeBurnRate,
    MaxNodeReputationFactor, MaxOverwatchNodes, MaxPauseEpochsSubnetReputationFactor,
    MaxQueueEpochs, MaxRewardRateDecrease, MaxSlashAmount, MaxSubnetBootnodeAccess,
    MaxSubnetDelegateStakeRewardsPercentageChange, MaxSubnetMinStake,
    MaxSubnetNodeMinWeightDecreaseReputationThreshold, MaxSubnetNodes, MaxSubnetPauseEpochs,
    MaxSubnetRemovalInterval, MaxSubnets, MaxSwapQueueCallsPerBlock, MaxUnbondings,
    MaximumHooksWeightV2, MinActiveNodeStakeEpochs, MinAttestationPercentage, MinChurnLimit,
    MinChurnLimitMultiplier, MinDelegateStakeDeposit, MinDelegateStakePercentage,
    MinIdleClassificationEpochs, MinIncludedClassificationEpochs, MinMaxRegisteredNodes,
    MinMinSubnetNodeReputation, MinNodeBurnRate, MinNodeReputationFactor, MinQueueEpochs,
    MinRegistrationCost, MinSubnetDelegateStakeFactor, MinSubnetMinStake, MinSubnetNodes,
    MinSubnetRegistrationEpochs, MinSubnetRemovalInterval, MinSubnetReputation,
    NetworkMaxStakeBalance, NewRegistrationCostMultiplier, NodeDelegateStakeCooldownEpochs,
    NodeRewardRateUpdatePeriod, NotInConsensusSubnetReputationFactor, OverwatchCommitCutoffPercent,
    OverwatchEpochLengthMultiplier, OverwatchMinAge, OverwatchMinAvgAttestationRatio,
    OverwatchMinDiversificationRatio, OverwatchMinRepScore, OverwatchMinStakeBalance,
    OverwatchNodeBlacklist, OverwatchStakeWeightFactor, OverwatchWeightFactor, QueueImmunityEpochs,
    RegistrationCostAlpha, RegistrationCostDecayBlocks, StakeCooldownEpochs,
    SubnetDelegateStakeRewardsUpdatePeriod, SubnetDistributionPower, SubnetEnactmentEpochs,
    SubnetName, SubnetOwnerPercentage, SubnetPauseCooldownEpochs, SubnetRegistrationEpochs,
    SubnetWeightFactors, SubnetWeightFactorsData, SuperMajorityAttestationRatio, TxRateLimit,
    ValidatorAbsentSubnetReputationFactor, ValidatorRewardK, ValidatorRewardMidpoint,
};
use frame_support::{assert_err, assert_ok};

//
// Admin / Collective Extrinsic Tests
//

// === Pause/Unpause Tests ===

#[test]
fn test_collective_pause() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        // Pause should succeed with collective origin
        assert_ok!(Network::pause(RuntimeOrigin::from(
            pallet_collective::RawOrigin::Members(2, 3)
        )));

        // Verify event emitted
        assert_eq!(*network_events().last().unwrap(), Event::SetTxPause {});
    });
}

#[test]
fn test_collective_pause_fails_without_majority() {
    new_test_ext().execute_with(|| {
        // Should fail with regular signed origin
        assert_err!(
            Network::pause(RuntimeOrigin::signed(account(1))),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_collective_unpause() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        // First pause
        assert_ok!(Network::pause(RuntimeOrigin::from(
            pallet_collective::RawOrigin::Members(2, 3)
        )));

        // Then unpause
        assert_ok!(Network::unpause(RuntimeOrigin::from(
            pallet_collective::RawOrigin::Members(2, 3)
        )));

        // Verify event emitted
        assert_eq!(*network_events().last().unwrap(), Event::SetTxUnpause {});
    });
}

// === Removal Function Tests ===

#[test]
fn test_collective_remove_subnet() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        // Build subnet
        build_activated_subnet(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name).unwrap();

        // Remove subnet with collective origin (super majority required)
        assert_ok!(Network::collective_remove_subnet(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            subnet_id
        ));
    });
}

#[test]
fn test_collective_remove_subnet_fails_without_super_majority() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name).unwrap();

        // Should fail with regular majority
        assert_err!(
            Network::collective_remove_subnet(
                RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
                subnet_id
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

// === Parameter Setter Tests ===

#[test]
fn test_set_subnet_owner_percentage() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 15000000000000000; // 1.5%

        // Set with super majority
        assert_ok!(Network::set_subnet_owner_percentage(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        // Verify storage updated
        assert_eq!(SubnetOwnerPercentage::<Test>::get(), new_value);

        // Verify event
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSubnetOwnerPercentage(new_value)
        );
    });
}

#[test]
fn test_set_subnet_owner_percentage_fails_without_super_majority() {
    new_test_ext().execute_with(|| {
        assert_err!(
            Network::set_subnet_owner_percentage(
                RuntimeOrigin::signed(account(1)),
                15000000000000000
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_set_max_subnets() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 20;

        assert_ok!(Network::set_max_subnets(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MaxSubnets::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxSubnets(new_value)
        );
    });
}

#[test]
fn test_set_max_pause_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 100;

        assert_ok!(Network::set_max_pause_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(MaxSubnetPauseEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxSubnetPauseEpochs(new_value)
        );
    });
}

#[test]
fn test_set_min_registration_cost() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 5000000000000000000000; // 5000 tokens

        assert_ok!(Network::set_min_registration_cost(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MinRegistrationCost::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMinRegistrationCost(new_value)
        );
    });
}

#[test]
fn test_set_registration_cost_alpha() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 60000000000000000; // 6%

        assert_ok!(Network::set_registration_cost_alpha(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(RegistrationCostAlpha::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetRegistrationCostAlpha(new_value)
        );
    });
}

#[test]
fn test_set_max_min_delegate_stake_multiplier() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = Network::percentage_factor_as_u128() + 1;

        assert_ok!(Network::set_max_min_delegate_stake_multiplier(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(5, 5)),
            new_value
        ));

        assert_eq!(MaxMinDelegateStakeMultiplier::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxMinDelegateStakeMultiplier(new_value)
        );
    });
}

#[test]
fn test_set_churn_limits() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u32 = 1;
        let max: u32 = 5;

        assert_ok!(Network::set_churn_limits(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            min,
            max
        ));

        assert_eq!(MinChurnLimit::<Test>::get(), min);
        assert_eq!(MaxChurnLimit::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetChurnLimits(min, max)
        );
    });
}

#[test]
fn test_set_min_delegate_stake_deposit() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 100000000000000000000; // 100 tokens

        assert_ok!(Network::set_min_delegate_stake_deposit(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MinDelegateStakeDeposit::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMinDelegateStakeDeposit(new_value)
        );
    });
}

#[test]
fn test_set_base_validator_reward() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 50000000000000000; // 5%

        assert_ok!(Network::set_base_validator_reward(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(BaseValidatorReward::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetBaseValidatorReward(new_value)
        );
    });
}

#[test]
fn test_set_super_majority_attestation_ratio() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 800000000000000000; // 80%

        assert_ok!(Network::set_super_majority_attestation_ratio(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(SuperMajorityAttestationRatio::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSuperMajorityAttestationRatio(new_value)
        );
    });
}

#[test]
fn test_set_overwatch_epoch_length_multiplier() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 10;

        assert_ok!(Network::set_overwatch_epoch_length_multiplier(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(OverwatchEpochLengthMultiplier::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetOverwatchEpochLengthMultiplier(new_value)
        );
    });
}

#[test]
fn test_set_overwatch_commit_cutoff_percent() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 700000000000000000; // 70%

        assert_ok!(Network::set_overwatch_commit_cutoff_percent(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(OverwatchCommitCutoffPercent::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetOverwatchCommitCutoffPercent(new_value)
        );
    });
}

#[test]
fn test_set_min_subnet_delegate_stake_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 500000000000000000; // 50%

        assert_ok!(Network::set_min_subnet_delegate_stake_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MinSubnetDelegateStakeFactor::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMinSubnetDelegateStakeFactor(new_value)
        );
    });
}

// === Edge Case Tests ===

#[test]
fn test_set_parameter_with_invalid_origin_fails() {
    new_test_ext().execute_with(|| {
        // Try various admin functions with wrong origin
        assert_err!(
            Network::set_max_subnets(RuntimeOrigin::signed(account(1)), 20),
            sp_runtime::DispatchError::BadOrigin
        );

        assert_err!(
            Network::set_min_registration_cost(RuntimeOrigin::signed(account(1)), 5000),
            sp_runtime::DispatchError::BadOrigin
        );

        assert_err!(
            Network::set_churn_limits(RuntimeOrigin::signed(account(1)), 1, 5),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_multiple_parameter_updates() {
    new_test_ext().execute_with(|| {
        // Update multiple parameters in sequence
        assert_ok!(Network::set_max_subnets(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            25
        ));

        assert_ok!(Network::set_max_pause_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            120
        ));

        assert_ok!(Network::set_base_validator_reward(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            60000000000000000
        ));

        // Verify all updated
        assert_eq!(MaxSubnets::<Test>::get(), 25);
        assert_eq!(MaxSubnetPauseEpochs::<Test>::get(), 120);
        assert_eq!(BaseValidatorReward::<Test>::get(), 60000000000000000);
    });
}

// === Additional Parameter Setter Tests ===

#[test]
fn test_set_max_bootnodes() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 15;

        assert_ok!(Network::set_max_bootnodes(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(MaxBootnodes::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxBootnodes(new_value)
        );
    });
}

#[test]
fn test_set_max_subnet_bootnodes_access() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 10;

        assert_ok!(Network::set_max_subnet_bootnodes_access(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(MaxSubnetBootnodeAccess::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxSubnetBootnodeAccess(new_value)
        );
    });
}

#[test]
fn test_set_registration_cost_delay_blocks() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 1000;

        assert_ok!(Network::set_registration_cost_delay_blocks(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(RegistrationCostDecayBlocks::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetRegistrationCostDecayBlocks(new_value)
        );
    });
}

#[test]
fn test_set_new_registration_cost_multiplier() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 2000000000000000000; // 2.0

        assert_ok!(Network::set_new_registration_cost_multiplier(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(NewRegistrationCostMultiplier::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetNewRegistrationCostMultiplier(new_value)
        );
    });
}

#[test]
fn test_set_churn_limit_multipliers() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u32 = 1;
        let max: u32 = 10;

        assert_ok!(Network::set_churn_limit_multipliers(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            min,
            max
        ));

        assert_eq!(MinChurnLimitMultiplier::<Test>::get(), min);
        assert_eq!(MaxChurnLimitMultiplier::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetChurnLimitMultipliers(min, max)
        );
    });
}

#[test]
fn test_set_queue_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u32 = 2;
        let max: u32 = 10;

        assert_ok!(Network::set_queue_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            min,
            max
        ));

        assert_eq!(MinQueueEpochs::<Test>::get(), min);
        assert_eq!(MaxQueueEpochs::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetQueueEpochs(min, max)
        );
    });
}

#[test]
fn test_set_min_idle_classification_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 5;

        assert_ok!(Network::set_min_idle_classification_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(MinIdleClassificationEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMinIdleClassificationEpochs(new_value)
        );
    });
}

#[test]
fn test_set_max_idle_classification_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 20;

        assert_ok!(Network::set_max_idle_classification_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(MaxIdleClassificationEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxIdleClassificationEpochs(new_value)
        );
    });
}

#[test]
fn test_set_subnet_activation_enactment_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 3;

        assert_ok!(Network::set_subnet_activation_enactment_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(SubnetEnactmentEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSubnetEnactmentEpochs(new_value)
        );
    });
}

#[test]
fn test_set_included_classification_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u32 = 2;
        let max: u32 = 15;

        assert_ok!(Network::set_included_classification_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            min,
            max
        ));

        assert_eq!(MinIncludedClassificationEpochs::<Test>::get(), min);
        assert_eq!(MaxIncludedClassificationEpochs::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetIncludedClassificationEpochs(min, max)
        );
    });
}

#[test]
fn test_set_subnet_stakes() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u128 = 1000000000000000000;
        let max: u128 = 100000000000000000000000;

        assert_ok!(Network::set_subnet_stakes(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            min,
            max
        ));

        assert_eq!(MinSubnetMinStake::<Test>::get(), min);
        assert_eq!(MaxSubnetMinStake::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSubnetStakesLimits(min, max)
        );
    });
}

#[test]
fn test_set_delegate_stake_percentages() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u128 = 100000000000000000;
        let max: u128 = 900000000000000000;

        assert_ok!(Network::set_delegate_stake_percentages(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            min,
            max
        ));

        assert_eq!(MinDelegateStakePercentage::<Test>::get(), min);
        assert_eq!(MaxDelegateStakePercentage::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetDelegateStakePercentages(min, max)
        );
    });
}

#[test]
fn test_set_min_max_registered_nodes() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u32 = 5;
        let max: u32 = 200;

        assert_ok!(Network::set_min_max_registered_nodes(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            min,
            max
        ));

        assert_eq!(MinMaxRegisteredNodes::<Test>::get(), min);
        assert_eq!(MaxMaxRegisteredNodes::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMinMaxRegisteredNodes(min, max)
        );
    });
}

#[test]
fn test_set_max_subnet_delegate_stake_rewards_percentage_change() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 50000000000000000;

        assert_ok!(
            Network::set_max_subnet_delegate_stake_rewards_percentage_change(
                RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
                new_value
            )
        );

        assert_eq!(
            MaxSubnetDelegateStakeRewardsPercentageChange::<Test>::get(),
            new_value
        );
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxSubnetDelegateStakeRewardsPercentageChange(new_value)
        );
    });
}

#[test]
fn test_set_subnet_delegate_stake_rewards_update_period() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 100;

        assert_ok!(Network::set_subnet_delegate_stake_rewards_update_period(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(
            SubnetDelegateStakeRewardsUpdatePeriod::<Test>::get(),
            new_value
        );
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSubnetDelegateStakeRewardsUpdatePeriod(new_value)
        );
    });
}

#[test]
fn test_set_min_attestation_percentage() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 600000000000000000;

        assert_ok!(Network::set_min_attestation_percentage(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MinAttestationPercentage::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMinAttestationPercentage(new_value)
        );
    });
}

#[test]
fn test_set_base_slash_percentage() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 100000000000000000;

        assert_ok!(Network::set_base_slash_percentage(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(BaseSlashPercentage::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetBaseSlashPercentage(new_value)
        );
    });
}

#[test]
fn test_set_max_slash_amount() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 10000000000000000000000;

        assert_ok!(Network::set_max_slash_amount(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MaxSlashAmount::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxSlashAmount(new_value)
        );
    });
}

#[test]
fn test_set_reputation_increase_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 1000000000000000000;

        assert_ok!(Network::set_reputation_increase_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(ColdkeyReputationIncreaseFactor::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetColdkeyReputationIncreaseFactor(new_value)
        );
    });
}

#[test]
fn test_set_reputation_decrease_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 950000000000000000;

        assert_ok!(Network::set_reputation_decrease_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(ColdkeyReputationDecreaseFactor::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetColdkeyReputationDecreaseFactor(new_value)
        );
    });
}

#[test]
fn test_set_network_max_stake_balance() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 1000000000000000000000000;

        assert_ok!(Network::set_network_max_stake_balance(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(NetworkMaxStakeBalance::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetNetworkMaxStakeBalance(new_value)
        );
    });
}

#[test]
fn test_set_node_reward_rate_update_period() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 50;

        assert_ok!(Network::set_node_reward_rate_update_period(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(NodeRewardRateUpdatePeriod::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetNodeRewardRateUpdatePeriod(new_value)
        );
    });
}

#[test]
fn test_set_max_reward_rate_decrease() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 100000000000000000;

        assert_ok!(Network::set_max_reward_rate_decrease(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MaxRewardRateDecrease::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxRewardRateDecrease(new_value)
        );
    });
}

#[test]
fn test_set_subnet_distribution_power() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 1500000000000000000;

        assert_ok!(Network::set_subnet_distribution_power(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(SubnetDistributionPower::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSubnetDistributionPower(new_value)
        );
    });
}

#[test]
fn test_set_delegate_stake_weight_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 1;

        assert_ok!(Network::set_delegate_stake_weight_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(DelegateStakeWeightFactor::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetDelegateStakeWeightFactor(new_value)
        );
    });
}

#[test]
fn test_set_inflation_sigmoid_steepness() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 5000000000000000000;

        assert_ok!(Network::set_inflation_sigmoid_steepness(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(InflationSigmoidSteepness::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSigmoidSteepness(new_value)
        );
    });
}

#[test]
fn test_set_max_overwatch_nodes() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 50;

        assert_ok!(Network::set_max_overwatch_nodes(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MaxOverwatchNodes::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxOverwatchNodes(new_value)
        );
    });
}

#[test]
fn test_set_overwatch_min_diversification_ratio() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 300000000000000000;

        assert_ok!(Network::set_overwatch_min_diversification_ratio(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(OverwatchMinDiversificationRatio::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetOverwatchMinDiversificationRatio(new_value)
        );
    });
}

#[test]
fn test_set_overwatch_min_rep_score() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 500000000000000000;

        assert_ok!(Network::set_overwatch_min_rep_score(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(OverwatchMinRepScore::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetOverwatchMinRepScore(new_value)
        );
    });
}

#[test]
fn test_set_overwatch_min_avg_attestation_ratio() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 600000000000000000;

        assert_ok!(Network::set_overwatch_min_avg_attestation_ratio(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(OverwatchMinAvgAttestationRatio::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetOverwatchMinAvgAttestationRatio(new_value)
        );
    });
}

#[test]
fn test_set_overwatch_min_age() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 100;

        assert_ok!(Network::set_overwatch_min_age(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(OverwatchMinAge::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetOverwatchMinAge(new_value)
        );
    });
}

#[test]
fn test_set_overwatch_min_stake_balance() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 10000000000000000000000;

        assert_ok!(Network::set_overwatch_min_stake_balance(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(OverwatchMinStakeBalance::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetOverwatchMinStakeBalance(new_value)
        );
    });
}

#[test]
fn test_set_min_max_subnet_node() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u32 = 5;
        let max: u32 = 500;

        assert_ok!(Network::set_min_max_subnet_node(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            min,
            max
        ));

        assert_eq!(MinSubnetNodes::<Test>::get(), min);
        assert_eq!(MaxSubnetNodes::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMinMaxSubnetNodes(min, max)
        );
    });
}

#[test]
fn test_set_tx_rate_limit() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 5;

        assert_ok!(Network::set_tx_rate_limit(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(TxRateLimit::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetTxRateLimit(new_value)
        );
    });
}

#[test]
fn test_set_delegate_stake_subnet_removal_interval() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 50;

        assert_ok!(Network::set_delegate_stake_subnet_removal_interval(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(DelegateStakeSubnetRemovalInterval::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetDelegateStakeSubnetRemovalInterval(new_value)
        );
    });
}

#[test]
fn test_set_subnet_removal_interval() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u32 = 100;
        let max: u32 = 1000;

        assert_ok!(Network::set_subnet_removal_intervals(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            min,
            max
        ));

        assert_eq!(MinSubnetRemovalInterval::<Test>::get(), min);
        assert_eq!(MaxSubnetRemovalInterval::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSubnetRemovalIntervals(min, max)
        );
    });
}

#[test]
fn test_set_subnet_pause_cooldown_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 10;

        assert_ok!(Network::set_subnet_pause_cooldown_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(SubnetPauseCooldownEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSubnetPauseCooldownEpochs(new_value)
        );
    });
}

#[test]
fn test_set_delegate_stake_cooldown_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 5;

        assert_ok!(Network::set_delegate_stake_cooldown_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(DelegateStakeCooldownEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetDelegateStakeCooldownEpochs(new_value)
        );
    });
}

#[test]
fn test_set_node_delegate_stake_cooldown_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 5;

        assert_ok!(Network::set_node_delegate_stake_cooldown_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(NodeDelegateStakeCooldownEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetNodeDelegateStakeCooldownEpochs(new_value)
        );
    });
}

#[test]
fn test_set_min_stake_cooldown_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 3;

        assert_ok!(Network::set_min_stake_cooldown_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(StakeCooldownEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetStakeCooldownEpochs(new_value)
        );
    });
}

#[test]
fn test_set_max_unbondings() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 100;

        assert_ok!(Network::set_max_unbondings(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MaxUnbondings::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxUnbondings(new_value)
        );
    });
}

#[test]
fn test_set_max_swap_queue_calls_per_block() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 20;

        assert_ok!(Network::set_max_swap_queue_calls_per_block(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MaxSwapQueueCallsPerBlock::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxSwapQueueCallsPerBlock(new_value)
        );
    });
}

// === Collective Removal Tests ===

#[test]
fn test_collective_remove_subnet_node() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name).unwrap();

        // Remove a subnet node with super majority
        assert_ok!(Network::collective_remove_subnet_node(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            subnet_id,
            1
        ));
    });
}

#[test]
fn test_collective_remove_overwatch_node() {
    new_test_ext().execute_with(|| {
        // This test requires setting up an overwatch node first
        // For now, just verify origin requirements
        assert_err!(
            Network::collective_remove_overwatch_node(RuntimeOrigin::signed(account(1)), 1),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_collective_set_coldkey_overwatch_node_eligibility() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let coldkey = account(1);
        let value = true;

        assert_ok!(Network::collective_set_coldkey_overwatch_node_eligibility(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            coldkey.clone(),
            value
        ));

        assert_eq!(OverwatchNodeBlacklist::<Test>::get(coldkey), value);
    });
}

// === Epoch and Timing Parameter Tests ===

#[test]
fn test_set_min_subnet_registration_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let registration_epochs = SubnetRegistrationEpochs::<Test>::get();
        let new_value: u32 = registration_epochs - 1;

        assert_ok!(Network::set_min_subnet_registration_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MinSubnetRegistrationEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMinSubnetRegistrationEpochs(new_value)
        );
    });
}

#[test]
fn test_set_subnet_registration_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min_registration_epochs = MinSubnetRegistrationEpochs::<Test>::get();
        let new_value: u32 = min_registration_epochs + 1;

        assert_ok!(Network::set_subnet_registration_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(SubnetRegistrationEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSubnetRegistrationEpochs(new_value)
        );
    });
}

#[test]
fn test_set_min_active_node_stake_epochs() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 3;

        assert_ok!(Network::set_min_active_node_stake_epochs(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(4, 5)),
            new_value
        ));

        assert_eq!(MinActiveNodeStakeEpochs::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMinActiveNodeStakeEpochs(new_value)
        );
    });
}

// === Sigmoid  and Burn Parameter Tests ===

#[test]
fn test_set_sigmoid_midpoint() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 500000000000000000;

        assert_ok!(Network::set_sigmoid_midpoint(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(InflationSigmoidMidpoint::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetInflationSigmoidMidpoint(new_value)
        );
    });
}

#[test]
fn test_set_maximum_hooks_weight() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u32 = 100;

        assert_ok!(Network::set_maximum_hooks_weight(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        // assert_eq!(MaximumHooksWeightV2::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaximumHooksWeight(new_value)
        );
    });
}

#[test]
fn test_set_base_node_burn_amount() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 1000000000000000000;

        assert_ok!(Network::set_base_node_burn_amount(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(BaseNodeBurnAmount::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetBaseNodeBurnAmount(new_value)
        );
    });
}

#[test]
fn test_set_node_burn_rates() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u128 = 500000000000000000;
        let max: u128 = 1000000000000000000;

        assert_ok!(Network::set_node_burn_rates(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            min,
            max
        ));

        assert_eq!(MinNodeBurnRate::<Test>::get(), min);
        assert_eq!(MaxNodeBurnRate::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetNodeBurnRates(min, max)
        );
    });
}

// === Validation and Reward Parameter Tests ===

#[test]
fn test_set_max_subnet_node_min_weight_decrease_reputation_threshold() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 100000000000000000;

        assert_ok!(
            Network::set_max_subnet_node_min_weight_decrease_reputation_threshold(
                RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
                new_value
            )
        );

        assert_eq!(
            MaxSubnetNodeMinWeightDecreaseReputationThreshold::<Test>::get(),
            new_value
        );
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxSubnetNodeMinWeightDecreaseReputationThreshold(new_value)
        );
    });
}

#[test]
fn test_set_validator_reward_k() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u64 = 5;

        assert_ok!(Network::set_validator_reward_k(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(ValidatorRewardK::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetValidatorRewardK(new_value)
        );
    });
}

#[test]
fn test_set_validator_reward_midpoint() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 600000000000000000;

        assert_ok!(Network::set_validator_reward_midpoint(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(ValidatorRewardMidpoint::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetValidatorRewardMidpoint(new_value)
        );
    });
}

#[test]
fn test_set_attestor_reward_exponent() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u64 = 3;

        assert_ok!(Network::set_attestor_reward_exponent(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(AttestorRewardExponent::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetAttestorRewardExponent(new_value)
        );
    });
}

#[test]
fn test_set_attestor_min_reward_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 200000000000000000;

        assert_ok!(Network::set_attestor_min_reward_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(AttestorMinRewardFactor::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetAttestorMinRewardFactor(new_value)
        );
    });
}

// === Reputation Parameter Tests ===

#[test]
fn test_set_min_max_node_reputation() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u128 = 1;
        let max: u128 = 2;

        assert_ok!(Network::set_min_max_node_reputation(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            min,
            max
        ));

        assert_eq!(MinMinSubnetNodeReputation::<Test>::get(), min);
        assert_eq!(MaxMinSubnetNodeReputation::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetNodeReputationLimits(min, max)
        );
    });
}

#[test]
fn test_set_min_max_node_reputation_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let min: u128 = 1;
        let max: u128 = 2;

        assert_ok!(Network::set_min_max_node_reputation_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            min,
            max
        ));

        assert_eq!(MinNodeReputationFactor::<Test>::get(), min);
        assert_eq!(MaxNodeReputationFactor::<Test>::get(), max);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetNodeReputationFactors(min, max)
        );
    });
}

#[test]
fn test_set_min_subnet_reputation() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 500000000000000000;

        assert_ok!(Network::set_min_subnet_reputation(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(MinSubnetReputation::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMinSubnetReputation(new_value)
        );
    });
}

#[test]
fn test_set_not_in_consensus_subnet_reputation_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 950000000000000000;

        assert_ok!(Network::set_not_in_consensus_subnet_reputation_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(
            NotInConsensusSubnetReputationFactor::<Test>::get(),
            new_value
        );
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetNotInConsensusSubnetReputationFactor(new_value)
        );
    });
}

#[test]
fn test_set_less_than_min_nodes_subnet_reputation_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 980000000000000000;

        assert_ok!(Network::set_less_than_min_nodes_subnet_reputation_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(
            LessThanMinNodesSubnetReputationFactor::<Test>::get(),
            new_value
        );
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetLessThanMinNodesSubnetReputationFactor(new_value)
        );
    });
}

#[test]
fn test_set_validator_proposal_absent_subnet_reputation_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 900000000000000000;

        assert_ok!(
            Network::set_validator_proposal_absent_subnet_reputation_factor(
                RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
                new_value
            )
        );

        assert_eq!(
            ValidatorAbsentSubnetReputationFactor::<Test>::get(),
            new_value
        );
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetValidatorAbsentSubnetReputationFactor(new_value)
        );
    });
}

#[test]
fn test_set_in_consensus_subnet_reputation_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 1;

        assert_ok!(Network::set_in_consensus_subnet_reputation_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(InConsensusSubnetReputationFactor::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetInConsensusSubnetReputationFactor(new_value)
        );
    });
}

// === Weight Factor Tests ===

#[test]
fn test_set_overwatch_weight_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 200000000000000000;

        assert_ok!(Network::set_overwatch_weight_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(OverwatchWeightFactor::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetOverwatchWeightFactor(new_value)
        );
    });
}

#[test]
fn test_set_max_emergency_validator_epochs_multiplier() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 1000000000000000000;

        assert_ok!(Network::set_max_emergency_validator_epochs_multiplier(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(
            MaxEmergencyValidatorEpochsMultiplier::<Test>::get(),
            new_value
        );
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxEmergencyValidatorEpochsMultiplier(new_value)
        );
    });
}

#[test]
fn test_set_max_emergency_subnet_nodes() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value = MinSubnetNodes::<Test>::get() + 1;

        assert_ok!(Network::set_max_emergency_subnet_nodes(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(MaxEmergencySubnetNodes::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxEmergencySubnetNodes(new_value)
        );

        let new_value = MinSubnetNodes::<Test>::get() - 1;

        assert_err!(
            Network::set_max_emergency_subnet_nodes(
                RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
                new_value
            ),
            Error::<Test>::InvalidMaxEmergencySubnetNodes
        );
    });
}

#[test]
fn test_set_overwatch_stake_weight_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let new_value: u128 = 1500000000000000000;

        assert_ok!(Network::set_overwatch_stake_weight_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            new_value
        ));

        assert_eq!(OverwatchStakeWeightFactor::<Test>::get(), new_value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetOverwatchStakeWeightFactor(new_value)
        );
    });
}

#[test]
fn test_set_subnet_weight_factors() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let value = SubnetWeightFactorsData {
            delegate_stake: 400000000000000000,
            node_count: 300000000000000000,
            net_flow: 300000000000000000,
        };

        assert_ok!(Network::set_subnet_weight_factors(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            value.clone()
        ));

        assert_eq!(SubnetWeightFactors::<Test>::get(), value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetSubnetWeightFactors(value)
        );
    });
}

#[test]
fn test_set_max_pause_epochs_subnet_reputation_factor() {
    new_test_ext().execute_with(|| {
        System::set_block_number(System::block_number() + 1);

        let value: u128 = 5;

        assert_ok!(Network::set_max_pause_epochs_subnet_reputation_factor(
            RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
            value.clone()
        ));

        assert_eq!(MaxPauseEpochsSubnetReputationFactor::<Test>::get(), value);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SetMaxPauseEpochsSubnetReputationFactor(value)
        );
    });
}
