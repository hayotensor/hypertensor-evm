use super::mock::*;
use super::test_utils::*;
use crate::Event;
use crate::{
    AccountSubnetDelegateStakeShares, AccountSubnetStake, BaseValidatorReward,
    ColdkeyReputationDecreaseFactor, ColdkeyReputationIncreaseFactor,
    EmergencySubnetNodeElectionData, Error, FinalSubnetEmissionWeights, HotkeySubnetNodeId,
    IdleClassificationEpochs, IncludedClassificationEpochs, MaxSubnetNodes, MaxSubnets,
    MinAttestationPercentage, MinSubnetMinStake, MinSubnetNodeReputation, MinSubnetReputation,
    NodeDelegateStakeBalance, QueueImmunityEpochs, RegisteredSubnetNodesData,
    SubnetConsensusSubmission, SubnetElectedValidator, SubnetName, SubnetNodeClass,
    SubnetNodeConsecutiveIncludedEpochs, SubnetNodeIdHotkey, SubnetNodeIdleConsecutiveEpochs,
    SubnetNodeMinWeightDecreaseReputationThreshold, SubnetNodeQueue, SubnetNodeQueueEpochs,
    SubnetNodeReputation, SubnetNodesData, SubnetOwner, SubnetPauseCooldownEpochs,
    SubnetRemovalReason, SubnetReputation, SubnetState, SubnetsData, SuperMajorityAttestationRatio,
    TotalActiveSubnets, TotalNodeDelegateStakeShares, TotalSubnetDelegateStakeBalance,
    TotalSubnetNodes, ValidatorAbsentSubnetNodeReputationFactor,
    ValidatorAbsentSubnetReputationFactor,
};
use frame_support::traits::Currency;
use frame_support::weights::WeightMeter;
use frame_support::{assert_err, assert_ok};
use sp_std::collections::btree_map::BTreeMap;

//
//
//
//
//
//
//
// Validate / Attest / Rewards
//
//
//
//
//
//
//

// Validate

#[test]
fn test_propose_attestation() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let end = 12;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let blockchain_epoch = Network::get_current_epoch_as_u32();
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");

        // Unwrap will panic if None
        let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(hotkey.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch).unwrap();

        assert_eq!(
            *network_events().last().unwrap(),
            Event::ValidatorSubmission {
                subnet_id,
                account_id: hotkey.clone(),
                epoch: subnet_epoch,
            }
        );

        assert_eq!(
            submission.validator_id,
            validator_id.unwrap(),
            "Err: validator"
        );
        assert_eq!(
            submission.data.len(),
            subnet_node_data_vec.len(),
            "Err: data len"
        );
        let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
        assert_eq!(sum, DEFAULT_SCORE * total_subnet_nodes as u128, "Err: sum");
        assert_eq!(submission.attests.len(), 1, "Err: attests"); // validator auto-attests
        assert_eq!(
            submission.subnet_nodes.len() as u32,
            end,
            "Err: Nodes length"
        );

        for node_id in submission.subnet_nodes.iter() {
            let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_id);
            assert!(subnet_node.has_classification(&SubnetNodeClass::Included, subnet_epoch));
            assert_ne!(
                subnet_node.classification.node_class,
                SubnetNodeClass::Registered
            );
            assert_ne!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
        }

        assert_err!(
            Network::propose_attestation(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                subnet_node_data_vec.clone(),
                None,
                None,
                None,
                None,
            ),
            Error::<Test>::SubnetRewardsAlreadySubmitted
        );
    });
}

#[test]
fn test_validator_absent_propose_attestation_decrease_reputation() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let end = 12;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        increase_epochs(1);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        ValidatorAbsentSubnetReputationFactor::<Test>::set(50000000000000000);
        ValidatorAbsentSubnetNodeReputationFactor::<Test>::insert(subnet_id, 50000000000000000);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let blockchain_epoch = Network::get_current_epoch_as_u32();
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");

        // Unwrap will panic if None
        let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let starting_subnet_rep = SubnetReputation::<Test>::get(subnet_id);
        let starting_node_rep = SubnetNodeReputation::<Test>::get(subnet_id, validator_id.unwrap());

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
        assert!(subnet.start_epoch < epoch);

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(starting_subnet_rep > SubnetReputation::<Test>::get(subnet_id));
        assert!(
            starting_node_rep > SubnetNodeReputation::<Test>::get(subnet_id, validator_id.unwrap())
        );
    });
}

// #[test]
// fn test_validator_absent_propose_attestation_decrease_reputation() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 12;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         increase_epochs(1);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         ValidatorAbsentSubnetReputationFactor::<Test>::set(50000000000000000);
//         ValidatorAbsentSubnetNodeReputationFactor::<Test>::insert(subnet_id, 50000000000000000);

//         let epoch_length = EpochLength::get();
//         let block_number = System::block_number();
//         let epoch = block_number / epoch_length;

//         set_block_to_subnet_slot_epoch(epoch, subnet_id);

//         let blockchain_epoch = Network::get_current_epoch_as_u32();
//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         Network::elect_validator(subnet_id, subnet_epoch, block_number);

//         let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
//         assert!(validator_id != None, "Validator is None");

//         // Unwrap will panic if None
//         let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

//         let starting_subnet_rep = SubnetReputation::<Test>::get(subnet_id);
//         let starting_node_rep = SubnetNodeReputation::<Test>::get(subnet_id, validator_id.unwrap());

//         increase_epochs(1);
//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
//         let epoch = Network::get_current_epoch_as_u32();

//         let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
//         assert!(subnet.start_epoch < epoch);

//         // ⸺ Generate subnet weights from stake/node count weights
//         let _ = Network::handle_subnet_emission_weights(epoch);
//         let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

//         let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
//         assert!(subnet_weight.is_some());

//         let (result, block_weight) = Network::precheck_subnet_consensus_submission(
//             subnet_id,
//             subnet_epoch - 1,
//             Network::get_current_epoch_as_u32(),
//         );

//         assert!(starting_subnet_rep > SubnetReputation::<Test>::get(subnet_id));
//         assert!(
//             starting_node_rep > SubnetNodeReputation::<Test>::get(subnet_id, validator_id.unwrap())
//         );
//     });
// }

#[test]
fn test_propose_attestation_no_validator_elected_error() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        build_activated_subnet_new(subnet_name.clone(), 0, 12, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

        assert_err!(
            Network::propose_attestation(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                Vec::new(),
                None,
                None,
                None,
                None,
            ),
            Error::<Test>::NoElectedValidator
        );
    });
}

#[test]
fn test_propose_attestation_after_slot_error() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        build_activated_subnet_new(subnet_name.clone(), 0, 12, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");

        let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap() + 1).unwrap();

        assert_err!(
            Network::propose_attestation(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                subnet_node_data_vec.clone(),
                None,
                None,
                None,
                None,
            ),
            Error::<Test>::InvalidValidator
        );
    });
}

#[test]
fn test_propose_attestation_score_overflow_error() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        build_activated_subnet_new(subnet_name.clone(), 0, 12, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let subnet_node_data_vec = get_subnet_node_consensus_data_with_custom_score(
            subnets,
            max_subnet_nodes,
            0,
            total_subnet_nodes,
            u128::MAX,
        );

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");

        let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        assert_err!(
            Network::propose_attestation(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                subnet_node_data_vec.clone(),
                None,
                None,
                None,
                None,
            ),
            Error::<Test>::ScoreOverflow
        );
    });
}

#[test]
fn test_propose_attestation_invalid_validator() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        if validator.clone() == account(1) {
            validator = account(2);
        }

        assert_err!(
            Network::propose_attestation(
                RuntimeOrigin::signed(account(1)),
                subnet_id,
                subnet_node_data_vec,
                None,
                None,
                None,
                None,
            ),
            Error::<Test>::InvalidValidator
        );
    });
}

// Attest

