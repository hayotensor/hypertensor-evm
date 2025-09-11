use super::mock::*;
use crate::tests::test_utils::*;
use frame_support::{
	assert_ok, assert_err
};
use frame_support::traits::Currency;
use crate::{
  Error,
  SubnetName,
  HotkeySubnetNodeId,
  SubnetElectedValidator,
  SubnetNodeIdHotkey,
  SubnetConsensusSubmission,
  SubnetNodesData,
  SubnetNodeClass,
};

// Overwatch node functions in the slot.rs file are in tests/overwatch_nodes.rs


#[test]
fn test_calculate_emission_weights() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    for s in 0..max_subnets {
			let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
			build_activated_subnet_new(
				subnet_name.clone().into(), 
				0, 
				max_subnet_nodes, 
				deposit_amount, 
				amount
			);
		}
  });
}

#[test]
fn test_emission_step() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    for s in 0..max_subnets {
			let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
			build_activated_subnet_new(
				subnet_name.clone().into(), 
				0, 
				max_subnet_nodes, 
				deposit_amount, 
				amount
			);
		}
  });
}

#[test]
fn test_handle_subnet_emission_weights() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    for s in 0..max_subnets {
			let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
			build_activated_subnet_new(
				subnet_name.clone().into(), 
				0, 
				max_subnet_nodes, 
				deposit_amount, 
				amount
			);
		}
  });
}

#[test]
fn test_calculate_subnet_weights() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    for s in 0..max_subnets {
			let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
			build_activated_subnet_new(
				subnet_name.clone().into(), 
				0, 
				max_subnet_nodes, 
				deposit_amount, 
				amount
			);
		}
  });
}

#[test]
fn test_calculate_subnet_weights_v2() {
  new_test_ext().execute_with(|| {
    let max_subnets = MaxSubnets::<Test>::get();
		let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
		let total_ows = 12;

		// - insert overwatchers
		for n in 0..total_ows {
			let _n = n + 1;
			let coldkey_n = _n;
			let hotkey_n = total_ows + _n;
			insert_overwatch_node(coldkey_n, hotkey_n);
			set_overwatch_stake(hotkey_n, 100);
		}

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    for s in 0..max_subnets {
			let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
			build_activated_subnet_new(
				subnet_name.clone().into(), 
				0, 
				max_subnet_nodes, 
				deposit_amount, 
				amount
			);
		}


    for s in 0..max_subnets {
			let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
			let subnet_id = SubnetName::<Test>::(subnet_name.clone().into()).unwrap(); 
			for n in 0..total_ows {
        let _n = n + 1;
        let coldkey_n = _n;
        let hotkey_n = total_ows + _n;
				let node_id = HotkeyOverwatchNodeId::<Test>::get(account(hotkey_n)).unwrap();
				submit_weight(epoch, subnet_id, node_id, 1000000000000000000);
			}
		}

    // let (stake_weights_normalized, stake_weights_weight) = Network::<Test>::calculate_subnet_weights_v2(epoch);
    // assert!(stake_weights_normalized.len() as u32 == max_subnets);

  });
}

#[test]
fn test_precheck_consensus_submission() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    for s in 0..max_subnets {
			let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
			build_activated_subnet_new(
				subnet_name.clone().into(), 
				0, 
				max_subnet_nodes, 
				deposit_amount, 
				amount
			);
		}
  });
}

#[test]
fn test_calculate_rewards() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    for s in 0..max_subnets {
			let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
			build_activated_subnet_new(
				subnet_name.clone().into(), 
				0, 
				max_subnet_nodes, 
				deposit_amount, 
				amount
			);
		}
  });
}
