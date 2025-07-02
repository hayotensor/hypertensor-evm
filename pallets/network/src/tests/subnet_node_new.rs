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
  PeerIdSubnetNode,
  BootstrapPeerIdSubnetNode,
  MaxRewardRateDecrease,
  RewardRateUpdatePeriod,
  MinStakeBalance,
  ActivationGraceEpochs,
  RegisteredSubnetNodesData,
  DeactivatedSubnetNodesData,
  MaxSubnetNodes,
  TotalActiveSubnets,
};

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

    let peer_account = PeerIdSubnetNode::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(peer_account, hotkey_subnet_node_id);

    let bootstrap_peer_account = BootstrapPeerIdSubnetNode::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
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
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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

    set_epoch(start_epoch);

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Queue);

    let new_total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(total_subnet_nodes + 1, new_total_nodes);
  })
}

#[test]
fn test_register_after_activate_with_same_keys() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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

    set_epoch(start_epoch);

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
      Error::<Test>::SubnetNodeExist
    );
  })
}

#[test]
fn test_register_after_deactivate_with_same_keys() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        1
      )
    );

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        peer(account_n),
        peer(account_n),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetNodeExist
    );

  })
}

#[test]
fn test_activate_subnet_node_not_start_epoch() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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

    let peer_account = PeerIdSubnetNode::<Test>::try_get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(peer_account, Err(()));

    let bootstrap_peer_account = BootstrapPeerIdSubnetNode::<Test>::try_get(subnet_id, peer(total_subnet_nodes+1));
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
      Error::<Test>::SubnetNotExist
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
      Error::<Test>::SubnetNotExist
    );
  })
}

#[test]
fn test_get_classification_subnet_nodes() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let end = 16;

    build_activated_subnet_new(subnet_path.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // try reregistering again
    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(max_subnet_nodes+1*subnets)),
        subnet_id,
        account(max_subnet_nodes+1*subnets),
        peer(max_subnet_nodes+1*subnets),
        peer(max_subnet_nodes+1*subnets),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetNodeExist
    );

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(max_subnet_nodes+end+1*subnets)),
        subnet_id,
        account(max_subnet_nodes+end+1*subnets),
        peer(max_subnet_nodes+1*subnets),
        peer(max_subnet_nodes+1*subnets),
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
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let deposit_amount: u128 = 100000;
    let amount: u128 = 1;

    let _ = Balances::deposit_creating(&account(1), deposit_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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
    let subnet_path: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let deposit_amount: u128 = 999999999999999999999;

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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
    let subnet_path: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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
fn test_add_subnet_node_remove_readd() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 16, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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
  });
}

#[test]
fn test_add_subnet_node_not_key_owner() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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
    let subnet_path: Vec<u8> = "subnet-name".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 16, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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

    assert_err!(
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
      ),
      Error::<Test>::MustUnstakeToRegister
    );
  });
}

#[test]
fn test_remove_subnet_node() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
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

    // let node_set = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Queue, epoch);
    let node_set: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Queue, epoch);

    assert_eq!(node_set.len(), (total_subnet_nodes - remove_n_peers) as usize);
    assert_eq!(Network::total_stake(), amount_staked);
    assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked);
    assert_eq!(TotalSubnetNodes::<Test>::get(subnet_id), total_subnet_nodes - remove_n_peers);

    for n in 1..remove_n_peers+1 {
      let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, account(n));
      assert_eq!(subnet_node_id, Err(()));

      let subnet_node_account = PeerIdSubnetNode::<Test>::try_get(subnet_id, peer(n));
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
    let subnet_path: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let n_peers = 8;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_path.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

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
        RuntimeOrigin::signed(account(account_n)),
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
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
        new_delegate_reward_rate
      )
    );

    // Higher than 100%
    assert_err!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
        1000000000000000001
      ),
      Error::<Test>::InvalidDelegateRewardRate
    );

    // Update rewards rate as an increase too soon
    assert_err!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(account(account_n)),
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
        RuntimeOrigin::signed(account(account_n)),
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
    let subnet_path: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let n_peers = 8;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_path.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

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
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Validator);    

    let epoch = get_epoch();

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
      )
    );

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node_data, Err(()));

    let subnet_node = DeactivatedSubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Deactivated);    
    assert_eq!(subnet_node.classification.start_epoch, epoch + 1);    

    increase_epochs(1);

    let epoch = get_epoch();

    assert_ok!(
      Network::reactivate_subnet_node(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
      )
    );

    let deactivated_subnet_node_data = DeactivatedSubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id);
    assert_eq!(deactivated_subnet_node_data, Err(()));

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Validator);    
    assert_eq!(subnet_node.classification.start_epoch, epoch + 1);
  })
}

#[test]
fn test_update_peer_id() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

    let current_peer_id = subnet_node.peer_id;

    assert_ok!(
      Network::update_peer_id(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
        peer(500)
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.peer_id, peer(500));
    assert_ne!(subnet_node.peer_id, current_peer_id);

    let peer_subnet_node_id = PeerIdSubnetNode::<Test>::get(subnet_id, peer(500));
    assert_eq!(peer_subnet_node_id, subnet_node_id);

    let prev_peer_subnet_node_id = PeerIdSubnetNode::<Test>::get(subnet_id, current_peer_id);
    assert_ne!(prev_peer_subnet_node_id, subnet_node_id);
  })
}

#[test]
fn test_update_peer_id_exists() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let end = 5;

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_path.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

    let current_peer_id = subnet_node.peer_id;

    assert_err!(
      Network::update_peer_id(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
        peer(max_subnet_nodes+end*subnets)
      ),
      Error::<Test>::PeerIdExist
    );
  })
}

#[test]
fn test_update_peer_id_not_key_owner() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = max_subnet_nodes+1*subnets;

    build_activated_subnet_new(subnet_path.clone(), 0, 5, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

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