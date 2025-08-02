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
  TotalStake, 
  SubnetName, 
  TotalSubnetNodes,
  SubnetNodeClass,
  AccountSubnetStake,
  TotalSubnetStake, 
  HotkeyOwner, 
  TotalSubnetNodeUids, 
  HotkeySubnetNodeId, 
  SubnetNodeIdHotkey, 
  SubnetNodesData,
  SubnetNode,
  PeerIdSubnetNodeId,
  BootstrapPeerIdSubnetNodeId,
  MaxRewardRateDecrease,
  RewardRateUpdatePeriod,
  NetworkMinStakeBalance,
  ActivationGraceEpochs,
  RegisteredSubnetNodesData,
  DeactivatedSubnetNodesData,
  MaxSubnetNodes,
  TotalActiveSubnets,
  SubnetNodeCountEMA,
  SubnetNodeCountEMALastUpdated,
  MinSubnetNodes,
  TotalActiveSubnetNodes,
  RegistrationQueueEpochs,
  MaxSubnets,
  NodeRemovalPolicy,
  LogicExpr,
  NodeRemovalConditionType,
  NodeRemovalSystemV2,
  ColdkeyReputation,
  NodeDelegateStakeBalance,
  Reputation,
  SubnetNodeClassification,
  DefaultMaxVectorLength,
};
use sp_core::U256;
///
///
///
///
///
///
///
/// Subnet Nodes Add/Remove
///
///
///
///
///
///
///

#[test]
fn test_register_subnet_node_post_subnet_activation() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 4;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);

    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
    let starting_balance = Balances::free_balance(&coldkey.clone());

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let post_balance = Balances::free_balance(&coldkey.clone());
    assert_eq!(post_balance, starting_balance - amount);

    let total_subnet_node_uids = TotalSubnetNodeUids::<Test>::get(subnet_id);
    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
    assert_eq!(total_subnet_node_uids, hotkey_subnet_node_id);

    let subnet_node_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node_hotkey, Some(hotkey.clone()));

    let coldkey = HotkeyOwner::<Test>::get(hotkey.clone());
    assert_eq!(coldkey, coldkey.clone());

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node.hotkey, hotkey.clone());
    assert_eq!(subnet_node.peer_id, peer_id.clone());
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Registered);

    let peer_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
    assert_eq!(peer_account, hotkey_subnet_node_id);

    let bootnode_peer_account = BootstrapPeerIdSubnetNodeId::<Test>::get(subnet_id, bootnode_peer_id.clone());
    assert_eq!(bootnode_peer_account, hotkey_subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
    assert_eq!(account_subnet_stake, amount);

    System::assert_last_event(RuntimeEvent::Network(crate::Event::SubnetNodeRegistered {
			subnet_id,
			subnet_node_id: hotkey_subnet_node_id,
			coldkey: coldkey.clone(),
			hotkey: hotkey.clone(),
			peer_id: peer_id.clone(),
		}));
  })
}

#[test]
fn test_activate_subnet_node_post_subnet_activation() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    let end = 4;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);
    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);


    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id,
        bootnode_peer_id,
        None,
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let start_epoch = subnet_node.classification.start_epoch;

    set_block_to_subnet_slot_epoch(start_epoch, subnet_id);

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
    assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

    let new_total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(total_subnet_nodes + 1, new_total_nodes);
  })
}

#[test]
fn test_register_after_activate_with_same_keys() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    let end = 4;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);

    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
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
    set_block_to_subnet_slot_epoch(start_epoch, subnet_id);

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let new_total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::HotkeyHasOwner
    );
  })
}

#[test]
fn test_register_after_deactivate_with_same_keys() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    
    let end = 3;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        end
      )
    );

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer(1),
        peer(1),
        None,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::HotkeyHasOwner
    );

  })
}

