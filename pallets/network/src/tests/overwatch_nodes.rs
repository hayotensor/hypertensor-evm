use super::mock::*;
use crate::tests::test_utils::*;
use crate::{
    AccountOverwatchStake, ColdkeyHotkeys, Error, HotkeyOverwatchNodeId, HotkeyOwner,
    HotkeySubnetNodeId, MaxOverwatchNodes, MaxSubnetNodes, MaxSubnets, MinSubnetNodes,
    NetworkMinStakeBalance, OverwatchCommit, OverwatchCommits, OverwatchMinAge,
    OverwatchMinStakeBalance, OverwatchNode, OverwatchNodeBlacklist, OverwatchNodeIdHotkey,
    OverwatchNodeIndex, OverwatchNodeWeights, OverwatchNodes, OverwatchReveal, OverwatchReveals,
    OverwatchSubnetWeights, PeerId, PeerIdOverwatchNode, PeerIdSubnetNodeId, StakeCooldownEpochs,
    StakeUnbondingLedgerV2, SubnetData, SubnetName, SubnetNodesData, SubnetState, SubnetsData,
    TotalActiveSubnets, TotalOverwatchNodeUids, TotalOverwatchNodes, TotalOverwatchStake,
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

        assert_err!(
            Network::register_overwatch_node(
                RuntimeOrigin::signed(coldkey.clone()),
                hotkey.clone(),
                amount,
            ),
            Error::<Test>::OverwatchEpochIsZero
        );

        make_overwatch_qualified(1);

        let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
        let uids = TotalOverwatchNodeUids::<Test>::get();
        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(!hotkeys.contains(&hotkey.clone()));

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        assert_eq!(
            init_total_overwatch_nodes + 1,
            TotalOverwatchNodes::<Test>::get()
        );
        assert_eq!(uids + 1, TotalOverwatchNodeUids::<Test>::get());

        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(hotkeys.contains(&hotkey.clone()));

        assert_eq!(HotkeyOwner::<Test>::get(hotkey.clone()), coldkey.clone());
        assert_eq!(
            uids + 1,
            HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap()
        );

        assert_eq!(
            OverwatchNodes::<Test>::get(uids + 1).unwrap().hotkey,
            hotkey.clone()
        );
        assert_eq!(
            OverwatchNodeIdHotkey::<Test>::get(uids + 1),
            Some(hotkey.clone())
        );
        assert_eq!(AccountOverwatchStake::<Test>::get(hotkey.clone()), amount);
    });
}

#[test]
fn test_register_overwatch_node_blacklisted() {
    new_test_ext().execute_with(|| {
        let amount = 100000000000000000000;

        let coldkey = account(1);
        let hotkey = account(2);

        OverwatchNodeBlacklist::<Test>::insert(coldkey.clone(), true);

        assert_err!(
            Network::register_overwatch_node(
                RuntimeOrigin::signed(coldkey.clone()),
                hotkey.clone(),
                amount,
            ),
            Error::<Test>::ColdkeyBlacklisted
        );
    });
}

#[test]
fn test_register_overwatch_node_min_stake_error() {
    new_test_ext().execute_with(|| {
        let min_age = OverwatchMinAge::<Test>::get();
        increase_epochs(min_age + 1);

        let coldkey = account(1);
        let hotkey = account(2);
        let amount = 1000;

        make_overwatch_qualified(1);

        assert_err!(
            Network::register_overwatch_node(
                RuntimeOrigin::signed(coldkey.clone()),
                hotkey.clone(),
                OverwatchMinStakeBalance::<Test>::get() - 1,
            ),
            Error::<Test>::MinStakeNotReached
        );

        assert_err!(
            Network::register_overwatch_node(
                RuntimeOrigin::signed(coldkey.clone()),
                hotkey.clone(),
                OverwatchMinStakeBalance::<Test>::get(),
            ),
            Error::<Test>::NotEnoughBalanceToStake
        );

        let _ =
            Balances::deposit_creating(&coldkey.clone(), OverwatchMinStakeBalance::<Test>::get());

        assert_err!(
            Network::register_overwatch_node(
                RuntimeOrigin::signed(coldkey.clone()),
                hotkey.clone(),
                OverwatchMinStakeBalance::<Test>::get(),
            ),
            Error::<Test>::BalanceWithdrawalError
        );
    });
}

