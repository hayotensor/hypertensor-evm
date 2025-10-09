// use super::mock::*;
// use crate::tests::test_utils::*;
// use crate::Event;
// use crate::{
//     AccountSubnetStake, ActivationGraceEpochs, BootnodePeerIdSubnetNodeId, BootnodeSubnetNodeId,
//     ClientPeerIdSubnetNodeId, ColdkeyReputation, ColdkeySubnetNodes, DefaultMaxVectorLength, Error,
//     HotkeyOwner, HotkeySubnetId, HotkeySubnetNodeId,
//     MaxDelegateStakePercentage, MaxRegisteredNodes, MaxRewardRateDecrease, MaxSubnetNodes,
//     MaxSubnets, MinSubnetNodes, MinSubnetRegistrationEpochs, MinSubnetMinStake,
//     NodeDelegateStakeBalance, NodeRemovalSystemV2,
//     NodeSlotIndex, PeerIdSubnetNodeId, RegisteredSubnetNodesData,
//     SubnetNodeQueueEpochs, Reputation, NodeRewardRateUpdatePeriod, SubnetElectedValidator,
//     SubnetMinStakeBalance, SubnetName, SubnetNode, SubnetNodeClass, SubnetNodeClassification,
//     SubnetNodeElectionSlots, SubnetNodeIdHotkey, SubnetNodeUniqueParam, SubnetNodesData,
//     SubnetOwner, SubnetRegistrationEpochs, SubnetState, SubnetsData, TotalActiveNodes,
//     TotalActiveSubnetNodes, TotalActiveSubnets, TotalElectableNodes, TotalNodes, TotalStake,
//     TotalSubnetElectableNodes, TotalSubnetNodeUids, TotalSubnetNodes, TotalSubnetStake,
// };
// use frame_support::traits::Currency;
// use frame_support::traits::ExistenceRequirement;
// use frame_support::BoundedVec;
// use frame_support::{assert_err, assert_ok};
// use sp_core::OpaquePeerId as PeerId;
// use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
// ///
// ///
// ///
// ///
// ///
// ///
// ///
// /// Subnet Nodes Add/Remove
// ///
// ///
// ///
// ///
// ///
// ///
// ///

// #[test]
// fn test_activate_subnet_then_register_subnet_node_then_activate() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id,
//             bootnode_peer_id,
//             client_peer_id,
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);

//         assert_eq!(
//             *network_events().last().unwrap(),
//             Event::SubnetNodeRegistered {
//                 subnet_id: subnet_id,
//                 subnet_node_id: hotkey_subnet_node_id,
//                 coldkey: coldkey,
//                 hotkey: hotkey,
//                 data: subnet_node.clone(),
//             }
//         );

//         let start_epoch = subnet_node.classification.start_epoch;

//         set_block_to_subnet_slot_epoch(start_epoch, subnet_id);

//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         assert_ok!(Network::activate_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id
//         ));

//         assert_eq!(
//             *network_events().last().unwrap(),
//             Event::SubnetNodeActivated {
//                 subnet_id: subnet_id,
//                 subnet_node_id: hotkey_subnet_node_id,
//             }
//         );

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
//         assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

//         let new_total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         assert_eq!(total_subnet_nodes + 1, new_total_nodes);
//     })
// }

// #[test]
// fn test_add_subnet_node_subnet_err() {
//     new_test_ext().execute_with(|| {
//         let subnet_id = 0;

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 0;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let amount: u128 = 1000;
//         assert_err!(
//             Network::add_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::InvalidSubnetId
//         );

//         let subnet_id = 1;

//         assert_err!(
//             Network::add_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::InvalidSubnetId
//         );
//     })
// }

// #[test]
// fn test_add_subnet_node_not_exists_err() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         // add same peer_id under new account error
//         assert_err!(
//             Network::add_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::PeerIdExist
//         );

//         // new peer id
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         // existing bootnode peer id
//         let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end);

//         // add same peer_id under new account error
//         assert_err!(
//             Network::add_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::BootnodePeerIdExist
//         );

//         // new bootnode peer id
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end);

//         // add same peer_id under new account error
//         assert_err!(
//             Network::add_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::ClientPeerIdExist
//         );
//     })
// }

// #[test]
// fn test_register_subnet_node_match_coldkey_hotkey_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
//         let starting_balance = Balances::free_balance(&coldkey.clone());

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(account(1)),
//                 subnet_id,
//                 account(1),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id,
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::ColdkeyMatchesHotkey
//         );
//     })
// }

// #[test]
// fn test_register_subnet_subnet_is_paused_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let original_owner = account(1);

//         // Set initial owner
//         SubnetOwner::<Test>::insert(subnet_id, &original_owner);
//         let epoch = Network::get_current_epoch_as_u32();

//         // Transfer to new owner
//         assert_ok!(Network::owner_pause_subnet(
//             RuntimeOrigin::signed(original_owner.clone()),
//             subnet_id,
//         ));

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
//         let starting_balance = Balances::free_balance(&coldkey.clone());

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id,
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::SubnetIsPaused
//         );
//     });
// }

// #[test]
// fn test_register_subnet_subnet_must_be_registering_or_active() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_registered_subnet_new(
//             subnet_name.clone(),
//             0,
//             4,
//             deposit_amount,
//             stake_amount,
//             true,
// None,
//         );
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         // --- increase to enactment period
//         let epochs = SubnetRegistrationEpochs::<Test>::get();
//         increase_epochs(epochs + 1);

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
//         let starting_balance = Balances::free_balance(&coldkey.clone());

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id,
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::SubnetMustBeRegisteringOrActivated
//         );
//     });
// }

// #[test]
// fn test_register_subnet_coldkey_registration_whitelist_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_registered_subnet_new(
//             subnet_name.clone(),
//             0,
//             4,
//             deposit_amount,
//             stake_amount,
//             true,
// None,
//         );
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
//         let starting_balance = Balances::free_balance(&coldkey.clone());

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id,
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::ColdkeyRegistrationWhitelist
//         );
//     });
// }

// #[test]
// fn test_register_subnet_max_registered_nodes_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let max_registered_nodes = 4;
//         MaxRegisteredNodes::<Test>::insert(subnet_id, max_registered_nodes);

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
//         let starting_balance = Balances::free_balance(&coldkey.clone());

//         log::error!("max_registered_nodes {:?}", max_registered_nodes);

//         let mut touched = false;
//         for n in end..max_registered_nodes + end + 2 {
//             let _n = n + 1;
//             log::error!("_n {:?}", _n);
//             let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
//             let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
//             let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
//             let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
//             let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);

//             assert_ok!(Balances::transfer(
//                 &account(0), // alice
//                 &coldkey.clone(),
//                 amount + 500,
//                 ExistenceRequirement::KeepAlive,
//             ));
//             if _n - end > max_registered_nodes + 1 {
//                 touched = true;
//                 assert_err!(
//                     Network::register_subnet_node(
//                         RuntimeOrigin::signed(coldkey.clone()),
//                         subnet_id,
//                         hotkey.clone(),
//                         peer_id.clone(),
//                         bootnode_peer_id.clone(),
//                         client_peer_id.clone(),
//                         None,
//                         0,
//                         amount,
//                         None,
//                         None,
//                     ),
//                     Error::<Test>::MaxRegisteredNodes
//                 );
//             } else {
//                 assert_ok!(Network::register_subnet_node(
//                     RuntimeOrigin::signed(coldkey.clone()),
//                     subnet_id,
//                     hotkey.clone(),
//                     peer_id.clone(),
//                     bootnode_peer_id.clone(),
//                     client_peer_id.clone(),
//                     None,
//                     0,
//                     amount,
//                     None,
//                     None,
//                 ));
//             }
//         }
//         assert!(touched);
//     });
// }

// #[test]
// fn test_register_subnet_node_and_then_update_a_param() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         assert_ok!(Balances::transfer(
//             &account(0), // alice
//             &coldkey.clone(),
//             amount + 500,
//             ExistenceRequirement::KeepAlive,
//         ));

//         let unique: Vec<u8> = "a".into();
//         let bounded_unique: BoundedVec<u8, DefaultMaxVectorLength> =
//             unique.try_into().expect("String too long");

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             Some(bounded_unique.clone()),
//             None,
//         ));

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.unique, Some(bounded_unique.clone()));

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 2);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);

