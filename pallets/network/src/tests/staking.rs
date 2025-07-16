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
  NetworkMinStakeBalance,
  TotalActiveSubnets,
  MaxSubnetNodes,
  DeactivatedSubnetNodesData,
  MinActiveNodeStakeEpochs,
  SubnetNodesData,
  RegisteredSubnetNodesData,
  TotalSubnetNodeUids,
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
      Error::<Test>::InvalidSubnet,
    );

    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
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
      Error::<Test>::InvalidSubnet,
    );

    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(account_n), deposit_amount);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
        account(account_n),
        amount,
      ) 
    );

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n)),
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
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(account_n), deposit_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
        account(account_n),
        amount,
      ) 
    );

    assert_eq!(Network::account_subnet_stake(account(account_n), subnet_id), amount + amount);
    // assert_eq!(Network::total_account_stake(account(account_n)), amount + amount);
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

// //     let subnet_name: Vec<u8> = "subnet-name".into();

// //     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

// //     build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

// //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
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
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let _ = Balances::deposit_creating(&account(account_n), deposit_amount);

    let subnet_name: Vec<u8> = "subnet-name".into();

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    // add double amount to stake
    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
        account(account_n),
        amount,
      ) 
    );

    assert_eq!(Network::account_subnet_stake(account(account_n), subnet_id), amount + amount);

    // remove amount ontop
    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        amount,
      )
    );

    assert_eq!(Network::account_subnet_stake(account(account_n), subnet_id), amount);
    // assert_eq!(Network::total_account_stake(account(account_n)), amount);
  });
}

#[test]
fn test_remove_stake_min_active_node_stake_epochs() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let end = 11;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_name: Vec<u8> = "subnet-name".into();

    let coldkey = account(subnet_id*total_subnet_nodes+1);
    let hotkey = account(max_subnet_nodes+end*subnets+2);

    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer(subnet_id*total_subnet_nodes+1),
        peer(subnet_id*total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let total_subnet_node_uids = TotalSubnetNodeUids::<Test>::get(subnet_id);
    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let start_epoch = subnet_node.classification.start_epoch;

    // set_epoch(start_epoch);
    set_block_to_subnet_slot(start_epoch, subnet_id);

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(hotkey.clone()),
        subnet_id,
        hotkey_subnet_node_id
      ),
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    // add double amount to stake
    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
        hotkey.clone(),
        amount,
      ) 
    );

    assert_eq!(Network::account_subnet_stake(hotkey.clone(), subnet_id), amount + amount);

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        amount,
      ),
      Error::<Test>::MinActiveNodeStakeEpochs
    );

    let min_stake_epochs = MinActiveNodeStakeEpochs::<Test>::get();
    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let start_epoch = subnet_node.classification.start_epoch;

    set_epoch(start_epoch + min_stake_epochs + 2); // increase by 2 to account for subnet epoch crossover

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        amount,
      )
    );

    // assert_eq!(Network::account_subnet_stake(hotkey.clone(), subnet_id), amount);
  });
}

// #[test]
// fn test_remove_stake_after_remove_subnet_node() {
//   new_test_ext().execute_with(|| {
//     let subnet_name: Vec<u8> = "subnet-name".into();
//     let deposit_amount: u128 = 1000000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
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
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    let epoch = get_epoch();

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
      )
    );

    let subnet_node = DeactivatedSubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

    let min_stake_epochs = MinActiveNodeStakeEpochs::<Test>::get();
    increase_epochs(min_stake_epochs + 2);

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        stake_amount,
      ),
      Error::<Test>::MinStakeNotReached
    );
  })
}

#[test]
fn test_register_try_removing_all_stake() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
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