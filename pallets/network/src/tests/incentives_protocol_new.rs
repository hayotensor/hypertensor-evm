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
  SubnetNodeConsecutiveIncludedEpochs,
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
		let (rewards_data, rewards_weight) = Network::calculate_rewards(
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
		let (rewards_data, rewards_weight) = Network::calculate_rewards(
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
		let (rewards_data, _) = Network::calculate_rewards(
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
    let idle_bootnode_peer_id = peer(subnets*max_subnet_nodes+end+2);
    let _ = Balances::deposit_creating(&idle_coldkey.clone(), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(idle_coldkey.clone()),
        subnet_id,
        idle_hotkey.clone(),
        idle_peer_id.clone(),
        idle_bootnode_peer_id.clone(),
        None,
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
		let (rewards_data, _) = Network::calculate_rewards(
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
		let (rewards_data, _) = Network::calculate_rewards(
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
		let (rewards_data, rewards_weight) = Network::calculate_rewards(
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

// #[test]
// fn test_distribute_rewards_graduate_included_to_validator() {
//   new_test_ext().execute_with(|| {
//     let subnet_name: Vec<u8> = "subnet-name".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//     let max_subnets = MaxSubnets::<Test>::get();
//     let end = max_subnet_nodes - 1;

//     build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//     let idle_epochs = IdleClassificationEpochs::<Test>::get(subnet_id);

//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = block_number / epoch_length;

//     // ⸺ Register and activate node into Idle classification
//     let idle_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 2);
//     let idle_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 2);
//     let idle_peer_id = peer(subnets*max_subnet_nodes+end+2);
//     let idle_bootnode_peer_id = peer(subnets*max_subnet_nodes+end+2);
//     let _ = Balances::deposit_creating(&idle_coldkey.clone(), deposit_amount);

//     assert_ok!(
//       Network::register_subnet_node(
//         RuntimeOrigin::signed(idle_coldkey.clone()),
//         subnet_id,
//         idle_hotkey.clone(),
//         idle_peer_id.clone(),
//         idle_bootnode_peer_id.clone(),
//         0,
//         amount,
//         None,
//         None,
//         None,
//       )
//     );

//     let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, idle_hotkey.clone()).unwrap();
//     let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//     let start_epoch = subnet_node.classification.start_epoch;

//     set_block_to_subnet_slot_epoch(start_epoch, subnet_id);

//     let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//     assert_ok!(
//       Network::activate_subnet_node(
//         RuntimeOrigin::signed(idle_coldkey.clone()),
//         subnet_id,
//         hotkey_subnet_node_id
//       )
//     );

//     let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//     assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
//     assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

//     // increase epochs up to when node should be able to graduate
//     increase_epochs(idle_epochs + 1);
//     let epoch = Network::get_current_epoch_as_u32();

//     // ⸺ Generate subnet weights from stake/node count weights
// 		let _ = Network::handle_subnet_emission_weights(epoch);
// 		let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

// 		let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
// 		assert!(subnet_weight.is_some());

//     // ⸺ Submit consnesus data
//     set_block_to_subnet_slot_epoch(epoch, subnet_id);

//     Network::elect_validator_v3(
//       subnet_id,
//       epoch,
//       block_number
//     );

//     let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
//     assert!(validator_id != None, "Validator is None");
//     assert!(validator_id != Some(0), "Validator is 0");

//     let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

//     let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
//     let epoch = Network::get_current_epoch_as_u32();

//     let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, end);

//     assert_ok!(
//       Network::validate(
//         RuntimeOrigin::signed(validator.clone()), 
//         subnet_id,
//         subnet_node_data_vec.clone(),
//         None,
//       )
//     );

//     for n in 0..end {
//       let _n = n + 1;
//       let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
//       if hotkey.clone() == validator.clone() {
//         continue
//       }
//       assert_ok!(
//         Network::attest(
//           RuntimeOrigin::signed(hotkey.clone()), 
//           subnet_id,
//         )
//       );
//     }

//     increase_epochs(1);
//     let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
//     let epoch = Network::get_current_epoch_as_u32();

//     let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

// 		assert!(result.is_some(), "Precheck consensus failed");

// 		let (consensus_submission_data, _) = result.unwrap();

// 		// ⸺ Calculate subnet distribution of rewards
// 		let (rewards_data, _) = Network::calculate_rewards(
// 			subnet_id, 
// 			subnet_emission_weights.validator_emissions, 
// 			*subnet_weight.unwrap()
// 		);

//     let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
// 		for n in 0..end {
// 			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

// 			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

// 			assert_ne!(stake, 0);
// 			stake_snapshot.insert(hotkey.clone(), stake);
// 		}

// 		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
//     let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
//     let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
//     let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

//     let epoch = Network::get_current_epoch_as_u32();
//     set_block_to_subnet_slot_epoch(epoch, subnet_id);
//     let block_number = System::block_number();

//     Network::distribute_rewards(
//       subnet_id,
//       block_number,
//       epoch,
//       consensus_submission_data,
//       rewards_data,
//       min_attestation_percentage,
//       reputation_increase_factor,
//       reputation_decrease_factor,
//       min_vast_majority_attestation_percentage
//     );

//     for n in 0..end {
// 			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

// 			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

// 			if let Some(old_stake) = stake_snapshot.get(&hotkey) {
// 				assert!(stake > *old_stake);
// 			} else {
// 				assert!(false); // auto-fail
// 			}
// 		}

//     let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//     assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Included);




//     // NEW EPOCH
//     let included_epochs = IncludedClassificationEpochs::<Test>::get(subnet_id);



//     increase_epochs(included_epochs + 1);
//     let block_number = System::block_number();
//     let epoch = Network::get_current_epoch_as_u32();

//     // ⸺ Generate subnet weights from stake/node count weights
// 		let _ = Network::handle_subnet_emission_weights(epoch);
// 		let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(epoch);

// 		let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
// 		assert!(subnet_weight.is_some());

//     // ⸺ Submit consnesus data
//     set_block_to_subnet_slot_epoch(epoch, subnet_id);

//     Network::elect_validator_v3(
//       subnet_id,
//       epoch,
//       block_number
//     );

//     let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, epoch);
//     assert!(validator_id != None, "Validator is None");
//     assert!(validator_id != Some(0), "Validator is 0");

//     let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

//     let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
//     let epoch = Network::get_current_epoch_as_u32();

//     // Get new node in consensus data
//     let subnet_node_data_vec = get_subnet_node_consensus_data(subnets, max_subnet_nodes, 0, end + 1);

//     assert_ok!(
//       Network::validate(
//         RuntimeOrigin::signed(validator.clone()), 
//         subnet_id,
//         subnet_node_data_vec.clone(),
//         None,
//       )
//     );

//     for n in 0..end {
//       let _n = n + 1;
//       let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
//       if hotkey.clone() == validator.clone() {
//         continue
//       }
//       assert_ok!(
//         Network::attest(
//           RuntimeOrigin::signed(hotkey.clone()), 
//           subnet_id,
//         )
//       );
//     }

//     increase_epochs(1);

//     let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
//     let epoch = Network::get_current_epoch_as_u32();

//     let result = Network::precheck_consensus_submission(subnet_id, epoch - 1);

// 		assert!(result.is_some(), "Precheck consensus failed");

// 		let (consensus_submission_data, _) = result.unwrap();

// 		// ⸺ Calculate subnet distribution of rewards
// 		let (rewards_data, _) = Network::calculate_rewards(
// 			subnet_id, 
// 			subnet_emission_weights.validator_emissions, 
// 			*subnet_weight.unwrap()
// 		);

//     let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
// 		for n in 0..end {
// 			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

// 			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

// 			assert_ne!(stake, 0);
// 			stake_snapshot.insert(hotkey.clone(), stake);
// 		}

// 		let min_attestation_percentage = MinAttestationPercentage::<Test>::get();
//     let reputation_increase_factor = ReputationIncreaseFactor::<Test>::get();
//     let reputation_decrease_factor = ReputationDecreaseFactor::<Test>::get();
//     let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<Test>::get();

//     let epoch = Network::get_current_epoch_as_u32();
//     set_block_to_subnet_slot_epoch(epoch, subnet_id);
//     let block_number = System::block_number();

//     Network::distribute_rewards(
//       subnet_id,
//       block_number,
//       epoch,
//       consensus_submission_data,
//       rewards_data,
//       min_attestation_percentage,
//       reputation_increase_factor,
//       reputation_decrease_factor,
//       min_vast_majority_attestation_percentage
//     );

//     for n in 0..end {
// 			let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, n+1);

// 			let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

// 			if let Some(old_stake) = stake_snapshot.get(&hotkey) {
// 				assert!(stake > *old_stake);
// 			} else {
// 				assert!(false); // auto-fail
// 			}
// 		}

//     let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//     assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Validator);
//   });
// }

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
    let idle_bootnode_peer_id = peer(subnets*max_subnet_nodes+end+2);
    let _ = Balances::deposit_creating(&idle_coldkey.clone(), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(idle_coldkey.clone()),
        subnet_id,
        idle_hotkey.clone(),
        idle_peer_id.clone(),
        idle_bootnode_peer_id.clone(),
        None,
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
		let (rewards_data, _) = Network::calculate_rewards(
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

    for e in 0..idle_epochs {
      increase_epochs(1);
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
      let (rewards_data, _) = Network::calculate_rewards(
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

    }

    let node_included_epochs = SubnetNodeConsecutiveIncludedEpochs::<Test>::get(subnet_id, hotkey_subnet_node_id);
    log::error!("node_included_epochs {:?}", node_included_epochs);
    
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
		let (rewards_data, rewards_weight) = Network::calculate_rewards(
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