//         assert_ok!(Balances::transfer(
//             &account(0), // alice
//             &coldkey.clone(),
//             amount + 500,
//             ExistenceRequirement::KeepAlive,
//         ));

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 Some(bounded_unique.clone()),
//                 None,
//             ),
//             Error::<Test>::SubnetNodeUniqueParamTaken
//         );
//     })
// }

// #[test]
// fn test_register_subnet_node_post_subnet_activation() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
//         let starting_balance = Balances::free_balance(&coldkey.clone());

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let post_balance = Balances::free_balance(&coldkey.clone());
//         assert_eq!(post_balance, starting_balance - amount);

//         let total_subnet_node_uids = TotalSubnetNodeUids::<Test>::get(subnet_id);
//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
//         assert_eq!(total_subnet_node_uids, hotkey_subnet_node_id);

//         let subnet_node_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         assert_eq!(subnet_node_hotkey, Some(hotkey.clone()));

//         let coldkey = HotkeyOwner::<Test>::get(hotkey.clone());
//         assert_eq!(coldkey, coldkey.clone());

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         assert_eq!(subnet_node.hotkey, hotkey.clone());
//         assert_eq!(subnet_node.peer_id, peer_id.clone());
//         assert_eq!(
//             subnet_node.classification.node_class,
//             SubnetNodeClass::Registered
//         );

//         let peer_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
//         assert_eq!(peer_account, hotkey_subnet_node_id);

//         let bootnode_peer_account =
//             BootnodePeerIdSubnetNodeId::<Test>::get(subnet_id, bootnode_peer_id.clone());
//         assert_eq!(bootnode_peer_account, hotkey_subnet_node_id);

//         let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
//         assert_eq!(account_subnet_stake, amount);

//         System::assert_last_event(RuntimeEvent::Network(crate::Event::SubnetNodeRegistered {
//             subnet_id,
//             subnet_node_id: hotkey_subnet_node_id,
//             coldkey: coldkey.clone(),
//             hotkey: hotkey.clone(),
//             data: subnet_node.clone(),
//         }));
//     })
// }

// #[test]
// fn test_activate_subnet_node_post_subnet_activation() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id,
//             bootnode_peer_id,
//             client_peer_id,
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let new_total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         assert_eq!(total_subnet_nodes + 1, new_total_nodes);

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let start_epoch = subnet_node.classification.start_epoch;

//         set_block_to_subnet_slot_epoch(start_epoch, subnet_id);

//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         let prev_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
//         let prev_total_active_nodes = TotalActiveNodes::<Test>::get();
//         let prev_coldkey_reputation = ColdkeyReputation::<Test>::get(coldkey.clone());

//         assert_ok!(Network::activate_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id
//         ));

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Idle);
//         assert_eq!(subnet_node.classification.start_epoch, subnet_epoch + 1);

//         assert_eq!(
//             prev_total_active_subnet_nodes + 1,
//             TotalActiveSubnetNodes::<Test>::get(subnet_id)
//         );
//         assert_eq!(prev_total_active_nodes + 1, TotalActiveNodes::<Test>::get());
//         assert_eq!(
//             prev_coldkey_reputation.lifetime_node_count + 1,
//             ColdkeyReputation::<Test>::get(coldkey.clone()).lifetime_node_count
//         );
//         assert_eq!(
//             prev_coldkey_reputation.total_active_nodes + 1,
//             ColdkeyReputation::<Test>::get(coldkey.clone()).total_active_nodes
//         );
//     })
// }

// #[test]
// fn test_register_after_activate_with_same_keys() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let total_subnet_node_uids = TotalSubnetNodeUids::<Test>::get(subnet_id);
//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let start_epoch = subnet_node.classification.start_epoch;

//         // set_epoch(start_epoch, 0);
//         set_block_to_subnet_slot_epoch(start_epoch, subnet_id);

//         assert_ok!(Network::activate_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id
//         ));

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let new_total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::HotkeyHasOwner
//         );
//     })
// }

// #[test]
// fn test_register_after_deactivate_with_same_keys() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_ok!(Network::pause_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             end
//         ));

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer(1),
//                 peer(1),
//                 peer(1),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::HotkeyHasOwner
//         );
//     })
// }

// #[test]
// fn test_activate_subnet_node_not_key_owner_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 12;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id,
//             client_peer_id,
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let start_epoch = subnet_node.classification.start_epoch;

//         // --- Try starting before start_epoch
//         assert_err!(
//             Network::activate_subnet_node(
//                 RuntimeOrigin::signed(account(1)),
//                 subnet_id,
//                 hotkey_subnet_node_id
//             ),
//             Error::<Test>::NotKeyOwner
//         );
//     })
// }

// #[test]
// fn test_activate_subnet_node_not_uid_owner_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 12;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id,
//             client_peer_id,
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let start_epoch = subnet_node.classification.start_epoch;

//         // --- Try starting before start_epoch
//         assert_err!(
//             Network::activate_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey_subnet_node_id + 99
//             ),
//             Error::<Test>::NotUidOwner
//         );
//     })
// }

// #[test]
// fn test_activate_subnet_node_not_registered_uid_owner_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 12;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end);
//         let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end);

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         // --- Try starting before start_epoch
//         assert_err!(
//             Network::activate_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey_subnet_node_id
//             ),
//             Error::<Test>::NotRegisteredSubnetNode
//         );
//     })
// }

// #[test]
// fn test_activate_subnet_node_min_stake_not_reached_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 12;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id,
//             client_peer_id,
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let start_epoch = subnet_node.classification.start_epoch;

//         SubnetMinStakeBalance::<Test>::insert(
//             subnet_id,
//             SubnetMinStakeBalance::<Test>::get(subnet_id) * 100,
//         );

//         let min_stake_balance = SubnetMinStakeBalance::<Test>::get(subnet_id);
//         let stake_balance = AccountSubnetStake::<Test>::get(&hotkey, subnet_id);

//         assert!(stake_balance < min_stake_balance);

//         // --- Try starting before start_epoch
//         assert_err!(
//             Network::activate_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey_subnet_node_id
//             ),
//             Error::<Test>::MinStakeNotReached
//         );
//     })
// }

// #[test]
// fn test_activate_subnet_node_not_start_epoch() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 12;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id,
//             client_peer_id,
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let start_epoch = subnet_node.classification.start_epoch;

//         // --- Try starting before start_epoch
//         assert_err!(
//             Network::activate_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey_subnet_node_id
//             ),
//             Error::<Test>::NotStartEpoch
//         );

//         let grace_epochs = ActivationGraceEpochs::<Test>::get(subnet_id);
//         set_epoch(start_epoch + grace_epochs + 2, 0);

//         // --- Try starting after ActivationGraceEpochs
//         assert_err!(
//             Network::activate_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey_subnet_node_id
//             ),
//             Error::<Test>::NotStartEpoch
//         );
//     })
// }

// #[test]
// fn test_remove_subnet_node_registered() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 12;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         let unique: Vec<u8> = "a".into();
//         let bounded_unique: BoundedVec<u8, DefaultMaxVectorLength> =
//             unique.try_into().expect("String too long");

//         let non_unique: Vec<u8> = "a".into();
//         let bounded_non_unique: BoundedVec<u8, DefaultMaxVectorLength> =
//             non_unique.try_into().expect("String too long");

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             Some(bounded_unique.clone()),
//             Some(bounded_non_unique.clone()),
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
//         assert!(coldkey_subnet_nodes
//             .get(&subnet_id)
//             .unwrap()
//             .contains(&hotkey_subnet_node_id));

//         let prev_total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let prev_total_nodes = TotalNodes::<Test>::get();

//         let prev_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
//         let prev_total_active_nodes = TotalActiveNodes::<Test>::get();

//         let prev_slot_list_len = SubnetNodeElectionSlots::<Test>::get(subnet_id).len();

//         assert_ok!(Network::remove_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id,
//         ));

//         assert_eq!(
//             *network_events().last().unwrap(),
//             Event::SubnetNodeRemoved {
//                 subnet_id: subnet_id,
//                 subnet_node_id: hotkey_subnet_node_id,
//             }
//         );

