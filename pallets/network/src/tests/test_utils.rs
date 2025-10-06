use super::mock::*;
use crate::Event;
use crate::{
    AccountOverwatchStake, AccountSubnetDelegateStakeShares, AccountSubnetStake, AttestEntry,
    BootnodePeerIdSubnetNodeId, ColdkeyHotkeys, ColdkeyReputation, ColdkeySubnetNodes,
    ConsensusData, HotkeyOverwatchNodeId, HotkeyOwner, HotkeySubnetId, HotkeySubnetNodeId, KeyType,
    MaxMaxRegisteredNodes, MaxOverwatchNodes, MaxSubnetMaxStake, MaxSubnetNodes, MaxSubnets,
    MinSubnetNodes, MinSubnetRegistrationEpochs, NetworkMaxStakeBalance, NetworkMinStakeBalance,
    OverwatchCommitCutoffPercent, OverwatchEpochLengthMultiplier, OverwatchMinAge,
    OverwatchMinStakeBalance, OverwatchNode, OverwatchNodeIdHotkey, OverwatchNodes,
    OverwatchReveals, PeerIdSubnetNodeId, RegisteredSubnetNodesData, RegistrationSubnetData,
    Reputation, StakeCooldownEpochs, StakeUnbondingLedger, SubnetConsensusSubmission, SubnetData,
    SubnetElectedValidator, SubnetMaxStakeBalance, SubnetMinStakeBalance, SubnetName, SubnetNode,
    SubnetNodeClass, SubnetNodeClassification, SubnetNodeConsensusData, SubnetNodeElectionSlots,
    SubnetNodeIdHotkey, SubnetNodePenalties, SubnetNodeUniqueParam, SubnetNodesData, SubnetOwner,
    SubnetPenaltyCount, SubnetRegistrationEpoch, SubnetRegistrationEpochs,
    SubnetRegistrationInitialColdkeys, SubnetSlot, SubnetState, SubnetsData, TotalActiveNodes,
    TotalActiveSubnetNodes, TotalActiveSubnets, TotalOverwatchNodeUids, TotalOverwatchNodes,
    TotalOverwatchStake, TotalStake, TotalSubnetDelegateStakeBalance, TotalSubnetNodeUids,
    TotalSubnetNodes, TotalSubnetStake, TotalSubnetUids,
};
use fp_account::AccountId20;
use frame_support::assert_ok;
use frame_support::storage::bounded_vec::BoundedVec;
use frame_support::traits::{Currency, ExistenceRequirement};
use sp_core::keccak_256;
use sp_core::OpaquePeerId as PeerId;
use sp_core::H160;
use sp_io::hashing::blake2_128;
use sp_runtime::traits::Hash;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;

pub type AccountIdOf<Test> = <Test as frame_system::Config>::AccountId;
pub const PERCENTAGE_FACTOR: u128 = 1000000000000000000_u128;
pub const DEFAULT_SCORE: u128 = 500000000000000000;
pub const MAX_SUBNET_NODES: u32 = 254;
pub const DEFAULT_REGISTRATION_BLOCKS: u32 = 130_000;
pub const DEFAULT_DELEGATE_REWARD_RATE: u128 = 100000000000000000; // 10%
pub const ALICE_EXPECTED_BALANCE: u128 = 1000000000000000000000000; // 1,000,000

// pub fn account(id: u32) -> AccountIdOf<Test> {
// 	[id as u8; 32].into()
// }

// Substrate bytes32 H256
// pub fn account(id: u32) -> AccountIdOf<Test> {
//     let mut bytes = [0u8; 32];
//     bytes[0..4].copy_from_slice(&id.to_le_bytes());
//     bytes.into()
// }

// pub fn account(id: u32) -> AccountIdOf<Test> {
//     let mut bytes = [0u8; 20];
//     bytes[0..4].copy_from_slice(&id.to_le_bytes());
//     bytes.into()
// }

// Substrate bytes20 H160
// pub fn account(id: u32) -> AccountIdOf<Test> {
//     let mut bytes = [0u8; 20];
//     bytes[16..20].copy_from_slice(&id.to_le_bytes());
//     bytes.into()
// }
// pub fn account(id: u32) -> H160 {
//     let mut bytes = [0u8; 20];
//     // Write the id into the last 4 bytes
//     bytes[16..20].copy_from_slice(&id.to_le_bytes());
//     H160::from(bytes)
// }
// pub fn account(id: u32) -> AccountIdOf<Test> {
//     let mut bytes = [0u8; 20];
//     bytes[16..20].copy_from_slice(&id.to_le_bytes());
//     AccountIdOf::<Test>::from(H160::from(bytes))
// }
// pub fn account(id: u32) -> AccountIdOf<Test> {
//     let mut bytes = [0u8; 20];
//     // Fill the first 4 bytes instead of just the last
//     bytes[0..4].copy_from_slice(&id.to_le_bytes());
//     // AccountIdOf::<Test>::from(bytes)
//     AccountId20::from(H160::from(bytes))
// }
pub fn account(id: u32) -> AccountIdOf<Test> {
    let hash = keccak_256(&id.to_le_bytes());
    AccountId20::from(H160::from_slice(&hash[0..20]))
}

pub fn get_alice() -> AccountIdOf<Test> {
    let alice = &account(0);
    let block_number = System::block_number();
    let cost = Network::get_current_registration_cost(block_number);
    if Balances::free_balance(alice) < cost {
        let _ = Balances::deposit_creating(&alice, cost + 500);
    }
    alice.clone()
}

pub fn get_coldkey_n(subnets: u32, max_subnet_nodes: u32, n: u32) -> u32 {
    subnets * max_subnet_nodes + n
}

pub fn get_coldkey(subnets: u32, max_subnet_nodes: u32, n: u32) -> AccountIdOf<Test> {
    account(get_coldkey_n(subnets, max_subnet_nodes, n))
}

pub fn get_hotkey_n(subnets: u32, max_subnet_nodes: u32, max_subnets: u32, n: u32) -> u32 {
    max_subnets * max_subnet_nodes + (subnets * max_subnet_nodes) + n
}

pub fn get_hotkey(
    subnets: u32,
    max_subnet_nodes: u32,
    max_subnets: u32,
    n: u32,
) -> AccountIdOf<Test> {
    account(get_hotkey_n(subnets, max_subnet_nodes, max_subnets, n))
}

pub fn get_peer_id(subnets: u32, max_subnet_nodes: u32, max_subnets: u32, n: u32) -> PeerId {
    peer(max_subnets * max_subnet_nodes + (subnets * max_subnet_nodes) + n)
}

pub fn get_bootnode_peer_id(
    subnets: u32,
    max_subnet_nodes: u32,
    max_subnets: u32,
    n: u32,
) -> PeerId {
    peer(
        (max_subnets * max_subnet_nodes * 2)
            + (max_subnets * max_subnet_nodes + (subnets * max_subnet_nodes) + n),
    )
}

pub fn get_client_peer_id(subnets: u32, max_subnet_nodes: u32, max_subnets: u32, n: u32) -> PeerId {
    peer(
        (max_subnets * max_subnet_nodes * 3)
            + (max_subnets * max_subnet_nodes + (subnets * max_subnet_nodes) + n),
    )
}

pub fn get_overwatch_coldkey(
    max_subnet_nodes: u32,
    max_subnets: u32,
    max_onodes: u32,
    n: u32,
) -> AccountIdOf<Test> {
    // account(max_subnet_nodes*max_subnets*max_subnet_nodes+n)
    account(max_subnets * max_subnet_nodes + max_subnets * max_subnet_nodes + n)
}

pub fn get_overwatch_hotkey(
    max_subnet_nodes: u32,
    max_subnets: u32,
    max_onodes: u32,
    n: u32,
) -> AccountIdOf<Test> {
    // account(max_subnets*max_subnets*max_subnet_nodes+max_subnets+max_subnet_nodes+n)
    account(max_subnets * max_subnet_nodes + max_subnets * max_subnet_nodes + max_onodes + n)
}

// it is possible to use `use libp2p::PeerId;` with `PeerId::random()`
// https://github.com/paritytech/substrate/blob/033d4e86cc7eff0066cd376b9375f815761d653c/frame/node-authorization/src/mock.rs#L90
pub fn peer(id: u32) -> PeerId {
    // let peer_id = format!("12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA{id}");
    let peer_id = format!("QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N{id}");
    PeerId(peer_id.into())
}
// bafzbeie5745rpv2m6tjyuugywy4d5ewrqgqqhfnf445he3omzpjbx5xqxe
// QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N
// 12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA

pub fn get_min_stake_balance() -> u128 {
    NetworkMinStakeBalance::<Test>::get()
}

pub fn get_min_overwatch_stake_balance() -> u128 {
    OverwatchMinStakeBalance::<Test>::get()
}

pub fn make_commit(weight: u128, salt: Vec<u8>) -> sp_core::H256 {
    Hashing::hash_of(&(weight, salt))
}

