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

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);
    let starting_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(subnet_id*total_subnet_nodes+1)),
        subnet_id,
        account(subnet_id*total_subnet_nodes+1),
        peer(subnet_id*total_subnet_nodes+1),
        peer(subnet_id*total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let post_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(post_balance, starting_balance - amount);

    let total_subnet_node_uids = TotalSubnetNodeUids::<Test>::get(subnet_id);
    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();
    assert_eq!(total_subnet_node_uids, hotkey_subnet_node_id);

    let subnet_node_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node_hotkey, Some(account(total_subnet_nodes+1)));

    let coldkey = HotkeyOwner::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(coldkey, account(total_subnet_nodes+1));

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node.hotkey, account(total_subnet_nodes+1));
    assert_eq!(subnet_node.peer_id, peer(total_subnet_nodes+1));
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Registered);

    let peer_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(peer_account, hotkey_subnet_node_id);

    let bootstrap_peer_account = BootstrapPeerIdSubnetNodeId::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(bootstrap_peer_account, hotkey_subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(account_subnet_stake, amount);

    // let queue = RegistrationQueue::<Test>::get(subnet_id);
		// assert_eq!(queue, vec![hotkey_subnet_node_id]);

    System::assert_last_event(RuntimeEvent::Network(crate::Event::SubnetNodeRegistered {
			subnet_id,
			subnet_node_id: hotkey_subnet_node_id,
			coldkey: account(total_subnet_nodes+1),
			hotkey: account(total_subnet_nodes+1),
			peer_id: peer(total_subnet_nodes+1),
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

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

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

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let start_epoch = subnet_node.classification.start_epoch;

    // let queue = RegistrationQueue::<Test>::get(subnet_id);
		// assert_eq!(queue, vec![hotkey_subnet_node_id]);

    // set_epoch(start_epoch);
    set_block_to_subnet_slot(start_epoch, subnet_id);

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);

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

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

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

    let total_subnet_node_uids = TotalSubnetNodeUids::<Test>::get(subnet_id);
    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let start_epoch = subnet_node.classification.start_epoch;

    // set_epoch(start_epoch);
    set_block_to_subnet_slot(start_epoch, subnet_id);

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let new_total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    assert_err!(
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
    log::error!("hotkey_subnet_node_id {:?}", hotkey_subnet_node_id);

    // assert!(false);

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

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(max_subnet_nodes+(total_subnet_nodes+1)*subnets), deposit_amount);
    
    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(max_subnet_nodes+(total_subnet_nodes+1)*subnets)),
        subnet_id,
        account(max_subnet_nodes+(total_subnet_nodes+1)*subnets),
        peer(max_subnet_nodes+(total_subnet_nodes+1)*subnets),
        peer(max_subnet_nodes+(total_subnet_nodes+1)*subnets),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(max_subnet_nodes+(total_subnet_nodes+1)*subnets)).unwrap();

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let start_epoch = subnet_node.classification.start_epoch;

    // --- Try starting before start_epoch
    assert_err!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(max_subnet_nodes+(total_subnet_nodes+1)*subnets)),
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
        RuntimeOrigin::signed(account(max_subnet_nodes+(total_subnet_nodes+1)*subnets)),
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

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

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

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id, 
        hotkey_subnet_node_id,
      )
    );

    assert_eq!(RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id).count(), 0);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, account(total_subnet_nodes+1));
    assert_eq!(subnet_node_id, Err(()));

    let peer_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(peer_account, Err(()));

    let bootstrap_peer_account = BootstrapPeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(bootstrap_peer_account, Err(()));

    let subnet_node_hotkey = SubnetNodeIdHotkey::<Test>::try_get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node_hotkey, Err(()));
  })
}