//         assert_eq!(
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id).count(),
//             0
//         );

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, hotkey.clone());
//         assert_eq!(subnet_node_id, Err(()));

//         let peer_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer_id.clone());
//         assert_eq!(peer_account, Err(()));

//         let bootnode_peer_account =
//             BootnodePeerIdSubnetNodeId::<Test>::try_get(subnet_id, bootnode_peer_id.clone());
//         assert_eq!(bootnode_peer_account, Err(()));

//         let subnet_node_hotkey =
//             SubnetNodeIdHotkey::<Test>::try_get(subnet_id, hotkey_subnet_node_id);
//         assert_eq!(subnet_node_hotkey, Err(()));

//         let subnet_node_hotkey = HotkeySubnetId::<Test>::try_get(hotkey.clone());
//         assert_eq!(subnet_node_hotkey, Err(()));

//         let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone()); // This is tested, see `test_clean_coldkey_subnet_nodes`
//         assert_eq!(coldkey_subnet_nodes.get(&subnet_id), None);

//         assert_eq!(
//             prev_total_subnet_nodes - 1,
//             TotalSubnetNodes::<Test>::get(subnet_id)
//         );
//         assert_eq!(prev_total_nodes - 1, TotalNodes::<Test>::get());

//         // Not active node, this shouldn't change
//         assert_eq!(
//             prev_total_active_subnet_nodes,
//             TotalActiveSubnetNodes::<Test>::get(subnet_id)
//         );
//         assert_eq!(prev_total_active_nodes, TotalActiveNodes::<Test>::get());
//         assert_eq!(
//             prev_slot_list_len,
//             SubnetNodeElectionSlots::<Test>::get(subnet_id).len()
//         );

//         //
//         //
//         //
//         // Test another node and force it into Idle
//         //
//         //
//         //

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 2);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             Some(bounded_unique.clone()),
//             Some(bounded_non_unique.clone()),
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let initial_start_epoch = subnet_node.classification.start_epoch;

//         set_block_to_subnet_slot_epoch(initial_start_epoch, subnet_id);

//         assert_ok!(Network::activate_subnet_node(
//             RuntimeOrigin::signed(hotkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id
//         ));

//         let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
//         assert!(coldkey_subnet_nodes
//             .get(&subnet_id)
//             .unwrap()
//             .contains(&hotkey_subnet_node_id));

//         let prev_total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let prev_total_nodes = TotalNodes::<Test>::get();

//         let prev_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
//         let prev_total_active_nodes = TotalActiveNodes::<Test>::get();

//         let prev_slot_list_len = SubnetNodeElectionSlots::<Test>::get(subnet_id).len();

//         assert_ok!(Network::remove_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id,
//         ));

//         assert_eq!(
//             SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
//             Err(())
//         );

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, hotkey.clone());
//         assert_eq!(subnet_node_id, Err(()));

//         let peer_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer_id.clone());
//         assert_eq!(peer_account, Err(()));

//         let bootnode_peer_account =
//             BootnodePeerIdSubnetNodeId::<Test>::try_get(subnet_id, bootnode_peer_id.clone());
//         assert_eq!(bootnode_peer_account, Err(()));

//         let subnet_node_hotkey =
//             SubnetNodeIdHotkey::<Test>::try_get(subnet_id, hotkey_subnet_node_id);
//         assert_eq!(subnet_node_hotkey, Err(()));

//         let subnet_node_hotkey = HotkeySubnetId::<Test>::try_get(hotkey.clone());
//         assert_eq!(subnet_node_hotkey, Err(()));

//         let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone()); // This is tested, see `test_clean_coldkey_subnet_nodes`
//         assert_eq!(coldkey_subnet_nodes.get(&subnet_id), None);

//         assert_eq!(
//             prev_total_subnet_nodes - 1,
//             TotalSubnetNodes::<Test>::get(subnet_id)
//         );
//         assert_eq!(prev_total_nodes - 1, TotalNodes::<Test>::get());

//         assert_eq!(
//             prev_total_active_subnet_nodes - 1,
//             TotalActiveSubnetNodes::<Test>::get(subnet_id)
//         );
//         assert_eq!(prev_total_active_nodes - 1, TotalActiveNodes::<Test>::get());

//         assert_eq!(
//             prev_slot_list_len,
//             SubnetNodeElectionSlots::<Test>::get(subnet_id).len()
//         );

//         //
//         //
//         //
//         // Test another node and force it into Included
//         //
//         //
//         //

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 3);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 3);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 3);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 3);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 3);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             Some(bounded_unique.clone()),
//             Some(bounded_non_unique.clone()),
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let initial_start_epoch = subnet_node.classification.start_epoch;

//         set_block_to_subnet_slot_epoch(initial_start_epoch, subnet_id);

//         assert_ok!(Network::activate_subnet_node(
//             RuntimeOrigin::signed(hotkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id
//         ));

//         let mut subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         subnet_node.classification.start_epoch = 0;
//         subnet_node.classification.node_class = SubnetNodeClass::Included;

//         SubnetNodesData::<Test>::insert(subnet_id, hotkey_subnet_node_id, subnet_node);

//         let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
//         assert!(coldkey_subnet_nodes
//             .get(&subnet_id)
//             .unwrap()
//             .contains(&hotkey_subnet_node_id));

//         let prev_total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let prev_total_nodes = TotalNodes::<Test>::get();

//         let prev_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
//         let prev_total_active_nodes = TotalActiveNodes::<Test>::get();

//         let prev_slot_list_len = SubnetNodeElectionSlots::<Test>::get(subnet_id).len();

//         assert_ok!(Network::remove_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id,
//         ));

//         assert_eq!(
//             SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
//             Err(())
//         );

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, hotkey.clone());
//         assert_eq!(subnet_node_id, Err(()));

//         let peer_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer_id.clone());
//         assert_eq!(peer_account, Err(()));

//         let bootnode_peer_account =
//             BootnodePeerIdSubnetNodeId::<Test>::try_get(subnet_id, bootnode_peer_id.clone());
//         assert_eq!(bootnode_peer_account, Err(()));

//         let subnet_node_hotkey =
//             SubnetNodeIdHotkey::<Test>::try_get(subnet_id, hotkey_subnet_node_id);
//         assert_eq!(subnet_node_hotkey, Err(()));

//         let subnet_node_hotkey = HotkeySubnetId::<Test>::try_get(hotkey.clone());
//         assert_eq!(subnet_node_hotkey, Err(()));

//         let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone()); // This is tested, see `test_clean_coldkey_subnet_nodes`
//         assert_eq!(coldkey_subnet_nodes.get(&subnet_id), None);

//         assert_eq!(
//             prev_total_subnet_nodes - 1,
//             TotalSubnetNodes::<Test>::get(subnet_id)
//         );
//         assert_eq!(prev_total_nodes - 1, TotalNodes::<Test>::get());

//         assert_eq!(
//             prev_total_active_subnet_nodes - 1,
//             TotalActiveSubnetNodes::<Test>::get(subnet_id)
//         );
//         assert_eq!(prev_total_active_nodes - 1, TotalActiveNodes::<Test>::get());

//         assert_eq!(
//             prev_slot_list_len,
//             SubnetNodeElectionSlots::<Test>::get(subnet_id).len()
//         );

//         //
//         //
//         //
//         // Test another node and force it into Validator
//         //
//         //
//         //

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 4);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 4);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 4);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 4);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 4);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             Some(bounded_unique.clone()),
//             Some(bounded_non_unique.clone()),
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let initial_start_epoch = subnet_node.classification.start_epoch;

//         set_block_to_subnet_slot_epoch(initial_start_epoch, subnet_id);

//         assert_ok!(Network::activate_subnet_node(
//             RuntimeOrigin::signed(hotkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id
//         ));

//         let mut subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         subnet_node.classification.start_epoch = 0;
//         subnet_node.classification.node_class = SubnetNodeClass::Validator;

//         SubnetNodesData::<Test>::insert(subnet_id, hotkey_subnet_node_id, subnet_node);

//         Network::insert_node_into_election_slot(subnet_id, hotkey_subnet_node_id);

//         let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
//         assert!(coldkey_subnet_nodes
//             .get(&subnet_id)
//             .unwrap()
//             .contains(&hotkey_subnet_node_id));

