use super::mock::*;
use crate::tests::test_utils::*;
use crate::{
    Error, HotkeyOverwatchNodeId, HotkeyOwner, MinSubnetMinStake, MinSubnetNodes, OverwatchCommit,
    OverwatchCommits, OverwatchNode, OverwatchNodeBlacklist, OverwatchNodeIdHotkey, OverwatchNodes,
    OverwatchReveal, OverwatchReveals, SubnetData, SubnetName, SubnetState, SubnetsData,
    TotalOverwatchNodeUids,
};
use frame_support::traits::Currency;
use frame_support::{assert_err, assert_ok};

//
//
//
//
//
//
//
// Overwatch Commit-Reveal
//
//
//
//
//
//
//

#[test]
fn test_do_commit_and_reveal_weights_success() {
    new_test_ext().execute_with(|| {
        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);
        let overwatch_node_id = 1;
        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup: assign ownership and create subnet
        let subnet_data = SubnetData {
            id: 1,
            friendly_id: 1,
            name: "subnet_name".into(),
            repo: "github".into(),
            description: "description".into(),
            misc: "misc".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };

        SubnetsData::<Test>::insert(subnet_id, subnet_data);

        TotalOverwatchNodeUids::<Test>::mutate(|n: &mut u32| *n += 1);
        let current_uid = TotalOverwatchNodeUids::<Test>::get();

        let overwatch_node = OverwatchNode {
            id: current_uid,
            hotkey: hotkey.clone(),
        };

        OverwatchNodes::<Test>::insert(overwatch_node_id, overwatch_node);
        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        OverwatchNodeIdHotkey::<Test>::insert(current_uid, hotkey.clone());

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        // Commit
        assert_ok!(Network::perform_commit_overwatch_subnet_weights(
            overwatch_node_id,
            vec![OverwatchCommit {
                subnet_id,
                weight: commit_hash
            }]
        ));

        // Ensure it's stored
        let stored =
            OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id)).unwrap();
        assert_eq!(stored, commit_hash);

        // Reveal
        assert_ok!(Network::perform_reveal_overwatch_subnet_weights(
            overwatch_node_id,
            vec![OverwatchReveal {
                subnet_id,
                weight,
                salt
            }]
        ));

        // Ensure revealed weight is correct
        let revealed =
            OverwatchReveals::<Test>::get((overwatch_epoch, subnet_id, overwatch_node_id)).unwrap();
        assert_eq!(revealed, weight);
    });
}

#[test]
fn test_do_commit_and_reveal_weights_not_key_owner_error() {
    new_test_ext().execute_with(|| {
        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);
        let overwatch_node_id = 1;
        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup: assign ownership and create subnet
        let subnet_data = SubnetData {
            id: 1,
            friendly_id: 1,
            name: "subnet_name".into(),
            repo: "github".into(),
            description: "description".into(),
            misc: "misc".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };

        SubnetsData::<Test>::insert(subnet_id, subnet_data);

        TotalOverwatchNodeUids::<Test>::mutate(|n: &mut u32| *n += 1);
        let current_uid = TotalOverwatchNodeUids::<Test>::get();

        let overwatch_node = OverwatchNode {
            id: current_uid,
            hotkey: hotkey.clone(),
        };

        OverwatchNodes::<Test>::insert(overwatch_node_id, overwatch_node);
        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        OverwatchNodeIdHotkey::<Test>::insert(current_uid, hotkey.clone());

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        // Commit
        assert_err!(
            Network::commit_overwatch_subnet_weights(
                RuntimeOrigin::signed(hotkey.clone()),
                0,
                vec![OverwatchCommit {
                    subnet_id,
                    weight: commit_hash
                }]
            ),
            Error::<Test>::NotKeyOwner
        );
    });
}