#[test]
fn test_add_subnet_node_subnet_err() {
  new_test_ext().execute_with(|| {
    let subnet_id = 0;

    let amount: u128 = 1000;
    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        peer(1),
        peer(1),
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
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        peer(1),
        peer(1),
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

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

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

    // let coldkey = account(subnets*max_subnet_nodes+end+1);
    // let hotkey = account(subnets*max_subnet_nodes+end+1);
    // let used_hotkey = account(subnets*max_subnet_nodes+end);

    // try reregistering again
    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        used_hotkey.clone(),
        peer(subnets*max_subnet_nodes+end),
        peer(subnets*max_subnet_nodes+end),
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

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let deposit_amount: u128 = 100000;
    let amount: u128 = 1;

    let _ = Balances::deposit_creating(&account(1), deposit_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    assert_err!(
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

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let deposit_amount: u128 = 999999999999999999999;

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_err!(
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

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let peer_id = format!("2");
    let peer: PeerId = PeerId(peer_id.clone().into());
    let bootstrap_peer: PeerId = PeerId(peer_id.clone().into());
    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer,
        bootstrap_peer,
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

    build_activated_subnet_new(subnet_name.clone(), 0, 16, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let _ = Balances::deposit_creating(&account(subnet_id*total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(subnet_id*total_subnet_nodes+1)),
        subnet_id,
        account(subnet_id*total_subnet_nodes+1),
        peer(subnet_id*total_subnet_nodes+1),
        peer(subnet_id*total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(subnet_id*total_subnet_nodes+1)).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(subnet_id*total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
      )
    );

    let account_subnet_stake = AccountSubnetStake::<Test>::get(&account(subnet_id*total_subnet_nodes+1), subnet_id);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(subnet_id*total_subnet_nodes+1)),
        subnet_id,
        account(subnet_id*total_subnet_nodes+1),
        account_subnet_stake,
      )
    );

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(subnet_id*total_subnet_nodes+1)),
        subnet_id,
        account(subnet_id*total_subnet_nodes+2),
        peer(subnet_id*total_subnet_nodes+1),
        peer(subnet_id*total_subnet_nodes+1),
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

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

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

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    assert_err!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
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

    build_activated_subnet_new(subnet_name.clone(), 0, 16, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let _ = Balances::deposit_creating(&account(subnet_id*total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(subnet_id*total_subnet_nodes+1)),
        subnet_id,
        account(subnet_id*total_subnet_nodes+1),
        peer(subnet_id*total_subnet_nodes+1),
        peer(subnet_id*total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(subnet_id*total_subnet_nodes+1)).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(subnet_id*total_subnet_nodes+1)),
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

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);
    let remove_n_peers = total_subnet_nodes / 2;

    let block_number = System::block_number();
    let epoch_length = EpochLength::get();
    let epoch = block_number / epoch_length;

    for n in 1..remove_n_peers+1 {
      let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n)).unwrap();
      assert_ok!(
        Network::remove_subnet_node(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
          subnet_node_id,
        ) 
      );
      let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id);
      assert_eq!(subnet_node_data, Err(()));
    }

    // let node_set = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Idle, epoch);
    let node_set: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Idle, epoch);

    assert_eq!(node_set.len(), (total_subnet_nodes - remove_n_peers) as usize);
    assert_eq!(Network::total_stake(), amount_staked);
    assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked);
    assert_eq!(TotalSubnetNodes::<Test>::get(subnet_id), total_subnet_nodes - remove_n_peers);

    for n in 1..remove_n_peers+1 {
      let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, account(n));
      assert_eq!(subnet_node_id, Err(()));

      let subnet_node_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer(n));
      assert_eq!(subnet_node_account, Err(()));
  
      let account_subnet_stake = AccountSubnetStake::<Test>::get(account(n), subnet_id);
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

// #[test]
// fn ema_should_lag_behind_increasing_node_count() {
//   new_test_ext().execute_with(|| {
//     let subnet_id = 77;
//     let mut block = 1;

//     // Start at 10 nodes, increase to 200 over time
//     let mut actual_node_count = 10u32;

//     // Run the loop for 20 simulated blocks
//     for _ in 0..20 {
//       Network::update_ema(subnet_id, actual_node_count, block);

//       let ema = SubnetNodeCountEMA::<Test>::get(subnet_id);
//       assert!(
//         ema < actual_node_count as u128 * Network::PERCENTAGE_FACTOR.low_u128(),
//         "EMA should be less than actual scaled node count (lagging behavior)"
//       );

//       log::error!("ema: {:?}, actual: {:?}", ema, actual_node_count);

//       // Increase the node count more aggressively over time
//       actual_node_count += 1;
//       block += 1;
//     }

//     assert!(false);

//     // At the end, the EMA should be close to—but still less than—the final value
//     let final_ema = SubnetNodeCountEMA::<Test>::get(subnet_id);
//     let final_node_scaled = U256::from(actual_node_count) * Network::PERCENTAGE_FACTOR;
//     assert!(
//       U256::from(final_ema) < final_node_scaled,
//       "Final EMA should still lag behind final node count"
//     );
//   });
// }

