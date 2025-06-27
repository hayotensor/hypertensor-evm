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
  HotkeyOwner, 
  HotkeySubnetNodeId, 
  SubnetNodeIdHotkey, 
  SubnetNodesData, 
  MinStakeBalance,
  TotalActiveSubnets,
  MaxSubnetNodes,
};


#[test]
fn test_update_coldkey() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let end = 16;

    let account_n = max_subnet_nodes+end*subnets;

    build_activated_subnet_new(subnet_path.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();
    let starting_account_subnet_stake = AccountSubnetStake::<Test>::get(account(account_n), subnet_id);

    // add extra stake and then add to ledger to check if it swapped
    let add_stake_amount = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(account_n), deposit_amount);

    //
    //
    // Coldkey = 1
    // Hotkey  = 1
    //
    //

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        hotkey_subnet_node_id,
        account(account_n),
        add_stake_amount,
      )
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(account_n), subnet_id);
    assert_eq!(stake_balance, starting_account_subnet_stake + add_stake_amount);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        amount,
      )
    );

    let original_unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n));
    let original_ledger_balance: u128 = original_unbondings.values().copied().sum();
    assert_eq!(original_unbondings.len() as u32, 1);  
    assert_eq!(original_ledger_balance, amount);  

    /// Update the coldkey to unused key
    //
    //
    // Coldkey = account_n
    // Hotkey  = account_n+1
    //
    //

    // Updating coldkey to account_n+1
    assert_ok!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(account_n)),
        account(account_n),
        account(account_n+1), // new_coldkey
      )
    );

    // check old coldkey balance is now removed because it was swapped to the new one
    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n));
    let ledger_balance: u128 = unbondings.values().copied().sum();
    assert_eq!(unbondings.len() as u32, 0);  
    assert_eq!(ledger_balance, 0);  

    // check new coldkey balance matches original
    let new_unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(account_n+1));
    let new_ledger_balance: u128 = new_unbondings.values().copied().sum();
    assert_eq!(new_unbondings.len() as u32, original_unbondings.len() as u32);  
    assert_eq!(new_ledger_balance, original_ledger_balance);  

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, account(account_n));

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, account(account_n));

    let key_owner = HotkeyOwner::<Test>::get(account(account_n));
    assert_eq!(key_owner, account(account_n+1));

    // Cold key is updated, shouldn't be able to make changes anywhere using coldkey

    let add_stake_amount: u128 = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(account_n), add_stake_amount);

    assert_err!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        hotkey_subnet_node_id,
        account(account_n),
        add_stake_amount,
      ),
      Error::<Test>::NotKeyOwner,
    );

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n)),
        subnet_id,
        account(account_n),
        1000,
      ),
      Error::<Test>::NotKeyOwner
    );    
    
    // `do_deactivate_subnet_node` allows both hotkey and coldkey
    assert_err!(
      Network::do_deactivate_subnet_node_new(
        RuntimeOrigin::signed(account(account_n+10)),
        subnet_id,
        hotkey_subnet_node_id
      ),
      Error::<Test>::NotKeyOwner
    );

    assert_err!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(account_n)),
        account(2),
        account(account_n+1), // new_coldkey
      ),
      Error::<Test>::NotKeyOwner
    );

    // new hotkey is 2
    assert_err!(
      Network::update_hotkey(
        RuntimeOrigin::signed(account(account_n)),
        account(2),
        account(account_n+1),
      ),
      Error::<Test>::NotKeyOwner
    );


    // Use new coldkey
    let add_stake_amount: u128 = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(account_n+1), add_stake_amount + 500);

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(account_n+1)),
        subnet_id,
        hotkey_subnet_node_id,
        account(account_n),
        add_stake_amount,
      )
    );

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(account_n+1)),
        subnet_id,
        account(account_n),
        add_stake_amount,
      )
    );

    // `do_deactivate_subnet_node` allows both hotkey and coldkey
    assert_ok!(
      Network::do_deactivate_subnet_node_new(
        RuntimeOrigin::signed(account(account_n+1)),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    assert_ok!(
      Network::update_hotkey(
        RuntimeOrigin::signed(account(account_n+1)),
        account(account_n),     // old_hotkey
        account(account_n+15),  // new hotkey
      )
    );

    assert_ok!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(account_n+1)),
        account(account_n+15),
        account(account_n+2), // new_coldkey
      )
    );

    assert_err!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(account_n+1)),
        account(account_n+15),
        account(account_n+2), // new_coldkey
      ),
      Error::<Test>::NotKeyOwner
    );    
  })
}

#[test]
fn test_update_coldkey_key_taken_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let end = 16;

    let account_n = max_subnet_nodes+end*subnets;

    build_activated_subnet_new(subnet_path.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    assert_err!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(account_n)),
        account(2),
        account(account_n),
      ),
      Error::<Test>::NotKeyOwner
    );
  });
}

#[test]
fn test_update_hotkey() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let end = 16;

    let account_n = max_subnet_nodes+end*subnets;

    build_activated_subnet_new(subnet_path.clone(), 0, end, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(account_n)).unwrap();
    let starting_account_subnet_stake = AccountSubnetStake::<Test>::get(account(account_n), subnet_id);

    assert_ok!(
      Network::update_hotkey(
        RuntimeOrigin::signed(account(account_n)),
        account(account_n),
        account(account_n+1000),
      )
    );

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, account(account_n+1000));

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, account(account_n+1000));

    let key_owner = HotkeyOwner::<Test>::get(account(account_n+1000));
    assert_eq!(key_owner, account(account_n));

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(account_n), subnet_id);
    assert_eq!(account_subnet_stake, 0);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(account_n+1000), subnet_id);
    assert_eq!(account_subnet_stake, starting_account_subnet_stake);
  })
}
