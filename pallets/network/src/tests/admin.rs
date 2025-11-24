use super::mock::*;
use super::test_utils::*;
use crate::Event;
use crate::{
    BaseValidatorReward, DelegateStakeSubnetRemovalInterval, Error, MaxChurnLimit,
    MaxMinDelegateStakeMultiplier, MaxSubnetPauseEpochs, MaxSubnets, MinChurnLimit,
    MinDelegateStakeDeposit, MinRegistrationCost, MinSubnetDelegateStakeFactor, MinSubnetMinStake,
    OverwatchCommitCutoffPercent, OverwatchEpochLengthMultiplier, QueueImmunityEpochs,
    RegistrationCostAlpha, RegistrationCostDecayBlocks, SubnetName, SubnetOwnerPercentage,
    SuperMajorityAttestationRatio,
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
        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);
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

        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);
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

        let new_value: u128 = 5;

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