pub fn build_activated_subnet_new(
    subnet_name: Vec<u8>,
    start: u32,
    mut end: u32,
    deposit_amount: u128,
    amount: u128,
) {
    let alice = account(0);
    if Balances::free_balance(alice) == 0 {
        let _ = Balances::deposit_creating(&alice.clone(), ALICE_EXPECTED_BALANCE);
    }

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let owner_coldkey = account(subnets * max_subnets * max_subnet_nodes);
    let owner_hotkey = account(subnets * max_subnets * max_subnet_nodes + 1);

    let cost = Network::get_current_registration_cost(block_number);
    let alice_balance = Balances::free_balance(&alice.clone());
    if alice_balance < cost {
        let _ = Balances::deposit_creating(&alice.clone(), cost + 500);
    }
    assert_ok!(Balances::transfer(
        &alice.clone(),
        &owner_coldkey.clone(),
        cost + 500,
        ExistenceRequirement::KeepAlive,
    ));

    let min_nodes = MinSubnetNodes::<Test>::get();

    if end == 0 {
        end = min_nodes;
    }

    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
        subnets,
        max_subnet_nodes,
        subnet_name.clone().into(),
        start,
        end,
    );

    // --- Register subnet for activation
    assert_ok!(Network::register_subnet(
        RuntimeOrigin::signed(owner_coldkey.clone()),
        owner_hotkey.clone(),
        100000000000000000000000,
        add_subnet_data,
    ));

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    assert!(subnet_id > 0);
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Registered);
    let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
    assert_eq!(owner, owner_coldkey.clone());

    // let min_stake = SubnetMinStakeBalance::<Test>::get(subnet_id);
    // let max_stake = SubnetMaxStakeBalance::<Test>::get(subnet_id);
    // assert_eq!(min_stake, add_subnet_data.min_stake);
    // assert_eq!(max_stake, add_subnet_data.max_stake);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    // --- Add subnet nodes
    let block_number = System::block_number();
    let mut amount_staked = 0;
    let burn_amount = Network::calculate_burn_amount(subnet_id);
    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);

        if Balances::free_balance(&alice.clone()) <= amount {
            let _ = Balances::deposit_creating(&alice.clone(), amount + 500);
        }

        assert_ok!(Balances::transfer(
            &alice.clone(), // alice
            &coldkey.clone(),
            amount + burn_amount + 500,
            ExistenceRequirement::KeepAlive,
        ));
        amount_staked += amount;
        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        // assert!(false);
        // assert_eq!(
        //   *network_events().last().unwrap(),
        //   Event::SubnetNodeActivated {
        //     subnet_id: subnet_id,
        //     subnet_node_id: n
        //   }
        // );

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
        // assert_eq!(hotkey_subnet_node_id, coldkey_n);

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        // Is activated, registered element is removed
        assert_eq!(
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );

        let subnet_node_data =
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());

        let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        assert_eq!(subnet_node_data.peer_id, peer_id.clone());

        // --- Is ``Validator`` if registered before subnet activation
        assert_eq!(
            subnet_node_data.classification.node_class,
            SubnetNodeClass::Validator
        );
        assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, subnet_epoch));

        let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
        assert_eq!(subnet_node_account, hotkey_subnet_node_id);

        let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, amount);

        let hotkey_subnet_id = HotkeySubnetId::<Test>::get(hotkey.clone());
        assert_eq!(hotkey_subnet_id, Some(subnet_id));

        let mut is_electable = false;
        for node_id in SubnetNodeElectionSlots::<Test>::get(subnet_id).iter() {
            if *node_id == hotkey_subnet_node_id {
                is_electable = true;
            }
        }
        assert!(is_electable);

        let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
        assert!(coldkey_subnet_nodes
            .get(&subnet_id)
            .unwrap()
            .contains(&hotkey_subnet_node_id))
    }

    let total_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(total_nodes, end);

    let slot_list = SubnetNodeElectionSlots::<Test>::get(subnet_id);
    assert_eq!(slot_list.len(), total_nodes as usize);

    let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<Test>::get();
    assert_eq!(total_subnet_stake, amount_staked);

    let delegate_staker_account = 1;
    // Add 100e18 to account for block increase on activation
    let mut min_subnet_delegate_stake =
        Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
    min_subnet_delegate_stake = min_subnet_delegate_stake
        + Network::percent_mul(min_subnet_delegate_stake, 10000000000000000);

    if Balances::free_balance(&alice.clone()) <= min_subnet_delegate_stake {
        let _ = Balances::deposit_creating(&alice, min_subnet_delegate_stake + 500);
    }

    assert_ok!(Balances::transfer(
        &alice.clone(), // alice
        &account(delegate_staker_account),
        min_subnet_delegate_stake + 500,
        ExistenceRequirement::KeepAlive,
    ));

    assert_ne!(min_subnet_delegate_stake, u128::MAX);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(delegate_staker_account)),
        subnet_id,
        min_subnet_delegate_stake,
    ));

    let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

    let min_registration_epochs = MinSubnetRegistrationEpochs::<Test>::get();
    increase_epochs(min_registration_epochs + 1);

    assert_ok!(Network::activate_subnet(
        RuntimeOrigin::signed(owner_coldkey.clone()),
        subnet_id,
    ));

    assert_eq!(
        *network_events().last().unwrap(),
        Event::SubnetActivated {
            subnet_id: subnet_id,
        }
    );

    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Active);

    // increase_epochs(2);
}

pub fn build_activated_subnet_new_excess_subnets(
    subnet_name: Vec<u8>,
    start: u32,
    mut end: u32,
    deposit_amount: u128,
    amount: u128,
    excess: u32,
) {
    let alice = account(0);
    if Balances::free_balance(alice) == 0 {
        let _ = Balances::deposit_creating(&alice.clone(), ALICE_EXPECTED_BALANCE);
    }

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnets = MaxSubnets::<Test>::get().saturating_add(excess);
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let owner_coldkey = account(subnets * max_subnets * max_subnet_nodes);
    let owner_hotkey = account(subnets * max_subnets * max_subnet_nodes + 1);

    let cost = Network::get_current_registration_cost(block_number);

    let alice_balance = Balances::free_balance(&alice.clone());
    if alice_balance < cost {
        let _ = Balances::deposit_creating(&alice.clone(), cost + 500);
    }
    assert_ok!(Balances::transfer(
        &alice.clone(),
        &owner_coldkey.clone(),
        cost + 500,
        ExistenceRequirement::KeepAlive,
    ));

    let min_nodes = MinSubnetNodes::<Test>::get();

    if end == 0 {
        end = min_nodes;
    }

    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
        subnets,
        max_subnet_nodes,
        subnet_name.clone().into(),
        start,
        end,
    );

    // --- Register subnet for activation
    assert_ok!(Network::register_subnet(
        RuntimeOrigin::signed(owner_coldkey.clone()),
        owner_hotkey.clone(),
        100000000000000000000000,
        add_subnet_data,
    ));

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

    assert_eq!(subnet.state, SubnetState::Registered);
    let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
    assert_eq!(owner, owner_coldkey.clone());

    // let min_stake = SubnetMinStakeBalance::<Test>::get(subnet_id);
    // let max_stake = SubnetMaxStakeBalance::<Test>::get(subnet_id);
    // assert_eq!(min_stake, add_subnet_data.min_stake);
    // assert_eq!(max_stake, add_subnet_data.max_stake);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    // --- Add subnet nodes
    let block_number = System::block_number();
    let mut amount_staked = 0;
    let burn_amount = Network::calculate_burn_amount(subnet_id);
    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
        let coldkey_n = get_coldkey_n(subnets, max_subnet_nodes, _n);
        let hotkey_n = get_hotkey_n(subnets, max_subnet_nodes, max_subnets, _n);

        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);

        if Balances::free_balance(&alice.clone()) <= amount {
            let _ = Balances::deposit_creating(&alice.clone(), amount + 500);
        }

        assert_ok!(Balances::transfer(
            &alice.clone(), // alice
            &coldkey.clone(),
            amount + burn_amount + 500,
            ExistenceRequirement::KeepAlive,
        ));
        amount_staked += amount;
        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
        // assert_eq!(hotkey_subnet_node_id, coldkey_n);

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        // Is activated, registered element is removed
        assert_eq!(
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );

        let subnet_node_data =
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());

        let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        assert_eq!(subnet_node_data.peer_id, peer_id.clone());

        // --- Is ``Validator`` if registered before subnet activation
        assert_eq!(
            subnet_node_data.classification.node_class,
            SubnetNodeClass::Validator
        );
        assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, subnet_epoch));

        let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
        assert_eq!(subnet_node_account, hotkey_subnet_node_id);

        let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, amount);

        let hotkey_subnet_id = HotkeySubnetId::<Test>::get(hotkey.clone());
        assert_eq!(hotkey_subnet_id, Some(subnet_id));

        let mut is_electable = false;
        for node_id in SubnetNodeElectionSlots::<Test>::get(subnet_id).iter() {
            if *node_id == hotkey_subnet_node_id {
                is_electable = true;
            }
        }
        assert!(is_electable);

        let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
        assert!(coldkey_subnet_nodes
            .get(&subnet_id)
            .unwrap()
            .contains(&hotkey_subnet_node_id))
    }

    let total_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(total_nodes, end);

    let slot_list = SubnetNodeElectionSlots::<Test>::get(subnet_id);
    assert_eq!(slot_list.len(), total_nodes as usize);

    let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<Test>::get();
    assert_eq!(total_subnet_stake, amount_staked);

    let delegate_staker_account = 1;
    // Add 100e18 to account for block increase on activation
    let mut min_subnet_delegate_stake =
        Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
    min_subnet_delegate_stake = min_subnet_delegate_stake
        + Network::percent_mul(min_subnet_delegate_stake, 10000000000000000);

    if Balances::free_balance(&alice.clone()) <= min_subnet_delegate_stake {
        let _ = Balances::deposit_creating(&alice, min_subnet_delegate_stake + 500);
    }

    assert_ok!(Balances::transfer(
        &alice.clone(), // alice
        &account(delegate_staker_account),
        min_subnet_delegate_stake + 500,
        ExistenceRequirement::KeepAlive,
    ));

    assert_ne!(min_subnet_delegate_stake, u128::MAX);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(delegate_staker_account)),
        subnet_id,
        min_subnet_delegate_stake,
    ));

    let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

    let min_registration_epochs = MinSubnetRegistrationEpochs::<Test>::get();
    increase_epochs(min_registration_epochs + 1);

    assert_ok!(Network::activate_subnet(
        RuntimeOrigin::signed(owner_coldkey.clone()),
        subnet_id,
    ));

    assert_eq!(
        *network_events().last().unwrap(),
        Event::SubnetActivated {
            subnet_id: subnet_id,
        }
    );

    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Active);
}

