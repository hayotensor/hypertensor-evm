use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use log::info;
use frame_support::traits::Currency;
use crate::{
  Error,
  SubnetsData,
  OverwatchCommits,
  OverwatchReveals,
  SubnetState,
  SubnetData,
  OverwatchNode,
  OverwatchNodes,
  OverwatchNodeIdHotkey,
  OverwatchCommit,
  OverwatchReveal,
  TotalOverwatchNodeUids,
  HotkeyOwner,
  HotkeyOverwatchNodeId,
  NetworkMinStakeBalance,
  MinSubnetNodes,
  SubnetName,
};
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use sp_runtime::traits::Hash;

//
//
//
//
//
//
//
// Overwatch Commit-Reveal
//
//
//
//
//
//
//

fn make_commit(weight: u128, salt: Vec<u8>) -> sp_core::H256 {
  Hashing::hash_of(&(weight, salt))
}

#[test]
fn test_do_commit_and_reveal_weights_success() {
  new_test_ext().execute_with(|| {
    let coldkey: AccountId = account(1);
    let hotkey: AccountId = account(2);
    let overwatch_node_id = 1;
    let subnet_id = 99;
    let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

    // Setup: assign ownership and create subnet
    let subnet_data = SubnetData {
      id: 1,
      name: "subnet_name".into(),
      repo: "github".into(),
      description: "description".into(),
      misc: "misc".into(),
      state: SubnetState::Active,
      start_epoch: 0,
    };

    SubnetsData::<Test>::insert(subnet_id, subnet_data);

    TotalOverwatchNodeUids::<Test>::mutate(|n: &mut u32| *n += 1);
    let current_uid = TotalOverwatchNodeUids::<Test>::get();

    let overwatch_node = OverwatchNode {
      id: current_uid,
      hotkey: hotkey.clone(),
    };

    OverwatchNodes::<Test>::insert(overwatch_node_id, overwatch_node);
    HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
    OverwatchNodeIdHotkey::<Test>::insert(current_uid, hotkey.clone());

    // Weight + salt
    let weight: u128 = 123456;
    let salt: Vec<u8> = b"secret-salt".to_vec();
    let commit_hash = make_commit(weight, salt.clone());

    // Commit
    assert_ok!(Network::perform_commit_overwatch_subnet_weights(
      overwatch_node_id,
      vec![OverwatchCommit {
        subnet_id,
        weight: commit_hash
      }]
    ));

    // Ensure it's stored
    let stored = OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id)).unwrap();
    assert_eq!(stored, commit_hash);

    // Reveal
    assert_ok!(Network::perform_reveal_overwatch_subnet_weights(
      overwatch_node_id,
      vec![OverwatchReveal {
        subnet_id,
        weight,
        salt
      }]
    ));

    // Ensure revealed weight is correct
    let revealed = OverwatchReveals::<Test>::get((overwatch_epoch, subnet_id, overwatch_node_id)).unwrap();
    assert_eq!(revealed, weight);
  });
}

#[test]
fn test_commit_and_reveal_extrinsics() {
  new_test_ext().execute_with(|| {
    
    // subnet
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let min_subnet_nodes = MinSubnetNodes::<Test>::get();
    let end = min_subnet_nodes;
    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let coldkey: AccountId = account(1);
    let hotkey: AccountId = account(2);

    let amount = 100000000000000000000;

    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);
    make_overwatch_qualified(1);

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()),
        hotkey.clone(),
        amount,
      )
    );

    let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

    let subnet_id = 99;
    let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

    // Setup: assign ownership and create subnet
    let subnet_data = SubnetData {
      id: 1,
      name: "subnet_name".into(),
      repo: "github".into(),
      description: "description".into(),
      misc: "misc".into(),
      state: SubnetState::Active,
      start_epoch: 0,
    };

    SubnetsData::<Test>::insert(subnet_id, subnet_data);

    // Weight + salt
    let weight: u128 = 123456;
    let salt: Vec<u8> = b"secret-salt".to_vec();
    let commit_hash = make_commit(weight, salt.clone());

    // Commit
    assert_ok!(Network::commit_overwatch_subnet_weights(
      RuntimeOrigin::signed(hotkey.clone()),
      overwatch_node_id,
      vec![OverwatchCommit {
        subnet_id,
        weight: commit_hash
      }]
    ));

    // Ensure it's stored
    let stored = OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id)).unwrap();
    assert_eq!(stored, commit_hash);

    set_block_to_overwatch_reveal_block(overwatch_epoch);

    // Reveal
    assert_ok!(Network::reveal_overwatch_subnet_weights(
      RuntimeOrigin::signed(hotkey.clone()),
      overwatch_node_id,
      vec![OverwatchReveal {
        subnet_id,
        weight,
        salt
      }]
    ));

    // Ensure revealed weight is correct
    let revealed = OverwatchReveals::<Test>::get((overwatch_epoch, subnet_id, overwatch_node_id)).unwrap();
    assert_eq!(revealed, weight);
  });
}


#[test]
fn test_commit_and_reveal_phase_errors() {
  new_test_ext().execute_with(|| {
    // subnet
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let min_subnet_nodes = MinSubnetNodes::<Test>::get();
    let end = min_subnet_nodes;
    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let coldkey: AccountId = account(1);
    let hotkey: AccountId = account(2);

    let amount = 100000000000000000000;

    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);
    make_overwatch_qualified(1);

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()),
        hotkey.clone(),
        amount,
      )
    );

    let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

    let subnet_id = 99;
    let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

    // Weight + salt
    let weight: u128 = 123456;
    let salt: Vec<u8> = b"secret-salt".to_vec();
    let commit_hash = make_commit(weight, salt.clone());


    // Reveal
    assert_err!(
      Network::reveal_overwatch_subnet_weights(
        RuntimeOrigin::signed(hotkey.clone()),
        overwatch_node_id,
        vec![OverwatchReveal {
          subnet_id,
          weight,
          salt
        }]
      ),
      Error::<Test>::NotRevealPeriod
    );

    set_block_to_overwatch_reveal_block(overwatch_epoch);

    // Commit fail
    assert_err!(
      Network::commit_overwatch_subnet_weights(
        RuntimeOrigin::signed(hotkey.clone()),
        overwatch_node_id,
        vec![OverwatchCommit {
          subnet_id,
          weight: commit_hash
        }]
      ), 
      Error::<Test>::NotCommitPeriod
    );
  });
}
