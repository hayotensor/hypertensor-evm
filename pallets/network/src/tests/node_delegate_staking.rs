use super::mock::*;
use crate::tests::test_utils::*;
use crate::{
    AccountNodeDelegateStakeShares, AccountSubnetDelegateStakeShares, Error, MaxSubnetNodes,
    MinDelegateStakeDeposit, MinSubnetMinStake, NextSwapQueueId, NodeDelegateStakeBalance,
    NodeDelegateStakeCooldownEpochs, QueuedSwapCall, StakeUnbondingLedger, SubnetName,
    SwapCallQueue, SwapQueueOrder, TotalActiveSubnets, TotalNodeDelegateStake,
    TotalNodeDelegateStakeShares, TotalSubnetNodes,
};
use frame_support::traits::Currency;
use frame_support::{assert_err, assert_ok};
use sp_std::collections::btree_map::BTreeMap;

//
//
//
//
//
//
//
// Node delegate staking
//
//
//
//
//
//
//

#[test]
fn test_add_to_node_delegate_stake() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 100e+18 as u128;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet_with_delegator_rewards(
            subnet_name.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let prev_account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get(
            (account(total_subnet_nodes + 1), subnet_id, 0),
        );

        let mut prev_total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, 0);

        let prev_total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, 0);
        let prev_total_node_delegate_stake = TotalNodeDelegateStake::<Test>::get();

        let mut node_delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
            amount,
            prev_total_node_delegate_stake_shares,
            prev_total_node_delegate_stake_balance,
        );

        let _ = Balances::deposit_creating(&account(total_subnet_nodes + 1), amount + 500);

        let subnet_node_id = 1;

        assert_ok!(Network::add_to_node_delegate_stake(
            RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
            subnet_id,
            subnet_node_id,
            amount,
        ));

        // Ensure user shares changed and is expected
        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(total_subnet_nodes + 1),
            subnet_id,
            subnet_node_id,
        ));
        assert_eq!(
            account_node_delegate_stake_shares,
            node_delegate_stake_to_be_added_as_shares
        );

        // Ensure node balance changed and is expected
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        assert_eq!(
            total_node_delegate_stake_balance,
            prev_total_node_delegate_stake_balance + amount
        );

        // Ensure network balance changed and is expected
        let total_node_delegate_stake = TotalNodeDelegateStake::<Test>::get();
        assert_eq!(
            total_node_delegate_stake,
            prev_total_node_delegate_stake + amount
        );

        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);

        let account_node_delegate_stake_balance = Network::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        // Ensure user balance changed and is expected
        assert!(
            (account_node_delegate_stake_balance
                >= Network::percent_mul(amount, 990000000000000000))
                && (account_node_delegate_stake_balance <= amount)
        );
    })
}

#[test]
fn test_add_to_node_delegate_stake_min_node_delegate_stake_deposit_not_reached_error() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 100e+18 as u128;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet_with_delegator_rewards(
            subnet_name.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let prev_account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get(
            (account(total_subnet_nodes + 1), subnet_id, 0),
        );

        let mut prev_total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, 0);

        let prev_total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, 0);
        let prev_total_node_delegate_stake = TotalNodeDelegateStake::<Test>::get();

        let mut node_delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
            amount,
            prev_total_node_delegate_stake_shares,
            prev_total_node_delegate_stake_balance,
        );

        let _ = Balances::deposit_creating(&account(total_subnet_nodes + 1), amount + 500);

        let subnet_node_id = 1;

        assert_err!(
            Network::add_to_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id,
                MinDelegateStakeDeposit::<Test>::get() - 1,
            ),
            Error::<Test>::MinNodeDelegateStakeDepositNotReached
        );
    })
}

#[test]
fn test_add_to_node_delegate_stake_not_enough_balance_to_stake_error() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 100e+18 as u128;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet_with_delegator_rewards(
            subnet_name.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let subnet_node_id = 1;

        let prev_account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get(
            (account(total_subnet_nodes + 1), subnet_id, subnet_node_id),
        );

        let mut prev_total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);

        let prev_total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        let prev_total_node_delegate_stake = TotalNodeDelegateStake::<Test>::get();

        let mut node_delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
            amount,
            prev_total_node_delegate_stake_shares,
            prev_total_node_delegate_stake_balance,
        );

        let _ = Balances::deposit_creating(&account(total_subnet_nodes + 1), amount + 500);

        assert_err!(
            Network::add_to_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id,
                amount + 501,
            ),
            Error::<Test>::NotEnoughBalanceToStake
        );
    })
}