//         let prev_total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let prev_total_nodes = TotalNodes::<Test>::get();

//         let prev_total_active_subnet_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
//         let prev_total_active_nodes = TotalActiveNodes::<Test>::get();

//         let prev_slot_list_len = SubnetNodeElectionSlots::<Test>::get(subnet_id).len();

//         let prev_total_subnet_electable_nodes = TotalSubnetElectableNodes::<Test>::get(subnet_id);
//         let prev_total_electable_nodes = TotalElectableNodes::<Test>::get();

//         let rep = ColdkeyReputation::<Test>::get(coldkey.clone());
//         let rep_total_active_nodes = rep.total_active_nodes;

//         assert_ok!(Network::remove_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id,
//         ));

//         assert_eq!(
//             SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
//             Err(())
//         );

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, hotkey.clone());
//         assert_eq!(subnet_node_id, Err(()));

//         let peer_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer_id.clone());
//         assert_eq!(peer_account, Err(()));

//         let bootnode_peer_account =
//             BootnodePeerIdSubnetNodeId::<Test>::try_get(subnet_id, bootnode_peer_id.clone());
//         assert_eq!(bootnode_peer_account, Err(()));

//         let subnet_node_hotkey =
//             SubnetNodeIdHotkey::<Test>::try_get(subnet_id, hotkey_subnet_node_id);
//         assert_eq!(subnet_node_hotkey, Err(()));

//         let subnet_node_hotkey = HotkeySubnetId::<Test>::try_get(hotkey.clone());
//         assert_eq!(subnet_node_hotkey, Err(()));

//         let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone()); // This is tested, see `test_clean_coldkey_subnet_nodes`
//         assert_eq!(coldkey_subnet_nodes.get(&subnet_id), None);

//         assert_eq!(
//             prev_total_subnet_nodes - 1,
//             TotalSubnetNodes::<Test>::get(subnet_id)
//         );
//         assert_eq!(prev_total_nodes - 1, TotalNodes::<Test>::get());

//         assert_eq!(
//             prev_total_active_subnet_nodes - 1,
//             TotalActiveSubnetNodes::<Test>::get(subnet_id)
//         );
//         assert_eq!(prev_total_active_nodes - 1, TotalActiveNodes::<Test>::get());

//         assert_eq!(
//             prev_total_subnet_electable_nodes - 1,
//             TotalSubnetElectableNodes::<Test>::get(subnet_id)
//         );
//         assert_eq!(
//             prev_total_electable_nodes - 1,
//             TotalElectableNodes::<Test>::get()
//         );

//         assert_eq!(
//             prev_slot_list_len - 1,
//             SubnetNodeElectionSlots::<Test>::get(subnet_id).len()
//         );

//         assert_eq!(
//             rep_total_active_nodes - 1,
//             ColdkeyReputation::<Test>::get(coldkey.clone()).total_active_nodes
//         );
//     })
// }

// #[test]
// fn test_register_subnet_node_subnet_err() {
//     new_test_ext().execute_with(|| {
//         let subnet_id = 0;

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 0;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let amount: u128 = 1000;
//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::InvalidSubnetId
//         );

//         let subnet_id = 1;

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::InvalidSubnetId
//         );
//     })
// }

// #[test]
// fn test_get_classification_subnet_nodes() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let epoch_length = EpochLength::get();
//         let subnet_epoch: u32 = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         let submittable = Network::get_active_classified_subnet_nodes(
//             subnet_id,
//             &SubnetNodeClass::Validator,
//             subnet_epoch,
//         );

//         assert_eq!(submittable.len() as u32, total_subnet_nodes);
//     })
// }

// #[test]
// fn test_register_subnet_node_not_exists_err() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 16;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let used_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         // try reregistering again
//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 used_hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::HotkeyHasOwner
//         );

//         assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);

//         let bad_peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end);

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 bad_peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::PeerIdExist
//         );

//         let bad_bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end);

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bad_bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::BootnodePeerIdExist
//         );

//         let bad_client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end);

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 bad_client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::ClientPeerIdExist
//         );
//     })
// }

// #[test]
// fn test_add_subnet_node_stake_err() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 12;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let deposit_amount: u128 = 100000;
//         let amount: u128 = 1;

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::MinStakeNotReached
//         );
//     })
// }

// #[test]
// fn test_add_subnet_node_stake_not_enough_balance_err() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let deposit_amount: u128 = 999999999999999999999;

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::NotEnoughBalanceToStake
//         );
//     })
// }

// #[test]
// fn test_add_subnet_node_invalid_peer_id_err() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let peer_id = format!("2");
//         let bootnode_peer_id = format!("3");
//         let client_peer_id = format!("4");

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bad_peer: PeerId = PeerId(peer_id.clone().into());
//         let bootnode_peer: PeerId = PeerId(bootnode_peer_id.clone().into());
//         let client_peer: PeerId = PeerId(client_peer_id.clone().into());

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 bad_peer,
//                 bootnode_peer.clone(),
//                 client_peer.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::InvalidPeerId
//         );

//         let valid_peer_id = peer(subnets * max_subnet_nodes + end + 1);

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 valid_peer_id,
//                 bootnode_peer.clone(),
//                 client_peer.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::InvalidBootnodePeerId
//         );
//     })
// }

// #[test]
// fn test_add_subnet_node_remove_readd_new_hotkey() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let deposit_amount: u128 = 1000000000000000000000000;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_ok!(Network::remove_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//         ));

//         let account_subnet_stake = AccountSubnetStake::<Test>::get(&hotkey.clone(), subnet_id);

//         assert_ok!(Network::remove_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             account_subnet_stake,
//         ));

//         // let new_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 2);
//         // let new_peer_id = peer(subnets * max_subnet_nodes + end + 2);
//         // let new_bootnode_peer_id = peer(subnets * max_subnet_nodes + end + 2);
//         let new_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let new_peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let new_bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let new_client_peer_id =
//             get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             new_hotkey.clone(),
//             new_peer_id.clone(),
//             new_bootnode_peer_id.clone(),
//             new_client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));
//     });
// }

// #[test]
// fn test_remove_subnet_node_not_key_owner() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let deposit_amount: u128 = 1000000000000000000000000;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_err!(
//             Network::remove_subnet_node(RuntimeOrigin::signed(coldkey.clone()), subnet_id, 1),
//             Error::<Test>::NotKeyOwner
//         );
//     });
// }

// #[test]
// fn test_add_subnet_node_remove_readd_must_unstake_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 12;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let deposit_amount: u128 = 1000000000000000000000000;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_ok!(Network::remove_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//         ));
//     });
// }

// #[test]
// fn test_remove_subnet_nodes() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);
//         let remove_n_peers = total_subnet_nodes / 2;

//         let block_number = System::block_number();
//         let epoch_length = EpochLength::get();
//         let subnet_epoch: u32 = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         for n in 0..remove_n_peers {
//             let _n = n + 1;
//             let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
//             let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
//             let subnet_node_id =
//                 HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
//             assert_ok!(Network::remove_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//             ));
//             let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id);
//             assert_eq!(subnet_node_data, Err(()));
//         }

//         let node_set: BTreeSet<<Test as frame_system::Config>::AccountId> =
//             Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Idle, subnet_epoch);

//         assert_eq!(
//             node_set.len(),
//             (total_subnet_nodes - remove_n_peers) as usize
//         );
//         assert_eq!(Network::total_stake(), amount_staked);
//         assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked);
//         assert_eq!(
//             TotalSubnetNodes::<Test>::get(subnet_id),
//             total_subnet_nodes - remove_n_peers
//         );

//         for n in 0..remove_n_peers {
//             let _n = n + 1;
//             let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
//             let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
//             let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//             let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, hotkey.clone());
//             assert_eq!(subnet_node_id, Err(()));

//             let subnet_node_account =
//                 PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer_id.clone());
//             assert_eq!(subnet_node_account, Err(()));

//             let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
//             assert_eq!(account_subnet_stake, stake_amount);
//         }

//         let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
//         assert_eq!(total_subnet_stake, amount_staked);

//         let total_stake = TotalStake::<Test>::get();
//         assert_eq!(total_subnet_stake, amount_staked);
//     });
// }

