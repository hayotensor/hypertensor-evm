//! Benchmarking setup for pallet-network
// frame-omni-bencher v1 benchmark pallet --runtime target/release/wbuild/solochain-template-runtime/solochain_template_runtime.compact.compressed.wasm --extrinsic "" --pallet "pallet_network" --output pallets/network/src/weights.rs --template ./.maintain/frame-weight-template.hbs

// frame-omni-bencher v1 benchmark pallet --runtime target/release/wbuild/solochain-template-runtime/solochain_template_runtime.compact.compressed.wasm --extrinsic "" --pallet "pallet_network"

// cargo build --release --features runtime-benchmarks
// cargo test --release --features runtime-benchmarks
// Build only this pallet
// cargo build --package pallet-network --features runtime-benchmarks
// cargo build --package pallet-collective --features runtime-benchmarks
// cargo +nightly build --release --features runtime-benchmarks

#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Network;
use crate::*;
use frame_benchmarking::v2::*;
use frame_support::pallet_prelude::DispatchError;
use frame_support::pallet_prelude::Zero;
use frame_support::Callable;
use frame_support::{
    assert_noop, assert_ok,
    traits::{EnsureOrigin, Get, OnInitialize, UnfilteredDispatchable},
    weights::WeightMeter,
};
use frame_system::limits::BlockWeights;
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
pub use pallet::*;
use pallet_collective::Pallet as Collective;
use pallet_collective::{Instance1, Members};
use pallet_treasury::Pallet as Treasury;
use scale_info::prelude::format;
use scale_info::prelude::vec;
use sp_core::blake2_128;
use sp_core::OpaquePeerId as PeerId;
use sp_runtime::traits::Hash;
use sp_runtime::traits::Header;
use sp_runtime::SaturatedConversion;
use sp_runtime::Vec;

const SEED: u32 = 0;
const DEFAULT_SCORE: u128 = 100e+18 as u128;
const DEFAULT_SUBNET_INIT_COST: u128 = 100e+18 as u128;
const DEFAULT_SUBNET_NAME: &str = "subnet-name";
const DEFAULT_SUBNET_NAME_2: &str = "subnet-name-2";
const DEFAULT_SUBNET_NODE_STAKE: u128 = 100e+18 as u128;
const DEFAULT_SUBNET_REGISTRATION_BLOCKS: u64 = 130_000;
const DEFAULT_STAKE_TO_BE_ADDED: u128 = 100e+18 as u128;
const DEFAULT_DELEGATE_STAKE_TO_BE_ADDED: u128 = 100e+18 as u128;
const DEFAULT_DEPOSIT_AMOUNT: u128 = 1000e+18 as u128;
const ALICE_EXPECTED_BALANCE: u128 = 1000000000000000000000000; // 1,000,000

pub type BalanceOf<T> = <T as Config>::Currency;
type TreasuryPallet<T> = pallet_treasury::Pallet<T, ()>;

fn peer(id: u32) -> PeerId {
    let peer_id = format!("QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N{id}");
    PeerId(peer_id.into())
}

fn get_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let caller: T::AccountId = account(name, index, SEED);
    caller
}

fn get_alice<T: Config>() -> T::AccountId {
    let alice: T::AccountId = get_account::<T>("alice", 0);
    let alice_balance = T::Currency::free_balance(&alice.clone());
    if alice_balance < ALICE_EXPECTED_BALANCE.try_into().ok().expect("REASON") {
        let _ = T::Currency::deposit_creating(
            &alice.clone(),
            ALICE_EXPECTED_BALANCE.try_into().ok().expect("REASON"),
        );
    }
    alice
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let caller: T::AccountId = account(name, index, SEED);
    // Give the account half of the maximum value of the `Balance` type.
    // Otherwise some transfers will fail with an overflow error.
    let deposit_amount: u128 = MinSubnetMinStake::<T>::get() + 10000000000000;
    T::Currency::deposit_creating(&caller, deposit_amount.try_into().ok().expect("REASON"));
    caller
}

fn funded_initializer<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let caller: T::AccountId = account(name, index, SEED);
    // Give the account half of the maximum value of the `Balance` type.
    // Otherwise some transfers will fail with an overflow error.
    let block_number = get_current_block_as_u32::<T>();
    let cost = Network::<T>::get_current_registration_cost(block_number) + 1000;
    let alice = get_alice::<T>();
    assert_ok!(T::Currency::transfer(
        &alice, // alice
        &caller.clone(),
        (cost + 500).try_into().ok().expect("REASON"),
        ExistenceRequirement::KeepAlive,
    ));

    caller
}

pub fn get_coldkey_n<T: Config>(subnets: u32, max_subnet_nodes: u32, n: u32) -> u32 {
    subnets * max_subnet_nodes + n
}

pub fn get_coldkey<T: Config>(subnets: u32, max_subnet_nodes: u32, n: u32) -> T::AccountId {
    get_account::<T>("coldkey", get_coldkey_n::<T>(subnets, max_subnet_nodes, n))
}

pub fn get_hotkey_n<T: Config>(
    subnets: u32,
    max_subnet_nodes: u32,
    max_subnets: u32,
    n: u32,
) -> u32 {
    max_subnets * max_subnet_nodes + (subnets * max_subnet_nodes) + n
}

pub fn get_hotkey<T: Config>(
    subnets: u32,
    max_subnet_nodes: u32,
    max_subnets: u32,
    n: u32,
) -> T::AccountId {
    get_account::<T>(
        "hotkey",
        get_hotkey_n::<T>(subnets, max_subnet_nodes, max_subnets, n),
    )
}

pub fn get_peer_id<T: Config>(
    subnets: u32,
    max_subnet_nodes: u32,
    max_subnets: u32,
    n: u32,
) -> PeerId {
    peer(max_subnets * max_subnet_nodes + (subnets * max_subnet_nodes) + n)
}

