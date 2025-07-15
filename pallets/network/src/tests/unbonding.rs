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
  AccountSubnetStake,
  StakeUnbondingLedger, 
  HotkeySubnetNodeId, 
  NetworkMinStakeBalance,
  RegisteredSubnetNodesData,
  TotalActiveSubnets,
  MaxSubnetNodes,
};

///
///
///
///
///
///
///
/// Unbondings
///
///
///
///
///
///
///

#[test]
fn test_register_remove_claim_stake_unbondings() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let account_n = 10000;

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(account_n), deposit_amount);

    let starting_balance = Balances::free_balance(&account(account_n));
    assert_eq!(starting_balance, deposit_amount);

    assert_ok!(
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
      ) 
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    let stake_balance = AccountSubnetStake::<Test>::get(&account(account_n), subnet_id);
    assert_eq!(stake_balance, amount);

    let after_stake_balance = Balances::free_balance(&account(account_n));
    assert_eq!(after_stake_balance, starting_balance - amount);

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(account_n)), 
        subnet_id,
        subnet_node_id,
      ) 
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(account_n), subnet_id);

    // remove amount ontop
    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        stake_balance,
      )
    );

    assert_eq!(Network::account_subnet_stake(account(account_n), 1), 0);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n));

    assert_eq!(unbondings.len(), 1);
    let (first_key, first_value) = unbondings.iter().next().unwrap();
    
    assert_eq!(*first_key, &epoch + StakeCooldownEpochs::get());
    assert!(*first_value <= stake_balance);
    
    let stake_cooldown_epochs = StakeCooldownEpochs::get();

    increase_epochs(stake_cooldown_epochs + 1);

    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(account_n)),
      )
    );

    let post_balance = Balances::free_balance(&account(account_n));

    assert_eq!(post_balance, starting_balance);

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n));

    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_register_activate_remove_claim_stake_unbondings() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = 10000;

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(account_n), deposit_amount);

    let starting_balance = Balances::free_balance(&account(account_n));
    assert_eq!(starting_balance, deposit_amount);

    assert_ok!(
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
      ) 
    );

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let start_epoch = subnet_node.classification.start_epoch;

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    let stake_balance = AccountSubnetStake::<Test>::get(&account(account_n), subnet_id);
    assert_eq!(stake_balance, amount);

    let after_stake_balance = Balances::free_balance(&account(account_n));
    assert_eq!(after_stake_balance, starting_balance - amount);

    // set_epoch(start_epoch);
    set_block_to_subnet_slot(start_epoch, subnet_id);

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(account_n)), 
        subnet_id,
        subnet_node_id,
      ) 
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(account_n), subnet_id);

    // remove amount ontop
    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        stake_balance,
      )
    );

    assert_eq!(Network::account_subnet_stake(account(account_n), 1), 0);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n));

    assert_eq!(unbondings.len(), 1);
    let (first_key, first_value) = unbondings.iter().next().unwrap();
    
    assert_eq!(*first_key, &epoch + StakeCooldownEpochs::get());
    assert!(*first_value <= stake_balance);
    
    let stake_cooldown_epochs = StakeCooldownEpochs::get();

    increase_epochs(stake_cooldown_epochs + 1);

    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(account_n)),
      )
    );

    let post_balance = Balances::free_balance(&account(account_n));

    assert_eq!(post_balance, starting_balance);

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n));

    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_remove_stake_twice_in_epoch() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = 10000;

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(account_n), deposit_amount);

    let starting_balance = Balances::free_balance(&account(account_n));
    assert_eq!(starting_balance, deposit_amount);

    assert_ok!(
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
      ) 
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();

    let stake_balance = AccountSubnetStake::<Test>::get(&account(account_n), subnet_id);
    assert_eq!(stake_balance, amount);

    let after_stake_balance = Balances::free_balance(&account(account_n));
    assert_eq!(after_stake_balance, starting_balance - amount);

    let _ = Balances::deposit_creating(&account(1), amount*2);

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        subnet_node_id,
        account(account_n),
        amount*3,
      ) 
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(account_n), subnet_id);
    assert_eq!(stake_balance, amount + amount*3);

    let epoch = System::block_number() / EpochLength::get();

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        amount,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n));
    let ledger_balance: u128 = unbondings.values().copied().sum();
    assert_eq!(unbondings.len() as u32, 1);  
    assert_eq!(ledger_balance, amount);  

    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + StakeCooldownEpochs::get());

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        amount,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n));
    let ledger_balance: u128 = unbondings.values().copied().sum();
    assert_eq!(unbondings.len() as u32, 1);  
    assert_eq!(ledger_balance, amount*2);

    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + StakeCooldownEpochs::get());

    increase_epochs(1);

    let epoch = System::block_number() / EpochLength::get();

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        amount,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n));
    let total_ledger_balance: u128 = unbondings.values().copied().sum();
    assert_eq!(unbondings.len() as u32, 2);  
    assert_eq!(total_ledger_balance, amount*3);

    let (ledger_epoch, ledger_balance) = unbondings.iter().last().unwrap();
    assert_eq!(*ledger_epoch, &epoch + StakeCooldownEpochs::get());
    assert_eq!(*ledger_balance, amount);

    System::set_block_number(System::block_number() + ((EpochLength::get()  + 1) * StakeCooldownEpochs::get()));
    // increase_epochs(StakeCooldownEpochs::get() + 11);
    
    let starting_balance = Balances::free_balance(&account(account_n));

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(account_n)),
      )
    );

    let ending_balance = Balances::free_balance(&account(account_n));
    assert_eq!(starting_balance + total_ledger_balance, ending_balance);

  });
}

#[test]
fn test_claim_stake_unbondings_no_unbondings_err() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = 10000;

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(account_n), deposit_amount);

    let starting_balance = Balances::free_balance(&account(account_n));
    assert_eq!(starting_balance, deposit_amount);

    assert_ok!(
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
      ) 
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(account_n), subnet_id);
    assert_eq!(stake_balance, amount);

    let after_stake_balance = Balances::free_balance(&account(account_n));
    assert_eq!(after_stake_balance, starting_balance - amount);

    assert_err!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(account_n)),
      ),
      Error::<Test>::NoStakeUnbondingsOrCooldownNotMet
    );
  });
}

#[test]
fn test_remove_to_stake_max_unlockings_reached_err() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let account_n = 10000;

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(account_n), deposit_amount);

    let starting_balance = Balances::free_balance(&account(account_n));

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        peer(account_n),
        peer(account_n),
        0,
        amount*2,
        None,
        None,
        None,
      ) 
    );

    let max_unlockings = MaxStakeUnlockings::get();
    for n in 1..max_unlockings+2 {
      increase_epochs(1);
      if n > max_unlockings {
        assert_err!(
          Network::remove_stake(
            RuntimeOrigin::signed(account(account_n)),
            subnet_id,
            account(account_n),
            1000,
          ),
          Error::<Test>::MaxUnlockingsReached
        );    
      } else {
        assert_ok!(
          Network::remove_stake(
            RuntimeOrigin::signed(account(account_n)),
            subnet_id,
            account(account_n),
            1000,
          )
        );

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n));

        assert_eq!(unbondings.len() as u32, n);  
      }
    }
  });
}

