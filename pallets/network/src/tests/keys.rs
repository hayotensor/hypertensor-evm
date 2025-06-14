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
  SubnetRewardsValidator,
  SubnetPaths, 
  TotalSubnetNodes,
  SubnetNodeClass,
  SubnetsData,
  AccountSubnetStake,
  RegistrationSubnetData,
  StakeUnbondingLedger, 
  TotalSubnetStake, 
  MinSubnetRegistrationBlocks,
  DefaultSubnetNodeUniqueParamLimit,
  HotkeyOwner, 
  TotalSubnetNodeUids, 
  HotkeySubnetNodeId, 
  SubnetNodeIdHotkey, 
  SubnetNodesData, 
  PeerIdSubnetNode,
  DeactivationLedger, 
  SubnetNodeDeactivation, 
  MaxRewardRateDecrease,
  RewardRateUpdatePeriod,
  SubnetRegistrationEpochs,
  MinStakeBalance,
  RegisteredStakeCooldownEpochs,
  RegistrationQueue,
  ChurnLimit,
  RegistrationQueueEpochs,
};


#[test]
fn test_update_coldkey() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "subnet-name".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, 16, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();
    let starting_account_subnet_stake = AccountSubnetStake::<Test>::get(account(1), subnet_id);

    // add extra stake and then add to ledger to check if it swapped
    let add_stake_amount = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(1), deposit_amount);

    //
    //
    // Coldkey = 1
    // Hotkey  = 1
    //
    //

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        hotkey_subnet_node_id,
        account(1),
        add_stake_amount,
      )
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(1), subnet_id);
    assert_eq!(stake_balance, starting_account_subnet_stake + add_stake_amount);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        amount,
      )
    );

    let original_unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(1));
    let original_ledger_balance: u128 = original_unbondings.values().copied().sum();
    assert_eq!(original_unbondings.len() as u32, 1);  
    assert_eq!(original_ledger_balance, amount);  

    /// Update the coldkey to unused key
    //
    //
    // Coldkey = total_subnet_nodes+1
    // Hotkey  = 1
    //
    //

    assert_ok!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(1)),
        account(1),
        account(total_subnet_nodes+1),
      )
    );

    // check old coldkey balance is now removed because it was swapped to the new one
    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(1));
    let ledger_balance: u128 = unbondings.values().copied().sum();
    assert_eq!(unbondings.len() as u32, 0);  
    assert_eq!(ledger_balance, 0);  

    // check new coldkey balance matches original
    let new_unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    let new_ledger_balance: u128 = new_unbondings.values().copied().sum();
    assert_eq!(new_unbondings.len() as u32, original_unbondings.len() as u32);  
    assert_eq!(new_ledger_balance, original_ledger_balance);  

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, account(1));

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, account(1));

    let key_owner = HotkeyOwner::<Test>::get(account(1));
    assert_eq!(key_owner, account(total_subnet_nodes+1));

    // Cold key is updated, shouldn't be able to make changes anywhere using coldkey

    let add_stake_amount: u128 = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(1), add_stake_amount);

    assert_err!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        hotkey_subnet_node_id,
        account(1),
        add_stake_amount,
      ),
      Error::<Test>::NotKeyOwner,
    );

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        1000,
      ),
      Error::<Test>::NotKeyOwner
    );    
    
    // `do_deactivate_subnet_node` allows both hotkey and coldkey
    assert_err!(
      Network::do_deactivate_subnet_node_new(
        RuntimeOrigin::signed(account(2)),
        subnet_id,
        hotkey_subnet_node_id
      ),
      Error::<Test>::NotKeyOwner
    );

    assert_err!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(1)),
        account(2),
        account(total_subnet_nodes+1),
      ),
      Error::<Test>::NotKeyOwner
    );

    assert_err!(
      Network::update_hotkey(
        RuntimeOrigin::signed(account(1)),
        account(2),
        account(total_subnet_nodes+1),
      ),
      Error::<Test>::NotKeyOwner
    );


    // Use new coldkey
    let add_stake_amount: u128 = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), add_stake_amount + 500);

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        hotkey_subnet_node_id,
        account(1),
        add_stake_amount,
      )
    );

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(1),
        add_stake_amount,
      )
    );

    // `do_deactivate_subnet_node` allows both hotkey and coldkey
    assert_ok!(
      Network::do_deactivate_subnet_node_new(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    assert_ok!(
      Network::update_hotkey(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        account(1),
        account(total_subnet_nodes+15),
      )
    );

    assert_ok!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        account(total_subnet_nodes+15),
        account(total_subnet_nodes+2),
      )
    );

    assert_err!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        account(total_subnet_nodes+15),
        account(total_subnet_nodes+2),
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

    let n_peers = 8;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_new(subnet_path.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    assert_err!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(1)),
        account(2),
        account(1),
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

    build_activated_subnet_new(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();
    let starting_account_subnet_stake = AccountSubnetStake::<Test>::get(account(1), subnet_id);

    assert_ok!(
      Network::update_hotkey(
        RuntimeOrigin::signed(account(1)),
        account(1),
        account(total_subnet_nodes+1),
      )
    );

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, account(total_subnet_nodes+1));

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, account(total_subnet_nodes+1));

    let key_owner = HotkeyOwner::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(key_owner, account(1));

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(1), subnet_id);
    assert_eq!(account_subnet_stake, 0);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(account_subnet_stake, starting_account_subnet_stake);
  })
}