#[test]
fn test_register_overwatch_node_errors() {
    new_test_ext().execute_with(|| {
        let amount = 100000000000000000000;

        let coldkey = account(1);
        let hotkey = account(2);

        set_overwatch_epoch(1);

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

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

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

        // let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
        // let uids = TotalOverwatchNodeUids::<Test>::get();
        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(!hotkeys.contains(&hotkey.clone()));

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();
        let peer_id = peer(1);

        assert_ok!(Network::set_overwatch_node_peer_id(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            uid,
            peer_id.clone(),
        ));

        assert_eq!(
            PeerIdOverwatchNode::<Test>::get(subnet_id, peer_id.clone()),
            uid
        );

        let exists = OverwatchNodeIndex::<Test>::get(uid)
            .get(&subnet_id)
            .map_or(false, |x_peer_id| *x_peer_id == peer_id);
        assert!(exists);
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

        // let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
        // let uids = TotalOverwatchNodeUids::<Test>::get();
        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(!hotkeys.contains(&hotkey.clone()));

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

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
            Error::<Test>::InvalidSubnetId
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

        let bad_peer_id = format!("2");
        let bad_peer: PeerId = PeerId(bad_peer_id.clone().into());

        assert_err!(
            Network::set_overwatch_node_peer_id(
                RuntimeOrigin::signed(coldkey.clone()),
                subnet_id,
                uid,
                bad_peer,
            ),
            Error::<Test>::InvalidPeerId
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

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, snn_hotkey.clone()).unwrap();
        let subnet_node_data =
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
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

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        assert_err!(
            Network::remove_overwatch_node(RuntimeOrigin::signed(coldkey.clone()), 0),
            Error::<Test>::NotKeyOwner
        );

        let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();
        let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();

        let peer_id = peer(1);

        assert_ok!(Network::set_overwatch_node_peer_id(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            uid,
            peer_id.clone(),
        ));

        assert_ok!(Network::remove_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap(),
        ));

        assert_eq!(OverwatchNodes::<Test>::try_get(uid), Err(()));
        assert_eq!(
            init_total_overwatch_nodes - 1,
            TotalOverwatchNodes::<Test>::get()
        );
        assert_eq!(OverwatchNodeIdHotkey::<Test>::try_get(uid), Err(()));
        assert_eq!(
            HotkeyOverwatchNodeId::<Test>::try_get(hotkey.clone()),
            Err(())
        );
        assert_eq!(
            PeerIdOverwatchNode::<Test>::try_get(subnet_id, peer_id.clone()),
            Err(())
        );
        let map = OverwatchNodeIndex::<Test>::take(uid);
        for (subnet_id, map_peer_id) in map {
            assert_ne!(peer_id.clone(), map_peer_id);
        }
    });
}

#[test]
fn test_anyone_remove_overwatch_node() {
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
        let coldkey = account(1);
        let hotkey = account(2);
        let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

        assert_err!(
            Network::anyone_remove_overwatch_node(RuntimeOrigin::signed(account(0)), 0),
            Error::<Test>::InvalidOverwatchNodeId
        );

        make_overwatch_qualified(1);

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            stake_amount,
        ));

        let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        assert_err!(
            Network::anyone_remove_overwatch_node(RuntimeOrigin::signed(account(0)), 1),
            Error::<Test>::ColdkeyOverwatchQualified
        );

        make_overwatch_unqualified(1);

        let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();

        assert_ok!(Network::anyone_remove_overwatch_node(
            RuntimeOrigin::signed(account(0)),
            uid
        ));

        assert_eq!(OverwatchNodes::<Test>::try_get(uid), Err(()));
        assert_eq!(
            init_total_overwatch_nodes - 1,
            TotalOverwatchNodes::<Test>::get()
        );
        assert_eq!(OverwatchNodeIdHotkey::<Test>::try_get(uid), Err(()));
        assert_eq!(
            HotkeyOverwatchNodeId::<Test>::try_get(hotkey.clone()),
            Err(())
        );
    });
}

