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
  OverwatchNodeIndex,
  TotalOverwatchNodes,
  SubnetName,
  MinSubnetNodes,
  PeerIdOverwatchNode,
  PeerIdSubnetNodeId,
  HotkeySubnetNodeId,
  SubnetNodesData,
  MaxOverwatchNodes,
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
fn test_register_overwatch_node() {
  new_test_ext().execute_with(|| {
    let amount = 100000000000000000000;

    let coldkey = account(1);
    let hotkey = account(2);
    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

    make_overwatch_qualified(1);

    let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
    let uids = TotalOverwatchNodeUids::<Test>::get();
    let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    assert!(!hotkeys.contains(&hotkey.clone()));

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      )
    );

    assert_eq!(init_total_overwatch_nodes + 1, TotalOverwatchNodes::<Test>::get());
    assert_eq!(uids + 1, TotalOverwatchNodeUids::<Test>::get());

    let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    assert!(hotkeys.contains(&hotkey.clone()));

    assert_eq!(HotkeyOwner::<Test>::get(hotkey.clone()), coldkey.clone());
    assert_eq!(uids + 1, HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap());

    assert_eq!(OverwatchNodes::<Test>::get(uids + 1).unwrap().hotkey, hotkey.clone());
    assert_eq!(OverwatchNodeIdHotkey::<Test>::get(uids + 1), Some(hotkey.clone()));
    assert_eq!(AccountOverwatchStake::<Test>::get(hotkey.clone()), amount);
  });
}

#[test]
fn test_register_overwatch_node_errors() {
  new_test_ext().execute_with(|| {
    let amount = 100000000000000000000;

    let coldkey = account(1);
    let hotkey = account(2);

    TotalOverwatchNodes::<Test>::set(MaxOverwatchNodes::<Test>::get());
    assert_err!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      ),
      Error::<Test>::MaxOverwatchNodes
    );

    TotalOverwatchNodes::<Test>::set(0);

    assert_err!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        coldkey.clone(),
        amount,
      ),
      Error::<Test>::ColdkeyMatchesHotkey
    );

    assert_err!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      ),
      Error::<Test>::ColdkeyNotOverwatchQualified
    );

    make_overwatch_qualified(1);

    let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    assert!(!hotkeys.contains(&hotkey.clone()));

    assert_err!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        0,
      ),
      Error::<Test>::MinStakeNotReached
    );

    assert_err!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      ),
      Error::<Test>::NotEnoughBalanceToStake
    );

    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000);
    assert_err!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      ),
      Error::<Test>::BalanceWithdrawalError
    );

    let _ = Balances::deposit_creating(&coldkey.clone(), 500);

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      )
    );

    let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    assert!(hotkeys.contains(&hotkey.clone()));

    assert_err!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      ),
      Error::<Test>::HotkeyHasOwner
    );

  });
}

#[test]
fn test_set_overwatch_peer_id() {
  new_test_ext().execute_with(|| {

    // subnet
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let min_subnet_nodes = MinSubnetNodes::<Test>::get();
    let end = min_subnet_nodes;
    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    // overwatch
    let amount = 100000000000000000000;

    let coldkey = account(1);
    let hotkey = account(2);
    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

    make_overwatch_qualified(1);

    let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
    let uids = TotalOverwatchNodeUids::<Test>::get();
    let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    assert!(!hotkeys.contains(&hotkey.clone()));

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      )
    );

    let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();
    let peer_id = peer(1);

    assert_ok!(
      Network::set_overwatch_node_peer_id(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        uid,
        peer_id.clone(),
      )
    );

    assert_eq!(PeerIdOverwatchNode::<Test>::get(subnet_id, peer_id.clone()), uid);
  });
}