#[test]
fn test_activate_subnet_node_not_start_epoch() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 12;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);

    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
    
    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id,
        None,
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let start_epoch = subnet_node.classification.start_epoch;

    // --- Try starting before start_epoch
    assert_err!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey_subnet_node_id
      ),
      Error::<Test>::NotStartEpoch
    );

    let grace_epochs = ActivationGraceEpochs::<Test>::get(subnet_id);
    set_epoch(start_epoch + grace_epochs + 2);

    // --- Try starting after ActivationGraceEpochs
    assert_err!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey_subnet_node_id
      ),
      Error::<Test>::NotStartEpoch
    );
  })
}

#[test]
fn test_register_subnet_node_and_remove() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 12;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);
    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id, 
        hotkey_subnet_node_id,
      )
    );

    assert_eq!(RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id).count(), 0);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, hotkey.clone());
    assert_eq!(subnet_node_id, Err(()));

    let peer_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer_id.clone());
    assert_eq!(peer_account, Err(()));

    let bootnode_peer_account = BootstrapPeerIdSubnetNodeId::<Test>::try_get(subnet_id, bootnode_peer_id.clone());
    assert_eq!(bootnode_peer_account, Err(()));

    let subnet_node_hotkey = SubnetNodeIdHotkey::<Test>::try_get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node_hotkey, Err(()));
  })
}

#[test]
fn test_add_subnet_node_subnet_err() {
  new_test_ext().execute_with(|| {
    let subnet_id = 0;

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 0;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);

    let amount: u128 = 1000;
    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::InvalidSubnet
    );

    let subnet_id = 1;

    assert_err!(Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::InvalidSubnet
    );
  })
}

#[test]
fn test_get_classification_subnet_nodes() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let end = 4;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;
  
    let submittable = Network::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Validator, epoch);

    assert_eq!(submittable.len() as u32, total_subnet_nodes);
  })
}

#[test]
fn test_add_subnet_node_not_exists_err() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    
    let end = 16;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
    let used_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // try reregistering again
    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        used_hotkey.clone(),
        peer(subnets*max_subnet_nodes+end),
        peer(subnets*max_subnet_nodes+end),
        None,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::HotkeyHasOwner
    );

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer(subnets*max_subnet_nodes+end),
        peer(subnets*max_subnet_nodes+end),
        None,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::PeerIdExist
    );
  })
}

#[test]
fn test_add_subnet_node_stake_err() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 12;


    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let deposit_amount: u128 = 100000;
    let amount: u128 = 1;

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);
    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::MinStakeNotReached
    );
  })
}

#[test]
fn test_add_subnet_node_stake_not_enough_balance_err() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 4;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let deposit_amount: u128 = 999999999999999999999;

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);
    // let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::NotEnoughBalanceToStake
    );
  })
}

#[test]
fn test_add_subnet_node_invalid_peer_id_err() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 4;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);


    let peer_id = format!("2");

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer: PeerId = PeerId(peer_id.clone().into());
    let bootnode_peer: PeerId = PeerId(peer_id.clone().into());

    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer,
        bootnode_peer,
        None,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::InvalidPeerId
    );
  })
}

#[test]
fn test_add_subnet_node_remove_readd_new_hotkey() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 4;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);
    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
      )
    );

    let account_subnet_stake = AccountSubnetStake::<Test>::get(&hotkey.clone(), subnet_id);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        account_subnet_stake,
      )
    );

    let new_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+2);
    let new_peer_id = peer(subnets*max_subnet_nodes+end+2);
    let new_bootnode_peer_id = peer(subnets*max_subnet_nodes+end+2);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        new_hotkey.clone(),
        new_peer_id.clone(),
        new_bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      )
    );
  });
}

#[test]
fn test_add_subnet_node_not_key_owner() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 4;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);
    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    assert_err!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        1,
      ),
      Error::<Test>::NotKeyOwner
    );

  });
}

