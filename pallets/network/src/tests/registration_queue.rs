use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use crate::{
    AccountSubnetStake, ActivationGraceEpochs, BootnodePeerIdSubnetNodeId, BootnodeSubnetNodeId,
    ClientPeerIdSubnetNodeId, ColdkeyReputation, DefaultMaxVectorLength, Error,
    HotkeyOwner, HotkeySubnetId, HotkeySubnetNodeId, LogicExpr, MaxDeactivationEpochs,
    MaxDelegateStakePercentage, MaxRegisteredNodes, MaxRewardRateDecrease, MaxSubnetNodes,
    MaxSubnets, MinSubnetNodes, MinSubnetRegistrationEpochs, NetworkMinStakeBalance,
    NodeDelegateStakeBalance, NodeRemovalConditionType, NodeRemovalPolicy, NodeRemovalSystemV2,
    NodeSlotIndex, PeerIdSubnetNodeId, SubnetNodeQueue,
    RegisteredSubnetNodesData, SubnetNodeQueueEpochs, Reputation, RewardRateUpdatePeriod,
    SubnetElectedValidator, SubnetMinStakeBalance, SubnetName, SubnetNode, SubnetNodeClass,
    SubnetNodeClassification, SubnetNodeElectionSlots, SubnetNodeIdHotkey, SubnetNodeUniqueParam,
    SubnetNodesData, SubnetOwner, SubnetRegistrationEpochs, SubnetState, SubnetsData,
    TotalActiveNodes, TotalActiveSubnetNodes, TotalActiveSubnets, TotalElectableNodes, TotalNodes,
    TotalStake, TotalSubnetElectableNodes, TotalSubnetNodeUids, TotalSubnetNodes, TotalSubnetStake,
    ChurnLimit
};
use frame_support::traits::Currency;
use frame_support::traits::ExistenceRequirement;
use frame_support::BoundedVec;
use frame_support::{assert_err, assert_ok};
use sp_core::OpaquePeerId as PeerId;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
///
///
///
///
///
///
///
/// Subnet Nodes Add/Remove
///
///
///
///
///
///
///

#[test]
fn test_register_subnet_node_v2() {
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

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id);

        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id,
            bootnode_peer_id,
            client_peer_id,
            None,
            0,
            amount,
            None,
            None,
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(
            subnet_node.classification.node_class,
            SubnetNodeClass::Registered
        );
        assert_eq!(
            subnet_node.classification.start_epoch,
            subnet_epoch + 1 // subnet_epoch + queue_epochs
        );

        let new_total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        assert_eq!(total_subnet_nodes + 1, new_total_nodes);

        let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
        let found = reg_queue
            .iter()
            .find(|node| node.id == hotkey_subnet_node_id);
        assert_eq!(found.unwrap().id, hotkey_subnet_node_id);

        assert_eq!(
            *network_events().last().unwrap(),
            Event::SubnetNodeRegistered {
                subnet_id: subnet_id,
                subnet_node_id: hotkey_subnet_node_id,
                coldkey: coldkey,
                hotkey: hotkey,
                data: subnet_node.clone(),
            }
        );
    })
}

#[test]
fn test_register_subnet_node_v2_and_activate() {
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

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id);

        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id,
            bootnode_peer_id,
            client_peer_id,
            None,
            0,
            amount,
            None,
            None,
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(
            subnet_node.classification.node_class,
            SubnetNodeClass::Registered
        );
        assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

        let new_total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        assert_eq!(total_subnet_nodes + 1, new_total_nodes);

        let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
        let found = reg_queue
            .iter()
            .find(|node| node.id == hotkey_subnet_node_id);
        assert_eq!(found.unwrap().id, hotkey_subnet_node_id);

        assert_eq!(
            *network_events().last().unwrap(),
            Event::SubnetNodeRegistered {
                subnet_id: subnet_id,
                subnet_node_id: hotkey_subnet_node_id,
                coldkey: coldkey,
                hotkey: hotkey,
                data: subnet_node.clone(),
            }
        );

        let start_epoch = subnet_node.classification.start_epoch;

        // increase to the nodes start epoch
        set_block_to_subnet_slot_epoch(start_epoch + queue_epochs + 1, subnet_id);

        let epoch = Network::get_current_epoch_as_u32();

        // Get subnet weights (nodes only activate from queue if there are weights)
        // Note: This means a subnet is active if it gets weights
        let _ = Network::handle_subnet_emission_weights(epoch);

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // Trigger the node activation
        Network::emission_step(System::block_number(), epoch, subnet_epoch, subnet_id);

        // Check out of queue
        assert_eq!(
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );
        let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
        let found = reg_queue
            .iter()
            .find(|node| node.id == hotkey_subnet_node_id);
        assert_eq!(found, None);

        // Check in activation
        let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
        assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);
    })
}