#[test]
fn test_set_overwatch_peer_id_errors() {
  new_test_ext().execute_with(|| {
    // overwatch
    let amount = 100000000000000000000;

    let coldkey = account(1);
    let hotkey = account(2);
    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

    make_overwatch_qualified(1);

    let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
    let uids = TotalOverwatchNodeUids::<Test>::get();
    let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    assert!(!hotkeys.contains(&hotkey.clone()));

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      )
    );

    let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();
    let peer_id = peer(1);

    let subnet_id = 999;

    assert_err!(
      Network::set_overwatch_node_peer_id(
        RuntimeOrigin::signed(coldkey.clone()),
        999,
        uid,
        peer_id.clone(),
      ),
      Error::<Test>::InvalidSubnet
    );

    insert_subnet(subnet_id, SubnetState::Active, 0);

    assert_err!(
      Network::set_overwatch_node_peer_id(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        uid + 1,
        peer_id.clone(),
      ),
      Error::<Test>::NotKeyOwner
    );

    // add subnet to get existing peer ids
    // subnet
    let subnet_name: Vec<u8> = "subnet-name-999".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let min_subnet_nodes = MinSubnetNodes::<Test>::get();
    let end = min_subnet_nodes;
    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let snn_hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, end);

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, snn_hotkey.clone()).unwrap();
    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    let snn_peer_id = subnet_node_data.peer_id;

    assert_err!(
      Network::set_overwatch_node_peer_id(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        uid,
        snn_peer_id.clone(),
      ),
      Error::<Test>::PeerIdExist
    );
  });
}

#[test]
fn test_remove_overwatch_node() {
  new_test_ext().execute_with(|| {
    // subnet
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let min_subnet_nodes = MinSubnetNodes::<Test>::get();
    let end = min_subnet_nodes;
    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    // overwatch
    let amount = 100000000000000000000;

    let coldkey = account(1);
    let hotkey = account(2);
    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

    make_overwatch_qualified(1);

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      )
    );

    let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();
    let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();

    let peer_id = peer(1);

    assert_ok!(
      Network::set_overwatch_node_peer_id(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        uid,
        peer_id.clone(),
      )
    );

    assert_ok!(
      Network::remove_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap(),
      )
    );
    assert_eq!(OverwatchNodes::<Test>::try_get(uid), Err(()));
    assert_eq!(init_total_overwatch_nodes - 1, TotalOverwatchNodes::<Test>::get());
    assert_eq!(OverwatchNodeIdHotkey::<Test>::try_get(uid), Err(()));
    assert_eq!(HotkeyOverwatchNodeId::<Test>::try_get(hotkey.clone()), Err(()));
    assert_eq!(PeerIdOverwatchNode::<Test>::try_get(subnet_id, peer_id.clone()), Err(()));
    let map = OverwatchNodeIndex::<Test>::take(uid);
    for (subnet_id, map_peer_id) in map {
      assert_ne!(peer_id.clone(), map_peer_id);
    }
  });
}

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

    // Same scores submitted, same rewards
    assert_eq!(score_1, score_2);
    assert_eq!(score_1, Some(500000000000000000_u128));
    assert_eq!(score_2, Some(500000000000000000_u128));

    let mut score_sum = 0;
    for (id, _) in OverwatchNodes::<Test>::iter() {
      let weight = OverwatchNodeWeights::<Test>::get(epoch, id);
      score_sum += weight.unwrap();
    }

    assert_eq!(score_sum, 1000000000000000000);
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

    let mut score_sum = 0;
    for (id, _) in OverwatchNodes::<Test>::iter() {
      let weight = OverwatchNodeWeights::<Test>::get(epoch, id);
      score_sum += weight.unwrap();
    }

    assert_eq!(score_sum, 1000000000000000000);
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

    let mut score_sum = 0;
    for (id, _) in OverwatchNodes::<Test>::iter() {
      let weight = OverwatchNodeWeights::<Test>::get(epoch, id);
      score_sum += weight.unwrap();
    }

    assert_eq!(score_sum, 1000000000000000000);

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
    // assert!(false);
    assert_eq!(subnet_weight_2, Some(566666666666666665)); // Rounding err

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

    // 2 has higher stake weight
    assert!(score_2 > score_1);

    let mut score_sum = 0;
    for (id, _) in OverwatchNodes::<Test>::iter() {
      let weight = OverwatchNodeWeights::<Test>::get(epoch, id);
      score_sum += weight.unwrap();
    }

    assert!(score_sum <= 1000000000000000000 && score_sum >= 999999999999999990);
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

    let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
    let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

    // 1 has higher stake weight
    assert!(score_1 > score_2);

    let mut score_sum = 0;
    for (id, _) in OverwatchNodes::<Test>::iter() {
      let weight = OverwatchNodeWeights::<Test>::get(epoch, id);
      score_sum += weight.unwrap();
    }

    assert!(score_sum <= 1000000000000000000 && score_sum >= 999999999999999990);
  });
}