#[test]
fn test_add_to_node_delegate_stake_balance_withdraw_error() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 100e+18 as u128;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet_with_delegator_rewards(
            subnet_name.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let subnet_node_id = 1;

        let prev_account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get(
            (account(total_subnet_nodes + 1), subnet_id, subnet_node_id),
        );

        let mut prev_total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);

        let prev_total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        let prev_total_node_delegate_stake = TotalNodeDelegateStake::<Test>::get();

        let mut node_delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
            amount,
            prev_total_node_delegate_stake_shares,
            prev_total_node_delegate_stake_balance,
        );

        let _ = Balances::deposit_creating(&account(total_subnet_nodes + 1), amount + 500);

        assert_err!(
            Network::add_to_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id,
                amount + 499,
            ),
            Error::<Test>::BalanceWithdrawalError
        );
    })
}

#[test]
fn test_remove_node_delegate_stake() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet_with_delegator_rewards(
            subnet_name.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        let subnet_node_id = 1;

        let subnet_name_2: Vec<u8> = "subnet-name-2".into();
        build_activated_subnet_with_delegator_rewards(
            subnet_name_2.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );
        let subnet_id_2 = SubnetName::<Test>::get(subnet_name_2.clone()).unwrap();

        let _ = Balances::deposit_creating(&account(total_subnet_nodes + 1), amount + 500);

        assert_ok!(Network::add_to_node_delegate_stake(
            RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
            subnet_id,
            subnet_node_id,
            amount,
        ));

        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(total_subnet_nodes + 1),
            subnet_id,
            subnet_node_id,
        ));
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);

        let account_node_delegate_stake_balance = Network::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        assert!(
            (account_node_delegate_stake_balance
                >= Network::percent_mul(amount, 990000000000000000))
                && (account_node_delegate_stake_balance <= amount)
        );

        let account_node_delegate_stake_shares_to_be_removed =
            account_node_delegate_stake_shares / 2;

        let expected_balance_to_be_removed = Network::convert_to_balance(
            account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        let expected_post_balance = Network::convert_to_balance(
            account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_balance - expected_balance_to_be_removed,
        );

        let epoch = System::block_number() / EpochLength::get();
        let block = System::block_number();

        assert_ok!(Network::remove_node_delegate_stake(
            RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
            subnet_id,
            subnet_node_id,
            account_node_delegate_stake_shares_to_be_removed,
        ));

        assert_err!(
            Network::remove_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id,
                0,
            ),
            Error::<Test>::SharesZero
        );

        assert_err!(
            Network::swap_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id, // unstaking from node 1
                subnet_id_2,
                subnet_node_id + 1,
                0,
            ),
            Error::<Test>::SharesZero
        );

        assert_err!(
            Network::swap_from_node_to_subnet(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id,
                subnet_id_2,
                0,
            ),
            Error::<Test>::SharesZero
        );

        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(total_subnet_nodes + 1),
            subnet_id,
            subnet_node_id,
        ));
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);

        assert_eq!(
            account_node_delegate_stake_shares,
            account_node_delegate_stake_shares_to_be_removed
        );

        let post_account_node_delegate_stake_balance = Network::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        // Ensure expected balance
        assert_eq!(
            expected_post_balance,
            post_account_node_delegate_stake_balance
        );

        // Ensure stake in ledger
        let unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes + 1));
        assert_eq!(unbondings.len(), 1);
        let (ledger_block, ledger_balance) = unbondings.iter().next().unwrap();
        assert_eq!(
            *ledger_block,
            &block + NodeDelegateStakeCooldownEpochs::<Test>::get() * EpochLength::get()
        );
        assert_eq!(*ledger_balance, expected_balance_to_be_removed);
    })
}

