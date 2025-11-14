use super::mock::*;
use crate::tests::test_utils::*;
use crate::{
    AccountOverwatchStake, FinalSubnetEmissionWeights, HotkeySubnetNodeId, MaxSubnetNodes,
    MaxSubnets, MinSubnetMinStake, NewRegistrationCostMultiplier, OverwatchReveals,
    QueueImmunityEpochs, RegisteredSubnetNodesData, SubnetConsensusSubmission,
    SubnetDelegateStakeRewardsPercentage, SubnetElectedValidator, SubnetName, SubnetNodeQueue,
    TotalActiveSubnets,
};
use sp_std::collections::btree_map::BTreeMap;

// Overwatch node functions in the slot.rs file are in tests/overwatch_nodes.rs

// calculate_overwatch_rewards: test_calculate_overwatch_rewards
// emission_step: See incentives_protocol.rs
// 	- test_distribute_rewards_prioritized_queue_node_id: Tests node registration queue prioritization
// 	- test_distribute_rewards_remove_queue_node_id: Tests removing node from registration
// 	- test_distribute_rewards_graduate_idle_to_included: Tests graduating nodes
// 	- test_distribute_rewards_graduate_included_to_validator: Tests emissions generation
// handle_registration_queue
// See:
//  - test_distribute_rewards_prioritized_queue_node_id
//  - test_distribute_rewards_remove_queue_node_id
// handle_subnet_emission_weights: test_handle_subnet_emission_weights
// calculate_subnet_weights: test_calculate_subnet_weights
// precheck_subnet_consensus_submission: test_precheck_subnet_consensus_submission
// calculate_rewards: test_calculate_rewards