#[test]
fn test_do_commit_and_reveal_weights_blacklisted_error() {
    new_test_ext().execute_with(|| {
        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);
        let overwatch_node_id = 1;
        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup: assign ownership and create subnet
        let subnet_data = SubnetData {
            id: 1,
            friendly_id: 1,
            name: "subnet_name".into(),
            repo: "github".into(),
            description: "description".into(),
            misc: "misc".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };

        SubnetsData::<Test>::insert(subnet_id, subnet_data);

        TotalOverwatchNodeUids::<Test>::mutate(|n: &mut u32| *n += 1);
        let current_uid = TotalOverwatchNodeUids::<Test>::get();

        let overwatch_node = OverwatchNode {
            id: current_uid,
            hotkey: hotkey.clone(),
        };

        OverwatchNodes::<Test>::insert(overwatch_node_id, overwatch_node);
        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        OverwatchNodeIdHotkey::<Test>::insert(current_uid, hotkey.clone());

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        OverwatchNodeBlacklist::<Test>::insert(coldkey.clone(), true);

        // Commit
        assert_err!(
            Network::commit_overwatch_subnet_weights(
                RuntimeOrigin::signed(hotkey.clone()),
                current_uid,
                vec![OverwatchCommit {
                    subnet_id,
                    weight: commit_hash
                }]
            ),
            Error::<Test>::ColdkeyBlacklisted
        );
    });
}

#[test]
fn test_do_commit_and_reveal_weights_commits_empty_error() {
    new_test_ext().execute_with(|| {
        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);
        let overwatch_node_id = 1;
        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup: assign ownership and create subnet
        let subnet_data = SubnetData {
            id: 1,
            friendly_id: 1,
            name: "subnet_name".into(),
            repo: "github".into(),
            description: "description".into(),
            misc: "misc".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };

        SubnetsData::<Test>::insert(subnet_id, subnet_data);

        TotalOverwatchNodeUids::<Test>::mutate(|n: &mut u32| *n += 1);
        let current_uid = TotalOverwatchNodeUids::<Test>::get();

        let overwatch_node = OverwatchNode {
            id: current_uid,
            hotkey: hotkey.clone(),
        };

        OverwatchNodes::<Test>::insert(overwatch_node_id, overwatch_node);
        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        OverwatchNodeIdHotkey::<Test>::insert(current_uid, hotkey.clone());

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        // Commit
        assert_err!(
            Network::commit_overwatch_subnet_weights(
                RuntimeOrigin::signed(hotkey.clone()),
                current_uid,
                vec![]
            ),
            Error::<Test>::CommitsEmpty
        );
    });
}

#[test]
fn test_do_commit_and_reveal_weights_already_committed_error() {
    new_test_ext().execute_with(|| {
        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);
        let overwatch_node_id = 1;
        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup: assign ownership and create subnet
        let subnet_data = SubnetData {
            id: 1,
            friendly_id: 1,
            name: "subnet_name".into(),
            repo: "github".into(),
            description: "description".into(),
            misc: "misc".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };

        SubnetsData::<Test>::insert(subnet_id, subnet_data);

        TotalOverwatchNodeUids::<Test>::mutate(|n: &mut u32| *n += 1);
        let current_uid = TotalOverwatchNodeUids::<Test>::get();

        let overwatch_node = OverwatchNode {
            id: current_uid,
            hotkey: hotkey.clone(),
        };

        OverwatchNodes::<Test>::insert(overwatch_node_id, overwatch_node);
        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        OverwatchNodeIdHotkey::<Test>::insert(current_uid, hotkey.clone());

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        // Commit
        assert_ok!(Network::commit_overwatch_subnet_weights(
            RuntimeOrigin::signed(hotkey.clone()),
            overwatch_node_id,
            vec![OverwatchCommit {
                subnet_id,
                weight: commit_hash
            }]
        ));

        // Ensure it's stored
        let stored =
            OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id)).unwrap();
        assert_eq!(stored, commit_hash);

        // Reveal
        assert_ok!(Network::perform_reveal_overwatch_subnet_weights(
            overwatch_node_id,
            vec![OverwatchReveal {
                subnet_id,
                weight,
                salt
            }]
        ));

        // Ensure revealed weight is correct
        let revealed =
            OverwatchReveals::<Test>::get((overwatch_epoch, subnet_id, overwatch_node_id)).unwrap();
        assert_eq!(revealed, weight);

        assert_err!(
            Network::commit_overwatch_subnet_weights(
                RuntimeOrigin::signed(hotkey.clone()),
                overwatch_node_id,
                vec![OverwatchCommit {
                    subnet_id,
                    weight: commit_hash
                }]
            ),
            Error::<Test>::AlreadyCommitted
        );
    });
}

