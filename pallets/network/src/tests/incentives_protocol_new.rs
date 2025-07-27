use super::mock::*;
use super::test_utils::*;
use crate::Event;
use sp_core::OpaquePeerId as PeerId;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use crate::{
  Error, 
  SubnetElectedValidator,
  SubnetName, 
  TotalSubnetNodes,
  SubnetNodeClass,
  SubnetNodeConsensusData,
  SubnetsData,
  AccountSubnetStake,
  AccountSubnetDelegateStakeShares, 
  SubnetConsensusSubmission, 
  BaseValidatorReward,
  SubnetPenaltyCount, 
  MaxSubnetNodePenalties, 
  SubnetNodePenalties, 
  SubnetRemovalReason,
  MaxSubnetPenaltyCount, 
  HotkeySubnetNodeId, 
  SubnetNodeIdHotkey, 
  PeerIdSubnetNodeId,
  NetworkMinStakeBalance,
  SubnetOwnerPercentage,
  SubnetNodesData,
  TotalNodeDelegateStakeShares,
  TotalActiveSubnets,
  SubnetDelegateStakeRewardsPercentage,
  MaxSubnetNodes,
  MaxSubnets,
  FinalSubnetEmissionWeights,
  MinAttestationPercentage,
  ReputationIncreaseFactor,
  ReputationDecreaseFactor,
  MinVastMajorityAttestationPercentage,
  IdleClassificationEpochs,
  RegisteredSubnetNodesData,
  IncludedClassificationEpochs,
  NodeDelegateStakeBalance,
  TotalSubnetDelegateStakeBalance,
};
use frame_support::BoundedVec;
use strum::IntoEnumIterator;
use sp_io::crypto::sr25519_sign;
use sp_runtime::{MultiSigner, MultiSignature};
use sp_io::crypto::sr25519_generate;
use frame_support::pallet_prelude::Encode;
use sp_runtime::traits::IdentifyAccount;
use sp_core::Pair;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};


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
fn test_validate() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

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

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    // for x in subnet_node_data_vec.iter() {
    //   let subnet_node = SubnetNodesData::<Test>::get(
    //     subnet_id, 
    //     x.subnet_node_id
    //   );
    // }

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");

    let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();
    // assert!(hotkey != None, "Validator is None");

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(hotkey.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, epoch).unwrap();

    assert_eq!(submission.validator_id, validator_id.unwrap(), "Err: validator");
    assert_eq!(submission.data.len(), subnet_node_data_vec.len(), "Err: data len");
    let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
    assert_eq!(sum, DEFAULT_SCORE * total_subnet_nodes as u128, "Err: sum");
    assert_eq!(submission.attests.len(), 1, "Err: attests"); // validator auto-attests
    assert_eq!(submission.subnet_nodes.len() as u32, end, "Err: Nodes length");

    assert_err!(
      Network::validate(
        RuntimeOrigin::signed(hotkey.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      ),
      Error::<Test>::SubnetRewardsAlreadySubmitted
    );
  });
}

#[test]
fn test_validate_after_slot_error() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, 12, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = block_number / epoch_length;

    set_block_to_subnet_slot_epoch(epoch, subnet_id);

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");

    let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let block_number = System::block_number();
    // Set to the next slot start
    System::set_block_number(block_number + epoch_length);

    assert_err!(
      Network::validate(
        RuntimeOrigin::signed(hotkey.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      ),
      Error::<Test>::InvalidValidator
    );
  });
}

#[test]
fn test_validate_invalid_validator() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = block_number / epoch_length;

    set_block_to_subnet_slot_epoch(epoch, subnet_id);

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    if validator.clone() == account(1) {
      validator = account(2);
    }
  
    assert_err!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        subnet_node_data_vec,
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

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = block_number / epoch_length;

    set_block_to_subnet_slot_epoch(epoch, subnet_id);

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, epoch).unwrap();

    assert_eq!(submission.validator_id, validator_id.unwrap());
    assert_eq!(submission.data.len(), subnet_node_data_vec.len());

    // Attest
    for n in 1..total_subnet_nodes+1 {
      let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
      if attestor == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(subnets*max_subnet_nodes+n)), 
          subnet_id,
        )
      );
    }
    
    let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, epoch).unwrap();
    assert_eq!(submission.attests.len(), total_subnet_nodes as usize);
    
    for n in 0..total_subnet_nodes {
      let _n = n + 1;
      let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, _n).unwrap();
      if attestor == validator.clone() {
        assert_ne!(submission.attests.get(&(_n)), None);
        assert_eq!(submission.attests.get(&(_n)), Some(&System::block_number()));
      } else {
        assert_ne!(submission.attests.get(&(_n)), None);
        assert_eq!(submission.attests.get(&(_n)), Some(&System::block_number()));
      }
    }
  });
}

#[test]
fn test_attest_no_submission_err() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = block_number / epoch_length;

    set_block_to_subnet_slot_epoch(epoch, subnet_id);

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    // --- Get validator
    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    assert_err!(
      Network::attest(
        RuntimeOrigin::signed(validator), 
        subnet_id,
      ),
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

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
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

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // Attest
    for n in 0..total_subnet_nodes {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(hotkey.clone()), 
          subnet_id,
        )
      );
    }
    
    let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, epoch).unwrap();

    assert_eq!(submission.validator_id, validator_id.unwrap());
    assert_eq!(submission.data.len(), subnet_node_data_vec.len());
    let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
    assert_eq!(sum, DEFAULT_SCORE * total_subnet_nodes as u128);
    assert_eq!(submission.attests.len(), total_subnet_nodes as usize);

    for n in 0..total_subnet_nodes {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      assert_ne!(submission.attests.get(&_n), None);
      assert_eq!(submission.attests.get(&_n), Some(&System::block_number()));
    }

    for n in 0..total_subnet_nodes {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      assert_err!(
        Network::attest(
          RuntimeOrigin::signed(account(subnets*max_subnet_nodes+n)), 
          subnet_id,
        ),
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

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, max_subnet_nodes, deposit_amount, stake_amount);

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

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    for n in 0..total_subnet_nodes {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(hotkey.clone()), 
          subnet_id,
        )
      );
    }

    increase_epochs(1);
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

		assert!(result.is_some(), "Precheck consensus failed");

		let (consensus_submission_data, _) = result.unwrap();

		// ⸺ Calculate subnet distribution of rewards
		let (rewards_data, rewards_weight) = Network::calculate_rewards_v2(
			subnet_id, 
			subnet_emission_weights.validator_emissions, 
			*subnet_weight.unwrap()
		);

    let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
		for n in 0..max_subnet_nodes {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			assert_ne!(stake, 0);
			stake_snapshot.insert(hotkey.clone(), stake);
		}

		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

    let epoch = Network::get_current_epoch_as_u32();
    set_block_to_subnet_slot_epoch(epoch, subnet_id);
    let block_number = System::block_number();

    let dstake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    Network::distribute_rewards(
      subnet_id,
      block_number,
      epoch,
      consensus_submission_data,
      rewards_data,
      min_attestation_percentage,
      reputation_increase_factor,
      reputation_decrease_factor,
      min_vast_majority_attestation_percentage
    );

    for n in 0..max_subnet_nodes {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			if let Some(old_stake) = stake_snapshot.get(&hotkey) {
				assert!(stake > *old_stake);
			} else {
				assert!(false); // auto-fail
			}
		}

    let post_dstake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    assert!(post_dstake_balance > dstake_balance);
  });
}