#[test]
fn test_attest() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch).unwrap();

        assert_eq!(submission.validator_id, validator_id.unwrap());
        assert_eq!(submission.data.len(), subnet_node_data_vec.len());

        // Attest
        for n in 1..total_subnet_nodes + 1 {
            let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
            if attestor == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(attestor.clone()),
                subnet_id,
                None,
            ));

            assert_eq!(
                *network_events().last().unwrap(),
                Event::Attestation {
                    subnet_id,
                    subnet_node_id: n,
                    epoch: subnet_epoch
                }
            );
        }

        let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, epoch).unwrap();
        assert_eq!(submission.attests.len(), total_subnet_nodes as usize);

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, _n).unwrap();
            if attestor == validator.clone() {
                assert_ne!(submission.attests.get(&(_n)), None);
                assert_eq!(
                    submission.attests.get(&(_n)).unwrap().block,
                    System::block_number()
                );
            } else {
                assert_ne!(submission.attests.get(&(_n)), None);
                assert_eq!(
                    submission.attests.get(&(_n)).unwrap().block,
                    System::block_number()
                );
            }
        }
    });
}

#[test]
fn test_attest_last_block_error() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch).unwrap();

        assert_eq!(submission.validator_id, validator_id.unwrap());
        assert_eq!(submission.data.len(), subnet_node_data_vec.len());

        System::set_block_number(System::block_number() + epoch_length);

        // Attest failure
        if submission.validator_id == max_subnet_nodes {
            let _n = submission.validator_id - 1;
            let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, _n).unwrap();
            assert_err!(
                Network::attest(RuntimeOrigin::signed(attestor), subnet_id, None),
                Error::<Test>::InvalidSubnetConsensusSubmission
            );
        } else {
            let _n = submission.validator_id + 1;
            let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, _n).unwrap();
            assert_err!(
                Network::attest(RuntimeOrigin::signed(validator), subnet_id, None),
                Error::<Test>::InvalidSubnetConsensusSubmission
            );
        }
    });
}

#[test]
fn test_attest_no_submission_err() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        // --- Get validator
        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        assert_err!(
            Network::attest(RuntimeOrigin::signed(validator), subnet_id, None),
            Error::<Test>::InvalidSubnetConsensusSubmission
        );
    });
}

#[test]
fn test_attest_already_attested_err() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        // Attest
        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch).unwrap();

        assert_eq!(submission.validator_id, validator_id.unwrap());
        assert_eq!(submission.data.len(), subnet_node_data_vec.len());
        let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
        assert_eq!(sum, DEFAULT_SCORE * total_subnet_nodes as u128);
        assert_eq!(submission.attests.len(), total_subnet_nodes as usize);

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ne!(submission.attests.get(&(_n)), None);
            assert_eq!(
                submission.attests.get(&(_n)).unwrap().block,
                System::block_number()
            );

            // assert_ne!(submission.attests.get(&_n), None);
            // assert_eq!(submission.attests.get(&_n), Some(&System::block_number()));
        }

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_err!(
                Network::attest(RuntimeOrigin::signed(hotkey.clone()), subnet_id, None),
                Error::<Test>::AlreadyAttested
            );
        }
    });
}

//
//
//
//
//
//
//
// Rewards
//
//
//
//
//
//
//

#[test]
fn test_distribute_rewards() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();
        assert_eq!(
            consensus_submission_data.clone().validator_subnet_node_id,
            validator_id.unwrap()
        );
        assert_eq!(
            consensus_submission_data.clone().validator_epoch_progress,
            0
        );
        assert!(consensus_submission_data.clone().validator_reward_factor > 990000000000000000);
        assert_eq!(
            consensus_submission_data.clone().attestation_ratio,
            1000000000000000000
        );
        assert_eq!(
            consensus_submission_data.clone().weight_sum,
            500000000000000000 * max_subnet_nodes as u128
        );
        assert_eq!(
            consensus_submission_data.clone().data_length,
            max_subnet_nodes
        );
        assert_eq!(
            consensus_submission_data.clone().data,
            subnet_node_data_vec.clone()
        );
        assert_eq!(
            consensus_submission_data.clone().attests.len(),
            max_subnet_nodes as usize
        );
        assert_eq!(
            consensus_submission_data.clone().subnet_nodes.len(),
            max_subnet_nodes as usize
        );
        assert_eq!(
            consensus_submission_data.clone().prioritize_queue_node_id,
            None
        );
        assert_eq!(consensus_submission_data.clone().remove_queue_node_id, None);

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let subnet_rewards = rewards_data.subnet_rewards;

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let block_number = System::block_number();
        let dstake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        let set_rep = 500000000000000000;
        SubnetReputation::<Test>::insert(subnet_id, set_rep);

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data.clone(),
            rewards_data.clone(),
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let total_weight = DEFAULT_SCORE * total_subnet_nodes as u128;
        let node_weight = Network::percent_div(DEFAULT_SCORE, total_weight as u128);
        let expected_node_reward =
            Network::percent_mul(node_weight, rewards_data.clone().subnet_node_rewards);

        let post_validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        let expected_validator_reward = Network::percent_mul(
            BaseValidatorReward::<Test>::get(),
            consensus_submission_data.clone().validator_reward_factor,
        );
        assert_eq!(
            validator_stake + expected_validator_reward + expected_node_reward,
            post_validator_stake
        );

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            if hotkey.clone() == validator.clone() {
                continue;
            }

            let subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let reward_factor = match consensus_submission_data.attests.get(&subnet_node_id) {
                Some(data) => data.reward_factor,
                None => return assert!(false),
            };

            assert_eq!(reward_factor, Network::percentage_factor_as_u128());

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert!(stake > *old_stake);
                assert_eq!(stake, *old_stake + expected_node_reward);
            } else {
                assert!(false); // auto-fail
            }
        }

        let post_dstake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        assert!(post_dstake_balance > dstake_balance);

        assert!(SubnetReputation::<Test>::get(subnet_id) > set_rep);
    });
}

#[test]
fn test_distribute_rewards_fork() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();
        assert_eq!(
            consensus_submission_data.clone().validator_subnet_node_id,
            validator_id.unwrap()
        );
        assert_eq!(
            consensus_submission_data.clone().validator_epoch_progress,
            0
        );
        assert!(consensus_submission_data.clone().validator_reward_factor > 990000000000000000);
        assert_eq!(
            consensus_submission_data.clone().attestation_ratio,
            1000000000000000000
        );
        assert_eq!(
            consensus_submission_data.clone().weight_sum,
            500000000000000000 * max_subnet_nodes as u128
        );
        assert_eq!(
            consensus_submission_data.clone().data_length,
            max_subnet_nodes
        );
        assert_eq!(
            consensus_submission_data.clone().data,
            subnet_node_data_vec.clone()
        );
        assert_eq!(
            consensus_submission_data.clone().attests.len(),
            max_subnet_nodes as usize
        );
        assert_eq!(
            consensus_submission_data.clone().subnet_nodes.len(),
            max_subnet_nodes as usize
        );
        assert_eq!(
            consensus_submission_data.clone().prioritize_queue_node_id,
            None
        );
        assert_eq!(consensus_submission_data.clone().remove_queue_node_id, None);

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let subnet_rewards = rewards_data.subnet_rewards;

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let block_number = System::block_number();
        let dstake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);

        SubnetReputation::<Test>::insert(subnet_id, 500000000000000000);
        let starting_rep = SubnetReputation::<Test>::get(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data.clone(),
            rewards_data.clone(),
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let total_weight = DEFAULT_SCORE * total_subnet_nodes as u128;
        let node_weight = Network::percent_div(DEFAULT_SCORE, total_weight as u128);
        let expected_node_reward =
            Network::percent_mul(node_weight, rewards_data.clone().subnet_node_rewards);

        let post_validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        let expected_validator_reward = Network::percent_mul(
            BaseValidatorReward::<Test>::get(),
            consensus_submission_data.clone().validator_reward_factor,
        );
        assert_eq!(
            validator_stake + expected_validator_reward + expected_node_reward,
            post_validator_stake
        );

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            if hotkey.clone() == validator.clone() {
                continue;
            }

            let subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let reward_factor = match consensus_submission_data.attests.get(&subnet_node_id) {
                Some(data) => data.reward_factor,
                None => return assert!(false),
            };

            assert_eq!(reward_factor, Network::percentage_factor_as_u128());

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert!(stake > *old_stake);
                assert_eq!(stake, *old_stake + expected_node_reward);
            } else {
                assert!(false); // auto-fail
            }
        }

        let post_dstake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        assert!(post_dstake_balance > dstake_balance);

        assert!(SubnetReputation::<Test>::get(subnet_id) > starting_rep);
    });
}