#[test]
fn test_calculate_overwatch_rewards() {
    new_test_ext().execute_with(|| {
        NewRegistrationCostMultiplier::<Test>::set(1000000000000000000);

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        let end = 4;

        for s in 0..max_subnets {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
            build_activated_subnet_new(subnet_name.clone().into(), 0, end, deposit_amount, amount);
        }

        set_overwatch_epoch(1);

        let default_weight = 1000000000000000000; // 1.0
        let overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();

        let overwatch_nodes = 4;
        for o in 0..overwatch_nodes {
            insert_overwatch_node(o, o);
            set_overwatch_stake(o, 100);
        }

        let mut ostake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..overwatch_nodes {
            let hotkey = account(n);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

            assert_ne!(overwatch_stake, 0);
            ostake_snapshot.insert(hotkey.clone(), overwatch_stake);
        }

        for s in 0..max_subnets {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
            let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

            for o in 0..overwatch_nodes {
                let node_id = o + 1;
                submit_weight(overwatch_epoch, subnet_id, node_id, default_weight);
            }
        }

        // increase one overwatch epoch
        set_overwatch_epoch(overwatch_epoch + 1);

        assert!(overwatch_epoch < Network::get_current_overwatch_epoch_as_u32());

        let reveals = OverwatchReveals::<Test>::iter_prefix((
            Network::get_current_overwatch_epoch_as_u32().saturating_sub(1),
        ));
        assert!(
            reveals.count() > 0,
            "No reveals found for the previous epoch"
        );

        for s in 0..max_subnets {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
            let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

            // Check if there's at least one reveal for this subnet in the previous epoch
            let has_reveal = OverwatchReveals::<Test>::iter_prefix((
                Network::get_current_overwatch_epoch_as_u32().saturating_sub(1),
                subnet_id,
            ))
            .next()
            .is_some();

            assert!(has_reveal, "No reveal found for subnet {}", subnet_id);
        }

        Network::calculate_overwatch_rewards();

        for n in 0..overwatch_nodes {
            let hotkey = account(n);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

            if let Some(old_stake) = ostake_snapshot.get(&hotkey) {
                assert!(overwatch_stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }
    });
}

// #[test]
// fn test_emission_step() {
//   new_test_ext().execute_with(|| {
// 		// See:
// 		// 	test_distribute_rewards_prioritized_queue_node_id: Tests node registration queue prioritization
// 		// 	test_distribute_rewards_remove_queue_node_id: Tests removing node from registration
// 		// 	test_distribute_rewards_graduate_idle_to_included: Tests graduating nodes
// 		// 	test_distribute_rewards_graduate_included_to_validator: Tests emissions generation
// 	});
// }

#[test]
fn test_handle_subnet_emission_weights() {
    new_test_ext().execute_with(|| {
        NewRegistrationCostMultiplier::<Test>::set(1000000000000000000);

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 12;

        for s in 0..max_subnets {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
            build_activated_subnet_new(subnet_name.clone().into(), 0, end, deposit_amount, amount);
        }

        let _ = Network::handle_subnet_emission_weights(Network::get_current_epoch_as_u32());

        let subnet_emission_weights =
            FinalSubnetEmissionWeights::<Test>::get(Network::get_current_epoch_as_u32());

        for s in 0..max_subnets {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
            let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

            let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
            assert!(subnet_weight.is_some());
            assert!(*subnet_weight.unwrap() > 0);
            assert!(*subnet_weight.unwrap() <= Network::percentage_factor_as_u128());
        }
    });
}

#[test]
fn test_calculate_subnet_weights() {
    new_test_ext().execute_with(|| {
        NewRegistrationCostMultiplier::<Test>::set(1000000000000000000);

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 12;

        for s in 0..max_subnets {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
            build_activated_subnet_new(subnet_name.clone().into(), 0, end, deposit_amount, amount);
        }

        let (subnet_weights, mut weight) =
            Network::calculate_subnet_weights(Network::get_current_epoch_as_u32());

        for s in 0..max_subnets {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
            let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

            let subnet_weight = subnet_weights.get(&subnet_id);
            assert!(subnet_weight.is_some());
            assert!(*subnet_weight.unwrap() > 0);
            assert!(*subnet_weight.unwrap() <= Network::percentage_factor_as_u128());
        }
    });
}

#[test]
fn test_precheck_subnet_consensus_submission() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let new_start = end + 1;
        let new_end = new_start + 4;
        build_registered_nodes_in_queue(subnet_id, new_start, new_end, deposit_amount, amount);

        QueueImmunityEpochs::<Test>::insert(subnet_id, 1);

        // Push passed immunity period so node can be removed from queue
        let immunity_epochs = QueueImmunityEpochs::<Test>::get(subnet_id);
        increase_epochs(immunity_epochs + 1);

        // Store data
        let mut registered_nodes_data: BTreeMap<u32, u32> = BTreeMap::new(); // node ID => start_epoch
        for n in new_start..new_end {
            let _n = n + 1;
            let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
            let subnet_node_data =
                RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id)
                    .unwrap();
            registered_nodes_data.insert(
                hotkey_subnet_node_id,
                subnet_node_data.classification.start_epoch,
            );
        }

        let queue = SubnetNodeQueue::<Test>::get(subnet_id);
        assert_eq!(queue.len() as u32, new_end - new_start);

        let first = queue.first().unwrap();
        let last = queue.last().unwrap();
        // Sanity check
        assert_ne!(first.id, last.id);

        let exists = queue.iter().any(|node| node.id == last.id);

        set_block_to_subnet_slot_epoch(Network::get_current_epoch_as_u32(), subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        Network::elect_validator(subnet_id, subnet_epoch, System::block_number());
        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        run_subnet_consensus_step(subnet_id, Some(last.id), Some(first.id));

        let submission = SubnetConsensusSubmission::<Test>::get(
            subnet_id,
            Network::get_current_subnet_epoch_as_u32(subnet_id),
        );
        assert!(submission
            .clone()
            .unwrap()
            .prioritize_queue_node_id
            .is_some());
        assert_eq!(
            submission
                .clone()
                .unwrap()
                .prioritize_queue_node_id
                .unwrap(),
            last.id
        );
        assert!(submission.clone().unwrap().remove_queue_node_id.is_some());
        assert_eq!(
            submission.clone().unwrap().remove_queue_node_id.unwrap(),
            first.id
        );

        increase_epochs(1);
        set_block_to_subnet_slot_epoch(Network::get_current_epoch_as_u32(), subnet_id);

        let (consensus_submission_data, consensus_submission_block_weight) =
            Network::precheck_subnet_consensus_submission(
                subnet_id,
                Network::get_current_epoch_as_u32() - 1,
                Network::get_current_epoch_as_u32(),
            );

        let consensus_results = consensus_submission_data.unwrap();

        let validator_subnet_node_id = consensus_results.validator_subnet_node_id;
        let attestation_ratio = consensus_results.attestation_ratio;
        let weight_sum = consensus_results.weight_sum;
        let data_length = consensus_results.data_length;
        let data = consensus_results.data;
        let attests = consensus_results.attests;
        let subnet_nodes = consensus_results.subnet_nodes;
        let prioritize_queue_node_id = consensus_results.prioritize_queue_node_id;
        let remove_queue_node_id = consensus_results.remove_queue_node_id;

        assert_eq!(validator_subnet_node_id, validator_id.unwrap());
        assert_eq!(attestation_ratio, Network::percentage_factor_as_u128());
        assert_ne!(weight_sum, 0);
        assert_eq!(data_length, end);
        // assert_eq!(data, Network::percentage_factor_as_u128());
        assert_eq!(attests.len(), end as usize);
        assert_eq!(subnet_nodes.len(), end as usize);
        assert_eq!(prioritize_queue_node_id, Some(last.id));
        assert_eq!(remove_queue_node_id, Some(first.id));
    });
}

#[test]
fn test_calculate_rewards() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let _ = Network::handle_subnet_emission_weights(Network::get_current_epoch_as_u32());

        let subnet_emission_weights =
            FinalSubnetEmissionWeights::<Test>::get(Network::get_current_epoch_as_u32());
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);

        let delegate_stake_rewards_percentage =
            SubnetDelegateStakeRewardsPercentage::<Test>::get(subnet_id);

        let (rewards_data, rewards_block_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let overall_subnet_reward = rewards_data.overall_subnet_reward;
        let subnet_owner_reward = rewards_data.subnet_owner_reward;
        let subnet_rewards = rewards_data.subnet_rewards;
        let delegate_stake_rewards = rewards_data.delegate_stake_rewards;
        let subnet_node_rewards = rewards_data.subnet_node_rewards;

        let expected_delegate_stake_rewards: u128 =
            Network::percent_mul(subnet_rewards, delegate_stake_rewards_percentage);
        let expected_subnet_node_rewards: u128 =
            subnet_rewards.saturating_sub(expected_delegate_stake_rewards);

        assert!(overall_subnet_reward > 0);
        assert!(subnet_owner_reward > 0);
        assert!(subnet_rewards > 0);
        assert_eq!(delegate_stake_rewards, expected_delegate_stake_rewards);
        assert_eq!(subnet_node_rewards, expected_subnet_node_rewards);
    });
}