#[test]
fn test_distribute_rewards_under_min_attest_slash_validator() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, max_subnet_nodes, deposit_amount, stake_amount);

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

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    increase_epochs(1);
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

		assert!(result.is_some(), "Precheck consensus failed");

		let (consensus_submission_data, _) = result.unwrap();

		// ⸺ Calculate subnet distribution of rewards
		let (rewards_data, rewards_weight) = Network::calculate_rewards_v2(
			subnet_id, 
			subnet_emission_weights.validator_emissions, 
			*subnet_weight.unwrap()
		);

    let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
		for n in 0..max_subnet_nodes {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			assert_ne!(stake, 0);
			stake_snapshot.insert(hotkey.clone(), stake);
		}

		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

    let validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
    assert_ne!(validator_stake, 0);

    let validator_penalties = SubnetNodePenalties::<Test>::get(subnet_id, validator_id.unwrap());
    assert_eq!(validator_penalties, 0);

    let epoch = Network::get_current_epoch_as_u32();
    set_block_to_subnet_slot_epoch(epoch, subnet_id);
    let block_number = System::block_number();

    Network::distribute_rewards(
      subnet_id,
      block_number,
      epoch,
      consensus_submission_data,
      rewards_data,
      min_attestation_percentage,
      reputation_increase_factor,
      reputation_decrease_factor,
      min_vast_majority_attestation_percentage
    );

    let post_validator_stake = AccountSubnetStake::<Test>::get(validator.clone(), subnet_id);
    assert!(validator_stake > post_validator_stake);

    let post_validator_penalties = SubnetNodePenalties::<Test>::get(subnet_id, validator_id.unwrap());
    assert_eq!(post_validator_penalties, 1);

    for n in 0..max_subnet_nodes {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);
      if hotkey.clone() == validator.clone() {
        continue
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
fn test_distribute_rewards_remove_node_at_max_penalties() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, max_subnet_nodes, deposit_amount, stake_amount);

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

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    let mut removing_subnet_node_id: Option<u32> = None;
    let max_subnet_node_penalties = MaxSubnetNodePenalties::<Test>::get(subnet_id);

    for n in 0..total_subnet_nodes {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      // mock penalties on the first non-validator to have them removed
      if removing_subnet_node_id.is_none() {
        removing_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone());
        SubnetNodePenalties::<Test>::insert(subnet_id, removing_subnet_node_id.unwrap(), max_subnet_node_penalties + 1);
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(hotkey.clone()), 
          subnet_id,
        )
      );
    }

    assert!(removing_subnet_node_id.is_some());

    increase_epochs(1);
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

		assert!(result.is_some(), "Precheck consensus failed");

		let (consensus_submission_data, _) = result.unwrap();

		// ⸺ Calculate subnet distribution of rewards
		let (rewards_data, _) = Network::calculate_rewards_v2(
			subnet_id, 
			subnet_emission_weights.validator_emissions, 
			*subnet_weight.unwrap()
		);

		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

    let epoch = Network::get_current_epoch_as_u32();
    set_block_to_subnet_slot_epoch(epoch, subnet_id);
    let block_number = System::block_number();

    Network::distribute_rewards(
      subnet_id,
      block_number,
      epoch,
      consensus_submission_data,
      rewards_data,
      min_attestation_percentage,
      reputation_increase_factor,
      reputation_decrease_factor,
      min_vast_majority_attestation_percentage
    );

    assert_eq!(SubnetNodesData::<Test>::try_get(subnet_id, removing_subnet_node_id.unwrap()), Err(()));
  });
}

#[test]
fn test_distribute_rewards_graduate_idle_to_included() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
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
    let idle_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 2);
    let idle_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 2);
    let idle_peer_id = peer(subnets*max_subnet_nodes+end+2);
    let idle_bootstrap_peer_id = peer(subnets*max_subnet_nodes+end+2);
    let _ = Balances::deposit_creating(&idle_coldkey.clone(), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(idle_coldkey.clone()),
        subnet_id,
        idle_hotkey.clone(),
        idle_peer_id.clone(),
        idle_bootstrap_peer_id.clone(),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, idle_hotkey.clone()).unwrap();
    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let start_epoch = subnet_node.classification.start_epoch;

    set_block_to_subnet_slot_epoch(start_epoch, subnet_id);

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(idle_coldkey.clone()),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
    assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

    // increase epochs up to when node should be able to graduate
    increase_epochs(idle_epochs + 1);
    let epoch = Network::get_current_epoch_as_u32();

    // ⸺ Generate subnet weights from stake/node count weights
		let _ = Network::handle_subnet_emission_weights(epoch);
		let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

		let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
		assert!(subnet_weight.is_some());

    // ⸺ Submit consnesus data
    set_block_to_subnet_slot_epoch(epoch, subnet_id);

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, end);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    for n in 0..end {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(hotkey.clone()), 
          subnet_id,
        )
      );
    }

    increase_epochs(1);
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

		assert!(result.is_some(), "Precheck consensus failed");

		let (consensus_submission_data, _) = result.unwrap();

		// ⸺ Calculate subnet distribution of rewards
		let (rewards_data, _) = Network::calculate_rewards_v2(
			subnet_id, 
			subnet_emission_weights.validator_emissions, 
			*subnet_weight.unwrap()
		);

    let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
		for n in 0..end {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			assert_ne!(stake, 0);
			stake_snapshot.insert(hotkey.clone(), stake);
		}

		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

    let epoch = Network::get_current_epoch_as_u32();
    set_block_to_subnet_slot_epoch(epoch, subnet_id);
    let block_number = System::block_number();

    Network::distribute_rewards(
      subnet_id,
      block_number,
      epoch,
      consensus_submission_data,
      rewards_data,
      min_attestation_percentage,
      reputation_increase_factor,
      reputation_decrease_factor,
      min_vast_majority_attestation_percentage
    );

    for n in 0..end {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			if let Some(old_stake) = stake_snapshot.get(&hotkey) {
				assert!(stake > *old_stake);
			} else {
				assert!(false); // auto-fail
			}
		}

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Included);
  });
}

#[test]
fn test_distribute_rewards_no_score_submitted_increase_penalties() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
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

    // ⸺ Generate subnet weights from stake/node count weights
		let _ = Network::handle_subnet_emission_weights(epoch);
		let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

		let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
		assert!(subnet_weight.is_some());

    // ⸺ Submit consnesus data
    set_block_to_subnet_slot_epoch(epoch, subnet_id);

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    // ⸺ Get scores leaving out the last node
    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, end - 1);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    for n in 0..end {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(hotkey.clone()), 
          subnet_id,
        )
      );
    }

    increase_epochs(1);
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

		assert!(result.is_some(), "Precheck consensus failed");

		let (consensus_submission_data, _) = result.unwrap();

		// ⸺ Calculate subnet distribution of rewards
		let (rewards_data, _) = Network::calculate_rewards_v2(
			subnet_id, 
			subnet_emission_weights.validator_emissions, 
			*subnet_weight.unwrap()
		);

    let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
		for n in 0..end {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			assert_ne!(stake, 0);
			stake_snapshot.insert(hotkey.clone(), stake);
		}

		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

    let end_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, end_hotkey.clone()).unwrap();
    assert_eq!(SubnetNodePenalties::<Test>::get(subnet_id, hotkey_subnet_node_id), 0);

    let epoch = Network::get_current_epoch_as_u32();
    set_block_to_subnet_slot_epoch(epoch, subnet_id);
    let block_number = System::block_number();

    Network::distribute_rewards(
      subnet_id,
      block_number,
      epoch,
      consensus_submission_data,
      rewards_data,
      min_attestation_percentage,
      reputation_increase_factor,
      reputation_decrease_factor,
      min_vast_majority_attestation_percentage
    );

    for n in 0..end-1 {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			if let Some(old_stake) = stake_snapshot.get(&hotkey) {
				assert!(stake > *old_stake);
			} else {
				assert!(false); // auto-fail
			}
		}

    assert_eq!(SubnetNodePenalties::<Test>::get(subnet_id, hotkey_subnet_node_id), 1);
  });
}

