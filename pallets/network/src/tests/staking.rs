use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use frame_support::BoundedVec;
use sp_core::OpaquePeerId as PeerId;
use crate::{
  Error,
  SubnetName, 
  TotalSubnetNodes,
  TotalSubnetStake,
  HotkeySubnetNodeId,
  MinStakeBalance,
};

// ///
// ///
// ///
// ///
// ///
// ///
// ///
// /// Staking
// ///
// ///
// ///
// ///
// ///
// ///
// ///

#[test]
fn test_add_to_stake_not_key_owner() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(1), deposit_amount);

    assert_err!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(1)),
        0,
        0,
        account(1),
        amount,
      ),
      Error::<Test>::SubnetNotExist,
    );

    let subnet_path: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(1), deposit_amount);

    assert_err!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        0,
        account(total_subnet_nodes+1),
        amount,
      ),
      Error::<Test>::NotKeyOwner,
    );

  });
}

#[test]
fn test_remove_stake_not_key_owner() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(1), deposit_amount);

    assert_err!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(1)),
        0,
        0,
        account(1),
        amount,
      ),
      Error::<Test>::SubnetNotExist,
    );

    let subnet_path: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(1), deposit_amount);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        account(1),
        amount,
      ) 
    );

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(2),
        amount,
      ),
      Error::<Test>::NotKeyOwner,
    );

  });
}

#[test]
fn test_add_to_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(1), deposit_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        account(1),
        amount,
      ) 
    );

    assert_eq!(Network::account_subnet_stake(account(1), subnet_id), amount + amount);
    // assert_eq!(Network::total_account_stake(account(1)), amount + amount);
    assert_eq!(Network::total_stake(), amount_staked + amount);
    assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked + amount);
  });
}

// // #[test]
// // fn test_remove_stake_err() {
// //   new_test_ext().execute_with(|| {
// //     let deposit_amount: u128 = 1000000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     // Not a node so should have no stake to remove
// //     assert_err!(
// //       Network::remove_stake(
// //         RuntimeOrigin::signed(account(255)),
// //         account(255),
// //         0,
// //         amount,
// //       ),
// //       Error::<Test>::NotEnoughStakeToWithdraw,
// //     );

// //     let subnet_path: Vec<u8> = "subnet-name".into();

// //     let stake_amount: u128 = MinStakeBalance::<Test>::get();

// //     build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

// //     let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
// //     let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

// //     assert_err!(
// //       Network::remove_stake(
// //         RuntimeOrigin::signed(account(255)),
// //         account(255),
// //         subnet_id,
// //         amount,
// //       ),
// //       Error::<Test>::NotEnoughStakeToWithdraw,
// //     );

// //     assert_err!(
// //       Network::remove_stake(
// //         RuntimeOrigin::signed(account(1)),
// //         account(1),
// //         subnet_id,
// //         0,
// //       ),
// //       Error::<Test>::NotEnoughStakeToWithdraw,
// //     );
// //   });
// // }

#[test]
fn test_remove_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let _ = Balances::deposit_creating(&account(1), deposit_amount);

    let subnet_path: Vec<u8> = "subnet-name".into();

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    // add double amount to stake
    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        account(1),
        amount,
      ) 
    );

    assert_eq!(Network::account_subnet_stake(account(1), subnet_id), amount + amount);

    // remove amount ontop
    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        amount,
      )
    );

    assert_eq!(Network::account_subnet_stake(account(1), subnet_id), amount);
    // assert_eq!(Network::total_account_stake(account(1)), amount);
  });
}

// #[test]
// fn test_remove_stake_after_remove_subnet_node() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "subnet-name".into();
//     let deposit_amount: u128 = 1000000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     let _ = Balances::deposit_creating(&account(1), deposit_amount);

//     assert_ok!(
//       Network::remove_subnet_node(
//         RuntimeOrigin::signed(account(1)),
//         subnet_id,
//       )
//     );

//     let epoch_length = EpochLength::get();
//     let min_required_unstake_epochs = StakeCooldownEpochs::get();
//     System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);

//     // remove amount ontop
//     assert_ok!(
//       Network::remove_stake(
//         RuntimeOrigin::signed(account(1)),
//         account(1),
//         subnet_id,
//         amount,
//       )
//     );

//     assert_eq!(Network::account_subnet_stake(account(1), 1), 0);
//     assert_eq!(Network::total_account_stake(account(1)), 0);
//     assert_eq!(Network::total_stake(), 0);
//     assert_eq!(Network::total_subnet_stake(1), 0);
//   });
// }


#[test]
fn test_deactivate_try_removing_all_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    let epoch = get_epoch();

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
      )
    );

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        stake_amount,
      ),
      Error::<Test>::MinStakeNotReached
    );
  })
}

#[test]
fn test_register_try_removing_all_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);
    let starting_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        stake_amount,
      ),
      Error::<Test>::MinStakeNotReached
    );
  })
}