#[test]
fn test_multiple_subnets_check_percent_acccuracy() {
  new_test_ext().execute_with(|| {
    let subnet_id_1 = 1;
    let subnet_id_2 = 2;
    let subnet_id_3 = 3;
    let subnet_id_4 = 4;
    let subnet_id_5 = 5;
    let epoch = Network::get_current_overwatch_epoch_as_u32();

    // --- Generate a bunch of subnets, nodes, and entries and ensure ~1.0
    let node_id_1 = insert_overwatch_node(1,1);
    let node_id_2 = insert_overwatch_node(2,2);
    let node_id_3 = insert_overwatch_node(3,3);
    let node_id_4 = insert_overwatch_node(4,4);
    let node_id_5 = insert_overwatch_node(5,5);
    let node_id_6 = insert_overwatch_node(6,6);
    let node_id_7 = insert_overwatch_node(7,7);
    let node_id_8 = insert_overwatch_node(8,8);

    set_stake(1, 100);
    set_stake(2, 50);
    set_stake(3, 25);
    set_stake(4, 500);
    set_stake(5, 200);
    set_stake(6, 340);
    set_stake(7, 1);
    set_stake(8, 9);

    // Subnet 1
    submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_2, 400000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_3, 600000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_4, 500000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_5, 400000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_6, 600000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_7, 600000000000000000);
    submit_weight(epoch, subnet_id_1, node_id_8, 300000000000000000);
    // Subnet 2
    submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_3, 800000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_4, 900000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_5, 600000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_6, 800000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_7, 900000000000000000);
    submit_weight(epoch, subnet_id_2, node_id_8, 600000000000000000);
    // Subnet 3
    submit_weight(epoch, subnet_id_3, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_3, node_id_2, 600000000000000000);
    submit_weight(epoch, subnet_id_3, node_id_3, 800000000000000000);
    submit_weight(epoch, subnet_id_3, node_id_4, 900000000000000000);
    submit_weight(epoch, subnet_id_3, node_id_5, 600000000000000000);
    submit_weight(epoch, subnet_id_3, node_id_6, 800000000000000000);
    submit_weight(epoch, subnet_id_3, node_id_7, 900000000000000000);
    submit_weight(epoch, subnet_id_3, node_id_8, 600000000000000000);
    // Subnet 4
    submit_weight(epoch, subnet_id_4, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_4, node_id_2, 600000000000000000);
    submit_weight(epoch, subnet_id_4, node_id_3, 800000000000000000);
    submit_weight(epoch, subnet_id_4, node_id_4, 900000000000000000);
    submit_weight(epoch, subnet_id_4, node_id_5, 600000000000000000);
    submit_weight(epoch, subnet_id_4, node_id_6, 800000000000000000);
    submit_weight(epoch, subnet_id_4, node_id_7, 900000000000000000);
    submit_weight(epoch, subnet_id_4, node_id_8, 600000000000000000);
    // Subnet 5
    submit_weight(epoch, subnet_id_5, node_id_1, 500000000000000000);
    submit_weight(epoch, subnet_id_5, node_id_2, 600000000000000000);
    submit_weight(epoch, subnet_id_5, node_id_3, 800000000000000000);
    submit_weight(epoch, subnet_id_5, node_id_4, 900000000000000000);
    submit_weight(epoch, subnet_id_5, node_id_5, 600000000000000000);
    submit_weight(epoch, subnet_id_5, node_id_6, 800000000000000000);
    submit_weight(epoch, subnet_id_5, node_id_7, 900000000000000000);
    submit_weight(epoch, subnet_id_5, node_id_8, 600000000000000000);

    let _ = Network::calculate_overwatch_rewards_v3();
    let subnet_weight_1 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_1);
    let subnet_weight_2 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_2);
    let subnet_weight_3 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_3);
    let subnet_weight_4 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_4);
    let subnet_weight_5 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_5);

    let mut score_sum = 0;
    let mut nodes = 0;
    for (id, _) in OverwatchNodes::<Test>::iter() {
      nodes += 1;
      let weight = OverwatchNodeWeights::<Test>::get(epoch, id);
      score_sum += weight.unwrap();
    }

    assert_eq!(nodes, 8);
    assert!(score_sum <= 1000000000000000000 && score_sum >= 999999999999999990);
  });
}