#[test]
fn test_attest_decrease_penalties_when_included() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, max_subnet_nodes, deposit_amount, stake_amount);

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

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    for n in 0..total_subnet_nodes {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(hotkey.clone()), 
          subnet_id,
        )
      );
    }

    increase_epochs(1);
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

		assert!(result.is_some(), "Precheck consensus failed");

		let (consensus_submission_data, _) = result.unwrap();

		// ⸺ Calculate subnet distribution of rewards
		let (rewards_data, rewards_weight) = Network::calculate_rewards_v2(
			subnet_id, 
			subnet_emission_weights.validator_emissions, 
			*subnet_weight.unwrap()
		);

    let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
		for n in 0..max_subnet_nodes {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			assert_ne!(stake, 0);
			stake_snapshot.insert(hotkey.clone(), stake);

      let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
      SubnetNodePenalties::<Test>::insert(subnet_id, subnet_node_id, 1);
      assert_eq!(SubnetNodePenalties::<Test>::get(subnet_id, subnet_node_id), 1);
		}

		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

    let epoch = Network::get_current_epoch_as_u32();
    set_block_to_subnet_slot_epoch(epoch, subnet_id);
    let block_number = System::block_number();

    Network::distribute_rewards(
      subnet_id,
      block_number,
      epoch,
      consensus_submission_data,
      rewards_data,
      min_attestation_percentage,
      reputation_increase_factor,
      reputation_decrease_factor,
      min_vast_majority_attestation_percentage
    );

    for n in 0..max_subnet_nodes {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			if let Some(old_stake) = stake_snapshot.get(&hotkey) {
				assert!(stake > *old_stake);
			} else {
				assert!(false); // auto-fail
			}

      let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
      assert_eq!(SubnetNodePenalties::<Test>::get(subnet_id, subnet_node_id), 0);
		}
  });
}

#[test]
fn test_distribute_rewards_graduate_included_to_validator() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
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
    let idle_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 2);
    let idle_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 2);
    let idle_peer_id = peer(subnets*max_subnet_nodes+end+2);
    let idle_bootstrap_peer_id = peer(subnets*max_subnet_nodes+end+2);
    let _ = Balances::deposit_creating(&idle_coldkey.clone(), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(idle_coldkey.clone()),
        subnet_id,
        idle_hotkey.clone(),
        idle_peer_id.clone(),
        idle_bootstrap_peer_id.clone(),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, idle_hotkey.clone()).unwrap();
    let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    let start_epoch = subnet_node.classification.start_epoch;

    set_block_to_subnet_slot_epoch(start_epoch, subnet_id);

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(idle_coldkey.clone()),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
    assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

    // increase epochs up to when node should be able to graduate
    increase_epochs(idle_epochs + 1);
    let epoch = Network::get_current_epoch_as_u32();

    // ⸺ Generate subnet weights from stake/node count weights
		let _ = Network::handle_subnet_emission_weights(epoch);
		let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

		let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
		assert!(subnet_weight.is_some());

    // ⸺ Submit consnesus data
    set_block_to_subnet_slot_epoch(epoch, subnet_id);

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, end);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    for n in 0..end {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(hotkey.clone()), 
          subnet_id,
        )
      );
    }

    increase_epochs(1);
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

		assert!(result.is_some(), "Precheck consensus failed");

		let (consensus_submission_data, _) = result.unwrap();

		// ⸺ Calculate subnet distribution of rewards
		let (rewards_data, _) = Network::calculate_rewards_v2(
			subnet_id, 
			subnet_emission_weights.validator_emissions, 
			*subnet_weight.unwrap()
		);

    let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
		for n in 0..end {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			assert_ne!(stake, 0);
			stake_snapshot.insert(hotkey.clone(), stake);
		}

		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

    let epoch = Network::get_current_epoch_as_u32();
    set_block_to_subnet_slot_epoch(epoch, subnet_id);
    let block_number = System::block_number();

    Network::distribute_rewards(
      subnet_id,
      block_number,
      epoch,
      consensus_submission_data,
      rewards_data,
      min_attestation_percentage,
      reputation_increase_factor,
      reputation_decrease_factor,
      min_vast_majority_attestation_percentage
    );

    for n in 0..end {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			if let Some(old_stake) = stake_snapshot.get(&hotkey) {
				assert!(stake > *old_stake);
			} else {
				assert!(false); // auto-fail
			}
		}

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Included);




    // NEW EPOCH
    let included_epochs = IncludedClassificationEpochs::<Test>::get(subnet_id);



    increase_epochs(included_epochs + 1);
    let block_number = System::block_number();
    let epoch = Network::get_current_epoch_as_u32();

    // ⸺ Generate subnet weights from stake/node count weights
		let _ = Network::handle_subnet_emission_weights(epoch);
		let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

		let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
		assert!(subnet_weight.is_some());

    // ⸺ Submit consnesus data
    set_block_to_subnet_slot_epoch(epoch, subnet_id);

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    // Get new node in consensus data
    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, end + 1);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    for n in 0..end {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(hotkey.clone()), 
          subnet_id,
        )
      );
    }

    increase_epochs(1);

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

		assert!(result.is_some(), "Precheck consensus failed");

		let (consensus_submission_data, _) = result.unwrap();

		// ⸺ Calculate subnet distribution of rewards
		let (rewards_data, _) = Network::calculate_rewards_v2(
			subnet_id, 
			subnet_emission_weights.validator_emissions, 
			*subnet_weight.unwrap()
		);

    let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
		for n in 0..end {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			assert_ne!(stake, 0);
			stake_snapshot.insert(hotkey.clone(), stake);
		}

		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

    let epoch = Network::get_current_epoch_as_u32();
    set_block_to_subnet_slot_epoch(epoch, subnet_id);
    let block_number = System::block_number();

    Network::distribute_rewards(
      subnet_id,
      block_number,
      epoch,
      consensus_submission_data,
      rewards_data,
      min_attestation_percentage,
      reputation_increase_factor,
      reputation_decrease_factor,
      min_vast_majority_attestation_percentage
    );

    for n in 0..end {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			if let Some(old_stake) = stake_snapshot.get(&hotkey) {
				assert!(stake > *old_stake);
			} else {
				assert!(false); // auto-fail
			}
		}

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Validator);
  });
}

#[test]
fn test_distribute_rewards_node_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    build_activated_subnet_new(subnet_name.clone(), 0, max_subnet_nodes, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let node_coldkey = get_coldkey(subnets, max_subnet_nodes, max_subnets);
    let node_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, max_subnets);
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, node_hotkey.clone()).unwrap();

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

    Network::elect_validator_v3(
      subnet_id,
      epoch,
      block_number
    );

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    for n in 0..total_subnet_nodes {
      let _n = n + 1;
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      if hotkey.clone() == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(hotkey.clone()), 
          subnet_id,
        )
      );
    }

    increase_epochs(1);
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
    let epoch = Network::get_current_epoch_as_u32();

    let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

		assert!(result.is_some(), "Precheck consensus failed");

		let (consensus_submission_data, _) = result.unwrap();

		// ⸺ Calculate subnet distribution of rewards
		let (rewards_data, rewards_weight) = Network::calculate_rewards_v2(
			subnet_id, 
			subnet_emission_weights.validator_emissions, 
			*subnet_weight.unwrap()
		);

    let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
		for n in 0..max_subnet_nodes {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			assert_ne!(stake, 0);
			stake_snapshot.insert(hotkey.clone(), stake);
		}

    let delegate_stake_balance = NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);

		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

    let epoch = Network::get_current_epoch_as_u32();
    set_block_to_subnet_slot_epoch(epoch, subnet_id);
    let block_number = System::block_number();

    Network::distribute_rewards(
      subnet_id,
      block_number,
      epoch,
      consensus_submission_data,
      rewards_data,
      min_attestation_percentage,
      reputation_increase_factor,
      reputation_decrease_factor,
      min_vast_majority_attestation_percentage
    );

    for n in 0..max_subnet_nodes {
			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

			if let Some(old_stake) = stake_snapshot.get(&hotkey) {
				assert!(stake > *old_stake);
			} else {
				assert!(false); // auto-fail
			}
		}

    let post_delegate_stake_balance = NodeDelegateStakeBalance::<Test>::get(subnet_id, subnet_node_id);
    assert!(post_delegate_stake_balance > delegate_stake_balance);
  });
}

