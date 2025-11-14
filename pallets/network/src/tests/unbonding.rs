use super::mock::*;
use crate::tests::test_utils::*;
use crate::{
    AccountNodeDelegateStakeShares, AccountSubnetDelegateStakeShares, AccountSubnetStake,
    DelegateStakeCooldownEpochs, Error, HotkeySubnetNodeId, MaxSubnetNodes, MaxSubnets,
    MaxUnbondings, MinSubnetMinStake, NodeDelegateStakeCooldownEpochs, RegisteredSubnetNodesData,
    StakeCooldownEpochs, StakeUnbondingLedger, SubnetName, SubnetNodeQueueEpochs,
    TotalActiveSubnets, TotalSubnetNodes,
};
use frame_support::traits::Currency;
use frame_support::weights::WeightMeter;
use frame_support::{assert_err, assert_ok};
use sp_std::collections::btree_map::BTreeMap;

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

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let burn_amount = Network::calculate_burn_amount(subnet_id);
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount + burn_amount + 500);

        let starting_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(starting_balance, deposit_amount + burn_amount + 500);

        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let stake_balance = AccountSubnetStake::<Test>::get(&hotkey.clone(), subnet_id);
        assert_eq!(stake_balance, amount);

        let after_stake_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(after_stake_balance, starting_balance - amount - burn_amount);

        assert_ok!(Network::remove_subnet_node(
            RuntimeOrigin::signed(hotkey.clone()),
            subnet_id,
            subnet_node_id,
        ));

        let stake_balance = AccountSubnetStake::<Test>::get(&hotkey.clone(), subnet_id);

        // remove amount ontop
        assert_ok!(Network::remove_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            stake_balance,
        ));

        assert_eq!(Network::account_subnet_stake(hotkey.clone(), 1), 0);

        let epoch_length = EpochLength::get();
        let epoch = System::block_number() / epoch_length;
        let block = System::block_number();

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(coldkey.clone());
        assert_eq!(unbondings.len(), 1);
        let (first_key, first_value) = unbondings.iter().next().unwrap();
        assert_eq!(
            *first_key,
            &block + StakeCooldownEpochs::<Test>::get() * EpochLength::get()
        );
        assert_eq!(*first_value, stake_balance);

        let stake_cooldown_epochs = StakeCooldownEpochs::<Test>::get();

        increase_epochs(stake_cooldown_epochs + 1);

        let epoch = System::block_number() / epoch_length;

        assert_ok!(Network::claim_unbondings(RuntimeOrigin::signed(
            coldkey.clone()
        )));

        // Check balance is in the wallet after unbonding
        let post_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(post_balance, starting_balance - burn_amount);

        // Check ledger removed the unbonding
        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(coldkey.clone());
        assert_eq!(unbondings.len(), 0);
    });
}

#[test]
fn test_register_remove_delegate_claim_stake_unbondings() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 1000000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let delegate_staker = account(999);
        let delegate_stake_amount = 1000e+18 as u128;
        let _ = Balances::deposit_creating(&delegate_staker.clone(), delegate_stake_amount + 500);

        let starting_balance = Balances::free_balance(&delegate_staker.clone());

        assert_ok!(Network::add_to_delegate_stake(
            RuntimeOrigin::signed(delegate_staker.clone()),
            subnet_id,
            delegate_stake_amount,
        ));

        let delegate_shares =
            AccountSubnetDelegateStakeShares::<Test>::get(delegate_staker.clone(), subnet_id);
        assert!(delegate_shares > 0);

        let after_stake_balance = Balances::free_balance(&delegate_staker.clone());
        assert_eq!(
            after_stake_balance,
            starting_balance - delegate_stake_amount
        );
        let block = System::block_number();

        let before_remove_balance = Balances::free_balance(&delegate_staker.clone());

        // remove
        assert_ok!(Network::remove_delegate_stake(
            RuntimeOrigin::signed(delegate_staker.clone()),
            subnet_id,
            delegate_shares,
        ));

        let unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(delegate_staker.clone());
        assert_eq!(unbondings.len(), 1);
        let (first_key, first_value) = unbondings.iter().next().unwrap();
        assert_eq!(
            *first_key,
            &block + DelegateStakeCooldownEpochs::<Test>::get() * EpochLength::get()
        );
        assert_ne!(*first_value, 0);

        let stake_cooldown_epochs = DelegateStakeCooldownEpochs::<Test>::get();

        increase_epochs(stake_cooldown_epochs + 1);

        assert_ok!(Network::claim_unbondings(RuntimeOrigin::signed(
            delegate_staker.clone()
        )));

        // Check balance is in the wallet after unbonding
        let post_balance = Balances::free_balance(&delegate_staker.clone());
        assert!(post_balance > before_remove_balance);

        // Check ledger removed the unbonding
        let unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(delegate_staker.clone());
        assert_eq!(unbondings.len(), 0);
    });
}

