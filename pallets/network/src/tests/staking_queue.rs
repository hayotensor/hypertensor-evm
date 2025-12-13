use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use crate::{
    AccountNodeDelegateStakeShares, AccountSubnetDelegateStakeShares, DelegateStakeCooldownEpochs,
    HotkeySubnetNodeId, MaxSubnetNodes, MaxSubnets, MinSubnetMinStake, NextSwapQueueId,
    QueuedSwapCall, QueuedSwapItem, StakeUnbondingLedger, SubnetName, SwapCallQueue,
    SwapQueueOrder, TotalNodeDelegateStakeBalance, TotalNodeDelegateStakeShares,
    TotalSubnetDelegateStakeBalance, TotalSubnetDelegateStakeShares,
};
use frame_support::assert_ok;
use frame_support::traits::Currency;
use frame_support::weights::WeightMeter;

//
//
//
//
//
//
//
// Staking queue
//
//
//
//
//
//
//

fn insert_to_subnet_swap_call_queue(account_id: AccountIdOf<Test>, subnet_id: u32, balance: u128) {
    let id = NextSwapQueueId::<Test>::get();

    let call = QueuedSwapCall::SwapToSubnetDelegateStake {
        account_id: account_id,
        to_subnet_id: subnet_id,
        balance: balance,
    };

    let queued_item = QueuedSwapItem {
        id,
        call,
        queued_at_block: Network::get_current_block_as_u32(),
        execute_after_blocks: EpochLength::get(),
    };

    // Add to data storage
    SwapCallQueue::<Test>::insert(&id, &queued_item);

    // Add ID to the end of the queue
    SwapQueueOrder::<Test>::mutate(|queue| {
        let _ = queue.try_push(id); // Handle error if queue is full
    });

    NextSwapQueueId::<Test>::mutate(|next_id| *next_id = next_id.saturating_add(1));
}

fn insert_to_node_swap_call_queue(
    account_id: AccountIdOf<Test>,
    subnet_id: u32,
    subnet_node_id: u32,
    balance: u128,
) {
    let id = NextSwapQueueId::<Test>::get();

    let call = QueuedSwapCall::SwapToNodeDelegateStake {
        account_id: account_id,
        to_subnet_id: subnet_id,
        to_subnet_node_id: subnet_node_id,
        balance: balance,
    };

    let queued_item = QueuedSwapItem {
        id,
        call,
        queued_at_block: Network::get_current_block_as_u32(),
        execute_after_blocks: EpochLength::get(),
    };

    // Add to data storage
    SwapCallQueue::<Test>::insert(&id, &queued_item);

    // Add ID to the end of the queue
    SwapQueueOrder::<Test>::mutate(|queue| {
        let _ = queue.try_push(id); // Handle error if queue is full
    });

    NextSwapQueueId::<Test>::mutate(|next_id| *next_id = next_id.saturating_add(1));
}