pub fn get_bootnode_peer_id<T: Config>(
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

pub fn get_client_peer_id<T: Config>(
    subnets: u32,
    max_subnet_nodes: u32,
    max_subnets: u32,
    n: u32,
) -> PeerId {
    peer(
        (max_subnets * max_subnet_nodes * 3)
            + (max_subnets * max_subnet_nodes + (subnets * max_subnet_nodes) + n),
    )
}

pub fn get_overwatch_coldkey<T: Config>(
    max_subnet_nodes: u32,
    max_subnets: u32,
    max_onodes: u32,
    n: u32,
) -> T::AccountId {
    get_account::<T>(
        "overwatch_coldkey",
        max_subnets * max_subnet_nodes + max_subnets * max_subnet_nodes + n,
    )
}

pub fn get_overwatch_hotkey<T: Config>(
    max_subnet_nodes: u32,
    max_subnets: u32,
    max_onodes: u32,
    n: u32,
) -> T::AccountId {
    get_account::<T>(
        "overwatch_hotkey",
        max_subnets * max_subnet_nodes + max_subnets * max_subnet_nodes + max_onodes + n,
    )
}

pub fn increase_epochs<T: Config>(n: u32) {
    if n == 0 {
        return;
    }

    let block = get_current_block_as_u32::<T>();
    let epoch_length = T::EpochLength::get();

    let advance_blocks = epoch_length.saturating_mul(n);
    let new_block = block.saturating_add(advance_blocks);

    frame_system::Pallet::<T>::set_block_number(new_block.into());
}

pub fn increase_overwatch_epochs<T: Config>(n: u32) {
    if n == 0 {
        return;
    }

    let block = get_current_block_as_u32::<T>();
    let multiplier = OverwatchEpochLengthMultiplier::<T>::get();

    let advance_blocks = n * multiplier * T::EpochLength::get();
    let new_block = block.saturating_add(advance_blocks);

    frame_system::Pallet::<T>::set_block_number(new_block.into());
}

pub fn set_overwatch_epoch<T: Config>(n: u32) {
    let multiplier = OverwatchEpochLengthMultiplier::<T>::get();
    frame_system::Pallet::<T>::set_block_number((n * multiplier * T::EpochLength::get()).into());
}

fn build_activated_subnet<T: Config>(
    name: Vec<u8>,
    start: u32,
    mut end: u32,
    deposit_amount: u128,
    amount: u128,
) {
    let alice = get_alice::<T>();

    let epoch_length = T::EpochLength::get();
    let block_number = get_current_block_as_u32::<T>();
    let epoch = block_number.saturating_div(epoch_length);

    let min_nodes = MinSubnetNodes::<T>::get();
    let subnets = TotalActiveSubnets::<T>::get() + 1;
    let max_subnets = MaxSubnets::<T>::get();
    let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    let owner_coldkey =
        funded_initializer::<T>("subnet_owner", subnets * max_subnets * max_subnet_nodes);
    let owner_hotkey =
        get_account::<T>("subnet_owner", subnets * max_subnets * max_subnet_nodes + 1);

    let register_subnet_data: RegistrationSubnetData<T::AccountId> =
        default_registration_subnet_data::<T>(
            subnets,
            max_subnet_nodes,
            name.clone().into(),
            start,
            end,
        );

    // --- Register subnet for activation
    assert_ok!(Network::<T>::register_subnet(
        RawOrigin::Signed(owner_coldkey.clone()).into(),
        100000000000000000000000,
        register_subnet_data,
    ));

    let subnet_id = SubnetName::<T>::get(name.clone()).unwrap();
    let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

    if end == 0 {
        end = min_nodes;
    }

    let epoch = get_current_block_as_u32::<T>() / epoch_length;
    let deposit_amount: u128 = MinSubnetMinStake::<T>::get() + 10000;

    // --- Add subnet nodes
    let block_number = get_current_block_as_u32::<T>();
    let mut amount_staked = 0;
    for n in start..end {
        let _n = n + 1;
        let coldkey = get_coldkey::<T>(subnets, max_subnet_nodes, _n);
        let hotkey = get_hotkey::<T>(subnets, max_subnet_nodes, max_subnets, _n);
        let peer_id = get_peer_id::<T>(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id =
            get_bootnode_peer_id::<T>(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id::<T>(subnets, max_subnet_nodes, max_subnets, _n);
        let alice = get_alice::<T>();
        assert_ok!(T::Currency::transfer(
            &alice, // alice
            &coldkey.clone(),
            (deposit_amount + DEFAULT_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        amount_staked += amount;
        assert_ok!(Network::<T>::register_subnet_node(
            RawOrigin::Signed(coldkey.clone()).into(),
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
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<T>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        let subnet_node_data =
            SubnetNodesData::<T>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());
        assert_eq!(subnet_node_data.delegate_reward_rate, 0);

        let key_owner = HotkeyOwner::<T>::get(subnet_node_data.hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        assert_eq!(subnet_node_data.peer_id, peer_id.clone());

        // --- Is ``Validator`` if registered before subnet activation
        assert_eq!(
            subnet_node_data.classification.node_class,
            SubnetNodeClass::Validator
        );
        assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, epoch));

        let peer_subnet_node_account = PeerIdSubnetNodeId::<T>::get(subnet_id, peer_id.clone());
        assert_eq!(peer_subnet_node_account, hotkey_subnet_node_id);

        let account_subnet_stake = AccountSubnetStake::<T>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, amount);

        let hotkey_subnet_id = HotkeySubnetId::<T>::get(hotkey.clone());
        assert_eq!(hotkey_subnet_id, Some(subnet_id));

        let mut is_electable = false;
        for node_id in SubnetNodeElectionSlots::<T>::get(subnet_id).iter() {
            if *node_id == hotkey_subnet_node_id {
                is_electable = true;
            }
        }
        assert!(is_electable);

        let coldkey_subnet_nodes = ColdkeySubnetNodes::<T>::get(coldkey.clone());
        assert!(coldkey_subnet_nodes
            .get(&subnet_id)
            .unwrap()
            .contains(&hotkey_subnet_node_id))
    }

    let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
    assert_eq!(active_nodes, end - start);

    let slot_list = SubnetNodeElectionSlots::<T>::get(subnet_id);
    assert_eq!(slot_list.len(), active_nodes as usize);

    let total_subnet_stake = TotalSubnetStake::<T>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<T>::get();
    assert_eq!(total_subnet_stake, amount_staked);

    let min_subnet_delegate_stake = Network::<T>::get_min_subnet_delegate_stake_balance(subnet_id)
        + (1000e+18 as u128 * subnets as u128);
    // --- Add the minimum required delegate stake balance to activate the subnet

    let delegate_staker_account: T::AccountId = funded_account::<T>("delegate_staker", 1);
    let alice = get_alice::<T>();
    assert_ok!(T::Currency::transfer(
        &alice, // alice
        &delegate_staker_account.clone(),
        (min_subnet_delegate_stake + 500)
            .try_into()
            .ok()
            .expect("REASON"),
        ExistenceRequirement::KeepAlive,
    ));
    assert_ok!(Network::<T>::add_to_delegate_stake(
        RawOrigin::Signed(delegate_staker_account.clone()).into(),
        subnet_id,
        min_subnet_delegate_stake,
    ));

    let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
    assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

    let delegate_shares =
        AccountSubnetDelegateStakeShares::<T>::get(&delegate_staker_account, subnet_id);

    let min_registration_epochs = MinSubnetRegistrationEpochs::<T>::get();
    increase_epochs::<T>(min_registration_epochs + 1);

    assert_ok!(Network::<T>::activate_subnet(
        RawOrigin::Signed(owner_coldkey.clone()).into(),
        subnet_id,
    ));

    let subnet = SubnetsData::<T>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Active);
}

fn build_registered_subnet<T: Config>(
    name: Vec<u8>,
    start: u32,
    mut end: u32,
    deposit_amount: u128,
    amount: u128,
    use_unique_coldkey: bool, // if to use unique coldkeys for each subnet
) {
    let alice = get_alice::<T>();

    let epoch_length = T::EpochLength::get();
    let block_number = get_current_block_as_u32::<T>();
    let epoch = block_number.saturating_div(epoch_length);

    let min_nodes = MinSubnetNodes::<T>::get();
    let subnets = TotalActiveSubnets::<T>::get() + 1;
    let max_subnets = MaxSubnets::<T>::get();
    let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    let owner_coldkey =
        funded_initializer::<T>("subnet_owner", subnets * max_subnets * max_subnet_nodes);
    let owner_hotkey =
        get_account::<T>("subnet_owner", subnets * max_subnets * max_subnet_nodes + 1);

    let register_subnet_data: RegistrationSubnetData<T::AccountId> =
        default_registration_subnet_data::<T>(
            subnets,
            max_subnet_nodes,
            name.clone().into(),
            start,
            end,
        );

    // --- Register subnet for activation
    assert_ok!(Network::<T>::register_subnet(
        RawOrigin::Signed(owner_coldkey.clone()).into(),
        100000000000000000000000,
        register_subnet_data,
    ));

    let subnet_id = SubnetName::<T>::get(name.clone()).unwrap();
    let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

    if end == 0 {
        end = min_nodes;
    }

    let epoch = get_current_block_as_u32::<T>() / epoch_length;
    let deposit_amount: u128 = MinSubnetMinStake::<T>::get() + 10000;

    // --- Add subnet nodes
    let block_number = get_current_block_as_u32::<T>();
    let mut amount_staked = 0;
    for n in start..end {
        let _n = n + 1;
        let coldkey = if use_unique_coldkey {
            get_coldkey::<T>(subnets, max_subnet_nodes, _n)
        } else {
            get_coldkey::<T>(1, max_subnet_nodes, _n)
        };
        let hotkey = get_hotkey::<T>(subnets, max_subnet_nodes, max_subnets, _n);
        let peer_id = get_peer_id::<T>(subnets, max_subnet_nodes, max_subnets, _n);
        let bootnode_peer_id =
            get_bootnode_peer_id::<T>(subnets, max_subnet_nodes, max_subnets, _n);
        let client_peer_id = get_client_peer_id::<T>(subnets, max_subnet_nodes, max_subnets, _n);
        let alice = get_alice::<T>();
        assert_ok!(T::Currency::transfer(
            &alice, // alice
            &coldkey.clone(),
            (deposit_amount + DEFAULT_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        amount_staked += amount;
        assert_ok!(Network::<T>::register_subnet_node(
            RawOrigin::Signed(coldkey.clone()).into(),
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
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<T>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        let subnet_node_data =
            SubnetNodesData::<T>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());
        assert_eq!(subnet_node_data.delegate_reward_rate, 0);

        let key_owner = HotkeyOwner::<T>::get(subnet_node_data.hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        assert_eq!(subnet_node_data.peer_id, peer_id.clone());

        // --- Is ``Validator`` if registered before subnet activation
        assert_eq!(
            subnet_node_data.classification.node_class,
            SubnetNodeClass::Validator
        );
        assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, epoch));

        let peer_subnet_node_account = PeerIdSubnetNodeId::<T>::get(subnet_id, peer_id.clone());
        assert_eq!(peer_subnet_node_account, hotkey_subnet_node_id);

        let account_subnet_stake = AccountSubnetStake::<T>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, amount);

        let hotkey_subnet_id = HotkeySubnetId::<T>::get(hotkey.clone());
        assert_eq!(hotkey_subnet_id, Some(subnet_id));

        let mut is_electable = false;
        for node_id in SubnetNodeElectionSlots::<T>::get(subnet_id).iter() {
            if *node_id == hotkey_subnet_node_id {
                is_electable = true;
            }
        }
        assert!(is_electable);

        let coldkey_subnet_nodes = ColdkeySubnetNodes::<T>::get(coldkey.clone());
        assert!(coldkey_subnet_nodes
            .get(&subnet_id)
            .unwrap()
            .contains(&hotkey_subnet_node_id))
    }

    let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
    assert_eq!(active_nodes, end - start);

    let slot_list = SubnetNodeElectionSlots::<T>::get(subnet_id);
    assert_eq!(slot_list.len(), active_nodes as usize);

    let total_subnet_stake = TotalSubnetStake::<T>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<T>::get();
    assert_eq!(total_subnet_stake, amount_staked);

    let min_subnet_delegate_stake = Network::<T>::get_min_subnet_delegate_stake_balance(subnet_id)
        + (1000e+18 as u128 * subnets as u128);
    // --- Add the minimum required delegate stake balance to activate the subnet

    let delegate_staker_account: T::AccountId = funded_account::<T>("delegate_staker", 1);
    let alice = get_alice::<T>();
    assert_ok!(T::Currency::transfer(
        &alice, // alice
        &delegate_staker_account.clone(),
        (min_subnet_delegate_stake + 500)
            .try_into()
            .ok()
            .expect("REASON"),
        ExistenceRequirement::KeepAlive,
    ));
    assert_ok!(Network::<T>::add_to_delegate_stake(
        RawOrigin::Signed(delegate_staker_account.clone()).into(),
        subnet_id,
        min_subnet_delegate_stake,
    ));

    let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
    assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

    let delegate_shares =
        AccountSubnetDelegateStakeShares::<T>::get(&delegate_staker_account, subnet_id);

    let subnet = SubnetsData::<T>::get(subnet_id).unwrap();
    assert_eq!(subnet.state, SubnetState::Registered);
}

pub fn default_registration_subnet_data<T: Config>(
    subnets: u32,
    max_subnet_nodes: u32,
    name: Vec<u8>,
    start: u32,
    end: u32,
) -> RegistrationSubnetData<T::AccountId> {
    let seed_bytes: &[u8] = &name;
    let add_subnet_data = RegistrationSubnetData {
        name: name.clone(),
        repo: blake2_128(seed_bytes).to_vec(), // must be unique
        description: Vec::new(),
        misc: Vec::new(),
        churn_limit: 4,
        min_stake: MinSubnetMinStake::<T>::get(),
        max_stake: NetworkMaxStakeBalance::<T>::get(),
        delegate_stake_percentage: 100000000000000000, // 10%
        subnet_node_queue_epochs: 4,
        idle_classification_epochs: 4,
        included_classification_epochs: 4,
        max_node_penalties: 3,
        max_registered_nodes: MaxMaxRegisteredNodes::<T>::get(),
        initial_coldkeys: get_initial_coldkeys::<T>(subnets, max_subnet_nodes, start, end),
        key_types: BTreeSet::from([KeyType::Rsa]),
        bootnodes: BTreeSet::from([BoundedVec::new()]),
    };
    add_subnet_data
}

pub fn insert_overwatch_node<T: Config>(coldkey_n: u32, hotkey_n: u32) -> u32 {
    let coldkey = get_account::<T>("overwatch_node", coldkey_n);
    let hotkey = get_account::<T>("overwatch_node", hotkey_n);

    TotalOverwatchNodeUids::<T>::mutate(|n: &mut u32| *n += 1);
    let current_uid = TotalOverwatchNodeUids::<T>::get();

    let overwatch_node = OverwatchNode {
        id: current_uid,
        hotkey: hotkey.clone(),
    };

    OverwatchNodes::<T>::insert(current_uid, overwatch_node);
    HotkeyOwner::<T>::insert(hotkey.clone(), coldkey.clone());
    OverwatchNodeIdHotkey::<T>::insert(current_uid, hotkey.clone());

    let mut hotkeys = ColdkeyHotkeys::<T>::get(&coldkey.clone());
    hotkeys.insert(hotkey.clone());
    ColdkeyHotkeys::<T>::insert(&coldkey.clone(), hotkeys);

    HotkeyOverwatchNodeId::<T>::insert(&hotkey.clone(), current_uid);

    current_uid
}

pub fn set_overwatch_stake<T: Config>(hotkey_n: u32, amount: u128) {
    let account = funded_account::<T>("overwatch_node", hotkey_n);
    // -- increase account staking balance
    AccountOverwatchStake::<T>::mutate(account, |mut n| *n += amount);
    // -- increase total stake
    TotalOverwatchStake::<T>::mutate(|mut n| *n += amount);
}

pub fn submit_overwatch_reveal<T: Config>(
    overwatch_epoch: u32,
    subnet_id: u32,
    node_id: u32,
    weight: u128,
) {
    OverwatchReveals::<T>::insert((overwatch_epoch, subnet_id, node_id), weight);
}

pub fn insert_subnet<T: Config>(id: u32, state: SubnetState, start_epoch: u32) {
    let data = new_subnet_data::<T>(id, state, start_epoch);
    SubnetsData::<T>::insert(id, data);
}

pub fn new_subnet_data<T: Config>(id: u32, state: SubnetState, start_epoch: u32) -> SubnetData {
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

pub fn insert_subnet_node<T: Config>(
    subnet_id: u32,
    node_id: u32,
    coldkey_n: u32,
    hotkey_n: u32,
    peer_n: u32,
    class: SubnetNodeClass,
    start_epoch: u32,
) {
    SubnetNodesData::<T>::insert(
        subnet_id,
        node_id,
        SubnetNode {
            id: node_id,
            hotkey: get_account::<T>("subnet_node", hotkey_n),
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

pub fn make_overwatch_qualified<T: Config>(coldkey_n: u32) {
    let coldkey = get_account::<T>("overwatch_node", coldkey_n);
    let max_subnets = MaxSubnets::<T>::get();
    let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    let mut subnet_nodes: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for n in 0..max_subnets - 1 {
        let mut node_ids = BTreeSet::new();
        let _n = n + 1;
        let hotkey_n = get_hotkey_n::<T>(_n, max_subnet_nodes, max_subnets, _n);
        insert_subnet::<T>(_n, SubnetState::Active, 0);
        insert_subnet_node::<T>(
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

        TotalSubnetUids::<T>::mutate(|n: &mut u32| *n += 1);
        TotalActiveSubnets::<T>::mutate(|n: &mut u32| *n += 1);
    }

    ColdkeySubnetNodes::<T>::insert(coldkey.clone(), subnet_nodes);

    // max reputation
    ColdkeyReputation::<T>::insert(
        coldkey.clone(),
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

    let min_age = OverwatchMinAge::<T>::get();
    increase_epochs::<T>(min_age + 1);
}

// Specifically for linear overwatch benchmarks
fn make_overwatch_node_qualified<T: Config>(coldkey_n: u32, x: u32) {
    let coldkey = get_account::<T>("overwatch_node", coldkey_n);
    let max_subnets = MaxSubnets::<T>::get();
    let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    let mut subnet_nodes: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    let subnet_node_id = 999;

    for s in 0..x {
        let path: Vec<u8> = format!("subnet-name-{s}").into();
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path.clone()).unwrap();
        let hotkey_n = get_hotkey_n::<T>(subnet_id, max_subnet_nodes, max_subnets, coldkey_n);
        insert_subnet_node::<T>(
            subnet_id,
            subnet_node_id, // node id
            coldkey_n,      // coldkey
            hotkey_n,       // hotkey
            hotkey_n,       // peer
            SubnetNodeClass::Validator,
            0,
        );
        ColdkeySubnetNodes::<T>::mutate(&coldkey, |node_map| {
            node_map
                .entry(subnet_id)
                .or_insert_with(BTreeSet::new)
                .insert(subnet_node_id);
        });
    }

    // max reputation
    ColdkeyReputation::<T>::insert(
        coldkey.clone(),
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

    let min_age = OverwatchMinAge::<T>::get();
    increase_epochs::<T>(min_age + 1);
}

pub fn make_overwatch_unqualified<T: Config>(coldkey_n: u32) {
    let coldkey = get_account::<T>("overwatch_node", coldkey_n);
    let max_subnets = MaxSubnets::<T>::get();
    let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    let mut subnet_nodes: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    ColdkeySubnetNodes::<T>::insert(coldkey.clone(), subnet_nodes);

    // max reputation
    ColdkeyReputation::<T>::insert(
        coldkey.clone(),
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

// fn get_subnet_node_data(start: u32, end: u32) -> Vec<SubnetNodeConsensusData> {
//   // initialize peer consensus data array
//   let mut subnet_node_data: Vec<SubnetNodeConsensusData> = Vec::new();
//   for n in start..end {
//     let peer_subnet_node_data: SubnetNodeConsensusData = SubnetNodeConsensusData {
//       peer_id: peer(n),
//       score: DEFAULT_SCORE,
//     };
//     subnet_node_data.push(peer_subnet_node_data);
//   }
//   subnet_node_data
// }

pub fn get_subnet_node_consensus_data<T: frame_system::Config>(
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

pub fn u32_to_block<T: frame_system::Config>(input: u32) -> BlockNumberFor<T> {
    input.try_into().ok().expect("REASON")
}

pub fn block_to_u32<T: frame_system::Config>(block: BlockNumberFor<T>) -> u32 {
    TryInto::try_into(block)
        .ok()
        .expect("blockchain will not exceed 2^64 blocks; QED.")
}

pub fn set_block_to_subnet_slot_epoch<T: Config>(epoch: u32, subnet_id: u32) {
    let epoch_length = T::EpochLength::get();
    let slot =
        SubnetSlot::<T>::get(subnet_id).expect("SubnetSlot must be assigned before setting block");
    let block = u32_to_block::<T>(slot + epoch * epoch_length);
    frame_system::Pallet::<T>::set_block_number(block);
}

pub fn get_current_block_as_u32<T: frame_system::Config>() -> u32 {
    TryInto::try_into(<frame_system::Pallet<T>>::block_number())
        .ok()
        .expect("blockchain will not exceed u32::MAX blocks; QED.")
}

pub fn set_block_to_overwatch_reveal_block<T: Config>(epoch: u32) {
    let epoch_length = T::EpochLength::get();
    let multiplier = OverwatchEpochLengthMultiplier::<T>::get();
    let cutoff_percentage = OverwatchCommitCutoffPercent::<T>::get();
    let overwatch_epoch_length = epoch_length.saturating_mul(multiplier);
    let block_increase_cutoff =
        Network::<T>::percent_mul(overwatch_epoch_length as u128, cutoff_percentage);
    let block = u32_to_block::<T>(epoch * multiplier * epoch_length + block_increase_cutoff as u32);
    frame_system::Pallet::<T>::set_block_number(block);
}

pub fn set_block_to_overwatch_commit_block<T: Config>(epoch: u32) {
    let epoch_length = T::EpochLength::get();
    let multiplier = OverwatchEpochLengthMultiplier::<T>::get();
    let cutoff_percentage = OverwatchCommitCutoffPercent::<T>::get();
    let overwatch_epoch_length = epoch_length.saturating_mul(multiplier);
    let block = u32_to_block::<T>(epoch * multiplier * epoch_length as u32);
    frame_system::Pallet::<T>::set_block_number(block);
}

pub fn u128_to_balance<T: frame_system::Config + pallet::Config>(
    input: u128,
) -> Option<
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
> {
    input.try_into().ok()
}

pub fn get_initial_coldkeys<T: Config>(
    subnets: u32,
    max_subnet_nodes: u32,
    start: u32,
    end: u32,
) -> BTreeMap<T::AccountId, u32> {
    let mut whitelist = BTreeMap::new();
    for n in start..end {
        let _n = n + 1;
        whitelist.insert(get_coldkey::<T>(subnets, max_subnet_nodes, _n), 1);
        // whitelist.insert(funded_account::<T>("coldkey", subnets*max_subnet_nodes+n));
    }
    whitelist
}

pub fn get_simulated_consensus_data<T: Config>(
    subnet_id: u32,
    node_count: u32,
) -> ConsensusData<T::AccountId> {
    let mut attests = BTreeMap::new();
    let mut data = Vec::new();

    let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    let block_number = get_current_block_as_u32::<T>();
    let epoch_length = T::EpochLength::get();
    let epoch = get_current_block_as_u32::<T>() / epoch_length;

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

    let included_subnet_nodes: Vec<SubnetNode<T::AccountId>> =
        Network::<T>::get_active_classified_subnet_nodes(
            subnet_id,
            &SubnetNodeClass::Included,
            epoch,
        );

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

pub fn run_subnet_consensus_step<T: Config>(
    subnet_id: u32,
    prioritize_queue_node_id: Option<u32>,
    remove_queue_node_id: Option<u32>,
) {
    let max_subnets = MaxSubnets::<T>::get();
    let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    let block_number = Network::<T>::get_current_block_as_u32();
    let epoch = Network::<T>::get_current_epoch_as_u32();

    let subnet_epoch = Network::<T>::get_current_subnet_epoch_as_u32(subnet_id);

    let validator_id = SubnetElectedValidator::<T>::get(subnet_id, subnet_epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id.unwrap()).unwrap();

    let total_subnet_nodes = TotalSubnetNodes::<T>::get(subnet_id);

    let subnet_node_data_vec =
        get_subnet_node_consensus_data::<T>(subnet_id, max_subnet_nodes, 0, total_subnet_nodes);

    assert_ok!(Network::<T>::propose_attestation(
        RawOrigin::Signed(validator.clone()).into(),
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
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, _n);
        if hotkey.clone() == validator.clone() {
            attested_nodes += 1;
            continue;
        }
        if let Some(subnet_node_id) = HotkeySubnetNodeId::<T>::get(subnet_id, &hotkey) {
            let is_validator = match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
                Ok(subnet_node) => {
                    subnet_node.has_classification(&SubnetNodeClass::Validator, subnet_epoch)
                }
                Err(()) => false,
            };
            if !is_validator {
                continue;
            }
            attested_nodes += 1;
            assert_ok!(Network::<T>::attest(
                RawOrigin::Signed(hotkey.clone()).into(),
                subnet_id,
                None,
            ));
        }
    }

    let submission = SubnetConsensusSubmission::<T>::get(subnet_id, subnet_epoch).unwrap();
    assert_eq!(submission.attests.len(), attested_nodes as usize);
    assert_ne!(submission.attests.len(), 0);

    for n in 0..total_subnet_nodes {
        let _n = n + 1;
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, _n);

        if let Some(subnet_node_id) = HotkeySubnetNodeId::<T>::get(subnet_id, &hotkey) {
            let is_validator = match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
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

        let subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        if hotkey.clone() == validator.clone() {
            assert_ne!(submission.attests.get(&(subnet_node_id)), None);
            assert_eq!(
                submission.attests.get(&(subnet_node_id)).unwrap().block,
                Network::<T>::get_current_block_as_u32()
            );
        } else {
            assert_ne!(submission.attests.get(&(subnet_node_id)), None);
            assert_eq!(
                submission.attests.get(&(subnet_node_id)).unwrap().block,
                Network::<T>::get_current_block_as_u32()
            );
        }
    }
}

fn to_bounded<Len: frame_support::traits::Get<u32>>(s: &str) -> BoundedVec<u8, Len> {
    BoundedVec::try_from(s.as_bytes().to_vec()).expect("String too long")
}

pub fn make_commit<T: Config>(weight: u128, salt: Vec<u8>) -> T::Hash {
    T::Hashing::hash_of(&(weight, salt))
}

// Collective pallet functions

fn is_member<T: Config>(account: T::AccountId)
where
    T: pallet_collective::Config<Instance1>,
{
    let is_member = pallet_collective::Pallet::<T, Instance1>::is_member(&account);
}

fn get_collective_members<T: Config>() -> Vec<T::AccountId>
where
    T: pallet_collective::Config<Instance1>,
{
    let members = pallet_collective::Members::<T, Instance1>::get();
    members
}

fn set_members<T: Config>()
where
    T: pallet_collective::Config<Instance1>,
{
    let members = vec![
        get_account::<T>("collective", 1),
        get_account::<T>("collective", 2),
        get_account::<T>("collective", 3),
        get_account::<T>("collective", 4),
        get_account::<T>("collective", 5),
    ];
    assert_ok!(pallet_collective::Pallet::<T, Instance1>::set_members(
        RawOrigin::Root.into(),
        members.clone(),
        Some(members[0].clone()),
        T::MaxMembers::get()
    ));
}

// fn make_set_max_activation_grace_epochs_proposal<C>(
// ) -> Box<<C as pallet_collective::Config<pallet_collective::Instance1>>::Proposal>
// where
//     C: Config + pallet_collective::Config<pallet_collective::Instance1>,
//     <C as pallet_collective::Config<pallet_collective::Instance1>>::Proposal: From<crate::Call<C>>,
// {
//     // Create your network call
//     let network_call = crate::Call::<C>::set_max_activation_grace_epochs { value: 999 };

//     // Convert to the collective's Proposal type and box it
//     let proposal = <C as pallet_collective::Config<pallet_collective::Instance1>>::Proposal::from(
//         network_call,
//     );
//     Box::new(proposal)
// }

// fn propose_collective<T: Config, I>(
//     proposer: T::AccountId,
//     threshold: u32,
//     proposal: Box<<T as pallet_collective::Config<pallet_collective::Instance1>>::Proposal>,
//     length_bound: u32,
// ) -> Result<(), sp_runtime::DispatchError>
// where
//     T: Config + pallet_collective::Config<pallet_collective::Instance1>,
// {
//     pallet_collective::Pallet::<T, pallet_collective::Instance1>::propose(
//         RawOrigin::Signed(proposer.clone()).into(),
//         threshold,
//         proposal,
//         length_bound,
//     )
// }

// fn get_treasury_pot<T: Config<I>, I: 'static>(
// ) {
// 	Treasury::<T, I>::pot();
// }

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_subnet() {
        let block_number = get_current_block_as_u32::<T>();
        let cost = Network::<T>::get_current_registration_cost(block_number) + 1000;

        let funded_initializer = funded_initializer::<T>("funded_initializer", 0);

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let subnets = TotalActiveSubnets::<T>::get() + 1;
        let register_subnet_data: RegistrationSubnetData<T::AccountId> =
            default_registration_subnet_data::<T>(
                subnets,
                max_subnet_nodes,
                DEFAULT_SUBNET_NAME.into(),
                0,
                min_nodes + 1,
            );

        let current_block_number = get_current_block_as_u32::<T>();

        #[extrinsic_call]
        register_subnet(
            RawOrigin::Signed(funded_initializer.clone()),
            100000000000000000000000,
            register_subnet_data,
        );

        let owner = SubnetOwner::<T>::get(1).unwrap();
        assert_eq!(owner, funded_initializer.clone());

        let subnet = SubnetsData::<T>::get(1).unwrap();
        assert_eq!(subnet.id, 1);
        let path: Vec<u8> = DEFAULT_SUBNET_NAME.into();
        assert_eq!(subnet.name, path);
    }

    #[benchmark]
    fn activate_subnet() {
        let start = 0;
        let end = 12;
        let alice = get_alice::<T>();
        let min_nodes = MinSubnetNodes::<T>::get();
        let subnets = TotalActiveSubnets::<T>::get() + 1;
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnets * max_subnets * max_subnet_nodes);
        let owner_hotkey =
            get_account::<T>("subnet_owner", subnets * max_subnets * max_subnet_nodes + 1);

        let name = DEFAULT_SUBNET_NAME;
        let register_subnet_data: RegistrationSubnetData<T::AccountId> =
            default_registration_subnet_data::<T>(
                subnets,
                max_subnet_nodes,
                name.clone().into(),
                start,
                end,
            );

        // --- Register subnet for activation
        assert_ok!(Network::<T>::register_subnet(
            RawOrigin::Signed(owner_coldkey.clone()).into(),
            100000000000000000000000,
            register_subnet_data,
        ));

        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

        assert_eq!(subnet.id, subnet_id);

        let epoch_length = T::EpochLength::get();
        let epoch = get_current_block_as_u32::<T>() / epoch_length;

        let block_number = get_current_block_as_u32::<T>();
        let mut amount_staked = 0;
        let amount = DEFAULT_STAKE_TO_BE_ADDED;
        let deposit_amount = DEFAULT_DEPOSIT_AMOUNT;
        for n in start..end {
            let _n = n + 1;
            let coldkey = get_coldkey::<T>(subnets, max_subnet_nodes, _n);
            let hotkey = get_hotkey::<T>(subnets, max_subnet_nodes, max_subnets, _n);
            let peer_id = get_peer_id::<T>(subnets, max_subnet_nodes, max_subnets, _n);
            let bootnode_peer_id =
                get_bootnode_peer_id::<T>(subnets, max_subnet_nodes, max_subnets, _n);
            let client_peer_id =
                get_client_peer_id::<T>(subnets, max_subnet_nodes, max_subnets, _n);
            assert_ok!(T::Currency::transfer(
                &get_alice::<T>(), // alice
                &coldkey.clone(),
                (deposit_amount + DEFAULT_STAKE_TO_BE_ADDED + 500)
                    .try_into()
                    .ok()
                    .expect("REASON"),
                ExistenceRequirement::KeepAlive,
            ));
            T::Currency::deposit_creating(
                &coldkey.clone(),
                DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"),
            );
            amount_staked += amount;
            assert_ok!(Network::<T>::register_subnet_node(
                RawOrigin::Signed(coldkey.clone()).into(),
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
                HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

            let subnet_node_id_hotkey =
                SubnetNodeIdHotkey::<T>::get(subnet_id, hotkey_subnet_node_id).unwrap();
            assert_eq!(subnet_node_id_hotkey, hotkey.clone());

            let subnet_node_data =
                SubnetNodesData::<T>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
            assert_eq!(subnet_node_data.hotkey, hotkey.clone());
            assert_eq!(subnet_node_data.delegate_reward_rate, 0);

            let key_owner = HotkeyOwner::<T>::get(subnet_node_data.hotkey.clone());
            assert_eq!(key_owner, coldkey.clone());

            assert_eq!(subnet_node_data.peer_id, peer_id.clone());

            // --- Is ``Validator`` if registered before subnet activation
            assert_eq!(
                subnet_node_data.classification.node_class,
                SubnetNodeClass::Validator
            );
            assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, epoch));

            let peer_subnet_node_account = PeerIdSubnetNodeId::<T>::get(subnet_id, peer_id.clone());
            assert_eq!(peer_subnet_node_account, hotkey_subnet_node_id);

            let account_subnet_stake = AccountSubnetStake::<T>::get(hotkey.clone(), subnet_id);
            assert_eq!(account_subnet_stake, amount);

            let hotkey_subnet_id = HotkeySubnetId::<T>::get(hotkey.clone());
            assert_eq!(hotkey_subnet_id, Some(subnet_id));

            let mut is_electable = false;
            for node_id in SubnetNodeElectionSlots::<T>::get(subnet_id).iter() {
                if *node_id == hotkey_subnet_node_id {
                    is_electable = true;
                }
            }
            assert!(is_electable);

            let coldkey_subnet_nodes = ColdkeySubnetNodes::<T>::get(coldkey.clone());
            assert!(coldkey_subnet_nodes
                .get(&subnet_id)
                .unwrap()
                .contains(&hotkey_subnet_node_id))
        }

        let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
        assert_eq!(active_nodes, end - start);

        let slot_list = SubnetNodeElectionSlots::<T>::get(subnet_id);
        assert_eq!(slot_list.len(), active_nodes as usize);

        let total_subnet_stake = TotalSubnetStake::<T>::get(subnet_id);
        assert_eq!(total_subnet_stake, amount_staked);

        let total_stake = TotalStake::<T>::get();
        assert_eq!(total_subnet_stake, amount_staked);

        let min_subnet_delegate_stake =
            Network::<T>::get_min_subnet_delegate_stake_balance(subnet_id)
                + (1000e+18 as u128 * subnets as u128);
        // --- Add the minimum required delegate stake balance to activate the subnet

        let delegate_staker_account: T::AccountId = funded_account::<T>("delegate_staker", 1);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_staker_account.clone(),
            (min_subnet_delegate_stake + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));
        assert_ok!(Network::<T>::add_to_delegate_stake(
            RawOrigin::Signed(delegate_staker_account.clone()).into(),
            subnet_id,
            min_subnet_delegate_stake,
        ));

        let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
        assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

        let delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(&delegate_staker_account, subnet_id);

        let min_registration_epochs = MinSubnetRegistrationEpochs::<T>::get();
        increase_epochs::<T>(min_registration_epochs + 1);

        #[extrinsic_call]
        activate_subnet(RawOrigin::Signed(owner_coldkey.clone()), subnet_id);

        let subnet = SubnetsData::<T>::get(subnet_id).unwrap();
        let path: Vec<u8> = DEFAULT_SUBNET_NAME.into();
        assert_eq!(subnet.name, path);
        assert_eq!(subnet.state, SubnetState::Active);
    }

    #[benchmark]
    fn owner_pause_subnet() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        #[extrinsic_call]
        owner_pause_subnet(RawOrigin::Signed(owner_coldkey.clone()), subnet_id);

        let subnet = SubnetsData::<T>::get(subnet_id).unwrap();
        assert_eq!(subnet.state, SubnetState::Paused);
    }

    #[benchmark]
    fn owner_unpause_subnet() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        assert_ok!(Network::<T>::owner_pause_subnet(
            RawOrigin::Signed(owner_coldkey.clone()).into(),
            subnet_id
        ));

        let subnet = SubnetsData::<T>::get(subnet_id).unwrap();
        assert_eq!(subnet.state, SubnetState::Paused);

        #[extrinsic_call]
        owner_unpause_subnet(RawOrigin::Signed(owner_coldkey.clone()), subnet_id);

        let subnet = SubnetsData::<T>::get(subnet_id).unwrap();
        assert_eq!(subnet.state, SubnetState::Active);
    }

    #[benchmark]
    fn owner_deactivate_subnet() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        #[extrinsic_call]
        owner_deactivate_subnet(RawOrigin::Signed(owner_coldkey.clone()), subnet_id);

        assert_eq!(SubnetsData::<T>::try_get(subnet_id), Err(()));
    }

    #[benchmark]
    fn owner_update_name() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_value: Vec<u8> = "new-subnet-name".into();

        #[extrinsic_call]
        owner_update_name(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value.clone(),
        );

        let subnet_data = SubnetsData::<T>::get(subnet_id).unwrap();
        assert_eq!(subnet_data.name, new_value.clone());

        assert_eq!(SubnetName::<T>::get(&new_value.clone()).unwrap(), subnet_id);
    }

    #[benchmark]
    fn owner_update_repo() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_value: Vec<u8> = "new-subnet-repo".into();

        #[extrinsic_call]
        owner_update_repo(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value.clone(),
        );

        let subnet_data = SubnetsData::<T>::get(subnet_id).unwrap();
        assert_eq!(subnet_data.repo, new_value.clone());

        assert_eq!(SubnetRepo::<T>::get(&new_value.clone()).unwrap(), subnet_id);
    }

    #[benchmark]
    fn owner_update_description() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_value: Vec<u8> = "new-subnet-description".into();

        #[extrinsic_call]
        owner_update_description(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value.clone(),
        );

        let subnet_data = SubnetsData::<T>::get(subnet_id).unwrap();
        assert_eq!(subnet_data.description, new_value.clone());
    }

    #[benchmark]
    fn owner_update_misc() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_value: Vec<u8> = "new-subnet-misc".into();

        #[extrinsic_call]
        owner_update_misc(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value.clone(),
        );

        let subnet_data = SubnetsData::<T>::get(subnet_id).unwrap();
        assert_eq!(subnet_data.misc, new_value.clone());
    }

    #[benchmark]
    fn owner_update_churn_limit() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let current_value = ChurnLimit::<T>::get(subnet_id);

        let new_value = current_value + 1;

        #[extrinsic_call]
        owner_update_churn_limit(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        let value = ChurnLimit::<T>::get(subnet_id);
        assert_eq!(value, new_value);
    }

    #[benchmark]
    fn owner_update_registration_queue_epochs() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let current_value = SubnetNodeQueueEpochs::<T>::get(subnet_id);

        let new_value = current_value + 1;

        #[extrinsic_call]
        owner_update_registration_queue_epochs(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        let value = SubnetNodeQueueEpochs::<T>::get(subnet_id);
        assert_eq!(value, new_value);
    }

    #[benchmark]
    fn owner_update_idle_classification_epochs() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let current_value = IdleClassificationEpochs::<T>::get(subnet_id);

        let new_value = current_value + 1;

        #[extrinsic_call]
        owner_update_idle_classification_epochs(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        let value = IdleClassificationEpochs::<T>::get(subnet_id);
        assert_eq!(value, new_value);
    }

    #[benchmark]
    fn owner_update_included_classification_epochs() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let current_value = IncludedClassificationEpochs::<T>::get(subnet_id);

        let new_value = current_value + 1;

        #[extrinsic_call]
        owner_update_included_classification_epochs(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        let value = IncludedClassificationEpochs::<T>::get(subnet_id);
        assert_eq!(value, new_value);
    }

    #[benchmark]
    fn owner_update_max_node_penalties() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let current_value = MaxSubnetNodePenalties::<T>::get(subnet_id);

        let new_value = current_value + 1;

        #[extrinsic_call]
        owner_update_max_node_penalties(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        let value = MaxSubnetNodePenalties::<T>::get(subnet_id);
        assert_eq!(value, new_value);
    }

    #[benchmark]
    fn owner_add_or_update_initial_coldkeys() {
        let block_number = get_current_block_as_u32::<T>();
        let cost = Network::<T>::get_current_registration_cost(block_number) + 1000;

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let subnets = TotalActiveSubnets::<T>::get() + 1;
        let mut register_subnet_data: RegistrationSubnetData<T::AccountId> =
            default_registration_subnet_data::<T>(
                subnets,
                max_subnet_nodes,
                DEFAULT_SUBNET_NAME.into(),
                0,
                min_nodes + 1,
            );

        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let rand_account = funded_initializer::<T>("0", 0);
        let rand_account_2 = funded_initializer::<T>("2", 2);
        let rand_account_3 = funded_initializer::<T>("3", 3);
        let initial_coldkeys = BTreeMap::from([
            (rand_account.clone(), 1),
            (rand_account_2.clone(), 1),
            (rand_account_3.clone(), 1),
        ]);
        register_subnet_data.initial_coldkeys = initial_coldkeys;

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnets * max_subnets * max_subnet_nodes);
        let owner_hotkey =
            get_account::<T>("subnet_owner", subnets * max_subnets * max_subnet_nodes + 1);

        assert_ok!(Network::<T>::register_subnet(
            RawOrigin::Signed(owner_coldkey.clone()).into(),
            100000000000000000000000,
            register_subnet_data,
        ));

        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let rand_account_4 = funded_initializer::<T>("4", 4);

        let new_value = BTreeMap::from([(rand_account_4.clone(), 1)]);

        #[extrinsic_call]
        owner_add_or_update_initial_coldkeys(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value.clone(),
        );

        let expected_coldkeys = BTreeMap::from([
            (rand_account.clone(), 1),
            (rand_account_2.clone(), 1),
            (rand_account_3.clone(), 1),
            (rand_account_4.clone(), 1),
        ]);
        let coldkeys = SubnetRegistrationInitialColdkeys::<T>::get(subnet_id).unwrap();
        assert_eq!(coldkeys, expected_coldkeys.clone());
    }

    #[benchmark]
    fn owner_remove_initial_coldkeys() {
        let block_number = get_current_block_as_u32::<T>();
        let cost = Network::<T>::get_current_registration_cost(block_number) + 1000;

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let subnets = TotalActiveSubnets::<T>::get() + 1;
        let mut register_subnet_data: RegistrationSubnetData<T::AccountId> =
            default_registration_subnet_data::<T>(
                subnets,
                max_subnet_nodes,
                DEFAULT_SUBNET_NAME.into(),
                0,
                min_nodes + 1,
            );

        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnets * max_subnets * max_subnet_nodes);
        let owner_hotkey =
            get_account::<T>("subnet_owner", subnets * max_subnets * max_subnet_nodes + 1);

        let rand_account = funded_initializer::<T>("0", 0);
        let rand_account_2 = funded_initializer::<T>("2", 2);
        let rand_account_3 = funded_initializer::<T>("3", 3);
        let initial_coldkeys = BTreeMap::from([
            (rand_account.clone(), 1),
            (rand_account_2.clone(), 1),
            (rand_account_3.clone(), 1),
        ]);
        register_subnet_data.initial_coldkeys = initial_coldkeys;

        assert_ok!(Network::<T>::register_subnet(
            RawOrigin::Signed(owner_coldkey.clone()).into(),
            100000000000000000000000,
            register_subnet_data,
        ));

        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let remove_value = BTreeSet::from([rand_account_3.clone()]);

        #[extrinsic_call]
        owner_remove_initial_coldkeys(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            remove_value.clone(),
        );

        let expected_coldkeys =
            BTreeMap::from([(rand_account.clone(), 1), (rand_account_2.clone(), 1)]);
        let coldkeys = SubnetRegistrationInitialColdkeys::<T>::get(subnet_id).unwrap();
        assert_eq!(coldkeys, expected_coldkeys.clone());
    }

    #[benchmark]
    fn owner_update_key_types() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_keytypes = BTreeSet::from([KeyType::Ed25519]);

        #[extrinsic_call]
        owner_update_key_types(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_keytypes.clone(),
        );

        let key_types = SubnetKeyTypes::<T>::get(subnet_id);
        assert_eq!(key_types, new_keytypes.clone());
    }

    // #[benchmark]
    // fn owner_update_min_stake() {
    //     let max_subnet_nodes = MaxSubnetNodes::<T>::get();
    //     build_activated_subnet::<T>(
    //         DEFAULT_SUBNET_NAME.into(),
    //         0,
    //         max_subnet_nodes,
    //         DEFAULT_DEPOSIT_AMOUNT,
    //         DEFAULT_SUBNET_NODE_STAKE,
    //     );
    //     let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

    //     let min_nodes = MinSubnetNodes::<T>::get();
    //     let max_subnets = MaxSubnets::<T>::get();
    //     let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    //     let owner_coldkey =
    //         funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

    //     let current_value = SubnetMinStakeBalance::<T>::get(subnet_id);
    //     let new_value = current_value + 1;

    //     #[extrinsic_call]
    //     owner_update_min_stake(
    //         RawOrigin::Signed(owner_coldkey.clone()),
    //         subnet_id,
    //         new_value,
    //     );

    //     let value = SubnetMinStakeBalance::<T>::get(subnet_id);
    //     assert_eq!(value, new_value);
    // }

    // #[benchmark]
    // fn owner_update_subnet_node_consecutive_included_epochs() {
    //     let max_subnet_nodes = MaxSubnetNodes::<T>::get();
    //     build_activated_subnet::<T>(
    //         DEFAULT_SUBNET_NAME.into(),
    //         0,
    //         max_subnet_nodes,
    //         DEFAULT_DEPOSIT_AMOUNT,
    //         DEFAULT_SUBNET_NODE_STAKE,
    //     );
    //     let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

    //     let min_nodes = MinSubnetNodes::<T>::get();
    //     let max_subnets = MaxSubnets::<T>::get();
    //     let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    //     let owner_coldkey =
    //         funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

    //     let current_value = SubnetNodeConsecutiveIncludedEpochs::<T>::get(subnet_id)
    //     let new_value = current_value + 1;

    //     #[extrinsic_call]
    //     do_owner_update_subnet_node_consecutive_included_epochs(
    //         RawOrigin::Signed(owner_coldkey.clone()),
    //         subnet_id,
    //         new_value,
    //     );

    //     let value = SubnetMinStakeBalance::<T>::get(subnet_id);
    //     assert_eq!(value, new_value);
    // }

    #[benchmark]
    fn owner_update_min_max_stake() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let min = SubnetMinStakeBalance::<T>::get(subnet_id);
        let new_min = min + 1;

        let max = SubnetMaxStakeBalance::<T>::get(subnet_id);
        let new_max = max - 1;

        #[extrinsic_call]
        owner_update_min_max_stake(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_min,
            new_max
        );

        let value = SubnetMaxStakeBalance::<T>::get(subnet_id);
        assert_eq!(value, new_value);
    }

    #[benchmark]
    fn owner_update_delegate_stake_percentage() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let current_value = SubnetDelegateStakeRewardsPercentage::<T>::get(subnet_id);

        let new_value = current_value + 1;

        let block_number = get_current_block_as_u32::<T>();

        let last_update = LastSubnetDelegateStakeRewardsUpdate::<T>::get(subnet_id);
        let update_period = SubnetDelegateStakeRewardsUpdatePeriod::<T>::get();

        let update_to_block = if block_number - last_update < update_period {
            last_update + update_period
        } else {
            block_number
        };

        frame_system::Pallet::<T>::set_block_number(u32_to_block::<T>(update_to_block + 1));

        #[extrinsic_call]
        owner_update_delegate_stake_percentage(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        let value = SubnetDelegateStakeRewardsPercentage::<T>::get(subnet_id);
        assert_eq!(value, new_value);
    }

    #[benchmark]
    fn owner_update_max_registered_nodes() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let current_value = MaxRegisteredNodes::<T>::get(subnet_id);

        let new_value = current_value - 1;

        #[extrinsic_call]
        owner_update_max_registered_nodes(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        let value = MaxRegisteredNodes::<T>::get(subnet_id);
        assert_eq!(value, new_value);
    }

    #[benchmark]
    fn transfer_subnet_ownership() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let current_value = MaxRegisteredNodes::<T>::get(subnet_id);

        let new_owner = funded_initializer::<T>("new_subnet_owner", 0);

        #[extrinsic_call]
        transfer_subnet_ownership(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_owner.clone(),
        );

        let pending_owner = PendingSubnetOwner::<T>::get(subnet_id).unwrap();
        assert_eq!(new_owner.clone(), pending_owner.clone());
    }

    #[benchmark]
    fn accept_subnet_ownership() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let current_value = MaxRegisteredNodes::<T>::get(subnet_id);

        let new_owner = funded_initializer::<T>("new_subnet_owner", 0);

        assert_ok!(Network::<T>::transfer_subnet_ownership(
            RawOrigin::Signed(owner_coldkey.clone()).into(),
            subnet_id,
            new_owner.clone(),
        ));

        #[extrinsic_call]
        accept_subnet_ownership(RawOrigin::Signed(new_owner.clone()), subnet_id);

        let owner = SubnetOwner::<T>::get(subnet_id).unwrap();
        assert_eq!(new_owner.clone(), owner);
    }

    #[benchmark]
    fn owner_add_bootnode_access() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_access = funded_initializer::<T>("new", 0);

        // sanity check
        assert!(SubnetBootnodeAccess::<T>::get(subnet_id)
            .get(&new_access.clone())
            .is_none());

        #[extrinsic_call]
        owner_add_bootnode_access(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_access.clone(),
        );

        let new_access_set = SubnetBootnodeAccess::<T>::get(subnet_id);
        assert!(new_access_set.get(&new_access.clone()).is_some());
    }

    #[benchmark]
    fn owner_remove_bootnode_access() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_access = funded_initializer::<T>("new", 0);

        // sanity check
        assert!(SubnetBootnodeAccess::<T>::get(subnet_id)
            .get(&new_access.clone())
            .is_none());

        assert_ok!(Network::<T>::owner_add_bootnode_access(
            RawOrigin::Signed(owner_coldkey.clone()).into(),
            subnet_id,
            new_access.clone()
        ));

        #[extrinsic_call]
        owner_remove_bootnode_access(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_access.clone(),
        );

        let new_access_set = SubnetBootnodeAccess::<T>::get(subnet_id);
        assert!(new_access_set.get(&new_access.clone()).into_iter().count() == 0);
    }

    #[benchmark]
    fn owner_update_target_node_registrations_per_epoch() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_value = TargetNodeRegistrationsPerEpoch::<T>::get(subnet_id) - 1;

        #[extrinsic_call]
        owner_update_target_node_registrations_per_epoch(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        assert_eq!(
            TargetNodeRegistrationsPerEpoch::<T>::get(subnet_id),
            new_value
        );
    }

    #[benchmark]
    fn owner_update_node_burn_rate_alpha() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_value = NodeBurnRateAlpha::<T>::get(subnet_id) - 1;

        #[extrinsic_call]
        owner_update_node_burn_rate_alpha(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        assert_eq!(NodeBurnRateAlpha::<T>::get(subnet_id), new_value);
    }

    #[benchmark]
    fn owner_update_queue_immunity_epochs() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_value = QueueImmunityEpochs::<T>::get(subnet_id) - 1;

        #[extrinsic_call]
        owner_update_queue_immunity_epochs(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        assert_eq!(QueueImmunityEpochs::<T>::get(subnet_id), new_value);
    }

    #[benchmark]
    fn owner_update_subnet_node_score_penalty_threshold() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_value = 1;

        #[extrinsic_call]
        owner_update_subnet_node_score_penalty_threshold(
            RawOrigin::Signed(owner_coldkey.clone()),
            subnet_id,
            new_value,
        );

        assert_eq!(
            SubnetNodeScorePenaltyThreshold::<T>::get(subnet_id),
            new_value
        );
    }

    #[benchmark]
    fn update_bootnodes() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let owner_coldkey =
            funded_initializer::<T>("subnet_owner", subnet_id * max_subnets * max_subnet_nodes);

        let new_access = funded_initializer::<T>("new", 0);
        SubnetBootnodeAccess::<T>::insert(subnet_id, BTreeSet::from([new_access.clone()]));
        let bv = |b: u8| BoundedVec::<u8, DefaultMaxVectorLength>::try_from(vec![b]).unwrap();
        let add_set = BTreeSet::from([bv(1), bv(2)]);

        #[extrinsic_call]
        update_bootnodes(
            RawOrigin::Signed(new_access.clone()),
            subnet_id,
            add_set.clone(),
            BTreeSet::new(),
        );

        // Verify bootnodes added
        let stored = SubnetBootnodes::<T>::get(subnet_id);
        assert!(stored.contains(&bv(1)));
        assert!(stored.contains(&bv(2)));
    }

    #[benchmark]
    fn remove_subnet_node() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        #[extrinsic_call]
        remove_subnet_node(
            RawOrigin::Signed(hotkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
        );

        assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), end - 1);

        let subnet_node_id = HotkeySubnetNodeId::<T>::try_get(subnet_id, hotkey.clone());
        assert_eq!(subnet_node_id, Err(()));
    }

    #[benchmark]
    fn register_or_update_identity() {
        let coldkey = funded_initializer::<T>("coldkey", 99);
        let hotkey = funded_initializer::<T>("hotkey", 0);

        HotkeyOwner::<T>::insert(hotkey.clone(), coldkey.clone());
        let name = to_bounded::<DefaultMaxVectorLength>("name");
        let url = to_bounded::<DefaultMaxUrlLength>("url");
        let image = to_bounded::<DefaultMaxUrlLength>("image");
        let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
        let x = to_bounded::<DefaultMaxSocialIdLength>("x");
        let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
        let github = to_bounded::<DefaultMaxUrlLength>("github");
        let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
        let description = to_bounded::<DefaultMaxVectorLength>("description");
        let misc = to_bounded::<DefaultMaxVectorLength>("misc");

        #[extrinsic_call]
        register_or_update_identity(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            name.clone(),
            url.clone(),
            image.clone(),
            discord.clone(),
            x.clone(),
            telegram.clone(),
            github.clone(),
            hugging_face.clone(),
            description.clone(),
            misc.clone(),
        );

        let coldkey_identity = ColdkeyIdentity::<T>::get(&coldkey);
        assert_eq!(coldkey_identity.name, name);
        assert_eq!(coldkey_identity.url, url);
        assert_eq!(coldkey_identity.image, image);
        assert_eq!(coldkey_identity.discord, discord);
        assert_eq!(coldkey_identity.x, x);
        assert_eq!(coldkey_identity.telegram, telegram);
        assert_eq!(coldkey_identity.github, github);
        assert_eq!(coldkey_identity.hugging_face, hugging_face);
        assert_eq!(coldkey_identity.description, description);
        assert_eq!(coldkey_identity.misc, misc);
        assert_eq!(
            ColdkeyIdentityNameOwner::<T>::get(name.clone()),
            coldkey.clone()
        );
    }

    #[benchmark]
    fn remove_identity() {
        let coldkey = funded_initializer::<T>("coldkey", 99);
        let hotkey = funded_initializer::<T>("hotkey", 0);

        HotkeyOwner::<T>::insert(hotkey.clone(), coldkey.clone());
        let name = to_bounded::<DefaultMaxVectorLength>("name");
        let url = to_bounded::<DefaultMaxUrlLength>("url");
        let image = to_bounded::<DefaultMaxUrlLength>("image");
        let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
        let x = to_bounded::<DefaultMaxSocialIdLength>("x");
        let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
        let github = to_bounded::<DefaultMaxUrlLength>("github");
        let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
        let description = to_bounded::<DefaultMaxVectorLength>("description");
        let misc = to_bounded::<DefaultMaxVectorLength>("misc");

        assert_ok!(Network::<T>::register_or_update_identity(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            name.clone(),
            url.clone(),
            image.clone(),
            discord.clone(),
            x.clone(),
            telegram.clone(),
            github.clone(),
            hugging_face.clone(),
            description.clone(),
            misc.clone(),
        ));

        // Sanity check
        assert_eq!(
            ColdkeyIdentityNameOwner::<T>::get(name.clone()),
            coldkey.clone()
        );

        #[extrinsic_call]
        remove_identity(RawOrigin::Signed(coldkey.clone()));

        assert_eq!(
            ColdkeyIdentityNameOwner::<T>::try_get(name.clone()),
            Err(())
        );
    }

    #[benchmark]
    fn update_node_delegate_reward_rate() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end);
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();
        let subnet_node = SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);
        let current_value = subnet_node.delegate_reward_rate;
        let new_value = current_value + 1;

        let max_reward_rate_decrease = MaxRewardRateDecrease::<T>::get();
        let reward_rate_update_period = NodeRewardRateUpdatePeriod::<T>::get();

        let block_number = get_current_block_as_u32::<T>();
        frame_system::Pallet::<T>::set_block_number(u32_to_block::<T>(
            block_number + reward_rate_update_period,
        ));

        #[extrinsic_call]
        update_node_delegate_reward_rate(
            RawOrigin::Signed(coldkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
            new_value,
        );

        let subnet_node = SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);
        assert_eq!(subnet_node.delegate_reward_rate, new_value);
    }

    #[benchmark]
    fn add_stake() {
        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id =
            get_client_peer_id::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + DEFAULT_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::register_subnet_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            DEFAULT_SUBNET_NODE_STAKE,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_STAKE_TO_BE_ADDED).try_into().ok().expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        #[extrinsic_call]
        add_stake(
            RawOrigin::Signed(coldkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
            hotkey.clone(),
            DEFAULT_STAKE_TO_BE_ADDED,
        );

        let account_subnet_stake = Network::<T>::account_subnet_stake(hotkey.clone(), subnet_id);
        assert_eq!(
            account_subnet_stake,
            DEFAULT_SUBNET_NODE_STAKE + DEFAULT_STAKE_TO_BE_ADDED
        );
    }

    #[benchmark]
    fn remove_stake() {
        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id =
            get_client_peer_id::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + DEFAULT_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::register_subnet_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            DEFAULT_SUBNET_NODE_STAKE,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_STAKE_TO_BE_ADDED).try_into().ok().expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::add_stake(
            RawOrigin::Signed(coldkey.clone()).into(),
            subnet_id,
            hotkey_subnet_node_id,
            hotkey.clone(),
            DEFAULT_STAKE_TO_BE_ADDED
        ));

        let account_subnet_stake = Network::<T>::account_subnet_stake(hotkey.clone(), subnet_id);
        assert_eq!(
            account_subnet_stake,
            DEFAULT_SUBNET_NODE_STAKE + DEFAULT_STAKE_TO_BE_ADDED
        );

        #[extrinsic_call]
        remove_stake(
            RawOrigin::Signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            DEFAULT_STAKE_TO_BE_ADDED,
        );

        let account_subnet_stake = Network::<T>::account_subnet_stake(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE);
    }

    #[benchmark]
    fn claim_unbondings() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_account.clone(),
            (DEFAULT_DELEGATE_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::add_to_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()).into(),
            subnet_id,
            DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
        ));
        let delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);

        let total_subnet_delegated_stake_shares =
            TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
        let total_subnet_delegated_stake_balance =
            TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

        let delegate_balance = Network::<T>::convert_to_balance(
            delegate_shares,
            total_subnet_delegated_stake_shares,
            total_subnet_delegated_stake_balance,
        );

        let block = get_current_block_as_u32::<T>();

        assert_ok!(Network::<T>::remove_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()).into(),
            subnet_id,
            delegate_shares
        ));

        let unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<T>::get(delegate_account.clone());
        assert_eq!(unbondings.len(), 1);
        let (unbonding_block, balance) = unbondings.iter().next().unwrap();
        assert_eq!(
            *unbonding_block,
            block + DelegateStakeCooldownEpochs::<T>::get() * T::EpochLength::get()
        );
        assert_eq!(*balance, delegate_balance);

        let pre_delegator_balance: u128 = T::Currency::free_balance(&delegate_account.clone())
            .try_into()
            .ok()
            .expect("REASON");

        frame_system::Pallet::<T>::set_block_number(u32_to_block::<T>(
            get_current_block_as_u32::<T>()
                + DelegateStakeCooldownEpochs::<T>::get() * T::EpochLength::get()
                + 1,
        ));

        #[extrinsic_call]
        claim_unbondings(RawOrigin::Signed(delegate_account.clone()));

        let post_delegator_balance: u128 = T::Currency::free_balance(&delegate_account.clone())
            .try_into()
            .ok()
            .expect("REASON");

        assert_eq!(post_delegator_balance, pre_delegator_balance + balance);
    }

    #[benchmark]
    fn add_to_delegate_stake() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

        let _ = T::Currency::deposit_creating(
            &delegate_account.clone(),
            (DEFAULT_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
        );
        let starting_delegator_balance = T::Currency::free_balance(&delegate_account.clone());

        #[extrinsic_call]
        add_to_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()),
            subnet_id,
            DEFAULT_STAKE_TO_BE_ADDED,
        );

        let post_delegator_balance = T::Currency::free_balance(&delegate_account.clone());
        assert_eq!(
            post_delegator_balance,
            starting_delegator_balance - DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON")
        );

        let total_subnet_delegated_stake_shares =
            TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
        let total_subnet_delegated_stake_balance =
            TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
        let delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
        let delegate_balance = Network::<T>::convert_to_balance(
            delegate_shares,
            total_subnet_delegated_stake_shares,
            total_subnet_delegated_stake_balance,
        );

        // Ensure balance is within <= 0.01% of deposited balance, and less than deposited balance
        assert!(
            (delegate_balance
                >= Network::<T>::percent_mul(DEFAULT_STAKE_TO_BE_ADDED, 990000000000000000))
                && (delegate_balance < DEFAULT_STAKE_TO_BE_ADDED)
        );
    }

    #[benchmark]
    fn swap_delegate_stake() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let from_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME_2.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let to_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME_2.into()).unwrap();

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_account.clone(),
            (DEFAULT_DELEGATE_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::add_to_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()).into(),
            from_subnet_id,
            DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
        ));

        let delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);
        let total_subnet_delegated_stake_shares =
            TotalSubnetDelegateStakeShares::<T>::get(from_subnet_id);
        let total_subnet_delegated_stake_balance =
            TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id);

        let from_delegate_balance = Network::<T>::convert_to_balance(
            delegate_shares,
            total_subnet_delegated_stake_shares,
            total_subnet_delegated_stake_balance,
        );
        let prev_total_subnet_delegate_stake_balance =
            TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id);
        let prev_next_id = NextSwapQueueId::<T>::get();

        #[extrinsic_call]
        swap_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()),
            from_subnet_id,
            to_subnet_id,
            delegate_shares,
        );

        let from_delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);
        assert_eq!(from_delegate_shares, 0);

        assert_ne!(
            prev_total_subnet_delegate_stake_balance,
            TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id)
        );
        assert!(
            prev_total_subnet_delegate_stake_balance
                > TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id)
        );

        // Check the queue
        let starting_to_subnet_id = to_subnet_id;
        let call_queue = SwapCallQueue::<T>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake {
                account_id,
                to_subnet_id,
                balance,
            } => {
                assert_eq!(*account_id, delegate_account.clone());
                assert_eq!(*to_subnet_id, starting_to_subnet_id);
                assert_ne!(*balance, 0);
            }
            QueuedSwapCall::SwapToNodeDelegateStake { .. } => assert!(false),
        };

        let next_id = NextSwapQueueId::<T>::get();
        assert_eq!(prev_next_id + 1, next_id);
        let queue = SwapQueueOrder::<T>::get();
        assert!(queue
            .first()
            .map_or(false, |&first_id| first_id == prev_next_id));
    }

    #[benchmark]
    fn transfer_delegate_stake() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_account.clone(),
            (DEFAULT_DELEGATE_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::add_to_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()).into(),
            subnet_id,
            DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
        ));

        let to_delegate_account: T::AccountId = funded_account::<T>("to_delegate_account", 0);

        let delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);

        #[extrinsic_call]
        transfer_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()),
            subnet_id,
            to_delegate_account.clone(),
            delegate_shares,
        );

        assert_eq!(
            0,
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id)
        );
        assert_eq!(
            delegate_shares,
            AccountSubnetDelegateStakeShares::<T>::get(to_delegate_account.clone(), subnet_id)
        )
    }

    #[benchmark]
    fn remove_delegate_stake() {
        let end = 12;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_account.clone(),
            (DEFAULT_DELEGATE_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));
        assert_ok!(Network::<T>::add_to_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()).into(),
            subnet_id,
            DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
        ));
        let delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);

        let total_subnet_delegated_stake_shares =
            TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
        let total_subnet_delegated_stake_balance =
            TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

        let delegate_balance = Network::<T>::convert_to_balance(
            delegate_shares,
            total_subnet_delegated_stake_shares,
            total_subnet_delegated_stake_balance,
        );

        let block = get_current_block_as_u32::<T>();

        #[extrinsic_call]
        remove_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()),
            subnet_id,
            delegate_shares,
        );

        let unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<T>::get(delegate_account.clone());
        assert_eq!(unbondings.len(), 1);
        let (unbonding_block, balance) = unbondings.iter().next().unwrap();
        assert_eq!(
            *unbonding_block,
            block + DelegateStakeCooldownEpochs::<T>::get() * T::EpochLength::get()
        );
        assert_eq!(*balance, delegate_balance);
    }

    #[benchmark]
    fn donate_delegate_stake() {
        let end = 12;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_account.clone(),
            (DEFAULT_DELEGATE_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::add_to_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()).into(),
            subnet_id,
            DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
        ));

        let delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
        let total_subnet_delegated_stake_shares =
            TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
        let total_subnet_delegated_stake_balance =
            TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

        let delegate_balance = Network::<T>::convert_to_balance(
            delegate_shares,
            total_subnet_delegated_stake_shares,
            total_subnet_delegated_stake_balance,
        );

        let funder = funded_account::<T>("funder", 0);

        #[extrinsic_call]
        donate_delegate_stake(
            RawOrigin::Signed(funder),
            subnet_id,
            DEFAULT_SUBNET_NODE_STAKE,
        );

        let increased_delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
        let increased_total_subnet_delegated_stake_shares =
            TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
        let increased_total_subnet_delegated_stake_balance =
            TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

        let increased_delegate_balance = Network::<T>::convert_to_balance(
            increased_delegate_shares,
            increased_total_subnet_delegated_stake_shares,
            increased_total_subnet_delegated_stake_balance,
        );
        assert_eq!(
            increased_total_subnet_delegated_stake_balance,
            total_subnet_delegated_stake_balance + DEFAULT_SUBNET_NODE_STAKE
        );

        assert_ne!(increased_delegate_balance, delegate_balance);
        assert!(increased_delegate_balance > delegate_balance);
    }

    #[benchmark]
    fn add_to_node_delegate_stake() {
        let end = 12;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let subnet_node_id = end;

        let delegate_node_account: T::AccountId = funded_account::<T>("delegate_node_account", 0);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_node_account.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        #[extrinsic_call]
        add_to_node_delegate_stake(
            RawOrigin::Signed(delegate_node_account.clone()),
            subnet_id,
            subnet_node_id,
            DEFAULT_SUBNET_NODE_STAKE,
        );

        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((
            delegate_node_account.clone(),
            subnet_id,
            subnet_node_id,
        ));
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);

        let account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        assert!(
            (account_node_delegate_stake_balance
                >= Network::<T>::percent_mul(DEFAULT_SUBNET_NODE_STAKE, 990000000000000000))
                && (account_node_delegate_stake_balance <= DEFAULT_SUBNET_NODE_STAKE)
        );
    }

    #[benchmark]
    fn swap_node_delegate_stake() {
        let end = 12;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let from_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let from_subnet_node_id = end;

        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME_2.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let to_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME_2.into()).unwrap();
        let to_subnet_node_id = end;

        let delegate_node_account: T::AccountId = funded_account::<T>("delegate_node_account", 0);

        assert_ok!(Network::<T>::add_to_node_delegate_stake(
            RawOrigin::Signed(delegate_node_account.clone()).into(),
            from_subnet_id,
            from_subnet_node_id,
            DEFAULT_SUBNET_NODE_STAKE
        ));

        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<T>::get(from_subnet_id, from_subnet_node_id);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<T>::get(from_subnet_id, from_subnet_node_id);

        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((
            delegate_node_account.clone(),
            from_subnet_id,
            from_subnet_node_id,
        ));
        let account_node_delegate_stake_shares_to_be_removed =
            account_node_delegate_stake_shares / 2;

        let expected_balance_to_be_removed = Network::<T>::convert_to_balance(
            account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        let expected_post_balance = Network::<T>::convert_to_balance(
            account_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_balance - expected_balance_to_be_removed,
        );
        let pre_transfer_balance = T::Currency::free_balance(&delegate_node_account.clone());
        let prev_next_id = NextSwapQueueId::<T>::get();

        #[extrinsic_call]
        swap_node_delegate_stake(
            RawOrigin::Signed(delegate_node_account.clone()),
            from_subnet_id,
            from_subnet_node_id,
            to_subnet_id,
            to_subnet_node_id,
            account_node_delegate_stake_shares_to_be_removed,
        );

        let post_transfer_balance = T::Currency::free_balance(&delegate_node_account.clone());
        assert_eq!(pre_transfer_balance, post_transfer_balance);

        //
        // from subnet ID and Subnet node 1
        // Get accounts delegate stake info from staking to node 1 (now removed partial)
        //
        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((
            delegate_node_account.clone(),
            from_subnet_id,
            to_subnet_node_id,
        ));
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<T>::get(from_subnet_id, to_subnet_node_id);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<T>::get(from_subnet_id, to_subnet_node_id);

        let account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        assert_eq!(account_node_delegate_stake_balance, expected_post_balance);

        //
        // Check queue
        //
        let starting_to_subnet_id = to_subnet_id;
        let starting_to_subnet_node_id = to_subnet_node_id;
        let call_queue = SwapCallQueue::<T>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake { .. } => assert!(false),
            QueuedSwapCall::SwapToNodeDelegateStake {
                account_id,
                to_subnet_id,
                to_subnet_node_id,
                balance,
            } => {
                assert_eq!(*account_id, delegate_node_account.clone());
                assert_eq!(*to_subnet_id, starting_to_subnet_id);
                assert_eq!(*to_subnet_node_id, starting_to_subnet_node_id);
                assert_ne!(*balance, 0);
            }
        };

        let next_id = NextSwapQueueId::<T>::get();
        assert_eq!(prev_next_id + 1, next_id);
        let queue = SwapQueueOrder::<T>::get();
        assert!(queue
            .first()
            .map_or(false, |&first_id| first_id == prev_next_id));
    }

    #[benchmark]
    fn transfer_node_delegate_stake() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let subnet_node_id = 1;

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_account.clone(),
            (DEFAULT_DELEGATE_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::add_to_node_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()).into(),
            subnet_id,
            subnet_node_id,
            DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
        ));

        let to_delegate_account: T::AccountId = funded_account::<T>("to_delegate_account", 0);

        let delegate_shares = AccountNodeDelegateStakeShares::<T>::get((
            delegate_account.clone(),
            subnet_id,
            subnet_node_id,
        ));

        #[extrinsic_call]
        transfer_node_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()),
            subnet_id,
            subnet_node_id,
            to_delegate_account.clone(),
            delegate_shares,
        );

        assert_eq!(
            0,
            AccountNodeDelegateStakeShares::<T>::get((
                delegate_account.clone(),
                subnet_id,
                subnet_node_id
            ))
        );
        assert_eq!(
            delegate_shares,
            AccountNodeDelegateStakeShares::<T>::get((
                to_delegate_account.clone(),
                subnet_id,
                subnet_node_id
            ))
        )
    }

    #[benchmark]
    fn remove_node_delegate_stake() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let subnet_node_id = 1;

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_account.clone(),
            (DEFAULT_DELEGATE_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::add_to_node_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()).into(),
            subnet_id,
            subnet_node_id,
            DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
        ));

        let delegate_shares = AccountNodeDelegateStakeShares::<T>::get((
            delegate_account.clone(),
            subnet_id,
            subnet_node_id,
        ));

        #[extrinsic_call]
        remove_node_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()),
            subnet_id,
            subnet_node_id,
            delegate_shares,
        );

        assert_eq!(
            0,
            AccountNodeDelegateStakeShares::<T>::get((
                delegate_account.clone(),
                subnet_id,
                subnet_node_id
            ))
        );
    }

    #[benchmark]
    fn donate_node_delegate_stake() {
        let end = 12;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let subnet_node_id = end;

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

        let pre_total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);

        #[extrinsic_call]
        donate_node_delegate_stake(
            RawOrigin::Signed(delegate_account),
            subnet_id,
            subnet_node_id,
            DEFAULT_SUBNET_NODE_STAKE,
        );

        let post_total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);

        assert_eq!(
            pre_total_node_delegate_stake_balance + DEFAULT_SUBNET_NODE_STAKE,
            post_total_node_delegate_stake_balance
        );
    }

    #[benchmark]
    fn swap_from_node_to_subnet() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let from_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let from_subnet_node_id = end;

        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME_2.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let to_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME_2.into()).unwrap();

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_account.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::add_to_node_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()).into(),
            from_subnet_id,
            from_subnet_node_id,
            DEFAULT_SUBNET_NODE_STAKE
        ));

        let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((
            delegate_account.clone(),
            from_subnet_id,
            from_subnet_node_id,
        ));
        let total_node_delegate_stake_balance =
            NodeDelegateStakeBalance::<T>::get(from_subnet_id, from_subnet_node_id);
        let total_node_delegate_stake_shares =
            TotalNodeDelegateStakeShares::<T>::get(from_subnet_id, from_subnet_node_id);

        let account_node_delegate_stake_shares_to_be_removed =
            account_node_delegate_stake_shares / 2;

        let expected_balance_to_be_removed = Network::<T>::convert_to_balance(
            account_node_delegate_stake_shares_to_be_removed,
            total_node_delegate_stake_shares,
            total_node_delegate_stake_balance,
        );

        let before_transfer_tensor = T::Currency::free_balance(&delegate_account.clone());

        let prev_next_id = NextSwapQueueId::<T>::get();

        #[extrinsic_call]
        swap_from_node_to_subnet(
            RawOrigin::Signed(delegate_account.clone()),
            from_subnet_id,
            from_subnet_node_id,
            to_subnet_id,
            account_node_delegate_stake_shares_to_be_removed,
        );

        let from_delegate_shares = AccountNodeDelegateStakeShares::<T>::get((
            delegate_account.clone(),
            from_subnet_id,
            from_subnet_node_id,
        ));
        assert_eq!(
            from_delegate_shares,
            account_node_delegate_stake_shares_to_be_removed
        );

        let starting_to_subnet_id = to_subnet_id;
        let call_queue = SwapCallQueue::<T>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake {
                account_id,
                to_subnet_id,
                balance,
            } => {
                assert_eq!(*account_id, delegate_account.clone());
                assert_eq!(*to_subnet_id, starting_to_subnet_id);
                assert_ne!(*balance, 0);
            }
            QueuedSwapCall::SwapToNodeDelegateStake { .. } => assert!(false),
        };

        let next_id = NextSwapQueueId::<T>::get();
        assert_eq!(prev_next_id + 1, next_id);
        let queue = SwapQueueOrder::<T>::get();
        assert!(queue
            .first()
            .map_or(false, |&first_id| first_id == prev_next_id));
    }

    #[benchmark]
    fn swap_from_subnet_to_node() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let from_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let from_subnet_node_id = end;

        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME_2.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let to_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME_2.into()).unwrap();
        let to_subnet_node_id = end;

        let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &delegate_account.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::add_to_delegate_stake(
            RawOrigin::Signed(delegate_account.clone()).into(),
            from_subnet_id,
            DEFAULT_SUBNET_NODE_STAKE
        ));

        let account_node_delegate_stake_shares =
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);

        let before_transfer_tensor = T::Currency::free_balance(&delegate_account.clone());

        let prev_next_id = NextSwapQueueId::<T>::get();

        #[extrinsic_call]
        swap_from_subnet_to_node(
            RawOrigin::Signed(delegate_account.clone()),
            from_subnet_id,
            to_subnet_id,
            to_subnet_node_id,
            account_node_delegate_stake_shares,
        );

        let from_delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);
        assert_eq!(from_delegate_shares, 0);

        let starting_to_subnet_id = to_subnet_id;
        let starting_to_subnet_node_id = to_subnet_node_id;
        let call_queue = SwapCallQueue::<T>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake { .. } => assert!(false),
            QueuedSwapCall::SwapToNodeDelegateStake {
                account_id,
                to_subnet_id,
                to_subnet_node_id,
                balance,
            } => {
                assert_eq!(*account_id, delegate_account.clone());
                assert_eq!(*to_subnet_id, starting_to_subnet_id);
                assert_eq!(*to_subnet_node_id, starting_to_subnet_node_id);
                assert_ne!(*balance, 0);
            }
        };

        let next_id = NextSwapQueueId::<T>::get();
        assert_eq!(prev_next_id + 1, next_id);
        let queue = SwapQueueOrder::<T>::get();
        assert!(queue
            .first()
            .map_or(false, |&first_id| first_id == prev_next_id));
    }

    #[benchmark]
    fn propose_attestation() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let end = 5;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

        let epoch_length = T::EpochLength::get();
        let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

        set_block_to_subnet_slot_epoch::<T>(epoch, subnet_id);
        let subnet_epoch = Network::<T>::get_current_subnet_epoch_as_u32(subnet_id);

        Network::<T>::elect_validator(subnet_id, subnet_epoch, get_current_block_as_u32::<T>());

        let validator_id = SubnetElectedValidator::<T>::get(subnet_id, subnet_epoch as u32);
        assert!(validator_id != None, "Validator is None");

        let hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data::<T>(subnet_id, max_subnet_nodes, 0, end);

        #[extrinsic_call]
        propose_attestation(
            RawOrigin::Signed(hotkey.clone()),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        );

        let submission =
            SubnetConsensusSubmission::<T>::get(subnet_id, subnet_epoch as u32).unwrap();

        assert_eq!(
            submission.validator_id,
            validator_id.unwrap(),
            "Err: validator"
        );
        assert_eq!(
            submission.data.len(),
            subnet_node_data_vec.clone().len(),
            "Err: data len"
        );
        assert_eq!(submission.attests.len(), 1, "Err: attests"); // validator auto-attests
    }

    #[benchmark]
    fn attest() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let end = 5;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
        let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

        let epoch_length = T::EpochLength::get();
        let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

        set_block_to_subnet_slot_epoch::<T>(epoch, subnet_id);
        let subnet_epoch = Network::<T>::get_current_subnet_epoch_as_u32(subnet_id);

        Network::<T>::elect_validator(subnet_id, subnet_epoch, get_current_block_as_u32::<T>());

        let validator_id = SubnetElectedValidator::<T>::get(subnet_id, subnet_epoch as u32);
        assert!(validator_id != None, "Validator is None");

        let hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id.unwrap()).unwrap();

        let subnet_node_data_vec =
            get_subnet_node_consensus_data::<T>(subnet_id, max_subnet_nodes, 0, end);

        assert_ok!(Network::<T>::propose_attestation(
            RawOrigin::Signed(hotkey.clone()).into(),
            subnet_id,
            subnet_node_data_vec.clone(),
            None,
            None,
            None,
            None,
        ));

        // Might be the same ID as validator_id
        let mut attester = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);
        let mut attester_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, attester.clone()).unwrap();

        // Make sure attestor isn't validator
        // Loop through nodes until we find the first non-validator
        if validator_id == Some(attester_subnet_node_id) {
            for n in 0..end {
                let _n = n + 1;
                attester = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, _n);
                attester_subnet_node_id =
                    HotkeySubnetNodeId::<T>::get(subnet_id, attester.clone()).unwrap();
                if Some(attester_subnet_node_id) != validator_id {
                    break;
                }
            }
        }

        let current_block_number = get_current_block_as_u32::<T>();

        #[extrinsic_call]
        attest(RawOrigin::Signed(attester.clone()), subnet_id, None);

        let submission =
            SubnetConsensusSubmission::<T>::get(subnet_id, subnet_epoch as u32).unwrap();

        // validator + attester
        assert_eq!(submission.attests.len(), 2 as usize);
        assert_eq!(
            submission
                .attests
                .get(&(attester_subnet_node_id))
                .unwrap()
                .block,
            current_block_number
        );
    }

    #[benchmark]
    fn update_unique() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        let unique: Vec<u8> = "a".into();
        let bounded_unique: BoundedVec<u8, DefaultMaxVectorLength> =
            unique.try_into().expect("String too long");

        #[extrinsic_call]
        update_unique(
            RawOrigin::Signed(hotkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
            Some(bounded_unique.clone()),
        );

        assert_eq!(
            SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id).unique,
            Some(bounded_unique.clone())
        )
    }

    #[benchmark]
    fn update_non_unique() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        let non_unique: Vec<u8> = "a".into();
        let bounded_non_unique: BoundedVec<u8, DefaultMaxVectorLength> =
            non_unique.try_into().expect("String too long");

        #[extrinsic_call]
        update_non_unique(
            RawOrigin::Signed(hotkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
            Some(bounded_non_unique.clone()),
        );

        assert_eq!(
            SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id).non_unique,
            Some(bounded_non_unique.clone())
        )
    }

    #[benchmark]
    fn update_coldkey() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end);
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);
        let new_coldkey: T::AccountId = get_account::<T>("new_coldkey", 0);

        #[extrinsic_call]
        update_coldkey(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            new_coldkey.clone(),
        );

        let key_owner = HotkeyOwner::<T>::get(hotkey.clone());
        assert_eq!(key_owner, new_coldkey.clone());
    }

    #[benchmark]
    fn update_hotkey() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end);
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);
        let new_hotkey: T::AccountId = get_account::<T>("new_coldkey", 0);

        #[extrinsic_call]
        update_hotkey(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            new_hotkey.clone(),
        );

        let key_owner = HotkeyOwner::<T>::get(new_hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());
    }

    #[benchmark]
    fn update_peer_id() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end);
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        let new_peer = peer(1);

        #[extrinsic_call]
        update_peer_id(
            RawOrigin::Signed(coldkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
            new_peer.clone(),
        );

        assert_eq!(
            SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id).peer_id,
            new_peer.clone()
        )
    }

    #[benchmark]
    fn update_bootnode() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end);
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        let bootnode: Vec<u8> = "new-bootnode".into();
        let bounded_bootnode: BoundedVec<u8, DefaultMaxVectorLength> =
            bootnode.try_into().expect("String too long");

        #[extrinsic_call]
        update_bootnode(
            RawOrigin::Signed(coldkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
            Some(bounded_bootnode.clone()),
        );

        assert_eq!(
            SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id).bootnode,
            Some(bounded_bootnode.clone())
        )
    }

    #[benchmark]
    fn update_bootnode_peer_id() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end);
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        let new_peer = peer(1);

        #[extrinsic_call]
        update_bootnode_peer_id(
            RawOrigin::Signed(coldkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
            new_peer.clone(),
        );

        assert_eq!(
            SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id).bootnode_peer_id,
            new_peer.clone()
        )
    }

    #[benchmark]
    fn update_client_peer_id() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end);
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        let new_peer = peer(1);

        #[extrinsic_call]
        update_client_peer_id(
            RawOrigin::Signed(coldkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
            new_peer.clone(),
        );

        assert_eq!(
            SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id).client_peer_id,
            new_peer.clone()
        )
    }

    #[benchmark]
    fn register_overwatch_node() {
        let hotkey = get_account::<T>("overwatch_node", 2);
        let coldkey_n = 1;
        let coldkey = get_account::<T>("overwatch_node", coldkey_n);
        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        make_overwatch_qualified::<T>(coldkey_n);

        // increase
        let multipler = OverwatchEpochLengthMultiplier::<T>::get();
        increase_epochs::<T>(OverwatchEpochLengthMultiplier::<T>::get() + 1);

        #[extrinsic_call]
        register_overwatch_node(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE,
        );

        assert_eq!(
            AccountOverwatchStake::<T>::get(hotkey.clone()),
            DEFAULT_SUBNET_NODE_STAKE
        );
    }

    #[benchmark]
    fn remove_overwatch_node() {
        let hotkey = get_account::<T>("overwatch_node", 2);
        let coldkey_n = 1;
        let coldkey = get_account::<T>("overwatch_node", coldkey_n);
        make_overwatch_qualified::<T>(coldkey_n);

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        // increase
        let multipler = OverwatchEpochLengthMultiplier::<T>::get();
        increase_epochs::<T>(OverwatchEpochLengthMultiplier::<T>::get() + 1);

        assert_ok!(Network::<T>::register_overwatch_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE
        ));

        // Sanity check
        assert_ne!(OverwatchNodes::<T>::try_get(1), Err(()));

        #[extrinsic_call]
        remove_overwatch_node(RawOrigin::Signed(coldkey.clone()), 1);

        assert_eq!(OverwatchNodes::<T>::try_get(1), Err(()));
    }

    #[benchmark]
    fn anyone_remove_overwatch_node() {
        MinSubnetRegistrationEpochs::<T>::set(10000);
        OverwatchEpochLengthMultiplier::<T>::set(2);
        OverwatchMinDiversificationRatio::<T>::set(1000000000000000000);
        OverwatchMinRepScore::<T>::set(1000000000000000000);
        OverwatchMinAvgAttestationRatio::<T>::set(1000000000000000000);
        OverwatchMinAge::<T>::set(1000);

        let hotkey = get_account::<T>("overwatch_node", 2);
        let coldkey_n = 1;
        let coldkey = get_account::<T>("overwatch_node", coldkey_n);
        make_overwatch_qualified::<T>(coldkey_n);

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        // increase
        let multipler = OverwatchEpochLengthMultiplier::<T>::get();
        increase_epochs::<T>(OverwatchEpochLengthMultiplier::<T>::get() + 1);

        assert_ok!(Network::<T>::register_overwatch_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE
        ));

        // Sanity check
        assert_ne!(OverwatchNodes::<T>::try_get(1), Err(()));

        make_overwatch_unqualified::<T>(coldkey_n);

        #[extrinsic_call]
        anyone_remove_overwatch_node(RawOrigin::Signed(coldkey.clone()), 1);

        assert_eq!(OverwatchNodes::<T>::try_get(1), Err(()));
    }

    #[benchmark]
    fn set_overwatch_node_peer_id() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let hotkey = get_account::<T>("overwatch_node", 2);
        let coldkey_n = 1;
        let coldkey = get_account::<T>("overwatch_node", coldkey_n);
        make_overwatch_qualified::<T>(coldkey_n);

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        // increase
        let multipler = OverwatchEpochLengthMultiplier::<T>::get();
        increase_epochs::<T>(OverwatchEpochLengthMultiplier::<T>::get() + 1);

        assert_ok!(Network::<T>::register_overwatch_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE
        ));

        let id = HotkeyOverwatchNodeId::<T>::get(hotkey.clone()).unwrap();

        let peer_id = peer(1);

        #[extrinsic_call]
        set_overwatch_node_peer_id(
            RawOrigin::Signed(coldkey.clone()),
            subnet_id,
            id,
            peer_id.clone(),
        );

        let exists = OverwatchNodeIndex::<T>::get(id)
            .get(&subnet_id)
            .map_or(false, |x_peer_id| *x_peer_id == peer_id.clone());
        assert!(exists);
    }

    #[benchmark]
    fn commit_overwatch_subnet_weights() {
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let hotkey = get_account::<T>("overwatch_node", 2);
        let coldkey_n = 1;
        let coldkey = get_account::<T>("overwatch_node", coldkey_n);
        make_overwatch_qualified::<T>(coldkey_n);

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        // increase
        let multipler = OverwatchEpochLengthMultiplier::<T>::get();
        increase_epochs::<T>(OverwatchEpochLengthMultiplier::<T>::get() + 1);

        assert_ok!(Network::<T>::register_overwatch_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE
        ));

        let id = HotkeyOverwatchNodeId::<T>::get(hotkey.clone()).unwrap();

        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit::<T>(weight, salt.clone());

        let overwatch_epoch = Network::<T>::get_current_overwatch_epoch_as_u32();

        #[extrinsic_call]
        commit_overwatch_subnet_weights(
            RawOrigin::Signed(coldkey.clone()),
            id,
            vec![OverwatchCommit {
                subnet_id,
                weight: commit_hash,
            }],
        );

        let stored = OverwatchCommits::<T>::get((overwatch_epoch, id, subnet_id)).unwrap();
        assert_eq!(stored, commit_hash);
    }

    #[benchmark]
    fn commit_overwatch_subnet_weights_v2(x: Linear<1, 65>) {
        // ENSURE EPOCH LENGTH IS BOAVE MAX LINEAR
        /// x: subnets
        // overwatch nodes
        let hotkey = get_account::<T>("overwatch_node", 2);
        let coldkey_n = 1;
        let coldkey = get_account::<T>("overwatch_node", coldkey_n);

        // Activate subnets
        let end = 4;
        for s in 0..x {
            let path: Vec<u8> = format!("subnet-name-{s}").into();
            build_activated_subnet::<T>(
                path,
                0,
                end,
                DEFAULT_DEPOSIT_AMOUNT,
                DEFAULT_SUBNET_NODE_STAKE,
            );
            increase_epochs::<T>(10000);
        }

        make_overwatch_node_qualified::<T>(coldkey_n, x);

        let alice = get_alice::<T>();
        assert_ok!(T::Currency::transfer(
            &alice, // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 1000)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));
        // increase epochs if needed (due to linear we can't after)
        let overwatch_epoch = Network::<T>::get_current_overwatch_epoch_as_u32();
        if overwatch_epoch == 0 {
            let multipler = OverwatchEpochLengthMultiplier::<T>::get();
            increase_epochs::<T>(OverwatchEpochLengthMultiplier::<T>::get() + 1);
        }
        assert_ok!(Network::<T>::register_overwatch_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE
        ));

        let id = HotkeyOverwatchNodeId::<T>::get(hotkey.clone()).unwrap();

        // universal commits for testing
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit::<T>(weight, salt.clone());

        let mut commits: Vec<OverwatchCommit<T::Hash>> = Vec::new();
        for s in 0..x {
            let path: Vec<u8> = format!("subnet-name-{s}").into();
            let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path.clone()).unwrap();
            commits.push(OverwatchCommit {
                subnet_id,
                weight: commit_hash,
            });
        }

        let epoch = Network::<T>::get_current_epoch_as_u32();
        set_block_to_overwatch_commit_block::<T>(epoch);

        let overwatch_epoch = Network::<T>::get_current_overwatch_epoch_as_u32();

        #[extrinsic_call]
        commit_overwatch_subnet_weights(RawOrigin::Signed(coldkey.clone()), id, commits);

        for s in 0..x {
            let path: Vec<u8> = format!("subnet-name-{s}").into();
            let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path.clone()).unwrap();
            let stored = OverwatchCommits::<T>::get((overwatch_epoch, id, subnet_id)).unwrap();
            assert_eq!(stored, commit_hash);
        }
    }

    #[benchmark]
    fn reveal_overwatch_subnet_weights(x: Linear<1, 65>) {
        // ENSURE EPOCH LENGTH IS BOAVE MAX LINEAR
        /// x: subnets
        // overwatch nodes
        let hotkey = get_account::<T>("overwatch_node", 2);
        let coldkey_n = 1;
        let coldkey = get_account::<T>("overwatch_node", coldkey_n);

        // Activate subnets
        let end = 4;
        for s in 0..x {
            let path: Vec<u8> = format!("subnet-name-{s}").into();
            build_activated_subnet::<T>(
                path,
                0,
                end,
                DEFAULT_DEPOSIT_AMOUNT,
                DEFAULT_SUBNET_NODE_STAKE,
            );
            increase_epochs::<T>(10000);
        }

        make_overwatch_node_qualified::<T>(coldkey_n, x);

        let alice = get_alice::<T>();
        assert_ok!(T::Currency::transfer(
            &alice, // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 1000)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));
        // increase epochs if needed (due to linear we can't after)
        let overwatch_epoch = Network::<T>::get_current_overwatch_epoch_as_u32();
        if overwatch_epoch == 0 {
            let multipler = OverwatchEpochLengthMultiplier::<T>::get();
            increase_epochs::<T>(OverwatchEpochLengthMultiplier::<T>::get() + 1);
        }
        assert_ok!(Network::<T>::register_overwatch_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE
        ));

        let id = HotkeyOverwatchNodeId::<T>::get(hotkey.clone()).unwrap();

        // universal commits for testing
        let weight: u128 = 123456;
        let salt: Vec<u8> = b"secret-salt".to_vec();
        let commit_hash = make_commit::<T>(weight, salt.clone());

        let mut commits: Vec<OverwatchCommit<T::Hash>> = Vec::new();
        let mut reveals: Vec<OverwatchReveal> = Vec::new();
        for s in 0..x {
            let path: Vec<u8> = format!("subnet-name-{s}").into();
            let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path.clone()).unwrap();
            commits.push(OverwatchCommit {
                subnet_id,
                weight: commit_hash,
            });
            reveals.push(OverwatchReveal {
                subnet_id,
                weight,
                salt: salt.clone(),
            })
        }

        let epoch = Network::<T>::get_current_epoch_as_u32();
        set_block_to_overwatch_commit_block::<T>(epoch);

        let overwatch_epoch = Network::<T>::get_current_overwatch_epoch_as_u32();

        assert_ok!(Network::<T>::commit_overwatch_subnet_weights(
            RawOrigin::Signed(coldkey.clone()).into(),
            id,
            commits
        ));

        set_block_to_overwatch_reveal_block::<T>(epoch);

        #[extrinsic_call]
        reveal_overwatch_subnet_weights(RawOrigin::Signed(coldkey.clone()), id, reveals);

        for s in 0..x {
            let path: Vec<u8> = format!("subnet-name-{s}").into();
            let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path.clone()).unwrap();
            let revealed = OverwatchReveals::<T>::get((overwatch_epoch, subnet_id, id)).unwrap();
            assert_eq!(revealed, weight);
        }
    }

    #[benchmark]
    fn add_to_overwatch_stake() {
        let hotkey = get_account::<T>("overwatch_node", 2);
        let coldkey_n = 1;
        let coldkey = get_account::<T>("overwatch_node", coldkey_n);
        make_overwatch_qualified::<T>(coldkey_n);

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        // increase
        let multipler = OverwatchEpochLengthMultiplier::<T>::get();
        increase_epochs::<T>(OverwatchEpochLengthMultiplier::<T>::get() + 1);

        assert_ok!(Network::<T>::register_overwatch_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE
        ));

        let id = HotkeyOverwatchNodeId::<T>::get(hotkey.clone()).unwrap();

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE).try_into().ok().expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        let prev_balance = AccountOverwatchStake::<T>::get(hotkey.clone());

        #[extrinsic_call]
        add_to_overwatch_stake(
            RawOrigin::Signed(coldkey.clone()),
            id,
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE,
        );

        assert_eq!(
            prev_balance + DEFAULT_SUBNET_NODE_STAKE,
            AccountOverwatchStake::<T>::get(hotkey.clone())
        );
    }

    #[benchmark]
    fn remove_overwatch_stake() {
        let hotkey = get_account::<T>("overwatch_node", 2);
        let coldkey_n = 1;
        let coldkey = get_account::<T>("overwatch_node", coldkey_n);
        make_overwatch_qualified::<T>(coldkey_n);

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        // increase
        let multipler = OverwatchEpochLengthMultiplier::<T>::get();
        increase_epochs::<T>(OverwatchEpochLengthMultiplier::<T>::get() + 1);

        assert_ok!(Network::<T>::register_overwatch_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE
        ));

        let id = HotkeyOverwatchNodeId::<T>::get(hotkey.clone()).unwrap();

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE).try_into().ok().expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::add_to_overwatch_stake(
            RawOrigin::Signed(coldkey.clone()).into(),
            id,
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE
        ));

        let prev_balance = AccountOverwatchStake::<T>::get(hotkey.clone());

        #[extrinsic_call]
        remove_overwatch_stake(
            RawOrigin::Signed(coldkey.clone()),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE,
        );

        assert_eq!(
            prev_balance - DEFAULT_SUBNET_NODE_STAKE,
            AccountOverwatchStake::<T>::get(hotkey.clone())
        );
    }

    #[benchmark]
    fn pause() {
        // sanity check
        assert!(!TxPause::<T>::get());

        #[block]
        {
            Network::<T>::do_pause();
        }

        assert!(TxPause::<T>::get());
    }

    #[benchmark]
    fn unpause() {
        // sanity check
        assert!(!TxPause::<T>::get());

        assert_ok!(Network::<T>::do_pause());
        assert!(TxPause::<T>::get());

        #[block]
        {
            Network::<T>::do_unpause();
        }

        assert!(!TxPause::<T>::get());
    }

    #[benchmark]
    fn collective_remove_subnet() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        #[block]
        {
            Network::<T>::do_collective_remove_subnet(subnet_id);
        }

        assert_eq!(SubnetsData::<T>::try_get(subnet_id), Err(()));
    }

    #[benchmark]
    fn collective_remove_subnet_node() {
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let min_nodes = MinSubnetNodes::<T>::get();
        let max_subnets = MaxSubnets::<T>::get();
        let end = 4;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end);

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        #[block]
        {
            Network::<T>::do_collective_remove_subnet_node(subnet_id, hotkey_subnet_node_id);
        }

        assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), end - 1);

        let subnet_node_id = HotkeySubnetNodeId::<T>::try_get(subnet_id, hotkey.clone());
        assert_eq!(subnet_node_id, Err(()));
    }

    #[benchmark]
    fn collective_remove_overwatch_node() {
        let hotkey = get_account::<T>("overwatch_node", 2);
        let coldkey_n = 1;
        let coldkey = get_account::<T>("overwatch_node", coldkey_n);
        make_overwatch_qualified::<T>(coldkey_n);

        assert_ok!(T::Currency::transfer(
            &get_alice::<T>(), // alice
            &coldkey.clone(),
            (DEFAULT_SUBNET_NODE_STAKE + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        // increase
        let multipler = OverwatchEpochLengthMultiplier::<T>::get();
        increase_epochs::<T>(OverwatchEpochLengthMultiplier::<T>::get() + 1);

        assert_ok!(Network::<T>::register_overwatch_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            hotkey.clone(),
            DEFAULT_SUBNET_NODE_STAKE
        ));

        // Sanity check
        assert_ne!(OverwatchNodes::<T>::try_get(1), Err(()));

        let id = HotkeyOverwatchNodeId::<T>::get(hotkey.clone()).unwrap();
        #[block]
        {
            Network::<T>::do_collective_remove_overwatch_node(id);
        }

        assert_eq!(OverwatchNodes::<T>::try_get(id), Err(()));
    }

    #[benchmark]
    fn set_min_subnet_delegate_stake_factor() {
        let value = MinSubnetDelegateStakeFactor::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_min_subnet_delegate_stake_factor(new_value);
        }

        assert_eq!(MinSubnetDelegateStakeFactor::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_subnet_owner_percentage() {
        let value = SubnetOwnerPercentage::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_subnet_owner_percentage(new_value);
        }

        assert_eq!(SubnetOwnerPercentage::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_subnets() {
        let value = MaxSubnets::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_max_subnets(new_value);
        }

        assert_eq!(MaxSubnets::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_bootnodes() {
        let value = MaxBootnodes::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_max_bootnodes(new_value);
        }

        assert_eq!(MaxBootnodes::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_subnet_bootnodes_access() {
        let value = MaxSubnetBootnodeAccess::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_max_subnet_bootnodes_access(new_value);
        }

        assert_eq!(MaxSubnetBootnodeAccess::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_subnet_penalty_count() {
        let value = MaxSubnetPenaltyCount::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_max_subnet_penalty_count(new_value);
        }

        assert_eq!(MaxSubnetPenaltyCount::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_pause_epochs() {
        let value = MaxSubnetPauseEpochs::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_max_pause_epochs(new_value);
        }

        assert_eq!(MaxSubnetPauseEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_delegate_stake_subnet_removal_interval() {
        let new_value = 1;

        #[block]
        {
            Network::<T>::do_set_delegate_stake_subnet_removal_interval(new_value);
        }

        assert_eq!(DelegateStakeSubnetRemovalInterval::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_subnet_removal_intervals() {
        let min = 1;
        let max = 2;

        #[block]
        {
            Network::<T>::do_set_subnet_removal_intervals(min, max);
        }

        assert_eq!(MinSubnetRemovalInterval::<T>::get(), min);
        assert_eq!(MaxSubnetRemovalInterval::<T>::get(), max);
    }

    #[benchmark]
    fn set_subnet_pause_cooldown_epochs() {
        let new_value = 1;

        #[block]
        {
            Network::<T>::do_set_subnet_pause_cooldown_epochs(new_value);
        }

        assert_eq!(SubnetPauseCooldownEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_min_registration_cost() {
        let value = MinRegistrationCost::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_min_registration_cost(new_value);
        }

        assert_eq!(MinRegistrationCost::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_registration_cost_delay_blocks() {
        let value = RegistrationCostDecayBlocks::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_registration_cost_delay_blocks(new_value);
        }

        assert_eq!(RegistrationCostDecayBlocks::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_registration_cost_alpha() {
        let value = RegistrationCostAlpha::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_registration_cost_alpha(new_value);
        }

        assert_eq!(RegistrationCostAlpha::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_new_registration_cost_multiplier() {
        let value = NewRegistrationCostMultiplier::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_new_registration_cost_multiplier(new_value);
        }

        assert_eq!(NewRegistrationCostMultiplier::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_min_delegate_stake_multiplier() {
        let value = MaxMinDelegateStakeMultiplier::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_max_min_delegate_stake_multiplier(new_value);
        }

        assert_eq!(MaxMinDelegateStakeMultiplier::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_churn_limits() {
        #[block]
        {
            Network::<T>::do_set_churn_limits(1, 2);
        }

        assert_eq!(MinChurnLimit::<T>::get(), 1);
        assert_eq!(MaxChurnLimit::<T>::get(), 2);
    }

    #[benchmark]
    fn set_min_queue_epochs() {
        #[block]
        {
            Network::<T>::do_set_queue_epochs(1, 2);
        }

        assert_eq!(MinQueueEpochs::<T>::get(), 1);
        assert_eq!(MaxQueueEpochs::<T>::get(), 2);
    }

    #[benchmark]
    fn set_max_swap_queue_calls_per_block() {
        let value = MaxSwapQueueCallsPerBlock::<T>::get();
        let new_value = value - 1;

        #[block]
        {
            Network::<T>::do_set_max_swap_queue_calls_per_block(new_value);
        }

        assert_eq!(MaxSwapQueueCallsPerBlock::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_min_idle_classification_epochs() {
        let value = MinIdleClassificationEpochs::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_min_idle_classification_epochs(new_value);
        }

        assert_eq!(MinIdleClassificationEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_idle_classification_epochs() {
        let value = MaxIdleClassificationEpochs::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_max_idle_classification_epochs(new_value);
        }

        assert_eq!(MaxIdleClassificationEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_subnet_activation_enactment_epochs() {
        let value = SubnetEnactmentEpochs::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_subnet_activation_enactment_epochs(new_value);
        }

        assert_eq!(SubnetEnactmentEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_included_classification_epochs() {
        let min = 1;
        let max = 2;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_included_classification_epochs(min, max);
        }

        assert_eq!(MinIncludedClassificationEpochs::<T>::get(), min);
        assert_eq!(MaxIncludedClassificationEpochs::<T>::get(), max);
    }

    #[benchmark]
    fn set_min_max_subnet_node_penalties() {
        let min = 5;
        let max = 6;

        #[block]
        {
            Network::<T>::do_set_max_subnet_node_penalties(min, max);
        }

        assert_eq!(MinMaxSubnetNodePenalties::<T>::get(), min);
        assert_eq!(MaxMaxSubnetNodePenalties::<T>::get(), max);
    }

    #[benchmark]
    fn set_subnet_min_stakes() {
        let min = 5;
        let max = 6;

        #[block]
        {
            Network::<T>::do_set_subnet_min_stakes(min, max);
        }

        assert_eq!(MinSubnetMinStake::<T>::get(), min);
        assert_eq!(MaxSubnetMinStake::<T>::get(), max);
    }
    
    #[benchmark]
    fn set_delegate_stake_percentages() {
        let min = 5;
        let max = 6;

        #[block]
        {
            Network::<T>::do_set_delegate_stake_percentages(min, max);
        }

        assert_eq!(MinDelegateStakePercentage::<T>::get(), min);
        assert_eq!(MaxDelegateStakePercentage::<T>::get(), max);
    }

    #[benchmark]
    fn set_max_registered_nodes() {
        let min = 5;
        let max = 6;

        #[block]
        {
            Network::<T>::do_set_max_registered_nodes(min, max);
        }

        assert_eq!(MinMaxRegisteredNodes::<T>::get(), min);
        assert_eq!(MaxMaxRegisteredNodes::<T>::get(), max);
    }

    #[benchmark]
    fn set_max_subnet_delegate_stake_rewards_percentage_change() {
        let value = MaxSubnetDelegateStakeRewardsPercentageChange::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_max_subnet_delegate_stake_rewards_percentage_change(new_value);
        }

        assert_eq!(
            MaxSubnetDelegateStakeRewardsPercentageChange::<T>::get(),
            new_value
        );
    }

    #[benchmark]
    fn set_subnet_delegate_stake_rewards_update_period() {
        let value = SubnetDelegateStakeRewardsUpdatePeriod::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_subnet_delegate_stake_rewards_update_period(new_value);
        }

        assert_eq!(
            SubnetDelegateStakeRewardsUpdatePeriod::<T>::get(),
            new_value
        );
    }

    #[benchmark]
    fn set_min_attestation_percentage() {
        let value = MinAttestationPercentage::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_min_attestation_percentage(new_value);
        }

        assert_eq!(MinAttestationPercentage::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_super_majority_attestation_ratio() {
        let value = SuperMajorityAttestationRatio::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_super_majority_attestation_ratio(new_value);
        }

        assert_eq!(SuperMajorityAttestationRatio::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_base_validator_reward() {
        let value = BaseValidatorReward::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_base_validator_reward(new_value);
        }

        assert_eq!(BaseValidatorReward::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_base_slash_percentage() {
        let value = BaseSlashPercentage::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_base_slash_percentage(new_value);
        }

        assert_eq!(BaseSlashPercentage::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_slash_amount() {
        let value = MaxSlashAmount::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_max_slash_amount(new_value);
        }

        assert_eq!(MaxSlashAmount::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_reputation_increase_factor() {
        let value = ReputationIncreaseFactor::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_reputation_increase_factor(new_value);
        }

        assert_eq!(ReputationIncreaseFactor::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_reputation_decrease_factor() {
        let value = ReputationDecreaseFactor::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_reputation_decrease_factor(new_value);
        }

        assert_eq!(ReputationDecreaseFactor::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_network_max_stake_balance() {
        let value = NetworkMaxStakeBalance::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_network_max_stake_balance(new_value);
        }

        assert_eq!(NetworkMaxStakeBalance::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_min_delegate_stake_deposit() {
        let value = MinDelegateStakeDeposit::<T>::get();
        let new_value = value + 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_min_delegate_stake_deposit(new_value);
        }

        assert_eq!(MinDelegateStakeDeposit::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_node_reward_rate_update_period() {
        let value = NodeRewardRateUpdatePeriod::<T>::get();
        let new_value = value + 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_node_reward_rate_update_period(new_value);
        }

        assert_eq!(NodeRewardRateUpdatePeriod::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_reward_rate_decrease() {
        let value = MaxRewardRateDecrease::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_max_reward_rate_decrease(new_value);
        }

        assert_eq!(MaxRewardRateDecrease::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_subnet_distribution_power() {
        let value = SubnetDistributionPower::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_subnet_distribution_power(new_value);
        }

        assert_eq!(SubnetDistributionPower::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_delegate_stake_weight_factor() {
        let value = DelegateStakeWeightFactor::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_delegate_stake_weight_factor(new_value);
        }

        assert_eq!(DelegateStakeWeightFactor::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_sigmoid_steepness() {
        let value = InflationSigmoidSteepness::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_sigmoid_steepness(new_value);
        }

        assert_eq!(InflationSigmoidSteepness::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_overwatch_nodes() {
        let value = MaxOverwatchNodes::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_max_overwatch_nodes(new_value);
        }

        assert_eq!(MaxOverwatchNodes::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_overwatch_epoch_length_multiplier() {
        let value = OverwatchEpochLengthMultiplier::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_overwatch_epoch_length_multiplier(new_value);
        }

        assert_eq!(OverwatchEpochLengthMultiplier::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_overwatch_commit_cutoff_percent() {
        let value = OverwatchCommitCutoffPercent::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_overwatch_commit_cutoff_percent(new_value);
        }

        assert_eq!(OverwatchCommitCutoffPercent::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_overwatch_node_penalties() {
        let value = MaxOverwatchNodePenalties::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_max_overwatch_node_penalties(new_value);
        }

        assert_eq!(MaxOverwatchNodePenalties::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_overwatch_min_diversification_ratio() {
        let value = OverwatchMinDiversificationRatio::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_overwatch_min_diversification_ratio(new_value);
        }

        assert_eq!(OverwatchMinDiversificationRatio::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_overwatch_min_rep_score() {
        let value = OverwatchMinRepScore::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_overwatch_min_rep_score(new_value);
        }

        assert_eq!(OverwatchMinRepScore::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_overwatch_min_avg_attestation_ratio() {
        let value = OverwatchMinAvgAttestationRatio::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_overwatch_min_avg_attestation_ratio(new_value);
        }

        assert_eq!(OverwatchMinAvgAttestationRatio::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_overwatch_min_age() {
        let value = OverwatchMinAge::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_overwatch_min_age(new_value);
        }

        assert_eq!(OverwatchMinAge::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_overwatch_min_stake_balance() {
        let value = OverwatchMinStakeBalance::<T>::get();
        let new_value = value - 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_overwatch_min_stake_balance(new_value);
        }

        assert_eq!(OverwatchMinStakeBalance::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_min_max_subnet_node() {
        let min = 1;
        let max = 2;

        #[block]
        {
            Network::<T>::do_set_min_max_subnet_node(min, max);
        }

        assert_eq!(MinSubnetNodes::<T>::get(), min);
        assert_eq!(MaxSubnetNodes::<T>::get(), max);
    }

    #[benchmark]
    fn set_tx_rate_limit() {
        let value = TxRateLimit::<T>::get();
        let new_value = value + 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_tx_rate_limit(new_value);
        }

        assert_eq!(TxRateLimit::<T>::get(), new_value);
    }

    #[benchmark]
    fn collective_set_coldkey_overwatch_node_eligibility() {
        let account = get_account::<T>("account", 0);
        let blacklisted = OverwatchNodeBlacklist::<T>::get(&account);
        assert!(!blacklisted);

        #[block]
        {
            Network::<T>::do_collective_set_coldkey_overwatch_node_eligibility(
                account.clone(),
                false,
            );
        }

        let blacklisted = OverwatchNodeBlacklist::<T>::get(&account);
        assert!(blacklisted);
    }

    #[benchmark]
    fn set_min_subnet_registration_epochs() {
        let value = MinSubnetRegistrationEpochs::<T>::get();
        let new_value = value + 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_min_subnet_registration_epochs(new_value);
        }

        assert_eq!(MinSubnetRegistrationEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_subnet_registration_epochs() {
        let value = SubnetRegistrationEpochs::<T>::get();
        let new_value = value + 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_subnet_registration_epochs(new_value);
        }

        assert_eq!(SubnetRegistrationEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_min_active_node_stake_epochs() {
        let value = MinActiveNodeStakeEpochs::<T>::get();
        let new_value = value + 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_min_active_node_stake_epochs(new_value);
        }

        assert_eq!(MinActiveNodeStakeEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_delegate_stake_cooldown_epochs() {
        let value = DelegateStakeCooldownEpochs::<T>::get();
        let new_value = value + 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_delegate_stake_cooldown_epochs(new_value);
        }

        assert_eq!(DelegateStakeCooldownEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_node_delegate_stake_cooldown_epochs() {
        let value = NodeDelegateStakeCooldownEpochs::<T>::get();
        let new_value = value + 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_node_delegate_stake_cooldown_epochs(new_value);
        }

        assert_eq!(NodeDelegateStakeCooldownEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_min_stake_cooldown_epochs() {
        let value = StakeCooldownEpochs::<T>::get();
        let new_value = value + 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_min_stake_cooldown_epochs(new_value);
        }

        assert_eq!(StakeCooldownEpochs::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_max_unbondings() {
        let value = MaxUnbondings::<T>::get();
        let new_value = value + 1;

        let account = get_account::<T>("account", 0);

        #[block]
        {
            Network::<T>::do_set_max_unbondings(new_value);
        }

        assert_eq!(MaxUnbondings::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_sigmoid_midpoint() {
        let value = InflationSigmoidMidpoint::<T>::get();
        let new_value = value + 1;

        #[block]
        {
            Network::<T>::do_set_sigmoid_midpoint(new_value);
        }

        assert_eq!(InflationSigmoidMidpoint::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_maximum_hooks_weight() {
        let new_value = 10;
        let expected_value =
            sp_runtime::Perbill::from_percent(new_value) * T::BlockWeights::get().max_block;

        #[block]
        {
            Network::<T>::do_set_maximum_hooks_weight(new_value);
        }

        assert_eq!(MaximumHooksWeightV2::<T>::get(), expected_value);
    }

    #[benchmark]
    fn set_base_node_burn_amount() {
        let new_value = 1;

        #[block]
        {
            Network::<T>::do_set_base_node_burn_amount(new_value);
        }

        assert_eq!(BaseNodeBurnAmount::<T>::get(), new_value);
    }

    #[benchmark]
    fn set_node_burn_rates() {
        let min = 1;
        let max = 2;

        #[block]
        {
            Network::<T>::do_set_node_burn_rates(min, max);
        }

        assert_eq!(MinNodeBurnRate::<T>::get(), min);
        assert_eq!(MaxNodeBurnRate::<T>::get(), max);
    }

    #[benchmark]
    fn set_max_subnet_node_score_penalty_threshold() {
        let new_value = 1;

        #[block]
        {
            Network::<T>::do_set_max_subnet_node_score_penalty_threshold(new_value);
        }

        assert_eq!(MaxSubnetNodeScorePenaltyThreshold::<T>::get(), new_value);
    }

    #[benchmark]
    fn update_swap_queue() {
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<T>::get();
        let end = MinSubnetNodes::<T>::get();

        let from_subnet_name: Vec<u8> = "subnet-name".into();
        build_activated_subnet::<T>(
            from_subnet_name.clone().into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let from_subnet_id = SubnetName::<T>::get(from_subnet_name.clone()).unwrap();

        let to_subnet_name: Vec<u8> = "subnet-name-2".into();
        build_activated_subnet::<T>(
            to_subnet_name.clone().into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let to_subnet_id = SubnetName::<T>::get(to_subnet_name.clone()).unwrap();

        let account = get_account::<T>("account", 0);

        let _ = T::Currency::deposit_creating(
            &account.clone(),
            (amount + 500).try_into().ok().expect("REASON"),
        );

        let total_subnet_delegate_stake_shares =
            TotalSubnetDelegateStakeShares::<T>::get(from_subnet_id);
        let total_subnet_delegate_stake_balance =
            TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id);

        let mut delegate_stake_to_be_added_as_shares = Network::<T>::convert_to_shares(
            amount,
            total_subnet_delegate_stake_shares,
            total_subnet_delegate_stake_balance,
        );

        if total_subnet_delegate_stake_shares == 0 {
            delegate_stake_to_be_added_as_shares =
                delegate_stake_to_be_added_as_shares.saturating_sub(1000);
        }

        frame_system::Pallet::<T>::set_block_number(u32_to_block::<T>(
            get_current_block_as_u32::<T>()
                + DelegateStakeCooldownEpochs::<T>::get() * T::EpochLength::get()
                + 1,
        ));

        let starting_delegator_balance = T::Currency::free_balance(&account.clone());

        assert_ok!(Network::<T>::add_to_delegate_stake(
            RawOrigin::Signed(account.clone()).into(),
            from_subnet_id,
            amount,
        ));

        let delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(account.clone(), from_subnet_id);
        assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
        assert_ne!(delegate_shares, 0);

        let total_subnet_delegate_stake_shares =
            TotalSubnetDelegateStakeShares::<T>::get(from_subnet_id);
        let total_subnet_delegate_stake_balance =
            TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id);

        let mut from_delegate_balance = Network::<T>::convert_to_balance(
            delegate_shares,
            total_subnet_delegate_stake_shares,
            total_subnet_delegate_stake_balance,
        );
        // The first depositor will lose a percentage of their deposit depending on the size
        // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
        // assert_eq!(from_delegate_balance, delegate_stake_to_be_added_as_shares);

        let prev_total_subnet_delegate_stake_balance =
            TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id);
        let prev_next_id = NextSwapQueueId::<T>::get();

        assert_ok!(Network::<T>::swap_delegate_stake(
            RawOrigin::Signed(account.clone()).into(),
            from_subnet_id,
            to_subnet_id,
            delegate_shares,
        ));
        let from_delegate_shares =
            AccountSubnetDelegateStakeShares::<T>::get(account.clone(), from_subnet_id);
        assert_eq!(from_delegate_shares, 0);

        assert_ne!(
            prev_total_subnet_delegate_stake_balance,
            TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id)
        );
        assert!(
            prev_total_subnet_delegate_stake_balance
                > TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id)
        );

        // Check the queue
        let starting_to_subnet_id = to_subnet_id;
        let call_queue = SwapCallQueue::<T>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake {
                account_id,
                to_subnet_id,
                balance,
            } => {
                assert_eq!(*account_id, account.clone());
                assert_eq!(*to_subnet_id, starting_to_subnet_id);
                assert_ne!(*balance, 0);
            }
            QueuedSwapCall::SwapToNodeDelegateStake { .. } => assert!(false),
        };

        let next_id = NextSwapQueueId::<T>::get();
        assert_eq!(prev_next_id + 1, next_id);
        let queue = SwapQueueOrder::<T>::get();
        assert!(queue
            .first()
            .map_or(false, |&first_id| first_id == prev_next_id));

        // UPDATE

        // Update back to the `from_subnet_id` staying as a `SwapToSubnetDelegateStake`
        let call = QueuedSwapCall::SwapToSubnetDelegateStake {
            account_id: account.clone(),
            to_subnet_id: from_subnet_id,
            balance: u128::MAX,
        };

        #[extrinsic_call]
        update_swap_queue(
            RawOrigin::Signed(account.clone()),
            prev_next_id,
            call.clone(),
        );

        let call_queue = SwapCallQueue::<T>::get(prev_next_id);
        assert_eq!(call_queue.clone().unwrap().id, prev_next_id);
        match &call_queue.clone().unwrap().call {
            QueuedSwapCall::SwapToSubnetDelegateStake {
                account_id,
                to_subnet_id,
                balance,
            } => {
                assert_eq!(*account_id, account.clone());
                assert_eq!(*to_subnet_id, from_subnet_id);
                assert_ne!(*balance, 0);
                assert_ne!(*balance, u128::MAX);
            }
            QueuedSwapCall::SwapToNodeDelegateStake { .. } => assert!(false),
        };
    }

    #[benchmark]
    fn elect_validator(x: Linear<3, 512>) {
        // x: min nodes, max nodes
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            x,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
        assert_eq!(x, active_nodes);

        let slot_list = SubnetNodeElectionSlots::<T>::get(subnet_id);
        assert_eq!(slot_list.len(), active_nodes as usize);

        let subnet_epoch = Network::<T>::get_current_subnet_epoch_as_u32(subnet_id);

        #[block]
        {
            Network::<T>::elect_validator(subnet_id, subnet_epoch, get_current_block_as_u32::<T>());
        }

        assert!(SubnetElectedValidator::<T>::get(subnet_id, subnet_epoch).is_some());
    }

    #[benchmark]
    fn handle_increase_account_delegate_stake() {
        let account_id: T::AccountId = get_account::<T>("account", 0);
        let subnet_id = 1;
        let delegate_stake_to_be_added = 100e+18 as u128;

        // Sanity check
        assert_eq!(
            AccountSubnetDelegateStakeShares::<T>::get(&account_id, subnet_id),
            0
        );
        #[block]
        {
            Network::<T>::handle_increase_account_delegate_stake(
                &account_id,
                subnet_id,
                delegate_stake_to_be_added,
            );
        }

        assert_ne!(
            AccountSubnetDelegateStakeShares::<T>::get(&account_id, subnet_id),
            0
        );
    }

    #[benchmark]
    fn handle_increase_account_node_delegate_stake_shares() {
        let account_id: T::AccountId = get_account::<T>("account", 0);
        let subnet_id = 1;
        let subnet_node_id = 1;
        let node_delegate_stake_to_be_added = 100e+18 as u128;

        // Sanity check
        assert_eq!(
            AccountNodeDelegateStakeShares::<T>::get((&account_id, subnet_id, subnet_node_id)),
            0
        );
        #[block]
        {
            Network::<T>::handle_increase_account_node_delegate_stake_shares(
                &account_id,
                subnet_id,
                subnet_node_id,
                node_delegate_stake_to_be_added,
            );
        }

        assert_ne!(
            AccountNodeDelegateStakeShares::<T>::get((&account_id, subnet_id, subnet_node_id)),
            0
        );
    }

    #[benchmark]
    fn do_remove_subnet(x: Linear<3, 512>) {
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            x,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        #[block]
        {
            let _ = Network::<T>::do_remove_subnet(subnet_id, SubnetRemovalReason::MaxPenalties);
        }

        assert_eq!(SubnetsData::<T>::try_get(subnet_id), Err(()));
    }

    #[benchmark]
    fn add_balance_to_treasury() {
        let amount = 100e+18 as u128;
        let amount_as_balance = Network::<T>::u128_to_balance(amount).unwrap();
        let treasury_account = T::TreasuryAccount::get();
        #[block]
        {
            let _ = Network::<T>::add_balance_to_treasury(amount_as_balance);
        }

        let pot =
            Network::<T>::balance_to_u128(T::Currency::free_balance(&treasury_account)).unwrap();
        assert_eq!(amount + 500, pot); // amount + EXISTENTIAL_DEPOSIT
    }

    #[benchmark]
    fn perform_remove_subnet_node(x: Linear<1, 12>, n: Linear<3, 25>) {
        // x represents subnets
        // n represents node count
        //
        // We use Linear (x) to see the impact of `ColdkeySubnetNodes` in the function
        // and Linear (n) SubnetNodeQueue
        // ColdkeySubnetNodes is impacted by how many subnets a coldkey has
        // SubnetNodeQueue is impacted by how many nodes are in the queue
        NewRegistrationCostMultiplier::<T>::set(1000000000000000000);

        let max_subnets: u32 = Network::<T>::max_subnets();
        let max_subnet_nodes: u32 = Network::<T>::max_subnet_nodes();

        // Activate subnets to test Coldkey Subnet Nodes
        let mut subnet_id = 0;
        for s in 0..x {
            let path: Vec<u8> = format!("subnet-name-{s}").into();
            build_registered_subnet::<T>(
                path.clone(),
                0,
                n,
                DEFAULT_DEPOSIT_AMOUNT,
                DEFAULT_SUBNET_NODE_STAKE,
                false,
            );
            if subnet_id == 0 {
                subnet_id = SubnetName::<T>::get::<Vec<u8>>(path.clone().into()).unwrap();
            }
        }

        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, x);
        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();
        let subnet_node = SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);

        #[block]
        {
            Network::<T>::perform_remove_subnet_node(subnet_id, hotkey_subnet_node_id);
        }

        let subnet_node_id = HotkeySubnetNodeId::<T>::try_get(subnet_id, hotkey.clone());
        assert_eq!(subnet_node_id, Err(()));
    }

    #[benchmark]
    fn slash_validator() {
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let min_subnet_nodes = MinSubnetNodes::<T>::get();
        let end = min_subnet_nodes;

        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end - 1);
        let subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();
        let subnet_node = SubnetNodesData::<T>::get(subnet_id, subnet_node_id);

        #[block]
        {
            Network::<T>::slash_validator(
                subnet_id,
                subnet_node_id,
                1, // attestation percentage
                MinAttestationPercentage::<T>::get(),
                ReputationDecreaseFactor::<T>::get(),
                Network::<T>::get_current_epoch_as_u32(),
            );
        }
    }

    #[benchmark]
    fn add_balance_to_coldkey_account() {
        let coldkey = get_account::<T>("coldkey", 0);
        let amount = 100e+18 as u128;
        let amount_as_balance = Network::<T>::u128_to_balance(amount).unwrap();

        // Sanity
        let balance = T::Currency::free_balance(&coldkey.clone());
        assert_eq!(balance, Network::<T>::u128_to_balance(0).unwrap());

        #[block]
        {
            Network::<T>::add_balance_to_coldkey_account(&coldkey.clone(), amount_as_balance);
        }

        let balance = T::Currency::free_balance(&coldkey.clone());
        assert_eq!(balance, amount_as_balance);
    }

    #[benchmark]
    fn graduate_class() {
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let min_subnet_nodes = MinSubnetNodes::<T>::get();
        let end = min_subnet_nodes;

        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            end,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end + 1);
        let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        let peer_id = get_peer_id::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        let bootnode_peer_id =
            get_bootnode_peer_id::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        let client_peer_id =
            get_client_peer_id::<T>(subnet_id, max_subnet_nodes, max_subnets, end + 1);
        let alice = get_alice::<T>();
        assert_ok!(T::Currency::transfer(
            &alice, // alice
            &coldkey.clone(),
            (DEFAULT_STAKE_TO_BE_ADDED + 500)
                .try_into()
                .ok()
                .expect("REASON"),
            ExistenceRequirement::KeepAlive,
        ));

        assert_ok!(Network::<T>::register_subnet_node(
            RawOrigin::Signed(coldkey.clone()).into(),
            subnet_id,
            hotkey.clone(),
            peer_id.clone(),
            bootnode_peer_id.clone(),
            client_peer_id.clone(),
            None,
            0,
            DEFAULT_STAKE_TO_BE_ADDED,
            None,
            None,
            u128::MAX
        ));

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()).unwrap();

        let mut subnet_node = RegisteredSubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);
        let mut weight_meter = WeightMeter::new();
        Network::<T>::do_activate_subnet_node(
            &mut weight_meter,
            subnet_id,
            subnet_node,
            Network::<T>::get_current_subnet_epoch_as_u32(subnet_id),
        );

        let subnet_node = SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);
        let old_node_class = subnet_node.classification.node_class;

        #[block]
        {
            Network::<T>::graduate_class(
                subnet_id,
                hotkey_subnet_node_id,
                Network::<T>::get_current_epoch_as_u32(),
            );
        }

        let subnet_node = SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);
        let node_class = subnet_node.classification.node_class;
        assert!(node_class > old_node_class);
    }

    #[benchmark]
    fn insert_node_into_election_slot() {
        let subnet_id = 1u32;
        let subnet_node_id = 42u32;

        #[block]
        {
            Network::<T>::insert_node_into_election_slot(subnet_id, subnet_node_id);
        }
    }

    #[benchmark]
    fn increase_coldkey_reputation() {
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let min_subnet_nodes = MinSubnetNodes::<T>::get();
        let end = min_subnet_nodes;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        let coldkey = get_coldkey::<T>(subnet_id, max_subnet_nodes, end - 1);
        let old_reputation = ColdkeyReputation::<T>::get(&coldkey.clone());
        #[block]
        {
            Network::<T>::increase_coldkey_reputation(
                coldkey.clone(),
                100000000000000000000,
                660000000000000000,
                ReputationIncreaseFactor::<T>::get(),
                Network::<T>::get_current_epoch_as_u32(),
            );
        }

        let reputation = ColdkeyReputation::<T>::get(&coldkey.clone());
        assert!(reputation.score > old_reputation.score);
    }

    #[benchmark]
    fn get_min_subnet_delegate_stake_balance() {
        let max_subnets = MaxSubnets::<T>::get();
        let max_subnet_nodes = MaxSubnetNodes::<T>::get();
        let min_subnet_nodes = MinSubnetNodes::<T>::get();
        let end = min_subnet_nodes;
        build_activated_subnet::<T>(
            DEFAULT_SUBNET_NAME.into(),
            0,
            max_subnet_nodes,
            DEFAULT_DEPOSIT_AMOUNT,
            DEFAULT_SUBNET_NODE_STAKE,
        );
        let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

        #[block]
        {
            Network::<T>::get_min_subnet_delegate_stake_balance(subnet_id);
        }
    }

    // // Informational purposes only
    // #[benchmark]
    // fn do_epoch_preliminaries(x: Linear<1, 65>, n: Linear<3, 25>) {
    //     NewRegistrationCostMultiplier::<T>::set(1000000000000000000);
    // 	for s in 0..x {
    // 		let path: Vec<u8> = format!("subnet-name-{s}").into();
    // 		build_activated_subnet::<T>(path, 0, n, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
    // 	}

    //     for s in 0..x {
    // 		let path: Vec<u8> = format!("subnet-name-{s}").into();
    //         let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();
    //         TotalSubnetDelegateStakeBalance::<T>::insert(subnet_id, 0);
    // 	}

    //     increase_epochs::<T>(2);

    //     let mut weight_meter = WeightMeter::new();

    //     #[block]
    //     {
    //         Network::<T>::do_epoch_preliminaries(&mut weight_meter, get_current_block_as_u32::<T>(), Network::<T>::get_current_epoch_as_u32());
    //     }

    //     for s in 0..x {
    // 		let path: Vec<u8> = format!("subnet-name-{s}").into();
    //         assert!(SubnetName::<T>::try_get::<Vec<u8>>(path).is_err());
    // 	}
    // }

    // // Informational purposes only
    // // This benchmark is for benchmarking how much subnets and overwatch nodes this function can handle
    // #[benchmark]
    // fn calculate_overwatch_rewards(x: Linear<1, 65>, o: Linear<1, 65>) {
    //     // Activate subnets
    //     let end = MinSubnetNodes::<T>::get();
    //     NewRegistrationCostMultiplier::<T>::set(1000000000000000000);
    // 	for s in 0..x {
    // 		let path: Vec<u8> = format!("subnet-name-{s}").into();
    // 		build_activated_subnet::<T>(path, 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
    // 	}

    //     let mut node_id_1 = 0;
    //     for n in 0..o {
    //         let _n = o + 1;
    //         let coldkey_n = _n;
    //         let hotkey_n = _n + 64;
    //         let node_id = insert_overwatch_node::<T>(coldkey_n, hotkey_n);
    //         if node_id_1 == 0 {
    //             node_id_1 = node_id;
    //         }
    //         set_overwatch_stake::<T>(hotkey_n, 100);
    //     }

    //     // Sanity
    //     assert_ne!(node_id_1, 0);

    //     let current_overwatch_epoch = Network::<T>::get_current_overwatch_epoch_as_u32();

    //     for s in 0..x {
    //         let path: Vec<u8> = format!("subnet-name-{s}").into();
    //         let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path.clone().into()).unwrap();

    //         for n in 0..o {
    //             let _n = o + 1;
    //             let coldkey_n = _n;
    //             let hotkey_n = _n + 64;
    //             let hotkey = get_account::<T>("overwatch_node", hotkey_n);
    //             let node_id = HotkeyOverwatchNodeId::<T>::get(&hotkey.clone()).unwrap();
    //             submit_overwatch_reveal::<T>(
    //                 current_overwatch_epoch,
    //                 subnet_id,
    //                 node_id,
    //                 500000000000000000,
    //             );
    //         }
    // 	}

    //     // increase overwatch epoch to next overwatch epoch
    //     set_overwatch_epoch::<T>(current_overwatch_epoch + 1);
    //     let current_overwatch_epoch = Network::<T>::get_current_overwatch_epoch_as_u32();

    //     #[block]
    //     {
    //         Network::<T>::calculate_overwatch_rewards();
    //     }

    //     let prev_score = OverwatchNodeWeights::<T>::get(current_overwatch_epoch, node_id_1);
    //     for n in 0..o {
    //         let _n = o + 1;
    //         let coldkey_n = _n;
    //         let hotkey_n = _n + 64;
    //         let hotkey = get_account::<T>("overwatch_node", hotkey_n);
    //         let node_id = HotkeyOverwatchNodeId::<T>::get(&hotkey.clone());
    //         let score = OverwatchNodeWeights::<T>::get(current_overwatch_epoch, node_id_1);
    //         assert_eq!(prev_score, score);
    //     }
    // }

    // // Informational purposes only
    // // Test with min nodes - 25 nodes
    // #[benchmark]
    // fn emission_step(n: Linear<3, 25>) {
    //     NewRegistrationCostMultiplier::<T>::set(1000000000000000000);
    //     let max_subnets = MaxSubnets::<T>::get();
    //     let max_subnet_nodes = MaxSubnetNodes::<T>::get();

    //     build_activated_subnet::<T>(
    //         DEFAULT_SUBNET_NAME.into(),
    //         0,
    //         n,
    //         DEFAULT_DEPOSIT_AMOUNT,
    //         DEFAULT_SUBNET_NODE_STAKE
    //     );
    //     let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

    //     // Get to activation epoch (not needed for this test but do anyway)
    //     increase_epochs::<T>(1);

    //     // Set to correct block to elect a validator
    //     set_block_to_subnet_slot_epoch::<T>(Network::<T>::get_current_epoch_as_u32(), subnet_id);
    //     let subnet_epoch = Network::<T>::get_current_subnet_epoch_as_u32(subnet_id);
    //     Network::<T>::elect_validator(subnet_id, subnet_epoch, Network::<T>::get_current_block_as_u32());

    //     // Run consensus, submit proposal, attest
    //     run_subnet_consensus_step::<T>(subnet_id, None);

    //     // Ensure it worked
    //     let submission = SubnetConsensusSubmission::<T>::try_get(subnet_id, Network::<T>::get_current_subnet_epoch_as_u32(subnet_id));
    //     assert!(submission.is_ok());
    //     assert_eq!(submission.unwrap().attests.len() as u32, n as u32);

    // 	let mut stake_snapshot: BTreeMap<T::AccountId, u128> = BTreeMap::new();
    // 	for n in 0..n {
    //         let _n = n + 1;
    //         let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, _n);
    // 		let stake = AccountSubnetStake::<T>::get(hotkey.clone(), subnet_id);
    // 		stake_snapshot.insert(hotkey.clone(), stake);
    // 	}

    //     increase_epochs::<T>(1);
    //     set_block_to_subnet_slot_epoch::<T>(Network::<T>::get_current_epoch_as_u32(), subnet_id);

    //     // Calc subnet weights
    // 	let _ = Network::<T>::handle_subnet_emission_weights(Network::<T>::get_current_epoch_as_u32());

    //     // Verify weights exist
    //     let subnet_emission_weights = FinalSubnetEmissionWeights::<T>::get(Network::<T>::get_current_epoch_as_u32());
    //     let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
    //     assert!(subnet_weight.is_some());

    // 	#[block]
    // 	{
    // 		let _ = Network::<T>::emission_step(
    // 			Network::<T>::get_current_block_as_u32(),
    // 			Network::<T>::get_current_epoch_as_u32(),
    //             Network::<T>::get_current_subnet_epoch_as_u32(subnet_id),
    // 			subnet_id,
    // 		);
    // 	}

    // 	for n in 0..n {
    //         let _n = n + 1;
    //         let hotkey = get_hotkey::<T>(subnet_id, max_subnet_nodes, max_subnets, _n);
    // 		let stake = AccountSubnetStake::<T>::get(hotkey.clone(), subnet_id);

    // 		if let Some(old_stake) = stake_snapshot.get(&hotkey) {
    // 			assert!(stake > *old_stake);
    // 		} else {
    // 			assert!(false); // auto-fail
    // 		}
    // 	}
    // }

    // // Informational purposes only
    // #[benchmark]
    // fn precheck_subnet_consensus_submission(x: Linear<3, 512>) {
    //     build_activated_subnet::<T>(
    //         DEFAULT_SUBNET_NAME.into(),
    //         0,
    //         x,
    //         DEFAULT_DEPOSIT_AMOUNT,
    //         DEFAULT_SUBNET_NODE_STAKE,
    //     );
    //     let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

    //     let epoch = Network::<T>::get_current_epoch_as_u32();

    //     // ⸺ Generate subnet weights
    //     let _ = Network::<T>::handle_subnet_emission_weights(epoch);
    //     let subnet_emission_weights = FinalSubnetEmissionWeights::<T>::get(epoch);

    //     let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
    //     assert!(subnet_weight.is_some());

    //     // ⸺ Submit consnesus data
    //     let subnet_nodes: Vec<SubnetNode<T::AccountId>> =
    //         Network::<T>::get_active_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);
    //     let subnet_node_count = subnet_nodes.len() as u128;

    //     let consensus_data = get_simulated_consensus_data::<T>(subnet_id, subnet_node_count as u32);

    //     let subnet_epoch = Network::<T>::get_current_subnet_epoch_as_u32(subnet_id);
    //     let current_epoch = Network::<T>::get_current_epoch_as_u32();

    //     // submit data for the previous epoch
    //     SubnetConsensusSubmission::<T>::insert(subnet_id, subnet_epoch - 1, consensus_data);

    //     #[block]
    //     {
    //         let (result, weight) = Network::<T>::precheck_subnet_consensus_submission(subnet_id, subnet_epoch - 1, current_epoch);

    //         // assert SubnetConsensusSubmission exists
    //         assert!(result.is_some(), "Precheck consensus failed");
    //     }

    //     let pen = SubnetPenaltyCount::<T>::get(subnet_id);
    //     assert_eq!(pen, 0);
    // }

    // // Informational purposes only
    // #[benchmark]
    // fn calculate_subnet_weights(x: Linear<1, 65>) {
    //     // Activate subnets
    //     let end = MinSubnetNodes::<T>::get();
    //     NewRegistrationCostMultiplier::<T>::set(1000000000000000000);
    // 	for s in 0..x {
    // 		let path: Vec<u8> = format!("subnet-name-{s}").into();
    // 		build_activated_subnet::<T>(path, 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
    // 	}

    //     let current_overwatch_epoch = Network::<T>::get_current_overwatch_epoch_as_u32();

    //     // Simulate overwatch subnet weights
    //     for s in 0..x {
    //         let path: Vec<u8> = format!("subnet-name-{s}").into();
    //         let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path.clone().into()).unwrap();

    //         OverwatchSubnetWeights::<T>::insert(
    //             current_overwatch_epoch.saturating_add(1),
    //             subnet_id,
    //             500000000000000000,
    //         );
    // 	}

    //     #[block]
    //     {
    //         let (stake_weights_normalized, stake_weights_weight) =
    //             Network::<T>::calculate_subnet_weights(Network::<T>::get_current_epoch_as_u32());
    //         assert!(stake_weights_normalized.len() as u32 == x);
    //     }
    // }

    // impl_benchmark_test_suite!(Network, crate::mock::new_test_ext(), crate::mock::Test);
    impl_benchmark_test_suite!(Network, tests::mock::new_test_ext(), tests::mock::Test);
}
