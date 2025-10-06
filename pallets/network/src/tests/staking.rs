// use super::mock::*;
// use crate::tests::test_utils::*;
// use crate::{
//     Error, HotkeySubnetNodeId, MaxSubnetNodes, MaxSubnets, MinActiveNodeStakeEpochs,
//     NetworkMinStakeBalance, RegisteredSubnetNodesData,
//     StakeUnbondingLedger, SubnetMaxStakeBalance, SubnetName, SubnetNodesData,
//     SubnetRemovalReason, SubnetsData, TotalActiveSubnets, TotalSubnetNodeUids, TotalSubnetNodes,
//     TotalSubnetStake, StakeCooldownEpochs
// };
// use frame_support::traits::Currency;
// use frame_support::{assert_err, assert_ok};
// use sp_std::collections::btree_map::BTreeMap;

// // ///
// // ///
// // ///
// // ///
// // ///
// // ///
// // ///
// // /// Staking
// // ///
// // ///
// // ///
// // ///
// // ///
// // ///
// // ///

// #[test]
// fn test_add_to_stake_not_key_owner() {
//     new_test_ext().execute_with(|| {
//         let deposit_amount: u128 = 1000000000000000000000000;
//         let amount: u128 = 1000000000000000000000;
//         let _ = Balances::deposit_creating(&account(1), deposit_amount);

//         assert_err!(
//             Network::add_stake(RuntimeOrigin::signed(account(1)), 0, 0, account(1), amount),
//             Error::<Test>::InvalidSubnetId,
//         );

//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

//         build_activated_subnet_new(subnet_name.clone(), 0, 0, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         let _ = Balances::deposit_creating(&account(1), deposit_amount);

//         assert_err!(
//             Network::add_stake(
//                 RuntimeOrigin::signed(account(total_subnet_nodes + 1)),
//                 subnet_id,
//                 0,
//                 account(total_subnet_nodes + 1),
//                 amount,
//             ),
//             Error::<Test>::NotKeyOwner,
//         );
//     });
// }

// #[test]
// fn test_remove_stake_not_key_owner() {
//     new_test_ext().execute_with(|| {
//         let deposit_amount: u128 = 1000000000000000000000000;
//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//         let _ = Balances::deposit_creating(&account(1), deposit_amount);

//         assert_err!(
//             Network::add_stake(
//                 RuntimeOrigin::signed(account(1)),
//                 0,
//                 0,
//                 account(1),
//                 stake_amount,
//             ),
//             Error::<Test>::InvalidSubnetId,
//         );

//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_ok!(Network::add_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             hotkey.clone(),
//             stake_amount,
//         ));

//         assert_err!(
//             Network::remove_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 account(2),
//                 stake_amount,
//             ),
//             Error::<Test>::NotKeyOwner,
//         );
//     });
// }

// #[test]
// fn test_add_to_stake() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;

//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_eq!(
//             Network::account_subnet_stake(hotkey.clone(), subnet_id),
//             stake_amount
//         );

//         assert_ok!(Network::add_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             hotkey.clone(),
//             stake_amount,
//         ));

//         assert_eq!(
//             Network::account_subnet_stake(hotkey.clone(), subnet_id),
//             stake_amount + stake_amount
//         );
//         assert_eq!(Network::total_stake(), amount_staked + stake_amount);
//         assert_eq!(
//             Network::total_subnet_stake(subnet_id),
//             amount_staked + stake_amount
//         );
//     });
// }

// #[test]
// fn test_add_to_stake_invalid_amount_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;

//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_eq!(
//             Network::account_subnet_stake(hotkey.clone(), subnet_id),
//             stake_amount
//         );

//         assert_err!(
//             Network::add_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 hotkey.clone(),
//                 0,
//             ),
//             Error::<Test>::InvalidAmount
//         );
//     });
// }

// #[test]
// fn test_add_to_stake_max_stake_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;

//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

//         let max_stake = SubnetMaxStakeBalance::<Test>::get(subnet_id);

//         let _ = Balances::deposit_creating(&coldkey.clone(), max_stake + 500);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_eq!(
//             Network::account_subnet_stake(hotkey.clone(), subnet_id),
//             stake_amount
//         );

//         assert_err!(
//             Network::add_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 hotkey.clone(),
//                 max_stake,
//             ),
//             Error::<Test>::MaxStakeReached
//         );
//     });
// }