#[test]
fn test_add_to_overwatch_stake() {
  new_test_ext().execute_with(|| {
    let amount = 100000000000000000000;

    let coldkey = account(1);
    let hotkey = account(2);
    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

    make_overwatch_qualified(1);

    let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
    let uids = TotalOverwatchNodeUids::<Test>::get();
    let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    assert!(!hotkeys.contains(&hotkey.clone()));

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      )
    );

    let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

    let increase_amount = 100000000000000000000;
    let _ = Balances::deposit_creating(&coldkey.clone(), increase_amount);

    assert_ok!(
      Network::add_to_overwatch_stake(
        RuntimeOrigin::signed(coldkey.clone()), 
        uid,
        hotkey.clone(),
        increase_amount,
      )
    );

    assert_eq!(AccountOverwatchStake::<Test>::get(hotkey.clone()), amount + increase_amount);
  });
}

#[test]
fn test_add_to_overwatch_stake_errors() {
  new_test_ext().execute_with(|| {
    let amount = 100000000000000000000;

    let coldkey = account(1);
    let hotkey = account(2);
    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

    make_overwatch_qualified(1);

    let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
    let uids = TotalOverwatchNodeUids::<Test>::get();
    let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    assert!(!hotkeys.contains(&hotkey.clone()));

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      )
    );

    let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();
    let increase_amount = 100000000000000000000;

    assert_err!(
      Network::add_to_overwatch_stake(
        RuntimeOrigin::signed(coldkey.clone()), 
        uid,
        hotkey.clone(),
        increase_amount,
      ),
      Error::<Test>::NotEnoughBalanceToStake
    );

    
    let _ = Balances::deposit_creating(&coldkey.clone(), increase_amount);

    assert_err!(
      Network::add_to_overwatch_stake(
        RuntimeOrigin::signed(coldkey.clone()), 
        uid,
        hotkey.clone(),
        increase_amount+500,
      ),
      Error::<Test>::BalanceWithdrawalError
    );
  });
}

#[test]
fn test_add_to_remove_overwatch_stake() {
  new_test_ext().execute_with(|| {
    let amount = 100000000000000000000;

    let coldkey = account(1);
    let hotkey = account(2);
    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

    make_overwatch_qualified(1);

    let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
    let uids = TotalOverwatchNodeUids::<Test>::get();
    let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    assert!(!hotkeys.contains(&hotkey.clone()));

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      )
    );

    let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

    let increase_amount = 100000000000000000000;
    let _ = Balances::deposit_creating(&coldkey.clone(), increase_amount);

    assert_ok!(
      Network::add_to_overwatch_stake(
        RuntimeOrigin::signed(coldkey.clone()), 
        uid,
        hotkey.clone(),
        increase_amount,
      )
    );

    assert_eq!(AccountOverwatchStake::<Test>::get(hotkey.clone()), amount + increase_amount);

    let remove_amount = 50000000000000000000;

    assert_ok!(
      Network::remove_overwatch_stake(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        remove_amount,
      )
    );

    assert_eq!(AccountOverwatchStake::<Test>::get(hotkey.clone()), amount + increase_amount - remove_amount);
  });
}

#[test]
fn test_add_to_remove_overwatch_stake_errors() {
  new_test_ext().execute_with(|| {
    let amount = 100000000000000000000;

    let coldkey = account(1);
    let hotkey = account(2);
    let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

    make_overwatch_qualified(1);

    let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
    let uids = TotalOverwatchNodeUids::<Test>::get();
    let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    assert!(!hotkeys.contains(&hotkey.clone()));

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount,
      )
    );

    let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

    let increase_amount = 100000000000000000000;
    let _ = Balances::deposit_creating(&coldkey.clone(), increase_amount);

    assert_ok!(
      Network::add_to_overwatch_stake(
        RuntimeOrigin::signed(coldkey.clone()), 
        uid,
        hotkey.clone(),
        increase_amount,
      )
    );

    assert_eq!(AccountOverwatchStake::<Test>::get(hotkey.clone()), amount + increase_amount);

    assert_err!(
      Network::remove_overwatch_stake(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        0,
      ),
      Error::<Test>::NotEnoughStakeToWithdraw
    );

    assert_err!(
      Network::remove_overwatch_stake(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount + increase_amount + increase_amount,
      ),
      Error::<Test>::NotEnoughStakeToWithdraw
    );

    assert_err!(
      Network::remove_overwatch_stake(
        RuntimeOrigin::signed(coldkey.clone()), 
        hotkey.clone(),
        amount + increase_amount,
      ),
      Error::<Test>::MinStakeNotReached
    );
  });
}
