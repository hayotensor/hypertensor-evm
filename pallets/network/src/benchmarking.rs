//! Benchmarking setup for pallet-network
// frame-omni-bencher v1 benchmark pallet --runtime target/release/wbuild/solochain-template-runtime/solochain_template_runtime.compact.compressed.wasm --extrinsic "" --pallet "pallet_network" --output pallets/network/src/weights.rs

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
// use crate::{
// 	SubnetPaths, 
// 	TotalStake, 
// 	TotalSubnetDelegateStakeBalance, 
// 	TotalSubnetDelegateStakeShares, 
// 	StakeUnbondingLedger,
// 	SubnetRegistrationEpochs,
// 	MinSubnetNodes,
// 	PeerIdSubnetNode,
// 	AccountSubnetStake,
// };
use crate::*;
pub use pallet::*;
use frame_benchmarking::v2::*;
use frame_support::{
	assert_noop, assert_ok,
	traits::{EnsureOrigin, Get, OnInitialize, UnfilteredDispatchable},
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::Vec;
use sp_core::OpaquePeerId as PeerId;
use scale_info::prelude::vec;
use scale_info::prelude::format;
use sp_runtime::SaturatedConversion;
const SEED: u32 = 0;


const DEFAULT_SCORE: u128 = 5000;
const DEFAULT_SUBNET_INIT_COST: u128 = 100e+18 as u128;
const DEFAULT_SUBNET_PATH: &str = "petals-team/StableBeluga2";
const DEFAULT_SUBNET_PATH_2: &str = "petals-team/StableBeluga3";
const DEFAULT_SUBNET_NODE_STAKE: u128 = 1000e+18 as u128;
const DEFAULT_SUBNET_REGISTRATION_BLOCKS: u64 = 130_000;
const DEFAULT_STAKE_TO_BE_ADDED: u128 = 1000e+18 as u128;
const DEFAULT_DELEGATE_STAKE_TO_BE_ADDED: u128 = 1000e+18 as u128;
const DEFAULT_DEPOSIT_AMOUNT: u128 = 10000e+18 as u128;

pub type BalanceOf<T> = <T as Config>::Currency;

fn peer(id: u32) -> PeerId {
  let peer_id = format!("QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N{id}"); 
	PeerId(peer_id.into())
}

fn get_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	caller
}

fn funded_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	// Give the account half of the maximum value of the `Balance` type.
	// Otherwise some transfers will fail with an overflow error.
	let deposit_amount: u128 = MinStakeBalance::<T>::get() + 10000;
	T::Currency::deposit_creating(&caller, deposit_amount.try_into().ok().expect("REASON"));
	caller
}

fn funded_initializer<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	// Give the account half of the maximum value of the `Balance` type.
	// Otherwise some transfers will fail with an overflow error.
	let deposit_amount: u128 = Network::<T>::registration_cost(0) + 1000000;
	T::Currency::deposit_creating(&caller, deposit_amount.try_into().ok().expect("REASON"));
	caller
}

fn increase_epochs<T: Config>(epochs: u32) {
  if epochs == 0 {
    return
  }

  let block = get_current_block_as_u32::<T>();

  let epoch_length = T::EpochLength::get();

  let next_epoch_start_block = (epoch_length * epochs) + block - (block % (epoch_length * epochs));

	frame_system::Pallet::<T>::set_block_number(next_epoch_start_block.into());
}

fn build_activated_subnet<T: Config>(
	subnet_path: Vec<u8>, 
	start: u32, 
	mut end: u32, 
	deposit_amount: u128, 
	amount: u128
) {
  let epoch_length = T::EpochLength::get();
  let block_number = get_current_block_as_u32::<T>();
  let epoch = block_number.saturating_div(epoch_length);
  let next_registration_epoch = Network::<T>::get_next_registration_epoch(epoch);
  increase_epochs::<T>(next_registration_epoch.saturating_sub(epoch));

	let funded_initializer = funded_initializer::<T>("funded_initializer", 0);
	let registration_blocks = MinSubnetRegistrationBlocks::<T>::get();

	let min_nodes = MinSubnetNodes::<T>::get();
	let whitelist = get_coldkey_whitelist::<T>(start, end);

	let register_subnet_data = RegistrationSubnetData {
		path: subnet_path.clone(),
		max_node_registration_epochs: 16,
		node_registration_interval: 0,
		node_activation_interval: 0,
		node_queue_period: 1,
		max_node_penalties: 3,
		coldkey_whitelist: whitelist,
	};

		// --- Register subnet for activation
  assert_ok!(
    Network::<T>::register_subnet(
      RawOrigin::Signed(funded_initializer.clone()).into(),
      register_subnet_data,
    )
  );

  let subnet_id = SubnetPaths::<T>::get(subnet_path.clone()).unwrap();
  let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

  if end == 0 {
    end = min_nodes;
  }

  let epoch = get_current_block_as_u32::<T>() / epoch_length;

  // --- Add subnet nodes
  let block_number = get_current_block_as_u32::<T>();
  let mut amount_staked = 0;
  for n in start+1..end+1 {
		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", n);
		T::Currency::deposit_creating(&subnet_node_account, DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));
    amount_staked += amount;
    assert_ok!(
      Network::<T>::add_subnet_node(
        RawOrigin::Signed(subnet_node_account.clone()).into(),
        subnet_id,
				subnet_node_account.clone(),
        peer(n),
				peer(n),
				0,
        amount,
        None,
        None,
        None,
      ) 
    );
    let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, subnet_node_account.clone());

    let subnet_node_data = SubnetNodesData::<T>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, subnet_node_account.clone());
    assert_eq!(subnet_node_data.delegate_reward_rate, 0);

    let key_owner = HotkeyOwner::<T>::get(subnet_node_data.hotkey.clone());
    assert_eq!(key_owner, subnet_node_account.clone());

    assert_eq!(subnet_node_data.peer_id, peer(n));

    // --- Is ``Validator`` if registered before subnet activation
    assert_eq!(subnet_node_data.classification.class, SubnetNodeClass::Validator);
    assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, epoch));

    let peer_subnet_node_account = PeerIdSubnetNode::<T>::get(subnet_id, peer(n));
    assert_eq!(peer_subnet_node_account, hotkey_subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);
    assert_eq!(account_subnet_stake, amount);
  }

  let total_subnet_stake = TotalSubnetStake::<T>::get(subnet_id);
  assert_eq!(total_subnet_stake, amount_staked);

  let total_stake = TotalStake::<T>::get();
  assert_eq!(total_subnet_stake, amount_staked);


  let min_subnet_delegate_stake = Network::<T>::get_min_subnet_delegate_stake_balance();
  // --- Add the minimum required delegate stake balance to activate the subnet

	let delegate_staker_account: T::AccountId = funded_account::<T>("subnet_node_account", 1);
	T::Currency::deposit_creating(&delegate_staker_account, min_subnet_delegate_stake.try_into().ok().expect("REASON"));
  assert_ok!(
    Network::<T>::add_to_delegate_stake(
      RawOrigin::Signed(delegate_staker_account.clone()).into(),
      subnet_id,
      min_subnet_delegate_stake,
    ) 
  );

  let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
  assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

  let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(&delegate_staker_account, subnet_id);

  // --- Increase blocks to max registration block
	// frame_system::Pallet::<T>::set_block_number(
	// 	frame_system::Pallet::<T>::block_number() + 
	// 	u32_to_block::<T>(subnet.registration_blocks + 1)
	// );

  // let current_block_number = get_current_block_as_u32::<T>();
	let epochs = SubnetRegistrationEpochs::<T>::get();
  increase_epochs::<T>(epochs + 1);

  assert_ok!(
    Network::<T>::activate_subnet(
      RawOrigin::Signed(funded_initializer.clone()).into(),
      subnet_id,
    )
  );
}