#[test]
fn test_update_swap_queue_delegate_stake() {
    new_test_ext().execute_with(|| {
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let from_subnet_name: Vec<u8> = "subnet-name".into();
        build_activated_subnet(from_subnet_name.clone(), 0, 0, deposit_amount, stake_amount);
        let from_subnet_id = SubnetName::<Test>::get(from_subnet_name.clone()).unwrap();

        let to_subnet_name: Vec<u8> = "subnet-name-2".into();
        build_activated_subnet(to_subnet_name.clone(), 0, 0, deposit_amount, stake_amount);
        let to_subnet_id = SubnetName::<Test>::get(to_subnet_name.clone()).unwrap();

        let n_account = 255;

        let _ = Balances::deposit_creating(&account(n_account), amount + 500);

        let total_subnet_delegate_stake_shares =
            TotalSubnetDelegateStakeShares::<Test>::get(from_subnet_id);
        let total_subnet_delegate_stake_balance =
            TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);

        let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
            amount,
            total_subnet_delegate_stake_shares,
            total_subnet_delegate_stake_balance,
        );

        System::set_block_number(
            System::block_number()
                + DelegateStakeCooldownEpochs::<Test>::get() * EpochLength::get(),
        );

        let starting_delegator_balance = Balances::free_balance(&account(n_account));

        assert_ok!(Network::add_to_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            from_subnet_id,
            amount,
        ));

        let delegate_shares =
            AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), from_subnet_id);
        assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
        assert_ne!(delegate_shares, 0);

        let total_subnet_delegate_stake_shares =
            TotalSubnetDelegateStakeShares::<Test>::get(from_subnet_id);
        let total_subnet_delegate_stake_balance =
            TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);

        let mut from_delegate_balance = Network::convert_to_balance(
            delegate_shares,
            total_subnet_delegate_stake_shares,
            total_subnet_delegate_stake_balance,
        );
        // The first depositor will lose a percentage of their deposit depending on the size
        // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
        // assert_eq!(from_delegate_balance, delegate_stake_to_be_added_as_shares);

        let prev_total_subnet_delegate_stake_balance =
            TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);
        let prev_next_id = NextSwapQueueId::<Test>::get();

        assert_ok!(Network::swap_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            from_subnet_id,
            to_subnet_id,
            delegate_shares,
        ));

        // Check ledger doesn't have any unbondings and is empty
        assert!(StakeUnbondingLedger::<Test>::get(account(n_account)).is_empty());

        let from_delegate_shares =
            AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), from_subnet_id);
        assert_eq!(from_delegate_shares, 0);

        assert_ne!(
            prev_total_subnet_delegate_stake_balance,
            TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id)
        );
        assert!(
            prev_total_subnet_delegate_stake_balance
                > TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id)
        );

        // Check the queue
        let starting_to_subnet_id = to_subnet_id;
        let call_queue = SwapCallQueue::<Test>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake {
                account_id,
                to_subnet_id,
                balance,
            } => {
                assert_eq!(*account_id, account(n_account));
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

        // UPDATE

        // Update back to the `from_subnet_id` staying as a `SwapToSubnetDelegateStake`
        let call = QueuedSwapCall::SwapToSubnetDelegateStake {
            account_id: account(n_account),
            to_subnet_id: from_subnet_id,
            balance: u128::MAX,
        };

        assert_ok!(Network::update_swap_queue(
            RuntimeOrigin::signed(account(n_account)),
            prev_next_id,
            call.clone(),
        ));

        let event_exists = network_events().iter().any(|event| {
            matches!(event,
                Event::SwapCallQueueUpdated {
                    id: prev_next_id_val,
                    account_id: account_id_val,
                    call: QueuedSwapCall::SwapToSubnetDelegateStake {
                        account_id: account_id_val2,
                        to_subnet_id: from_subnet_id_val,
                        balance: _, // Ignore balance
                    }
                } if *prev_next_id_val == prev_next_id
                && *account_id_val == account(n_account)
                && *account_id_val2 == account(n_account)
                && *from_subnet_id_val == from_subnet_id
            )
        });
        assert!(event_exists);

        let call_queue = SwapCallQueue::<Test>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake {
                account_id,
                to_subnet_id,
                balance,
            } => {
                assert_eq!(*account_id, account(n_account));
                assert_eq!(*to_subnet_id, from_subnet_id);
                assert_ne!(*balance, 0);
                assert_ne!(*balance, u128::MAX);
            }
            QueuedSwapCall::SwapToNodeDelegateStake { .. } => assert!(false),
        };

        //
        // Update back to the `starting_to_subnet_id` with node ID as a `SwapToNodeDelegateStake`
        //
        let call = QueuedSwapCall::SwapToNodeDelegateStake {
            account_id: account(n_account),
            to_subnet_id: starting_to_subnet_id,
            to_subnet_node_id: 1,
            balance: u128::MAX,
        };

        assert_ok!(Network::update_swap_queue(
            RuntimeOrigin::signed(account(n_account)),
            prev_next_id,
            call.clone(),
        ));

        let event_exists = network_events().iter().any(|event| {
            matches!(event,
                Event::SwapCallQueueUpdated {
                    id: prev_next_id_val,
                    account_id: account_id_val,
                    call: QueuedSwapCall::SwapToSubnetDelegateStake {
                        account_id: account_id_val2,
                        to_subnet_id: from_subnet_id_val,
                        balance: _, // Ignore balance
                    }
                } if *prev_next_id_val == prev_next_id
                && *account_id_val == account(n_account)
                && *account_id_val2 == account(n_account)
                && *from_subnet_id_val == from_subnet_id
            )
        });
        assert!(event_exists);

        let call_queue = SwapCallQueue::<Test>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake { .. } => assert!(false),
            QueuedSwapCall::SwapToNodeDelegateStake {
                account_id,
                to_subnet_id,
                to_subnet_node_id,
                balance,
            } => {
                assert_eq!(*account_id, account(n_account));
                assert_eq!(*to_subnet_id, starting_to_subnet_id);
                assert_eq!(*to_subnet_node_id, 1);
                assert_ne!(*balance, 0);
                assert_ne!(*balance, u128::MAX);
            }
        };
    });
}