// // #[test]
// // fn ema_updates_correctly_on_first_insert() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_id = 42;
// //     let node_count = 100;
// //     let block = 10;

// //     // let min_nodes = MinSubnetNodes::<Test>::get();
// //     // Ensure storage is empty
// //     // assert_eq!(SubnetNodeCountEMA::<Test>::get(subnet_id), min_nodes as u128 * 1e+18 as u128);
// //     assert_eq!(SubnetNodeCountEMA::<Test>::get(subnet_id), 0);
// //     assert_eq!(SubnetNodeCountEMALastUpdated::<Test>::get(subnet_id), 0);

// //     // Run the function
// //     Network::update_ema(subnet_id, node_count, block);

// //     // The EMA should match node_count * percentage_factor on first update
// //     let expected = U256::from(node_count) * Network::PERCENTAGE_FACTOR;
// //     let stored = U256::from(SubnetNodeCountEMA::<Test>::get(subnet_id));
// //     assert_eq!(stored, expected);

// //     // Last block updated should be stored
// //     assert_eq!(SubnetNodeCountEMALastUpdated::<Test>::get(subnet_id), block);
// //   });
// // }

// #[test]
// fn ema_updates_smoothly_on_constant_value() {
//   new_test_ext().execute_with(|| {
//     let subnet_id = 7;
//     let node_count = 120;

//     // First update at block 5
//     Network::update_ema(subnet_id, node_count, 5);
//     let ema_1 = SubnetNodeCountEMA::<Test>::get(subnet_id);

//     // Update again at block 6 (short interval)
//     Network::update_ema(subnet_id, node_count, 6);
//     let ema_2 = SubnetNodeCountEMA::<Test>::get(subnet_id);

//     // Since the value didn’t change, EMA should stabilize near scaled node count
//     assert!(ema_2 >= ema_1);
//   });
// }

// #[test]
// fn ema_lags_on_increase_and_clamps() {
//   new_test_ext().execute_with(|| {
//     let subnet_id = 1;
//     let initial_count = 50;
//     let updated_count = 200;

//     // Initialize with block 10
//     Network::update_ema(subnet_id, initial_count, 10);

//     // Save initial EMA value
//     let initial_ema = SubnetNodeCountEMA::<Test>::get(subnet_id);
//     assert!(initial_ema > 0);

//     // Advance to block 15 and increase node count
//     Network::update_ema(subnet_id, updated_count, 15);

//     let updated_ema = SubnetNodeCountEMA::<Test>::get(subnet_id);
//     assert!(updated_ema > initial_ema, "EMA should increase");
//     assert!(updated_ema < updated_count * Network::PERCENTAGE_FACTOR.low_u128(), "EMA should lag behind new value");
//   });
// }

// #[test]
// fn test_node_count_ema_updates_correctly() {
//   new_test_ext().execute_with(|| {
//     let subnet_id = 1;
//     let alpha = Network::EMA_ALPHA_NUMERATOR;
//     let percentage_factor = Network::percentage_factor_as_u128();

//     // Initial value
//     let initial_node_count = 10;
//     let scaled_initial = (initial_node_count as u128) * percentage_factor;
//     SubnetNodeCountEMA::<Test>::insert(subnet_id, scaled_initial);

//     // Step 1: EMA = 10, new count = 20
//     Network::update_ema(subnet_id, 20);

//     let ema_after_1 = SubnetNodeCountEMA::<Test>::get(subnet_id);
//     let expected_ema_1 = (
//         alpha * 20 * percentage_factor +
//         (percentage_factor - alpha) * scaled_initial
//     ) / percentage_factor;

//     assert_eq!(ema_after_1, expected_ema_1);
//     assert!(ema_after_1 < scaled_initial); // Should be less than actual ndoe count
//     assert_eq!(Network::ema_as_rounded_up_integer(subnet_id), 11); // should be slightly > 10

//     // Step 2: Update again with node count = 30
//     Network::update_ema(subnet_id, 30);
//     let ema_after_2 = SubnetNodeCountEMA::<Test>::get(subnet_id);
//     let expected_ema_2 = (
//         alpha * 30 * percentage_factor +
//         (percentage_factor - alpha) * ema_after_1
//     ) / percentage_factor;