#[test]
fn test_commit_and_reveal_extrinsics() {
    new_test_ext().execute_with(|| {
        // subnet
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let min_subnet_nodes = MinSubnetNodes::<Test>::get();
        let end = min_subnet_nodes;
        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);

        let amount = 100000000000000000000;

        let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);
        make_overwatch_qualified(1);

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup: assign ownership and create subnet
        let subnet_data = SubnetData {
            id: 1,
            friendly_id: 1,
            name: "subnet_name".into(),
            repo: "github".into(),
            description: "description".into(),
            misc: "misc".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };

        SubnetsData::<Test>::insert(subnet_id, subnet_data);

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        // Commit
        assert_ok!(Network::commit_overwatch_subnet_weights(
            RuntimeOrigin::signed(hotkey.clone()),
            overwatch_node_id,
            vec![OverwatchCommit {
                subnet_id,
                weight: commit_hash
            }]
        ));

        // Ensure it's stored
        let stored =
            OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id)).unwrap();
        assert_eq!(stored, commit_hash);

        set_block_to_overwatch_reveal_block(overwatch_epoch);

        // Reveal
        assert_ok!(Network::reveal_overwatch_subnet_weights(
            RuntimeOrigin::signed(hotkey.clone()),
            overwatch_node_id,
            vec![OverwatchReveal {
                subnet_id,
                weight,
                salt
            }]
        ));

        // Ensure revealed weight is correct
        let revealed =
            OverwatchReveals::<Test>::get((overwatch_epoch, subnet_id, overwatch_node_id)).unwrap();
        assert_eq!(revealed, weight);
    });
}

#[test]
fn test_reveal_overwatch_subnet_weights_not_key_owner_error() {
    new_test_ext().execute_with(|| {
        // subnet
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let min_subnet_nodes = MinSubnetNodes::<Test>::get();
        let end = min_subnet_nodes;
        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);

        let amount = 100000000000000000000;

        let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);
        make_overwatch_qualified(1);

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup: assign ownership and create subnet
        let subnet_data = SubnetData {
            id: 1,
            friendly_id: 1,
            name: "subnet_name".into(),
            repo: "github".into(),
            description: "description".into(),
            misc: "misc".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };

        SubnetsData::<Test>::insert(subnet_id, subnet_data);

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        // Commit
        assert_ok!(Network::commit_overwatch_subnet_weights(
            RuntimeOrigin::signed(hotkey.clone()),
            overwatch_node_id,
            vec![OverwatchCommit {
                subnet_id,
                weight: commit_hash
            }]
        ));

        // Ensure it's stored
        let stored =
            OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id)).unwrap();
        assert_eq!(stored, commit_hash);

        set_block_to_overwatch_reveal_block(overwatch_epoch);

        // Reveal
        assert_err!(
            Network::reveal_overwatch_subnet_weights(
                RuntimeOrigin::signed(account(0)),
                overwatch_node_id,
                vec![OverwatchReveal {
                    subnet_id,
                    weight: weight.clone(),
                    salt: salt.clone()
                }]
            ),
            Error::<Test>::NotKeyOwner
        );

        assert_err!(
            Network::reveal_overwatch_subnet_weights(
                RuntimeOrigin::signed(hotkey.clone()),
                123,
                vec![OverwatchReveal {
                    subnet_id,
                    weight: weight.clone(),
                    salt: salt.clone()
                }]
            ),
            Error::<Test>::NotKeyOwner
        );
    });
}

#[test]
fn test_reveal_overwatch_subnet_weights_blacklisted_error() {
    new_test_ext().execute_with(|| {
        // subnet
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let min_subnet_nodes = MinSubnetNodes::<Test>::get();
        let end = min_subnet_nodes;
        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);

        let amount = 100000000000000000000;

        let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);
        make_overwatch_qualified(1);

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup: assign ownership and create subnet
        let subnet_data = SubnetData {
            id: 1,
            friendly_id: 1,
            name: "subnet_name".into(),
            repo: "github".into(),
            description: "description".into(),
            misc: "misc".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };

        SubnetsData::<Test>::insert(subnet_id, subnet_data);

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        // Commit
        assert_ok!(Network::commit_overwatch_subnet_weights(
            RuntimeOrigin::signed(hotkey.clone()),
            overwatch_node_id,
            vec![OverwatchCommit {
                subnet_id,
                weight: commit_hash
            }]
        ));

        // Ensure it's stored
        let stored =
            OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id)).unwrap();
        assert_eq!(stored, commit_hash);

        set_block_to_overwatch_reveal_block(overwatch_epoch);

        OverwatchNodeBlacklist::<Test>::insert(coldkey.clone(), true);

        // Reveal
        assert_err!(
            Network::reveal_overwatch_subnet_weights(
                RuntimeOrigin::signed(coldkey.clone()),
                overwatch_node_id,
                vec![OverwatchReveal {
                    subnet_id,
                    weight: weight.clone(),
                    salt: salt.clone()
                }]
            ),
            Error::<Test>::ColdkeyBlacklisted
        );
    });
}