// #[test]
// fn test_update_delegate_reward_rate() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let n_peers = 8;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         // let account_n = max_subnet_nodes+1*subnets;

//         build_activated_subnet_new(
//             subnet_name.clone(),
//             0,
//             n_peers,
//             deposit_amount,
//             stake_amount,
//         );

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.delegate_reward_rate, 0);
//         assert_eq!(subnet_node.last_delegate_reward_rate_update, 0);

//         let max_reward_rate_decrease = MaxRewardRateDecrease::<Test>::get();
//         let reward_rate_update_period = NodeRewardRateUpdatePeriod::<Test>::get();
//         let new_delegate_reward_rate = 50_000_000;

//         System::set_block_number(System::block_number() + reward_rate_update_period);

//         let block_number = System::block_number();

//         // Increase reward rate to 5% then test decreasing
//         assert_ok!(Network::update_node_delegate_reward_rate(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             new_delegate_reward_rate
//         ));

//         assert_eq!(
//             *network_events().last().unwrap(),
//             Event::SubnetNodeUpdateDelegateRewardRate {
//                 subnet_id,
//                 subnet_node_id,
//                 delegate_reward_rate: new_delegate_reward_rate
//             }
//         );

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.delegate_reward_rate, new_delegate_reward_rate);
//         assert_eq!(subnet_node.last_delegate_reward_rate_update, block_number);

//         System::set_block_number(System::block_number() + reward_rate_update_period);

//         let new_delegate_reward_rate = new_delegate_reward_rate - max_reward_rate_decrease;

//         // allow decreasing by 1%
//         assert_ok!(Network::update_node_delegate_reward_rate(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             new_delegate_reward_rate
//         ));

//         // Higher than 100%
//         assert_err!(
//             Network::update_node_delegate_reward_rate(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 MaxDelegateStakePercentage::<Test>::get() + 1
//             ),
//             Error::<Test>::InvalidDelegateRewardRate
//         );

//         // Update rewards rate as an increase too soon
//         assert_err!(
//             Network::update_node_delegate_reward_rate(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 new_delegate_reward_rate + 1
//             ),
//             Error::<Test>::MaxRewardRateUpdates
//         );

//         System::set_block_number(System::block_number() + reward_rate_update_period);

//         // Update rewards rate with no changes, don't allow
//         assert_err!(
//             Network::update_node_delegate_reward_rate(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 new_delegate_reward_rate
//             ),
//             Error::<Test>::NoDelegateRewardRateChange
//         );

//         // greater than max change
//         let new_delegate_reward_rate = new_delegate_reward_rate - max_reward_rate_decrease - 1;

//         assert_err!(
//             Network::update_node_delegate_reward_rate(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 new_delegate_reward_rate
//             ),
//             Error::<Test>::SurpassesMaxRewardRateDecrease
//         );
//     });
// }

// #[test]
// fn test_update_delegate_reward_rate_not_key_owner() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.delegate_reward_rate, 0);
//         assert_eq!(subnet_node.last_delegate_reward_rate_update, 0);

//         let max_reward_rate_decrease = MaxRewardRateDecrease::<Test>::get();
//         let reward_rate_update_period = NodeRewardRateUpdatePeriod::<Test>::get();
//         let new_delegate_reward_rate = 50_000_000;

//         System::set_block_number(System::block_number() + reward_rate_update_period);

//         let block_number = System::block_number();

//         // Increase reward rate to 5% then test decreasing
//         assert_err!(
//             Network::update_node_delegate_reward_rate(
//                 RuntimeOrigin::signed(account(2)),
//                 subnet_id,
//                 subnet_node_id,
//                 new_delegate_reward_rate
//             ),
//             Error::<Test>::NotKeyOwner
//         );
//     });
// }

// #[test]
// fn test_deactivate_subnet_node_invalid_subnet_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 4;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_err!(
//             Network::pause_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 999,
//                 hotkey_subnet_node_id,
//             ),
//             Error::<Test>::InvalidSubnetId
//         );
//     })
// }

// #[test]
// fn test_deactivate_subnet_node_not_uid_owner_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_err!(
//             Network::pause_subnet_node(
//                 RuntimeOrigin::signed(coldkey),
//                 subnet_id,
//                 hotkey_subnet_node_id + 999,
//             ),
//             Error::<Test>::NotUidOwner
//         );
//     })
// }

// #[test]
// fn test_deactivate_subnet_node_not_key_owner_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 4;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_err!(
//             Network::pause_subnet_node(
//                 RuntimeOrigin::signed(account(0)),
//                 subnet_id,
//                 hotkey_subnet_node_id,
//             ),
//             Error::<Test>::NotKeyOwner
//         );
//     })
// }

// #[test]
// fn test_deactivate_subnet_node_not_not_activated_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         // force update subnet node class
//         SubnetNodesData::<Test>::mutate(subnet_id, hotkey_subnet_node_id, |params| {
//             params.classification.node_class = SubnetNodeClass::Idle;
//         });

//         assert_err!(
//             Network::pause_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey_subnet_node_id,
//             ),
//             Error::<Test>::SubnetNodeNotActivated
//         );
//     })
// }

// #[test]
// fn test_deactivate_subnet_node_not_exist_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id,
//             bootnode_peer_id,
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_err!(
//             Network::pause_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey_subnet_node_id,
//             ),
//             Error::<Test>::InvalidSubnetNodeId
//         );
//     })
// }

// #[test]
// fn test_deactivate_subnet_node_is_validator_cannot_deactivate_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         SubnetElectedValidator::<Test>::insert(subnet_id, subnet_epoch, hotkey_subnet_node_id);

//         assert_err!(
//             Network::pause_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey_subnet_node_id,
//             ),
//             Error::<Test>::IsValidatorCannotDeactivate
//         );
//     })
// }

// #[test]
// fn test_reactivate_subnet_node_not_uid_owner_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_err!(
//             Network::reactivate_subnet_node(
//                 RuntimeOrigin::signed(coldkey),
//                 subnet_id,
//                 hotkey_subnet_node_id + 999,
//             ),
//             Error::<Test>::NotUidOwner
//         );
//     })
// }

// #[test]
// fn test_reactivate_subnet_node_not_key_owner_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 4;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_err!(
//             Network::reactivate_subnet_node(
//                 RuntimeOrigin::signed(account(0)),
//                 subnet_id,
//                 hotkey_subnet_node_id,
//             ),
//             Error::<Test>::NotKeyOwner
//         );
//     })
// }

// #[test]
// fn test_reactivate_subnet_node_not_registered_uid_owner_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 12;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end);
//         let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end);

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         // --- Try starting before start_epoch
//         assert_err!(
//             Network::reactivate_subnet_node(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey_subnet_node_id
//             ),
//             Error::<Test>::NotDeactivatedSubnetNode
//         );
//     })
// }

// #[test]
// fn test_update_peer_id() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end);
//         let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let current_peer_id = subnet_node.peer_id;

//         assert_ok!(Network::update_peer_id(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             peer(500)
//         ));

//         assert_eq!(
//             *network_events().last().unwrap(),
//             Event::SubnetNodeUpdatePeerId {
//                 subnet_id,
//                 subnet_node_id,
//                 peer_id: peer(500)
//             }
//         );

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.peer_id, peer(500));
//         assert_ne!(subnet_node.peer_id, current_peer_id);

//         let peer_subnet_node_id = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer(500));
//         assert_eq!(peer_subnet_node_id, subnet_node_id);

//         assert_eq!(
//             PeerIdSubnetNodeId::<Test>::try_get(subnet_id, &current_peer_id),
//             Err(())
//         );

//         let prev_peer_subnet_node_id = PeerIdSubnetNodeId::<Test>::get(subnet_id, &current_peer_id);
//         assert_ne!(prev_peer_subnet_node_id, subnet_node_id);

