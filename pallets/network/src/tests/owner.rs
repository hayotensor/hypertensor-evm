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
  TotalSubnetNodes,
  SubnetOwner,
};
use sp_runtime::traits::TrailingZeroInput;
use codec::{Decode, Encode};

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

// #[test]
// fn test_register_subnet() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "subnet-name".into();
    
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let stake_amount: u128 = MinStakeBalance::<Test>::get();

//     build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     let n_account = total_subnet_nodes + 1;

//   })
// }

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