#[test]
fn test_remove_node_delegate_stake_not_enough_stake_to_withdraw() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet_with_delegator_rewards(
            subnet_name.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        let subnet_node_id = 1;

        let subnet_name_2: Vec<u8> = "subnet-name-2".into();
        build_activated_subnet_with_delegator_rewards(
            subnet_name_2.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let subnet_id_2 = SubnetName::<Test>::get(subnet_name_2.clone()).unwrap();

        assert_err!(
            Network::remove_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id,
                amount,
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        assert_err!(
            Network::swap_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id, // unstaking from node 1
                subnet_id_2,
                subnet_node_id + 1,
                amount,
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        assert_err!(
            Network::swap_from_node_to_subnet(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id,
                subnet_id_2,
                amount,
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    })
}

#[test]
fn test_swap_node_delegate_stake() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        let from_account_n = max_subnet_nodes + 1 * subnets;

        build_activated_subnet_with_delegator_rewards(
            subnet_name.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let from_subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_from_subnet_nodes = TotalSubnetNodes::<Test>::get(from_subnet_id);

        let to_subnet_name: Vec<u8> = "to-subnet-name".into();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;

        let to_account_n = max_subnet_nodes + 1 * subnets;

        build_activated_subnet_with_delegator_rewards(
            to_subnet_name.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let to_subnet_id = SubnetName::<Test>::get(to_subnet_name.clone()).unwrap();

        let _ = Balances::deposit_creating(&account(total_from_subnet_nodes + 1), amount + 500);

        assert_ok!(Network::add_to_node_delegate_stake(
            RuntimeOrigin::signed(account(total_from_subnet_nodes + 1)),
            from_subnet_id,
            1,
            amount,
        ));

        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(total_from_subnet_nodes + 1),
            from_subnet_id,
            1,
        ));
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(from_subnet_id, 1);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(from_subnet_id, 1);

        let account_node_delegate_stake_balance = Network::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        assert!(
            (account_node_delegate_stake_balance
                >= Network::percent_mul(amount, 990000000000000000))
                && (account_node_delegate_stake_balance <= amount)
        );

        let account_node_delegate_stake_shares_to_be_removed =
            account_node_delegate_stake_shares / 2;
        let expected_node_delegate_stake_shares_balance =
            account_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed;

        // Get expected balance to be removed from subnet node 1
        let expected_balance_to_be_removed = Network::convert_to_balance(
            account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        // Get expected balance after removal from subnet node 1
        let expected_post_balance = Network::convert_to_balance(
            account_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_balance - expected_balance_to_be_removed,
        );
        let unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(account(total_from_subnet_nodes + 1));
        assert_eq!(unbondings.len(), 0);

        let pre_transfer_balance = Balances::free_balance(&account(total_from_subnet_nodes + 1));
        let prev_next_id = NextSwapQueueId::<Test>::get();

        let starting_to_subnet_node_id = 2;

        assert_ok!(Network::swap_node_delegate_stake(
            RuntimeOrigin::signed(account(total_from_subnet_nodes + 1)),
            from_subnet_id,
            1, // unstaking from node 1
            to_subnet_id,
            starting_to_subnet_node_id,
            account_node_delegate_stake_shares_to_be_removed,
        ));

        let post_transfer_balance = Balances::free_balance(&account(total_from_subnet_nodes + 1));
        assert_eq!(pre_transfer_balance, post_transfer_balance);

        //
        // from subnet ID and Subnet node 1
        // Get accounts delegate stake info from staking to node 1 (now removed partial)
        //
        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(total_from_subnet_nodes + 1),
            from_subnet_id,
            1,
        ));
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(from_subnet_id, 1);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(from_subnet_id, 1);

        let account_node_delegate_stake_balance = Network::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        assert_eq!(account_node_delegate_stake_balance, expected_post_balance);

        //
        // Check queue
        //
        let starting_to_subnet_id = to_subnet_id;
        let call_queue = SwapCallQueue::<Test>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake {
                account_id,
                to_subnet_id,
                balance,
            } => assert!(false),
            QueuedSwapCall::SwapToNodeDelegateStake {
                account_id,
                to_subnet_id,
                to_subnet_node_id,
                balance,
            } => {
                assert_eq!(*account_id, account(total_from_subnet_nodes + 1));
                assert_eq!(*to_subnet_id, starting_to_subnet_id);
                assert_eq!(*to_subnet_node_id, starting_to_subnet_node_id);
                assert_ne!(*balance, 0);
            }
        };

        let next_id = NextSwapQueueId::<Test>::get();
        assert_eq!(prev_next_id + 1, next_id);
        let queue = SwapQueueOrder::<Test>::get();
        assert!(queue
            .first()
            .map_or(false, |&first_id| first_id == prev_next_id));

        //
        // to subnet ID and Subnet node 2
        // Get accounts delegate stake info transferred to node 2 (now staked to)
        //

        // let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((
        //     account(total_from_subnet_nodes + 1),
        //     to_subnet_id,
        //     2,
        // ));
        // let total_node_delegate_stake_balance =
        //     NodeDelegateStakeBalance::<Test>::get(to_subnet_id, 2);
        // let total_node_delegate_stake_shares =
        //     TotalNodeDelegateStakeShares::<Test>::get(to_subnet_id, 2);

        // let account_node_delegate_stake_balance = Network::convert_to_balance(
        //     account_node_delegate_stake_shares,
        //     total_node_delegate_stake_shares,
        //     total_node_delegate_stake_balance,
        // );

        // assert_ne!(account_node_delegate_stake_balance, 0);

        // assert!(
        //     (account_node_delegate_stake_balance
        //         >= Network::percent_mul(expected_balance_to_be_removed, 990000000000000000))
        //         && (account_node_delegate_stake_balance <= expected_balance_to_be_removed)
        // );

        // // Ensure the code didn't create an unbonding insert
        // let unbondings: BTreeMap<u32, u128> =
        //     StakeUnbondingLedger::<Test>::get(account(total_from_subnet_nodes + 1));
        // assert_eq!(unbondings.len(), 0);
    })
}

