use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use frame_support::BoundedVec;
use sp_core::OpaquePeerId as PeerId;
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

#[test]
fn test_do_epoch_preliminaries() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();

    for s in 0..max_subnets {
			let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
			build_activated_subnet(
				subnet_name.clone().into(), 
				0, 
				max_subnet_nodes, 
				deposit_amount, 
				amount
			);
		}

  });
}