#[test]
fn test_reveal_overwatch_subnet_weights_no_commit_found_error() {
    new_test_ext().execute_with(|| {
        // subnet
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let min_subnet_nodes = MinSubnetNodes::<Test>::get();
        let end = min_subnet_nodes;
        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);

        let amount = 100000000000000000000;

        let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);
        make_overwatch_qualified(1);

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup: assign ownership and create subnet
        let subnet_data = SubnetData {
            id: 1,
            friendly_id: 1,
            name: "subnet_name".into(),
            repo: "github".into(),
            description: "description".into(),
            misc: "misc".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };

        SubnetsData::<Test>::insert(subnet_id, subnet_data);

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        set_block_to_overwatch_reveal_block(overwatch_epoch);

        // Reveal
        assert_err!(
            Network::reveal_overwatch_subnet_weights(
                RuntimeOrigin::signed(hotkey.clone()),
                overwatch_node_id,
                vec![OverwatchReveal {
                    subnet_id,
                    weight: weight.clone(),
                    salt: salt.clone()
                }]
            ),
            Error::<Test>::NoCommitFound
        );
    });
}

#[test]
fn test_reveal_overwatch_subnet_weights_reveal_mismatch_error() {
    new_test_ext().execute_with(|| {
        // subnet
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let min_subnet_nodes = MinSubnetNodes::<Test>::get();
        let end = min_subnet_nodes;
        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);

        let amount = 100000000000000000000;

        let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);
        make_overwatch_qualified(1);

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Setup: assign ownership and create subnet
        let subnet_data = SubnetData {
            id: 1,
            friendly_id: 1,
            name: "subnet_name".into(),
            repo: "github".into(),
            description: "description".into(),
            misc: "misc".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };

        SubnetsData::<Test>::insert(subnet_id, subnet_data);

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        let fake_salt: Vec<u8> = b"fake-salt".to_vec();

        // Commit
        assert_ok!(Network::commit_overwatch_subnet_weights(
            RuntimeOrigin::signed(hotkey.clone()),
            overwatch_node_id,
            vec![OverwatchCommit {
                subnet_id,
                weight: commit_hash
            }]
        ));

        // Ensure it's stored
        let stored =
            OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id)).unwrap();
        assert_eq!(stored, commit_hash);

        set_block_to_overwatch_reveal_block(overwatch_epoch);

        // Reveal
        assert_err!(
            Network::reveal_overwatch_subnet_weights(
                RuntimeOrigin::signed(hotkey.clone()),
                overwatch_node_id,
                vec![OverwatchReveal {
                    subnet_id,
                    weight: weight.clone(),
                    salt: fake_salt.clone()
                }]
            ),
            Error::<Test>::RevealMismatch
        );

        assert_err!(
            Network::reveal_overwatch_subnet_weights(
                RuntimeOrigin::signed(hotkey.clone()),
                overwatch_node_id,
                vec![OverwatchReveal {
                    subnet_id,
                    weight: weight.clone() + 1,
                    salt: salt.clone()
                }]
            ),
            Error::<Test>::RevealMismatch
        );
    });
}

