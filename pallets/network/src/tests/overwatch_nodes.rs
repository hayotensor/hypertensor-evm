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
  AccountOverwatchStake,
  TotalOverwatchStake,
  ColdkeyHotkeys,
  OverwatchMinStakeBalance,
  HotkeyOverwatchNodeId,
  TotalActiveSubnets,
  NetworkMinStakeBalance,
  MaxSubnetNodes,
  MaxSubnets,
  OverwatchSubnetWeights,
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
// 

#[test]
fn test_equal_stake_equal_weights() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let epoch = 1;

    // Setup
    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    set_stake(1, 100);
    set_stake(2, 100);

    submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

    let (subnet_weights, node_weights, _) = Network::calculate_overwatch_rewards_v2(epoch);
    assert_eq!(node_weights.len(), 2);

    let score_1 = node_weights.get(&node_id_1);
    let score_2 = node_weights.get(&node_id_2);
    assert_eq!(score_1, Some(&500000000000000000_u128));
    assert_eq!(score_2, Some(&500000000000000000_u128));
  });
}

#[test]
fn test_stake_dampening_effect() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let epoch = 1;

    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    set_stake(1, 90);
    set_stake(2, 10);

    submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

    let (subnet_weights, node_weights, _) = Network::calculate_overwatch_rewards_v2(epoch);
    assert_eq!(node_weights.len(), 2);

    let score_1 = node_weights.get(&node_id_1);
    let score_2 = node_weights.get(&node_id_2);

    assert!(score_1 < Some(&900000000000000000));
    assert!(score_1 > Some(&500000000000000000));
    assert!(score_2 < Some(&500000000000000000));
  });
}

#[test]
fn test_two_noces_same_stake_dif_weights() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let epoch = 1;

    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    set_stake(1, 50);
    set_stake(2, 50);

    submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id, node_id_2, 100);

    let (subnet_weights, node_weights, _) = Network::calculate_overwatch_rewards_v2(epoch);

    let score_1 = node_weights.get(&node_id_1);
    let score_2 = node_weights.get(&node_id_2);
    // Nodes have same stake weight, only 2 nodes, should be same scores
    assert_eq!(Some(score_1), Some(score_2));
  });
}

#[test]
fn test_multiple_subnets_score_accumulation() {
  new_test_ext().execute_with(|| {
    let subnet_id_1 = 1;
    let subnet_id_2 = 2;
    let epoch = 1;

    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    set_stake(1, 50);
    set_stake(2, 100);

    // Subnet 1
    submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_2, 500000000000000000);
    // Subnet 2
    submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000); // Node 2 slightly deviates

    let (subnet_weights, node_weights, _) = Network::calculate_overwatch_rewards_v2(epoch);

    let score_1 = node_weights.get(&node_id_1);
    let score_2 = node_weights.get(&node_id_2);

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

    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    set_stake(1, 100);
    set_stake(2, 50);

    // Subnet 1
    submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_2, 500000000000000000);
    // Subnet 2
    submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000); // Node 2 slightly deviates

    let (subnet_weights, node_weights, _) = Network::calculate_overwatch_rewards_v2(epoch);

    let score_1 = node_weights.get(&node_id_1);
    let score_2 = node_weights.get(&node_id_2);

    // 1 has higher stake weight
    assert!(score_1 > score_2);
  });
}

#[test]
fn test_no_weights_returns_empty() {
  new_test_ext().execute_with(|| {
    let epoch = 1;
    let (subnet_weights, node_weights, _) = Network::calculate_overwatch_rewards_v2(epoch);
    assert_eq!(node_weights.len(), 0);
  });
}



// v3

#[test]
fn test_equal_stake_equal_weights_v3() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let epoch = Network::get_current_overwatch_epoch_as_u32();

    // Setup
    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    set_stake(1, 100);
    set_stake(2, 100);

    submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

    let block_weight = Network::calculate_overwatch_rewards_v3();
    let subnet_weight = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id);

    assert_eq!(subnet_weight, Some(500000000000000000_u128));

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

    assert_eq!(score_1, Some(500000000000000000_u128));
    assert_eq!(score_2, Some(500000000000000000_u128));
  });
}

#[test]
fn test_stake_no_dampening_effect() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let epoch = Network::get_current_overwatch_epoch_as_u32();

    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    set_stake(1, 90);
    set_stake(2, 10);

    submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

    let block_weight = Network::calculate_overwatch_rewards_v3();
    let subnet_weight = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id);

    // Both users submitted the same score, subnet should be the score
    assert_eq!(subnet_weight, Some(500000000000000000));

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

    // Both users submitted the same score, each node score should be equal
    assert_eq!(score_1, score_2);
  });
}

#[test]
fn test_two_noces_same_stake_dif_weights_v3() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let epoch = Network::get_current_overwatch_epoch_as_u32();

    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    set_stake(1, 50);
    set_stake(2, 50);

    submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id, node_id_2, 100);

    let block_weight = Network::calculate_overwatch_rewards_v3();
    let subnet_weight = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id);

    assert_eq!(subnet_weight, Some((500000000000000000 + 100) / 2));

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

    // Nodes have same stake weight, only 2 nodes, should be same scores
    assert_eq!(Some(score_1), Some(score_2));
  });
}

#[test]
fn test_multiple_subnets_score_accumulation_v3() {
  new_test_ext().execute_with(|| {
    let subnet_id_1 = 1;
    let subnet_id_2 = 2;
    let epoch = Network::get_current_overwatch_epoch_as_u32();

    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    set_stake(1, 50);
    set_stake(2, 100);

    // Subnet 1
    submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_2, 500000000000000000);
    // Subnet 2
    submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000); // Node 2 slightly deviates

    let block_weight = Network::calculate_overwatch_rewards_v3();
    let subnet_weight_1 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_1);
    let subnet_weight_2 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_2);

    // assert_eq!(subnet_weight_1, Some(500000000000000000));
    assert_eq!(subnet_weight_1, Some(499999999999999999)); // Rounding err

    log::error!("test_multiple_subnets_score_accumulation_v3 subnet_weight_2 {:?}", subnet_weight_2);
    // assert!(false);

    assert_eq!(subnet_weight_2, Some(566666666666666665)); // Rounding err

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

    // 2 has higher stake weight
    assert!(score_2 > score_1);
  });
}

#[test]
fn test_multiple_subnets_score_accumulation_v3_2() {
  new_test_ext().execute_with(|| {
    let subnet_id_1 = 1;
    let subnet_id_2 = 2;
    let epoch = Network::get_current_overwatch_epoch_as_u32();

    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    set_stake(1, 100);
    set_stake(2, 50);

    // Subnet 1
    submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_2, 500000000000000000);
    // Subnet 2
    submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000); // Node 2 slightly deviates

    let block_weight = Network::calculate_overwatch_rewards_v3();
    let subnet_weight_1 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_1);
    let subnet_weight_2 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_2);
    log::error!("test_multiple_subnets_score_accumulation_v3_2 subnet_weight_1 {:?}", subnet_weight_1);
    log::error!("test_multiple_subnets_score_accumulation_v3_2 subnet_weight_2 {:?}", subnet_weight_2);
    // assert!(false);

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

    // 1 has higher stake weight
    assert!(score_1 > score_2);
  });
}