// // #[test]
// // fn test_reward_subnets() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_name: Vec<u8> = "subnet-name".into();
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

// //     build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

// //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     // increase_epochs(1);

// //     let epoch_length = EpochLength::get();
// //     let epoch = System::block_number() / epoch_length;

// //     Network::do_epoch_preliminaries(System::block_number(), epoch);


// //     let subnet_node_data_vec = get_subnet_node_consensus_data(0, total_subnet_nodes);

// //     // --- Get validator
// //     let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch).unwrap();
// //     let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

// //     assert_ok!(
// //       Network::validate(
// //         RuntimeOrigin::signed(validator.clone()), 
// //         subnet_id,
// //         subnet_node_data_vec.clone(),
// //         None,
// //       )
// //     );

// //     // Attest
// //     for n in 1..total_subnet_nodes+1 {
// //       if account(n) == validator.clone() {
// //         continue
// //       }
// //       assert_ok!(
// //         Network::attest(
// //           RuntimeOrigin::signed(account(n)), 
// //           subnet_id,
// //         )
// //       );
// //     }
    
// //     Network::reward_subnets(System::block_number(), epoch);
// //   });
// // }

// #[test]
// fn test_reward_subnets_v2() {
//   new_test_ext().execute_with(|| {
//     let subnet_name: Vec<u8> = "subnet-name".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

//     build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     // increase_epochs(1);

//     let epoch_length = EpochLength::get();
//     let epoch = System::block_number() / epoch_length;

//     Network::do_epoch_preliminaries(System::block_number(), epoch);

//     let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

//     // --- Get validator
//     let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch).unwrap();
//     let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

//     assert_ok!(
//       Network::validate(
//         RuntimeOrigin::signed(validator.clone()), 
//         subnet_id,
//         subnet_node_data_vec.clone(),
//         None,
//       )
//     );

//     // Attest
//     for n in 1..total_subnet_nodes+1 {
//       if account(subnets*max_subnet_nodes+n) == validator.clone() {
//         continue
//       }
//       assert_ok!(
//         Network::attest(
//           RuntimeOrigin::signed(account(subnets*max_subnet_nodes+n)), 
//           subnet_id,
//         )
//       );
//     }
    
//     Network::reward_subnets_v2(System::block_number(), epoch);
//   });
// }


// // #[test]
// // fn test_reward_subnets_remove_subnet_node() {
// //   new_test_ext().execute_with(|| {
// //     let max_absent = MaxSubnetNodePenalties::<Test>::get();
// //     let subnet_name: Vec<u8> = "subnet-name".into();
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// // let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

// // build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

// //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     increase_epochs(1);

// //     let epoch_length = EpochLength::get();

// //     // shift node classes
// //     // validate n-1
// //     // attest   n-1
// //     // Simulate epochs
// //     for num in 0..max_absent+1 {
// //       let epoch = System::block_number() / epoch_length;
  
// //       let subnet_node_data_vec = get_subnet_node_consensus_data(0, total_subnet_nodes-1);
    
// //       // --- Insert validator
// //       SubnetElectedValidator::<Test>::insert(subnet_id, epoch, 1);
// //       let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

// //       // validate without n-1
// //       assert_ok!(
// //         Network::validate(
// //           RuntimeOrigin::signed(account(1)), 
// //           subnet_id,
// //           subnet_node_data_vec.clone(),
// //           None,
// //         )
// //       );
  
// //       // Attest without n-1
// //       for n in 1..total_subnet_nodes {
// //         let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
// //         if attestor == validator.clone() {
// //           continue
// //         }  
// //         assert_ok!(
// //           Network::attest(
// //             RuntimeOrigin::signed(account(n)), 
// //             subnet_id,
// //           )
// //         );
// //       }
      
// //       // --- Get submission data and count before node is removed
// //       // Check rewards
// //       // Ensure only attestors, validators, and validated get rewards
// //       let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, epoch).unwrap();

// //       // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
// //       let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(
// //         subnet_id, 
// //         &SubnetNodeClass::Validator, 
// //         epoch
// //       );

// //       let submission_nodes_count = submission_nodes.len() as u128;

// //       Network::reward_subnets(System::block_number(), epoch);
// //       let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, total_subnet_nodes);

// //       if num + 1 > max_absent {
// //         post_remove_subnet_node_ensures(total_subnet_nodes, subnet_id);
// //         // when node is removed they're SubnetNodePenalties is reset to zero
// //         assert_eq!(node_absent_count, 0);  
// //       } else {
// //         assert_eq!(node_absent_count, num+1);  
// //       }

// //       let base_reward_per_mb: u128 = BaseRewardPerMB::<Test>::get();
// //       let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<Test>::get();
// //       let overall_subnet_reward: u128 = Network::percent_mul(base_reward_per_mb, DEFAULT_MEM_MB);
// //       let delegate_stake_reward: u128 = Network::percent_mul(overall_subnet_reward, delegate_stake_rewards_percentage);
// //       let subnet_reward: u128 = overall_subnet_reward.saturating_sub(delegate_stake_reward);
      
// //       let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);

// //       let reward_ratio: u128 = Network::percent_div(DEFAULT_SCORE, sum);
// //       let account_reward: u128 = Network::percent_mul(reward_ratio, subnet_reward);
  
// //       let base_reward = BaseValidatorReward::<Test>::get();
  
// //       let submission_attestations: u128 = submission.attests.len() as u128;
// //       let attestation_percentage: u128 = Network::percent_div(submission_attestations, submission_nodes_count);

// //       // check each subnet nodes balance increased
// //       for n in 1..total_subnet_nodes+1 {
// //         if n == 1 {
// //           // validator
// //           let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
// //           let validator_reward: u128 = Network::percent_mul(base_reward, attestation_percentage);
// //           assert_eq!(stake_balance, amount + (account_reward * (num+1) as u128) + (validator_reward * (num+1) as u128));
// //         } else if n == total_subnet_nodes {
// //           // node removed | should have no rewards
// //           let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
// //           assert!(stake_balance == amount, "Invalid subnet node staking rewards");
// //         } else {
// //           // attestors
// //           let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
// //           assert!(stake_balance == amount + (account_reward * (num+1) as u128), "Invalid subnet node staking rewards");
// //         }
// //       }

// //       increase_epochs(1);
// //     }

// //     // node should be removed
// //     let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, account(total_subnet_nodes));
// //     assert_eq!(subnet_node_id, Err(()));

// //     let subnet_node_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer(total_subnet_nodes));
// //     assert_eq!(subnet_node_account, Err(()));
// //   });
// // }

// // // #[test]
// // // fn test_reward_subnets_absent_node_increment_decrement() {
// // //   new_test_ext().execute_with(|| {
// // //     let max_absent = MaxSubnetNodePenalties::<Test>::get();
// // //     let subnet_name: Vec<u8> = "subnet-name".into();
// // //     let deposit_amount: u128 = 10000000000000000000000;
// // //     let amount: u128 = 1000000000000000000000;

// // //     build_activated_subnet_new(subnet_name.clone(), 0, 15, deposit_amount, amount);

// // //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
// // //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// // //     increase_epochs(1);

// // //     let epoch_length = EpochLength::get();
// // //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);

// // //     // simulate epochs
// // //     for num in 0..10 {
// // //       let epoch = System::block_number() / epoch_length;

// // //       // --- Insert validator
// // //       SubnetElectedValidator::<Test>::insert(subnet_id, epoch, account(1));
    
// // //       if num % 2 == 0 {
// // //         // increment on even epochs

// // //         let subnet_node_data_vec = get_subnet_node_consensus_data(0, total_subnet_nodes-1);
    
// // //         assert_ok!(
// // //           Network::validate(
// // //             RuntimeOrigin::signed(account(1)), 
// // //             subnet_id,
// // //             subnet_node_data_vec.clone()
// // //           )
// // //         );
    
