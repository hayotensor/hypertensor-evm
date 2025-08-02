use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::btree_set::BTreeSet;
use crate::{
  Error,
  SubnetName, 
  SubnetOwner,
  SubnetsData,
  SubnetState,
  RegisteredSubnetNodesData,
  HotkeySubnetNodeId,
  SubnetNodeClass,
  SubnetNode,
  SubnetNodeClassification,
  NetworkMinStakeBalance,
  SubnetRemovalReason,
};
use sp_runtime::traits::TrailingZeroInput;
use codec::{Decode, Encode};
use sp_runtime::BoundedVec;

//
//
//
//
//
//
//
// Subnets Add/Remove
//
//
//
//
//
//
//
// owner_pause_subnet
// owner_unpause_subnet
// owner_deactivate_subnet
// owner_update_name
// owner_update_repo
// owner_update_description
// owner_update_misc
// owner_update_churn_limit
// owner_update_registration_queue_epochs
// owner_update_activation_grace_epochs
// owner_update_idle_classification_epochs
// owner_update_included_classification_epochs
// owner_update_max_node_penalties
// owner_add_initial_coldkeys
// owner_remove_initial_coldkeys
// owner_update_key_types
// owner_update_node_removal_policy
// owner_remove_subnet_node
// owner_update_min_stake
// owner_update_max_stake
// owner_update_delegate_stake_percentage
// owner_update_max_registered_nodes
// transfer_subnet_ownership
// accept_subnet_ownership
// owner_add_bootnode_access

#[test]
fn test_owner_pause_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    // Transfer to new owner
    assert_ok!(Network::owner_pause_subnet(
      RuntimeOrigin::signed(original_owner),
      subnet_id,
    ));

    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet_data.state, SubnetState::Paused);
    assert_eq!(subnet_data.start_epoch, epoch);
  });
}

#[test]
fn test_owner_unpause_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let coldkey = account(1000);
    let hotkey = account(1001);
    let start_epoch = epoch + 100;

    let hotkey_subnet_node_id = 1000;
    RegisteredSubnetNodesData::<Test>::insert(subnet_id, hotkey_subnet_node_id, SubnetNode {
      id: hotkey_subnet_node_id,
      hotkey: hotkey.clone(),
      peer_id: peer(0),
      bootnode_peer_id: peer(0),
      client_peer_id: peer(0),
      bootnode: None,
      delegate_reward_rate: 10,
      last_delegate_reward_rate_update: 0,
      classification: SubnetNodeClassification {
        node_class: SubnetNodeClass::Validator,
        start_epoch: start_epoch,
      },
      a: Some(BoundedVec::new()),
      b: Some(BoundedVec::new()),
      c: Some(BoundedVec::new()),
    });

    // Transfer to new owner
    assert_ok!(Network::owner_pause_subnet(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
    ));

    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet_data.state, SubnetState::Paused);
    assert_eq!(subnet_data.start_epoch, epoch);

    increase_epochs(10);

    let curr_epoch = Network::get_current_epoch_as_u32();
    let delta = curr_epoch - epoch;

    assert_ok!(Network::owner_unpause_subnet(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
    ));

    // Ensure was activated
    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet_data.state, SubnetState::Active);
    assert_eq!(subnet_data.start_epoch, curr_epoch + 1);

    let node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    // The start epoch update increases the epoch by 1
    assert_eq!(node.classification.start_epoch, start_epoch + delta + 1);
  });
}

#[test]
fn test_owner_deactivate_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    // Transfer to new owner
    assert_ok!(Network::owner_deactivate_subnet(
      RuntimeOrigin::signed(original_owner),
      subnet_id,
    ));

    assert_eq!(
			*network_events().last().unwrap(),
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::Owner
      }
		);

    assert_eq!(SubnetsData::<Test>::try_get(subnet_id), Err(()));
  });
}

#[test]
fn test_transfer_and_accept_ownership_works() {
  new_test_ext().execute_with(|| {
    let subnet_id = 0;
    let original_owner = account(1);
    let new_owner = account(2);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    // Transfer to new owner
    assert_ok!(Network::do_transfer_subnet_ownership(
      RuntimeOrigin::signed(original_owner),
      subnet_id,
      new_owner.clone()
    ));

    // Accept by new owner
    assert_ok!(Network::do_accept_subnet_ownership(
      RuntimeOrigin::signed(new_owner.clone()),
      subnet_id
    ));

    // Check ownership
    assert_eq!(SubnetOwner::<Test>::get(subnet_id), Some(new_owner.clone()));
  });
}

#[test]
fn test_transfer_cannot_be_accepted_by_wrong_account() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let original_owner = account(3);
    let new_owner = account(4);
    let wrong_account = account(5);

    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    assert_ok!(Network::do_transfer_subnet_ownership(
      RuntimeOrigin::signed(original_owner),
      subnet_id,
      new_owner
    ));

    assert_noop!(
      Network::do_accept_subnet_ownership(
        RuntimeOrigin::signed(wrong_account),
        subnet_id
      ),
      Error::<Test>::NotPendingSubnetOwner
    );
  });
}

#[test]
fn test_owner_can_cancel_transfer_by_resetting_owner() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let original_owner = account(6);
    let new_owner = account(7);
    let zero_address = <Test as frame_system::Config>::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap();

    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    assert_ok!(Network::do_transfer_subnet_ownership(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_owner.clone()
    ));

    assert_ok!(Network::do_transfer_subnet_ownership(
      RuntimeOrigin::signed(original_owner),
      subnet_id,
      zero_address
    ));

    assert_noop!(
      Network::do_accept_subnet_ownership(
        RuntimeOrigin::signed(new_owner.clone()),
        subnet_id
      ),
      Error::<Test>::NotPendingSubnetOwner
    );
  });
}

#[test]
fn test_accept_without_pending_transfer_should_fail() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let user = account(8);

    assert_noop!(
      Network::do_accept_subnet_ownership(
        RuntimeOrigin::signed(user),
        subnet_id
      ),
      Error::<Test>::NoPendingSubnetOwner
    );
  });
}

#[test]
fn test_non_owner_cannot_transfer() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let actual_owner = account(9);
    let fake_owner = account(10);
    let target = account(11);

    SubnetOwner::<Test>::insert(subnet_id, &actual_owner);

    assert_noop!(
      Network::do_transfer_subnet_ownership(
        RuntimeOrigin::signed(fake_owner),
        subnet_id,
        target
      ),
      Error::<Test>::NotSubnetOwner
    );
  });
}