#[test]
fn test_register_subnet_node_v2_and_activate_max_churn_limit() {
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

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id);
        let churn_limit = ChurnLimit::<Test>::get(subnet_id);
        let prev_active_total_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);

        let reg_start = end + 1;
        let reg_end = reg_start + churn_limit * 2;

        // Put a bunch of nodes into the queue
        for n in reg_start..reg_end {
            let _n = n + 1;
            let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);

            let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            assert_ok!(Balances::transfer(
                &account(0), // alice
                &coldkey.clone(),
                amount + 500,
                ExistenceRequirement::KeepAlive,
            ));

            let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
            let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

            assert_ok!(Network::register_subnet_node(
                RuntimeOrigin::signed(coldkey.clone()),
                subnet_id,
                hotkey.clone(),
                peer_id,
                bootnode_peer_id,
                client_peer_id,
                None,
                0,
                amount,
                None,
                None,
            ));

            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
            assert_eq!(
                subnet_node.classification.node_class,
                SubnetNodeClass::Registered
            );
            assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

            assert_eq!(total_subnet_nodes + 1, TotalSubnetNodes::<Test>::get(subnet_id));

            let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
            let found = reg_queue
                .iter()
                .find(|node| node.id == hotkey_subnet_node_id);
            assert_eq!(found.unwrap().id, hotkey_subnet_node_id);
            System::set_block_number(System::block_number() + 1);
        }

        assert_eq!(SubnetNodeQueue::<Test>::get(subnet_id).len() as u32, reg_end - reg_start);

        let total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // increase to the nodes start epoch
        set_block_to_subnet_slot_epoch(subnet_epoch + queue_epochs + 2, subnet_id);

        let epoch = Network::get_current_epoch_as_u32();
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // Get subnet weights (nodes only activate from queue if there are weights)
        // Note: This means a subnet is active if it gets weights
        let _ = Network::handle_subnet_emission_weights(epoch);

        // Trigger the node activation
        Network::emission_step(System::block_number(), epoch, subnet_epoch, subnet_id);

        // Only activate up to the churn limit
        assert_eq!(prev_active_total_nodes + churn_limit, TotalActiveSubnetNodes::<Test>::get(subnet_id));

        assert_eq!(SubnetNodeQueue::<Test>::get(subnet_id).len() as u32, reg_end - reg_start - churn_limit);
        
        for n in reg_start..reg_end {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);

            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            // Ensure all nodes up to the churn limit were activated
            if _n <= reg_start + churn_limit {

                // Check in activation
                let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
                assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
                assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

                // Check out of queue
                assert_eq!(
                    RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
                    Err(())
                );
                let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
                let found = reg_queue
                    .iter()
                    .find(|node| node.id == hotkey_subnet_node_id);
                assert_eq!(found, None);
            } else {
                // Other nodes still in queue
                assert_eq!(
                    SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
                    Err(())
                );
                let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
                let found = reg_queue
                    .iter()
                    .find(|node| node.id == hotkey_subnet_node_id);
                assert_eq!(found.unwrap().id, hotkey_subnet_node_id);
            }
        }
    })
}

#[test]
fn test_register_subnet_node_v2_with_max_nodes() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let end = max_subnet_nodes;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id);
        let churn_limit = ChurnLimit::<Test>::get(subnet_id);
        let prev_active_total_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);

        let reg_start = end + 1;
        let reg_end = reg_start + churn_limit * 2;

        // Put a bunch of nodes into the queue
        for n in reg_start..reg_end {
            let _n = n + 1;
            let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);

            let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            assert_ok!(Balances::transfer(
                &account(0), // alice
                &coldkey.clone(),
                amount + 500,
                ExistenceRequirement::KeepAlive,
            ));

            let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
            let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

            assert_ok!(Network::register_subnet_node(
                RuntimeOrigin::signed(coldkey.clone()),
                subnet_id,
                hotkey.clone(),
                peer_id,
                bootnode_peer_id,
                client_peer_id,
                None,
                0,
                amount,
                None,
                None,
            ));

            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
            assert_eq!(
                subnet_node.classification.node_class,
                SubnetNodeClass::Registered
            );
            assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

            assert_eq!(total_subnet_nodes + 1, TotalSubnetNodes::<Test>::get(subnet_id));

            let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
            let found = reg_queue
                .iter()
                .find(|node| node.id == hotkey_subnet_node_id);
            assert_eq!(found.unwrap().id, hotkey_subnet_node_id);
            System::set_block_number(System::block_number() + 1);
        }

        assert_eq!(SubnetNodeQueue::<Test>::get(subnet_id).len() as u32, reg_end - reg_start);

        let total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // increase to the nodes start epoch
        set_block_to_subnet_slot_epoch(subnet_epoch + queue_epochs + 2, subnet_id);

        let epoch = Network::get_current_epoch_as_u32();
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // Get subnet weights (nodes only activate from queue if there are weights)
        // Note: This means a subnet is active if it gets weights
        let _ = Network::handle_subnet_emission_weights(epoch);

        // Trigger the node activation
        Network::emission_step(System::block_number(), epoch, subnet_epoch, subnet_id);

        // Only activate up to the churn limit
        assert_eq!(max_subnet_nodes, TotalActiveSubnetNodes::<Test>::get(subnet_id));
        assert_eq!(max_subnet_nodes + reg_end - reg_start, TotalSubnetNodes::<Test>::get(subnet_id));

        assert_eq!(SubnetNodeQueue::<Test>::get(subnet_id).len() as u32, reg_end - reg_start);
        
        // no nodes should be activated
        for n in reg_start..reg_end {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);

            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
            
            // Other nodes still in queue
            assert_eq!(
                SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
                Err(())
            );
            let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
            let found = reg_queue
                .iter()
                .find(|node| node.id == hotkey_subnet_node_id);
            assert_eq!(found.unwrap().id, hotkey_subnet_node_id);
        }
    })
}