#[test]
fn test_transfer_node_delegate_stake() {
    new_test_ext().execute_with(|| {
        let _ = env_logger::builder().is_test(true).try_init();

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnet_name: Vec<u8> = "subnet-name".into();
        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let subnet_node_id = 1;

        let n_account = 1000;
        let to_n_account = 1001;

        let _ = Balances::deposit_creating(&account(n_account), amount + 500);

        assert_ok!(Network::add_to_node_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            subnet_id,
            subnet_node_id,
            amount,
        ));

        let n_account_balance = Balances::free_balance(&account(n_account));
        let to_n_account_balance = Balances::free_balance(&account(to_n_account));

        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(n_account),
            subnet_id,
            subnet_node_id,
        ));
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);

        let account_node_delegate_stake_balance = Network::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        assert!(
            (account_node_delegate_stake_balance
                >= Network::percent_mul(amount, 990000000000000000))
                && (account_node_delegate_stake_balance <= amount)
        );

        let to_delegate_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(to_n_account),
            subnet_id,
            subnet_node_id,
        ));

        assert_eq!(to_delegate_shares, 0);

        assert_ok!(Network::transfer_node_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            subnet_id,
            subnet_node_id,
            account(to_n_account),
            account_node_delegate_stake_shares,
        ));

        // no changes to balance
        let after_n_account_balance = Balances::free_balance(&account(n_account));
        assert_eq!(n_account_balance, after_n_account_balance);
        let after_to_n_account_balance = Balances::free_balance(&account(to_n_account));
        assert_eq!(to_n_account_balance, after_to_n_account_balance);

        // no ledger balances
        let n_account_unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(account(n_account));
        assert_eq!(n_account_unbondings.len(), 0);
        let to_n_account_unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(account(to_n_account));
        assert_eq!(to_n_account_unbondings.len(), 0);

        // from shares
        let after_delegate_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(n_account),
            subnet_id,
            subnet_node_id,
        ));

        // to shares
        let after_to_node_delegate_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(to_n_account),
            subnet_id,
            subnet_node_id,
        ));

        let after_total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);
        let after_total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);

        assert_eq!(after_delegate_shares, 0);
        assert_eq!(
            account_node_delegate_stake_shares,
            after_to_node_delegate_shares
        );
        assert_eq!(
            total_node_delegate_stake_shares,
            after_total_node_delegate_stake_shares
        );
        assert_eq!(
            total_node_delegate_stake_balance,
            after_total_node_delegate_stake_balance
        );
    });
}