// // //         // Attest
// // //         for n in 1..total_subnet_nodes-1 {
// // //           assert_ok!(
// // //             Network::attest(
// // //               RuntimeOrigin::signed(account(n)), 
// // //               subnet_id,
// // //             )
// // //           );
// // //         }
        
// // //         Network::reward_subnets(System::block_number(), epoch);
  
// // //         let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, (total_subnet_nodes-1));
// // //         assert_eq!(node_absent_count, 1);
// // //       } else {
// // //         // decrement on odd epochs
// // //         let subnet_node_data_vec = get_subnet_node_consensus_data(0, total_subnet_nodes);
        
// // //         assert_ok!(
// // //           Network::validate(
// // //             RuntimeOrigin::signed(account(1)), 
// // //             subnet_id,
// // //             subnet_node_data_vec.clone()
// // //           )
// // //         );
    
// // //         // Attest
// // //         for n in 1..total_subnet_nodes {
// // //           assert_ok!(
// // //             Network::attest(
// // //               RuntimeOrigin::signed(account(n)), 
// // //               subnet_id,
// // //             )
// // //           );
// // //         }
        
// // //         Network::reward_subnets(System::block_number(), epoch);
  
// // //         let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, (total_subnet_nodes-1));
// // //         assert_eq!(node_absent_count, 0);  
// // //       }

// // //       increase_epochs(1);
// // //     }
// // //   });
// // // }

// // #[test]
// // fn test_reward_subnets_check_balances() {
// //   new_test_ext().execute_with(|| {
// //     let max_absent = MaxSubnetNodePenalties::<Test>::get();

// //     let subnet_name: Vec<u8> = "subnet-name".into();
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     build_activated_subnet_new(subnet_name.clone(), 0, 15, deposit_amount, amount);

// //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     increase_epochs(1);

// //     let epoch_length = EpochLength::get();
// //     let epoch = System::block_number() / epoch_length;

// //     let subnet_node_data_vec = get_subnet_node_consensus_data(0, total_subnet_nodes);
  
// //     // --- Insert validator
// //     SubnetElectedValidator::<Test>::insert(subnet_id, epoch, 1);
// //     let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

// //     // validate without n-1
// //     assert_ok!(
// //       Network::validate(
// //         RuntimeOrigin::signed(account(1)), 
// //         subnet_id,
// //         subnet_node_data_vec.clone(),
// //         None,
// //       )
// //     );

// //     // Attest without n-1
// //     for n in 1..total_subnet_nodes {
// //       let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
// //       if attestor == validator.clone() {
// //         continue
// //       }
// //       assert_ok!(
// //         Network::attest(
// //           RuntimeOrigin::signed(account(n)), 
// //           subnet_id,
// //         )
// //       );
// //     }
    
// //     // --- Get submission data and count before node is removed
// //     // Check rewards
// //     // Ensure only attestors, validators, and validated get rewards
// //     let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, epoch).unwrap();

// //     // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
// //     let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);
// //     let submission_nodes_count = submission_nodes.len() as u128;

// //     Network::reward_subnets(System::block_number(), epoch);
// //     let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, total_subnet_nodes-1);
// //     assert_eq!(node_absent_count, 0); 
          
// //     let base_reward_per_mb: u128 = BaseRewardPerMB::<Test>::get();
// //     let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<Test>::get();
// //     let overall_subnet_reward: u128 = Network::percent_mul(base_reward_per_mb, DEFAULT_MEM_MB);
// //     let delegate_stake_reward: u128 = Network::percent_mul(overall_subnet_reward, delegate_stake_rewards_percentage);
// //     let subnet_reward: u128 = overall_subnet_reward.saturating_sub(delegate_stake_reward);

// //     let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
// //     let reward_ratio: u128 = Network::percent_div(DEFAULT_SCORE, sum);
// //     let account_reward: u128 = Network::percent_mul(reward_ratio, subnet_reward);

// //     let base_reward = BaseValidatorReward::<Test>::get();

// //     let submission_attestations: u128 = submission.attests.len() as u128;
// //     let attestation_percentage: u128 = Network::percent_div(submission_attestations, submission_nodes_count);

// //     // check each subnet nodes balance increased
// //     for n in 1..total_subnet_nodes {
// //       if n == 1 {
// //         // validator
// //         let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
// //         let validator_reward: u128 = Network::percent_mul(base_reward, attestation_percentage);
// //         assert_eq!(stake_balance, amount + (account_reward as u128) + (validator_reward as u128));
// //       } else {
// //         // attestors
// //         let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
// //         assert_eq!(stake_balance, amount + (account_reward as u128));
// //       }
// //     }
// //   });
// // }

// // #[test]
// // fn test_reward_subnets_validator_slash() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_name: Vec<u8> = "subnet-name".into();
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

// //     build_activated_subnet_new(subnet_name.clone(), 0, 15, deposit_amount, stake_amount);

// //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     // increase_epochs(1);

// //     let epoch_length = EpochLength::get();
// //     let epoch = System::block_number() / epoch_length;

// //     Network::do_epoch_preliminaries(System::block_number(), epoch);

// //     let subnet_node_data_vec = get_subnet_node_consensus_data(0, total_subnet_nodes);

// //     // --- Get validator
// //     let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch).unwrap();
// //     let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

// //     assert_ok!(
// //       Network::validate(
// //         RuntimeOrigin::signed(validator.clone()), 
// //         subnet_id,
// //         subnet_node_data_vec.clone(),
// //         None,
// //       )
// //     );

// //     // No attests to ensure validator is slashed
    
// //     let before_slash_validator_stake_balance: u128 = AccountSubnetStake::<Test>::get(&validator.clone(), subnet_id);

// //     Network::reward_subnets(System::block_number(), epoch);

// //     let slashed_validator_stake_balance: u128 = AccountSubnetStake::<Test>::get(&validator.clone(), subnet_id);

// //     // Ensure validator was slashed
// //     assert!(before_slash_validator_stake_balance > slashed_validator_stake_balance, "Validator was not slashed")
// //   });
// // }

// #[test]
// fn test_reward_subnets_v2_validator_slash() {
//   new_test_ext().execute_with(|| {
//     let subnet_name: Vec<u8> = "subnet-name".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

//     build_activated_subnet_new(subnet_name.clone(), 0, 15, deposit_amount, stake_amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     // increase_epochs(1);

//     let epoch_length = EpochLength::get();
//     let epoch = System::block_number() / epoch_length;

//     Network::do_epoch_preliminaries(System::block_number(), epoch);

//     let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

//     // --- Get validator
//     let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch).unwrap();
//     let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

//     assert_ok!(
//       Network::validate(
//         RuntimeOrigin::signed(validator.clone()), 
//         subnet_id,
//         subnet_node_data_vec.clone(),
//         None,
//       )
//     );

//     // No attests to ensure validator is slashed
    
//     let before_slash_validator_stake_balance: u128 = AccountSubnetStake::<Test>::get(&validator.clone(), subnet_id);

//     Network::reward_subnets_v2(System::block_number(), epoch);

//     let slashed_validator_stake_balance: u128 = AccountSubnetStake::<Test>::get(&validator.clone(), subnet_id);

//     // Ensure validator was slashed
//     assert!(before_slash_validator_stake_balance > slashed_validator_stake_balance, "Validator was not slashed")
//   });
// }

// // #[test]
// // fn test_reward_subnets_subnet_penalty_count() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_name: Vec<u8> = "subnet-name".into();
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

// //     build_activated_subnet_new(subnet_name.clone(), 0, 15, deposit_amount, stake_amount);

// //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     increase_epochs(1);

// //     let epoch_length = EpochLength::get();
// //     let epoch = System::block_number() / epoch_length;

// //     let subnet_node_data_vec = get_subnet_node_consensus_data(0, total_subnet_nodes);

// //     // --- Insert validator
// //     SubnetElectedValidator::<Test>::insert(subnet_id, epoch, 1);
// //     let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

// //     assert_ok!(
// //       Network::validate(
// //         RuntimeOrigin::signed(account(1)), 
// //         subnet_id,
// //         Vec::new(),
// //         None,
// //       )
// //     );