#[test]
fn test_equal_stake_equal_weights_v3() {
    new_test_ext().execute_with(|| {
        let subnet_id = 1;
        let epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup
        let node_id_1 = insert_overwatch_node(1, 1);
        let node_id_2 = insert_overwatch_node(2, 2);
        set_overwatch_stake(1, 100);
        set_overwatch_stake(2, 100);

        submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
        submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

        let mut ostake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..2 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());
            assert_ne!(overwatch_stake, 0);
            ostake_snapshot.insert(hotkey.clone(), overwatch_stake);
        }

        let block_weight = Network::calculate_overwatch_rewards();

        for n in 0..2 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

            if let Some(old_stake) = ostake_snapshot.get(&hotkey) {
                assert!(overwatch_stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

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

        let node_id_1 = insert_overwatch_node(1, 1);
        let node_id_2 = insert_overwatch_node(2, 2);
        set_overwatch_stake(1, 90);
        set_overwatch_stake(2, 10);

        submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
        submit_weight(epoch, subnet_id, node_id_2, 500000000000000000);

        let mut ostake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..2 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());
            assert_ne!(overwatch_stake, 0);
            ostake_snapshot.insert(hotkey.clone(), overwatch_stake);
        }

        let block_weight = Network::calculate_overwatch_rewards();

        for n in 0..2 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

            if let Some(old_stake) = ostake_snapshot.get(&hotkey) {
                assert!(overwatch_stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

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

        let node_id_1 = insert_overwatch_node(1, 1);
        let node_id_2 = insert_overwatch_node(2, 2);
        set_overwatch_stake(1, 50);
        set_overwatch_stake(2, 50);

        submit_weight(epoch, subnet_id, node_id_1, 500000000000000000);
        submit_weight(epoch, subnet_id, node_id_2, 100);

        let mut ostake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..2 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());
            assert_ne!(overwatch_stake, 0);
            ostake_snapshot.insert(hotkey.clone(), overwatch_stake);
        }

        let block_weight = Network::calculate_overwatch_rewards();

        for n in 0..2 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

            if let Some(old_stake) = ostake_snapshot.get(&hotkey) {
                assert!(overwatch_stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

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

        let node_id_1 = insert_overwatch_node(1, 1);
        let node_id_2 = insert_overwatch_node(2, 2);
        set_overwatch_stake(1, 50);
        set_overwatch_stake(2, 100);

        // Subnet 1
        submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
        submit_weight(epoch, subnet_id_1, node_id_2, 500000000000000000);
        // Subnet 2
        submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
        submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000); // Node 2 slightly deviates

        let mut ostake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..2 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());
            assert_ne!(overwatch_stake, 0);
            ostake_snapshot.insert(hotkey.clone(), overwatch_stake);
        }

        let block_weight = Network::calculate_overwatch_rewards();

        for n in 0..2 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

            if let Some(old_stake) = ostake_snapshot.get(&hotkey) {
                assert!(overwatch_stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

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

        let node_id_1 = insert_overwatch_node(1, 1);
        let node_id_2 = insert_overwatch_node(2, 2);
        set_overwatch_stake(1, 100);
        set_overwatch_stake(2, 50);

        // Subnet 1
        submit_weight(epoch, subnet_id_1, node_id_1, 500000000000000000);
        submit_weight(epoch, subnet_id_1, node_id_2, 500000000000000000);
        // Subnet 2
        submit_weight(epoch, subnet_id_2, node_id_1, 500000000000000000);
        submit_weight(epoch, subnet_id_2, node_id_2, 600000000000000000); // Node 2 slightly deviates

        let mut ostake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..2 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());
            assert_ne!(overwatch_stake, 0);
            ostake_snapshot.insert(hotkey.clone(), overwatch_stake);
        }

        let block_weight = Network::calculate_overwatch_rewards();

        for n in 0..2 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

            if let Some(old_stake) = ostake_snapshot.get(&hotkey) {
                assert!(overwatch_stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

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
        let node_id_1 = insert_overwatch_node(1, 1);
        let node_id_2 = insert_overwatch_node(2, 2);
        let node_id_3 = insert_overwatch_node(3, 3);
        let node_id_4 = insert_overwatch_node(4, 4);
        let node_id_5 = insert_overwatch_node(5, 5);
        let node_id_6 = insert_overwatch_node(6, 6);
        let node_id_7 = insert_overwatch_node(7, 7);
        let node_id_8 = insert_overwatch_node(8, 8);

        set_overwatch_stake(1, 100);
        set_overwatch_stake(2, 50);
        set_overwatch_stake(3, 25);
        set_overwatch_stake(4, 500);
        set_overwatch_stake(5, 200);
        set_overwatch_stake(6, 340);
        set_overwatch_stake(7, 1);
        set_overwatch_stake(8, 9);

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

        let mut ostake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..8 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());
            assert_ne!(overwatch_stake, 0);
            ostake_snapshot.insert(hotkey.clone(), overwatch_stake);
        }

        let _ = Network::calculate_overwatch_rewards();

        for n in 0..8 {
            let hotkey = account(n + 1);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

            if let Some(old_stake) = ostake_snapshot.get(&hotkey) {
                assert!(overwatch_stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

        // let subnet_weight_1 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_1);
        // let subnet_weight_2 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_2);
        // let subnet_weight_3 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_3);
        // let subnet_weight_4 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_4);
        // let subnet_weight_5 = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id_5);

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

        // let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
        // let uids = TotalOverwatchNodeUids::<Test>::get();
        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(!hotkeys.contains(&hotkey.clone()));

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let increase_amount = 100000000000000000000;
        let _ = Balances::deposit_creating(&coldkey.clone(), increase_amount);

        let prev_account_balance = AccountOverwatchStake::<Test>::get(hotkey.clone());
        let prev_total_overwatch_balance = TotalOverwatchStake::<Test>::get();

        assert_ok!(Network::add_to_overwatch_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            uid,
            hotkey.clone(),
            increase_amount,
        ));

        assert_eq!(
            prev_account_balance + increase_amount,
            AccountOverwatchStake::<Test>::get(hotkey.clone())
        );

        assert_eq!(
            prev_total_overwatch_balance + increase_amount,
            TotalOverwatchStake::<Test>::get()
        );

        assert_eq!(
            AccountOverwatchStake::<Test>::get(hotkey.clone()),
            amount + increase_amount
        );
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

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

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
                increase_amount + 500,
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

        // let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
        // let uids = TotalOverwatchNodeUids::<Test>::get();
        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(!hotkeys.contains(&hotkey.clone()));

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let increase_amount = 100000000000000000000;
        let _ = Balances::deposit_creating(&coldkey.clone(), increase_amount);

        assert_ok!(Network::add_to_overwatch_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            uid,
            hotkey.clone(),
            increase_amount,
        ));

        assert_eq!(
            AccountOverwatchStake::<Test>::get(hotkey.clone()),
            amount + increase_amount
        );

        let remove_amount = 50000000000000000000;

        let starting_balance = Balances::free_balance(&coldkey.clone());

        let prev_account_balance = AccountOverwatchStake::<Test>::get(hotkey.clone());
        let prev_total_overwatch_balance = TotalOverwatchStake::<Test>::get();

        assert_ok!(Network::remove_overwatch_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            remove_amount,
        ));

        assert_eq!(
            prev_account_balance - remove_amount,
            AccountOverwatchStake::<Test>::get(hotkey.clone())
        );
        assert_eq!(
            prev_total_overwatch_balance - remove_amount,
            TotalOverwatchStake::<Test>::get()
        );

        assert_eq!(
            AccountOverwatchStake::<Test>::get(hotkey.clone()),
            amount + increase_amount - remove_amount
        );

        assert_eq!(starting_balance, Balances::free_balance(&coldkey.clone()));
    });
}