#[test]
fn test_transfer_node_delegate_stake_partial_balance() {
    new_test_ext().execute_with(|| {
        let _ = env_logger::builder().is_test(true).try_init();

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnet_name: Vec<u8> = "subnet-name".into();
        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let subnet_node_id = 1;

        let n_account = 1000;
        let to_n_account = 1001;

        let _ = Balances::deposit_creating(&account(n_account), amount + 500);

        assert_ok!(Network::add_to_node_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            subnet_id,
            subnet_node_id,
            amount,
        ));

        let n_account_balance = Balances::free_balance(&account(n_account));
        let to_n_account_balance = Balances::free_balance(&account(to_n_account));

        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(n_account),
            subnet_id,
            subnet_node_id,
        ));
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);

        let account_node_delegate_stake_balance = Network::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        assert!(
            (account_node_delegate_stake_balance
                >= Network::percent_mul(amount, 990000000000000000))
                && (account_node_delegate_stake_balance <= amount)
        );

        let to_delegate_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(to_n_account),
            subnet_id,
            subnet_node_id,
        ));

        assert_eq!(to_delegate_shares, 0);

        let shares_to_transfer = account_node_delegate_stake_shares / 2;

        assert_ok!(Network::transfer_node_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            subnet_id,
            subnet_node_id,
            account(to_n_account),
            shares_to_transfer,
        ));

        // no changes to balance
        let after_n_account_balance = Balances::free_balance(&account(n_account));
        assert_eq!(n_account_balance, after_n_account_balance);
        let after_to_n_account_balance = Balances::free_balance(&account(to_n_account));
        assert_eq!(to_n_account_balance, after_to_n_account_balance);

        // no ledger balances
        let n_account_unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(account(n_account));
        assert_eq!(n_account_unbondings.len(), 0);
        let to_n_account_unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(account(to_n_account));
        assert_eq!(to_n_account_unbondings.len(), 0);

        // from shares
        let after_delegate_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(n_account),
            subnet_id,
            subnet_node_id,
        ));

        // to shares
        let after_to_node_delegate_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(to_n_account),
            subnet_id,
            subnet_node_id,
        ));

        let after_total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);
        let after_total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);

        assert_eq!(
            after_delegate_shares,
            account_node_delegate_stake_shares - shares_to_transfer
        );
        assert_eq!(shares_to_transfer, after_to_node_delegate_shares);
        assert_eq!(
            total_node_delegate_stake_shares,
            after_total_node_delegate_stake_shares
        );
        assert_eq!(
            total_node_delegate_stake_balance,
            after_total_node_delegate_stake_balance
        );
    });
}

#[test]
fn test_inflation_exploit_mitigation_dead_shares() {
    new_test_ext().execute_with(|| {
        let subnet_id = 1;
        let subnet_node_id = 1;
        let first_user = account(1);
        let second_user = account(2);
        let stake = 1_000_000_000_000;

        // Give both users balances to stake
        Balances::deposit_creating(&first_user, stake * 10);
        Balances::deposit_creating(&second_user, stake * 10);

        // First user delegates stake
        Network::do_add_node_delegate_stake(
            RuntimeOrigin::signed(first_user.clone()),
            subnet_id,
            subnet_node_id,
            stake,
        );

        // Get shares after first stake

        let first_user_shares =
            AccountNodeDelegateStakeShares::<Test>::get((&first_user, subnet_id, subnet_node_id));
        let total_shares_after_first =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);

        // Ensure that shares given are less than 100% of total because of pre-injected 1000 shares
        assert!(first_user_shares < total_shares_after_first);

        // Second user adds same stake
        Network::do_add_node_delegate_stake(
            RuntimeOrigin::signed(second_user.clone()),
            subnet_id,
            subnet_node_id,
            stake,
        );

        // Get second user shares
        let second_user_shares =
            AccountNodeDelegateStakeShares::<Test>::get((&second_user, subnet_id, subnet_node_id));
        let total_shares_after_both =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);
        let total_balance_after_both =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);

        log::error!("first_user_shares  {:?}", first_user_shares);
        log::error!("second_user_shares {:?}", second_user_shares);

        // Check that second user also received a fair share
        assert!(second_user_shares > 0);
        assert!(first_user_shares <= second_user_shares);

        let first_user_balance = Network::convert_to_balance(
            first_user_shares,
            total_shares_after_both,
            total_balance_after_both,
        );

        let second_user_balance = Network::convert_to_balance(
            second_user_shares,
            total_shares_after_both,
            total_balance_after_both,
        );

        log::error!("first_user_balance  {:?}", first_user_balance);
        log::error!("second_user_balance {:?}", second_user_balance);

        assert!(first_user_balance < second_user_balance);

        // Check that total shares increased correctly
        assert_eq!(
            first_user_shares + second_user_shares + 1000,
            total_shares_after_both
        );
    });
}