// //     // Attest
// //     for n in 1..total_subnet_nodes+1 {
// //       let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
// //       if attestor == validator.clone() {
// //         continue
// //       }
// //       assert_ok!(
// //         Network::attest(
// //           RuntimeOrigin::signed(account(n)), 
// //           subnet_id,
// //         )
// //       );
// //     }
    
// //     Network::reward_subnets(System::block_number(), epoch);

// //     let subnet_penalty_count = SubnetPenaltyCount::<Test>::get(subnet_id);
// //     assert_eq!(subnet_penalty_count, 1);

// //     let subnet_node_penalty_count = SubnetNodePenalties::<Test>::get(subnet_id, 0);
// //     assert_eq!(subnet_node_penalty_count, 0);
// //   });
// // }

// #[test]
// fn test_reward_subnets_v2_subnet_penalty_count() {
//   new_test_ext().execute_with(|| {
//     let _ = env_logger::builder().is_test(true).try_init();

//     let subnet_name: Vec<u8> = "subnet-name".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

//     build_activated_subnet_new(subnet_name.clone(), 0, 15, deposit_amount, stake_amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     increase_epochs(1);

//     let epoch_length = EpochLength::get();
//     let epoch = System::block_number() / epoch_length;

//     let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

//     // --- Insert validator
//     SubnetElectedValidator::<Test>::insert(subnet_id, epoch, 1);
//     let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

//     assert_ok!(
//       Network::validate(
//         RuntimeOrigin::signed(validator.clone()), 
//         subnet_id,
//         Vec::new(),
//         None,
//       )
//     );

//     // Attest
//     for n in 1..total_subnet_nodes+1 {
//       let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
//       if attestor == validator.clone() {
//         continue
//       }
//       assert_ok!(
//         Network::attest(
//           RuntimeOrigin::signed(account(subnets*max_subnet_nodes+n)), 
//           subnet_id,
//         )
//       );
//     }
    
//     Network::reward_subnets_v2(System::block_number(), epoch);

//     let subnet_penalty_count = SubnetPenaltyCount::<Test>::get(subnet_id);
//     assert_eq!(subnet_penalty_count, 1);

//     let subnet_node_penalty_count = SubnetNodePenalties::<Test>::get(subnet_id, 0);
//     assert_eq!(subnet_node_penalty_count, 0);
//   });
// }

// // #[test]
// // fn test_reward_subnets_account_penalty_count() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_name: Vec<u8> = "subnet-name".into();
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

// //     build_activated_subnet_new(subnet_name.clone(), 0, 15, deposit_amount, stake_amount);

// //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     increase_epochs(1);

// //     let epoch_length = EpochLength::get();
// //     let epoch = System::block_number() / epoch_length;

// //     let subnet_node_data_vec = get_subnet_node_consensus_data(0, total_subnet_nodes);

// //     // --- Insert validator
// //     SubnetElectedValidator::<Test>::insert(subnet_id, epoch, 1);

// //     assert_ok!(
// //       Network::validate(
// //         RuntimeOrigin::signed(account(1)), 
// //         subnet_id,
// //         Vec::new(),
// //         None,
// //       )
// //     );

// //     // No Attest

// //     Network::reward_subnets(System::block_number(), epoch);

// //     let subnet_penalty_count = SubnetPenaltyCount::<Test>::get(subnet_id);
// //     assert_eq!(subnet_penalty_count, 1);

// //     let subnet_node_penalty_count = SubnetNodePenalties::<Test>::get(subnet_id, 1);
// //     assert_eq!(subnet_node_penalty_count, 1);
// //   });
// // }

// #[test]
// fn test_reward_subnets_v2_account_penalty_count() {
//   new_test_ext().execute_with(|| {
//     let subnet_name: Vec<u8> = "subnet-name".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

//     build_activated_subnet_new(subnet_name.clone(), 0, 15, deposit_amount, stake_amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     increase_epochs(1);

//     let epoch_length = EpochLength::get();
//     let epoch = System::block_number() / epoch_length;

//     let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);

//     // --- Insert validator
//     Network::do_epoch_preliminaries(System::block_number(), epoch);

//     // --- Get validator
//     let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch).unwrap();
//     let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

//     assert_ok!(
//       Network::validate(
//         RuntimeOrigin::signed(validator.clone()), 
//         subnet_id,
//         Vec::new(),
//         None,
//       )
//     );

//     // No Attest

//     Network::reward_subnets_v2(System::block_number(), epoch);

//     let subnet_penalty_count = SubnetPenaltyCount::<Test>::get(subnet_id);
//     assert_eq!(subnet_penalty_count, 1);

//     let subnet_node_penalty_count = SubnetNodePenalties::<Test>::get(subnet_id, 1);
//     assert_eq!(subnet_node_penalty_count, 1);
//   });
// }

// // ///

// // ///



// // #[test]
// // fn test_do_epoch_preliminaries_deactivate_subnet_enactment_period() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_name: Vec<u8> = "subnet-name".into();

// //     let epoch_length = EpochLength::get();
// //     let block_number = System::block_number();
// //     let epoch = System::block_number().saturating_div(epoch_length);
  
// //     let cost = Network::registration_cost(epoch);
  
// //     let _ = Balances::deposit_creating(&account(1), cost+1000);
  
// //     let add_subnet_data = RegistrationSubnetData {
// //       name: subnet_name.clone().into(),
// //       max_node_registration_epochs: 16,
// //       node_registration_interval: 0,
// //       node_queue_period: 1,
//       // initial_coldkeys: Some(BTreeSet::new()),
//       // initial_coldkeys: None,
// //     };
  
// //     let epoch_length = EpochLength::get();
// //     let block_number = System::block_number();
// //     let epoch = System::block_number().saturating_div(epoch_length);
// //     let next_registration_epoch = Network::get_next_registration_epoch(epoch);
// //     increase_epochs(next_registration_epoch - epoch);

// //     // --- Register subnet for activation
// //     assert_ok!(
// //       Network::register_subnet(
// //         RuntimeOrigin::signed(account(1)),
// //         add_subnet_data,
// //       )
// //     );

// //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
// //     let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

// //     let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance();
// //     let _ = Balances::deposit_creating(&account(1), min_subnet_delegate_stake+1000);
  
// //     let mut subnet_registering = true;
// //     let subnet_activation_enactment_blocks = SubnetActivationEnactmentBlocks::<Test>::get();

// //     while subnet_registering {
// //       increase_epochs(1);
// //       let block_number = System::block_number();

// //       let epoch_length = EpochLength::get();
// //       let epoch = System::block_number() / epoch_length;  

// //       Network::do_epoch_preliminaries(block_number, epoch, epoch_length);
      
// //       if block_number > max_registration_block + subnet_activation_enactment_blocks {
// //         assert_eq!(
// //           *network_events().last().unwrap(),
// //           Event::SubnetDeactivated {
// //             subnet_id: subnet_id, 
// //             reason: SubnetRemovalReason::EnactmentPeriod
// //           }
// //         );

// //         let removed_subnet = SubnetsData::<Test>::try_get(subnet_id);
// //         assert_eq!(removed_subnet, Err(()));
// //         subnet_registering = false;
// //       } else {
// //         let registered_subnet = SubnetsData::<Test>::try_get(subnet_id).unwrap();
// //         assert_eq!(registered_subnet.id, subnet_id);
// //       }
// //     }
// //   });
// // }

// #[test]
// fn test_do_epoch_preliminaries_deactivate_min_subnet_delegate_stake() {
//   new_test_ext().execute_with(|| {
//     let subnet_name: Vec<u8> = "subnet-name".into();
    
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

//     build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     // --- Remove delegate stake to force MinSubnetDelegateStake removal reason
//     let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(1000), subnet_id);
//     assert_ok!(
//       Network::remove_delegate_stake(
//         RuntimeOrigin::signed(account(1000)),
//         subnet_id,
//         delegate_shares,
//       ) 
//     );

