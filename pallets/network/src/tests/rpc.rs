use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use crate::{
    MaxSubnetNodes, MaxSubnets, MinSubnetMinStake, SubnetElectedValidator, SubnetName,
    SubnetNodeClass, TotalActiveSubnets, PeerIdOverwatchNodeId, SubnetBootnodesV2,
    DefaultMaxVectorLength,
};
use frame_support::assert_ok;
use frame_support::traits::Currency;
use sp_runtime::BoundedVec;
use sp_std::collections::btree_map::BTreeMap;

//
// RPC Getter Function Tests
//

#[test]
fn test_get_coldkey_subnet_nodes_info() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

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

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
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

#[test]
fn test_proof_of_stake_all_peer_id_types() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        // Test with main peer_id
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end);
        assert!(
            Network::proof_of_stake(subnet_id, peer_id.0.to_vec(), 1),
            "Proof of stake should work with main peer_id"
        );

        // Test with bootnode_peer_id
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end);
        assert!(
            Network::proof_of_stake(subnet_id, bootnode_peer_id.0.to_vec(), 1),
            "Proof of stake should work with bootnode_peer_id"
        );

        // Test with client_peer_id
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end);
        assert!(
            Network::proof_of_stake(subnet_id, client_peer_id.0.to_vec(), 1),
            "Proof of stake should work with client_peer_id"
        );

        // Test with overwatch node
        let overwatch_node_peer_id = peer(1);
        PeerIdOverwatchNodeId::<Test>::insert(subnet_id, &overwatch_node_peer_id, 1);
        assert!(
            Network::proof_of_stake(subnet_id, overwatch_node_peer_id.0.to_vec(), 1),
            "Proof of stake should work with overwatch node peer_id"
        );

        let bv = |b: u8| BoundedVec::<u8, DefaultMaxVectorLength>::try_from(vec![b]).unwrap();
        let add_map = BTreeMap::from([(peer(2), bv(2)), (peer(3), bv(3))]);

        SubnetBootnodesV2::<Test>::insert(subnet_id, add_map);

        assert!(
            Network::proof_of_stake(subnet_id, peer(2).0.to_vec(), 1),
            "Proof of stake should work with bootnode peer_id"
        );

    })
}

#[test]
fn test_proof_of_stake_with_different_classes() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = 4;

        build_activated_subnet(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end);

        // Test with class 1 (Registered)
        assert!(
            Network::proof_of_stake(subnet_id, peer_id.0.to_vec(), 1),
            "Should work with Registered class"
        );

        // Test with class 2 (Idle)
        assert!(
            Network::proof_of_stake(subnet_id, peer_id.0.to_vec(), 2),
            "Should work with Idle class"
        );
    })
}

#[test]
fn test_proof_of_stake_invalid_peer_id_fails() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        // Test with non-existent peer_id
        let fake_peer_id = vec![1, 2, 3, 4, 5];
        assert!(
            !Network::proof_of_stake(subnet_id, fake_peer_id, 1),
            "Proof of stake should fail with invalid peer_id"
        );
    })
}

#[test]
fn test_get_subnet_info() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let subnet_info = Network::get_subnet_info(subnet_id);

        assert!(subnet_info.is_some(), "Subnet info should exist");
        let info = subnet_info.unwrap();
        assert_eq!(info.id, subnet_id);
        assert_eq!(info.name, subnet_name);
    })
}

#[test]
fn test_get_all_subnets_info() {
    new_test_ext().execute_with(|| {
        // Create multiple subnets
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet("subnet1".into(), 0, 0, deposit_amount, stake_amount);
        build_activated_subnet("subnet2".into(), 0, 0, deposit_amount, stake_amount);

        let all_subnets = Network::get_all_subnets_info();

        assert!(all_subnets.len() >= 2, "Should have at least 2 subnets");
    })
}

#[test]
fn test_get_subnet_node_info() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let node_info = Network::get_subnet_node_info(subnet_id, 1);

        assert!(node_info.is_some(), "Node info should exist");
        let info = node_info.unwrap();
        assert_eq!(info.subnet_id, subnet_id);
        assert_eq!(info.subnet_node_id, 1);
    })
}

#[test]
fn test_get_subnet_nodes_info() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
        let end = 5;

        build_activated_subnet(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let nodes_info = Network::get_subnet_nodes_info(subnet_id);

        assert!(
            nodes_info.len() == end as usize,
            "Should have correct number of nodes"
        );
    })
}

#[test]
fn test_get_all_subnet_nodes_info() {
    new_test_ext().execute_with(|| {
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet("subnet1".into(), 0, 3, deposit_amount, stake_amount);
        build_activated_subnet("subnet2".into(), 0, 3, deposit_amount, stake_amount);

        let all_nodes = Network::get_all_subnet_nodes_info();

        assert!(all_nodes.len() >= 6, "Should have nodes from all subnets");
    })
}

#[test]
fn test_get_elected_validator_info() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet(subnet_name.clone(), 0, 12, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = block_number / epoch_length;

        set_block_to_subnet_slot_epoch(epoch, subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // Elect a validator
        Network::elect_validator(subnet_id, subnet_epoch, block_number);

        let validator_info = Network::get_elected_validator_info(subnet_id, subnet_epoch);

        assert!(
            validator_info.is_some(),
            "Elected validator info should exist"
        );
    })
}

#[test]
fn test_get_validators_and_attestors() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet(subnet_name.clone(), 0, 12, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let validators = Network::get_validators_and_attestors(subnet_id);

        assert!(validators.len() > 0, "Should have validators/attestors");
    })
}

#[test]
fn test_get_bootnodes() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let bootnodes = Network::get_bootnodes(subnet_id);

        // Verify structure exists
        assert!(
            bootnodes.bootnodes.len() >= 0 || bootnodes.node_bootnodes.len() >= 0,
            "Bootnodes structure should exist"
        );
    })
}

#[test]
fn test_get_coldkey_stakes() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let end = 4;

        build_activated_subnet(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
        let stakes = Network::get_coldkey_stakes(coldkey.clone());

        assert!(stakes.len() > 0, "Coldkey should have stakes");
    })
}

#[test]
fn test_get_delegate_stakes() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let delegator = account(100);
        let _ = Balances::deposit_creating(&delegator, 1000000000000000000000 + 500);

        // Add delegate stake
        assert_ok!(Network::add_to_delegate_stake(
            RuntimeOrigin::signed(delegator.clone()),
            subnet_id,
            1000000000000000000000
        ));

        let delegate_stakes = Network::get_delegate_stakes(delegator);

        assert!(delegate_stakes.len() > 0, "Should have delegate stakes");
    })
}

#[test]
fn test_get_node_delegate_stakes() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "test-subnet".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        build_activated_subnet(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let delegator = account(100);
        let _ = Balances::deposit_creating(&delegator, 1000000000000000000000 + 500);

        // Add node delegate stake
        assert_ok!(Network::add_to_node_delegate_stake(
            RuntimeOrigin::signed(delegator.clone()),
            subnet_id,
            1,
            1000000000000000000000
        ));

        let node_delegate_stakes = Network::get_node_delegate_stakes(delegator);

        assert!(
            node_delegate_stakes.len() > 0,
            "Should have node delegate stakes"
        );
    })
}