#[test]
fn test_register_subnet_node_v2_activate_up_to_max_nodes() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();
        let expected_activated = 2;
        let end = max_subnet_nodes - expected_activated;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id);
        let churn_limit = ChurnLimit::<Test>::get(subnet_id);
        let prev_active_total_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);

        let reg_start = end + 1;
        let reg_end = reg_start + churn_limit * 2;

        // Put a bunch of nodes into the queue
        for n in reg_start..reg_end {
            let _n = n + 1;
            let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);

            let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            assert_ok!(Balances::transfer(
                &account(0), // alice
                &coldkey.clone(),
                amount + 500,
                ExistenceRequirement::KeepAlive,
            ));

            let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
            let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

            assert_ok!(Network::register_subnet_node(
                RuntimeOrigin::signed(coldkey.clone()),
                subnet_id,
                hotkey.clone(),
                peer_id,
                bootnode_peer_id,
                client_peer_id,
                None,
                0,
                amount,
                None,
                None,
            ));

            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
            assert_eq!(
                subnet_node.classification.node_class,
                SubnetNodeClass::Registered
            );
            assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

            assert_eq!(total_subnet_nodes + 1, TotalSubnetNodes::<Test>::get(subnet_id));

            let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
            let found = reg_queue
                .iter()
                .find(|node| node.id == hotkey_subnet_node_id);
            assert_eq!(found.unwrap().id, hotkey_subnet_node_id);
        }

        assert_eq!(SubnetNodeQueue::<Test>::get(subnet_id).len() as u32, reg_end - reg_start);
        assert_eq!(end, TotalActiveSubnetNodes::<Test>::get(subnet_id));
        assert_eq!(end + reg_end - reg_start, TotalSubnetNodes::<Test>::get(subnet_id));

        let total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // increase to the nodes start epoch
        set_block_to_subnet_slot_epoch(subnet_epoch + queue_epochs + 2, subnet_id);

        let epoch = Network::get_current_epoch_as_u32();
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // Get subnet weights (nodes only activate from queue if there are weights)
        // Note: This means a subnet is active if it gets weights
        let _ = Network::handle_subnet_emission_weights(epoch);

        // Trigger the node activation
        Network::emission_step(System::block_number(), epoch, subnet_epoch, subnet_id);

        // Only activate up to the churn limit
        assert_eq!(max_subnet_nodes, TotalActiveSubnetNodes::<Test>::get(subnet_id));
        assert_eq!(end + reg_end - reg_start, TotalSubnetNodes::<Test>::get(subnet_id));

        assert_eq!(SubnetNodeQueue::<Test>::get(subnet_id).len() as u32, reg_end - reg_start - expected_activated);

        // no nodes should be activated
        for n in reg_start..reg_end {
            let _n = n + 1;
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);

            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            // Ensure all nodes up to the churn limit were activated
            if _n <= reg_start + expected_activated {

                // Check in activation
                let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
                assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
                assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

                // Check out of queue
                assert_eq!(
                    RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
                    Err(())
                );
                let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
                let found = reg_queue
                    .iter()
                    .find(|node| node.id == hotkey_subnet_node_id);
                assert_eq!(found, None);
            } else {
                // Other nodes still in queue
                assert_eq!(
                    SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
                    Err(())
                );
                let reg_queue = SubnetNodeQueue::<Test>::get(subnet_id);
                let found = reg_queue
                    .iter()
                    .find(|node| node.id == hotkey_subnet_node_id);
                assert_eq!(found.unwrap().id, hotkey_subnet_node_id);
            }
        }
    })
}