#[test]
fn test_no_inflation_exploit_via_increase_delegate_stake() {
    new_test_ext().execute_with(|| {
        let subnet_id = 1;
        let subnet_node_id = 1;
        let attacker = account(1);
        let initial_balance = 1_000_000;
        let stake_amount = 100_000;
        let reward_amount = 100_000;

        // Step 0: Fund attacker
        Balances::make_free_balance_be(&attacker, initial_balance);

        // Step 1: Attacker stakes
        assert_ok!(Network::do_add_node_delegate_stake(
            RuntimeOrigin::signed(attacker.clone()),
            subnet_id,
            subnet_node_id,
            stake_amount
        ));

        let shares_before =
            AccountNodeDelegateStakeShares::<Test>::get((&attacker, subnet_id, subnet_node_id));
        let shares_total_before =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);
        let pool_balance_before = NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        assert!(shares_before > 0);
        assert!(shares_total_before > 0);
        assert!(pool_balance_before > 0);

        // Step 2: Attacker deposits reward (donation-style increase)
        Network::do_increase_node_delegate_stake(subnet_id, subnet_node_id, reward_amount);

        // Step 3: Check that no new shares were minted
        let shares_after_reward =
            AccountNodeDelegateStakeShares::<Test>::get((&attacker, subnet_id, subnet_node_id));
        let shares_total_after_reward =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);
        let pool_balance_before = NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);

        assert_eq!(shares_after_reward, shares_before);
        assert_eq!(shares_total_after_reward, shares_total_before);

        // Step 4: Unstake all
        assert_ok!(Network::do_remove_node_delegate_stake(
            RuntimeOrigin::signed(attacker.clone()),
            subnet_id,
            subnet_node_id,
            shares_after_reward
        ));

        // Step 5: Check final balance — should not exceed stake + reward
        let final_balance = Balances::free_balance(&attacker);
        let expected_max_balance = initial_balance; // he started with this

        // attacker should never receive more than they fairly deserve
        assert!(final_balance <= expected_max_balance + reward_amount);

        // In fact, he should end up with exactly stake + reward back
        assert!(final_balance <= initial_balance); // restaked and unstaked exactly once, reward goes to share value
    });
}