#[test]
fn test_add_subnet_node_remove_readd_must_unstake_error() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 12;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);
    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
      )
    );

    // assert_err!(
    //   Network::register_subnet_node(
    //     RuntimeOrigin::signed(account(subnet_id*total_subnet_nodes+1)),
    //     subnet_id,
    //     account(subnet_id*total_subnet_nodes+1),
    //     peer(subnet_id*total_subnet_nodes+1),
    //     peer(subnet_id*total_subnet_nodes+1),
    //     None,
    //     0,
    //     amount,
    //     None,
    //     None,
    //     None,
    //   ),
    //   Error::<Test>::MustUnstakeToRegister
    // );
  });
}

#[test]
fn test_remove_subnet_node() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = 4;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    // let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    // let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    // let peer_id = peer(subnets*max_subnet_nodes+end+1);
    // let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);

    // let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);
    let remove_n_peers = total_subnet_nodes / 2;

    let block_number = System::block_number();
    let epoch_length = EpochLength::get();
    let epoch = block_number / epoch_length;

    for n in 0..remove_n_peers {
      let _n = n + 1;
      let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
      assert_ok!(
        Network::remove_subnet_node(
          RuntimeOrigin::signed(coldkey.clone()),
          subnet_id,
          subnet_node_id,
        ) 
      );
      let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id);
      assert_eq!(subnet_node_data, Err(()));
    }

    let node_set: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Idle, epoch);

    assert_eq!(node_set.len(), (total_subnet_nodes - remove_n_peers) as usize);
    assert_eq!(Network::total_stake(), amount_staked);
    assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked);
    assert_eq!(TotalSubnetNodes::<Test>::get(subnet_id), total_subnet_nodes - remove_n_peers);

    for n in 0..remove_n_peers {
      let _n = n + 1;
      let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      let peer_id = peer(subnets*max_subnet_nodes+_n);

      let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, hotkey.clone());
      assert_eq!(subnet_node_id, Err(()));

      let subnet_node_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer_id.clone());
      assert_eq!(subnet_node_account, Err(()));
  
      let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
      assert_eq!(account_subnet_stake, amount);
    }

    let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<Test>::get();
    assert_eq!(total_subnet_stake, amount_staked);
  });
}

#[test]
fn test_update_delegate_reward_rate() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let n_peers = 8;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    
    let end = 3;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

    // let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_name.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.delegate_reward_rate, 0);
    assert_eq!(subnet_node.last_delegate_reward_rate_update, 0);


    let max_reward_rate_decrease = MaxRewardRateDecrease::<Test>::get();
    let reward_rate_update_period = RewardRateUpdatePeriod::<Test>::get();
    let new_delegate_reward_rate = 50_000_000;

    System::set_block_number(System::block_number() + reward_rate_update_period);

    let block_number = System::block_number();

    // Increase reward rate to 5% then test decreasing
    assert_ok!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
        new_delegate_reward_rate
      )
    );
  
    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.delegate_reward_rate, new_delegate_reward_rate);
    assert_eq!(subnet_node.last_delegate_reward_rate_update, block_number);

    System::set_block_number(System::block_number() + reward_rate_update_period);

    let new_delegate_reward_rate = new_delegate_reward_rate - max_reward_rate_decrease;

    // allow decreasing by 1%
    assert_ok!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
        new_delegate_reward_rate
      )
    );

    // Higher than 100%
    assert_err!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
        1000000000000000001
      ),
      Error::<Test>::InvalidDelegateRewardRate
    );

    // Update rewards rate as an increase too soon
    assert_err!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
        new_delegate_reward_rate+1
      ),
      Error::<Test>::MaxRewardRateUpdates
    );

    System::set_block_number(System::block_number() + reward_rate_update_period);

    // Update rewards rate with no changes, don't allow
    assert_err!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
        new_delegate_reward_rate
      ),
      Error::<Test>::NoDelegateRewardRateChange
    );

  });
}