#[test]
fn test_distribute_rewards_prioritized_queue_node_id() {
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
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let new_start = end + 1;
        let new_end = new_start + 4;
        build_registered_nodes_in_queue(subnet_id, new_start, new_end, deposit_amount, amount);

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

        run_subnet_consensus_step(subnet_id, Some(last.id), None);

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

        increase_epochs(1);
        set_block_to_subnet_slot_epoch(Network::get_current_epoch_as_u32(), subnet_id);

        // Calculate weights
        Network::handle_subnet_emission_weights(Network::get_current_epoch_as_u32());

        // Verify weights exist
        let subnet_emission_weights =
            FinalSubnetEmissionWeights::<Test>::get(Network::get_current_epoch_as_u32());
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        // Emissions
        Network::emission_step(
            &mut WeightMeter::new(),
            System::block_number(),
            Network::get_current_epoch_as_u32(),
            Network::get_current_subnet_epoch_as_u32(subnet_id),
            subnet_id,
        );

        let queue = SubnetNodeQueue::<Test>::get(subnet_id);

        assert!(queue.len() > 0);

        // Ensure last is now the first
        let first = queue.first().unwrap();
        assert_eq!(first.id, last.id)
    });
}

#[test]
fn test_distribute_rewards_prioritized_queue_node_id_v2() {
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
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let new_start = end + 1;
        let new_end = new_start + 4;
        build_registered_nodes_in_queue(subnet_id, new_start, new_end, deposit_amount, amount);

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

        run_subnet_consensus_step(subnet_id, Some(last.id), None);

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

        increase_epochs(1);
        set_block_to_subnet_slot_epoch(Network::get_current_epoch_as_u32(), subnet_id);

        // Calculate weights
        Network::handle_subnet_emission_weights(Network::get_current_epoch_as_u32());

        // Verify weights exist
        let subnet_emission_weights =
            FinalSubnetEmissionWeights::<Test>::get(Network::get_current_epoch_as_u32());
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        // Emissions
        Network::emission_step(
            &mut WeightMeter::new(),
            System::block_number(),
            Network::get_current_epoch_as_u32(),
            Network::get_current_subnet_epoch_as_u32(subnet_id),
            subnet_id,
        );

        let queue = SubnetNodeQueue::<Test>::get(subnet_id);

        assert!(queue.len() > 0);

        // Ensure last is now the first
        let first = queue.first().unwrap();
        assert_eq!(first.id, last.id)
    });
}

#[test]
fn test_distribute_rewards_remove_queue_node_id() {
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
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let new_start = end + 1;
        let new_end = new_start + 4;
        build_registered_nodes_in_queue(subnet_id, new_start, new_end, deposit_amount, amount);

        // Set queue immunity epochs less than the registration queue epochs
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

        run_subnet_consensus_step(subnet_id, None, Some(first.id));

        let submission = SubnetConsensusSubmission::<Test>::get(
            subnet_id,
            Network::get_current_subnet_epoch_as_u32(subnet_id),
        );
        assert!(submission.clone().unwrap().remove_queue_node_id.is_some());
        assert_eq!(
            submission.clone().unwrap().remove_queue_node_id.unwrap(),
            first.id
        );

        increase_epochs(1);
        set_block_to_subnet_slot_epoch(Network::get_current_epoch_as_u32(), subnet_id);

        // Calculate weights
        Network::handle_subnet_emission_weights(Network::get_current_epoch_as_u32());

        // Verify weights exist
        let subnet_emission_weights =
            FinalSubnetEmissionWeights::<Test>::get(Network::get_current_epoch_as_u32());
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        // Emissions
        Network::emission_step(
            &mut WeightMeter::new(),
            System::block_number(),
            Network::get_current_epoch_as_u32(),
            Network::get_current_subnet_epoch_as_u32(subnet_id),
            subnet_id,
        );
        let queue = SubnetNodeQueue::<Test>::get(subnet_id);
        assert!(queue.len() > 0);

        let mut included = false;
        for n in queue {
            if n.id == first.id {
                included = true;
            }
        }

        assert!(!included);
    });
}

#[test]
fn test_distribute_rewards_remove_queue_node_id_v2() {
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
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let new_start = end + 1;
        let new_end = new_start + 4;
        build_registered_nodes_in_queue(subnet_id, new_start, new_end, deposit_amount, amount);

        // Set queue immunity epochs less than the registration queue epochs
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

        run_subnet_consensus_step(subnet_id, None, Some(first.id));

        let submission = SubnetConsensusSubmission::<Test>::get(
            subnet_id,
            Network::get_current_subnet_epoch_as_u32(subnet_id),
        );
        assert!(submission.clone().unwrap().remove_queue_node_id.is_some());
        assert_eq!(
            submission.clone().unwrap().remove_queue_node_id.unwrap(),
            first.id
        );

        increase_epochs(1);
        set_block_to_subnet_slot_epoch(Network::get_current_epoch_as_u32(), subnet_id);

        // Calculate weights
        Network::handle_subnet_emission_weights(Network::get_current_epoch_as_u32());

        // Verify weights exist
        let subnet_emission_weights =
            FinalSubnetEmissionWeights::<Test>::get(Network::get_current_epoch_as_u32());
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        // Emissions
        Network::emission_step(
            &mut WeightMeter::new(),
            System::block_number(),
            Network::get_current_epoch_as_u32(),
            Network::get_current_subnet_epoch_as_u32(subnet_id),
            subnet_id,
        );
        let queue = SubnetNodeQueue::<Test>::get(subnet_id);
        assert!(queue.len() > 0);

        let mut included = false;
        for n in queue {
            if n.id == first.id {
                included = true;
            }
        }

        assert!(!included);
    });
}

#[test]
fn test_distribute_rewards_non_consensus_reputation() {
    new_test_ext().execute_with(|| {
        // Tests:
        // - NonConsensusAttestorDecreaseReputationFactor
        // - ValidatorNonConsensusSubnetNodeReputationFactor
        // - NotInConsensusSubnetReputationFactor

        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let non_consensus_count: u32 = (max_subnet_nodes as f64 * 0.66) as u32 - 1;
        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..non_consensus_count {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let mut reputation_snapshot: BTreeMap<u32, u128> = BTreeMap::new();
        for n in 0..non_consensus_count {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let rep = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);

            assert_ne!(rep, 0);
            reputation_snapshot.insert(hotkey_subnet_node_id, rep);
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert_ne!(validator_stake, 0);

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        let starting_subnet_rep = SubnetReputation::<Test>::get(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let post_validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert!(validator_stake > post_validator_stake);

        assert!(starting_subnet_rep > SubnetReputation::<Test>::get(subnet_id));

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            if hotkey.clone() == validator.clone() {
                continue;
            }

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert_eq!(stake, *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

        for n in 0..non_consensus_count {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let rep = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);

            if let Some(old_rep) = reputation_snapshot.get(&hotkey_subnet_node_id) {
                assert!(rep < *old_rep);
                assert_ne!(rep, 0);
            } else {
                assert!(false); // auto-fail
            }
        }
    });
}

#[test]
fn test_distribute_rewards_absent_consensus_reputation() {
    new_test_ext().execute_with(|| {
        // AbsentDecreaseReputationFactor

        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let non_consensus_count: u32 = (max_subnet_nodes as f64 * 0.66) as u32 - 1;
        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes - 1);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..max_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnet_nodes);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
        let prev_reputation = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert_ne!(validator_stake, 0);

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let new_rep = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert!(new_rep < prev_reputation);
    });
}