//         // test using previous peer id under a diff subnet node
//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end - 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end - 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end - 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end - 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end - 1);

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_ok!(Network::update_peer_id(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             current_peer_id.clone()
//         ));

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.peer_id, current_peer_id.clone());

//         let peer_subnet_node_id =
//             PeerIdSubnetNodeId::<Test>::get(subnet_id, current_peer_id.clone());
//         assert_eq!(peer_subnet_node_id, subnet_node_id);
//     })
// }

// #[test]
// fn test_update_peer_id_exists() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let current_peer_id = subnet_node.peer_id;

//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end - 1);

//         assert_err!(
//             Network::update_peer_id(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 peer_id
//             ),
//             Error::<Test>::PeerIdExist
//         );

//         // --- fail if same peer id
//         assert_err!(
//             Network::update_peer_id(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 current_peer_id
//             ),
//             Error::<Test>::PeerIdExist
//         );
//     })
// }

// #[test]
// fn test_update_peer_id_not_key_owner() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let current_peer_id = subnet_node.peer_id;

//         assert_err!(
//             Network::update_peer_id(
//                 RuntimeOrigin::signed(account(2)),
//                 subnet_id,
//                 subnet_node_id,
//                 peer(1)
//             ),
//             Error::<Test>::NotKeyOwner
//         );
//     })
// }

// #[test]
// fn test_update_peer_id_invalid_peer_id() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let peer_id = format!("2");

//         let bad_peer: PeerId = PeerId(peer_id.clone().into());

//         assert_err!(
//             Network::update_peer_id(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 bad_peer
//             ),
//             Error::<Test>::InvalidPeerId
//         );
//     })
// }

// #[test]
// fn test_update_bootnode() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let bootnode: Vec<u8> = "new-bootnode".into();
//         let bounded_bootnode: BoundedVec<u8, DefaultMaxVectorLength> =
//             bootnode.try_into().expect("String too long");

//         assert_ok!(Network::update_bootnode(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             Some(bounded_bootnode.clone())
//         ));

//         assert_eq!(
//             *network_events().last().unwrap(),
//             Event::SubnetNodeUpdateBootnode {
//                 subnet_id,
//                 subnet_node_id,
//                 bootnode: Some(bounded_bootnode.clone())
//             }
//         );

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.bootnode, Some(bounded_bootnode.clone()));
//         assert_eq!(
//             BootnodeSubnetNodeId::<Test>::get(subnet_id, bounded_bootnode.clone()),
//             subnet_node_id
//         );

//         // Can update to None
//         assert_ok!(Network::update_bootnode(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             None
//         ));
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.bootnode, None);
//         assert_eq!(
//             BootnodeSubnetNodeId::<Test>::try_get(subnet_id, bounded_bootnode.clone()),
//             Err(())
//         );

//         assert_ok!(Network::update_bootnode(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             Some(bounded_bootnode.clone())
//         ));
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.bootnode, Some(bounded_bootnode.clone()));
//         assert_eq!(
//             BootnodeSubnetNodeId::<Test>::get(subnet_id, bounded_bootnode.clone()),
//             subnet_node_id
//         );

//         // Another node should be able to use the removed bootnode
//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end - 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end - 1);
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_ok!(Network::update_bootnode(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             Some(bounded_bootnode.clone())
//         ));
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.bootnode, Some(bounded_bootnode.clone()));
//         assert_eq!(
//             BootnodeSubnetNodeId::<Test>::get(subnet_id, bounded_bootnode.clone()),
//             subnet_node_id
//         );
//     })
// }

// #[test]
// fn test_update_bootnode_not_key_owner() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let bootnode: Vec<u8> = "new-bootnode".into();
//         let bounded_bootnode: BoundedVec<u8, DefaultMaxVectorLength> =
//             bootnode.try_into().expect("String too long");

//         assert_err!(
//             Network::update_bootnode(
//                 RuntimeOrigin::signed(account(2)),
//                 subnet_id,
//                 subnet_node_id,
//                 Some(bounded_bootnode.clone())
//             ),
//             Error::<Test>::NotKeyOwner
//         );
//     })
// }

// #[test]
// fn test_update_bootnode_peer_id() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let current_bootnode_peer_id = subnet_node.bootnode_peer_id;

//         assert_ok!(Network::update_bootnode_peer_id(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             peer(500)
//         ));

//         assert_eq!(
//             *network_events().last().unwrap(),
//             Event::SubnetNodeUpdateBootnodePeerId {
//                 subnet_id,
//                 subnet_node_id,
//                 bootnode_peer_id: peer(500)
//             }
//         );

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.bootnode_peer_id, peer(500));
//         assert_ne!(subnet_node.bootnode_peer_id, current_bootnode_peer_id);

//         let bootnode_peer_subnet_node_id =
//             BootnodePeerIdSubnetNodeId::<Test>::get(subnet_id, peer(500));
//         assert_eq!(bootnode_peer_subnet_node_id, subnet_node_id);

//         assert_eq!(
//             BootnodePeerIdSubnetNodeId::<Test>::try_get(subnet_id, &current_bootnode_peer_id),
//             Err(())
//         );

//         let prev_bootnode_peer_subnet_node_id =
//             BootnodePeerIdSubnetNodeId::<Test>::get(subnet_id, &current_bootnode_peer_id);
//         assert_ne!(prev_bootnode_peer_subnet_node_id, subnet_node_id);