// #[test]
// fn test_add_to_stake_not_enough_balance_to_stake_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(
//             subnet_name.clone(),
//             0,
//             end,
//             stake_amount + 500,
//             stake_amount,
//         );

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_err!(
//             Network::add_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 hotkey.clone(),
//                 stake_amount,
//             ),
//             Error::<Test>::NotEnoughBalanceToStake
//         );
//     });
// }

// #[test]
// fn test_add_to_stake_balance_withdraw_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(
//             subnet_name.clone(),
//             0,
//             end,
//             stake_amount + 500,
//             stake_amount,
//         );

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

//         let max_stake = SubnetMaxStakeBalance::<Test>::get(subnet_id);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_err!(
//             Network::add_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 subnet_node_id,
//                 hotkey.clone(),
//                 400,
//             ),
//             Error::<Test>::BalanceWithdrawalError
//         );
//     });
// }

// #[test]
// fn test_remove_stake_not_enough_stake_error() {
//     new_test_ext().execute_with(|| {
//         let deposit_amount: u128 = 1000000000000000000000000;
//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let coldkey = get_coldkey(subnet_id, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, end);

//         let min_stake_epochs = MinActiveNodeStakeEpochs::<Test>::get();
//         increase_epochs(min_stake_epochs + 2);

//         assert_err!(
//             Network::remove_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 stake_amount + 1,
//             ),
//             Error::<Test>::NotEnoughStakeToWithdraw,
//         );

//         assert_err!(
//             Network::remove_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 0,
//             ),
//             Error::<Test>::AmountZero,
//         );
//     });
// }

// #[test]
// fn test_remove_stake_min_stake_not_reached_error() {
//     new_test_ext().execute_with(|| {
//         let deposit_amount: u128 = 1000000000000000000000000;
//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let end = 4;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);

//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let coldkey = get_coldkey(subnet_id, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, end);

//         let min_stake_epochs = MinActiveNodeStakeEpochs::<Test>::get();
//         increase_epochs(min_stake_epochs + 2);

//         assert_err!(
//             Network::remove_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 stake_amount,
//             ),
//             Error::<Test>::MinStakeNotReached,
//         );
//     });
// }

// #[test]
// fn test_remove_stake() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

//         // add double amount to stake
//         assert_ok!(Network::add_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             hotkey.clone(),
//             stake_amount,
//         ));

//         assert_eq!(
//             Network::account_subnet_stake(hotkey.clone(), subnet_id),
//             stake_amount + stake_amount
//         );

//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
//         let min_stake_epochs = MinActiveNodeStakeEpochs::<Test>::get();

//         set_block_to_subnet_slot_epoch(
//             subnet_node.classification.start_epoch + min_stake_epochs + 1,
//             subnet_id,
//         );

//         log::error!("subnet_epoch {:?}", subnet_epoch);
//         log::error!(
//             "subnet_node.classification.start_epoch + min_stake_epochs {:?}",
//             subnet_node.classification.start_epoch + min_stake_epochs
//         );

//         let block = System::block_number();

//         // remove amount ontop
//         assert_ok!(Network::remove_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             stake_amount,
//         ));

//         assert_eq!(
//             Network::account_subnet_stake(hotkey.clone(), subnet_id),
//             stake_amount
//         );

//         let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(coldkey.clone());
//         let total_ledger_balance: u128 = unbondings.values().copied().sum();
//         assert_eq!(unbondings.len() as u32, 1);
//         assert_eq!(total_ledger_balance, stake_amount);
//         let (ledger_block, ledger_balance) = unbondings.iter().last().unwrap();
//         assert_eq!(
//             *ledger_block,
//             &block + StakeCooldownEpochs::<Test>::get() * EpochLength::get()
//         );
//         assert_eq!(*ledger_balance, stake_amount);
//     });
// }

// #[test]
// fn test_remove_stake_min_active_node_stake_epochs() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;
//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 11;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
//         let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, end + 1);
//         let peer_id = get_peer_id(subnet_id, max_subnet_nodes, max_subnets, end + 1);
//         let bootnode_peer_id =
//             get_bootnode_peer_id(subnet_id, max_subnet_nodes, max_subnets, end + 1);
//         let client_peer_id = get_client_peer_id(subnet_id, max_subnet_nodes, max_subnets, end + 1);

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
//             stake_amount,
//             None,
//             None,
// u128::MAX
//         ));

//         let total_subnet_node_uids = TotalSubnetNodeUids::<Test>::get(subnet_id);
//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let subnet_node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let start_epoch = subnet_node.classification.start_epoch;

//         // set_epoch(start_epoch, 0);
//         set_block_to_subnet_slot_epoch(start_epoch, subnet_id);

//         assert_ok!(Network::activate_subnet_node(
//             RuntimeOrigin::signed(hotkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id
//         ));

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         // add double amount to stake
//         assert_ok!(Network::add_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             subnet_node_id,
//             hotkey.clone(),
//             stake_amount,
//         ));

//         assert_eq!(
//             Network::account_subnet_stake(hotkey.clone(), subnet_id),
//             stake_amount + stake_amount
//         );

//         assert_err!(
//             Network::remove_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 stake_amount,
//             ),
//             Error::<Test>::MinActiveNodeStakeEpochs
//         );

//         let min_stake_epochs = MinActiveNodeStakeEpochs::<Test>::get();
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
//         let start_epoch = subnet_node.classification.start_epoch;

//         set_epoch(start_epoch + min_stake_epochs + 2, 0); // increase by 2 to account for subnet epoch crossover

//         assert_ok!(Network::remove_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             stake_amount,
//         ));

//         // assert_eq!(Network::account_subnet_stake(hotkey.clone(), subnet_id), amount);
//     });
// }

// #[test]
// fn test_remove_stake_after_remove_subnet_node() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let end = 11;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
//         let hotkey_subnet_node_id =
//             HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         assert_ok!(Network::remove_subnet_node(
//             RuntimeOrigin::signed(hotkey.clone()),
//             subnet_id,
//             hotkey_subnet_node_id,
//         ));

//         let epoch_length = EpochLength::get();
//         let min_required_unstake_epochs = StakeCooldownEpochs::<Test>::get();
//         System::set_block_number(
//             System::block_number() + epoch_length * min_required_unstake_epochs,
//         );

//         // remove amount ontop
//         assert_ok!(Network::remove_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             stake_amount,
//         ));

//         assert_eq!(Network::account_subnet_stake(hotkey.clone(), subnet_id), 0);

//         // assert_eq!(Network::account_subnet_stake(account(1), 1), 0);
//         // assert_eq!(Network::total_account_stake(account(1)), 0);
//         // assert_eq!(Network::total_stake(), 0);
//         // assert_eq!(Network::total_subnet_stake(1), 0);
//     });
// }

// #[test]
// fn test_remove_stake_after_remove_subnet() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let end = 11;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         Network::do_remove_subnet(subnet_id, SubnetRemovalReason::MinSubnetDelegateStake);

//         assert_eq!(SubnetsData::<Test>::contains_key(subnet_id), false);
//         assert_eq!(
//             HotkeySubnetNodeId::<Test>::try_get(subnet_id, &hotkey),
//             Err(())
//         );

//         let epoch_length = EpochLength::get();
//         let min_required_unstake_epochs = StakeCooldownEpochs::<Test>::get();
//         System::set_block_number(
//             System::block_number() + epoch_length * min_required_unstake_epochs,
//         );

//         // remove amount ontop
//         assert_ok!(Network::remove_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             stake_amount,
//         ));

//         assert_eq!(Network::account_subnet_stake(hotkey.clone(), subnet_id), 0);
//     });
// }

// #[test]
// fn test_remove_stake_after_remove_subnet_twice() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();
//         let deposit_amount: u128 = 1000000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();
//         let end = 11;

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

//         Network::do_remove_subnet(subnet_id, SubnetRemovalReason::MinSubnetDelegateStake);

//         assert_eq!(SubnetsData::<Test>::contains_key(subnet_id), false);
//         assert_eq!(
//             HotkeySubnetNodeId::<Test>::try_get(subnet_id, &hotkey),
//             Err(())
//         );

//         let epoch_length = EpochLength::get();
//         let min_required_unstake_epochs = StakeCooldownEpochs::<Test>::get();
//         System::set_block_number(
//             System::block_number() + epoch_length * min_required_unstake_epochs,
//         );

//         // remove amount ontop
//         assert_ok!(Network::remove_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             stake_amount / 2,
//         ));

//         assert_ok!(Network::remove_stake(
//             RuntimeOrigin::signed(coldkey.clone()),
//             subnet_id,
//             hotkey.clone(),
//             stake_amount / 2,
//         ));

//         assert_eq!(Network::account_subnet_stake(hotkey.clone(), subnet_id), 0);
//     });
// }

// #[test]
// fn test_subnet_node_try_removing_all_stake_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;
//         let amount: u128 = 1000000000000000000000;

//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//         let max_subnets = MaxSubnets::<Test>::get();

//         let end = 3;

//         let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
//         let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

//         let min_stake_epochs = MinActiveNodeStakeEpochs::<Test>::get();
//         increase_epochs(min_stake_epochs + 2);

//         assert_err!(
//             Network::remove_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 stake_amount,
//             ),
//             Error::<Test>::MinStakeNotReached
//         );
//     })
// }

// #[test]
// fn test_register_try_removing_all_stake_error() {
//     new_test_ext().execute_with(|| {
//         let subnet_name: Vec<u8> = "subnet-name".into();

//         let deposit_amount: u128 = 10000000000000000000000;

//         let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();
//         let end = 12;

//         let subnets = TotalActiveSubnets::<Test>::get() + 1;
//         let max_subnets = MaxSubnets::<Test>::get();
//         let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

//         build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

//         let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//         let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

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
//             peer_id,
//             bootnode_peer_id,
//             client_peer_id,
//             None,
//             0,
//             stake_amount,
//             None,
//             None,
// u128::MAX
//         ));

//         assert_err!(
//             Network::remove_stake(
//                 RuntimeOrigin::signed(coldkey.clone()),
//                 subnet_id,
//                 hotkey.clone(),
//                 stake_amount,
//             ),
//             Error::<Test>::MinStakeNotReached
//         );
//     })
// }