#[test]
fn test_update_delegate_reward_rate_not_key_owner() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    
    let end = 3;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.delegate_reward_rate, 0);
    assert_eq!(subnet_node.last_delegate_reward_rate_update, 0);


    let max_reward_rate_decrease = MaxRewardRateDecrease::<Test>::get();
    let reward_rate_update_period = RewardRateUpdatePeriod::<Test>::get();
    let new_delegate_reward_rate = 50_000_000;

    System::set_block_number(System::block_number() + reward_rate_update_period);

    let block_number = System::block_number();

    // Increase reward rate to 5% then test decreasing
    assert_err!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(account(2)),
        subnet_id,
        subnet_node_id,
        new_delegate_reward_rate
      ),
      Error::<Test>::NotKeyOwner
    );
  });
}

#[test]
fn test_deactivate_subnet_node_reactivate() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    
    let end = 3;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Validator);    

    let epoch = get_epoch();
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
      )
    );

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node_data, Err(()));

    let subnet_node = DeactivatedSubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Deactivated);    
    assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);    

    increase_epochs(1);

    let epoch = get_epoch();
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    assert_ok!(
      Network::reactivate_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
      )
    );

    let deactivated_subnet_node_data = DeactivatedSubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id);
    assert_eq!(deactivated_subnet_node_data, Err(()));

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Validator);    
    assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);
  })
}

#[test]
fn test_update_peer_id() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    
    let end = 3;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

    let current_peer_id = subnet_node.peer_id;

    assert_ok!(
      Network::update_peer_id(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
        peer(500)
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.peer_id, peer(500));
    assert_ne!(subnet_node.peer_id, current_peer_id);

    let peer_subnet_node_id = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer(500));
    assert_eq!(peer_subnet_node_id, subnet_node_id);

    let prev_peer_subnet_node_id = PeerIdSubnetNodeId::<Test>::get(subnet_id, current_peer_id);
    assert_ne!(prev_peer_subnet_node_id, subnet_node_id);
  })
}

#[test]
fn test_update_peer_id_exists() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    
    let end = 3;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

    let current_peer_id = subnet_node.peer_id;

    assert_err!(
      Network::update_peer_id(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
        peer(subnets*max_subnet_nodes+end-1)
      ),
      Error::<Test>::PeerIdExist
    );

    // --- fail if same peer id
    assert_err!(
      Network::update_peer_id(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
        current_peer_id
      ),
      Error::<Test>::PeerIdExist
    );
  })
}

#[test]
fn test_update_peer_id_not_key_owner() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    
    let end = 3;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

    let current_peer_id = subnet_node.peer_id;

    assert_err!(
      Network::update_peer_id(
        RuntimeOrigin::signed(account(2)),
        subnet_id,
        subnet_node_id,
        peer(1)
      ),
      Error::<Test>::NotKeyOwner
    );
  })
}

#[test]
fn test_update_bootnode() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    
    let end = 3;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

    let bootnode: Vec<u8> = "new-bootnode".into();
    let bounded_bootnode: BoundedVec<u8, DefaultMaxVectorLength> = bootnode.try_into().expect("String too long");

    assert_ok!(
      Network::update_bootnode(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        subnet_node_id,
        Some(bounded_bootnode.clone())
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.bootnode, Some(bounded_bootnode.clone()));
  })
}

#[test]
fn test_update_bootnode_not_key_owner() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    
    let end = 3;

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

    let bootnode: Vec<u8> = "new-bootnode".into();
    let bounded_bootnode: BoundedVec<u8, DefaultMaxVectorLength> = bootnode.try_into().expect("String too long");

    assert_err!(
      Network::update_bootnode(
        RuntimeOrigin::signed(account(2)),
        subnet_id,
        subnet_node_id,
        Some(bounded_bootnode.clone())
      ),
      Error::<Test>::NotKeyOwner
    );
  })
}