#[test]
fn test_distribute_rewards_absent_consensus_then_in_consensus_reputation() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let non_consensus_count: u32 = (max_subnet_nodes as f64 * 0.66) as u32 - 1;
        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes - 1);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..max_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnet_nodes);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
        let prev_reputation = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert_ne!(validator_stake, 0);

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let new_rep = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert!(new_rep < prev_reputation);

        //
        // NEXT CONSENSUS
        //

        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..max_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnet_nodes);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
        let prev_reputation = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert_ne!(validator_stake, 0);

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let new_rep = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert!(new_rep > prev_reputation);
    });
}

#[test]
fn test_distribute_rewards_below_min_weight_reputation() {
    new_test_ext().execute_with(|| {
        // BelowMinWeightDecreaseReputationFactor

        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let non_consensus_count: u32 = (max_subnet_nodes as f64 * 0.66) as u32 - 1;
        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        SubnetNodeMinWeightDecreaseReputationThreshold::<Test>::insert(
            subnet_id,
            Network::percentage_factor_as_u128(),
        );
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes - 1);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..max_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        let mut reputation_snapshot: BTreeMap<u32, u128> = BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let rep = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);

            assert_ne!(rep, 0);
            reputation_snapshot.insert(hotkey_subnet_node_id, rep);
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnet_nodes);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
        let prev_reputation = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert_ne!(validator_stake, 0);

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let rep = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);

            if let Some(old_rep) = reputation_snapshot.get(&hotkey_subnet_node_id) {
                assert!(rep < *old_rep);
                assert_ne!(rep, 0);
            } else {
                assert!(false); // auto-fail
            }
        }
    });
}

#[test]
fn test_distribute_rewards_non_attest_vast_majoriy_reputation() {
    new_test_ext().execute_with(|| {
        // NonAttestorDecreaseReputationFactor

        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let majority = (Network::get_percent_as_f64(SuperMajorityAttestationRatio::<Test>::get())
            * max_subnet_nodes as f64) as u32
            + 1;
        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..majority {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let mut reputation_snapshot: BTreeMap<u32, u128> = BTreeMap::new();
        for n in majority..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let rep = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);

            assert_ne!(rep, 0);
            reputation_snapshot.insert(hotkey_subnet_node_id, rep);
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert_ne!(validator_stake, 0);

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let post_validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert!(validator_stake < post_validator_stake);

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            if hotkey.clone() == validator.clone() {
                continue;
            }

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert!(stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

        for n in majority..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let rep = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);

            if let Some(old_rep) = reputation_snapshot.get(&hotkey_subnet_node_id) {
                assert!(rep < *old_rep);
                assert_ne!(rep, 0);
            } else {
                assert!(false); // auto-fail
            }
        }
    });
}

#[test]
fn test_distribute_rewards_under_min_attest_slash_validator() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert_ne!(validator_stake, 0);

        let starting_rep = SubnetNodeReputation::<Test>::get(subnet_id, validator_id.unwrap());

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let post_validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert!(validator_stake > post_validator_stake);

        assert!(starting_rep > SubnetNodeReputation::<Test>::get(subnet_id, validator_id.unwrap()));

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            if hotkey.clone() == validator.clone() {
                continue;
            }

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert_eq!(stake, *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }
    });
}

#[test]
fn test_distribute_rewards_fork_under_min_attest_slash_validator() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert_ne!(validator_stake, 0);

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        let starting_rep = SubnetNodeReputation::<Test>::get(subnet_id, validator_id.unwrap());

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let post_validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        assert!(validator_stake > post_validator_stake);

        assert!(starting_rep > SubnetNodeReputation::<Test>::get(subnet_id, validator_id.unwrap()));

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            if hotkey.clone() == validator.clone() {
                continue;
            }

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert_eq!(stake, *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }
    });
}

#[test]
fn test_distribute_rewards_fork_remove_node_at_min_reputation() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        let mut removing_subnet_node_id: Option<u32> = None;
        let min_rep = MinSubnetNodeReputation::<Test>::get(subnet_id);

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            // mock reputation on the first non-validator to have them removed
            if removing_subnet_node_id.is_none() {
                removing_subnet_node_id =
                    HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone());
                SubnetNodeReputation::<Test>::insert(
                    subnet_id,
                    removing_subnet_node_id.unwrap(),
                    min_rep - 1,
                );
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        assert!(removing_subnet_node_id.is_some());

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, _) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        assert_eq!(
            SubnetNodesData::<Test>::try_get(subnet_id, removing_subnet_node_id.unwrap()),
            Err(())
        );
    });
}

#[test]
fn test_distribute_rewards_fork_no_score_submitted_decrease_reputation() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = max_subnet_nodes - 1;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        let idle_epochs = IdleClassificationEpochs::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Get scores leaving out the last node
        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, end - 1);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..end {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, _) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..end {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let end_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, end_hotkey.clone()).unwrap();

        let starting_rep = SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(starting_rep, Network::percentage_factor_as_u128());

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        for n in 0..end - 1 {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert!(stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

        assert!(starting_rep > SubnetNodeReputation::<Test>::get(subnet_id, hotkey_subnet_node_id));
    });
}

#[test]
fn test_distribute_rewards_late_validator_and_attestors() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        System::set_block_number(System::block_number() + epoch_length / 2);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        System::set_block_number(System::block_number() + epoch_length / 2 / 2);

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();
        assert_eq!(
            consensus_submission_data.clone().validator_reward_factor,
            500000000000000000
        );

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let subnet_rewards = rewards_data.subnet_rewards;

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let block_number = System::block_number();
        let dstake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        let set_rep = 500000000000000000;
        SubnetReputation::<Test>::insert(subnet_id, set_rep);

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data.clone(),
            rewards_data.clone(),
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let total_weight = DEFAULT_SCORE * total_subnet_nodes as u128;
        let node_weight = Network::percent_div(DEFAULT_SCORE, total_weight as u128);
        let expected_node_reward =
            Network::percent_mul(node_weight, rewards_data.clone().subnet_node_rewards);

        let post_validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        let expected_validator_reward = Network::percent_mul(
            BaseValidatorReward::<Test>::get(),
            consensus_submission_data.clone().validator_reward_factor,
        );
        assert_eq!(
            validator_stake + expected_validator_reward + expected_node_reward,
            post_validator_stake
        );

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            if hotkey.clone() == validator.clone() {
                continue;
            }

            let subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let reward_factor = match consensus_submission_data.attests.get(&subnet_node_id) {
                Some(data) => data.reward_factor,
                None => return assert!(false),
            };

            assert_ne!(reward_factor, Network::percentage_factor_as_u128());
            assert_ne!(reward_factor, 0);

            let expected_node_reward = Network::percent_mul(expected_node_reward, reward_factor);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert!(stake > *old_stake);
                assert_eq!(stake, *old_stake + expected_node_reward);
            } else {
                assert!(false); // auto-fail
            }
        }

        let post_dstake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        assert!(post_dstake_balance > dstake_balance);

        assert!(SubnetReputation::<Test>::get(subnet_id) > set_rep);
    });
}

#[test]
fn test_distribute_rewards_fork_late_validator_and_attestors() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        System::set_block_number(System::block_number() + epoch_length / 2);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        System::set_block_number(System::block_number() + epoch_length / 2 / 2);

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();
        assert_eq!(
            consensus_submission_data.clone().validator_reward_factor,
            500000000000000000
        );

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let subnet_rewards = rewards_data.subnet_rewards;

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let block_number = System::block_number();
        let dstake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        SubnetReputation::<Test>::insert(subnet_id, 500000000000000000);
        let starting_rep = SubnetReputation::<Test>::get(subnet_id);

        let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data.clone(),
            rewards_data.clone(),
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        let total_weight = DEFAULT_SCORE * total_subnet_nodes as u128;
        let node_weight = Network::percent_div(DEFAULT_SCORE, total_weight as u128);
        let expected_node_reward =
            Network::percent_mul(node_weight, rewards_data.clone().subnet_node_rewards);

        let post_validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
        let expected_validator_reward = Network::percent_mul(
            BaseValidatorReward::<Test>::get(),
            consensus_submission_data.clone().validator_reward_factor,
        );
        assert_eq!(
            validator_stake + expected_validator_reward + expected_node_reward,
            post_validator_stake
        );

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            if hotkey.clone() == validator.clone() {
                continue;
            }

            let subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let reward_factor = match consensus_submission_data.attests.get(&subnet_node_id) {
                Some(data) => data.reward_factor,
                None => return assert!(false),
            };

            assert_ne!(reward_factor, Network::percentage_factor_as_u128());
            assert_ne!(reward_factor, 0);

            let expected_node_reward = Network::percent_mul(expected_node_reward, reward_factor);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert!(stake > *old_stake);
                assert_eq!(stake, *old_stake + expected_node_reward);
            } else {
                assert!(false); // auto-fail
            }
        }

        let post_dstake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        assert!(post_dstake_balance > dstake_balance);

        assert!(SubnetReputation::<Test>::get(subnet_id) > starting_rep);
    });
}