#[test]
fn test_register_remove_node_delegate_claim_stake_unbondings() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 1000000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end - 1);
        let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let delegate_staker = account(999);
        let delegate_stake_amount = 1000e+18 as u128;
        let _ = Balances::deposit_creating(&delegate_staker.clone(), delegate_stake_amount + 500);

        let starting_balance = Balances::free_balance(&delegate_staker.clone());

        assert_ok!(Network::add_to_node_delegate_stake(
            RuntimeOrigin::signed(delegate_staker.clone()),
            subnet_id,
            subnet_node_id,
            delegate_stake_amount,
        ));

        let delegate_shares = AccountNodeDelegateStakeShares::<Test>::get((
            delegate_staker.clone(),
            subnet_id,
            subnet_node_id,
        ));
        assert!(delegate_shares > 0);

        let after_stake_balance = Balances::free_balance(&delegate_staker.clone());
        assert_eq!(
            after_stake_balance,
            starting_balance - delegate_stake_amount
        );
        let block = System::block_number();

        let before_remove_balance = Balances::free_balance(&delegate_staker.clone());

        // remove
        assert_ok!(Network::remove_node_delegate_stake(
            RuntimeOrigin::signed(delegate_staker.clone()),
            subnet_id,
            subnet_node_id,
            delegate_shares,
        ));

        let unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(delegate_staker.clone());
        assert_eq!(unbondings.len(), 1);
        let (first_key, first_value) = unbondings.iter().next().unwrap();
        assert_eq!(
            *first_key,
            &block + NodeDelegateStakeCooldownEpochs::<Test>::get() * EpochLength::get()
        );
        assert_ne!(*first_value, 0);

        let stake_cooldown_epochs = NodeDelegateStakeCooldownEpochs::<Test>::get();

        increase_epochs(stake_cooldown_epochs + 1);

        assert_ok!(Network::claim_unbondings(RuntimeOrigin::signed(
            delegate_staker.clone()
        )));

        // Check balance is in the wallet after unbonding
        let post_balance = Balances::free_balance(&delegate_staker.clone());
        assert!(post_balance > before_remove_balance);

        // Check ledger removed the unbonding
        let unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(delegate_staker.clone());
        assert_eq!(unbondings.len(), 0);
    });
}