#[test]
fn test_add_to_remove_overwatch_stake_unbond() {
    new_test_ext().execute_with(|| {
        let amount = 100000000000000000000;

        let coldkey = account(1);
        let hotkey = account(2);
        let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

        make_overwatch_qualified(1);

        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(!hotkeys.contains(&hotkey.clone()));

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let increase_amount = 100000000000000000000;
        let _ = Balances::deposit_creating(&coldkey.clone(), increase_amount);

        assert_ok!(Network::add_to_overwatch_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            uid,
            hotkey.clone(),
            increase_amount,
        ));

        assert_eq!(
            AccountOverwatchStake::<Test>::get(hotkey.clone()),
            amount + increase_amount
        );

        let remove_amount = 50000000000000000000;

        let starting_balance = Balances::free_balance(&coldkey.clone());
        let block = System::block_number();

        assert_ok!(Network::remove_overwatch_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            remove_amount,
        ));

        assert_eq!(
            AccountOverwatchStake::<Test>::get(hotkey.clone()),
            amount + increase_amount - remove_amount
        );

        assert_eq!(starting_balance, Balances::free_balance(&coldkey.clone()));

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedgerV2::<Test>::get(coldkey.clone());
        assert_eq!(unbondings.len(), 1);
        let (ledger_block, ledger_balance) = unbondings.iter().next().unwrap();
        assert_eq!(
            *ledger_block,
            &block + StakeCooldownEpochs::<Test>::get() * EpochLength::get()
        );
        assert_eq!(*ledger_balance, remove_amount);

        System::set_block_number(block + StakeCooldownEpochs::<Test>::get() * EpochLength::get());

        let starting_balance = Balances::free_balance(&coldkey.clone());

        assert_ok!(Network::claim_unbondings(RuntimeOrigin::signed(
            coldkey.clone()
        )));

        assert_eq!(
            Balances::free_balance(&coldkey.clone()),
            starting_balance + remove_amount
        );

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedgerV2::<Test>::get(coldkey.clone());
        assert_eq!(unbondings.len(), 0);
    });
}

