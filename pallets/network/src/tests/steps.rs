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