// #[test]
// fn test_distribute_rewards_graduate_idle_to_included() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let end = max_subnet_nodes - 1;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let idle_epochs = IdleClassificationEpochs::<Test>::get(subnet_id);

//         let epoch_length = EpochLength::get();
//         let block_number = System::block_number();
//         let epoch = block_number / epoch_length;

//         // ⸺ Register and activate node into Idle classification
//         let idle_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let idle_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let idle_peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let idle_bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let idle_client_peer_id =
//             get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let burn_amount = Network::calculate_burn_amount(subnet_id);
//         let _ = Balances::deposit_creating(&idle_coldkey.clone(), deposit_amount + burn_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(idle_coldkey.clone()),
//             subnet_id,
//             idle_hotkey.clone(),
//             idle_peer_id.clone(),
//             idle_bootnode_peer_id.clone(),
//             idle_client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             None,
//             None,
//             u128::MAX
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, idle_hotkey.clone()).unwrap();
//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let start_epoch = subnet_node.classification.start_epoch;

//         let queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id);

//         let epoch = Network::get_current_epoch_as_u32();
//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         // increase to the nodes start epoch
//         set_block_to_subnet_slot_epoch(subnet_epoch + queue_epochs + 2, subnet_id);

//         let epoch = Network::get_current_epoch_as_u32();
//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         // Get subnet weights (nodes only activate from queue if there are weights)
//         // Note: This means a subnet is active if it gets weights
//         let _ = Network::handle_subnet_emission_weights(epoch);

//         // Trigger the node activation
//         Network::emission_step(
//             &mut WeightMeter::new(),
//             System::block_number(),
//             Network::get_current_epoch_as_u32(),
//             Network::get_current_subnet_epoch_as_u32(subnet_id),
//             subnet_id,
//         );

//         assert_eq!(
//             RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
//             Err(())
//         );

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
//         assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

//         // increase epochs up to when node should be able to graduate
//         increase_epochs(idle_epochs + 1);
//         let epoch = Network::get_current_epoch_as_u32();

//         // ⸺ Submit consnesus data
//         set_block_to_subnet_slot_epoch(epoch, subnet_id);
//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         Network::elect_validator(subnet_id, subnet_epoch, block_number);

//         let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
//         assert!(validator_id != None, "Validator is None");
//         assert!(validator_id != Some(0), "Validator is 0");

//         let mut validator =
//             SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
//         let epoch = Network::get_current_epoch_as_u32();

//         let subnet_node_data_vec =
//             get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, end);

//         assert_ok!(Network::propose_attestation(
//             RuntimeOrigin::signed(validator.clone()),
//             subnet_id,
//             subnet_node_data_vec.clone(),
//             None,
//             None,
//             None,
//             None,
//         ));

//         for n in 0..end {
//             let _n = n + 1;
//             let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
//             if hotkey.clone() == validator.clone() {
//                 continue;
//             }
//             assert_ok!(Network::attest(
//                 RuntimeOrigin::signed(hotkey.clone()),
//                 subnet_id,
//                 None,
//             ));
//         }

//         increase_epochs(1);
//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
//         let epoch = Network::get_current_epoch_as_u32();

//         // ⸺ Generate subnet weights from stake/node count weights
//         let _ = Network::handle_subnet_emission_weights(epoch);
//         let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
//         let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
//         assert!(subnet_weight.is_some());

//         let (result, block_weight) = Network::precheck_subnet_consensus_submission(
//             subnet_id,
//             subnet_epoch - 1,
//             Network::get_current_epoch_as_u32(),
//         );

//         assert!(result.is_some(), "Precheck consensus failed");

//         let consensus_submission_data = result.unwrap();

//         // ⸺ Calculate subnet distribution of rewards
//         let (rewards_data, _) = Network::calculate_rewards(
//             subnet_id,
//             subnet_emission_weights.validator_emissions,
//             *subnet_weight.unwrap(),
//         );

//         let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
//             BTreeMap::new();
//         for n in 0..end {
//             let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

//             let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

//             assert_ne!(stake, 0);
//             stake_snapshot.insert(hotkey.clone(), stake);
//         }

//         let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
//         let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
//         let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
//         let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

//         let epoch = Network::get_current_epoch_as_u32();
//         set_block_to_subnet_slot_epoch(epoch, subnet_id);
//         let block_number = System::block_number();

//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         Network::distribute_rewards_fork(
//             &mut WeightMeter::new(),
//             subnet_id,
//             block_number,
//             epoch,
//             subnet_epoch,
//             consensus_submission_data,
//             rewards_data,
//             min_attestation_percentage,
//             coldkey_reputation_increase_factor,
//             coldkey_reputation_decrease_factor,
//             super_majority_threshold,
//         );

//         for n in 0..end {
//             let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

//             let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

//             if let Some(old_stake) = stake_snapshot.get(&hotkey) {
//                 assert!(stake > *old_stake);
//             } else {
//                 assert!(false); // auto-fail
//             }
//         }

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         assert_eq!(
//             subnet_node.classification.node_class,
//             SubnetNodeClass::Included
//         );
//     });
// }

#[test]
fn test_distribute_rewards_fork_graduate_idle_to_included() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = max_subnet_nodes - 1;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        let idle_epochs = IdleClassificationEpochs::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Register and activate node into Idle classification
        let idle_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let idle_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let idle_peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let idle_bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let idle_client_peer_id =
            get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let burn_amount = Network::calculate_burn_amount(subnet_id);
        let _ = Balances::deposit_creating(&idle_coldkey.clone(), deposit_amount + burn_amount);

        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(idle_coldkey.clone()),
            subnet_id,
            idle_hotkey.clone(),
            idle_peer_id.clone(),
            idle_bootnode_peer_id.clone(),
            idle_client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, idle_hotkey.clone()).unwrap();
        let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        let start_epoch = subnet_node.classification.start_epoch;

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

        let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
        assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

        SubnetNodeIdleConsecutiveEpochs::<Test>::insert(subnet_id, subnet_node.id, idle_epochs);

        // increase epochs up to when node should be able to graduate
        increase_epochs(idle_epochs + 1);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, end);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..end {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, _) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..end {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        for n in 0..end {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert!(stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

        let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(
            subnet_node.classification.node_class,
            SubnetNodeClass::Included
        );
    });
}