//         // test using previous peer id under a diff subnet node
//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end - 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end - 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end - 1);

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_ok!(Network::update_bootnode_peer_id(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             current_bootnode_peer_id.clone()
//         ));

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(
//             subnet_node.bootnode_peer_id,
//             current_bootnode_peer_id.clone()
//         );

//         let bootnode_peer_subnet_node_id =
//             BootnodePeerIdSubnetNodeId::<Test>::get(subnet_id, current_bootnode_peer_id.clone());
//         assert_eq!(bootnode_peer_subnet_node_id, subnet_node_id);
//     })
// }

// #[test]
// fn test_update_bootnode_peer_id_exists() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let current_bootnode_peer_id = subnet_node.bootnode_peer_id;

//         let someone_elses_bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end - 1);

//         assert_err!(
//             Network::update_bootnode_peer_id(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 someone_elses_bootnode_peer_id
//             ),
//             Error::<Test>::BootnodePeerIdExist
//         );

//         // --- fail if same peer id
//         assert_err!(
//             Network::update_bootnode_peer_id(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 current_bootnode_peer_id
//             ),
//             Error::<Test>::BootnodePeerIdExist
//         );
//     })
// }

// #[test]
// fn test_update_bootnode_peer_id_not_key_owner() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let current_bootnode_peer_id = subnet_node.bootnode_peer_id;

//         assert_err!(
//             Network::update_bootnode_peer_id(
//                 RuntimeOrigin::signed(account(2)),
//                 subnet_id,
//                 subnet_node_id,
//                 peer(1)
//             ),
//             Error::<Test>::NotKeyOwner
//         );
//     })
// }

// #[test]
// fn test_update_bootnode_peer_id_invalid_peer_id() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let bootnode_peer_id = format!("2");

//         let bad_bootnode_peer: PeerId = PeerId(bootnode_peer_id.clone().into());

//         assert_err!(
//             Network::update_bootnode_peer_id(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 bad_bootnode_peer
//             ),
//             Error::<Test>::InvalidBootnodePeerId
//         );
//     })
// }

// #[test]
// fn test_update_client_peer_id() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         // current peer id
//         let current_client_peer_id = subnet_node.client_peer_id;

//         // new and unused peer id
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         assert_ok!(Network::update_client_peer_id(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             client_peer_id.clone()
//         ));

//         assert_eq!(
//             *network_events().last().unwrap(),
//             Event::SubnetNodeUpdateClientPeerId {
//                 subnet_id,
//                 subnet_node_id,
//                 client_peer_id: client_peer_id.clone()
//             }
//         );

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.client_peer_id, client_peer_id.clone());
//         assert_ne!(subnet_node.client_peer_id, current_client_peer_id);

//         let client_peer_subnet_node_id =
//             ClientPeerIdSubnetNodeId::<Test>::get(subnet_id, client_peer_id.clone());
//         assert_eq!(client_peer_subnet_node_id, subnet_node_id);

//         assert_eq!(
//             ClientPeerIdSubnetNodeId::<Test>::try_get(subnet_id, &current_client_peer_id),
//             Err(())
//         );

//         let prev_client_peer_subnet_node_id =
//             ClientPeerIdSubnetNodeId::<Test>::get(subnet_id, &current_client_peer_id);
//         assert_ne!(prev_client_peer_subnet_node_id, subnet_node_id);

//         // test using previous peer id under a diff subnet node
//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end - 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end - 1);

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_ok!(Network::update_client_peer_id(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             current_client_peer_id.clone()
//         ));

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.client_peer_id, current_client_peer_id.clone());

//         let client_peer_subnet_node_id =
//             ClientPeerIdSubnetNodeId::<Test>::get(subnet_id, current_client_peer_id.clone());
//         assert_eq!(client_peer_subnet_node_id, subnet_node_id);
//     })
// }

// #[test]
// fn test_update_client_peer_id_exists() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let current_client_peer_id = subnet_node.client_peer_id;

//         let peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end - 1);

//         assert_err!(
//             Network::update_client_peer_id(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 peer_id
//             ),
//             Error::<Test>::ClientPeerIdExist
//         );

//         // --- fail if same peer id
//         assert_err!(
//             Network::update_client_peer_id(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 current_client_peer_id
//             ),
//             Error::<Test>::ClientPeerIdExist
//         );
//     })
// }

// #[test]
// fn test_update_client_peer_id_not_key_owner() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let current_client_peer_id = subnet_node.client_peer_id;

//         assert_err!(
//             Network::update_client_peer_id(
//                 RuntimeOrigin::signed(account(2)),
//                 subnet_id,
//                 subnet_node_id,
//                 peer(1)
//             ),
//             Error::<Test>::NotKeyOwner
//         );
//     })
// }

// #[test]
// fn test_update_client_peer_id_invalid_peer_id() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let client_peer_id = format!("2");

//         let bad_client_peer: PeerId = PeerId(client_peer_id.clone().into());

//         assert_err!(
//             Network::update_client_peer_id(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 bad_client_peer
//             ),
//             Error::<Test>::InvalidClientPeerId
//         );
//     })
// }

// #[test]
// fn subnet_stake_multiplier_works() {
//     new_test_ext().execute_with(|| {
//         let subnet_id = 1;

//         // Set test constants
//         MinSubnetNodes::<Test>::put(10);
//         MaxSubnetNodes::<Test>::put(100);
//         TotalActiveSubnetNodes::<Test>::insert(subnet_id, 10);

//         // Multiplier should be 100% at min
//         let mult = Network::get_subnet_min_delegate_staking_multiplier(10);
//         assert_eq!(mult, Network::percentage_factor_as_u128()); // 100%

//         // Multiplier should be 400% at max
//         TotalActiveSubnetNodes::<Test>::insert(subnet_id, 100);
//         let mult = Network::get_subnet_min_delegate_staking_multiplier(100);
//         assert_eq!(mult, 4000000000000000000); // 400%

//         // Multiplier should be ~250% halfway
//         TotalActiveSubnetNodes::<Test>::insert(subnet_id, 55); // halfway between 10 and 100
//         let mult = Network::get_subnet_min_delegate_staking_multiplier(55);
//         let expected = Network::percentage_factor_as_u128() + (3000000000000000000 / 2);
//         assert_eq!(mult, expected);
//     });
// }

// #[test]
// fn test_subnet_overwatch_node_unique_hotkeys() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let end = 16;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

//         let deposit_amount: u128 = 1000000000000000000000000;

//         let free_coldkey = account(subnet_id * total_subnet_nodes + 1);
//         let hotkey = account(max_subnet_nodes + end * subnets + 1);
//         let free_hotkey = account(max_subnet_nodes + end * subnets + 2);

//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 2);

//         let _ = Balances::deposit_creating(&free_coldkey, deposit_amount);

//         make_overwatch_qualified(subnet_id * total_subnet_nodes + 1);

//         assert_ok!(Network::register_overwatch_node(
//             RuntimeOrigin::signed(free_coldkey.clone()),
//             hotkey.clone(),
//             stake_amount,
//         ));

//         assert_err!(
//             Network::register_subnet_node(
//                 RuntimeOrigin::signed(free_coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 peer_id.clone(),
//                 bootnode_peer_id.clone(),
//                 client_peer_id.clone(),
//                 None,
//                 0,
//                 amount,
//                 None,
//                 None,
//             ),
//             Error::<Test>::HotkeyHasOwner
//         );

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(free_coldkey.clone()),
//             subnet_id,
//             free_hotkey.clone(),
//             peer_id,
//             bootnode_peer_id,
//             client_peer_id,
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));
//     });
// }

// #[test]
// fn test_defer_node() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = max_subnet_nodes;

//         build_activated_subnet_new(
//             subnet_name.clone(),
//             0,
//             max_subnet_nodes,
//             deposit_amount,
//             stake_amount,
//         );

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let subnet_node_queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id);

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, end + 1);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
//         let starting_balance = Balances::free_balance(&coldkey.clone());

//         assert_ok!(Network::register_subnet_node(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             peer_id.clone(),
//             bootnode_peer_id.clone(),
//             client_peer_id.clone(),
//             None,
//             0,
//             amount,
//             None,
//             None,
//         ));

//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let initial_start_epoch = subnet_node.classification.start_epoch;

//         set_block_to_subnet_slot_epoch(initial_start_epoch, subnet_id);

//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         assert_ok!(Network::activate_subnet_node(
//             RuntimeOrigin::signed(hotkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id
//         ));

//         assert_eq!(
//             SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
//             Err(())
//         );

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let deferred_start_epoch = subnet_node.classification.start_epoch;
//         assert_ne!(initial_start_epoch, deferred_start_epoch);
//         assert_eq!(
//             deferred_start_epoch,
//             subnet_epoch + subnet_node_queue_epochs
//         );
//     })
// }

// #[test]
// fn test_clean_coldkey_subnet_nodes() {
//     new_test_ext().execute_with(|| {
//         insert_subnet(1, SubnetState::Active, 0);
//         insert_subnet(2, SubnetState::Active, 0);

//         // Seed data
//         let coldkey = account(1);
//         let mut subnet_nodes: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();

//         // Subnet 1: Valid subnet with valid and invalid nodes
//         insert_subnet_node(
//             1,
//             100, // node id
//             1,   // coldkey
//             2,   // hotkey
//             2,   // peer
//             SubnetNodeClass::Validator,
//             0,
//         );
//         let mut node_ids1 = BTreeSet::new();
//         node_ids1.insert(100); // Valid node
//                                // insert nodes for subnet 1
//         subnet_nodes.insert(1, node_ids1);

//         // Subnet 2: Valid subnet with only valid nodes
//         insert_subnet_node(
//             2,
//             200, // node id
//             1,   // coldkey
//             3,   // hotkey
//             3,   // peer
//             SubnetNodeClass::Validator,
//             0,
//         );
//         let mut node_ids2 = BTreeSet::new();
//         node_ids2.insert(200); // Valid node
//                                // insert nodes for subnet 2
//         subnet_nodes.insert(2, node_ids2);

//         // Subnet 3: Invalid subnet
//         let mut node_ids3 = BTreeSet::new();
//         node_ids3.insert(300); // Valid node
//         node_ids3.insert(301); // Invalid node
//                                // insert nodes for subnet 3
//         subnet_nodes.insert(3, node_ids3);

//         // Insert seed data into storage
//         ColdkeySubnetNodes::<Test>::insert(coldkey.clone(), subnet_nodes);

//         // Verify initial state
//         let initial = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
//         assert_eq!(initial.len(), 3);
//         assert_eq!(initial.get(&1).unwrap().len(), 1);
//         assert_eq!(initial.get(&2).unwrap().len(), 1);

//         // Subnet doesn't exist, both will be removed later
//         assert_eq!(initial.get(&3).unwrap().len(), 2);

//         // Call the function to clean invalid subnets and nodes
//         Network::clean_coldkey_subnet_nodes(coldkey.clone());

//         // Verify final state
//         let final_state = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
//         log::error!("final_state {:?}", final_state);

//         assert_eq!(final_state.len(), 2, "Invalid subnet 3 should be removed");

//         assert_eq!(
//             final_state.get(&1).unwrap().len(),
//             1,
//             "Invalid node 101 should be removed from subnet 1"
//         );

//         assert!(final_state.get(&1).unwrap().contains(&100));
//         assert!(final_state.get(&1).unwrap().contains(&101) == false);

//         assert_eq!(
//             final_state.get(&2).unwrap().len(),
//             1,
//             "Subnet 2 should remain unchanged"
//         );
//         assert!(final_state.get(&2).unwrap().contains(&200));
//         assert!(final_state.get(&3).is_none(), "Subnet 3 should be gone");
//     })
// }

// #[test]
// fn test_clean_expired_registered_subnet_nodes_with_no_data() {
//     new_test_ext().execute_with(|| {
//         let subnet_name_1: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();
//         let end = 4;
//         build_activated_subnet_new(subnet_name_1.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id_1 = SubnetName::<Test>::get(subnet_name_1.clone()).unwrap();

//         let subnet_name_2: Vec<u8> = "subnet-name-2".into();
//         build_activated_subnet_new(subnet_name_2.clone(), 0, end, deposit_amount, stake_amount);
//         let subnet_id_2 = SubnetName::<Test>::get(subnet_name_2.clone()).unwrap();

//         // make sure data is fresh in the later checks and asserts
//         let prev_registered_count =
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id_1).count();
//         assert_eq!(prev_registered_count as u32, 0);