#[test]
fn test_register_activate_remove_claim_stake_unbondings() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 1000000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let burn_amount = Network::calculate_burn_amount(subnet_id);
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount + burn_amount + 500);

        let starting_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(starting_balance, deposit_amount + burn_amount + 500);

        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        let start_epoch = subnet_node.classification.start_epoch;

        let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let stake_balance = AccountSubnetStake::<Test>::get(&hotkey.clone(), subnet_id);
        assert_eq!(stake_balance, amount);

        let after_stake_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(after_stake_balance, starting_balance - amount - burn_amount);

        let queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id);

        let epoch = Network::get_current_epoch_as_u32();
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // increase to the nodes start epoch
        set_block_to_subnet_slot_epoch(subnet_epoch + queue_epochs + 2, subnet_id);

        let epoch = Network::get_current_epoch_as_u32();
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // Get subnet weights (nodes only activate from queue if there are weights)
        // Note: This means a subnet is active if it gets weights
        let _ = Network::handle_subnet_emission_weights(epoch);

        // Trigger the node activation
        Network::emission_step(
            &mut WeightMeter::new(),
            System::block_number(),
            Network::get_current_epoch_as_u32(),
            Network::get_current_subnet_epoch_as_u32(subnet_id),
            subnet_id,
        );

        assert_eq!(
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );

        assert_ok!(Network::remove_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            subnet_node_id,
        ));

        let stake_balance = AccountSubnetStake::<Test>::get(&hotkey.clone(), subnet_id);

        // remove amount ontop
        assert_ok!(Network::remove_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            stake_balance,
        ));

        assert_eq!(Network::account_subnet_stake(hotkey.clone(), 1), 0);

        let epoch_length = EpochLength::get();
        let epoch = System::block_number() / epoch_length;
        let block = System::block_number();

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(coldkey.clone());
        assert_eq!(unbondings.len(), 1);
        let (first_key, first_value) = unbondings.iter().next().unwrap();
        assert_eq!(
            *first_key,
            &block + StakeCooldownEpochs::<Test>::get() * EpochLength::get()
        );
        assert!(*first_value <= stake_balance);

        let stake_cooldown_epochs = StakeCooldownEpochs::<Test>::get();

        increase_epochs(stake_cooldown_epochs + 1);

        let epoch = System::block_number() / epoch_length;

        assert_ok!(Network::claim_unbondings(RuntimeOrigin::signed(
            coldkey.clone()
        )));

        let post_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(post_balance, starting_balance - burn_amount);

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(coldkey.clone());
        assert_eq!(unbondings.len(), 0);
    });
}

#[test]
fn test_remove_stake_twice_in_epoch() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 1000000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let burn_amount = Network::calculate_burn_amount(subnet_id);
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount + burn_amount + 500);

        let starting_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(starting_balance, deposit_amount + burn_amount + 500);

        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            stake_amount,
            None,
            None,
            u128::MAX
        ));

        let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let stake_balance = AccountSubnetStake::<Test>::get(&hotkey.clone(), subnet_id);
        assert_eq!(stake_balance, stake_amount);

        let after_stake_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(
            after_stake_balance,
            starting_balance - stake_amount - burn_amount
        );

        let _ = Balances::deposit_creating(&account(1), stake_amount * 2);

        assert_ok!(Network::add_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            subnet_node_id,
            hotkey.clone(),
            stake_amount * 3,
        ));

        let stake_balance = AccountSubnetStake::<Test>::get(&hotkey.clone(), subnet_id);
        assert_eq!(stake_balance, stake_amount + stake_amount * 3);

        let epoch = System::block_number() / EpochLength::get();
        let block = System::block_number();

        assert_ok!(Network::remove_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            stake_amount,
        ));

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(coldkey.clone());
        let ledger_balance: u128 = unbondings.values().copied().sum();
        assert_eq!(unbondings.len() as u32, 1);
        assert_eq!(ledger_balance, stake_amount);
        let (ledger_block, ledger_balance) = unbondings.iter().next().unwrap();
        assert_eq!(
            *ledger_block,
            &block + StakeCooldownEpochs::<Test>::get() * EpochLength::get()
        );

        assert_ok!(Network::remove_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            stake_amount,
        ));

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(coldkey.clone());
        let ledger_balance: u128 = unbondings.values().copied().sum();
        assert_eq!(unbondings.len() as u32, 1);
        assert_eq!(ledger_balance, stake_amount * 2);
        let (ledger_block, ledger_balance) = unbondings.iter().next().unwrap();
        assert_eq!(
            *ledger_block,
            &block + StakeCooldownEpochs::<Test>::get() * EpochLength::get()
        );

        increase_epochs(1);

        let epoch = System::block_number() / EpochLength::get();
        let block = System::block_number();

        assert_ok!(Network::remove_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            stake_amount,
        ));

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(coldkey.clone());
        let total_ledger_balance: u128 = unbondings.values().copied().sum();
        assert_eq!(unbondings.len() as u32, 2);
        assert_eq!(total_ledger_balance, stake_amount * 3);
        let (ledger_block, ledger_balance) = unbondings.iter().last().unwrap();
        assert_eq!(
            *ledger_block,
            &block + StakeCooldownEpochs::<Test>::get() * EpochLength::get()
        );
        assert_eq!(*ledger_balance, stake_amount);

        System::set_block_number(
            System::block_number()
                + ((EpochLength::get() + 1) * StakeCooldownEpochs::<Test>::get()),
        );

        let starting_balance = Balances::free_balance(&coldkey.clone());

        assert_ok!(Network::claim_unbondings(RuntimeOrigin::signed(
            coldkey.clone()
        )));

        let ending_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(starting_balance + total_ledger_balance, ending_balance);

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(coldkey.clone());
        assert_eq!(unbondings.len(), 0);
    });
}