pub fn build_registered_subnet_new(
    subnet_name: Vec<u8>,
    start: u32,
    mut end: u32,
    deposit_amount: u128,
    amount: u128,
    delegate_stake_conditional: bool,
) {
    let alice = account(0);
    if Balances::free_balance(alice) == 0 {
        let _ = Balances::deposit_creating(&alice.clone(), ALICE_EXPECTED_BALANCE);
    }

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    // let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    // increase_epochs(next_registration_epoch.saturating_sub(epoch));

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let owner_coldkey = account(subnets * max_subnets * max_subnet_nodes);
    let owner_hotkey = account(subnets * max_subnets * max_subnet_nodes + 1);

    let cost = Network::get_current_registration_cost(block_number);

    let alice_balance = Balances::free_balance(&alice.clone());
    if alice_balance < cost {
        let _ = Balances::deposit_creating(&alice.clone(), cost + 500);
    }
    assert_ok!(Balances::transfer(
        &alice.clone(),
        &owner_coldkey.clone(),
        cost + 500,
        ExistenceRequirement::KeepAlive,
    ));

    let min_nodes = MinSubnetNodes::<Test>::get();
    let mut initial_coldkeys_end = end;
    if end < min_nodes {
        initial_coldkeys_end = min_nodes;
    }

    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
        subnets,
        max_subnet_nodes,
        subnet_name.clone().into(),
        start,
        initial_coldkeys_end,
    );

    // --- Register subnet for activation
    assert_ok!(Network::register_subnet(
        RuntimeOrigin::signed(owner_coldkey.clone()),
        owner_hotkey.clone(),
        100000000000000000000000,
        add_subnet_data,
    ));

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Registered);
    let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
    assert_eq!(owner, owner_coldkey.clone());

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    // --- Add subnet nodes
    let block_number = System::block_number();
    let mut amount_staked = 0;
    let burn_amount = Network::calculate_burn_amount(subnet_id);
    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        if Balances::free_balance(&alice.clone()) <= amount {
            let _ = Balances::deposit_creating(&alice.clone(), amount + 500);
        }

        assert_ok!(Balances::transfer(
            &alice.clone(), // alice
            &coldkey.clone(),
            amount + burn_amount + 500,
            ExistenceRequirement::KeepAlive,
        ));

        amount_staked += amount;
        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        // Is activated, registered element is removed
        assert_eq!(
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );

        let subnet_node_data =
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());

        let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        assert_eq!(subnet_node_data.peer_id, peer_id.clone());

        // --- Is ``Validator`` if registered before subnet activation
        assert_eq!(
            subnet_node_data.classification.node_class,
            SubnetNodeClass::Validator
        );
        assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, subnet_epoch));

        let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
        assert_eq!(subnet_node_account, hotkey_subnet_node_id);

        let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, amount);

        let hotkey_subnet_id = HotkeySubnetId::<Test>::get(hotkey.clone());
        assert_eq!(hotkey_subnet_id, Some(subnet_id));

        let mut is_electable = false;
        for node_id in SubnetNodeElectionSlots::<Test>::get(subnet_id).iter() {
            if *node_id == hotkey_subnet_node_id {
                is_electable = true;
            }
        }
        assert!(is_electable);

        let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
        assert!(coldkey_subnet_nodes
            .get(&subnet_id)
            .unwrap()
            .contains(&hotkey_subnet_node_id))
    }

    let total_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(total_nodes, end);

    let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<Test>::get();
    assert_eq!(total_subnet_stake, amount_staked);

    if delegate_stake_conditional {
        let delegate_staker_account = 1;
        // Add 100e18 to account for block increase on activation
        let mut min_subnet_delegate_stake =
            Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
        min_subnet_delegate_stake = min_subnet_delegate_stake
            + Network::percent_mul(min_subnet_delegate_stake, 10000000000000000);
        if Balances::free_balance(&alice.clone()) <= min_subnet_delegate_stake {
            let _ = Balances::deposit_creating(&alice, min_subnet_delegate_stake + 500);
        }

        assert_ok!(Balances::transfer(
            &alice.clone(), // alice
            &account(delegate_staker_account),
            min_subnet_delegate_stake + 500,
            ExistenceRequirement::KeepAlive,
        ));

        assert_ne!(min_subnet_delegate_stake, u128::MAX);
        // --- Add the minimum required delegate stake balance to activate the subnet
        assert_ok!(Network::add_to_delegate_stake(
            RuntimeOrigin::signed(account(delegate_staker_account)),
            subnet_id,
            min_subnet_delegate_stake,
        ));

        let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);
    }

    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Registered);
}

pub fn build_registered_subnet_and_subnet_nodes(
    subnet_name: Vec<u8>,
    start: u32,
    mut end: u32,
    deposit_amount: u128,
    amount: u128,
    delegate_stake_conditional: bool,
) {
    let alice = 0;
    let alice_balance = Balances::free_balance(&account(alice));
    if alice_balance == 0 {
        let _ = Balances::deposit_creating(&account(alice), ALICE_EXPECTED_BALANCE);
    }

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    // let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    // increase_epochs(next_registration_epoch.saturating_sub(epoch));

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let owner_coldkey = account(subnets * max_subnets * max_subnet_nodes);
    let owner_hotkey = account(subnets * max_subnets * max_subnet_nodes + 1);

    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
    // let _ = Balances::deposit_creating(&owner_coldkey.clone(), cost+500);
    assert_ok!(Balances::transfer(
        &account(0), // alice
        &owner_coldkey.clone(),
        cost + 500,
        ExistenceRequirement::KeepAlive,
    ));

    let min_nodes = MinSubnetNodes::<Test>::get();

    if end == 0 {
        end = min_nodes;
    }

    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
        subnets,
        max_subnet_nodes,
        subnet_name.clone().into(),
        start,
        end,
    );

    // --- Register subnet for activation
    assert_ok!(Network::register_subnet(
        RuntimeOrigin::signed(owner_coldkey.clone()),
        owner_hotkey.clone(),
        100000000000000000000000,
        add_subnet_data,
    ));

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Registered);
    let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
    assert_eq!(owner, owner_coldkey.clone());

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    // --- Add subnet nodes
    let block_number = System::block_number();
    let mut amount_staked = 0;
    let burn_amount = Network::calculate_burn_amount(subnet_id);
    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        assert_ok!(Balances::transfer(
            &account(0), // alice
            &coldkey.clone(),
            amount + burn_amount + 500,
            ExistenceRequirement::KeepAlive,
        ));
        amount_staked += amount;
        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id,
            client_peer_id,
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        // Is activated, registered element is removed
        assert_eq!(
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );

        let subnet_node_data =
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());

        let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        assert_eq!(subnet_node_data.peer_id, peer_id.clone());

        // --- Is ``Validator`` if registered before subnet activation
        assert_eq!(
            subnet_node_data.classification.node_class,
            SubnetNodeClass::Registered
        );
        // use 0 because registered nodes are in queue, just check it's working
        // assert!(subnet_node_data.has_classification(&SubnetNodeClass::Registered, 0));

        let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
        assert_eq!(subnet_node_account, hotkey_subnet_node_id);

        let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, amount);

        let hotkey_subnet_id = HotkeySubnetId::<Test>::get(hotkey.clone());
        assert_eq!(hotkey_subnet_id, Some(subnet_id));

        let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
        assert!(coldkey_subnet_nodes
            .get(&subnet_id)
            .unwrap()
            .contains(&hotkey_subnet_node_id))
    }

    // let total_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
    let total_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(total_nodes, end);

    let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<Test>::get();
    assert_eq!(total_subnet_stake, amount_staked);

    if delegate_stake_conditional {
        let delegate_staker_account = 1;
        // Add 100e18 to account for block increase on activation
        let mut min_subnet_delegate_stake =
            Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
        min_subnet_delegate_stake = min_subnet_delegate_stake
            + Network::percent_mul(min_subnet_delegate_stake, 10000000000000000);
        assert_ok!(Balances::transfer(
            &account(0), // alice
            &account(delegate_staker_account),
            min_subnet_delegate_stake + 500,
            ExistenceRequirement::KeepAlive,
        ));

        assert_ne!(min_subnet_delegate_stake, u128::MAX);
        // --- Add the minimum required delegate stake balance to activate the subnet
        assert_ok!(Network::add_to_delegate_stake(
            RuntimeOrigin::signed(account(delegate_staker_account)),
            subnet_id,
            min_subnet_delegate_stake,
        ));

        let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);
    }

    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Registered);
}

pub fn build_registered_nodes_in_queue(
    subnet_id: u32,
    start: u32,
    mut end: u32,
    deposit_amount: u128,
    amount: u128,
) {
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    let mut amount_staked = 0;
    let burn_amount = Network::calculate_burn_amount(subnet_id);
    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey(subnet_id, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
        let peer_id = get_peer_id(subnet_id, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id = get_bootnode_peer_id(subnet_id, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id(subnet_id, max_subnet_nodes, max_subnets, _n);
        assert_ok!(Balances::transfer(
            &account(0), // alice
            &coldkey.clone(),
            amount + burn_amount + 500,
            ExistenceRequirement::KeepAlive,
        ));
        amount_staked += amount;
        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id,
            client_peer_id,
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        // Is activated, registered element is removed
        assert_eq!(
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );

        let subnet_node_data =
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());

        let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        assert_eq!(subnet_node_data.peer_id, peer_id.clone());

        // --- Is ``Registered`` if registered before subnet activation
        assert_eq!(
            subnet_node_data.classification.node_class,
            SubnetNodeClass::Registered
        );
        assert!(subnet_node_data.has_classification(&SubnetNodeClass::Registered, u32::MAX));
        // Check other classifications
        assert!(!subnet_node_data.has_classification(&SubnetNodeClass::Validator, u32::MAX));
        assert!(!subnet_node_data.has_classification(&SubnetNodeClass::Idle, u32::MAX));
        assert!(!subnet_node_data.has_classification(&SubnetNodeClass::Included, u32::MAX));

        let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
        assert_eq!(subnet_node_account, hotkey_subnet_node_id);

        let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, amount);

        let hotkey_subnet_id = HotkeySubnetId::<Test>::get(hotkey.clone());
        assert_eq!(hotkey_subnet_id, Some(subnet_id));

        let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
        assert!(coldkey_subnet_nodes
            .get(&subnet_id)
            .unwrap()
            .contains(&hotkey_subnet_node_id))
    }
}

