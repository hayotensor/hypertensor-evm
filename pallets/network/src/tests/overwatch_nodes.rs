use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use log::info;
use crate::{
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
  AccountOverwatchStake,
  TotalOverwatchStake,
  OverwatchNodeWeights,
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
// Overwatch Nodes
//
//
//
//
//
//
//
fn insert_overwatch_node(account_id: u32) -> u32 {
  let coldkey = account(account_id);
  let hotkey = account(account_id);

  TotalOverwatchNodeUids::<Test>::mutate(|n: &mut u32| *n += 1);
  let current_uid = TotalOverwatchNodeUids::<Test>::get();

  let overwatch_node = OverwatchNode {
    id: current_uid,
    hotkey: hotkey.clone(),
  };

  OverwatchNodes::<Test>::insert(current_uid, overwatch_node);
  HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
  OverwatchNodeIdHotkey::<Test>::insert(current_uid, hotkey.clone());

  current_uid
}

fn set_stake(account_id: u32, amount: u128) {
    // -- increase account staking balance
  AccountOverwatchStake::<Test>::mutate(account(account_id), |mut n| *n += amount);
  // -- increase total stake
  TotalOverwatchStake::<Test>::mutate(|mut n| *n += amount);
}

fn submit_weight(
  epoch: u32,
  subnet_id: u32,
  node_id: u32,
  weight: u128
) {
  OverwatchReveals::<Test>::insert((epoch, subnet_id, node_id), weight);
}

#[test]
fn test_equal_stake_equal_weights() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let epoch = 1;

    // Setup
    let node_id_1 = insert_overwatch_node(1);
    let node_id_2 = insert_overwatch_node(2);
    set_stake(1, 100);
    set_stake(2, 100);

    submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

    Network::calculate_overwatch_rewards(epoch);

    let scores = OverwatchNodeWeights::<Test>::iter_prefix(epoch);
    assert_eq!(scores.count(), 2);

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);
    assert_eq!(score_1, Some(500000000000000000));
    assert_eq!(score_2, Some(500000000000000000));
    // assert_eq!(scores[&1], 500000000000000000);
    // assert_eq!(scores[&2], 500000000000000000);

  });
}

#[test]
fn test_stake_dampening_effect() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let epoch = 1;

    let node_id_1 = insert_overwatch_node(1);
    let node_id_2 = insert_overwatch_node(2);
    set_stake(1, 90);
    set_stake(2, 10);

    submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

    Network::calculate_overwatch_rewards(epoch);

    let scores = OverwatchNodeWeights::<Test>::iter_prefix(epoch);
    assert_eq!(scores.count(), 2);

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

    assert!(score_1 < Some(900000000000000000));
    assert!(score_1 > Some(500000000000000000));
    assert!(score_2 < Some(500000000000000000));
  });
}

#[test]
fn test_two_noces_same_stake_dif_weights() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let epoch = 1;

    let node_id_1 = insert_overwatch_node(1);
    let node_id_2 = insert_overwatch_node(2);
    set_stake(1, 50);
    set_stake(2, 50);

    submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id, node_id_2, 100);

    Network::calculate_overwatch_rewards(epoch);

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);
    // Nodes have same stake weight, only 2 nodes, should be same scores
    assert_eq!(Some(score_1), Some(score_2));
  });
}

#[test]
fn test_missing_stake_gets_zero_score() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let epoch = 1;

    // Only node 1 has registered stake
    let node_id_1 = insert_overwatch_node(1);
    let node_id_2 = insert_overwatch_node(2);
    set_stake(1, 100);

    submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);

    // Node leaves after?
    submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

    Network::calculate_overwatch_rewards(epoch);

    assert!(OverwatchNodeWeights::<Test>::get(epoch, node_id_1).is_some());
    // No stake = not scored
    assert_eq!(OverwatchNodeWeights::<Test>::try_get(epoch, node_id_2), Err(()));
  });
}

#[test]
fn test_multiple_subnets_score_accumulation() {
  new_test_ext().execute_with(|| {
    let subnet_id_1 = 1;
    let subnet_id_2 = 2;
    let epoch = 1;

    let node_id_1 = insert_overwatch_node(1);
    let node_id_2 = insert_overwatch_node(2);
    set_stake(1, 50);
    set_stake(2, 100);

    // Subnet 1
    submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_2, 500000000000000000);
    // Subnet 2
    submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000); // Node 2 slightly deviates

    let scores = Network::calculate_overwatch_rewards(epoch);

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

    // 2 has higher stake weight
    assert!(score_2 > score_1);
  });
}

#[test]
fn test_multiple_subnets_score_accumulation_v2() {
  new_test_ext().execute_with(|| {
    let subnet_id_1 = 1;
    let subnet_id_2 = 2;
    let epoch = 1;

    let node_id_1 = insert_overwatch_node(1);
    let node_id_2 = insert_overwatch_node(2);
    set_stake(1, 100);
    set_stake(2, 50);

    // Subnet 1
    submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_2, 500000000000000000);
    // Subnet 2
    submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000); // Node 2 slightly deviates

    Network::calculate_overwatch_rewards(epoch);

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

    // 1 has higher stake weight
    assert!(score_1 > score_2);
  });
}

#[test]
fn test_no_weights_returns_empty() {
  new_test_ext().execute_with(|| {
    let epoch = 1;
    Network::calculate_overwatch_rewards(epoch);
    let scores = OverwatchNodeWeights::<Test>::iter_prefix(epoch);
    assert_eq!(scores.count(), 0);
  });
}