#[test]
fn test_commit_reveal_multiple_times_in_same_epoch() {
    new_test_ext().execute_with(|| {
        // Subnet 1
        let subnet_id_1 = 1;
        let subnet_data = SubnetData {
            id: subnet_id_1,
            friendly_id: subnet_id_1,
            name: "subnet_name_1".into(),
            repo: "github-1".into(),
            description: "description-1".into(),
            misc: "misc-1".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };
        SubnetsData::<Test>::insert(subnet_id_1, subnet_data);
        // Subnet 2
        let subnet_id_2 = 2;
        let subnet_data = SubnetData {
            id: subnet_id_2,
            friendly_id: subnet_id_2,
            name: "subnet_name_2".into(),
            repo: "github-2".into(),
            description: "description-2".into(),
            misc: "misc-2".into(),
            state: SubnetState::Active,
            start_epoch: 0,
        };
        SubnetsData::<Test>::insert(subnet_id_2, subnet_data);

        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);

        let amount = 100000000000000000000;

        let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);
        make_overwatch_qualified(1);

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        // Weight + salt
        // Subnet 1
        let weight_1: u128 = 123456;
        let salt_1: Vec<u8> = b"secret-salt-1".to_vec();
        let commit_hash_1 = make_commit(weight_1, salt_1.clone());
        // Subnet 2
        let weight_2: u128 = 78910;
        let salt_2: Vec<u8> = b"secret-salt-2".to_vec();
        let commit_hash_2 = make_commit(weight_2, salt_2.clone());

        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Commit
        assert_ok!(Network::commit_overwatch_subnet_weights(
            RuntimeOrigin::signed(hotkey.clone()),
            overwatch_node_id,
            vec![OverwatchCommit {
                subnet_id: subnet_id_1,
                weight: commit_hash_1
            }]
        ));

        // Ensure it's stored
        let stored =
            OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id_1))
                .unwrap();
        assert_eq!(stored, commit_hash_1);
        assert_eq!(
            OverwatchCommits::<Test>::try_get((overwatch_epoch, overwatch_node_id, subnet_id_2)),
            Err(())
        );

        assert_ok!(Network::commit_overwatch_subnet_weights(
            RuntimeOrigin::signed(hotkey.clone()),
            overwatch_node_id,
            vec![OverwatchCommit {
                subnet_id: subnet_id_2,
                weight: commit_hash_2
            }]
        ));

        let stored =
            OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id_1))
                .unwrap();
        assert_eq!(stored, commit_hash_1);
        let stored =
            OverwatchCommits::<Test>::get((overwatch_epoch, overwatch_node_id, subnet_id_2))
                .unwrap();
        assert_eq!(stored, commit_hash_2);

        set_block_to_overwatch_reveal_block(overwatch_epoch);

        assert_ok!(Network::reveal_overwatch_subnet_weights(
            RuntimeOrigin::signed(hotkey.clone()),
            overwatch_node_id,
            vec![OverwatchReveal {
                subnet_id: subnet_id_1,
                weight: weight_1,
                salt: salt_1
            }]
        ));

        // Ensure revealed weight is correct
        let revealed =
            OverwatchReveals::<Test>::get((overwatch_epoch, subnet_id_1, overwatch_node_id))
                .unwrap();
        assert_eq!(revealed, weight_1);
        assert_eq!(
            OverwatchReveals::<Test>::try_get((overwatch_epoch, subnet_id_2, overwatch_node_id)),
            Err(())
        );

        assert_ok!(Network::reveal_overwatch_subnet_weights(
            RuntimeOrigin::signed(hotkey.clone()),
            overwatch_node_id,
            vec![OverwatchReveal {
                subnet_id: subnet_id_2,
                weight: weight_2,
                salt: salt_2
            }]
        ));

        // Ensure revealed weight is correct
        let revealed =
            OverwatchReveals::<Test>::get((overwatch_epoch, subnet_id_1, overwatch_node_id))
                .unwrap();
        assert_eq!(revealed, weight_1);
        let revealed =
            OverwatchReveals::<Test>::get((overwatch_epoch, subnet_id_2, overwatch_node_id))
                .unwrap();
        assert_eq!(revealed, weight_2);
    });
}

#[test]
fn test_commit_and_reveal_phase_errors() {
    new_test_ext().execute_with(|| {
        // subnet
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let min_subnet_nodes = MinSubnetNodes::<Test>::get();
        let end = min_subnet_nodes;
        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let coldkey: AccountId = account(1);
        let hotkey: AccountId = account(2);

        let amount = 100000000000000000000;

        let _ = Balances::deposit_creating(&coldkey.clone(), 100000000000000000000 + 500);
        make_overwatch_qualified(1);

        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        let subnet_id = 99;
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        // Weight + salt
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit(weight, salt.clone());

        // Reveal
        assert_err!(
            Network::reveal_overwatch_subnet_weights(
                RuntimeOrigin::signed(hotkey.clone()),
                overwatch_node_id,
                vec![OverwatchReveal {
                    subnet_id,
                    weight,
                    salt
                }]
            ),
            Error::<Test>::NotRevealPeriod
        );

        set_block_to_overwatch_reveal_block(overwatch_epoch);

        // Commit fail
        assert_err!(
            Network::commit_overwatch_subnet_weights(
                RuntimeOrigin::signed(hotkey.clone()),
                overwatch_node_id,
                vec![OverwatchCommit {
                    subnet_id,
                    weight: commit_hash
                }]
            ),
            Error::<Test>::NotCommitPeriod
        );
    });
}