pub fn build_activated_subnet_with_overwatch_nodes(
    subnet_name: Vec<u8>,
    start: u32,
    mut end: u32,
    overwatch_count: u32,
    deposit_amount: u128,
    amount: u128,
) {
    let alice = 0;
    let alice_balance = Balances::free_balance(&account(alice));
    if alice_balance == 0 {
        let _ = Balances::deposit_creating(&account(alice), ALICE_EXPECTED_BALANCE);
    }

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    // let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    // increase_epochs(next_registration_epoch.saturating_sub(epoch));

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    assert!(end - start + overwatch_count <= max_subnet_nodes);

    let owner_coldkey = account(subnets * max_subnets * max_subnet_nodes);
    let owner_hotkey = account(subnets * max_subnets * max_subnet_nodes + 1);

    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
    // let _ = Balances::deposit_creating(&owner_coldkey.clone(), cost+500);
    assert_ok!(Balances::transfer(
        &account(0), // alice
        &owner_coldkey.clone(),
        cost + 500,
        ExistenceRequirement::KeepAlive,
    ));

    let min_nodes = MinSubnetNodes::<Test>::get();

    if end == 0 {
        end = min_nodes;
    }

    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
        subnets,
        max_subnet_nodes,
        subnet_name.clone().into(),
        start,
        end + overwatch_count,
    );

    // --- Register subnet for activation
    assert_ok!(Network::register_subnet(
        RuntimeOrigin::signed(owner_coldkey.clone()),
        owner_hotkey.clone(),
        100000000000000000000000,
        add_subnet_data,
    ));

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Registered);
    let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
    assert_eq!(owner, owner_coldkey.clone());

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    // --- Add subnet nodes
    let block_number = System::block_number();
    let mut amount_staked = 0;
    let burn_amount = Network::calculate_burn_amount(subnet_id);
    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        assert_ok!(Balances::transfer(
            &account(0), // alice
            &coldkey.clone(),
            amount + burn_amount + 500,
            ExistenceRequirement::KeepAlive,
        ));
        amount_staked += amount;
        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
        // assert_eq!(hotkey_subnet_node_id, coldkey_n);

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        // Is activated, registered element is removed
        assert_eq!(
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );

        let subnet_node_data =
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());

        let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        assert_eq!(subnet_node_data.peer_id, peer_id.clone());

        // --- Is ``Validator`` if registered before subnet activation
        assert_eq!(
            subnet_node_data.classification.node_class,
            SubnetNodeClass::Validator
        );
        assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, subnet_epoch));

        let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
        assert_eq!(subnet_node_account, hotkey_subnet_node_id);

        let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, amount);

        let hotkey_subnet_id = HotkeySubnetId::<Test>::get(hotkey.clone());
        assert_eq!(hotkey_subnet_id, Some(subnet_id));

        let mut is_electable = false;
        for node_id in SubnetNodeElectionSlots::<Test>::get(subnet_id).iter() {
            if *node_id == hotkey_subnet_node_id {
                is_electable = true;
            }
        }
        assert!(is_electable);

        let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
        assert!(coldkey_subnet_nodes
            .get(&subnet_id)
            .unwrap()
            .contains(&hotkey_subnet_node_id))
    }

    let max_onodes = MaxOverwatchNodes::<Test>::get();
    let burn_amount = Network::calculate_burn_amount(subnet_id);
    for n in end..end + overwatch_count {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        assert_ok!(Balances::transfer(
            &account(0), // alice
            &coldkey.clone(),
            amount + burn_amount + 500,
            ExistenceRequirement::KeepAlive,
        ));
        amount_staked += amount;
        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        // max reputation
        ColdkeyReputation::<Test>::insert(
            coldkey.clone(),
            Reputation {
                start_epoch: 0,
                score: 1_000_000_000_000_000_000,
                lifetime_node_count: 0,
                total_active_nodes: 0,
                total_increases: 0,
                total_decreases: 0,
                average_attestation: 1_000_000_000_000_000_000,
                last_validator_epoch: 0,
                ow_score: 1_000_000_000_000_000_000,
            },
        );

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        // Is activated, registered element is removed
        assert_eq!(
            RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
            Err(())
        );

        let subnet_node_data =
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());

        let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        assert_eq!(subnet_node_data.peer_id, peer_id.clone());

        // --- Is ``Validator`` if registered before subnet activation
        assert_eq!(
            subnet_node_data.classification.node_class,
            SubnetNodeClass::Validator
        );
        assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, subnet_epoch));

        let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
        assert_eq!(subnet_node_account, hotkey_subnet_node_id);

        let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, amount);

        let hotkey_subnet_id = HotkeySubnetId::<Test>::get(hotkey.clone());
        assert_eq!(hotkey_subnet_id, Some(subnet_id));

        let mut is_electable = false;
        for node_id in SubnetNodeElectionSlots::<Test>::get(subnet_id).iter() {
            if *node_id == hotkey_subnet_node_id {
                is_electable = true;
            }
        }
        assert!(is_electable);

        let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
        assert!(coldkey_subnet_nodes
            .get(&subnet_id)
            .unwrap()
            .contains(&hotkey_subnet_node_id));
    }

    let min_age = OverwatchMinAge::<Test>::get();
    increase_epochs(min_age + 1);
    for n in end..end + overwatch_count {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);

        // Add overwatch node
        assert_ok!(Balances::transfer(
            &account(0), // alice
            &coldkey.clone(),
            amount,
            ExistenceRequirement::KeepAlive,
        ));
        let overwatch_hotkey = get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);
        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            overwatch_hotkey.clone(),
            amount,
        ));
    }

    let total_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(total_nodes, end - start + overwatch_count);

    let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<Test>::get();
    assert_eq!(total_subnet_stake, amount_staked);

    // --- Increase epochs to max registration epoch
    // let epochs = SubnetRegistrationEpochs::<Test>::get();
    // increase_epochs(epochs + 1);

    let delegate_staker_account = 1;
    // Add 100e18 to account for block increase on activation
    let mut min_subnet_delegate_stake =
        Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
    min_subnet_delegate_stake = min_subnet_delegate_stake
        + Network::percent_mul(min_subnet_delegate_stake, 10000000000000000);
    // let _ = Balances::deposit_creating(&account(delegate_staker_account), min_subnet_delegate_stake+500);
    assert_ok!(Balances::transfer(
        &account(0), // alice
        &account(delegate_staker_account),
        min_subnet_delegate_stake + 500,
        ExistenceRequirement::KeepAlive,
    ));

    assert_ne!(min_subnet_delegate_stake, u128::MAX);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(delegate_staker_account)),
        subnet_id,
        min_subnet_delegate_stake,
    ));

    let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);

    let min_registration_epochs = MinSubnetRegistrationEpochs::<Test>::get();
    increase_epochs(min_registration_epochs + 1);

    assert_ok!(Network::activate_subnet(
        RuntimeOrigin::signed(owner_coldkey.clone()),
        subnet_id,
    ));

    assert_eq!(
        *network_events().last().unwrap(),
        Event::SubnetActivated {
            subnet_id: subnet_id,
        }
    );

    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Active);

    increase_epochs(2);
}

