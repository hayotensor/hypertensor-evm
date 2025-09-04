use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use crate::{
    ColdkeyHotkeys, Error, HotkeyOverwatchNodeId, HotkeySubnetNodeId, MaxSubnetNodes, MaxSubnets,
    MinSubnetDelegateStakeFactor, MinSubnetNodes, MinSubnetRegistrationEpochs,
    NetworkMinStakeBalance, OverwatchEpochLengthMultiplier, OverwatchNodeIdHotkey, OverwatchNodes,
    ProposalMinSubnetNodes, SigmoidMidpoint, SigmoidSteepness, SubnetName, SubnetOwnerPercentage,
    SubnetRegistrationEpochs, SubnetRemovalReason, TotalActiveSubnets, TotalOverwatchNodeUids,
    TotalOverwatchNodes, TxPause, TxRateLimit,
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
fn test_do_set_proposal_min_subnet_nodes() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_proposal_min_subnet_nodes(1));
        assert_eq!(ProposalMinSubnetNodes::<Test>::get(), 1);

        assert_ok!(Network::do_set_proposal_min_subnet_nodes(2));
        assert_eq!(ProposalMinSubnetNodes::<Test>::get(), 2);
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
fn test_do_set_min_subnet_nodes() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_min_subnet_nodes(5));
        assert_eq!(MinSubnetNodes::<Test>::get(), 5);

        assert_ok!(Network::do_set_min_subnet_nodes(6));
        assert_eq!(MinSubnetNodes::<Test>::get(), 6);

        assert_err!(
            Network::do_set_min_subnet_nodes(0),
            Error::<Test>::InvalidMinSubnetNodes
        );

        assert_err!(
            Network::do_set_min_subnet_nodes(MaxSubnetNodes::<Test>::get() + 1),
            Error::<Test>::InvalidMinSubnetNodes
        );
    })
}

#[test]
fn test_do_set_max_subnet_nodes() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_max_subnet_nodes(5));
        assert_eq!(MaxSubnetNodes::<Test>::get(), 5);

        assert_ok!(Network::do_set_max_subnet_nodes(6));
        assert_eq!(MaxSubnetNodes::<Test>::get(), 6);

        assert_err!(
            Network::do_set_max_subnet_nodes(MinSubnetNodes::<Test>::get()),
            Error::<Test>::InvalidMaxSubnetNodes
        );
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
        let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
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
        let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
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
        let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
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
fn test_do_set_sigmoid_steepness() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_sigmoid_steepness(100));
        assert_eq!(SigmoidSteepness::<Test>::get(), 100);

        assert_ok!(Network::do_set_sigmoid_steepness(200));
        assert_eq!(SigmoidSteepness::<Test>::get(), 200);
    })
}

#[test]
fn test_do_set_overwatch_epoch_length_multiplier() {
    new_test_ext().execute_with(|| {
        assert_ok!(Network::do_set_overwatch_epoch_length_multiplier(100));
        assert_eq!(OverwatchEpochLengthMultiplier::<Test>::get(), 100);

        assert_ok!(Network::do_set_overwatch_epoch_length_multiplier(200));
        assert_eq!(OverwatchEpochLengthMultiplier::<Test>::get(), 200);
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