//     assert_eq!(ema_after_2, expected_ema_2);
//     let rounded = Network::ema_as_rounded_up_integer(subnet_id);
//     assert!(rounded >= 12 && rounded <= 13); // expect EMA to slowly approach 30

//     // Step 3: Drop back down to 15
//     Network::update_ema(subnet_id, 15);
//     let ema_after_3 = SubnetNodeCountEMA::<Test>::get(subnet_id);
//     let expected_ema_3 = (
//         alpha * 15 * percentage_factor +
//         (percentage_factor - alpha) * ema_after_2
//     ) / percentage_factor;

//     assert_eq!(ema_after_3, expected_ema_3);
//     let rounded = Network::ema_as_rounded_up_integer(subnet_id);
//     assert!(rounded >= 12 && rounded <= 14); // smooth reaction to decrease
//   });
// }

// #[test]
// fn test_ema_update_with_block_delta_and_clamp() {
//   new_test_ext().execute_with(|| {
//     let subnet_id = 42;
//     let percentage_factor = Network::percentage_factor_as_u128();
//     let pf = U256::from(percentage_factor);

//     // Step 0: Initialize with node count = 10 at block 1
//     let node_count_1 = 10;
//     let block_1 = 1;
//     Network::update_ema(subnet_id, node_count_1, block_1);

//     // Check that EMA is exactly 10 * 1e18
//     let ema_1 = SubnetNodeCountEMA::<Test>::get(subnet_id);
//     assert_eq!(U256::from(ema_1), U256::from(10u128) * pf);
//     assert_eq!(SubnetNodeCountEMALastUpdated::<Test>::get(subnet_id), block_1);

//     // Step 1: Next block, node count increases slightly to 12
//     let block_2 = 2;
//     let node_count_2 = 12u32;
//     Network::update_ema(subnet_id, node_count_2, block_2);

//     let ema_2 = SubnetNodeCountEMA::<Test>::get(subnet_id);
//     let expected_max = U256::from(node_count_2) * pf;
//     assert!(U256::from(ema_2) < expected_max, "EMA should not reach full value in 1 block");
//     assert!(U256::from(ema_2) > U256::from(10u128) * pf, "EMA should increase");
//     assert_eq!(SubnetNodeCountEMALastUpdated::<Test>::get(subnet_id), block_2);
    
//     // Step 2: Jump ahead 100 blocks, node count = 15
//     let block_100 = 102;
//     let node_count_3 = 15u32;
//     Network::update_ema(subnet_id, node_count_3, block_100);

//     let ema_3 = SubnetNodeCountEMA::<Test>::get(subnet_id);
//     let max_ema_3 = U256::from(node_count_3) * pf;

//     assert!(ema_3 > ema_2, "EMA should increase due to time decay");
//     assert!(U256::from(ema_3) <= max_ema_3, "EMA should be clamped to node count");
//     assert_eq!(SubnetNodeCountEMALastUpdated::<Test>::get(subnet_id), block_100);

//     // Step 3: Drop node count to 13, same block
//     let node_count_4 = 13u32;
//     Network::update_ema(subnet_id, node_count_4, block_100);

//     // let ema_4 = SubnetNodeCountEMA::<Test>::get(subnet_id);
//     // let max_ema_4 = U256::from(node_count_4) * pf;
//     // assert!(U256::from(ema_4) <= max_ema_4, "EMA clamped to 13 nodes");
//     // assert!(ema_4 < ema_3, "EMA should decrease slightly");
//     // assert_eq!(SubnetNodeCountEMALastUpdated::<Test>::get(subnet_id), block_100);
//   });
// }

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

    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, max_subnet_nodes, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
    let registration_queue_epochs = RegistrationQueueEpochs::<Test>::get(subnet_id);

    let coldkey = account(subnet_id*max_subnet_nodes+max_subnet_nodes+1);
    let hotkey = account(subnet_id*max_subnet_nodes+max_subnet_nodes+1);
    let peer_id = peer(subnet_id*max_subnet_nodes+max_subnet_nodes+1);
    let bootstrap_peer_id = peer(subnet_id*max_subnet_nodes+max_subnet_nodes+1);

    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
    let starting_balance = Balances::free_balance(&coldkey.clone());

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootstrap_peer_id.clone(),
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

    set_block_to_subnet_slot(initial_start_epoch, subnet_id);

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
