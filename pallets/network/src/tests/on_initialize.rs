use super::mock::*;
use crate::tests::test_utils::*;
use crate::{
    AccountOverwatchStake, AccountSubnetStake, FinalSubnetEmissionWeights, HotkeyOverwatchNodeId,
    MaxOverwatchNodes, MaxSubnetNodes, MaxSubnets, MinSubnetNodes, NewRegistrationCostMultiplier,
    OverwatchCommit, OverwatchCommits, OverwatchEpochLengthMultiplier, OverwatchReveal,
    OverwatchReveals, SlotAssignment, SubnetConsensusSubmission, SubnetElectedValidator,
    SubnetName, SubnetPenaltyCount, TotalSubnetDelegateStakeBalance,
};
use frame_support::assert_ok;
use frame_support::traits::OnInitialize;
use sp_std::collections::btree_map::BTreeMap;

//
//
//
//
//
//
//
// On Initialize Hook
//
//
//
//
//
//
//

/// Verifies:
/// - Emmissions to nodes
/// - Subnets stay active

// Helper to change the overwatch weights
fn is_even(num: u32) -> bool {
    if num % 2 == 0 {
        return true;
    }
    return false;
}

// Simulated commit that bounces between 1e18 and 0.5e18
fn get_commit(num: u32) -> (u128, Vec<u8>, sp_core::H256) {
    // default onode weights
    let weights: Vec<u128> = vec![1000000000000000000, 500000000000000000];

    let mut weight: u128 = 1000000000000000000;
    if is_even(num) {
        weight = weights[0];
    } else {
        weight = weights[1];
    }
    let salt: Vec<u8> = b"secret-salt".to_vec();
    let commit_hash = make_commit(weight, salt.clone());

    (weight, salt, commit_hash)
}