pub fn build_activated_subnet_with_overwatch_nodes_v2(
    start: u32,
    mut end: u32,
    overwatch_count: u32,
    deposit_amount: u128,
    amount: u128,
) {
    let alice = 0;
    let alice_balance = Balances::free_balance(&account(alice));
    if alice_balance == 0 {
        let _ = Balances::deposit_creating(&account(alice), ALICE_EXPECTED_BALANCE);
    }

    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_onodes = MaxOverwatchNodes::<Test>::get();

    for s in 0..max_subnets {
        let epoch_length = EpochLength::get();
        let block_number = System::block_number();
        let epoch = System::block_number().saturating_div(epoch_length);

        let subnets = TotalActiveSubnets::<Test>::get() + 1;

        assert!(end - start + overwatch_count <= max_subnet_nodes);

        let owner_coldkey = account(subnets * max_subnets * max_subnet_nodes);
        let owner_hotkey = account(subnets * max_subnets * max_subnet_nodes + 1);

        let cost = Network::get_current_registration_cost(block_number);
        assert_ok!(Balances::transfer(
            &account(0), // alice
            &owner_coldkey.clone(),
            cost + 500,
            ExistenceRequirement::KeepAlive,
        ));

        let min_nodes = MinSubnetNodes::<Test>::get();

        if end == 0 {
            end = min_nodes;
        }

        let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();

        let add_subnet_data: RegistrationSubnetData<AccountId> =
            default_registration_subnet_data_with_onodes(
                subnets,
                max_subnets,
                max_subnet_nodes,
                max_onodes,
                subnet_name.clone(),
                start,
                end,
                overwatch_count,
            );

        // --- Register subnet for activation
        assert_ok!(Network::register_subnet(
            RuntimeOrigin::signed(owner_coldkey.clone()),
            owner_hotkey.clone(),
            100000000000000000000000,
            add_subnet_data,
        ));

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
        assert_eq!(subnet.state, SubnetState::Registered);
        let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
        assert_eq!(owner, owner_coldkey.clone());

        let epoch_length = EpochLength::get();
        let epoch = System::block_number() / epoch_length;

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // --- Add subnet nodes
        let block_number = System::block_number();
        let mut amount_staked = 0;
        let burn_amount = Network::calculate_burn_amount(subnet_id);
        for n in start..end + overwatch_count {
            let _n = n + 1;

            let mut coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
            let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);

            if _n >= end {
                let o_n = _n - end + 1;
                coldkey = get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
            }

            let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
            let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);

            assert_ok!(Balances::transfer(
                &account(0), // alice
                &coldkey.clone(),
                amount + burn_amount + 500,
                ExistenceRequirement::KeepAlive,
            ));
            amount_staked += amount;
            let prev_coldkey_reputation = ColdkeyReputation::<Test>::get(coldkey.clone());
            assert_ok!(Network::register_subnet_node(
                RuntimeOrigin::signed(coldkey.clone()),
                subnet_id,
                hotkey.clone(),
                peer_id.clone(),
                bootnode_peer_id.clone(),
                client_peer_id.clone(),
                None,
                0,
                amount,
                None,
                None,
                u128::MAX
            ));

            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

            let subnet_node_id_hotkey =
                SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
            assert_eq!(subnet_node_id_hotkey, hotkey.clone());

            // Is activated, registered element is removed
            assert_eq!(
                RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id),
                Err(())
            );

            let subnet_node_data =
                SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
            assert_eq!(subnet_node_data.hotkey, hotkey.clone());

            let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
            assert_eq!(key_owner, coldkey.clone());

            assert_eq!(subnet_node_data.peer_id, peer_id.clone());

            // --- Is ``Validator`` if registered before subnet activation
            assert_eq!(
                subnet_node_data.classification.node_class,
                SubnetNodeClass::Validator
            );
            assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, subnet_epoch));

            let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
            assert_eq!(subnet_node_account, hotkey_subnet_node_id);

            let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
            assert_eq!(account_subnet_stake, amount);

            let hotkey_subnet_id = HotkeySubnetId::<Test>::get(hotkey.clone());
            assert_eq!(hotkey_subnet_id, Some(subnet_id));

            let mut is_electable = false;
            for node_id in SubnetNodeElectionSlots::<Test>::get(subnet_id).iter() {
                if *node_id == hotkey_subnet_node_id {
                    is_electable = true;
                }
            }
            assert!(is_electable);

            let coldkey_subnet_nodes = ColdkeySubnetNodes::<Test>::get(coldkey.clone());
            assert!(coldkey_subnet_nodes
                .get(&subnet_id)
                .unwrap()
                .contains(&hotkey_subnet_node_id));

            let coldkey_reputation = ColdkeyReputation::<Test>::get(coldkey.clone());
            assert_eq!(
                coldkey_reputation.total_active_nodes,
                prev_coldkey_reputation.total_active_nodes + 1
            );
            assert_eq!(
                coldkey_reputation.lifetime_node_count,
                prev_coldkey_reputation.lifetime_node_count + 1
            );
        }

        let total_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
        assert_eq!(total_nodes, end - start + overwatch_count);

        let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
        assert_eq!(total_subnet_stake, amount_staked);

        let total_stake = TotalStake::<Test>::get();
        assert_eq!(total_subnet_stake, amount_staked);

        // --- Increase epochs to max registration epoch
        let epochs = SubnetRegistrationEpochs::<Test>::get();
        increase_epochs(epochs + 1);

        let delegate_staker_account = 1;
        // Add 100e18 to account for block increase on activation
        let mut min_subnet_delegate_stake =
            Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
        min_subnet_delegate_stake = min_subnet_delegate_stake
            + Network::percent_mul(min_subnet_delegate_stake, 10000000000000000);
        // let _ = Balances::deposit_creating(&account(delegate_staker_account), min_subnet_delegate_stake+500);
        assert_ok!(Balances::transfer(
            &account(0), // alice
            &account(delegate_staker_account),
            min_subnet_delegate_stake + 500,
            ExistenceRequirement::KeepAlive,
        ));

        assert_ne!(min_subnet_delegate_stake, u128::MAX);
        // --- Add the minimum required delegate stake balance to activate the subnet
        assert_ok!(Network::add_to_delegate_stake(
            RuntimeOrigin::signed(account(delegate_staker_account)),
            subnet_id,
            min_subnet_delegate_stake,
        ));

        let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

        let min_subnet_delegate_stake =
            Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);

        assert_ok!(Network::activate_subnet(
            RuntimeOrigin::signed(owner_coldkey.clone()),
            subnet_id,
        ));

        assert_eq!(
            *network_events().last().unwrap(),
            Event::SubnetActivated {
                subnet_id: subnet_id,
            }
        );

        let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
        assert_eq!(subnet.state, SubnetState::Active);
    }

    let min_age = OverwatchMinAge::<Test>::get();
    increase_epochs(min_age + 1);
    for n in end - 1..end + overwatch_count {
        let _n = n + 1;
        let o_n = _n - end + 1;
        let coldkey = get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
        let hotkey = get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);
        // Force max reputation
        ColdkeyReputation::<Test>::insert(
            coldkey.clone(),
            Reputation {
                start_epoch: 0,
                score: 1_000_000_000_000_000_000,
                lifetime_node_count: 0,
                total_active_nodes: 0,
                total_increases: 0,
                total_decreases: 0,
                average_attestation: 1_000_000_000_000_000_000,
                last_validator_epoch: 0,
                ow_score: 1_000_000_000_000_000_000,
            },
        );

        // Add overwatch node
        assert_ok!(Balances::transfer(
            &account(0), // alice
            &coldkey.clone(),
            amount,
            ExistenceRequirement::KeepAlive,
        ));
        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        // let _n = n + 1;
        // let coldkey = get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, _n);
        // let hotkey = get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);

        // // Force max reputation
        // ColdkeyReputation::<Test>::insert(coldkey.clone(), Reputation {
        //   start_epoch: 0,
        //   score: 1_000_000_000_000_000_000,
        //   lifetime_node_count: 0,
        //   total_active_nodes: 0,
        //   total_increases: 0,
        //   total_decreases: 0,
        //   average_attestation: 1_000_000_000_000_000_000,
        //   last_validator_epoch: 0,
        //   ow_score: 1_000_000_000_000_000_000,
        // });

        // // Add overwatch node
        // assert_ok!(
        //   Balances::transfer(
        //     &account(0), // alice
        //     &coldkey.clone(),
        //     amount,
        //     ExistenceRequirement::KeepAlive,
        //   )
        // );
        // assert_ok!(
        //   Network::register_overwatch_node(
        //     RuntimeOrigin::signed(coldkey.clone()),
        //     hotkey.clone(),
        //     amount,
        //   )
        // );
    }
}

pub fn build_activated_subnet_with_delegator_rewards(
    subnet_name: Vec<u8>,
    start: u32,
    mut end: u32,
    deposit_amount: u128,
    amount: u128,
    delegate_reward_rate: u128,
) {
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    // let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    // increase_epochs(next_registration_epoch.saturating_sub(epoch));

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let owner_coldkey = account(subnets * max_subnets * max_subnet_nodes);
    let owner_hotkey = account(subnets * max_subnets * max_subnet_nodes + 1);

    let cost = Network::get_current_registration_cost(block_number);
    let _ = Balances::deposit_creating(&owner_coldkey.clone(), cost + 1000);

    let min_nodes = MinSubnetNodes::<Test>::get();

    if end == 0 {
        end = min_nodes;
    }

    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
        subnets,
        max_subnet_nodes,
        subnet_name.clone().into(),
        start,
        end,
    );

    // --- Register subnet for activation
    assert_ok!(Network::register_subnet(
        RuntimeOrigin::signed(owner_coldkey.clone()),
        owner_hotkey.clone(),
        100000000000000000000000,
        add_subnet_data,
    ));

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
    assert_eq!(owner, owner_coldkey.clone());

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;
    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    // --- Add subnet nodes
    let block_number = System::block_number();
    let mut amount_staked = 0;
    let burn_amount = Network::calculate_burn_amount(subnet_id);
    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);

        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount + burn_amount);
        amount_staked += amount;
        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
        // assert_eq!(hotkey_subnet_node_id, coldkey_n);

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        let subnet_node_data =
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());

        let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        assert_eq!(subnet_node_data.peer_id, peer_id.clone());

        // --- Is ``Validator`` if registered before subnet activation
        assert_eq!(
            subnet_node_data.classification.node_class,
            SubnetNodeClass::Validator
        );
        assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, subnet_epoch));

        let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
        assert_eq!(subnet_node_account, hotkey_subnet_node_id);

        let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, amount);
    }

    let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<Test>::get();
    assert_eq!(total_subnet_stake, amount_staked);

    let delegate_staker_account = 1000;
    // Add 100e18 to account for block increase on activation
    let min_subnet_delegate_stake =
        Network::get_min_subnet_delegate_stake_balance_v2(subnet_id) + 100e+18 as u128;

    let _ = Balances::deposit_creating(
        &account(delegate_staker_account),
        min_subnet_delegate_stake + 500,
    );
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(delegate_staker_account)),
        subnet_id,
        min_subnet_delegate_stake,
    ));

    let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

    let delegate_shares =
        AccountSubnetDelegateStakeShares::<Test>::get(account(delegate_staker_account), subnet_id);
    // 1000 is for inflation attack mitigation
    // assert_eq!(min_subnet_delegate_stake - 1000, delegate_shares);

    // --- Increase epochs to max registration epoch
    let epochs = SubnetRegistrationEpochs::<Test>::get();
    increase_epochs(epochs + 1);

    assert_ok!(Network::activate_subnet(
        RuntimeOrigin::signed(owner_coldkey.clone()),
        subnet_id,
    ));

    increase_epochs(2);

    assert_eq!(
        *network_events().last().unwrap(),
        Event::SubnetActivated {
            subnet_id: subnet_id,
        }
    );
}