//     increase_epochs(1);
//     let block_number = System::block_number();

//     let epoch_length = EpochLength::get();
//     let epoch = System::block_number() / epoch_length;  

//     Network::do_epoch_preliminaries(block_number, epoch);
//     assert_eq!(
//       *network_events().last().unwrap(),
//       Event::SubnetDeactivated {
//         subnet_id: subnet_id, 
//         reason: SubnetRemovalReason::MinSubnetDelegateStake
//       }
//     ); 
//   });
// }

// #[test]
// fn test_do_epoch_preliminaries_deactivate_max_penalties() {
//   new_test_ext().execute_with(|| {
//     let subnet_name: Vec<u8> = "subnet-name".into();
    
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

//     build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     let max_subnet_penalty_count = MaxSubnetPenaltyCount::<Test>::get();
//     SubnetPenaltyCount::<Test>::insert(subnet_id, max_subnet_penalty_count + 1);

//     increase_epochs(1);
//     let block_number = System::block_number();

//     let epoch_length = EpochLength::get();
//     let epoch = System::block_number() / epoch_length;

//     Network::do_epoch_preliminaries(block_number, epoch);
//     assert_eq!(
//       *network_events().last().unwrap(),
//       Event::SubnetDeactivated {
//         subnet_id: subnet_id, 
//         reason: SubnetRemovalReason::MaxPenalties
//       }
//     ); 
//   });
// }

// #[test]
// fn test_do_epoch_preliminaries_choose_validator() {
//   new_test_ext().execute_with(|| {
//     let subnet_name: Vec<u8> = "subnet-name".into();
    
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

//     build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     increase_epochs(1);
//     let block_number = System::block_number();

//     let epoch_length = EpochLength::get();
//     let epoch = System::block_number() / epoch_length;

//     Network::do_epoch_preliminaries(block_number, epoch);
//     let validator = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
//     assert_ne!(validator, None);
//   });
// }

// // // // #[test]
// // // // fn test_add_subnet_node_signature() {
// // // //   new_test_ext().execute_with(|| {
// // // //     let subnet_name: Vec<u8> = "subnet-name".into();

// // // //     build_subnet(subnet_name.clone());
// // // //     assert_eq!(Network::total_subnets(), 1);

// // // // let mut n_peers: u32 = Network::max_subnet_nodes();
// // // // if n_peers > MAX_SUBNET_NODES {
// // // //   n_peers = MAX_SUBNET_NODES
// // // // }

// // // //     let deposit_amount: u128 = 1000000000000000000000000;
// // // //     let amount: u128 = 1000000000000000000000;
// // // //     let mut amount_staked: u128 = 0;

// // // //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

// // // //     let encoded_peer_id = Encode::encode(&peer(1).0.to_vec());
// // // //     let public = sr25519_generate(0.into(), None);
// // // //     let who_account: AccountIdOf<Test> = MultiSigner::Sr25519(public).into_account().into();
// // // //     let signature =
// // // //       MultiSignature::Sr25519(sr25519_sign(0.into(), &public, &encoded_peer_id).unwrap());

// // // //     assert_ok!(
// // // //       Network::add_subnet_node(
// // // //         RuntimeOrigin::signed(account(1)),
// // // account(1),
// // // //         subnet_id,
// // // //         peer(1),
// // // //         amount,
// // // //         // signature,
// // // //         // who_account
// // // //       ) 
// // // //     );

// // // //     let node_set = SubnetNodesClasses::<Test>::get(subnet_id, SubnetNodeClass::Idle);
// // // //     assert_eq!(node_set.len(), n_peers as usize);

// // // //   })
// // // // }

// // // // #[test]
// // // // fn validate_signature() {
// // // // 	new_test_ext().execute_with(|| {
// // // // 		let user_1_pair = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
// // // // 		let user_1_signer = MultiSigner::Sr25519(user_1_pair.public());
// // // // 		let user_1 = user_1_signer.clone().into_account();
// // // // 		let peer_id: PeerId = peer(1);
// // // // 		let encoded_data = Encode::encode(&peer_id);
// // // // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&encoded_data));
// // // // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));

// // // // 		let mut wrapped_data: Vec<u8> = Vec::new();
// // // // 		wrapped_data.extend(b"<Bytes>");
// // // // 		wrapped_data.extend(&encoded_data);
// // // // 		wrapped_data.extend(b"</Bytes>");

// // // // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&wrapped_data));
// // // // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));
// // // // 	})
// // // // }

// // // // #[test]
// // // // fn validate_signature_and_peer() {
// // // // 	new_test_ext().execute_with(|| {
// // // //     // validate signature
// // // // 		let user_1_pair = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
// // // // 		let user_1_signer = MultiSigner::Sr25519(user_1_pair.public());
// // // // 		let user_1 = user_1_signer.clone().into_account();
// // // // 		let peer_id: PeerId = peer(1);
// // // // 		let encoded_data = Encode::encode(&peer_id);
// // // // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&encoded_data));
// // // // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));

// // // // 		let mut wrapped_data: Vec<u8> = Vec::new();
// // // // 		wrapped_data.extend(b"<Bytes>");
// // // // 		wrapped_data.extend(&encoded_data);
// // // // 		wrapped_data.extend(b"</Bytes>");

// // // // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&wrapped_data));
// // // // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));

// // // //     // validate signature is the owner of the peer_id
// // // //     let subnet_name: Vec<u8> = "subnet-name".into();

// // // //     build_subnet(subnet_name.clone());

// // // //     let deposit_amount: u128 = 10000000000000000000000;
// // // //     let amount: u128 = 1000000000000000000000;

// // // //     let mut total_staked: u128 = 0;

// // // //     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

// // // //     let _ = Balances::deposit_creating(&user_1, deposit_amount);
    
// // // //     assert_ok!(
// // // //       Network::add_subnet_node(
// // // //         RuntimeOrigin::signed(user_1),
// // // //         subnet_id,
// // // //         peer(1),
// // // //         amount,
// // // //       ) 
// // // //     );
// // // // 	})
// // // // }

// #[test]
// fn test_reward_subnets_check_balances() {
//   new_test_ext().execute_with(|| {
//     let _ = env_logger::builder().is_test(true).try_init();

//     let subnet_name: Vec<u8> = "subnet-name".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

//     build_activated_subnet_with_delegator_rewards(
//       subnet_name.clone(), 
//       0, 
//       16, 
//       deposit_amount, 
//       stake_amount,
//       DEFAULT_DELEGATE_REWARD_RATE,
//     );

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), amount+500);

//     assert_ok!(
//       Network::add_to_node_delegate_stake(
//         RuntimeOrigin::signed(account(total_subnet_nodes+1)), 
//         subnet_id,
//         0,
//         amount,
//       )
//     );

//     increase_epochs(1);

//     let epoch = get_epoch();

//     let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);
  
//     // --- Insert validator
//     SubnetElectedValidator::<Test>::insert(subnet_id, epoch, 1);
//     let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

//     // validate without n-1
//     assert_ok!(
//       Network::validate(
//         RuntimeOrigin::signed(validator.clone()), 
//         subnet_id,
//         subnet_node_data_vec.clone(),
//         None,
//       )
//     );

//     // Attest without n-1
//     for n in 1..total_subnet_nodes+1 {
//       let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
//       if attestor == validator.clone() {
//         continue
//       }
//       assert_ok!(
//         Network::attest(
//           RuntimeOrigin::signed(account(subnets*max_subnet_nodes+n)), 
//           subnet_id,
//         )
//       );
//     }
    
//     // --- Get submission data and count before node is removed
//     // Check rewards
//     // Ensure only attestors, validators, and validated get rewards
//     let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, epoch).unwrap();

//     assert_ok!(Network::reward_subnets_v2(System::block_number(), epoch));

//     let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, total_subnet_nodes-1);
//     assert_eq!(node_absent_count, 0); 

//     let mut rewards: u128 = Network::get_epoch_emissions(epoch);

//     let total_issuance: u128 = Network::get_total_network_issuance();