#[test]
fn test_on_initialize() {
    new_test_ext().execute_with(|| {
        NewRegistrationCostMultiplier::<Test>::put(1200000000000000000);
        OverwatchEpochLengthMultiplier::<Test>::set(2);
        let alice = 0;

        let max_onodes = MaxOverwatchNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let min_subnet_nodes = MinSubnetNodes::<Test>::get();
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let deposit_amount: u128 = get_min_stake_balance() + 500;
        let amount: u128 = get_min_stake_balance();

        let node_count = min_subnet_nodes;
        let end = node_count;
        let overwatch_count = 16;

        // Register max subnets and subnet nodes
        build_activated_subnet_with_overwatch_nodes_v2(
            0,
            end,
            overwatch_count,
            deposit_amount,
            amount,
        );

        // default onode weights
        let weights: Vec<u128> = vec![1000000000000000000, 500000000000000000];

        let epoch_length = EpochLength::get();
        let multiplier = OverwatchEpochLengthMultiplier::<Test>::get();
        let overwatch_epoch_length = epoch_length.saturating_mul(multiplier);

        let overwatch_epochs = 2;
        let total_epochs = overwatch_epochs * multiplier;

        let mut epochs_complete = 0;
        let mut overwatch_epochs_complete = 0;

        let mut last_overwatch_commit_epoch = u32::MAX;
        let mut last_overwatch_reveal_epoch = u32::MAX;

        let mut snodes_rewarded = false;
        let mut overwatch_rewarded = false;

        let mut calculate_overwatch_rewards_v3_ran = false;
        let mut do_epoch_preliminaries_ran = false;
        let mut handle_subnet_emission_weights_ran = false;
        let mut emission_step_ran = false;
        let mut do_epoch_preliminaries_touched = false;
        let mut handle_subnet_emission_weights_touched = false;
        let mut emission_step_touched = false;
        let mut weight_change_verified = false;
        let mut commits_checked = false;
        let mut reveals_checked = false;

        let mut ow_commits = 0;
        let mut ow_reveals = 0;

        let block = System::block_number();
        let start_epoch = block.saturating_div(epoch_length);
        let start_overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        for _ in 0..(overwatch_epochs * epoch_length * multiplier) {
            let block = System::block_number();

            let current_epoch = block.saturating_div(epoch_length);
            let current_overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();
            let epoch_slot = block % epoch_length;

            // Run Overwatch reveals from the previous epoch and generate rewards
            // Check that rewards were generated
            if (block - 1) >= overwatch_epoch_length && (block - 1) % overwatch_epoch_length == 0 {
                let mut ostake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
                    BTreeMap::new();

                // Get overwatch node balances pre-emissions
                if last_overwatch_reveal_epoch > 0
                    && current_overwatch_epoch > last_overwatch_reveal_epoch
                {
                    for n in end - 1..end + overwatch_count {
                        let _n = n + 1;
                        let o_n = _n - end + 1;
                        let coldkey =
                            get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
                        let hotkey =
                            get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);
                        let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

                        assert_ne!(overwatch_stake, 0);
                        ostake_snapshot.insert(hotkey.clone(), overwatch_stake);
                    }
                }

                // calculate_overwatch_rewards();
                Network::on_initialize(block);

                // Make sure rewards were given to overwatch nodes
                if last_overwatch_reveal_epoch > 0
                    && current_overwatch_epoch > last_overwatch_reveal_epoch
                {
                    calculate_overwatch_rewards_v3_ran = true;
                    for n in end - 1..end + overwatch_count {
                        let _n = n + 1;
                        let o_n = _n - end + 1;
                        let coldkey =
                            get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
                        let hotkey =
                            get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);
                        let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

                        if let Some(old_stake) = ostake_snapshot.get(&hotkey) {
                            assert!(overwatch_stake > *old_stake);
                            if !overwatch_rewarded {
                                overwatch_rewarded = true;
                            }
                        } else {
                            assert!(false); // auto-fail
                        }
                    }
                }
            }

            // Overwatch
            let in_commit_period = Network::in_overwatch_commit_period();

            // Commit overwatch weights for all subnets
            if in_commit_period && last_overwatch_commit_epoch != current_overwatch_epoch {
                ow_commits += 1;

                let mut commits = Vec::new();
                for s in 0..max_subnets {
                    let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
                    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
                    let (weight, salt, commit_hash) = get_commit(s);

                    let commit = OverwatchCommit {
                        subnet_id: subnet_id,
                        weight: commit_hash,
                    };
                    commits.push(commit);
                }

                // Submit extrinsics for each node
                for n in end - 1..end + overwatch_count {
                    let _n = n + 1;
                    let o_n = _n - end + 1;
                    let coldkey =
                        get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
                    let hotkey =
                        get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);
                    let overwatch_node_id =
                        HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();
                    assert_ok!(Network::commit_overwatch_subnet_weights(
                        RuntimeOrigin::signed(hotkey.clone()),
                        overwatch_node_id,
                        commits.clone()
                    ));

                    for s in 0..max_subnets {
                        let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
                        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
                        let (weight, salt, commit_hash) = get_commit(s);

                        let stored = OverwatchCommits::<Test>::get((
                            current_overwatch_epoch,
                            overwatch_node_id,
                            subnet_id,
                        ))
                        .unwrap();
                        assert_eq!(stored, commit_hash);
                        if !commits_checked {
                            commits_checked = true;
                        }
                    }
                }

                last_overwatch_commit_epoch = current_overwatch_epoch;
            } else if !in_commit_period && last_overwatch_reveal_epoch != current_overwatch_epoch {
                ow_reveals += 1;
                let mut reveals = Vec::new();
                for s in 0..max_subnets {
                    let (weight, salt, commit_hash) = get_commit(s);

                    let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
                    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
                    let reveal = OverwatchReveal {
                        subnet_id: subnet_id,
                        weight: weight,
                        salt: salt.clone(),
                    };
                    reveals.push(reveal);
                }

                for n in end - 1..end + overwatch_count {
                    let _n = n + 1;
                    let o_n = _n - end + 1;
                    let coldkey =
                        get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
                    let hotkey =
                        get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);
                    let overwatch_node_id =
                        HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();
                    assert_ok!(Network::reveal_overwatch_subnet_weights(
                        RuntimeOrigin::signed(hotkey.clone()),
                        overwatch_node_id,
                        reveals.clone()
                    ));
                    for s in 0..max_subnets {
                        let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
                        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
                        let (weight, salt, commit_hash) = get_commit(s);

                        let revealed = OverwatchReveals::<Test>::get((
                            current_overwatch_epoch,
                            subnet_id,
                            overwatch_node_id,
                        ))
                        .unwrap();
                        assert_eq!(revealed, weight);
                        if !reveals_checked {
                            reveals_checked = true;
                        }
                    }
                }

                last_overwatch_reveal_epoch = current_overwatch_epoch;
                overwatch_epochs_complete += 1;
            }

            if block >= epoch_length && block % epoch_length == 0 {
                for s in 0..max_subnets {
                    // - Subnets must have min dstake at this time in the block steps
                    let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
                    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
                    let total_delegate_stake_balance =
                        TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
                    let mut min_subnet_delegate_stake =
                        Network::get_min_subnet_delegate_stake_balance(subnet_id);
                    if total_delegate_stake_balance < min_subnet_delegate_stake {
                        let mut delta = min_subnet_delegate_stake - total_delegate_stake_balance;
                        assert_ok!(Network::add_to_delegate_stake(
                            RuntimeOrigin::signed(account(alice)),
                            subnet_id,
                            delta,
                        ));
                    }

                    // Get subnet epoch, is previous epoch because we're on block step 0
                    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
                    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
                    assert!(epochs_complete == 0 || validator_id != None);
                    if epochs_complete > 0 {
                        assert_ne!(validator_id, None);
                    }
                    if validator_id != None {
                        // Run attestation proposal and attestation voting
                        run_subnet_consensus_step(subnet_id, None, None);
                    }
                }
                // Remove unqualified subnets
                // Network::do_epoch_preliminaries(block, current_epoch);
                Network::on_initialize(block);
                do_epoch_preliminaries_touched = true;
                epochs_complete += 1;
            } else if (block - 2) >= epoch_length && (block - 2) % epoch_length == 0 {
                // Network::handle_subnet_emission_weights(current_epoch);
                Network::on_initialize(block);

                // - Ensure `handle_subnet_emission_weights` ran
                let subnet_emission_weights =
                    FinalSubnetEmissionWeights::<Test>::get(current_epoch);

                let mut prev_subnet_weight = 0;
                for s in 0..max_subnets {
                    let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
                    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
                    let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
                    assert!(subnet_weight.is_some());

                    let (weight, salt, commit_hash) = get_commit(s);

                    // Ensure the overwatch nodes are impacting the subnet weights

                    // This test is designed that even numbers have higher weights
                    // This is basic and simple check it works, other tests accomplish accuracy
                    // The overwatch weights are from the previous overwatch epoch
                    if overwatch_epochs_complete > 0 {
                        if is_even(s) {
                            let w = *subnet_weight.unwrap();
                            assert!(w > prev_subnet_weight);
                            prev_subnet_weight = w;
                        } else {
                            let w = *subnet_weight.unwrap();
                            assert!(w < prev_subnet_weight);
                            prev_subnet_weight = w;
                        }
                        weight_change_verified = true;
                    }
                }
                handle_subnet_emission_weights_touched = true;
            } else if let Some(subnet_id) = SlotAssignment::<Test>::get(epoch_slot) {
                let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

                let submission =
                    SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch - 1);
                if epochs_complete > 1 {
                    assert!(submission != None);
                }
                let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
                    BTreeMap::new();
                if submission != None {
                    for n in 0..end {
                        let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, n + 1);

                        let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

                        assert_ne!(stake, 0);
                        stake_snapshot.insert(hotkey.clone(), stake);
                    }
                }

                // Network::emission_step(block, current_epoch, subnet_epoch, subnet_id);
                Network::on_initialize(block);
                emission_step_touched = true;

                // - Ensure rewards were distributed
                if submission != None {
                    for n in 0..end {
                        let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, n + 1);

                        let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

                        if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                            assert!(stake > *old_stake);
                            if !snodes_rewarded {
                                snodes_rewarded = true;
                            }
                        } else {
                            assert!(false); // auto-fail
                        }
                    }
                }
            }

            // becuase of the way this test is set up, the subnets will accrue 1 penalty each
            for s in 0..max_subnets {
                let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
                let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
                assert!(SubnetPenaltyCount::<Test>::get(subnet_id) <= 1);
            }

            System::set_block_number(block + 1);
        }

        assert_eq!(total_epochs, epochs_complete);
        assert!(last_overwatch_commit_epoch > 0);
        assert!(last_overwatch_reveal_epoch > 0);
        assert!(snodes_rewarded);
        assert!(do_epoch_preliminaries_touched);
        assert!(emission_step_touched);
        assert!(handle_subnet_emission_weights_touched);

        assert_ne!(ow_commits, 0);
        assert_ne!(ow_reveals, 0);

        assert_eq!(overwatch_epochs_complete, overwatch_epochs);
        assert_eq!(ow_commits, overwatch_epochs);
        assert_eq!(ow_commits, ow_reveals);

        assert!(commits_checked);
        assert!(reveals_checked);

        // This will fail if overwatch_epochs <= 1
        assert!(calculate_overwatch_rewards_v3_ran);
        assert!(overwatch_rewarded);

        // This will fail is we are doing  <=1 overwatch epoch
        assert!(weight_change_verified);

        for s in 0..max_subnets {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
            // - Ensure subnet is present. `unwrap` will panic is doesn't exist
            let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        }
    });
}

#[test]
fn test_on_initialize_verify_overwatch_weights() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn test_on_initialize_verify_unpause_queue() {
    new_test_ext().execute_with(|| {});
}