#[test]
fn test_claim_stake_unbondings_no_unbondings_err() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 1000000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let burn_amount = Network::calculate_burn_amount(subnet_id);
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount + burn_amount + 500);

        let starting_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(starting_balance, deposit_amount + burn_amount + 500);

        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            stake_amount,
            None,
            None,
            u128::MAX
        ));

        let stake_balance = AccountSubnetStake::<Test>::get(&hotkey.clone(), subnet_id);
        assert_eq!(stake_balance, stake_amount);

        let after_stake_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(
            after_stake_balance,
            starting_balance - stake_amount - burn_amount
        );

        assert_err!(
            Network::claim_unbondings(RuntimeOrigin::signed(coldkey.clone())),
            Error::<Test>::NoStakeUnbondingsOrCooldownNotMet
        );

        let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        assert_ok!(Network::add_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            subnet_node_id,
            hotkey.clone(),
            100,
        ));

        assert_ok!(Network::remove_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            100,
        ));

        // No cooldown, should have same error
        assert_err!(
            Network::claim_unbondings(RuntimeOrigin::signed(coldkey.clone())),
            Error::<Test>::NoStakeUnbondingsOrCooldownNotMet
        );
    });
}

#[test]
fn test_remove_to_stake_max_unlockings_reached_err() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let burn_amount = Network::calculate_burn_amount(subnet_id);
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount + burn_amount + 500);

        let starting_balance = Balances::free_balance(&coldkey.clone());

        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            stake_amount * 2,
            None,
            None,
            u128::MAX
        ));

        // let max_unlockings = MaxUnbondings::get();
        let max_unlockings = MaxUnbondings::<Test>::get();
        for n in 0..max_unlockings + 2 {
            let _n = n + 1;
            // increase_epochs(1);
            System::set_block_number(System::block_number() + 1);
            if _n > max_unlockings {
                assert_err!(
                    Network::remove_stake(
                        RuntimeOrigin::signed(coldkey.clone()),
                        subnet_id,
                        hotkey.clone(),
                        1000,
                    ),
                    Error::<Test>::MaxUnlockingsReached
                );
            } else {
                assert_ok!(Network::remove_stake(
                    RuntimeOrigin::signed(coldkey.clone()),
                    subnet_id,
                    hotkey.clone(),
                    1000,
                ));

                let unbondings: BTreeMap<u32, u128> =
                    StakeUnbondingLedger::<Test>::get(coldkey.clone());
                assert_eq!(unbondings.len() as u32, _n);
            }
        }
    });
}