pub fn build_overwatch_nodes(start: u32, mut end: u32, amount: u128) {
    let alice = 0;
    let alice_balance = Balances::free_balance(&account(alice));
    if alice_balance == 0 {
        let _ = Balances::deposit_creating(&account(alice), ALICE_EXPECTED_BALANCE);
    }

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let total_subnet_nodes = TotalActiveNodes::<Test>::get() + 1;
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let max_onodes = MaxOverwatchNodes::<Test>::get();

    let mut total_staked = 0;
    for n in (start + (subnets * total_subnet_nodes))..(end + (subnets * total_subnet_nodes)) {
        total_staked += amount;

        let _n = n + 1;
        let coldkey_n = get_coldkey_n(subnets, max_subnet_nodes, _n);
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);

        assert_ok!(Balances::transfer(
            &account(0), // alice
            &coldkey.clone(),
            amount + 500,
            ExistenceRequirement::KeepAlive,
        ));

        make_overwatch_qualified_sim(coldkey_n);
        assert_ok!(Network::register_overwatch_node(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            amount,
        ));

        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert!(hotkeys.contains(&hotkey.clone()));
        assert_eq!(HotkeyOwner::<Test>::get(hotkey.clone()), coldkey.clone());

        let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();

        assert_eq!(
            OverwatchNodes::<Test>::get(overwatch_node_id)
                .unwrap()
                .hotkey,
            hotkey.clone()
        );
        assert_eq!(
            OverwatchNodeIdHotkey::<Test>::get(overwatch_node_id),
            Some(hotkey.clone())
        );
        assert_eq!(AccountOverwatchStake::<Test>::get(hotkey.clone()), amount);
    }

    let nodes = TotalOverwatchNodes::<Test>::get();
    assert_eq!(nodes, end - start);

    let stake = TotalOverwatchStake::<Test>::get();
    assert_eq!(total_staked, stake);
}

pub fn get_initial_coldkeys(
    subnets: u32,
    max_subnet_nodes: u32,
    start: u32,
    end: u32,
) -> BTreeSet<AccountId> {
    let mut whitelist = BTreeSet::new();
    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        whitelist.insert(coldkey);
    }
    whitelist
}

pub fn get_initial_coldkeys_with_onodes(
    subnets: u32,
    max_subnets: u32,
    max_subnet_nodes: u32,
    max_onodes: u32,
    start: u32,
    end: u32,
    overwatch_count: u32,
) -> BTreeSet<AccountId> {
    let mut whitelist = BTreeSet::new();
    for n in start..end + overwatch_count {
        let _n = n + 1;
        let mut coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        if _n >= end {
            let o_n = _n - end + 1;
            coldkey = get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
        }
        whitelist.insert(coldkey);
    }
    whitelist
}

pub fn default_registration_subnet_data(
    subnets: u32,
    max_subnet_nodes: u32,
    name: Vec<u8>,
    start: u32,
    end: u32,
) -> RegistrationSubnetData<AccountId> {
    let seed_bytes: &[u8] = &name;
    let add_subnet_data = RegistrationSubnetData {
        name: name.clone(),
        repo: blake2_128(seed_bytes).to_vec(), // must be unique
        description: Vec::new(),
        misc: Vec::new(),
        churn_limit: 4,
        min_stake: NetworkMinStakeBalance::<Test>::get(),
        max_stake: NetworkMaxStakeBalance::<Test>::get(),
        delegate_stake_percentage: 100000000000000000, // 10%
        subnet_node_queue_epochs: 4,
        // activation_grace_epochs: 4,
        idle_classification_epochs: 4,
        included_classification_epochs: 4,
        max_node_penalties: 3,
        max_registered_nodes: MaxMaxRegisteredNodes::<Test>::get(),
        initial_coldkeys: get_initial_coldkeys(subnets, max_subnet_nodes, start, end),
        key_types: BTreeSet::from([KeyType::Rsa]),
        bootnodes: BTreeSet::from([BoundedVec::new()]),
    };
    add_subnet_data
}

pub fn default_registration_subnet_data_with_onodes(
    subnets: u32,
    max_subnets: u32,
    max_subnet_nodes: u32,
    max_onodes: u32,
    name: Vec<u8>,
    start: u32,
    end: u32,
    overwatch_count: u32,
) -> RegistrationSubnetData<AccountId> {
    let seed_bytes: &[u8] = &name;
    let add_subnet_data = RegistrationSubnetData {
        name: name.clone(),
        repo: blake2_128(seed_bytes).to_vec(), // must be unique
        description: Vec::new(),
        misc: Vec::new(),
        churn_limit: 4,
        min_stake: NetworkMinStakeBalance::<Test>::get(),
        max_stake: NetworkMaxStakeBalance::<Test>::get(),
        delegate_stake_percentage: 100000000000000000, // 10%
        subnet_node_queue_epochs: 4,
        idle_classification_epochs: 4,
        included_classification_epochs: 4,
        max_node_penalties: 3,
        max_registered_nodes: MaxMaxRegisteredNodes::<Test>::get(),
        initial_coldkeys: get_initial_coldkeys_with_onodes(
            subnets,
            max_subnets,
            max_subnet_nodes,
            max_onodes,
            start,
            end,
            overwatch_count,
        ),
        key_types: BTreeSet::from([KeyType::Rsa]),
        bootnodes: BTreeSet::from([BoundedVec::new()]),
    };
    add_subnet_data
}

pub fn post_subnet_removal_ensures(
    subnet_id: u32,
    subnets: u32, // count
    max_subnet_nodes: u32,
    name: Vec<u8>,
    start: u32,
    end: u32,
) {
    assert_eq!(SubnetsData::<Test>::try_get(subnet_id), Err(()));
    assert_eq!(SubnetName::<Test>::try_get(name), Err(()));
    // assert_eq!(LastSubnetRegistration::<Test>::try_get(subnet_id), Err(()));
    // assert_eq!(SubnetRegistrationEpoch::<Test>::try_get(subnet_id), Err(()));
    assert_eq!(
        SubnetRegistrationInitialColdkeys::<Test>::try_get(subnet_id),
        Err(())
    );
    assert_eq!(SubnetNodesData::<Test>::iter_prefix(subnet_id).count(), 0);
    assert_eq!(TotalSubnetNodes::<Test>::contains_key(subnet_id), false);
    assert_eq!(TotalSubnetNodeUids::<Test>::contains_key(subnet_id), false);
    assert_eq!(
        PeerIdSubnetNodeId::<Test>::iter_prefix(subnet_id).count(),
        0
    );
    assert_eq!(
        BootnodePeerIdSubnetNodeId::<Test>::iter_prefix(subnet_id).count(),
        0
    );
    assert_eq!(
        SubnetNodeUniqueParam::<Test>::iter_prefix(subnet_id).count(),
        0
    );
    assert_eq!(
        HotkeySubnetNodeId::<Test>::iter_prefix(subnet_id).count(),
        0
    );
    assert_eq!(
        SubnetNodeIdHotkey::<Test>::iter_prefix(subnet_id).count(),
        0
    );
    assert_eq!(SubnetPenaltyCount::<Test>::contains_key(subnet_id), false);
    assert_eq!(
        SubnetElectedValidator::<Test>::iter_prefix(subnet_id).count(),
        0
    );
    assert_eq!(
        SubnetConsensusSubmission::<Test>::iter_prefix(subnet_id).count(),
        0
    );

    assert_eq!(
        SubnetNodePenalties::<Test>::iter_prefix(subnet_id).count(),
        0
    );
    let max_subnets = MaxSubnets::<Test>::get();

    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
        assert_eq!(
            HotkeySubnetNodeId::<Test>::get(subnet_id, coldkey.clone()),
            None
        );
        assert_eq!(
            PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer(subnets * max_subnet_nodes + _n)),
            Err(())
        );

        let stake_balance = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
        assert_ok!(Network::remove_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            stake_balance,
        ));

        let delegate_shares =
            AccountSubnetDelegateStakeShares::<Test>::get(hotkey.clone(), subnet_id);
        if delegate_shares != 0 {
            // increase epoch becuse must have only one unstaking per epoch
            increase_epochs(1);

            assert_ok!(Network::remove_delegate_stake(
                RuntimeOrigin::signed(coldkey.clone()),
                subnet_id,
                delegate_shares,
            ));
        }
    }

    let epoch_length = EpochLength::get();
    let stake_cooldown_epochs = StakeCooldownEpochs::<Test>::get();

    let starting_block_number = System::block_number();

    // --- Ensure unstaking is stable
    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
        System::set_block_number(
            System::block_number() + ((epoch_length + 1) * stake_cooldown_epochs),
        );
        let starting_balance = Balances::free_balance(&coldkey.clone());
        let unbondings = StakeUnbondingLedger::<Test>::get(coldkey.clone());
        // assert_eq!(unbondings.len(), 1);
        // let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
        let ledger_balance: u128 = unbondings.values().copied().sum();
        assert_ok!(Network::claim_unbondings(RuntimeOrigin::signed(
            coldkey.clone()
        )));
        let ending_balance = Balances::free_balance(&coldkey.clone());
        assert_eq!(starting_balance + ledger_balance, ending_balance);
        System::set_block_number(starting_block_number);
    }
}

// pub fn build_for_submit_consensus_data(subnet_id: u32, start: u32, end: u32, start_data: u32, end_data: u32) {
//   let subnet_node_data_vec = get_subnet_node_consensus_data(start_data, end_data);

//   for n in start+1..end+1 {
//     assert_ok!(
//       Network::submit_consensus_data(
//         RuntimeOrigin::signed(account(n)),
//         subnet_id,
//         subnet_node_data_vec.clone(),
//       )
//     );
//   }
// }

pub fn increase_blocks(blocks: u32) {
    System::set_block_number(System::block_number() + blocks);
}

pub fn increase_epochs(epochs: u32) {
    if epochs == 0 {
        return;
    }

    let block = System::block_number();
    let epoch_length = EpochLength::get();

    let advance_blocks = epoch_length.saturating_mul(epochs);
    let new_block = block.saturating_add(advance_blocks);

    System::set_block_number(new_block);
}

// pub fn increase_subnet_epochs(epochs: u32, subnet_id: u32) {
//   if epochs == 0 {
//       return;
//   }