#[test]
fn test_distribute_rewards_graduate_included_to_validator() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = max_subnet_nodes - 1;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        let idle_epochs = IdleClassificationEpochs::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Register and activate node into Idle classification
        let idle_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let idle_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let idle_peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let idle_bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let idle_client_peer_id =
            get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let burn_amount = Network::calculate_burn_amount(subnet_id);
        let _ = Balances::deposit_creating(&idle_coldkey.clone(), deposit_amount + burn_amount);

        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(idle_coldkey.clone()),
            subnet_id,
            idle_hotkey.clone(),
            idle_peer_id.clone(),
            idle_bootnode_peer_id.clone(),
            idle_client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, idle_hotkey.clone()).unwrap();
        let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        let start_epoch = subnet_node.classification.start_epoch;

        //
        // Force node to Included classification
        //
        let mut subnet_node =
            RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        Network::do_activate_subnet_node(
            &mut WeightMeter::new(),
            subnet_id,
            subnet_node,
            Network::get_current_subnet_epoch_as_u32(subnet_id),
        );
        Network::graduate_class(
            subnet_id,
            hotkey_subnet_node_id,
            Network::get_current_subnet_epoch_as_u32(subnet_id),
        );

        assert_eq!(
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );

        let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(
            subnet_node.classification.node_class,
            SubnetNodeClass::Included
        );

        set_block_to_subnet_slot_epoch(Network::get_current_epoch_as_u32(), subnet_id);

        // NEW EPOCH
        let included_epochs = IncludedClassificationEpochs::<Test>::get(subnet_id);

        let mut staked_checked = false;

        let starting_epoch = Network::get_current_epoch_as_u32();
        for e in 0..included_epochs.saturating_add(1) {
            let epoch = Network::get_current_epoch_as_u32();
            let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
            set_block_to_subnet_slot_epoch(Network::get_current_epoch_as_u32(), subnet_id);
            Network::elect_validator(subnet_id, subnet_epoch, System::block_number());

            // Start of epoch, check stake balances
            let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
                BTreeMap::new();
            if starting_epoch != epoch {
                for n in 0..end {
                    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

                    if let Some(subnet_node_id) =
                        HotkeySubnetNodeId::<Test>::get(subnet_id, &hotkey)
                    {
                        let is_validator =
                            match SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id) {
                                Ok(subnet_node) => subnet_node.has_classification(
                                    &SubnetNodeClass::Validator,
                                    Network::get_current_subnet_epoch_as_u32(subnet_id),
                                ),
                                Err(()) => false,
                            };

                        if !is_validator {
                            continue;
                        }
                    }
                    let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

                    assert_ne!(stake, 0);
                    stake_snapshot.insert(hotkey.clone(), stake);
                }
            }

            // Calculate weights
            Network::handle_subnet_emission_weights(Network::get_current_epoch_as_u32());

            // Verify weights exist
            let subnet_emission_weights =
                FinalSubnetEmissionWeights::<Test>::get(Network::get_current_epoch_as_u32());
            let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
            assert!(subnet_weight.is_some());

            // Propose attestation and attest
            run_subnet_consensus_step(subnet_id, None, None);

            // Emissions
            Network::emission_step(
                &mut WeightMeter::new(),
                System::block_number(),
                Network::get_current_epoch_as_u32(),
                Network::get_current_subnet_epoch_as_u32(subnet_id),
                subnet_id,
            );

            // End of epoch, verify stake balances increased
            if starting_epoch != epoch {
                for n in 0..end {
                    staked_checked = true;
                    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
                    if let Some(subnet_node_id) =
                        HotkeySubnetNodeId::<Test>::get(subnet_id, &hotkey)
                    {
                        let is_validator =
                            match SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id) {
                                Ok(subnet_node) => subnet_node.has_classification(
                                    &SubnetNodeClass::Validator,
                                    Network::get_current_subnet_epoch_as_u32(subnet_id),
                                ),
                                Err(()) => false,
                            };

                        if !is_validator {
                            continue;
                        }
                    }

                    let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

                    if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                        assert!(stake > *old_stake);
                    } else {
                        assert!(false); // auto-fail
                    }
                }
            }
            increase_epochs(1);
        }

        assert!(staked_checked);

        let node_included_epochs =
            SubnetNodeConsecutiveIncludedEpochs::<Test>::get(subnet_id, hotkey_subnet_node_id);
        let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(
            subnet_node.classification.node_class,
            SubnetNodeClass::Validator
        );
    });
}

#[test]
fn test_distribute_rewards_graduate_included_to_validator_v2() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = max_subnet_nodes - 1;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        let idle_epochs = IdleClassificationEpochs::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Register and activate node into Idle classification
        let idle_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let idle_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let idle_peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let idle_bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let idle_client_peer_id =
            get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let burn_amount = Network::calculate_burn_amount(subnet_id);
        let _ = Balances::deposit_creating(&idle_coldkey.clone(), deposit_amount + burn_amount);

        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(idle_coldkey.clone()),
            subnet_id,
            idle_hotkey.clone(),
            idle_peer_id.clone(),
            idle_bootnode_peer_id.clone(),
            idle_client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, idle_hotkey.clone()).unwrap();
        let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        let start_epoch = subnet_node.classification.start_epoch;

        //
        // Force node to Included classification
        //
        let mut subnet_node =
            RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        Network::do_activate_subnet_node(
            &mut WeightMeter::new(),
            subnet_id,
            subnet_node,
            Network::get_current_subnet_epoch_as_u32(subnet_id),
        );
        Network::graduate_class(
            subnet_id,
            hotkey_subnet_node_id,
            Network::get_current_subnet_epoch_as_u32(subnet_id),
        );

        assert_eq!(
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );

        let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(
            subnet_node.classification.node_class,
            SubnetNodeClass::Included
        );

        set_block_to_subnet_slot_epoch(Network::get_current_epoch_as_u32(), subnet_id);

        // NEW EPOCH
        let included_epochs = IncludedClassificationEpochs::<Test>::get(subnet_id);

        let mut staked_checked = false;

        let starting_epoch = Network::get_current_epoch_as_u32();
        for e in 0..included_epochs.saturating_add(1) {
            let epoch = Network::get_current_epoch_as_u32();
            let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
            set_block_to_subnet_slot_epoch(Network::get_current_epoch_as_u32(), subnet_id);
            Network::elect_validator(subnet_id, subnet_epoch, System::block_number());

            // Start of epoch, check stake balances
            let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
                BTreeMap::new();
            if starting_epoch != epoch {
                for n in 0..end {
                    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

                    if let Some(subnet_node_id) =
                        HotkeySubnetNodeId::<Test>::get(subnet_id, &hotkey)
                    {
                        let is_validator =
                            match SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id) {
                                Ok(subnet_node) => subnet_node.has_classification(
                                    &SubnetNodeClass::Validator,
                                    Network::get_current_subnet_epoch_as_u32(subnet_id),
                                ),
                                Err(()) => false,
                            };

                        if !is_validator {
                            continue;
                        }
                    }
                    let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

                    assert_ne!(stake, 0);
                    stake_snapshot.insert(hotkey.clone(), stake);
                }
            }

            // Calculate weights
            Network::handle_subnet_emission_weights(Network::get_current_epoch_as_u32());

            // Verify weights exist
            let subnet_emission_weights =
                FinalSubnetEmissionWeights::<Test>::get(Network::get_current_epoch_as_u32());
            let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
            assert!(subnet_weight.is_some());

            // Propose attestation and attest
            run_subnet_consensus_step(subnet_id, None, None);

            // Emissions
            Network::emission_step(
                &mut WeightMeter::new(),
                System::block_number(),
                Network::get_current_epoch_as_u32(),
                Network::get_current_subnet_epoch_as_u32(subnet_id),
                subnet_id,
            );

            // End of epoch, verify stake balances increased
            if starting_epoch != epoch {
                for n in 0..end {
                    staked_checked = true;
                    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
                    if let Some(subnet_node_id) =
                        HotkeySubnetNodeId::<Test>::get(subnet_id, &hotkey)
                    {
                        let is_validator =
                            match SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id) {
                                Ok(subnet_node) => subnet_node.has_classification(
                                    &SubnetNodeClass::Validator,
                                    Network::get_current_subnet_epoch_as_u32(subnet_id),
                                ),
                                Err(()) => false,
                            };

                        if !is_validator {
                            continue;
                        }
                    }

                    let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

                    if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                        assert!(stake > *old_stake);
                    } else {
                        assert!(false); // auto-fail
                    }
                }
            }
            increase_epochs(1);
        }

        assert!(staked_checked);

        let node_included_epochs =
            SubnetNodeConsecutiveIncludedEpochs::<Test>::get(subnet_id, hotkey_subnet_node_id);
        let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(
            subnet_node.classification.node_class,
            SubnetNodeClass::Validator
        );
    });
}