//         let prev_registered_count =
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id_2).count();
//         assert_eq!(prev_registered_count as u32, 0);

//         let start = end + 1;
//         let end = start + 4;

//         build_registered_nodes_in_queue(subnet_id_1, start, end, deposit_amount, stake_amount);

//         build_registered_nodes_in_queue(subnet_id_2, start, end, deposit_amount, stake_amount);

//         // Increase past registration epochs
//         let queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id_1);
//         let grace_epochs = ActivationGraceEpochs::<Test>::get(subnet_id_1);

//         increase_epochs(queue_epochs + grace_epochs + 1);

//         // Subnet 1
//         let prev_registered_count_1 =
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id_1).count();
//         assert_eq!(prev_registered_count_1 as u32, end - start);
//         assert_ne!(prev_registered_count_1 as u32, 0);
//         // Subnet 2
//         let prev_registered_count_2 =
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id_2).count();
//         assert_eq!(prev_registered_count_2 as u32, end - start);
//         assert_ne!(prev_registered_count_2 as u32, 0);

//         assert_ok!(Network::clean_expired_registered_subnet_nodes(
//             RuntimeOrigin::signed(account(999)),
//             0,
//             0
//         ));

//         // Subnet 1
//         assert_eq!(
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id_1).count(),
//             0
//         );
//         // Subnet 2
//         assert_eq!(
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id_2).count(),
//             0
//         );
//     })
// }

// #[test]
// fn test_clean_expired_registered_subnet_nodes_with_subnet_id() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         // make sure data is fresh in the later checks and asserts
//         let prev_registered_count =
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id).count();
//         assert_eq!(prev_registered_count as u32, 0);

//         let start = end + 1;
//         let end = start + 4;

//         build_registered_nodes_in_queue(subnet_id, start, end, deposit_amount, stake_amount);

//         // Increase past registration epochs
//         let queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id);
//         let grace_epochs = ActivationGraceEpochs::<Test>::get(subnet_id);

//         increase_epochs(queue_epochs + grace_epochs + 1);

//         let prev_registered_count =
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id).count();
//         assert_eq!(prev_registered_count as u32, end - start);
//         assert_ne!(prev_registered_count as u32, 0);

//         assert_ok!(Network::clean_expired_registered_subnet_nodes(
//             RuntimeOrigin::signed(account(999)),
//             subnet_id,
//             0
//         ));

//         let registered_count = RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id).count();
//         assert_eq!(registered_count, 0);
//     })
// }

// #[test]
// fn test_clean_expired_registered_subnet_nodes_with_subnet_node_id() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let max_subnets = MaxSubnets::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         // make sure data is fresh in the later checks and asserts
//         let prev_registered_count =
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id).count();
//         assert_eq!(prev_registered_count as u32, 0);

//         let start = end + 1;
//         let end = start + 4;

//         build_registered_nodes_in_queue(subnet_id, start, end, deposit_amount, stake_amount);

//         // get specific subnet node id
//         let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, end);
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         // Sanity check
//         assert_ne!(subnet_id, 0);
//         assert_ne!(subnet_node_id, 0);
//         assert!(RegisteredSubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id).is_ok());

//         // Increase past registration epochs
//         let queue_epochs = SubnetNodeQueueEpochs::<Test>::get(subnet_id);
//         let grace_epochs = ActivationGraceEpochs::<Test>::get(subnet_id);

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         increase_epochs(queue_epochs + grace_epochs + 1);

//         let prev_registered_count =
//             RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id).count();
//         assert_eq!(prev_registered_count as u32, end - start);
//         assert_ne!(prev_registered_count as u32, 0);

//         assert_ok!(Network::clean_expired_registered_subnet_nodes(
//             RuntimeOrigin::signed(account(999)),
//             subnet_id,
//             subnet_node_id
//         ));

//         let registered_count = RegisteredSubnetNodesData::<Test>::iter_prefix(subnet_id).count();
//         assert_eq!(registered_count, prev_registered_count - 1);

//         // Check node ID was removed
//         assert_eq!(
//             RegisteredSubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id),
//             Err(())
//         );
//     })
// }

// #[test]
// fn test_update_unique() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let unique: Vec<u8> = "a".into();
//         let bounded_unique: BoundedVec<u8, DefaultMaxVectorLength> =
//             unique.try_into().expect("String too long");

//         // sanity check
//         assert_eq!(
//             SubnetNodeUniqueParam::<Test>::try_get(subnet_id, &bounded_unique),
//             Err(())
//         );

//         // update unique parameter
//         assert_ok!(Network::update_unique(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             bounded_unique.clone()
//         ));

//         assert_eq!(
//             *network_events().last().unwrap(),
//             Event::SubnetNodeUpdateUnique {
//                 subnet_id,
//                 subnet_node_id,
//                 unique: bounded_unique.clone()
//             }
//         );

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.unique, Some(bounded_unique.clone()));
//         let unique_owner_id = SubnetNodeUniqueParam::<Test>::get(subnet_id, &bounded_unique);
//         assert_eq!(subnet_node_id, unique_owner_id);

//         // Allow same parameter if owner
//         assert_ok!(Network::update_unique(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             bounded_unique.clone()
//         ));

//         // Shouldn't allow same parameter unless owner
//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end - 1);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end - 1);

//         assert_err!(
//             Network::update_unique(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 bounded_unique.clone()
//             ),
//             Error::<Test>::NotKeyOwner
//         );

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_err!(
//             Network::update_unique(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 bounded_unique.clone()
//             ),
//             Error::<Test>::UniqueParameterTaken
//         );

//         // back to original node and update to a new value
//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let new_unique: Vec<u8> = "new".into();
//         let new_bounded_unique: BoundedVec<u8, DefaultMaxVectorLength> =
//             new_unique.try_into().expect("String too long");

//         assert_ok!(Network::update_unique(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             new_bounded_unique.clone()
//         ));

//         // ensure old deletes
//         assert_eq!(
//             SubnetNodeUniqueParam::<Test>::try_get(subnet_id, &bounded_unique),
//             Err(())
//         );

//         // new
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.unique, Some(new_bounded_unique.clone()));
//         let unique_owner_id = SubnetNodeUniqueParam::<Test>::get(subnet_id, &new_bounded_unique);
//         assert_eq!(subnet_node_id, unique_owner_id);
//     })
// }

// #[test]
// fn test_update_non_unique() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         let non_unique: Vec<u8> = "a".into();
//         let bounded_non_unique: BoundedVec<u8, DefaultMaxVectorLength> =
//             non_unique.try_into().expect("String too long");

//         assert_ok!(Network::update_non_unique(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             bounded_non_unique.clone()
//         ));

//         assert_eq!(
//             *network_events().last().unwrap(),
//             Event::SubnetNodeUpdateNonUnique {
//                 subnet_id,
//                 subnet_node_id,
//                 non_unique: bounded_non_unique.clone()
//             }
//         );

//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.non_unique, Some(bounded_non_unique.clone()));

//         assert_err!(
//             Network::update_unique(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 0,
//                 bounded_non_unique.clone()
//             ),
//             Error::<Test>::NotKeyOwner
//         );
//     })
// }