//   let block = System::block_number();
//   let epoch_length = EpochLength::get();

//   let advance_blocks = epoch_length.saturating_mul(epochs);
//   let new_block = block.saturating_add(advance_blocks);

//   System::set_block_number(new_block);
// }

pub fn set_epoch(epoch: u32, block_offset: u32) {
    let epoch_length = EpochLength::get();
    System::set_block_number(epoch * epoch_length + block_offset);
}

pub fn get_epoch() -> u32 {
    let current_block = System::block_number();
    let epoch_length: u32 = EpochLength::get();
    current_block.saturating_div(epoch_length)
}

pub fn set_block_to_subnet_slot_epoch(epoch: u32, subnet_id: u32) {
    let epoch_length = EpochLength::get();
    let slot = SubnetSlot::<Test>::get(subnet_id)
        .expect("SubnetSlot must be assigned before setting block");
    let block = slot + epoch * epoch_length;

    System::set_block_number(block);
}

pub fn set_overwatch_epoch(epoch: u32) {
    let epoch_length = EpochLength::get();
    let multiplier = OverwatchEpochLengthMultiplier::<Test>::get();
    System::set_block_number(epoch * multiplier * epoch_length);
}

pub fn set_block_to_overwatch_reveal_block(epoch: u32) {
    let epoch_length = EpochLength::get();
    let multiplier = OverwatchEpochLengthMultiplier::<Test>::get();
    let cutoff_percentage = OverwatchCommitCutoffPercent::<Test>::get();
    let overwatch_epoch_length = epoch_length.saturating_mul(multiplier);
    let block_increase_cutoff =
        Network::percent_mul(overwatch_epoch_length as u128, cutoff_percentage);
    System::set_block_number(epoch * multiplier * epoch_length + block_increase_cutoff as u32);
}

pub fn set_block_to_overwatch_commit_block(epoch: u32) {
    let epoch_length = EpochLength::get();
    let multiplier = OverwatchEpochLengthMultiplier::<Test>::get();
    let cutoff_percentage = OverwatchCommitCutoffPercent::<Test>::get();
    let overwatch_epoch_length = epoch_length.saturating_mul(multiplier);
    let block_increase_cutoff =
        Network::percent_mul(overwatch_epoch_length as u128, cutoff_percentage);
    System::set_block_number(epoch * multiplier * epoch_length as u32);
}

// pub fn get_subnet_node_consensus_data(
//   subnets: u32,
//   max_subnet_nodes: u32,
//   start: u32,
//   end: u32
// ) -> Vec<SubnetNodeConsensusData> {
//   // initialize peer consensus data array
//   let mut subnet_node_data: Vec<SubnetNodeConsensusData> = Vec::new();
//   for n in start+1..end+1 {
//     let peer_subnet_node_data: SubnetNodeConsensusData = SubnetNodeConsensusData {
//       peer_id: peer(subnets*max_subnet_nodes+n),
//       score: DEFAULT_SCORE,
//     };

//     subnet_node_data.push(peer_subnet_node_data);
//   }
//   subnet_node_data
// }

pub fn get_subnet_node_consensus_data(
    subnets: u32,
    max_subnet_nodes: u32,
    start: u32,
    end: u32,
) -> Vec<SubnetNodeConsensusData> {
    // initialize peer consensus data array
    let mut subnet_node_data: Vec<SubnetNodeConsensusData> = Vec::new();
    for n in start..end {
        let peer_subnet_node_data: SubnetNodeConsensusData = SubnetNodeConsensusData {
            subnet_node_id: n + 1,
            score: DEFAULT_SCORE,
        };

        subnet_node_data.push(peer_subnet_node_data);
    }
    subnet_node_data
}

pub fn get_subnet_node_consensus_data_with_custom_score(
    subnets: u32,
    max_subnet_nodes: u32,
    start: u32,
    end: u32,
    score: u128,
) -> Vec<SubnetNodeConsensusData> {
    let mut subnet_node_data: Vec<SubnetNodeConsensusData> = Vec::new();
    for n in start..end {
        let peer_subnet_node_data: SubnetNodeConsensusData = SubnetNodeConsensusData {
            subnet_node_id: n + 1,
            score: score,
        };

        subnet_node_data.push(peer_subnet_node_data);
    }
    subnet_node_data
}

pub fn get_simulated_consensus_data(
    subnet_id: u32,
    node_count: u32,
) -> ConsensusData<<Test as frame_system::Config>::AccountId> {
    let mut attests = BTreeMap::new();
    let mut data = Vec::new();

    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let block_number = Network::get_current_block_as_u32();
    let epoch_length = EpochLength::get();
    let epoch = Network::get_current_block_as_u32() / epoch_length;

    for n in 0..node_count {
        // let node_id = subnet_id*max_subnet_nodes+n+1;
        let node_id = subnet_id * max_subnet_nodes - max_subnet_nodes + n + 1;

        // Simulate some score and block number
        let score = 1e+18 as u128;

        attests.insert(
            node_id,
            AttestEntry {
                block: block_number,
                data: None,
            },
        );
        data.push(SubnetNodeConsensusData {
            subnet_node_id: node_id,
            score,
        });
    }

    let included_subnet_nodes: Vec<SubnetNode<<Test as frame_system::Config>::AccountId>> =
        Network::get_active_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);

    ConsensusData {
        validator_id: subnet_id * max_subnet_nodes,
        attests,
        data,
        prioritize_queue_node_id: None,
        remove_queue_node_id: None,
        subnet_nodes: included_subnet_nodes,
        args: None,
    }
}

// pub fn subnet_node_data_invalid_scores(start: u32, end: u32) -> Vec<SubnetNodeConsensusData> {
//   // initialize peer consensus data array
//   // let mut subnet_node_data: Vec<SubnetNodeConsensusData<<Test as frame_system::Config>::AccountId>> = Vec::new();
//   let mut subnet_node_data: Vec<SubnetNodeConsensusData> = Vec::new();
//   for n in start+1..end+1 {
//     // let peer_subnet_node_data: SubnetNodeConsensusData<<Test as frame_system::Config>::AccountId> = SubnetNodeConsensusData {
//     //   // account_id: account(n),
//     //   peer_id: peer(n),
//     //   score: 10000000000,
//     // };
//     let peer_subnet_node_data: SubnetNodeConsensusData = SubnetNodeConsensusData {
//       // peer_id: peer(n),
//       subnet_node_id: subnets*max_subnet_nodes+n,
//       score: 10000000000,
//     };
//     subnet_node_data.push(peer_subnet_node_data);
//   }
//   subnet_node_data
// }

pub fn post_successful_add_subnet_node_asserts(n: u32, subnet_id: u32, amount: u128) {
    assert_eq!(Network::account_subnet_stake(account(n), subnet_id), amount);
    // assert_eq!(Network::total_account_stake(account(n)), amount);
    assert_eq!(Network::total_subnet_nodes(subnet_id), (n + 1) as u32);
}

// check data after adding multiple peers
// each peer must have equal staking amount per subnet
pub fn post_successful_add_subnet_nodes_asserts(
    total_peers: u32,
    stake_per_peer: u128,
    subnet_id: u32,
) {
    let amount_staked = total_peers as u128 * stake_per_peer;
    assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked);
}

pub fn post_remove_subnet_node_ensures(n: u32, subnet_id: u32) {
    // ensure SubnetNodesData removed
    let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, account(n));
    assert_eq!(subnet_node_id, Err(()));

    assert_eq!(SubnetNodesData::<Test>::iter_prefix(subnet_id).count(), 0);
    // assert_eq!(subnet_node_hotkey, Err(()));

    // ensure PeerIdSubnetNodeId removed
    let subnet_node_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer(n));
    assert_eq!(subnet_node_account, Err(()));
}

pub fn post_remove_unstake_ensures(n: u32, subnet_id: u32) {}

// pub fn add_subnet_node(
//     account_id: u32,
//     subnet_id: u32,
//     peer_id: u32,
//     ip: String,
//     port: u16,
//     amount: u128,
// ) -> Result<(), sp_runtime::DispatchError> {
//     Network::add_subnet_node(
//         RuntimeOrigin::signed(account(account_id)),
//         subnet_id,
//         account(account_id),
//         peer(peer_id),
//         peer(peer_id),
//         peer(peer_id),
//         None,
//         0,
//         amount,
//         None,
//         None,
//     )
// }

pub fn to_bounded<Len: frame_support::traits::Get<u32>>(s: &str) -> BoundedVec<u8, Len> {
    BoundedVec::try_from(s.as_bytes().to_vec()).expect("String too long")
}

// When using this function, manually add stake because some tests require fine tuning stake
pub fn insert_overwatch_node(coldkey_n: u32, hotkey_n: u32) -> u32 {
    let coldkey = account(coldkey_n);
    let hotkey = account(hotkey_n);

    TotalOverwatchNodeUids::<Test>::mutate(|n: &mut u32| *n += 1);
    let current_uid = TotalOverwatchNodeUids::<Test>::get();

    let overwatch_node = OverwatchNode {
        id: current_uid,
        hotkey: hotkey.clone(),
    };

    OverwatchNodes::<Test>::insert(current_uid, overwatch_node);
    HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
    OverwatchNodeIdHotkey::<Test>::insert(current_uid, hotkey.clone());

    let mut hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
    hotkeys.insert(hotkey.clone());
    ColdkeyHotkeys::<Test>::insert(&coldkey.clone(), hotkeys);

    HotkeyOverwatchNodeId::<Test>::insert(&hotkey.clone(), current_uid);

    // let stake_balance = OverwatchMinStakeBalance::<Test>::get();

    // AccountOverwatchStake::<Test>::insert(hotkey.clone(), stake_balance);

    current_uid
}