#[test]
fn test_attest_increase_reputation_when_included() {
    new_test_ext().execute_with(|| {
        // IncludedIncreaseReputationFactor

        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, (u128, u128)> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);

            let subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
            SubnetNodeReputation::<Test>::insert(subnet_id, subnet_node_id, 500000000000000000);
            let rep = SubnetNodeReputation::<Test>::get(subnet_id, subnet_node_id);

            stake_snapshot.insert(hotkey.clone(), (stake, rep));
        }

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);
            let subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some((old_stake, old_rep)) = stake_snapshot.get(&hotkey) {
                assert!(stake > *old_stake);
                assert!(SubnetNodeReputation::<Test>::get(subnet_id, subnet_node_id) > *old_rep);
            } else {
                assert!(false); // auto-fail
            }
        }
    });
}

#[test]
fn test_distribute_rewards_node_delegate_stake() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let node_coldkey = get_coldkey(subnets, max_subnet_nodes, max_subnets);
        let node_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnets);
        let subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, node_hotkey.clone()).unwrap();

        // Update node to have delegate stake rate
        SubnetNodesData::<Test>::mutate(subnet_id, subnet_node_id, |params| {
            params.delegate_reward_rate = 100000000000000000;
        });

        // increase shares manually
        // *Distribution requires shares to distribute to stakers*
        TotalNodeDelegateStakeShares::<Test>::insert(subnet_id, subnet_node_id, 1);

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        // run_subnet_consensus_step(subnet_id, None, None);
        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnet_id, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert!(stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

        let post_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        assert!(post_delegate_stake_balance > delegate_stake_balance);
    });
}

#[test]
fn test_distribute_rewards_fork_node_delegate_stake() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let node_coldkey = get_coldkey(subnets, max_subnet_nodes, max_subnets);
        let node_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnets);
        let subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, node_hotkey.clone()).unwrap();

        // Update node to have delegate stake rate
        SubnetNodesData::<Test>::mutate(subnet_id, subnet_node_id, |params| {
            params.delegate_reward_rate = 100000000000000000;
        });

        // increase shares manually
        // *Distribution requires shares to distribute to stakers*
        TotalNodeDelegateStakeShares::<Test>::insert(subnet_id, subnet_node_id, 1);

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Submit consnesus data
        // run_subnet_consensus_step(subnet_id, None, None);
        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnet_id, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        for n in 0..total_subnet_nodes {
            let _n = n + 1;
            let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
            if hotkey.clone() == validator.clone() {
                continue;
            }
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }

        increase_epochs(1);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch = Network::get_current_epoch_as_u32();

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        let (result, block_weight) = Network::precheck_subnet_consensus_submission(
            subnet_id,
            subnet_epoch - 1,
            Network::get_current_epoch_as_u32(),
        );

        assert!(result.is_some(), "Precheck consensus failed");

        let consensus_submission_data = result.unwrap();

        // ⸺ Calculate subnet distribution of rewards
        let (rewards_data, rewards_weight) = Network::calculate_rewards(
            subnet_id,
            subnet_emission_weights.validator_emissions,
            *subnet_weight.unwrap(),
        );

        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> =
            BTreeMap::new();
        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
        }

        let delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);

        let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
        let coldkey_reputation_increase_factor = ColdkeyReputationIncreaseFactor::<Test>::get();
        let coldkey_reputation_decrease_factor = ColdkeyReputationDecreaseFactor::<Test>::get();
        let super_majority_threshold = SuperMajorityAttestationRatio::<Test>::get();

        let epoch = Network::get_current_epoch_as_u32();
        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let block_number = System::block_number();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::distribute_rewards_fork(
            &mut WeightMeter::new(),
            subnet_id,
            block_number,
            epoch,
            subnet_epoch,
            consensus_submission_data,
            rewards_data,
            min_attestation_percentage,
            coldkey_reputation_increase_factor,
            coldkey_reputation_decrease_factor,
            super_majority_threshold,
        );

        for n in 0..max_subnet_nodes {
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n + 1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
                assert!(stake > *old_stake);
            } else {
                assert!(false); // auto-fail
            }
        }

        let post_delegate_stake_balance =
            NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
        assert!(post_delegate_stake_balance > delegate_stake_balance);
    });
}

#[test]
fn test_do_epoch_preliminaries_deactivate_min_reputation() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let min_rep = MinSubnetReputation::<Test>::get();
        SubnetReputation::<Test>::insert(subnet_id, min_rep - 1);

        increase_epochs(1);
        let block_number = System::block_number();

        let epoch_length = EpochLength::get();
        let epoch = System::block_number() / epoch_length;

        Network::do_epoch_preliminaries(&mut WeightMeter::new(), block_number, epoch);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SubnetDeactivated {
                subnet_id: subnet_id,
                reason: SubnetRemovalReason::MinReputation
            }
        );
    });
}

#[test]
fn test_do_epoch_preliminaries_deactivate_min_subnet_delegate_stake() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        // --- Remove delegate stake to force MinSubnetDelegateStake removal reason
        let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(1), subnet_id);
        assert_ok!(Network::remove_delegate_stake(
            RuntimeOrigin::signed(account(1)),
            subnet_id,
            delegate_shares,
        ));

        increase_epochs(1);
        let block_number = System::block_number();

        let epoch_length = EpochLength::get();
        let epoch = System::block_number() / epoch_length;

        Network::do_epoch_preliminaries(&mut WeightMeter::new(), block_number, epoch);
        assert_eq!(
            *network_events().last().unwrap(),
            Event::SubnetDeactivated {
                subnet_id: subnet_id,
                reason: SubnetRemovalReason::MinSubnetDelegateStake
            }
        );
    });
}

#[test]
fn test_propose_attestation_epoch_progression_0() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let node_coldkey = get_coldkey(subnets, max_subnet_nodes, max_subnets);
        let node_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnets);
        let subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, node_hotkey.clone()).unwrap();

        // Update node to have delegate stake rate
        SubnetNodesData::<Test>::mutate(subnet_id, subnet_node_id, |params| {
            params.delegate_reward_rate = 100000000000000000;
        });

        // increase shares manually
        // *Distribution requires shares to distribute to stakers*
        TotalNodeDelegateStakeShares::<Test>::insert(subnet_id, subnet_node_id, 1);

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        // ⸺ Submit consnesus data
        // run_subnet_consensus_step(subnet_id, None, None);
        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnet_id, max_subnet_nodes, 0, total_subnet_nodes);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch).unwrap();

        assert_eq!(submission.validator_epoch_progress, 0);
    });
}

#[test]
fn test_propose_attestation_epoch_progression_50() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let node_coldkey = get_coldkey(subnets, max_subnet_nodes, max_subnets);
        let node_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnets);
        let subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, node_hotkey.clone()).unwrap();

        // Update node to have delegate stake rate
        SubnetNodesData::<Test>::mutate(subnet_id, subnet_node_id, |params| {
            params.delegate_reward_rate = 100000000000000000;
        });

        // increase shares manually
        // *Distribution requires shares to distribute to stakers*
        TotalNodeDelegateStakeShares::<Test>::insert(subnet_id, subnet_node_id, 1);

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnet_id, max_subnet_nodes, 0, total_subnet_nodes);

        System::set_block_number(System::block_number() + epoch_length / 2);

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch).unwrap();

        assert_eq!(submission.validator_epoch_progress, 500000000000000000);
    });
}

#[test]
fn test_propose_attestation_epoch_progression_99() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let node_coldkey = get_coldkey(subnets, max_subnet_nodes, max_subnets);
        let node_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnets);
        let subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, node_hotkey.clone()).unwrap();

        // Update node to have delegate stake rate
        SubnetNodesData::<Test>::mutate(subnet_id, subnet_node_id, |params| {
            params.delegate_reward_rate = 100000000000000000;
        });

        // increase shares manually
        // *Distribution requires shares to distribute to stakers*
        TotalNodeDelegateStakeShares::<Test>::insert(subnet_id, subnet_node_id, 1);

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnet_id, max_subnet_nodes, 0, total_subnet_nodes);

        System::set_block_number(
            System::block_number()
                + (Network::percent_mul(epoch_length as u128, 990000000000000000) as u32),
        );

        assert_ok!(Network::propose_attestation(
            RuntimeOrigin::signed(validator.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch).unwrap();

        assert_eq!(submission.validator_epoch_progress, 990000000000000000);
    });
}

