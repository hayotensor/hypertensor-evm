use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use crate::{MaxSubnetNodes, MaxSubnets, NetworkMinStakeBalance, SubnetName, TotalActiveSubnets};

#[test]
fn test_get_coldkey_subnet_nodes_info() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

        let rpc_results = Network::get_coldkey_subnet_nodes_info(coldkey.clone());

        assert!(rpc_results.len() > 0);
    })
}

#[test]
fn test_proof_of_stake() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end);
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end);

        let rpc_results = Network::proof_of_stake(subnet_id, peer_id.0.to_vec(), 1);

        assert!(rpc_results);
    })
}