// ——————————————————————————————————————————————————————————————
// ERC‑4626 Donation Attack Scenario:
//
// 1) totalAssets=0, totalShares=0
// 2) Attacker deposits 1 → totalAssets=1, totalShares=1
// 3) Attacke "donates" 10_000 via do_increase_delegate_stake
//    → totalAssets=10_001, totalShares=1
// 4) Innocent LP deposits 10_000 → would mint
//    floor(10_000 * 1 / 10_001) = 0 shares
//    → WITHOUT mitigation: they get 0 shares silently
//    → WITH our mitigation: we detect zero shares and return Err(CouldNotConvertToShares)
//
// Inflation exploits are mitigated via:
//  - Min deposit of 1000 TENSOR
//  - minting of dead shares when at zero shares
//  - use of virtual shares using decimal offset is converting assets/shares
//
//
// ——————————————————————————————————————————————————————————————
#[test]
fn test_donation_attack_simulation() {
    new_test_ext().execute_with(|| {
        let _ = env_logger::builder().is_test(true).try_init();

        let subnet_id = 1;
        let subnet_node_id = 1;

        let attacker = account(1);
        let victim = account(2);

        // Initial attacker tokens
        // const ATTACKER_INITIAL_TOKENS: u128 = 10000;
        const ATTACKER_INITIAL_TOKENS: u128 = 10000000;
        // Small amount to initially deposit
        // const ATTACKER_INITIAL_DEPOSIT: u128 = 1;
        const ATTACKER_INITIAL_DEPOSIT: u128 = 1000;
        // Large amount to donate directly
        // const ATTACKER_DONATION: u128 = 9999;
        const ATTACKER_DONATION: u128 = 9999000;
        // Victim deposit amount
        // const VICTIM_DEPOSIT: u128 = 1000;
        const VICTIM_DEPOSIT: u128 = 1000000;

        Balances::make_free_balance_be(&attacker, ATTACKER_INITIAL_TOKENS);
        Balances::make_free_balance_be(&victim, VICTIM_DEPOSIT + 500);

        // ---- Step 1: Attacker deposits minimal amount ----
        // The MinDelegateStakeDeposit (deposit min) is 1000, otherwise reverts with CouldNotConvertToBalance
        assert_ok!(Network::do_add_node_delegate_stake(
            RuntimeOrigin::signed(attacker.clone()),
            subnet_id,
            subnet_node_id,
            ATTACKER_INITIAL_DEPOSIT,
        ));

        let total_subnet_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);
        let total_subnet_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);

        // Validate initial deposit
        let attacker_balance = Network::convert_to_balance(
            AccountNodeDelegateStakeShares::<Test>::get((&attacker, subnet_id, subnet_node_id)),
            total_subnet_delegate_stake_shares,
            total_subnet_delegate_stake_balance,
        );
        log::error!("attacker_balance         {:?}", attacker_balance);

        assert_eq!(
            AccountNodeDelegateStakeShares::<Test>::get((&attacker, subnet_id, subnet_node_id)),
            ATTACKER_INITIAL_DEPOSIT
        );
        // assert_eq!(TotalNodeDelegateStakeShares::<Test>::get(subnet_id), ATTACKER_INITIAL_DEPOSIT);
        // ---- We mint 1000 dead shares so we check against this
        assert_eq!(
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id),
            ATTACKER_INITIAL_DEPOSIT + 1000
        );
        assert_eq!(
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id),
            ATTACKER_INITIAL_DEPOSIT
        );

        // ---- Step 2: Attacker donates to inflate share price ----
        Network::do_increase_node_delegate_stake(subnet_id, subnet_node_id, ATTACKER_DONATION);

        // ---- Step 3: Victim deposits and gets almost no shares ----
        // We ensure they get shares
        assert_ok!(Network::do_add_node_delegate_stake(
            RuntimeOrigin::signed(victim.clone()),
            subnet_id,
            subnet_node_id,
            VICTIM_DEPOSIT,
        ));

        let victim_shares =
            AccountNodeDelegateStakeShares::<Test>::get((&victim, subnet_id, subnet_node_id));

        let total_subnet_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(subnet_id, subnet_node_id);
        let total_subnet_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);

        let victim_balance = Network::convert_to_balance(
            victim_shares,
            total_subnet_delegate_stake_shares,
            total_subnet_delegate_stake_balance,
        );

        assert!(
            (victim_balance >= Network::percent_mul(VICTIM_DEPOSIT, 990000000000000000))
                && (victim_balance <= VICTIM_DEPOSIT)
        );

        let attacker_balance = Network::convert_to_balance(
            AccountNodeDelegateStakeShares::<Test>::get((&attacker, subnet_id, subnet_node_id)),
            total_subnet_delegate_stake_shares,
            total_subnet_delegate_stake_balance,
        );

        assert!(attacker_balance < ATTACKER_INITIAL_DEPOSIT + ATTACKER_DONATION);

        // ---- Step 4: Attacker withdraws and gets profit ----
        // We ensure they do not profit from this attack
        assert_ok!(Network::do_remove_node_delegate_stake(
            RuntimeOrigin::signed(attacker.clone()),
            subnet_id,
            subnet_node_id,
            AccountNodeDelegateStakeShares::<Test>::get((&attacker, subnet_id, subnet_node_id))
        ));

        let attacker_final_balance = Balances::free_balance(&attacker);

        assert!(attacker_final_balance < ATTACKER_INITIAL_TOKENS);
    });
}

#[test]
fn test_donate_node_delegate_stake() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 1000000000000000000000000;
        let amount: u128 = 1000000000000000000000; // 1000
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let subnet_node_id = 1;
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let _ = Balances::deposit_creating(&account(total_subnet_nodes + 1), amount + 500);

        assert_err!(
            Network::donate_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                0,
                amount,
            ),
            Error::<Test>::InvalidSubnetNodeId
        );

        assert_err!(
            Network::donate_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id,
                0,
            ),
            Error::<Test>::MinDelegateStake
        );

        assert_err!(
            Network::donate_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id,
                amount + 501,
            ),
            Error::<Test>::NotEnoughBalance
        );

        assert_err!(
            Network::donate_node_delegate_stake(
                RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
                subnet_id,
                subnet_node_id,
                amount + 500,
            ),
            Error::<Test>::BalanceWithdrawalError
        );

        let prev_total_subnet_ndstake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        let prev_total_ndstake = TotalNodeDelegateStake::<Test>::get();

        assert_ok!(Network::donate_node_delegate_stake(
            RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
            subnet_id,
            subnet_node_id,
            amount,
        ));

        let total_subnet_dstake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        let total_ndstake = TotalNodeDelegateStake::<Test>::get();
        assert_eq!(
            total_subnet_dstake_balance,
            prev_total_subnet_ndstake_balance + amount
        );
        assert_eq!(total_ndstake, prev_total_ndstake + amount);

        // again

        let _ = Balances::deposit_creating(&account(total_subnet_nodes + 1), amount + 500);

        let prev_total_subnet_ndstake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        let prev_total_ndstake = TotalNodeDelegateStake::<Test>::get();

        assert_ok!(Network::donate_node_delegate_stake(
            RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
            subnet_id,
            subnet_node_id,
            amount,
        ));

        let total_subnet_ndstake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        let total_ndstake = TotalNodeDelegateStake::<Test>::get();
        assert_eq!(
            total_subnet_ndstake_balance,
            prev_total_subnet_ndstake_balance + amount
        );
        assert_eq!(total_ndstake, prev_total_ndstake + amount);
    });
}

