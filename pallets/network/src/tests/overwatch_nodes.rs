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
  OverwatchNodeWeights,
  ColdkeyHotkeys,
  OverwatchMinStakeBalance,
  HotkeyOverwatchNodeId,
  TotalActiveSubnets,
  NetworkMinStakeBalance,
  MaxSubnetNodes,
  MaxSubnets,
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

// #[test]
// fn test_register_overwatch_node() {
//   new_test_ext().execute_with(|| {
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let stake_amount: u128 = OverwatchMinStakeBalance::<Test>::get();

//     let coldkey = account(0);
//     let hotkey = account(1);

//     let _ = Balances::deposit_creating(&coldkey, deposit_amount);

//     assert_ok!(
//       Network::register_overwatch_node(
//         RuntimeOrigin::signed(coldkey.clone()),
//         hotkey.clone(),
//         stake_amount,
//       )
//     );

//     let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey);
//     assert!(hotkeys.contains(&hotkey));

//     let owner = HotkeyOwner::<Test>::get(&hotkey.clone());
//     assert_eq!(owner, coldkey.clone());

//     let node_id = HotkeyOverwatchNodeId::<Test>::get(&hotkey.clone()).unwrap();
//     assert_eq!(node_id, 1);

//     let ow_hotkey = OverwatchNodeIdHotkey::<Test>::get(node_id);
//     assert_eq!(ow_hotkey, Some(hotkey.clone()));

//     let ow_node = OverwatchNodes::<Test>::get(node_id);
//     assert_eq!(ow_node.unwrap().hotkey, hotkey.clone());
//   });
// }

// #[test]
// fn test_overwatch_subnet_node_unique_hotkeys() {
//   new_test_ext().execute_with(|| {
//     let subnet_name: Vec<u8> = "subnet-name".into();
    
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let end = 16;

//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    
//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//     let max_subnets = MaxSubnets::<Test>::get();

//     // subnet node keys, same keys as the last node
//     let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//     let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//     build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//     let _ = Balances::deposit_creating(&coldkey, deposit_amount);

//     assert_err!(
//       Network::register_overwatch_node(
//         RuntimeOrigin::signed(coldkey.clone()),
//         hotkey.clone(),
//         stake_amount,
//       ),
//       Error::<Test>::HotkeyHasOwner
//     );

//     let free_hotkey = account(max_subnet_nodes+end*subnets+1);

//     assert_ok!(
//       Network::register_overwatch_node(
//         RuntimeOrigin::signed(coldkey.clone()),
//         free_hotkey.clone(),
//         stake_amount,
//       )
//     );
//   });
// }

// #[test]
// fn test_equal_stake_equal_weights() {
//   new_test_ext().execute_with(|| {
//     let subnet_id = 1;
//     let epoch = 1;

//     // Setup
//     let node_id_1 = insert_overwatch_node(1,1);
//     let node_id_2 = insert_overwatch_node(2,2);
//     set_stake(1, 100);
//     set_stake(2, 100);

//     submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
//     submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

//     Network::calculate_overwatch_rewards(epoch);

//     let scores = OverwatchNodeWeights::<Test>::iter_prefix(epoch);
//     assert_eq!(scores.count(), 2);

//     let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
//     let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);
//     assert_eq!(score_1, Some(500000000000000000));
//     assert_eq!(score_2, Some(500000000000000000));
//   });
// }

// #[test]
// fn test_stake_dampening_effect() {
//   new_test_ext().execute_with(|| {
//     let subnet_id = 1;
//     let epoch = 1;

//     let node_id_1 = insert_overwatch_node(1,1);
//     let node_id_2 = insert_overwatch_node(2,2);
//     set_stake(1, 90);
//     set_stake(2, 10);

//     submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
//     submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

//     Network::calculate_overwatch_rewards(epoch);

//     let scores = OverwatchNodeWeights::<Test>::iter_prefix(epoch);
//     assert_eq!(scores.count(), 2);

//     let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
//     let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

//     assert!(score_1 < Some(900000000000000000));
//     assert!(score_1 > Some(500000000000000000));
//     assert!(score_2 < Some(500000000000000000));
//   });
// }

// #[test]
// fn test_two_noces_same_stake_dif_weights() {
//   new_test_ext().execute_with(|| {
//     let subnet_id = 1;
//     let epoch = 1;

//     let node_id_1 = insert_overwatch_node(1,1);
//     let node_id_2 = insert_overwatch_node(2,2);
//     set_stake(1, 50);
//     set_stake(2, 50);

//     submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
//     submit_weight(epoch, subnet_id, node_id_2, 100);

//     Network::calculate_overwatch_rewards(epoch);

//     let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
//     let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);
//     // Nodes have same stake weight, only 2 nodes, should be same scores
//     assert_eq!(Some(score_1), Some(score_2));
//   });
// }

// #[test]
// fn test_multiple_subnets_score_accumulation() {
//   new_test_ext().execute_with(|| {
//     let subnet_id_1 = 1;
//     let subnet_id_2 = 2;
//     let epoch = 1;

//     let node_id_1 = insert_overwatch_node(1,1);
//     let node_id_2 = insert_overwatch_node(2,2);
//     set_stake(1, 50);
//     set_stake(2, 100);

//     // Subnet 1
//     submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
//     submit_weight(epoch, subnet_id_1, node_id_2, 500000000000000000);
//     // Subnet 2
//     submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
//     submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000); // Node 2 slightly deviates

//     let scores = Network::calculate_overwatch_rewards(epoch);

//     let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
//     let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

//     // 2 has higher stake weight
//     assert!(score_2 > score_1);
//   });
// }

// #[test]
// fn test_multiple_subnets_score_accumulation_v2() {
//   new_test_ext().execute_with(|| {
//     let subnet_id_1 = 1;
//     let subnet_id_2 = 2;
//     let epoch = 1;

//     let node_id_1 = insert_overwatch_node(1,1);
//     let node_id_2 = insert_overwatch_node(2,2);
//     set_stake(1, 100);
//     set_stake(2, 50);

//     // Subnet 1
//     submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
//     submit_weight(epoch, subnet_id_1, node_id_2, 500000000000000000);
//     // Subnet 2
//     submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
//     submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000); // Node 2 slightly deviates

//     Network::calculate_overwatch_rewards(epoch);

//     let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
//     let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

//     // 1 has higher stake weight
//     assert!(score_1 > score_2);
//   });
// }

// #[test]
// fn test_no_weights_returns_empty() {
//   new_test_ext().execute_with(|| {
//     let epoch = 1;
//     Network::calculate_overwatch_rewards(epoch);
//     let scores = OverwatchNodeWeights::<Test>::iter_prefix(epoch);
//     assert_eq!(scores.count(), 0);
//   });
// }

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