//     let subnet_owner_percentage = SubnetOwnerPercentage::<Test>::get();
//     let delegate_stake_rewards_percentage = SubnetDelegateStakeRewardsPercentage::<Test>::get(subnet_id);

//     // test weight
//     let weight = 1e+18 as u128;

//     let overall_subnet_reward: u128 = Network::percent_mul(rewards, weight);

//     // --- Get owner rewards
//     let subnet_owner_reward: u128 = Network::percent_mul(overall_subnet_reward, subnet_owner_percentage);

//     // --- Get subnet rewards minus owner cut
//     let subnet_reward: u128 = overall_subnet_reward.saturating_sub(subnet_owner_reward);

//     // --- Get delegators rewards
//     let delegate_stake_reward: u128 = Network::percent_mul(subnet_reward, delegate_stake_rewards_percentage);

//     // --- Get subnet nodes rewards
//     let subnet_node_reward: u128 = subnet_reward.saturating_sub(delegate_stake_reward);

//     // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
//     let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);
//     let submission_nodes_count = submission_nodes.len() as u128;

//     let attestations: u128 = submission.attests.len() as u128;
//     let attestation_percentage: u128 = Network::percent_div(attestations, submission_nodes_count);
//     assert_eq!(attestation_percentage, 1e+18 as u128);

//     let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
//     let score_percentage: u128 = Network::percent_div(DEFAULT_SCORE, sum);

//     let mut account_reward: u128 = Network::percent_mul(score_percentage, subnet_node_reward);

    
//     for n in 1..total_subnet_nodes+1 {
//       let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(subnets*max_subnet_nodes+n)).unwrap();
//       let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
//       let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//       let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(subnets*max_subnet_nodes+n), subnet_id);

//       if subnet_node_id_hotkey == validator.clone() {
//         // validator
//         let validator_reward: u128 = Network::get_validator_reward(attestation_percentage);
//         let validator_total_reward: u128 = (account_reward as u128) + (validator_reward as u128);
//         assert_eq!(stake_balance, amount + validator_total_reward);
//       } else {
//         assert_eq!(stake_balance, amount + account_reward);
//       }
//     }
//   });
// }

// #[test]
// fn test_reward_subnets_with_delegate_node_staking_check_balances() {
//   new_test_ext().execute_with(|| {
//     let _ = env_logger::builder().is_test(true).try_init();

//     let subnet_name: Vec<u8> = "subnet-name".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

//     build_activated_subnet_with_delegator_rewards(
//       subnet_name.clone(), 
//       0, 
//       16, 
//       deposit_amount, 
//       stake_amount,
//       DEFAULT_DELEGATE_REWARD_RATE,
//     );

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    
//     for n in 1..total_subnet_nodes+1 {
//       let _ = Balances::deposit_creating(&account((subnets*max_subnet_nodes+n)+2), amount+500);

//       assert_ok!(
//         Network::add_to_node_delegate_stake(
//           RuntimeOrigin::signed(account((subnets*max_subnet_nodes+n)+2)), 
//           subnet_id,
//           n,
//           amount,
//         )
//       );  
//     }

//     increase_epochs(1);

//     let epoch = get_epoch();

//     let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, total_subnet_nodes);
  
//     // --- Insert validator
//     SubnetElectedValidator::<Test>::insert(subnet_id, epoch, 1);
//     let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

//     // validate without n-1
//     assert_ok!(
//       Network::validate(
//         RuntimeOrigin::signed(validator.clone()), 
//         subnet_id,
//         subnet_node_data_vec.clone(),
//         None,
//       )
//     );

//     // Attest without n-1
//     for n in 1..total_subnet_nodes+1 {
//       let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
//       if attestor == validator.clone() {
//         continue
//       }
//       assert_ok!(
//         Network::attest(
//           RuntimeOrigin::signed(account(subnets*max_subnet_nodes+n)), 
//           subnet_id,
//         )
//       );
//     }
    
//     // --- Get submission data and count before node is removed
//     // Check rewards
//     // Ensure only attestors, validators, and validated get rewards
//     let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, epoch).unwrap();

//     assert_ok!(Network::reward_subnets_v2(System::block_number(), epoch));

//     let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, total_subnet_nodes-1);
//     assert_eq!(node_absent_count, 0); 

//     let mut rewards: u128 = Network::get_epoch_emissions(epoch);

//     let total_issuance: u128 = Network::get_total_network_issuance();

//     let subnet_owner_percentage = SubnetOwnerPercentage::<Test>::get();
//     let delegate_stake_rewards_percentage = SubnetDelegateStakeRewardsPercentage::<Test>::get(subnet_id);

//     // 100% in this example, only one subnet in this test case
//     let weight = 1e+18 as u128;

//     let overall_subnet_reward: u128 = Network::percent_mul(rewards, weight);

//     // --- Get owner rewards
//     let subnet_owner_reward: u128 = Network::percent_mul(overall_subnet_reward, subnet_owner_percentage);

//     // --- Get subnet rewards minus owner cut
//     let subnet_reward: u128 = overall_subnet_reward.saturating_sub(subnet_owner_reward);

//     // --- Get delegators rewards
//     let delegate_stake_reward: u128 = Network::percent_mul(subnet_reward, delegate_stake_rewards_percentage);

//     // --- Get subnet nodes rewards
//     let subnet_node_reward: u128 = subnet_reward.saturating_sub(delegate_stake_reward);

//     // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
//     let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);
//     let submission_nodes_count = submission_nodes.len() as u128;

//     let attestations: u128 = submission.attests.len() as u128;
//     let attestation_percentage: u128 = Network::percent_div(attestations, submission_nodes_count);
//     assert_eq!(attestation_percentage, 1e+18 as u128);

//     let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
//     let score_percentage: u128 = Network::percent_div(DEFAULT_SCORE, sum);

//     let mut account_reward: u128 = Network::percent_mul(score_percentage, subnet_node_reward);

    
//     for n in 1..total_subnet_nodes+1 {
//       let mut node_reward = account_reward;
//       let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(subnets*max_subnet_nodes+n)).unwrap();
//       let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
//       let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//       let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(subnets*max_subnet_nodes+n), subnet_id);


//       assert_ne!(subnet_node.delegate_reward_rate, 0);

//       if subnet_node_id_hotkey == validator.clone() {
//         // validator
//         // let validator_reward: u128 = Network::get_validator_reward(attestation_percentage);
//         node_reward = node_reward + Network::get_validator_reward(attestation_percentage);

//         if subnet_node.delegate_reward_rate != 0 {
//           let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(subnet_id, hotkey_subnet_node_id);
//           if total_node_delegated_stake_shares != 0 {
//             let node_delegate_reward = Network::percent_mul(node_reward, subnet_node.delegate_reward_rate);
//             node_reward = node_reward - node_delegate_reward;
//           }
//         }
//         // let validator_total_reward: u128 = (node_reward as u128) + (validator_reward as u128);

//         assert_eq!(stake_balance, amount + node_reward);
//       } else {
//         if subnet_node.delegate_reward_rate != 0 {
//           let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(subnet_id, hotkey_subnet_node_id);
//           if total_node_delegated_stake_shares != 0 {
//             let node_delegate_reward = Network::percent_mul(node_reward, subnet_node.delegate_reward_rate);
//             node_reward = node_reward - node_delegate_reward;
//           }
//         }
//         assert_eq!(stake_balance, amount + node_reward);
//       }
//     }
//   });
// }

// #[test]
// fn test_calculate_stake_weights_v2() {
//   new_test_ext().execute_with(|| {
//     let _ = env_logger::builder().is_test(true).try_init();

//     let subnet_name: Vec<u8> = "subnet-name".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

//     build_activated_subnet_new(subnet_name.clone(), 0, 12, deposit_amount, stake_amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//     increase_epochs(2);
// 		let epoch = get_epoch();

//     let stake_weights = Network::calculate_stake_weights_v2(epoch);

//     // assert!(stake_weights.iter().len() > 0);
//   });
// }