#[test]
fn test_swap_from_node_to_subnet() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        log::error!("subnets count {:?}", subnets);
        build_activated_subnet_with_delegator_rewards(
            subnet_name.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let from_subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_from_subnet_nodes = TotalSubnetNodes::<Test>::get(from_subnet_id);

        let to_subnet_name: Vec<u8> = "subnet-name-2".into();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        log::error!("subnets count {:?}", subnets);

        build_activated_subnet_with_delegator_rewards(
            to_subnet_name.clone(),
            0,
            16,
            deposit_amount,
            stake_amount,
            DEFAULT_DELEGATE_REWARD_RATE,
        );

        let to_subnet_id = SubnetName::<Test>::get(to_subnet_name.clone()).unwrap();

        let _ = Balances::deposit_creating(&account(total_from_subnet_nodes + 1), amount + 500);

        assert_ok!(Network::add_to_node_delegate_stake(
            RuntimeOrigin::signed(account(total_from_subnet_nodes + 1)),
            from_subnet_id,
            1,
            amount,
        ));

        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(total_from_subnet_nodes + 1),
            from_subnet_id,
            1,
        ));
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(from_subnet_id, 1);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(from_subnet_id, 1);

        let account_node_delegate_stake_balance = Network::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        assert!(
            (account_node_delegate_stake_balance
                >= Network::percent_mul(amount, 990000000000000000))
                && (account_node_delegate_stake_balance < amount)
        );

        let account_node_delegate_stake_shares_to_be_removed =
            account_node_delegate_stake_shares / 2;

        let expected_balance_to_be_removed = Network::convert_to_balance(
            account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        let before_transfer_tensor = Balances::free_balance(&account(total_from_subnet_nodes + 1));

        let unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(account(total_from_subnet_nodes + 1));
        assert_eq!(unbondings.len(), 0);

        let prev_next_id = NextSwapQueueId::<Test>::get();

        assert_ok!(Network::swap_from_node_to_subnet(
            RuntimeOrigin::signed(account(total_from_subnet_nodes + 1)),
            from_subnet_id,
            1,
            to_subnet_id,
            account_node_delegate_stake_shares_to_be_removed,
        ));

        let unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(account(total_from_subnet_nodes + 1));
        assert_eq!(unbondings.len(), 0);

        let after_transfer_tensor = Balances::free_balance(&account(total_from_subnet_nodes + 1));
        assert_eq!(after_transfer_tensor, before_transfer_tensor);

        let from_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(
            account(total_from_subnet_nodes + 1),
            from_subnet_id,
        );
        assert_eq!(from_delegate_shares, 0);

        let starting_to_subnet_id = to_subnet_id;
        let call_queue = SwapCallQueue::<Test>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake {
                account_id,
                to_subnet_id,
                balance,
            } => {
                assert_eq!(*account_id, account(total_from_subnet_nodes + 1));
                assert_eq!(*to_subnet_id, starting_to_subnet_id);
                assert_ne!(*balance, 0);
            }
            QueuedSwapCall::SwapToNodeDelegateStake { .. } => assert!(false),
        };

        let next_id = NextSwapQueueId::<Test>::get();
        assert_eq!(prev_next_id + 1, next_id);
        let queue = SwapQueueOrder::<Test>::get();
        assert!(queue
            .first()
            .map_or(false, |&first_id| first_id == prev_next_id));
    });
}