pub fn set_overwatch_stake(hotkey_n: u32, amount: u128) {
    // -- increase account staking balance
    AccountOverwatchStake::<Test>::mutate(account(hotkey_n), |mut n| *n += amount);
    // -- increase total stake
    TotalOverwatchStake::<Test>::mutate(|mut n| *n += amount);
}

pub fn submit_weight(epoch: u32, subnet_id: u32, node_id: u32, weight: u128) {
    OverwatchReveals::<Test>::insert((epoch, subnet_id, node_id), weight);
}

pub fn new_subnet_data(id: u32, state: SubnetState, start_epoch: u32) -> SubnetData {
    SubnetData {
        id,
        name: vec![],
        repo: vec![],
        description: vec![],
        misc: vec![],
        state,
        start_epoch,
    }
}

// Helper to set up a subnet in storage
pub fn insert_subnet(id: u32, state: SubnetState, start_epoch: u32) {
    let data = new_subnet_data(id, state, start_epoch);
    SubnetsData::<Test>::insert(id, data);
}

pub fn insert_subnet_requirements(id: u32) {
    let delegate_staker_account = 1;
    // Add 100e18 to account for block increase on activation
    let mut min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(id);
    min_subnet_delegate_stake = min_subnet_delegate_stake
        + Network::percent_mul(min_subnet_delegate_stake, 10000000000000000);
    // let _ = Balances::deposit_creating(&account(delegate_staker_account), min_subnet_delegate_stake+500);
    assert_ok!(Balances::transfer(
        &account(0), // alice
        &account(delegate_staker_account),
        min_subnet_delegate_stake + 500,
        ExistenceRequirement::KeepAlive,
    ));

    assert_ne!(min_subnet_delegate_stake, u128::MAX);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(delegate_staker_account)),
        id,
        min_subnet_delegate_stake,
    ));

    let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(id);
    assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);
}

pub fn insert_subnet_node(
    subnet_id: u32,
    node_id: u32,
    coldkey_n: u32,
    hotkey_n: u32,
    peer_n: u32,
    class: SubnetNodeClass,
    start_epoch: u32,
) {
    SubnetNodesData::<Test>::insert(
        subnet_id,
        node_id,
        SubnetNode {
            id: node_id,
            hotkey: account(hotkey_n),
            peer_id: peer(peer_n),
            bootnode_peer_id: peer(peer_n),
            client_peer_id: peer(peer_n),
            bootnode: None,
            delegate_reward_rate: 0,
            last_delegate_reward_rate_update: 0,
            classification: SubnetNodeClassification {
                node_class: class,
                start_epoch: 0,
            },
            unique: Some(BoundedVec::new()),
            non_unique: Some(BoundedVec::new()),
        },
    );
}

// Helper to set registration epoch
pub fn set_registration_epoch(id: u32, epoch: u32) {
    SubnetRegistrationEpoch::<Test>::insert(id, epoch);
}

// Helper to set active nodes count
pub fn set_active_nodes(id: u32, count: u32) {
    TotalActiveSubnetNodes::<Test>::insert(id, count);
}

// Helper to set delegate stake balance
pub fn set_delegate_stake(id: u32, stake: u128) {
    TotalSubnetDelegateStakeBalance::<Test>::insert(id, stake);
}

// Helper to set penalties count
pub fn set_penalties(id: u32, count: u32) {
    SubnetPenaltyCount::<Test>::insert(id, count);
}

pub fn run_subnet_consensus_step(
    subnet_id: u32,
    prioritize_queue_node_id: Option<u32>,
    remove_queue_node_id: Option<u32>,
) {
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let block_number = Network::get_current_block_as_u32();
    let epoch = Network::get_current_epoch_as_u32();

    let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

    let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_data_vec =
        get_subnet_node_consensus_data(subnet_id, max_subnet_nodes, 0, total_subnet_nodes);

    assert_ok!(Network::propose_attestation(
        RuntimeOrigin::signed(validator.clone()),
        subnet_id,
        subnet_node_data_vec.clone(),
        prioritize_queue_node_id,
        remove_queue_node_id,
        None,
        None,
    ));

    let mut attested_nodes = 0;
    for n in 0..total_subnet_nodes {
        let _n = n + 1;
        let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
        if hotkey.clone() == validator.clone() {
            attested_nodes += 1;
            continue;
        }
        if let Some(subnet_node_id) = HotkeySubnetNodeId::<Test>::get(subnet_id, &hotkey) {
            let is_validator = match SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id) {
                Ok(subnet_node) => {
                    subnet_node.has_classification(&SubnetNodeClass::Validator, subnet_epoch)
                }
                Err(()) => false,
            };
            if !is_validator {
                continue;
            }
            attested_nodes += 1;
            assert_ok!(Network::attest(
                RuntimeOrigin::signed(hotkey.clone()),
                subnet_id,
                None,
            ));
        }
    }

    let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch).unwrap();
    assert_eq!(submission.attests.len(), attested_nodes as usize);
    assert_ne!(submission.attests.len(), 0);

    for n in 0..total_subnet_nodes {
        let _n = n + 1;
        let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);

        if let Some(subnet_node_id) = HotkeySubnetNodeId::<Test>::get(subnet_id, &hotkey) {
            let is_validator = match SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id) {
                Ok(subnet_node) => {
                    subnet_node.has_classification(&SubnetNodeClass::Validator, subnet_epoch)
                }
                Err(()) => false,
            };
            if !is_validator {
                continue;
            }
        } else {
            continue;
        }

        let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();

        if hotkey == validator.clone() {
            assert_ne!(submission.attests.get(&(subnet_node_id)), None);
            assert_eq!(
                submission.attests.get(&(subnet_node_id)).unwrap().block,
                System::block_number()
            );
        } else {
            assert_ne!(submission.attests.get(&(subnet_node_id)), None);
            assert_eq!(
                submission.attests.get(&(subnet_node_id)).unwrap().block,
                System::block_number()
            );
        }
    }
}

/// Force overwatch node qualified
// This only works in non network simulations
pub fn make_overwatch_qualified(coldkey_n: u32) {
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let mut subnet_nodes: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for n in 0..max_subnets - 1 {
        let mut node_ids = BTreeSet::new();
        let _n = n + 1;
        let hotkey_n = get_hotkey_n(_n, max_subnet_nodes, max_subnets, _n);
        insert_subnet(_n, SubnetState::Active, 0);
        insert_subnet_node(
            _n,
            1,         // node id
            coldkey_n, // coldkey
            hotkey_n,  // hotkey
            hotkey_n,  // peer
            SubnetNodeClass::Validator,
            0,
        );
        node_ids.insert(1);
        subnet_nodes.insert(_n, node_ids);

        TotalSubnetUids::<Test>::mutate(|n: &mut u32| *n += 1);
        TotalActiveSubnets::<Test>::mutate(|n: &mut u32| *n += 1);
    }

    ColdkeySubnetNodes::<Test>::insert(account(coldkey_n), subnet_nodes);

    // max reputation
    ColdkeyReputation::<Test>::insert(
        account(coldkey_n),
        Reputation {
            start_epoch: 0,
            score: 1_000_000_000_000_000_000,
            lifetime_node_count: max_subnets * max_subnet_nodes,
            total_active_nodes: max_subnets * max_subnet_nodes,
            total_increases: 999,
            total_decreases: 0,
            average_attestation: 1_000_000_000_000_000_000,
            last_validator_epoch: 0,
            ow_score: 1_000_000_000_000_000_000,
        },
    );

    let min_age = OverwatchMinAge::<Test>::get();
    increase_epochs(min_age + 1);
}

pub fn make_overwatch_unqualified(coldkey_n: u32) {
    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

    let mut subnet_nodes: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    ColdkeySubnetNodes::<Test>::insert(account(coldkey_n), subnet_nodes);

    // max reputation
    ColdkeyReputation::<Test>::insert(
        account(coldkey_n),
        Reputation {
            start_epoch: 0,
            score: 50_000_000_000_000_000,
            lifetime_node_count: 0,
            total_active_nodes: 0,
            total_increases: 0,
            total_decreases: 0,
            average_attestation: 50_000_000_000_000_000,
            last_validator_epoch: 0,
            ow_score: 50_000_000_000_000_000,
        },
    );
}

/// Force overwatch node qualified
// This only works in non network simulations
pub fn make_overwatch_qualified_sim(coldkey_n: u32) {
    let coldkey = account(coldkey_n);
    let subnets = TotalActiveSubnets::<Test>::get();

    let max_subnets = MaxSubnets::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let amount = NetworkMinStakeBalance::<Test>::get();

    let mut subnet_nodes: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();

    let mut amount_staked = 0;
    for n in 0..subnets {
        let _n = n + 1;
        let subnet_id = _n;
        let burn_amount = Network::calculate_burn_amount(subnet_id);
        let node_count = TotalSubnetNodes::<Test>::get(subnet_id);
        let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, node_count + 1);
        let peer_id = get_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id = get_bootnode_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id(subnets, max_subnet_nodes, max_subnets, _n);
        assert_ok!(Balances::transfer(
            &account(0), // alice
            &coldkey.clone(),
            amount + burn_amount + 500,
            ExistenceRequirement::KeepAlive,
        ));
        amount_staked += amount;

        // Add overwatch node on each subnet
        assert_ok!(Network::register_subnet_node(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            amount,
            None,
            None,
            u128::MAX
        ));
    }

    // max reputation
    ColdkeyReputation::<Test>::insert(
        account(coldkey_n),
        Reputation {
            start_epoch: 0,
            score: 1_000_000_000_000_000_000,
            lifetime_node_count: max_subnets * max_subnet_nodes,
            total_active_nodes: max_subnets * max_subnet_nodes,
            total_increases: 999,
            total_decreases: 0,
            average_attestation: 1_000_000_000_000_000_000,
            last_validator_epoch: 0,
            ow_score: 1_000_000_000_000_000_000,
        },
    );

    let min_age = OverwatchMinAge::<Test>::get();
    increase_epochs(min_age + 1);
}