#[test]
fn test_remove_overwatch_stake_after_removing_overwatch_node() {
    new_test_ext().execute_with(|| {
        let amount = 100000000000000000000;

        let coldkey = account(1);
        let hotkey = account(2);
        let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);

        make_overwatch_qualified(1);

        // let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
        // let uids = TotalOverwatchNodeUids::<Test>::get();
        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(!hotkeys.contains(&hotkey.clone()));

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let increase_amount = 100000000000000000000;
        let _ = Balances::deposit_creating(&coldkey.clone(), increase_amount);

        assert_ok!(Network::add_to_overwatch_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            uid,
            hotkey.clone(),
            increase_amount,
        ));

        assert_eq!(
            AccountOverwatchStake::<Test>::get(hotkey.clone()),
            amount + increase_amount
        );

        assert_ok!(Network::remove_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap(),
        ));

        let remove_amount = 50000000000000000000;

        let starting_balance = Balances::free_balance(&coldkey.clone());
        assert_ok!(Network::remove_overwatch_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            remove_amount,
        ));

        assert_eq!(
            AccountOverwatchStake::<Test>::get(hotkey.clone()),
            amount + increase_amount - remove_amount
        );
        assert_eq!(starting_balance, Balances::free_balance(&coldkey.clone()));
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

        // let init_total_overwatch_nodes = TotalOverwatchNodes::<Test>::get();
        // let uids = TotalOverwatchNodeUids::<Test>::get();
        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(!hotkeys.contains(&hotkey.clone()));

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let uid = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let increase_amount = 100000000000000000000;
        let _ = Balances::deposit_creating(&coldkey.clone(), increase_amount);

        assert_ok!(Network::add_to_overwatch_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            uid,
            hotkey.clone(),
            increase_amount,
        ));

        assert_eq!(
            AccountOverwatchStake::<Test>::get(hotkey.clone()),
            amount + increase_amount
        );

        assert_err!(
            Network::remove_overwatch_stake(
                RuntimeOrigin::signed(coldkey.clone()),
                hotkey.clone(),
                0,
            ),
            Error::<Test>::AmountZero
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

#[test]
fn test_zero_score() {
    new_test_ext().execute_with(|| {
        let subnet_id = 1;
        let epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup
        let node_id_1 = insert_overwatch_node(1, 1);
        let node_id_2 = insert_overwatch_node(2, 2);
        set_overwatch_stake(1, 90);
        set_overwatch_stake(2, 10);

        submit_weight(epoch, subnet_id, node_id_1, 0);
        submit_weight(epoch, subnet_id, node_id_2, 1000000000000000000);

        let block_weight = Network::calculate_overwatch_rewards();

        let subnet_weight = OverwatchSubnetWeights::<Test>::get(epoch, subnet_id);

        // Score should be 0.1
        assert_eq!(subnet_weight, Some(100000000000000000));

        let score_1 = OverwatchNodeWeights::<Test>::get(epoch, node_id_1);
        let score_2 = OverwatchNodeWeights::<Test>::get(epoch, node_id_2);

        assert_eq!(score_1, Some(900000000000000000));
        assert_eq!(score_2, Some(100000000000000000));

        let mut score_sum = 0;
        let mut nodes = 0;
        for (id, _) in OverwatchNodes::<Test>::iter() {
            nodes += 1;
            let weight = OverwatchNodeWeights::<Test>::get(epoch, id);
            score_sum += weight.unwrap();
        }

        assert_eq!(nodes, 2);
        assert!(score_sum <= 1000000000000000000 && score_sum >= 999999999999999990);
    });
}