// Returns total staked on subnet
// fn build_subnet_nodes<T: Config>(subnet_id: u32, start: u32, end: u32, amount: u128) -> u128 {
//   let mut amount_staked = 0;
//   for n in start+1..end+1 {
//     let subnet_node = funded_account::<T>("subnet_node_account", n);
//     amount_staked += amount;
//     assert_ok!(
//       Network::<T>::add_subnet_node(
//         RawOrigin::Signed(subnet_node).into(),
//         subnet_id,
// 				subnet_node.clone(),
//         peer(n),
// 				peer(n),
// 				0,
//         amount,
// 				None,
// 				None,
// 				None,
//       ) 
//     );
//   }
//   amount_staked
// }

fn subnet_node_data(start: u32, end: u32) -> Vec<SubnetNodeData> {
  // initialize peer consensus data array
  let mut subnet_node_data: Vec<SubnetNodeData> = Vec::new();
  for n in start+1..end+1 {
    let peer_subnet_node_data: SubnetNodeData = SubnetNodeData {
      peer_id: peer(n),
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

pub fn get_current_block_as_u32<T: frame_system::Config>() -> u32 {
	TryInto::try_into(<frame_system::Pallet<T>>::block_number())
		.ok()
		.expect("blockchain will not exceed u32::MAX blocks; QED.")
}

pub fn u128_to_balance<T: frame_system::Config + pallet::Config>(
	input: u128,
) -> Option<
	<<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
> {
	input.try_into().ok()
}

pub fn get_coldkey_whitelist<T: Config>(start: u32, end: u32) -> BTreeSet<T::AccountId> {
  let mut whitelist = BTreeSet::new();
  for n in start+1..end+1 {
    whitelist.insert(funded_account::<T>("subnet_node_account", n));
  }
  whitelist
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register_subnet() {
		let epoch_length = T::EpochLength::get();
    let block_number = get_current_block_as_u32::<T>();
    let epoch = block_number.saturating_div(epoch_length);

		let cost = Network::<T>::registration_cost(epoch);

		let funded_initializer = funded_initializer::<T>("funded_initializer", 0);
		let registration_blocks = MinSubnetRegistrationBlocks::<T>::get();

		let min_nodes = MinSubnetNodes::<T>::get();
		let whitelist = get_coldkey_whitelist::<T>(0, min_nodes);

		let register_subnet_data = RegistrationSubnetData {
			path: DEFAULT_SUBNET_PATH.into(),
			max_node_registration_epochs: 16,
			node_registration_interval: 0,
			node_activation_interval: 0,
			node_queue_period: 1,
			max_node_penalties: 3,
			coldkey_whitelist: whitelist,
		};
	
		let current_block_number = get_current_block_as_u32::<T>();
	
		#[extrinsic_call]
		register_subnet(RawOrigin::Signed(funded_initializer.clone()), register_subnet_data);

		let owner = SubnetOwner::<T>::get(1).unwrap();
		assert_eq!(owner, funded_initializer.clone());
	
		let subnet = SubnetsData::<T>::get(1).unwrap();
		assert_eq!(subnet.id, 1);
		let path: Vec<u8> = DEFAULT_SUBNET_PATH.into();
		assert_eq!(subnet.path, path);
		
    // let minimum_balance = T::Currency::minimum_balance();
    // // let pot = T::Treasury::pot();
		// let pot = <T as Config>::Treasury::pot();
    // assert_eq!(cost, pot + minimum_balance);
	}

	#[benchmark]
	fn activate_subnet() {
		let funded_initializer = funded_initializer::<T>("funded_initializer", 0);
		let start: u32 = 0; 
		let mut end: u32 = 12; 
		let deposit_amount: u128 = DEFAULT_DEPOSIT_AMOUNT;
		let amount: u128 = DEFAULT_SUBNET_NODE_STAKE;
		let min_nodes = MinSubnetNodes::<T>::get();
		let whitelist = get_coldkey_whitelist::<T>(0, end);

		let register_subnet_data = RegistrationSubnetData {
			path: DEFAULT_SUBNET_PATH.into(),
			max_node_registration_epochs: 16,
			node_registration_interval: 0,
			node_activation_interval: 0,
			node_queue_period: 1,
			max_node_penalties: 3,
			coldkey_whitelist: whitelist,
		};

		assert_ok!(Network::<T>::register_subnet(RawOrigin::Signed(funded_initializer.clone()).into(), register_subnet_data));

		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();
		let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

		let epoch_length = T::EpochLength::get();
		let epoch = get_current_block_as_u32::<T>() / epoch_length;
	
		let block_number = get_current_block_as_u32::<T>();
		let mut amount_staked = 0;
		for n in start+1..end+1 {
			let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", n);
			T::Currency::deposit_creating(&subnet_node_account, DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));
			amount_staked += amount;
			assert_ok!(
				Network::<T>::add_subnet_node(
					RawOrigin::Signed(subnet_node_account.clone()).into(),
					subnet_id,
					subnet_node_account.clone(),
					peer(n),
					peer(n),
					0,
					amount,
					None,
					None,
					None,
				) 
			);
			let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

			let subnet_node_id_hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, hotkey_subnet_node_id).unwrap();
			assert_eq!(subnet_node_id_hotkey, subnet_node_account.clone());

			let subnet_node_data = SubnetNodesData::<T>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
			assert_eq!(subnet_node_data.hotkey, subnet_node_account.clone());
			assert_eq!(subnet_node_data.delegate_reward_rate, 0);

			let key_owner = HotkeyOwner::<T>::get(subnet_node_data.hotkey.clone());
			assert_eq!(key_owner, subnet_node_account.clone());

			assert_eq!(subnet_node_data.peer_id, peer(n));

			// --- Is ``Validator`` if registered before subnet activation
			assert_eq!(subnet_node_data.classification.class, SubnetNodeClass::Validator);
			assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, epoch));

			let peer_subnet_node_account = PeerIdSubnetNode::<T>::get(subnet_id, peer(n));
			assert_eq!(peer_subnet_node_account, hotkey_subnet_node_id);

			let account_subnet_stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);
			assert_eq!(account_subnet_stake, amount);
		}
	
		let total_subnet_stake = TotalSubnetStake::<T>::get(subnet_id);
		assert_eq!(total_subnet_stake, amount_staked);
	
		let total_stake = TotalStake::<T>::get();
		assert_eq!(total_subnet_stake, amount_staked);
	
	
		let min_subnet_delegate_stake = Network::<T>::get_min_subnet_delegate_stake_balance();
		// --- Add the minimum required delegate stake balance to activate the subnet
	
		let delegate_staker_account: T::AccountId = funded_account::<T>("subnet_node_account", 1);
		T::Currency::deposit_creating(&delegate_staker_account, min_subnet_delegate_stake.try_into().ok().expect("REASON"));
		assert_ok!(
			Network::<T>::add_to_delegate_stake(
				RawOrigin::Signed(delegate_staker_account.clone()).into(),
				subnet_id,
				min_subnet_delegate_stake,
			) 
		);
	
		let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_staker_account.clone(), subnet_id);
	
		let epochs = SubnetRegistrationEpochs::<T>::get();
		increase_epochs::<T>(epochs + 1);
	
		let current_block_number = get_current_block_as_u32::<T>();
	
		#[extrinsic_call]
		activate_subnet(RawOrigin::Signed(funded_initializer.clone()), subnet_id);

		let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

    assert_eq!(subnet.id, subnet_id);

    // ensure subnet exists and nothing changed but the activation block
		let path: Vec<u8> = DEFAULT_SUBNET_PATH.into();
    assert_eq!(subnet.path, path);
    assert_eq!(subnet.state, SubnetState::Active);
	}

	#[benchmark]
	fn add_subnet_node() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_node_account = funded_account::<T>("subnet_node_account", end+1);

		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		let current_block_number = get_current_block_as_u32::<T>();

		let epoch = get_current_block_as_u32::<T>() / T::EpochLength::get();

		#[extrinsic_call]
		add_subnet_node(
			RawOrigin::Signed(subnet_node_account.clone()), 
			subnet_id, 
			subnet_node_account.clone(),
			peer(end+1), 
			peer(end+1), 
			0,
			DEFAULT_SUBNET_NODE_STAKE,
			None,
			None,
			None,
		);
		
		let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

		let subnet_node_id_hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, hotkey_subnet_node_id).unwrap();
		assert_eq!(subnet_node_id_hotkey, subnet_node_account.clone());

		let subnet_node_data = SubnetNodesData::<T>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
		assert_eq!(subnet_node_data.hotkey, subnet_node_account.clone());
		assert_eq!(subnet_node_data.delegate_reward_rate, 0);

		let key_owner = HotkeyOwner::<T>::get(subnet_node_data.hotkey.clone());
		assert_eq!(key_owner, subnet_node_account.clone());

		assert_eq!(subnet_node_data.peer_id, peer(end+1));

		// --- Is ``Validator`` if registered before subnet activation
		// assert_eq!(subnet_node_data.classification.class, SubnetNodeClass::Queue);
		// assert!(subnet_node_data.has_classification(&SubnetNodeClass::Queue, epoch));

		let peer_subnet_node_account = PeerIdSubnetNode::<T>::get(subnet_id, peer(end+1));
		assert_eq!(peer_subnet_node_account, hotkey_subnet_node_id);

		let account_subnet_stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);
		assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE);
	}

	#[benchmark]
	fn register_subnet_node() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_node_account = funded_account::<T>("subnet_node_account", end+1);

		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		#[extrinsic_call]
		register_subnet_node(
			RawOrigin::Signed(subnet_node_account.clone()), 
			subnet_id, 
			subnet_node_account.clone(),
			peer(end+1), 
			peer(end+1), 
			0,
			DEFAULT_SUBNET_NODE_STAKE,
			None,
			None,
			None,
		);

		assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), end+1);

	}

	#[benchmark]
	fn activate_subnet_node() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_node_account = funded_account::<T>("subnet_node_account", end+1);

		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();
		assert_ok!(
			Network::<T>::register_subnet_node(
				RawOrigin::Signed(subnet_node_account.clone()).into(), 
				subnet_id, 
				subnet_node_account.clone(),
				peer(end+1), 
				peer(end+1), 
				0,
				DEFAULT_SUBNET_NODE_STAKE,
				None,
				None,
				None,
			) 
		);

		let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

		#[extrinsic_call]
		activate_subnet_node(RawOrigin::Signed(subnet_node_account.clone()), subnet_id, hotkey_subnet_node_id);

		assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), end+1);
	}

	// // #[benchmark]
	// // fn deactivate_subnet_node() {
	// // 	let end = 12;
	// // 	build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// // 	let subnet_node_account = funded_account::<T>("subnet_node_account", end+1);

	// // 	let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();
	// // 	assert_ok!(
	// // 		Network::<T>::register_subnet_node(
	// // 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// // 			subnet_id, 
	// // 			peer(end+1), 
	// // 			DEFAULT_SUBNET_NODE_STAKE,
	// // 			None,
	// // 			None,
	// // 			None,
	// // 		) 
	// // 	);

	// // 	assert_ok!(
	// // 		Network::<T>::activate_subnet_node(
	// // 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// // 			subnet_id, 
	// // 		) 
	// // 	);

	// // 	#[extrinsic_call]
	// // 	deactivate_subnet_node(RawOrigin::Signed(subnet_node_account.clone()), subnet_id);

	// // 	assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), end+1);
	// // }

	#[benchmark]
	fn remove_subnet_node() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", end+1);
		assert_ok!(
			Network::<T>::add_subnet_node(
				RawOrigin::Signed(subnet_node_account.clone()).into(), 
				subnet_id, 
				subnet_node_account.clone(),
				peer(end+1), 
				peer(end+1), 
				0,
				DEFAULT_SUBNET_NODE_STAKE,
				None,
				None,
				None,
      )
		);

		let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

		#[extrinsic_call]
		remove_subnet_node(RawOrigin::Signed(subnet_node_account.clone()), subnet_id, hotkey_subnet_node_id);
		
		assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), end);

		let subnet_node_id = HotkeySubnetNodeId::<T>::try_get(subnet_id, subnet_node_account.clone());
		assert_eq!(subnet_node_id, Err(()));

		let subnet_node_account = PeerIdSubnetNode::<T>::try_get(subnet_id, peer(end+1));
		assert_eq!(subnet_node_account, Err(()));

	}

	#[benchmark]
	fn add_to_stake() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", end+1);
		assert_ok!(
			Network::<T>::add_subnet_node(
				RawOrigin::Signed(subnet_node_account.clone()).into(), 
				subnet_id, 
				subnet_node_account.clone(),
				peer(end+1), 
				peer(end+1), 
				0,
				DEFAULT_SUBNET_NODE_STAKE,
				None,
				None,
				None,
      )
		);

		T::Currency::deposit_creating(&subnet_node_account, DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));

		let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

		#[extrinsic_call]
		add_to_stake(
			RawOrigin::Signed(subnet_node_account.clone()), 
			subnet_id, 
			hotkey_subnet_node_id,
			subnet_node_account.clone(),
			DEFAULT_STAKE_TO_BE_ADDED
		);
		
		let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node_account.clone(), subnet_id);
		assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE + DEFAULT_STAKE_TO_BE_ADDED);
	}

	#[benchmark]
	fn remove_stake() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", end+1);
		assert_ok!(
			Network::<T>::add_subnet_node(
				RawOrigin::Signed(subnet_node_account.clone()).into(), 
				subnet_id, 
				subnet_node_account.clone(),
				peer(end+1), 
				peer(end+1), 
				0,
				DEFAULT_SUBNET_NODE_STAKE,
				None,
				None,
				None,
      )
		);

		T::Currency::deposit_creating(&subnet_node_account, DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));
		let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

		assert_ok!(
			Network::<T>::add_to_stake(
				RawOrigin::Signed(subnet_node_account.clone()).into(), 
				subnet_id, 
				hotkey_subnet_node_id,
				subnet_node_account.clone(),
				DEFAULT_STAKE_TO_BE_ADDED
			)
		);
		let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node_account.clone(), subnet_id);
		assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE + DEFAULT_STAKE_TO_BE_ADDED);

		#[extrinsic_call]
		remove_stake(RawOrigin::Signed(subnet_node_account.clone()), subnet_id, subnet_node_account.clone(), DEFAULT_STAKE_TO_BE_ADDED);
		
		let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node_account.clone(), subnet_id);
		assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE);
	}

	#[benchmark]
	fn add_to_delegate_stake() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

    let _ = T::Currency::deposit_creating(&delegate_account.clone(), (DEFAULT_STAKE_TO_BE_ADDED + 500).try_into().ok().expect("REASON"));
    let starting_delegator_balance = T::Currency::free_balance(&delegate_account.clone());

		#[extrinsic_call]
		add_to_delegate_stake(RawOrigin::Signed(delegate_account.clone()), subnet_id, DEFAULT_DELEGATE_STAKE_TO_BE_ADDED);

    let post_delegator_balance = T::Currency::free_balance(&delegate_account.clone());
    assert_eq!(post_delegator_balance, starting_delegator_balance - DEFAULT_DELEGATE_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
    let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
    let delegate_balance = Network::<T>::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    // Ensure balance is within <= 0.01% of deposited balance, and less than deposited balance
    assert!(
      (delegate_balance >= Network::<T>::percent_mul(DEFAULT_DELEGATE_STAKE_TO_BE_ADDED, 990000000)) &&
      (delegate_balance < DEFAULT_DELEGATE_STAKE_TO_BE_ADDED)
    );
	}

	#[benchmark]
	fn swap_delegate_stake() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let from_subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH_2.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let to_subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH_2.into()).unwrap();

		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

		assert_ok!(
			Network::<T>::add_to_delegate_stake(
				RawOrigin::Signed(delegate_account.clone()).into(), 
				from_subnet_id, 
				DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
			)
		);

		let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);
		let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id);

		let from_delegate_balance = Network::<T>::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

		#[extrinsic_call]
		swap_delegate_stake(
			RawOrigin::Signed(delegate_account.clone()), 
			from_subnet_id, 
			to_subnet_id, 
			delegate_shares
		);

    let from_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);
    assert_eq!(from_delegate_shares, 0);

    let to_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), to_subnet_id);
    assert_ne!(to_delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(to_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(to_subnet_id);

    let to_delegate_balance = Network::<T>::convert_to_balance(
      to_delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // Will lose about .01% of the transfer value on first transfer into a pool
    // The balance should be about ~99% of the ``from`` subnet to the ``to`` subnet
    assert!(
      (to_delegate_balance >= Network::<T>::percent_mul(from_delegate_balance, 990000000)) &&
      (to_delegate_balance <= from_delegate_balance)
    );
	}

	#[benchmark]
	fn remove_delegate_stake() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
		assert_ok!(
			Network::<T>::add_to_delegate_stake(
				RawOrigin::Signed(delegate_account.clone()).into(), 
				subnet_id, 
				DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
			)
		);
		let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);

		let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

		let delegate_balance = Network::<T>::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

		let epoch_length = T::EpochLength::get();
		let current_epoch = get_current_block_as_u32::<T>() / epoch_length;

		#[extrinsic_call]
		remove_delegate_stake(
			RawOrigin::Signed(delegate_account.clone()), 
			subnet_id, 
			delegate_shares
		);

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<T>::get(delegate_account.clone());
    assert_eq!(unbondings.len(), 1);

		let (epoch, balance) = unbondings.iter().next().unwrap();
    assert_eq!(*epoch, current_epoch + T::DelegateStakeCooldownEpochs::get());
    assert_eq!(*balance, delegate_balance);
	}

	#[benchmark]
	fn claim_unbondings() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
		assert_ok!(
			Network::<T>::add_to_delegate_stake(
				RawOrigin::Signed(delegate_account.clone()).into(), 
				subnet_id, 
				DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
			)
		);
		let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);

		let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

		let delegate_balance = Network::<T>::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

		let epoch_length = T::EpochLength::get();
		let current_epoch = get_current_block_as_u32::<T>() / epoch_length;

		assert_ok!(
			Network::<T>::remove_delegate_stake(
				RawOrigin::Signed(delegate_account.clone()).into(), 
				subnet_id, 
				delegate_shares
			)
		);

		let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<T>::get(delegate_account.clone());
    assert_eq!(unbondings.len(), 1);

		let (epoch, balance) = unbondings.iter().next().unwrap();
    assert_eq!(*epoch, current_epoch + T::DelegateStakeCooldownEpochs::get());
    assert_eq!(*balance, delegate_balance);

		let pre_delegator_balance: u128 = T::Currency::free_balance(&delegate_account.clone()).try_into().ok().expect("REASON");		

		let current_block_number = get_current_block_as_u32::<T>();
		frame_system::Pallet::<T>::set_block_number(u32_to_block::<T>(current_block_number + ((epoch_length  + 1) * T::DelegateStakeCooldownEpochs::get())));

		#[extrinsic_call]
		claim_unbondings(RawOrigin::Signed(delegate_account.clone()));

		let post_delegator_balance: u128 = T::Currency::free_balance(&delegate_account.clone()).try_into().ok().expect("REASON");

		assert_eq!(post_delegator_balance, pre_delegator_balance + balance);
	}

	#[benchmark]
	fn increase_delegate_stake() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
		assert_ok!(
			Network::<T>::add_to_delegate_stake(
				RawOrigin::Signed(delegate_account.clone()).into(), 
				subnet_id, 
				DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
			)
		);

		let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
		let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

		let delegate_balance = Network::<T>::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

		let funder = funded_account::<T>("funder", 0);

		#[extrinsic_call]
		increase_delegate_stake(RawOrigin::Signed(funder), subnet_id, DEFAULT_SUBNET_NODE_STAKE);
		
		let increased_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
		let increased_total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let increased_total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

		let increased_delegate_balance = Network::<T>::convert_to_balance(
      increased_delegate_shares,
      increased_total_subnet_delegated_stake_shares,
      increased_total_subnet_delegated_stake_balance
    );
		assert_eq!(increased_total_subnet_delegated_stake_balance, total_subnet_delegated_stake_balance + DEFAULT_SUBNET_NODE_STAKE);

		assert_ne!(increased_delegate_balance, delegate_balance);
		assert!(increased_delegate_balance > delegate_balance);
	}

	#[benchmark]
	fn add_to_node_delegate_stake() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();
		let subnet_node_id = end;

		let delegate_node_account: T::AccountId = funded_account::<T>("delegate_node_account", 0);

		#[extrinsic_call]
		add_to_node_delegate_stake(RawOrigin::Signed(delegate_node_account.clone()), subnet_id, subnet_node_id, DEFAULT_SUBNET_NODE_STAKE);
		
    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), subnet_id, subnet_node_id));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);

    let account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    assert!(
      (account_node_delegate_stake_balance >= Network::<T>::percent_mul(DEFAULT_SUBNET_NODE_STAKE, 990000000)) &&
      (account_node_delegate_stake_balance <= DEFAULT_SUBNET_NODE_STAKE)
    );
	}

	#[benchmark]
	fn swap_node_delegate_stake() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let from_subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();
		let from_subnet_node_id = end;

		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH_2.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let to_subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH_2.into()).unwrap();
		let to_subnet_node_id = end;

		let delegate_node_account: T::AccountId = funded_account::<T>("delegate_node_account", 0);

		assert_ok!(
			Network::<T>::add_to_node_delegate_stake(
				RawOrigin::Signed(delegate_node_account.clone()).into(), 
				from_subnet_id, 
				from_subnet_node_id, 
				DEFAULT_SUBNET_NODE_STAKE
			)
		);

		let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(from_subnet_id, from_subnet_node_id);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(from_subnet_id, from_subnet_node_id);

		let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), from_subnet_id, from_subnet_node_id));
    let account_node_delegate_stake_shares_to_be_removed = account_node_delegate_stake_shares / 2;

		let expected_balance_to_be_removed = Network::<T>::convert_to_balance(
      account_node_delegate_stake_shares_to_be_removed,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

		let expected_post_balance = Network::<T>::convert_to_balance(
      account_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
      total_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
      total_node_delegate_stake_balance - expected_balance_to_be_removed
    );

		#[extrinsic_call]
		swap_node_delegate_stake(
			RawOrigin::Signed(delegate_node_account.clone()), 
			from_subnet_id,
			from_subnet_node_id,
			to_subnet_id,
			to_subnet_node_id,
			account_node_delegate_stake_shares_to_be_removed,
		);
		
    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), from_subnet_id, from_subnet_node_id));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(from_subnet_id, from_subnet_node_id);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(from_subnet_id, from_subnet_node_id);

    let account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

		assert_eq!(account_node_delegate_stake_balance, expected_post_balance);




		let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), to_subnet_id, to_subnet_node_id));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(to_subnet_id, to_subnet_node_id);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(to_subnet_id, to_subnet_node_id);

    let account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    assert_ne!(account_node_delegate_stake_balance, 0);

    assert!(
      (account_node_delegate_stake_balance >= Network::<T>::percent_mul(expected_balance_to_be_removed, 990000000)) &&
      (account_node_delegate_stake_balance <= expected_balance_to_be_removed)
    );

    // Ensure the code didn't create an unbonding insert
    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<T>::get(delegate_node_account.clone());
    assert_eq!(unbondings.len(), 0);
	}

	#[benchmark]
	fn remove_node_delegate_stake() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();
		let subnet_node_id = end;

		let delegate_node_account: T::AccountId = funded_account::<T>("delegate_node_account", 0);
		assert_ok!(
			Network::<T>::add_to_node_delegate_stake(
				RawOrigin::Signed(delegate_node_account.clone()).into(), 
				subnet_id, 
				subnet_node_id, 
				DEFAULT_SUBNET_NODE_STAKE
			)
		);

    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);

		let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), subnet_id, subnet_node_id));
		let account_node_delegate_stake_shares_to_be_removed = account_node_delegate_stake_shares / 2;

		let expected_balance_to_be_removed = Network::<T>::convert_to_balance(
      account_node_delegate_stake_shares_to_be_removed,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    let expected_post_balance = Network::<T>::convert_to_balance(
      account_node_delegate_stake_shares_to_be_removed,
      total_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
      total_node_delegate_stake_balance - expected_balance_to_be_removed
    );

		let epoch_length = T::EpochLength::get();
		let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

		#[extrinsic_call]
		remove_node_delegate_stake(RawOrigin::Signed(delegate_node_account.clone()), subnet_id, subnet_node_id, account_node_delegate_stake_shares_to_be_removed);
		
    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), subnet_id, subnet_node_id));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);

    assert_eq!(account_node_delegate_stake_shares, account_node_delegate_stake_shares_to_be_removed);

    let post_account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    assert_eq!(expected_post_balance, post_account_node_delegate_stake_balance);

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<T>::get(delegate_node_account.clone());
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + T::NodeDelegateStakeCooldownEpochs::get());
    assert_eq!(*ledger_balance, expected_balance_to_be_removed);
	}

	#[benchmark]
	fn increase_node_delegate_stake() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();
		let subnet_node_id = end;

		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

		let pre_total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);

		#[extrinsic_call]
		increase_node_delegate_stake(RawOrigin::Signed(delegate_account), subnet_id, subnet_node_id, DEFAULT_SUBNET_NODE_STAKE);
		
		let post_total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);

		assert_eq!(pre_total_node_delegate_stake_balance + DEFAULT_SUBNET_NODE_STAKE, post_total_node_delegate_stake_balance);
	}

	#[benchmark]
	fn transfer_from_node_to_subnet() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let from_subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();
		let from_subnet_node_id = end;

		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH_2.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let to_subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH_2.into()).unwrap();

		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

    assert_ok!(
      Network::<T>::add_to_node_delegate_stake(
        RawOrigin::Signed(delegate_account.clone()).into(), 
				from_subnet_id, 
				from_subnet_node_id, 
				DEFAULT_SUBNET_NODE_STAKE
      )
    );

		let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_account.clone(), from_subnet_id, from_subnet_node_id));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(from_subnet_id, from_subnet_node_id);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(from_subnet_id, from_subnet_node_id);

    let account_node_delegate_stake_shares_to_be_removed = account_node_delegate_stake_shares / 2;

    let expected_balance_to_be_removed = Network::<T>::convert_to_balance(
      account_node_delegate_stake_shares_to_be_removed,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

		#[extrinsic_call]
		transfer_from_node_to_subnet(
			RawOrigin::Signed(delegate_account.clone()), 
			from_subnet_id, 
			from_subnet_node_id, 
			to_subnet_id,
			account_node_delegate_stake_shares_to_be_removed
		);
		
    let to_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), to_subnet_id);
    assert_ne!(to_delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(to_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(to_subnet_id);

    let mut to_delegate_balance = Network::<T>::convert_to_balance(
      to_delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

		// The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // Will lose about .01% of the transfer value on first transfer into a pool
    // The balance should be about ~99% of the ``from`` subnet to the ``to`` subnet
    assert!(
      (to_delegate_balance >= Network::<T>::percent_mul(expected_balance_to_be_removed, 990000000)) &&
      (to_delegate_balance < expected_balance_to_be_removed)
    );
	}

	#[benchmark]
	fn transfer_from_subnet_to_node() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let from_subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();

		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH_2.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let to_subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH_2.into()).unwrap();
		let to_subnet_node_id = end;

		let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

    assert_ok!(
      Network::<T>::add_to_delegate_stake(
        RawOrigin::Signed(delegate_account.clone()).into(), 
				from_subnet_id, 
				DEFAULT_SUBNET_NODE_STAKE
      )
    );


		let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);

		let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id);

    let mut from_delegate_balance = Network::<T>::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

		#[extrinsic_call]
		transfer_from_subnet_to_node(
			RawOrigin::Signed(delegate_account.clone()), 
			from_subnet_id, 
			to_subnet_id,
			to_subnet_node_id, 
			delegate_shares
		);
		
    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_account.clone(), to_subnet_id, to_subnet_node_id));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(to_subnet_id, to_subnet_node_id);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(to_subnet_id, to_subnet_node_id);

    let account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    assert_ne!(account_node_delegate_stake_balance, 0);

    assert!(
      (account_node_delegate_stake_balance >= Network::<T>::percent_mul(from_delegate_balance, 990000000)) &&
      (account_node_delegate_stake_balance < from_delegate_balance)
    );
	}

	#[benchmark]
	fn validate() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();
		let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

		let n_nodes: u32 = TotalSubnetNodes::<T>::get(subnet_id);

		let epoch_length = T::EpochLength::get();

		let current_block_number = get_current_block_as_u32::<T>();
		let next_epoch_block = current_block_number - (current_block_number % epoch_length) + epoch_length;
		frame_system::Pallet::<T>::set_block_number(u32_to_block::<T>(next_epoch_block));

		let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

    Network::<T>::do_epoch_preliminaries(get_current_block_as_u32::<T>(), epoch as u32, epoch_length);

		let validator_id = SubnetRewardsValidator::<T>::get(subnet_id, epoch as u32);
    assert!(validator_id != None, "Validator is None");

		let hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id.unwrap()).unwrap();

		let subnet_node_data_vec = subnet_node_data(0, n_nodes);

		#[extrinsic_call]
		validate(RawOrigin::Signed(hotkey.clone()), subnet_id, subnet_node_data_vec.clone(), None);

		let submission = SubnetRewardsSubmission::<T>::get(subnet_id, epoch as u32).unwrap();

    assert_eq!(submission.validator_id, validator_id.unwrap(), "Err: validator");
    assert_eq!(submission.data.len(), subnet_node_data_vec.clone().len(), "Err: data len");
    assert_eq!(submission.attests.len(), 1, "Err: attests"); // validator auto-attests
	}

	#[benchmark]
	fn attest() {
		let end = 12;
		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH.into()).unwrap();
		let subnet = SubnetsData::<T>::get(subnet_id).unwrap();
		let n_nodes: u32 = TotalSubnetNodes::<T>::get(subnet_id);

		let epoch_length = T::EpochLength::get();

		let current_block_number = get_current_block_as_u32::<T>();
		let next_epoch_block = current_block_number - (current_block_number % epoch_length) + epoch_length;
		frame_system::Pallet::<T>::set_block_number(u32_to_block::<T>(next_epoch_block));

		let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

    Network::<T>::do_epoch_preliminaries(get_current_block_as_u32::<T>(), epoch as u32, epoch_length);

		let validator_id = SubnetRewardsValidator::<T>::get(subnet_id, epoch as u32);
    assert!(validator_id != None, "Validator is None");

		let hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id.unwrap()).unwrap();

		let subnet_node_data_vec = subnet_node_data(0, n_nodes);

		assert_ok!(
			Network::<T>::validate(
				RawOrigin::Signed(hotkey.clone()).into(), 
				subnet_id, 
				subnet_node_data_vec.clone(),
				None,
			)
		);
	
		// Might be the same ID as validator_id
		let attester = funded_account::<T>("subnet_node_account", 2);
    let attester_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, attester.clone()).unwrap();

		let current_block_number = get_current_block_as_u32::<T>();

		#[extrinsic_call]
		attest(RawOrigin::Signed(attester.clone()), subnet_id);

		let submission = SubnetRewardsSubmission::<T>::get(subnet_id, epoch as u32).unwrap();

		// validator + attester
    assert_eq!(submission.attests.len(), 2 as usize);
    assert_eq!(submission.attests.get(&attester_subnet_node_id), Some(&current_block_number));
	}

	#[benchmark]
	fn rewards_v2(x: Linear<1, 64>, p: Linear<1, 512>) {
		/// x: subnets
		/// p: nodes

		let max_subnets: u32 = Network::<T>::max_subnets();
		let n_nodes: u32 = Network::<T>::max_subnet_nodes();

		// Activate subnets
		for s in 0..x {
			let path: Vec<u8> = format!("subnet-name-{s}").into(); 
			build_activated_subnet::<T>(path, 0, n_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		}

		let epoch_length = T::EpochLength::get();
		let block_number = get_current_block_as_u32::<T>();
		let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

		// Insert validator and validate
		for s in 0..x {
			let path: Vec<u8> = format!("subnet-name-{s}").into(); 
			let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

			let validator_id: u32 = 1;
			let validator_hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id).unwrap();

			SubnetRewardsValidator::<T>::insert(subnet_id, epoch, validator_id);
			let subnet_node_data_vec = subnet_node_data(0, n_nodes);

			assert_ok!(
				Network::<T>::validate(
					RawOrigin::Signed(validator_hotkey.clone()).into(), 
					subnet_id, 
					subnet_node_data_vec.clone(),
					None,
				)
			);		
		}

		let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

		// Attest
		for s in 0..x {
			let path: Vec<u8> = format!("subnet-name-{s}").into(); 
			let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

			for n in 1..p+1 {
				if n == 1 {
					continue
				}	
				let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
        assert_ok!(
          Network::<T>::attest(
            RawOrigin::Signed(subnet_node_account.clone()).into(), 
            subnet_id,
          )
        );
			}
		}

		let pre_total_issuance: u128 = Network::<T>::get_total_network_issuance();


		#[block]
		{
			Network::<T>::reward_subnets_v2(0, 0);
		}

		let post_total_issuance: u128 = Network::<T>::get_total_network_issuance();

		// assert!(post_total_issuance > pre_total_issuance);
		assert!(true);
	}

	// #[benchmark]
	// fn do_single_subnet_deactivation_ledger() {
	// 	let n_nodes: u32 = Network::<T>::max_subnet_nodes();

	// 	let path: Vec<u8> = DEFAULT_SUBNET_PATH.into();
	// 	build_activated_subnet::<T>(path.clone(), 0, n_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path.clone()).unwrap();

	// 	let epoch_length = T::EpochLength::get();
	// 	let block_number = get_current_block_as_u32::<T>();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Insert validator and validate
	// 	let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", 0);
	// 	SubnetRewardsValidator::<T>::insert(subnet_id, epoch as u32, subnet_node_account.clone());
	// 	let subnet_node_data_vec = subnet_node_data(0, n_nodes);
	// 	assert_ok!(
	// 		Network::<T>::validate(
	// 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 			subnet_id, 
	// 			subnet_node_data_vec.clone(),
	// 			None,
	// 		)
	// 	);		

	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Attest so node can be in the deactivate ledger
	// 	for n in 0..n_nodes {
	// 		if n == 0 {
	// 			continue
	// 		}
	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 		assert_ok!(
	// 			Network::<T>::attest(
	// 				RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 				subnet_id,
	// 			)
	// 		);
	// 	}

	// 	for n in 0..n_nodes {
	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 		assert_ok!(
	// 			Network::<T>::deactivate_subnet_node(
	// 				RawOrigin::Signed(subnet_node_account.clone()).into(),
	// 				subnet_id,
	// 			)
	// 		);
	// 	}

	// 	#[block]
	// 	{
	// 		Network::<T>::do_deactivation_ledger();
	// 	}

	// 	for n in 0..n_nodes {
	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 		let subnet_node = SubnetNodesData::<T>::get(subnet_id, subnet_node_account.clone());
	// 		assert_eq!(subnet_node.classification.class, SubnetNodeClass::Deactivated);		
	// 	}
	// }

	// #[benchmark]
	// fn do_deactivation_ledger(x: Linear<0, 64>, d: Linear<0, 128>) {
	// 	let max_subnets: u32 = Network::<T>::max_subnets();
	// 	let n_nodes: u32 = Network::<T>::max_subnet_nodes();

	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		build_activated_subnet::<T>(path, 0, n_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	}

	// 	let epoch_length = T::EpochLength::get();
	// 	let block_number = get_current_block_as_u32::<T>();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Insert validator and validate
	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", 0);
	// 		SubnetRewardsValidator::<T>::insert(subnet_id, epoch as u32, subnet_node_account.clone());
	// 		let subnet_node_data_vec = subnet_node_data(0, n_nodes);
	// 		assert_ok!(
	// 			Network::<T>::validate(
	// 				RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 				subnet_id, 
	// 				subnet_node_data_vec.clone(),
	// 				None,
	// 			)
	// 		);		
	// 	}

	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Attest so node can be in the deactivate ledger
	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..n_nodes {
	// 			if n == 0 {
	// 				continue
	// 			}	
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
  //       assert_ok!(
  //         Network::<T>::attest(
  //           RawOrigin::Signed(subnet_node_account.clone()).into(), 
  //           subnet_id,
  //         )
  //       );
	// 		}
	// 	}

	// 	// let path: Vec<u8> = "subnet-name-{0}".into(); 
	// 	// let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();
	// 	let mut i = 0;

	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..d {
	// 			if i == 128 {
	// 				break
	// 			}
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 			assert_ok!(
	// 				Network::<T>::deactivate_subnet_node(
	// 					RawOrigin::Signed(subnet_node_account.clone()).into(),
	// 					subnet_id,
	// 				)
	// 			);
	// 			i += 1;
	// 		}
	// 	}

	// 	#[block]
	// 	{
	// 		Network::<T>::do_deactivation_ledger();
	// 	}

	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..n_nodes {
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 			let subnet_node = SubnetNodesData::<T>::get(subnet_id, subnet_node_account.clone());
	// 			// assert_eq!(subnet_node.classification.class, SubnetNodeClass::Deactivated);		
	// 		}
	// 	}
	// }

	// #[benchmark]
	// fn do_deactivation_ledger(x: Linear<0, 64>, p: Linear<0, 512>, d: Linear<0, 128>) {
	// 	let max_subnets: u32 = Network::<T>::max_subnets();
	// 	let n_nodes: u32 = Network::<T>::max_subnet_nodes();

	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		build_activated_subnet::<T>(path, 0, n_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	}

	// 	let epoch_length = T::EpochLength::get();
	// 	let block_number = get_current_block_as_u32::<T>();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Insert validator and validate
	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", 0);
	// 		SubnetRewardsValidator::<T>::insert(subnet_id, epoch as u32, subnet_node_account.clone());
	// 		let subnet_node_data_vec = subnet_node_data(0, n_nodes);
	// 		assert_ok!(
	// 			Network::<T>::validate(
	// 				RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 				subnet_id, 
	// 				subnet_node_data_vec.clone(),
	// 				None,
	// 			)
	// 		);		
	// 	}

	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Attest so node can be in the deactivate ledger
	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..p {
	// 			if n == 0 {
	// 				continue
	// 			}	
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
  //       assert_ok!(
  //         Network::<T>::attest(
  //           RawOrigin::Signed(subnet_node_account.clone()).into(), 
  //           subnet_id,
  //         )
  //       );
	// 		}
	// 	}

	// 	// let path: Vec<u8> = "subnet-name-{0}".into(); 
	// 	// let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();
	// 	let mut i = 0;

	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..d {
	// 			if i == 128 {
	// 				break
	// 			}
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 			assert_ok!(
	// 				Network::<T>::deactivate_subnet_node(
	// 					RawOrigin::Signed(subnet_node_account.clone()).into(),
	// 					subnet_id,
	// 				)
	// 			);
	// 			i += 1;
	// 		}
	// 	}

	// 	#[block]
	// 	{
	// 		Network::<T>::do_deactivation_ledger();
	// 	}

	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..p {
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 			let subnet_node = SubnetNodesData::<T>::get(subnet_id, subnet_node_account.clone());
	// 			// assert_eq!(subnet_node.classification.class, SubnetNodeClass::Deactivated);		
	// 		}
	// 	}
	// }

	// #[benchmark]
	// fn do_deactivation_ledger() {
	// 	let max_subnets: u32 = Network::<T>::max_subnets();
	// 	let n_nodes: u32 = Network::<T>::max_subnet_nodes();

	// 	for s in 0..max_subnets {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		build_activated_subnet::<T>(path, 0, n_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	}

	// 	let epoch_length = T::EpochLength::get();
	// 	let block_number = get_current_block_as_u32::<T>();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Insert validator and validate
	// 	for s in 0..max_subnets {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", 0);
	// 		SubnetRewardsValidator::<T>::insert(subnet_id, epoch as u32, subnet_node_account.clone());
	// 		let subnet_node_data_vec = subnet_node_data(0, n_nodes);
	// 		assert_ok!(
	// 			Network::<T>::validate(
	// 				RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 				subnet_id, 
	// 				subnet_node_data_vec.clone(),
	// 				None,
	// 			)
	// 		);		
	// 	}

	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Attest so node can be in the deactivate ledger
	// 	for s in 0..max_subnets {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..n_nodes {
	// 			if n == 0 {
	// 				continue
	// 			}	
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
  //       assert_ok!(
  //         Network::<T>::attest(
  //           RawOrigin::Signed(subnet_node_account.clone()).into(), 
  //           subnet_id,
  //         )
  //       );
	// 		}
	// 	}

	// 	for s in 0..max_subnets {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..n_nodes {
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 			assert_ok!(
	// 				Network::<T>::deactivate_subnet_node(
	// 					RawOrigin::Signed(subnet_node_account.clone()).into(),
	// 					subnet_id,
	// 				)
	// 			);
	// 		}
	// 	}

	// 	#[block]
	// 	{
	// 		Network::<T>::do_deactivation_ledger();
	// 	}

	// 	for s in 0..max_subnets {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..n_nodes {
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 			let subnet_node = SubnetNodesData::<T>::get(subnet_id, subnet_node_account.clone());
	// 			assert_eq!(subnet_node.classification.class, SubnetNodeClass::Deactivated);		
	// 		}
	// 	}
	// }

	// #[benchmark]
	// fn do_single_subnet_deactivation_ledger() {
	// 	let n_nodes: u32 = Network::<T>::max_subnet_nodes();

	// 	let path: Vec<u8> = DEFAULT_SUBNET_PATH.into();
	// 	build_activated_subnet::<T>(path.clone(), 0, n_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetPaths::<T>::get::<Vec<u8>>(path.clone()).unwrap();

	// 	let epoch_length = T::EpochLength::get();
	// 	let block_number = get_current_block_as_u32::<T>();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Insert validator and validate
	// 	let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", 0);
	// 	SubnetRewardsValidator::<T>::insert(subnet_id, epoch as u32, subnet_node_account.clone());
	// 	let subnet_node_data_vec = subnet_node_data(0, n_nodes);
	// 	assert_ok!(
	// 		Network::<T>::validate(
	// 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 			subnet_id, 
	// 			subnet_node_data_vec.clone(),
	// 			None,
	// 		)
	// 	);		

	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Attest so node can be in the deactivate ledger
	// 	for n in 0..n_nodes {
	// 		if n == 0 {
	// 			continue
	// 		}
	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 		assert_ok!(
	// 			Network::<T>::attest(
	// 				RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 				subnet_id,
	// 			)
	// 		);
	// 	}

	// 	for n in 0..n_nodes {
	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 		assert_ok!(
	// 			Network::<T>::deactivate_subnet_node(
	// 				RawOrigin::Signed(subnet_node_account.clone()).into(),
	// 				subnet_id,
	// 			)
	// 		);
	// 	}

	// 	#[block]
	// 	{
	// 		Network::<T>::do_deactivation_ledger();
	// 	}

	// 	for n in 0..n_nodes {
	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 		let subnet_node = SubnetNodesData::<T>::get(subnet_id, subnet_node_account.clone());
	// 		assert_eq!(subnet_node.classification.class, SubnetNodeClass::Deactivated);		
	// 	}
	// }

	// #[benchmark]
	// fn on_initialize_do_choose_validator_and_accountants() {
	// 	let max_subnets: u32 = Network::<T>::max_subnets();
	// 	let n_nodes: u32 = Network::<T>::max_subnet_nodes();

	// 	for s in 0..max_subnets {
	// 		build_activated_subnet::<T>(DEFAULT_SUBNET_PATH.into(), 0, n_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	}

	// 	let epoch_length = T::EpochLength::get();

	// 	let block_number = get_current_block_as_u32::<T>();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	#[block]
	// 	{
	// 		let block = get_current_block_as_u32::<T>();
	// 		let epoch_length = T::EpochLength::get();
	// 		let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;
	// 		Network::<T>::do_epoch_preliminaries(
	// 			block, 
	// 			epoch as u32, 
	// 			epoch_length
	// 		);
	// 	}

	// 	// ensure nodes were rewarded
	// 	for s in 0..max_subnets {
	// 		let subnet_id = s+1;

	// 		let validator = SubnetRewardsValidator::<T>::get(subnet_id, epoch as u32);
	// 		assert!(validator != None, "Validator is None");
	// 	}
	// }

	// #[benchmark]
	// fn on_initialize() {
	// 	// get to a block where none of the functions will be ran
	// 	frame_system::Pallet::<T>::set_block_number(
	// 		frame_system::Pallet::<T>::block_number() + u32_to_block::<T>(1)
	// 	);

	// 	#[block]
	// 	{
	// 		let block = frame_system::Pallet::<T>::block_number();
	// 		Network::<T>::on_initialize(block);
	// 	}
	// }

	// impl_benchmark_test_suite!(Network, crate::mock::new_test_ext(), crate::mock::Test);
	impl_benchmark_test_suite!(Network, tests::mock::new_test_ext(), tests::mock::Test);
}