#[test]
fn subnet_stake_multiplier_works() {
  new_test_ext().execute_with(|| {
      let subnet_id = 1;

      // Set test constants
      MinSubnetNodes::<Test>::put(10);
      MaxSubnetNodes::<Test>::put(100);
      TotalActiveSubnetNodes::<Test>::insert(subnet_id, 10);

      // Multiplier should be 100% at min
      let mult = Network::get_subnet_min_delegate_staking_multiplier(10);
      assert_eq!(mult, Network::percentage_factor_as_u128()); // 100%

      // Multiplier should be 400% at max
      TotalActiveSubnetNodes::<Test>::insert(subnet_id, 100);
      let mult = Network::get_subnet_min_delegate_staking_multiplier(100);
      assert_eq!(mult, 4000000000000000000); // 400%

      // Multiplier should be ~250% halfway
      TotalActiveSubnetNodes::<Test>::insert(subnet_id, 55); // halfway between 10 and 100
      let mult = Network::get_subnet_min_delegate_staking_multiplier(55);
      let expected = Network::percentage_factor_as_u128() + (3000000000000000000 / 2);
      assert_eq!(mult, expected);
  });
}

#[test]
fn test_subnet_overwatch_node_unique_hotkeys() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let end = 16;

    build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let deposit_amount: u128 = 1000000000000000000000000;

    let free_coldkey = account(subnet_id*total_subnet_nodes+1);
    let hotkey = account(max_subnet_nodes+end*subnets+1);
    let free_hotkey = account(max_subnet_nodes+end*subnets+2);

    let _ = Balances::deposit_creating(&free_coldkey, deposit_amount);

    assert_ok!(
      Network::register_overwatch_node(
        RuntimeOrigin::signed(free_coldkey.clone()),
        hotkey.clone(),
        stake_amount,
      )
    );

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(free_coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer(subnet_id*total_subnet_nodes+1),
        peer(subnet_id*total_subnet_nodes+1),
        None,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::HotkeyHasOwner
    );

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(free_coldkey.clone()),
        subnet_id,
        free_hotkey.clone(),
        peer(subnet_id*total_subnet_nodes+1),
        peer(subnet_id*total_subnet_nodes+1),
        None,
        0,
        amount,
        None,
        None,
        None,
      )
    );
  });
}

#[test]
fn test_defer_node() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let end = max_subnet_nodes;

    build_activated_subnet_new(subnet_name.clone(), 0, max_subnet_nodes, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
    let registration_queue_epochs = RegistrationQueueEpochs::<Test>::get(subnet_id);

    let coldkey = get_coldkey(subnets, max_subnet_nodes, end+1);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end+1);
    let peer_id = peer(subnets*max_subnet_nodes+end+1);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+end+1);

    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
    let starting_balance = Balances::free_balance(&coldkey.clone());

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id.clone(),
        None,
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let initial_start_epoch = subnet_node.classification.start_epoch;

    set_block_to_subnet_slot_epoch(initial_start_epoch, subnet_id);

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(hotkey.clone()),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    assert_eq!(SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id), Err(()));

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let deferred_start_epoch = subnet_node.classification.start_epoch;
    assert_ne!(initial_start_epoch, deferred_start_epoch);
    assert_eq!(deferred_start_epoch, subnet_epoch + registration_queue_epochs);
  })
}