#[test]
fn test_update_swap_queue_node_delegate_stake() {
    new_test_ext().execute_with(|| {
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let end = 4;

        let from_subnet_name: Vec<u8> = "subnet-name".into();
        build_activated_subnet(
            from_subnet_name.clone(),
            0,
            end,
            deposit_amount,
            stake_amount,
        );
        let from_subnet_id = SubnetName::<Test>::get(from_subnet_name.clone()).unwrap();
        let from_subnet_node_id = 1;

        let to_subnet_name: Vec<u8> = "subnet-name-2".into();
        build_activated_subnet(to_subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let to_subnet_id = SubnetName::<Test>::get(to_subnet_name.clone()).unwrap();
        let to_subnet_node_id = 1;

        let n_account = 255;

        let _ = Balances::deposit_creating(&account(n_account), amount + 500);

        let total_subnet_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(from_subnet_id, from_subnet_node_id);
        let total_subnet_node_delegate_stake_balance =
            TotalNodeDelegateStakeBalance::<Test>::get(from_subnet_id, from_subnet_node_id);

        let mut node_delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
            amount,
            total_subnet_node_delegate_stake_shares,
            total_subnet_node_delegate_stake_balance,
        );

        System::set_block_number(
            System::block_number()
                + DelegateStakeCooldownEpochs::<Test>::get() * EpochLength::get(),
        );

        let starting_delegator_balance = Balances::free_balance(&account(n_account));

        assert_ok!(Network::add_to_node_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            from_subnet_id,
            from_subnet_node_id,
            amount,
        ));

        let node_delegate_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(n_account),
            from_subnet_id,
            from_subnet_node_id,
        ));
        assert_eq!(
            node_delegate_shares,
            node_delegate_stake_to_be_added_as_shares
        );
        assert_ne!(node_delegate_shares, 0);

        let total_subnet_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<Test>::get(from_subnet_id, from_subnet_node_id);
        let total_subnet_node_delegate_stake_balance =
            TotalNodeDelegateStakeBalance::<Test>::get(from_subnet_id, from_subnet_node_id);

        let mut from_node_delegate_balance = Network::convert_to_balance(
            node_delegate_shares,
            total_subnet_node_delegate_stake_shares,
            total_subnet_node_delegate_stake_balance,
        );
        // The first depositor will lose a percentage of their deposit depending on the size
        // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
        // assert_eq!(from_delegate_balance, delegate_stake_to_be_added_as_shares);

        let prev_total_subnet_node_delegate_stake_balance =
            TotalNodeDelegateStakeBalance::<Test>::get(from_subnet_id, from_subnet_node_id);
        let prev_next_id = NextSwapQueueId::<Test>::get();

        assert_ok!(Network::swap_node_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            from_subnet_id,
            from_subnet_node_id,
            to_subnet_id,
            to_subnet_node_id,
            node_delegate_shares,
        ));

        // Check ledger doesn't have any unbondings and is empty
        assert!(StakeUnbondingLedger::<Test>::get(account(n_account)).is_empty());

        let from_node_delegate_shares = AccountNodeDelegateStakeShares::<Test>::get((
            account(n_account),
            from_subnet_id,
            from_subnet_node_id,
        ));
        assert_eq!(from_node_delegate_shares, 0);

        assert_ne!(
            prev_total_subnet_node_delegate_stake_balance,
            TotalNodeDelegateStakeBalance::<Test>::get(from_subnet_id, from_subnet_node_id)
        );
        assert!(
            prev_total_subnet_node_delegate_stake_balance
                > TotalNodeDelegateStakeBalance::<Test>::get(from_subnet_id, from_subnet_node_id)
        );

        // Check the queue
        let starting_to_subnet_id = to_subnet_id;
        let call_queue = SwapCallQueue::<Test>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake { .. } => assert!(false),
            QueuedSwapCall::SwapToNodeDelegateStake {
                account_id,
                to_subnet_id,
                to_subnet_node_id,
                balance,
            } => {
                assert_eq!(*account_id, account(n_account));
                assert_eq!(*to_subnet_id, starting_to_subnet_id);
                assert_eq!(*to_subnet_node_id, 1);
                assert_ne!(*balance, 0);
            }
        };

        let next_id = NextSwapQueueId::<Test>::get();
        assert_eq!(prev_next_id + 1, next_id);
        let queue = SwapQueueOrder::<Test>::get();
        assert!(queue
            .first()
            .map_or(false, |&first_id| first_id == prev_next_id));

        // UPDATE

        // Update back to the `from_subnet_id` staying as a `SwapToSubnetDelegateStake`
        let call = QueuedSwapCall::SwapToSubnetDelegateStake {
            account_id: account(n_account),
            to_subnet_id: from_subnet_id,
            balance: u128::MAX,
        };

        assert_ok!(Network::update_swap_queue(
            RuntimeOrigin::signed(account(n_account)),
            prev_next_id,
            call.clone(),
        ));

        let event_exists = network_events().iter().any(|event| {
            matches!(event,
                Event::SwapCallQueueUpdated {
                    id: prev_next_id_val,
                    account_id: account_id_val,
                    call: QueuedSwapCall::SwapToSubnetDelegateStake {
                        account_id: account_id_val2,
                        to_subnet_id: from_subnet_id_val,
                        balance: _, // Ignore balance
                    }
                } if *prev_next_id_val == prev_next_id
                && *account_id_val == account(n_account)
                && *account_id_val2 == account(n_account)
                && *from_subnet_id_val == from_subnet_id
            )
        });
        assert!(event_exists);

        let call_queue = SwapCallQueue::<Test>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake {
                account_id,
                to_subnet_id,
                balance,
            } => {
                assert_eq!(*account_id, account(n_account));
                assert_eq!(*to_subnet_id, from_subnet_id);
                assert_ne!(*balance, 0);
                assert_ne!(*balance, u128::MAX);
            }
            QueuedSwapCall::SwapToNodeDelegateStake { .. } => assert!(false),
        };

        //
        // Update back to the `starting_to_subnet_id` with node ID as a `SwapToNodeDelegateStake`
        //
        let call = QueuedSwapCall::SwapToNodeDelegateStake {
            account_id: account(n_account),
            to_subnet_id: starting_to_subnet_id,
            to_subnet_node_id: 1,
            balance: u128::MAX,
        };

        assert_ok!(Network::update_swap_queue(
            RuntimeOrigin::signed(account(n_account)),
            prev_next_id,
            call.clone(),
        ));

        let event_exists = network_events().iter().any(|event| {
            matches!(event,
                Event::SwapCallQueueUpdated {
                    id: prev_next_id_val,
                    account_id: account_id_val,
                    call: QueuedSwapCall::SwapToSubnetDelegateStake {
                        account_id: account_id_val2,
                        to_subnet_id: from_subnet_id_val,
                        balance: _, // Ignore balance
                    }
                } if *prev_next_id_val == prev_next_id
                && *account_id_val == account(n_account)
                && *account_id_val2 == account(n_account)
                && *from_subnet_id_val == from_subnet_id
            )
        });
        assert!(event_exists);

        let call_queue = SwapCallQueue::<Test>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake { .. } => assert!(false),
            QueuedSwapCall::SwapToNodeDelegateStake {
                account_id,
                to_subnet_id,
                to_subnet_node_id,
                balance,
            } => {
                assert_eq!(*account_id, account(n_account));
                assert_eq!(*to_subnet_id, starting_to_subnet_id);
                assert_eq!(*to_subnet_node_id, 1);
                assert_ne!(*balance, 0);
                assert_ne!(*balance, u128::MAX);
            }
        };
    });
}