#[test]
fn test_propose_attestation_epoch_progression_100() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        build_activated_subnet_new(
            subnet_name.clone(),
            0,
            max_subnet_nodes,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let node_coldkey = get_coldkey(subnets, max_subnet_nodes, max_subnets);
        let node_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnets);
        let subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, node_hotkey.clone()).unwrap();

        // Update node to have delegate stake rate
        SubnetNodesData::<Test>::mutate(subnet_id, subnet_node_id, |params| {
            params.delegate_reward_rate = 100000000000000000;
        });

        // increase shares manually
        // *Distribution requires shares to distribute to stakers*
        TotalNodeDelegateStakeShares::<Test>::insert(subnet_id, subnet_node_id, 1);

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        // ⸺ Generate subnet weights from stake/node count weights
        let _ = Network::handle_subnet_emission_weights(epoch);
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
        let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
        assert!(subnet_weight.is_some());

        // ⸺ Submit consnesus data
        set_block_to_subnet_slot_epoch(epoch, subnet_id);

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
        assert!(validator_id != None, "Validator is None");
        assert!(validator_id != Some(0), "Validator is 0");

        let mut validator =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let subnet_node_data_vec =
            get_subnet_node_consensus_data(subnet_id, max_subnet_nodes, 0, total_subnet_nodes);

        System::set_block_number(System::block_number() + epoch_length);

        assert_err!(
            Network::propose_attestation(
                RuntimeOrigin::signed(validator.clone()),
                subnet_id,
                subnet_node_data_vec.clone(),
                None,
                None,
                None,
                None,
            ),
            Error::<Test>::NoElectedValidator
        );
    });
}

#[test]
fn test_get_validator_reward_multiplier() {
    new_test_ext().execute_with(|| {
        let factor = Network::get_validator_reward_multiplier(0);
        assert!(factor > 990000000000000000);
    });
}

#[test]
fn test_emergency_validator_subnet_rewards() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let max = 12;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let subnets = TotalActiveSubnets::<Test>::get() + 1;

        build_activated_subnet_new(subnet_name.clone(), 0, max, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let original_owner = account(1);

        let pause_cooldown_epochs = SubnetPauseCooldownEpochs::<Test>::get();
        increase_epochs(pause_cooldown_epochs + 1);

        // Set initial owner
        SubnetOwner::<Test>::insert(subnet_id, &original_owner);
        let epoch = Network::get_current_epoch_as_u32();

        // Transfer to new owner
        assert_ok!(Network::owner_pause_subnet(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
        ));

        assert_eq!(
            *network_events().last().unwrap(),
            Event::SubnetPaused {
                subnet_id: subnet_id,
                owner: original_owner.clone(),
            }
        );

        let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
        assert_eq!(subnet_data.state, SubnetState::Paused);
        assert_eq!(subnet_data.start_epoch, epoch);

        let mut original_subnet_node_ids: Vec<u32> = Vec::new();
        for (id, _) in SubnetNodesData::<Test>::iter_prefix(subnet_id) {
            original_subnet_node_ids.push(id);
        }

        let mut subnet_node_ids: Vec<u32> = Vec::new();
        for (id, _) in SubnetNodesData::<Test>::iter_prefix(subnet_id).take((max - 1) as usize) {
            subnet_node_ids.push(id);
        }

        let pre_emergency_validator_data = EmergencySubnetNodeElectionData::<Test>::get(subnet_id);
        assert!(pre_emergency_validator_data.is_none());

        assert_ok!(Network::owner_set_emergency_validator_set(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
            subnet_node_ids.clone()
        ));

        let emergency_validator_data = EmergencySubnetNodeElectionData::<Test>::get(subnet_id);
        assert!(emergency_validator_data.is_some());
        assert_eq!(
            emergency_validator_data.clone().unwrap().subnet_node_ids,
            subnet_node_ids
        );
        assert_ne!(
            emergency_validator_data.clone().unwrap().subnet_node_ids,
            original_subnet_node_ids
        );
        assert_ne!(
            emergency_validator_data
                .clone()
                .unwrap()
                .target_emergency_validators_epochs,
            0
        );

        assert_ok!(Network::owner_unpause_subnet(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
        ));

        let emergency_validator_data = EmergencySubnetNodeElectionData::<Test>::get(subnet_id);
        assert_ne!(
            emergency_validator_data
                .clone()
                .unwrap()
                .target_emergency_validators_epochs,
            0
        );
        assert_eq!(emergency_validator_data.clone().unwrap().total_epochs, 0);

        let epoch_length = EpochLength::get();

        for i in 0..emergency_validator_data
            .clone()
            .unwrap()
            .target_emergency_validators_epochs
            + 2
        {
            log::error!("i: {:?}", i);

            let block_number = System::block_number();
            let epoch = block_number / epoch_length;

            set_block_to_subnet_slot_epoch(epoch, subnet_id);
            let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

            Network::elect_validator(subnet_id, subnet_epoch, block_number);

            let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
            assert!(validator_id != None, "Validator is None");
            assert!(validator_id != Some(0), "Validator is 0");
            assert_ne!(validator_id.unwrap(), max);

            let mut validator =
                SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

            let subnet_node_data_vec =
                get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, max - 1);

            assert_ok!(Network::propose_attestation(
                RuntimeOrigin::signed(validator.clone()),
                subnet_id,
                subnet_node_data_vec.clone(),
                None,
                None,
                None,
                None,
            ));

            let submission =
                SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch).unwrap();
            assert_eq!(submission.validator_id, validator_id.unwrap());
            assert_eq!(submission.data.len(), subnet_node_data_vec.len());
            assert_eq!(submission.data.len(), subnet_node_data_vec.len());

            for n in 0..max - 1 {
                let _n = n + 1;
                let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
                if hotkey.clone() == validator.clone() {
                    continue;
                }
                assert_ok!(Network::attest(
                    RuntimeOrigin::signed(hotkey.clone()),
                    subnet_id,
                    None,
                ));
            }

            increase_epochs(1);
            let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
            let epoch = Network::get_current_epoch_as_u32();

            let _ = Network::handle_subnet_emission_weights(epoch);
            let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);
            let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
            assert!(subnet_weight.is_some());

            let (result, block_weight) = Network::precheck_subnet_consensus_submission(
                subnet_id,
                subnet_epoch - 1,
                Network::get_current_epoch_as_u32(),
            );

            assert!(result.is_some(), "Precheck consensus failed");

            let consensus_submission_data = result.unwrap();
            assert_eq!(
                consensus_submission_data.clone().validator_subnet_node_id,
                validator_id.unwrap()
            );
            assert_eq!(
                consensus_submission_data.clone().validator_epoch_progress,
                0
            );
            assert!(consensus_submission_data.clone().validator_reward_factor > 990000000000000000);
            assert_eq!(
                consensus_submission_data.clone().attestation_ratio,
                1000000000000000000
            );
            assert_eq!(
                consensus_submission_data.clone().weight_sum,
                500000000000000000 * ((max - 1) as u128)
            );
            assert_eq!(consensus_submission_data.clone().data_length, max - 1);
            assert_eq!(
                consensus_submission_data.clone().data,
                subnet_node_data_vec.clone()
            );
            assert_eq!(
                consensus_submission_data.clone().attests.len(),
                (max - 1) as usize
            );
            if i >= emergency_validator_data
                .clone()
                .unwrap()
                .target_emergency_validators_epochs
            {
                assert_eq!(
                    consensus_submission_data.clone().subnet_nodes.len(),
                    (max - 1) as usize
                );
            } else {
                assert_eq!(
                    consensus_submission_data.clone().subnet_nodes.len(),
                    max as usize
                );
            }

            Network::emission_step(
                &mut WeightMeter::new(),
                System::block_number(),
                Network::get_current_epoch_as_u32(),
                Network::get_current_subnet_epoch_as_u32(subnet_id),
                subnet_id,
            );
        }

        assert_eq!(
            EmergencySubnetNodeElectionData::<Test>::try_get(subnet_id),
            Err(())
        );
    });
}