#[test]
fn test_get_removing_node_respects_policy() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let owner = account(0);
    let challenger_coldkey = account(100);
    let challenger_hotkey = account(101);

    let removal_policy = NodeRemovalPolicy {
      logic: LogicExpr::And(
        Box::new(LogicExpr::Condition(NodeRemovalConditionType::DeltaBelowScore(200))),
        Box::new(LogicExpr::Condition(NodeRemovalConditionType::DeltaBelowNodeDelegateStakeBalance(100))),
      )
    };

    NodeRemovalSystemV2::<Test>::insert(subnet_id, removal_policy);

    // Insert challenger node
    RegisteredSubnetNodesData::<Test>::insert(subnet_id, 0, SubnetNode {
      id: 0,
      hotkey: challenger_hotkey.clone(),
      peer_id: peer(0),
      bootnode_peer_id: peer(0),
      client_peer_id: peer(0),
      bootnode: None,
      delegate_reward_rate: 10,
      last_delegate_reward_rate_update: 0,
      classification: SubnetNodeClassification {
        node_class: SubnetNodeClass::Validator,
        start_epoch: 100,
      },
      a: Some(BoundedVec::new()),
      b: Some(BoundedVec::new()),
      c: Some(BoundedVec::new()),
    });

    ColdkeyReputation::<Test>::insert(&challenger_coldkey, Reputation { 
      score: 1000000000000000000, 
      average_attestation: 500000000000000000,
      start_epoch: 0,
      lifetime_node_count: 0,
      total_active_nodes: 0,
      total_increases: 0,
      total_decreases: 0,
      last_validator_epoch: 0,
      ow_score: 0,
    });
    NodeDelegateStakeBalance::<Test>::insert(subnet_id, 0, 300);
    AccountSubnetStake::<Test>::insert(&challenger_hotkey, subnet_id, 500);

    // Candidate 1 (should be chosen)
    let candidate_hotkey = account(1);
    let candidate_coldkey = account(2);
    SubnetNodesData::<Test>::insert(subnet_id, 1, SubnetNode {
      id: 1,
      hotkey: candidate_hotkey.clone(),
      peer_id: peer(1),
      bootnode_peer_id: peer(1),
      client_peer_id: peer(1),
      bootnode: None,
      delegate_reward_rate: 5,
      last_delegate_reward_rate_update: 0,
      classification: SubnetNodeClassification {
        node_class: SubnetNodeClass::Validator,
        start_epoch: 50,
      },
      a: Some(BoundedVec::new()),
      b: Some(BoundedVec::new()),
      c: Some(BoundedVec::new()),
    });
    ColdkeyReputation::<Test>::insert(&candidate_coldkey, Reputation {
      score: 700000000000000000, 
      average_attestation: 400000000000000000,
      start_epoch: 0,
      lifetime_node_count: 0,
      total_active_nodes: 0,
      total_increases: 0,
      total_decreases: 0,
      last_validator_epoch: 0,
      ow_score: 0,
    });
    NodeDelegateStakeBalance::<Test>::insert(subnet_id, 1, 100); // delta = 200
    AccountSubnetStake::<Test>::insert(&candidate_hotkey, subnet_id, 300);

    // Candidate 2 (should be skipped: score difference too small)
    let other_hotkey = account(3);
    let other_coldkey = account(4);
    SubnetNodesData::<Test>::insert(subnet_id, 2, SubnetNode {
      id: 2,
      hotkey: other_hotkey.clone(),
      peer_id: peer(2),
      bootnode_peer_id: peer(2),
      client_peer_id: peer(2),
      bootnode: None,
      delegate_reward_rate: 5,
      last_delegate_reward_rate_update: 0,
      classification: SubnetNodeClassification {
        node_class: SubnetNodeClass::Validator,
        start_epoch: 30,
      },
      a: Some(BoundedVec::new()),
      b: Some(BoundedVec::new()),
      c: Some(BoundedVec::new()),
    });
    ColdkeyReputation::<Test>::insert(&other_coldkey, Reputation {
      score: 950000000000000000,
      average_attestation: 400000000000000000,
      start_epoch: 0,
      lifetime_node_count: 0,
      total_active_nodes: 0,
      total_increases: 0,
      total_decreases: 0,
      last_validator_epoch: 0,
      ow_score: 0,
    });
    NodeDelegateStakeBalance::<Test>::insert(subnet_id, 2, 250); // delta = 50 (fails threshold)
    AccountSubnetStake::<Test>::insert(&other_hotkey, subnet_id, 400);

    let maybe_uid = Network::get_removing_node(
      subnet_id,
      &challenger_coldkey,
      &challenger_hotkey,
      &RegisteredSubnetNodesData::<Test>::get(subnet_id, 0)
    );

    assert_eq!(maybe_uid, Some(1));
  });
}