#[test]
fn test_execute_ready_swap_calls() {
    new_test_ext().execute_with(|| {
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let end = 4;

        let name_1: Vec<u8> = "subnet-name".into();
        build_activated_subnet(name_1.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id_1 = SubnetName::<Test>::get(name_1.clone()).unwrap();

        let queues_count = 12;

        for n in 0..queues_count {
            let _ = Balances::deposit_creating(&account(n), amount + 500);
            if n % queues_count == 0 {
                // nothing in queue
                insert_to_subnet_swap_call_queue(account(n), subnet_id_1, amount);
                // Sanity check
                let user_shares =
                    AccountSubnetDelegateStakeShares::<Test>::get(&account(n), subnet_id_1);
                assert_eq!(user_shares, 0);
            } else {
                let hotkey = get_hotkey(subnet_id_1, max_subnet_nodes, max_subnets, end - 1);
                let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id_1, hotkey);
                insert_to_node_swap_call_queue(
                    account(n),
                    subnet_id_1,
                    hotkey_subnet_node_id.unwrap(),
                    amount,
                );
                let user_shares = AccountNodeDelegateStakeShares::<Test>::get((
                    &account(n),
                    subnet_id_1,
                    hotkey_subnet_node_id.unwrap(),
                ));
                assert_eq!(user_shares, 0);
            }
        }

        // SANITY CHECK EVERYTHING IS THERE QUEUED
        assert_eq!(SwapQueueOrder::<Test>::get().len(), queues_count as usize);
        assert_eq!(SwapCallQueue::<Test>::iter().count(), queues_count as usize);
        let mut n = 0;
        for (_, call_queue) in SwapCallQueue::<Test>::iter() {
            let _n = n + 1;
            if n % queues_count == 0 {
                match &call_queue.call {
                    QueuedSwapCall::SwapToSubnetDelegateStake {
                        account_id,
                        to_subnet_id,
                        balance,
                    } => {
                        assert_eq!(*account_id, account(n));
                        assert_eq!(*to_subnet_id, subnet_id_1);
                        assert_ne!(*balance, 0);
                        assert_ne!(*balance, u128::MAX);
                    }
                    QueuedSwapCall::SwapToNodeDelegateStake { .. } => assert!(false),
                };
            } else {
            }
            n += 1;
        }

        // NOTHING SHOULD BE EXECUTED
        let _ = Network::execute_ready_swap_calls(System::block_number(), &mut WeightMeter::new());
        assert_eq!(SwapQueueOrder::<Test>::get().len(), queues_count as usize);
        assert_eq!(SwapCallQueue::<Test>::iter().count(), queues_count as usize);

        // INCREASE BLOCKS TO BE ABLE TO EXECUTE
        System::set_block_number(System::block_number() + EpochLength::get() + 1);

        // Swaps SHOULD be executed
        let _ = Network::execute_ready_swap_calls(System::block_number(), &mut WeightMeter::new());

        // Ensure swaps removed from queue
        assert_eq!(SwapQueueOrder::<Test>::get().len(), 0 as usize);
        assert_eq!(SwapCallQueue::<Test>::iter().count(), 0 as usize);

        // Ensure swaps were executed
        for n in 0..queues_count {
            if n % queues_count == 0 {
                // check subnet delegate stake balance
                let user_shares =
                    AccountSubnetDelegateStakeShares::<Test>::get(&account(n), subnet_id_1);
                assert!(user_shares > 0);
            } else {
                // check node delegate stake balance
                let hotkey = get_hotkey(subnet_id_1, max_subnet_nodes, max_subnets, end - 1);
                let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id_1, hotkey);
                let user_shares = AccountNodeDelegateStakeShares::<Test>::get((
                    &account(n),
                    subnet_id_1,
                    hotkey_subnet_node_id.unwrap(),
                ));
                assert!(user_shares > 0);
            }
        }
    });
}
