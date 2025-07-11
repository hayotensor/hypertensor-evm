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
use sp_core::blake2_128;
use frame_support::pallet_prelude::DispatchError;
use sp_runtime::traits::Header;
const SEED: u32 = 0;


const DEFAULT_SCORE: u128 = 100e+18 as u128;
const DEFAULT_SUBNET_INIT_COST: u128 = 100e+18 as u128;
const DEFAULT_SUBNET_NAME: &str = "subnet-name";
const DEFAULT_SUBNET_PATH_2: &str = "subnet-name-2";
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
	let deposit_amount: u128 = NetworkMinStakeBalance::<T>::get() + 10000;
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

// fn increase_epochs<T: Config>(epochs: u32) {
//   if epochs == 0 {
//     return
//   }

//   let block = get_current_block_as_u32::<T>();

//   let epoch_length = T::EpochLength::get();

//   let next_epoch_start_block = (epoch_length * epochs) + block - (block % (epoch_length * epochs));

// 	frame_system::Pallet::<T>::set_block_number(next_epoch_start_block.into());
// }

pub fn increase_epochs<T: Config>(epochs: u32) {
  if epochs == 0 {
		return;
  }

  let block = get_current_block_as_u32::<T>();
  let epoch_length = T::EpochLength::get();

  let advance_blocks = epoch_length.saturating_mul(epochs);
  let new_block = block.saturating_add(advance_blocks);

	frame_system::Pallet::<T>::set_block_number(new_block.into());
}

fn build_activated_subnet<T: Config>(
	name: Vec<u8>, 
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

	let min_nodes = MinSubnetNodes::<T>::get();
	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	let subnets = TotalActiveSubnets::<T>::get() + 1;

	let register_subnet_data: RegistrationSubnetData<T::AccountId> = default_registration_subnet_data::<T>(
    subnets,
    max_subnet_nodes,
    name.clone().into(),
    start, 
    end
  );

		// --- Register subnet for activation
  assert_ok!(
    Network::<T>::register_subnet(
      RawOrigin::Signed(funded_initializer.clone()).into(),
      register_subnet_data,
    )
  );

  let subnet_id = SubnetName::<T>::get(name.clone()).unwrap();
  let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

  if end == 0 {
    end = min_nodes;
  }

  let epoch = get_current_block_as_u32::<T>() / epoch_length;

  // --- Add subnet nodes
  let block_number = get_current_block_as_u32::<T>();
  let mut amount_staked = 0;
  for n in start..end {
		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", subnets*max_subnet_nodes+n);
		T::Currency::deposit_creating(&subnet_node_account, DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));
    amount_staked += amount;
    assert_ok!(
      Network::<T>::add_subnet_node(
        RawOrigin::Signed(subnet_node_account.clone()).into(),
        subnet_id,
				subnet_node_account.clone(),
        peer(subnets*max_subnet_nodes+n),
				peer(subnets*max_subnet_nodes+n),
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

    assert_eq!(subnet_node_data.peer_id, peer(subnets*max_subnet_nodes+n));

    // --- Is ``Validator`` if registered before subnet activation
    assert_eq!(subnet_node_data.classification.node_class, SubnetNodeClass::Validator);
    assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, epoch));

    let peer_subnet_node_account = PeerIdSubnetNode::<T>::get(subnet_id, peer(subnets*max_subnet_nodes+n));
    assert_eq!(peer_subnet_node_account, hotkey_subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);
    assert_eq!(account_subnet_stake, amount);
  }

	let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
	assert_eq!(active_nodes, end - start);

	let slot_list = SubnetNodeElectionSlots::<T>::get(subnet_id);
	assert_eq!(slot_list.len(), active_nodes as usize);

  let total_subnet_stake = TotalSubnetStake::<T>::get(subnet_id);
  assert_eq!(total_subnet_stake, amount_staked);

  let total_stake = TotalStake::<T>::get();
  assert_eq!(total_subnet_stake, amount_staked);


	let min_subnet_delegate_stake = Network::<T>::get_min_subnet_delegate_stake_balance() + (1000e+18 as u128 * subnets as u128);
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

  // let current_block_number = get_current_block_as_u32::<T>();
	let epochs = SubnetRegistrationEpochs::<T>::get();
  increase_epochs::<T>(epochs + 1);

  assert_ok!(
    Network::<T>::activate_subnet(
      RawOrigin::Signed(funded_initializer.clone()).into(),
      subnet_id,
    )
  );

	increase_epochs::<T>(2);
}

// Returns total staked on subnet
// fn build_subnet_nodes<T: Config>(subnet_id: u32, start: u32, end: u32, amount: u128) -> u128 {
//   let mut amount_staked = 0;
//   for n in start..end {
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

pub fn default_registration_subnet_data<T: Config>(
  subnets: u32,
  max_subnet_nodes: u32,
  name: Vec<u8>,
  start: u32, 
  end: u32
) -> RegistrationSubnetData<T::AccountId> {
  let seed_bytes: &[u8] = &name;
  let add_subnet_data = RegistrationSubnetData {
    name: name.clone(),
    repo: blake2_128(seed_bytes).to_vec(), // must be unique
    description: Vec::new(),
    misc: Vec::new(),
    churn_limit: 4,
    min_stake: 100e+18 as u128,
    max_stake: 10000e+18 as u128,
    delegate_stake_percentage: 100000000000000000, // 10%
    registration_queue_epochs: 4,
    activation_grace_epochs: 4,
    queue_classification_epochs: 4,
    included_classification_epochs: 4,
    max_node_penalties: 3,
    initial_coldkeys: get_initial_coldkeys::<T>(subnets, max_subnet_nodes, start, end),
    max_registered_nodes: 100,
  };
  add_subnet_data
}

fn get_subnet_node_data(start: u32, end: u32) -> Vec<SubnetNodeConsensusData> {
  // initialize peer consensus data array
  let mut subnet_node_data: Vec<SubnetNodeConsensusData> = Vec::new();
  for n in start..end {
    let peer_subnet_node_data: SubnetNodeConsensusData = SubnetNodeConsensusData {
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

pub fn get_initial_coldkeys<T: Config>(subnets: u32, max_subnet_nodes: u32, start: u32, end: u32) -> BTreeSet<T::AccountId> {
  let mut whitelist = BTreeSet::new();
  for n in start..end {
    whitelist.insert(funded_account::<T>("subnet_node_account", subnets*max_subnet_nodes+n));
  }
  whitelist
}

pub fn get_subnet_node_consensus_data<T: Config>(
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
		let node_id = subnet_id*max_subnet_nodes+n;
		// let peer_id = peer(node_id);

		// Simulate some score and block number
		let score = 1000e+18 as u128;

		attests.insert(node_id, block_number);
		data.push(SubnetNodeConsensusData {
			node_id,
			score,
		});
	}

	let included_subnet_nodes: Vec<SubnetNode<T::AccountId>> = Network::<T>::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);

	ConsensusData {
		validator_id: subnet_id*max_subnet_nodes,
		attests,
		data,
		included_subnet_nodes,
		args: None
	}
}

#[benchmarks]
mod benchmarks {
	use super::*;

	// #[benchmark]
	// fn register_subnet() {
	// 	let epoch_length = T::EpochLength::get();
  //   let block_number = get_current_block_as_u32::<T>();
  //   let epoch = block_number.saturating_div(epoch_length);

	// 	let cost = Network::<T>::registration_cost(epoch);

	// 	let funded_initializer = funded_initializer::<T>("funded_initializer", 0);

	// 	let min_nodes = MinSubnetNodes::<T>::get();
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	let subnets = TotalActiveSubnets::<T>::get() + 1;
	// 	let register_subnet_data: RegistrationSubnetData<T::AccountId> = default_registration_subnet_data::<T>(
	// 		subnets,
	// 		max_subnet_nodes,
	// 		DEFAULT_SUBNET_NAME.into(),
	// 		0, 
	// 		min_nodes + 1
	// 	);
	
	// 	let current_block_number = get_current_block_as_u32::<T>();
	
	// 	#[extrinsic_call]
	// 	register_subnet(RawOrigin::Signed(funded_initializer.clone()), register_subnet_data);

	// 	let owner = SubnetOwner::<T>::get(1).unwrap();
	// 	assert_eq!(owner, funded_initializer.clone());
	
	// 	let subnet = SubnetsData::<T>::get(1).unwrap();
	// 	assert_eq!(subnet.id, 1);
	// 	let path: Vec<u8> = DEFAULT_SUBNET_NAME.into();
	// 	assert_eq!(subnet.name, path);
		
  //   // let minimum_balance = T::Currency::minimum_balance();
	// 	// let pot = Treasury::pot();
	// 	// // let pot = <T as Config>::Treasury::pot();
  //   // assert_eq!(cost, pot + minimum_balance);
	// }

	// #[benchmark]
	// fn get_lowest_stake_balance_node_benchmark() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	#[block]
	// 	{
	// 		let _ = Network::<T>::get_lowest_stake_balance_node(subnet_id);
	// 	}
	// }

	// #[benchmark]
	// fn get_classified_hotkeys() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	#[block]
	// 	{
	// 		let _: Vec<T::AccountId> = Network::<T>::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, 0);
	// 	}
	// }

	// #[benchmark]
	// fn get_classified_subnet_nodes() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
	// 	assert_eq!(max_subnet_nodes, active_nodes);

	// 	#[block]
	// 	{
	// 		let _: Vec<SubnetNode<T::AccountId>> = Network::<T>::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Queue, 0);
	// 	}
	// }

	// #[benchmark]
	// fn perform_remove_subnet_node() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let epoch_length = T::EpochLength::get();
	// 	let block = get_current_block_as_u32::<T>();
	// 	let epoch = block / epoch_length as u32;

	// 	let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", 1*max_subnet_nodes+1);

	// 	let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

	// 	let subnet_node = SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);

	// 	#[block]
	// 	{
	// 		Network::<T>::perform_remove_subnet_node(
	// 			subnet_id,
	// 			hotkey_subnet_node_id, 
	// 		);
	// 	}

	// 	// let subnet_node = SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);
	// 	let subnet_node_id = HotkeySubnetNodeId::<T>::try_get(subnet_id, subnet_node_account.clone());
	// 	assert_eq!(subnet_node_id, Err(()));
	// }

	// #[benchmark]
	// fn do_increase_delegate_stake() {
	// 	let subnet_id = 1;

	// 	let old_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
	// 	let old_total_balance = TotalDelegateStake::<T>::get();

	// 	#[block]
	// 	{
	// 		Network::<T>::do_increase_delegate_stake(
	// 			subnet_id,
	// 			100000000000000000000,
	// 		);
	// 	}

	// 	let balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
	// 	let total_balance = TotalDelegateStake::<T>::get();

	// 	assert!(balance > old_balance);
	// 	assert!(total_balance > old_total_balance);
	// }

	// #[benchmark]
	// fn calculate_stake_weights_normalize_block() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();


	// 	let mut stake_weights: BTreeMap<u32, f64> = BTreeMap::new();
  //   let mut stake_weight_sum: f64 = 0.0;

	// 	let weight: f64 = 100000000000000000000_u128 as f64 / 100000000000000000000000000_u128 as f64;
	// 	let weight_sqrt: f64 = Network::<T>::pow(weight, 0.5);

	// 	stake_weights.insert(1, weight_sqrt);
	// 	stake_weight_sum += weight_sqrt;

	// 	let mut stake_weights_normalized: BTreeMap<u32, u128> = BTreeMap::new();
  //   let percentage_factor = Network::<T>::percentage_factor_as_u128();

	// 	#[block]
	// 	{
	// 		let weight_normalized: u128 = (weight / stake_weight_sum * percentage_factor as f64) as u128;
  //     stake_weights_normalized.insert(1, weight_normalized);
	// 	}
		
	// }

	#[benchmark]
	fn get_epoch_emissions() {
		let max_subnet_nodes = MaxSubnetNodes::<T>::get();
		build_activated_subnet::<T>(
			DEFAULT_SUBNET_NAME.into(), 
			0, 
			max_subnet_nodes, 
			DEFAULT_DEPOSIT_AMOUNT, 
			DEFAULT_SUBNET_NODE_STAKE
		);

		let epoch_length = T::EpochLength::get();
		let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;
		#[block]
		{
			let overall_rewards = Network::<T>::get_epoch_emissions(epoch);
			assert!(overall_rewards > 0);
		}
	}

	#[benchmark]
	fn precheck_consensus_submission() {
		let max_subnet_nodes = MaxSubnetNodes::<T>::get();
		build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

		let epoch_length = T::EpochLength::get();
		let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

		let subnet_nodes: Vec<SubnetNode<T::AccountId>> = Network::<T>::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);
    let subnet_node_count = subnet_nodes.len() as u128;

		let consensus_data = get_subnet_node_consensus_data::<T>(
			subnet_id,
			subnet_node_count as u32,
		);

		SubnetConsensusSubmission::<T>::insert(subnet_id, epoch, consensus_data);

		#[block]
		{
			let result = Network::<T>::precheck_consensus_submission(subnet_id, epoch)
				.ok_or(DispatchError::Other("Precheck consensus failed"));

			assert!(result.is_ok(), "Precheck consensus failed");

			let (consensus_submission_data, consensus_submission_data_weight) = result.unwrap();
		}
	}

	#[benchmark]
	fn calculate_rewards_v2() {
		let max_subnet_nodes = MaxSubnetNodes::<T>::get();
		build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

		let block = get_current_block_as_u32::<T>();
		let epoch_length = T::EpochLength::get();
		let epoch = block / epoch_length as u32;

		// ⸺ Generate subnet weights
		let _ = Network::<T>::handle_subnet_emission_weights(epoch);
		let subnet_emission_weights = FinalSubnetEmissionWeights::<T>::get(epoch);

		let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
		assert!(subnet_weight.is_some());

		// ⸺ Submit consnesus data
		let subnet_nodes: Vec<SubnetNode<T::AccountId>> = Network::<T>::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);
    let subnet_node_count = subnet_nodes.len() as u128;

		let consensus_data = get_subnet_node_consensus_data::<T>(
			subnet_id,
			subnet_node_count as u32,
		);

		// submit data for the previous epoch
		SubnetConsensusSubmission::<T>::insert(subnet_id, epoch - 1, consensus_data);

		#[block]
		{
			let result = Network::<T>::precheck_consensus_submission(subnet_id, epoch - 1)
				.ok_or(DispatchError::Other("Precheck consensus failed"));

			assert!(result.is_ok(), "Precheck consensus failed");
		}
	}

	#[benchmark]
	fn distribute_rewards() {
		let max_subnet_nodes = MaxSubnetNodes::<T>::get();
		build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

		let block = get_current_block_as_u32::<T>();
		let epoch_length = T::EpochLength::get();
		let epoch = block / epoch_length as u32;

		// ⸺ Generate subnet weights
		let _ = Network::<T>::handle_subnet_emission_weights(epoch);
		let subnet_emission_weights = FinalSubnetEmissionWeights::<T>::get(epoch);

		let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
		assert!(subnet_weight.is_some());

		// ⸺ Submit consnesus data
		let subnet_nodes: Vec<SubnetNode<T::AccountId>> = Network::<T>::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);
    let subnet_node_count = subnet_nodes.len() as u128;

		let consensus_data = get_subnet_node_consensus_data::<T>(
			subnet_id,
			subnet_node_count as u32,
		);

		// submit data for the previous epoch
		SubnetConsensusSubmission::<T>::insert(subnet_id, epoch - 1, consensus_data);

		let result = Network::<T>::precheck_consensus_submission(subnet_id, epoch - 1)
			.ok_or(DispatchError::Other("Precheck consensus failed"));

		assert!(result.is_ok(), "Precheck consensus failed");

		let (consensus_submission_data, _) = result.unwrap();

		// ⸺ Calculate subnet distribution of rewards
		let (rewards_data, rewards_weight) = Network::<T>::calculate_rewards_v2(
			subnet_id, 
			subnet_emission_weights.total_issuance, 
			*subnet_weight.unwrap()
		);

		let max_subnet_nodes = MaxSubnetNodes::<T>::get();

		let mut stake_snapshot: BTreeMap<T::AccountId, u128> = BTreeMap::new();
		for n in 0..max_subnet_nodes {
			let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", max_subnet_nodes+n);

			let stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);
			stake_snapshot.insert(subnet_node_account.clone(), stake);
		}

		let min_attestation_percentage = MinAttestationPercentage::<T>::get();
    let reputation_increase_factor = ReputationIncreaseFactor::<T>::get();
    let reputation_decrease_factor = ReputationDecreaseFactor::<T>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<T>::get();

		#[block]
		{
			let _ = Network::<T>::distribute_rewards(
				subnet_id,
				block,
				epoch,
				consensus_submission_data,
				rewards_data,
				min_attestation_percentage,
				reputation_increase_factor,
				reputation_decrease_factor,
				min_vast_majority_attestation_percentage
			);
		}

		for n in 0..max_subnet_nodes {
			let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", max_subnet_nodes+n);

			let stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);

			if let Some(old_stake) = stake_snapshot.get(&subnet_node_account) {
				assert!(stake > *old_stake);
			} else {
				assert!(false); // auto-fail
			}
		}
	}

	#[benchmark]
	fn elect_validator_v3() {
		let max_subnet_nodes = MaxSubnetNodes::<T>::get();
		build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

		let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
		assert_eq!(max_subnet_nodes, active_nodes);

		let current_block_number = get_current_block_as_u32::<T>();
		let epoch_length = T::EpochLength::get();
		let block = get_current_block_as_u32::<T>();
		let epoch = block / epoch_length as u32;

		let slot_list = SubnetNodeElectionSlots::<T>::get(subnet_id);
		assert_eq!(slot_list.len(), active_nodes as usize);

		#[block]
		{
			Network::<T>::elect_validator_v3(
				subnet_id,
				1,
				current_block_number,
			);
		}
	}

	#[benchmark]
	fn get_random_number() {
		let mut parent_hash = frame_system::Pallet::<T>::parent_hash();

		for i in 1..100 {
			frame_system::Pallet::<T>::reset_events();
			let u32_to_block = u32_to_block::<T>(i);
			frame_system::Pallet::<T>::initialize(&u32_to_block, &parent_hash, &Default::default());
			// <frame_system::Pallet<T> as Example>::InsecureRandomnessCollectiveFlip::on_initialize(i);
			// T::Randomness::on_initialize(i);

			let header = frame_system::Pallet::<T>::finalize();
			parent_hash = header.hash();
			frame_system::Pallet::<T>::set_block_number(*header.number());
		}

		let current_block_number = get_current_block_as_u32::<T>();

		#[block]
		{
			Network::<T>::get_random_number(current_block_number);
		}
	}


	// #[benchmark]
	// fn emission_step() {
	// 	let max_subnets = MaxSubnets::<T>::get();
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	for s in 0..max_subnets {
	// 		let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		build_activated_subnet::<T>(
	// 			subnet_name.clone().into(), 
	// 			0, 
	// 			max_subnet_nodes, 
	// 			DEFAULT_DEPOSIT_AMOUNT, 
	// 			DEFAULT_SUBNET_NODE_STAKE
	// 		);
	// 	}

	// 	let block = get_current_block_as_u32::<T>();
	// 	let epoch_length = T::EpochLength::get();
	// 	let epoch = block / epoch_length as u32;

	// 	let overall_rewards: u128 = Network::<T>::get_epoch_emissions(epoch);

	// 	for s in 0..max_subnets {
	// 		let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(subnet_name.clone().into()).unwrap();

	// 		let subnet_nodes: Vec<SubnetNode<T::AccountId>> = Network::<T>::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);
	// 		let subnet_node_count = subnet_nodes.len() as u128;

	// 		let consensus_data = get_subnet_node_consensus_data::<T>(
	// 			subnet_id,
	// 			subnet_node_count as u32,
	// 		);

	// 		SubnetConsensusSubmission::<T>::insert(subnet_id, epoch, consensus_data);		
	// 	}

	// 	let _ = Network::<T>::handle_subnet_emission_weights(epoch);

	// 	let final_weights = FinalSubnetEmissionWeights::<T>::try_get(epoch);
	// 	assert!(final_weights.is_ok());

	// 	let mut stake_snapshot: BTreeMap<T::AccountId, u128> = BTreeMap::new();
	// 	for n in 0..max_subnet_nodes {
	// 		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", max_subnet_nodes+n);

	// 		let stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), 1);
	// 		stake_snapshot.insert(subnet_node_account.clone(), stake);
	// 	}

	// 	#[block]
	// 	{
	// 		let _ = Network::<T>::emission_step(
	// 			block,
	// 			epoch,
	// 			1,
	// 		);
	// 	}

	// 	for n in 0..max_subnet_nodes {
	// 		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", max_subnet_nodes+n);

	// 		let stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), 1);

	// 		if let Some(old_stake) = stake_snapshot.get(&subnet_node_account) {
	// 			assert!(stake > *old_stake);
	// 		} else {
	// 			assert!(false); // auto-fail
	// 		}
	// 	}
	// }


	// #[benchmark]
	// fn emission_step() {
	// 	let max_subnets = MaxSubnets::<T>::get();
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	for s in 0..max_subnets {
	// 		let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		build_activated_subnet::<T>(
	// 			subnet_name.clone().into(), 
	// 			0, 
	// 			max_subnet_nodes, 
	// 			DEFAULT_DEPOSIT_AMOUNT, 
	// 			DEFAULT_SUBNET_NODE_STAKE
	// 		);
	// 	}


	// 	let block = get_current_block_as_u32::<T>();
	// 	let epoch_length = T::EpochLength::get();
	// 	let epoch = block / epoch_length as u32;

	// 	let overall_rewards: u128 = Network::<T>::get_epoch_emissions(epoch);

	// 	for s in 0..max_subnets {
	// 		let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(subnet_name.clone().into()).unwrap();

	// 		let subnet_nodes: Vec<SubnetNode<T::AccountId>> = Network::<T>::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);
	// 		let subnet_node_count = subnet_nodes.len() as u128;

	// 		let consensus_data = get_subnet_node_consensus_data::<T>(
	// 			subnet_id,
	// 			subnet_node_count as u32,
	// 		);

	// 		SubnetConsensusSubmission::<T>::insert(subnet_id, epoch, consensus_data);		
	// 	}

	// 	let _ = Network::<T>::handle_subnet_emission_weights(epoch);

	// 	let final_weights = FinalSubnetEmissionWeights::<T>::try_get(epoch);
	// 	assert!(final_weights.is_ok());
	// 	assert!(final_weights.clone().unwrap().len() > 0);
	// 	assert!(!final_weights.clone().unwrap().is_empty());

	// 	let mut santity_check = false;
	// 	let mut stake_snapshot: BTreeMap<u32, BTreeMap<T::AccountId, u128>> = BTreeMap::new();
	// 	for s in 0..max_subnets {
	// 		let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(subnet_name.clone().into()).unwrap();
	// 		for n in 0..max_subnet_nodes {
	// 			let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", subnet_id*max_subnet_nodes+n);

	// 			let stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);
	// 			assert!(stake > 0);
	// 			stake_snapshot
	// 				.entry(subnet_id)
	// 				.or_default()
	// 				.insert(subnet_node_account.clone(), stake);
	// 			santity_check = true;
	// 		}
	// 	}
	// 	assert!(santity_check);

	// 	#[block]
	// 	{
	// 		let _ = Network::<T>::emission_step(
	// 			block,
	// 			epoch,
	// 		);
	// 	}

	// 	for (subnet_id, node_map) in &stake_snapshot {
	// 		for (account, old_stake) in node_map {
	// 			let current_stake = AccountSubnetStake::<T>::get(account.clone(), *subnet_id);

	// 			assert!(current_stake > *old_stake);
	// 		}
	// 	}
	// }

	// #[benchmark]
	// fn increase_coldkey_reputation() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let epoch_length = T::EpochLength::get();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", max_subnet_nodes+1);

	// 	let reputation_increase_factor = ReputationIncreaseFactor::<T>::get();

	// 	let old_reputation = ColdkeyReputation::<T>::get(&subnet_node_account.clone());
	// 	#[block]
	// 	{
	// 		Network::<T>::increase_coldkey_reputation(
	// 			subnet_node_account.clone(),
	// 			100000000000000000000, 
	// 			660000000000000000, 
	// 			reputation_increase_factor,
	// 			epoch
	// 		);
	// 	}

	// 	let reputation = ColdkeyReputation::<T>::get(&subnet_node_account.clone());
	// 	assert!(reputation.weight > old_reputation.weight);
	// }

	// #[benchmark]
	// fn graduate_class() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes-1, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let epoch_length = T::EpochLength::get();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", max_subnet_nodes+max_subnet_nodes);
	// 	T::Currency::deposit_creating(&subnet_node_account, (DEFAULT_DEPOSIT_AMOUNT + 500).try_into().ok().expect("REASON"));

  //   assert_ok!(
  //     Network::<T>::register_subnet_node(
  //       RawOrigin::Signed(subnet_node_account.clone()).into(),
  //       subnet_id,
	// 			subnet_node_account.clone(),
  //       peer(max_subnet_nodes+max_subnet_nodes),
	// 			peer(max_subnet_nodes+max_subnet_nodes),
	// 			0,
  //       DEFAULT_DEPOSIT_AMOUNT,
  //       None,
  //       None,
  //       None,
  //     ) 
  //   );

	// 	let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

	// 	let subnet_node = RegisteredSubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);
  //   let start_epoch = subnet_node.classification.start_epoch;

	// 	increase_epochs::<T>(start_epoch - epoch);

	// 	assert_ok!(
	// 		Network::<T>::activate_subnet_node(
	// 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 			subnet_id, 
	// 			hotkey_subnet_node_id
	// 		)
  //   );

	// 	let subnet_node = SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);
	// 	let old_node_class = subnet_node.classification.node_class;

	// 	#[block]
	// 	{
	// 		Network::<T>::graduate_class(
	// 			subnet_id,
	// 			hotkey_subnet_node_id, 
	// 			epoch
	// 		);
	// 	}

	// 	let subnet_node = SubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);
	// 	let node_class = subnet_node.classification.node_class;

	// 	assert!(node_class > old_node_class);
	// }

	// #[benchmark]
	// fn do_increase_node_delegate_stake() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let epoch_length = T::EpochLength::get();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

  //   // -- increase total subnet delegate stake 
  //   let old_node_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, 1);

  //   let old_total_node_stake_balance = TotalNodeDelegateStake::<T>::get();

	// 	#[block]
	// 	{
	// 		Network::<T>::do_increase_node_delegate_stake(
	// 			subnet_id,
	// 			1, 
	// 			100000000000000000000000000
	// 		);
	// 	}

  //   let node_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, 1);

  //   let total_node_stake_balance = TotalNodeDelegateStake::<T>::get();

	// 	assert!(old_node_stake_balance < node_stake_balance);
	// 	assert!(old_total_node_stake_balance < total_node_stake_balance);
	// }

	// #[benchmark]
	// fn increase_account_stake() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let epoch_length = T::EpochLength::get();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", max_subnet_nodes+max_subnet_nodes);

	// 	let old_account_stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);
	// 	let old_subnet_stake = TotalSubnetStake::<T>::get(subnet_id);
	// 	let old_total_stake = TotalStake::<T>::get();

	// 	#[block]
	// 	{
	// 		Network::<T>::increase_account_stake(
	// 			&subnet_node_account.clone(),
	// 			subnet_id, 
	// 			100000000000000000000000000
	// 		);
	// 	}

	// 	let account_stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);
	// 	let subnet_stake = TotalSubnetStake::<T>::get(subnet_id);
	// 	let total_stake = TotalStake::<T>::get();

	// 	assert!(old_account_stake < account_stake);
	// 	assert!(old_subnet_stake < subnet_stake);
	// 	assert!(old_total_stake < total_stake);
	// }

	// #[benchmark]
	// fn do_remove_subnet() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
	// 	assert_eq!(max_subnet_nodes, active_nodes);

	// 	#[block]
	// 	{
	// 		Network::<T>::do_remove_subnet(subnet_id, SubnetRemovalReason::MaxPenalties);
	// 	}
	// }






























	// #[benchmark]
	// fn generate_rewards_for_subnet() {
	// 	let max_subnet_nodes = MaxSubnetNodes::<T>::get();
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, max_subnet_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let epoch_length = T::EpochLength::get();
	// 	let current_block = get_current_block_as_u32::<T>();
	// 	let epoch = current_block / epoch_length as u32;

	// 	let subnet_nodes: Vec<SubnetNode<T::AccountId>> = Network::<T>::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);
  //   let subnet_node_count = subnet_nodes.len() as u128;

	// 	let overall_rewards: u128 = Network::<T>::get_epoch_emissions(epoch);

	// 	let stake_weights_normalized = Network::<T>::calculate_stake_weights(epoch);

	// 	#[block]
	// 	{
	// 		let _ = Network::<T>::generate_rewards_for_subnet(
	// 			current_block, 
	// 			epoch, 
	// 			subnet_id, 
	// 			overall_rewards,
	// 			stake_weights_normalized
	// 		)
	// 	}
	// }

	// #[benchmark]
	// fn activate_subnet() {
	// 	let funded_initializer = funded_initializer::<T>("funded_initializer", 0);
	// 	let start: u32 = 0; 
	// 	let mut end: u32 = 12; 
	// 	let deposit_amount: u128 = DEFAULT_DEPOSIT_AMOUNT;
	// 	let amount: u128 = DEFAULT_SUBNET_NODE_STAKE;
	// 	let min_nodes = MinSubnetNodes::<T>::get();
	// 	let whitelist = get_initial_coldkeys::<T>(0, end + 1);

	// 	let register_subnet_data = RegistrationSubnetData {
	// 		name: DEFAULT_SUBNET_NAME.into(),
	// 		repo: Vec::new(),
	// 		description: Vec::new(),
	// 		misc: Vec::new(),
	// 		churn_limit: 4,
	// 		registration_queue_epochs: 4,
	// 		activation_grace_epochs: 4,
	// 		queue_classification_epochs: 4,
	// 		included_classification_epochs: 4,
	// 		max_node_penalties: 3,
	// 		initial_coldkeys: whitelist
	// 	};
	
	// 	assert_ok!(
	// 		Network::<T>::register_subnet(
	// 			RawOrigin::Signed(funded_initializer.clone()).into(), 
	// 			register_subnet_data
	// 		)
	// 	);

	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

	// 	assert_eq!(subnet.id, subnet_id);

	// 	let epoch_length = T::EpochLength::get();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length;
	
	// 	let block_number = get_current_block_as_u32::<T>();
	// 	let mut amount_staked = 0;
	// 	for n in start..end+1 {
	// 		let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", n);
	// 		T::Currency::deposit_creating(&subnet_node_account, DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));
	// 		amount_staked += amount;
	// 		assert_ok!(
	// 			Network::<T>::add_subnet_node(
	// 				RawOrigin::Signed(subnet_node_account.clone()).into(),
	// 				subnet_id,
	// 				subnet_node_account.clone(),
	// 				peer(n),
	// 				peer(n),
	// 				0,
	// 				amount,
	// 				None,
	// 				None,
	// 				None,
	// 			) 
	// 		);
	// 		let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

	// 		let subnet_node_id_hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, hotkey_subnet_node_id).unwrap();
	// 		assert_eq!(subnet_node_id_hotkey, subnet_node_account.clone());

	// 		let subnet_node_data = SubnetNodesData::<T>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
	// 		assert_eq!(subnet_node_data.hotkey, subnet_node_account.clone());
	// 		assert_eq!(subnet_node_data.delegate_reward_rate, 0);

	// 		let key_owner = HotkeyOwner::<T>::get(subnet_node_data.hotkey.clone());
	// 		assert_eq!(key_owner, subnet_node_account.clone());

	// 		assert_eq!(subnet_node_data.peer_id, peer(n));

	// 		// --- Is ``Validator`` if registered before subnet activation
	// 		assert_eq!(subnet_node_data.classification.node_class, SubnetNodeClass::Validator);
	// 		assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, epoch));

	// 		let peer_subnet_node_account = PeerIdSubnetNode::<T>::get(subnet_id, peer(n));
	// 		assert_eq!(peer_subnet_node_account, hotkey_subnet_node_id);

	// 		let account_subnet_stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);
	// 		assert_eq!(account_subnet_stake, amount);
	// 	}
	
	// 	let total_subnet_stake = TotalSubnetStake::<T>::get(subnet_id);
	// 	assert_eq!(total_subnet_stake, amount_staked);
	
	// 	let total_stake = TotalStake::<T>::get();
	// 	assert_eq!(total_subnet_stake, amount_staked);
	
	// 	let total_subnet_nodes = TotalSubnetNodes::<T>::get(subnet_id);
	// 	assert!(total_subnet_nodes > min_nodes);

	// 	// --- Add the minimum required delegate stake balance to activate the subnet
	// 	// Add 100e18 to account for block increase on activation
	// 	let min_subnet_delegate_stake = Network::<T>::get_min_subnet_delegate_stake_balance() + 100e+18 as u128;
	
	// 	let delegate_staker_account: T::AccountId = funded_account::<T>("subnet_node_account", 1);
	// 	T::Currency::deposit_creating(&delegate_staker_account, min_subnet_delegate_stake.try_into().ok().expect("REASON"));
	// 	assert_ok!(
	// 		Network::<T>::add_to_delegate_stake(
	// 			RawOrigin::Signed(delegate_staker_account.clone()).into(),
	// 			subnet_id,
	// 			min_subnet_delegate_stake,
	// 		) 
	// 	);
		
	// 	let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
	// 	assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

	// 	let epochs = SubnetRegistrationEpochs::<T>::get();
	// 	increase_epochs::<T>(epochs + 1);
		
	// 	#[extrinsic_call]
	// 	activate_subnet(RawOrigin::Signed(funded_initializer.clone()), subnet_id);

	// 	let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

  //   assert_eq!(subnet.id, subnet_id);

  //   // ensure subnet exists and nothing changed but the activation block
	// 	let path: Vec<u8> = DEFAULT_SUBNET_NAME.into();
  //   assert_eq!(subnet.name, path);
  //   assert_eq!(subnet.state, SubnetState::Active);
	// }

	// #[benchmark]
	// fn add_subnet_node() {
	// 	let funded_initializer = funded_initializer::<T>("funded_initializer", 0);
	// 	let start: u32 = 0; 
	// 	let mut end: u32 = 12; 
	// 	let deposit_amount: u128 = DEFAULT_DEPOSIT_AMOUNT;
	// 	let amount: u128 = DEFAULT_SUBNET_NODE_STAKE;
	// 	let min_nodes = MinSubnetNodes::<T>::get();
	// 	let whitelist = get_initial_coldkeys::<T>(0, end);

	// 	let register_subnet_data = RegistrationSubnetData {
	// 		name: DEFAULT_SUBNET_NAME.into(),
	// 		repo: Vec::new(),
	// 		description: Vec::new(),
	// 		misc: Vec::new(),
	// 		churn_limit: 4,
	// 		registration_queue_epochs: 4,
	// 		activation_grace_epochs: 4,
	// 		queue_classification_epochs: 4,
	// 		included_classification_epochs: 4,
	// 		max_node_penalties: 3,
	// 		initial_coldkeys: whitelist
	// 	};
	
	// 	assert_ok!(
	// 		Network::<T>::register_subnet(
	// 			RawOrigin::Signed(funded_initializer.clone()).into(), 
	// 			register_subnet_data
	// 		)
	// 	);

	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", 1);

	// 	let epoch_length = T::EpochLength::get();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length;

	// 	#[extrinsic_call]
	// 	add_subnet_node(
	// 		RawOrigin::Signed(subnet_node_account.clone()), 
	// 		subnet_id, 
	// 		subnet_node_account.clone(),
	// 		peer(1), 
	// 		peer(1),
	// 		0,
	// 		DEFAULT_SUBNET_NODE_STAKE,
	// 		None,
	// 		None,
	// 		None,
	// 	);
		
	// 	let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

	// 	let subnet_node_id_hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, hotkey_subnet_node_id).unwrap();
	// 	assert_eq!(subnet_node_id_hotkey, subnet_node_account.clone());

	// 	let subnet_node_data = SubnetNodesData::<T>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
	// 	assert_eq!(subnet_node_data.hotkey, subnet_node_account.clone());
	// 	assert_eq!(subnet_node_data.delegate_reward_rate, 0);

	// 	let key_owner = HotkeyOwner::<T>::get(subnet_node_data.hotkey.clone());
	// 	assert_eq!(key_owner, subnet_node_account.clone());

	// 	assert_eq!(subnet_node_data.peer_id, peer(1));

	// 	// --- Is ``Validator`` if registered before subnet activation
	// 	assert_eq!(subnet_node_data.classification.node_class, SubnetNodeClass::Validator);
	// 	assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, epoch));

	// 	let peer_subnet_node_account = PeerIdSubnetNode::<T>::get(subnet_id, peer(1));
	// 	assert_eq!(peer_subnet_node_account, hotkey_subnet_node_id);

	// 	let account_subnet_stake = AccountSubnetStake::<T>::get(subnet_node_account.clone(), subnet_id);
	// 	assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE);
	// }

	// #[benchmark]
	// fn register_subnet_node() {
	// 	let funded_initializer = funded_initializer::<T>("funded_initializer", 0);
	// 	let start: u32 = 0; 
	// 	let mut end: u32 = 12; 
	// 	let deposit_amount: u128 = DEFAULT_DEPOSIT_AMOUNT;
	// 	let amount: u128 = DEFAULT_SUBNET_NODE_STAKE;
	// 	let min_nodes = MinSubnetNodes::<T>::get();
	// 	let whitelist = get_initial_coldkeys::<T>(0, end);

	// 	let seed_bytes: &[u8] = &DEFAULT_SUBNET_NAME.into();
	// 	let register_subnet_data = RegistrationSubnetData {
	// 		name: DEFAULT_SUBNET_NAME.into(),
	// 		repo: Vec::new(),
	// 		description: Vec::new(),
	// 		misc: Vec::new(),
	// 		churn_limit: 4,
	// 		registration_queue_epochs: 4,
	// 		activation_grace_epochs: 4,
	// 		queue_classification_epochs: 4,
	// 		included_classification_epochs: 4,
	// 		max_node_penalties: 3,
	// 		initial_coldkeys: whitelist
	// 	};
	
	// 	assert_ok!(
	// 		Network::<T>::register_subnet(
	// 			RawOrigin::Signed(funded_initializer.clone()).into(), 
	// 			register_subnet_data
	// 		)
	// 	);

	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let subnet_node_account = funded_account::<T>("subnet_node_account", 1);

	// 	#[extrinsic_call]
	// 	register_subnet_node(
	// 		RawOrigin::Signed(subnet_node_account.clone()), 
	// 		subnet_id, 
	// 		subnet_node_account.clone(),
	// 		peer(1), 
	// 		peer(1), 
	// 		0,
	// 		DEFAULT_SUBNET_NODE_STAKE,
	// 		None,
	// 		None,
	// 		None,
	// 	);

	// 	// assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), end);
	// 	let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

  //   let subnet_node_id_hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, hotkey_subnet_node_id).unwrap();
  //   assert_eq!(subnet_node_id_hotkey, subnet_node_account.clone());

	// }

	// #[benchmark]
	// fn activate_subnet_node() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_node_account = funded_account::<T>("subnet_node_account", end);

	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	assert_ok!(
	// 		Network::<T>::register_subnet_node(
	// 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 			subnet_id, 
	// 			subnet_node_account.clone(),
	// 			peer(end), 
	// 			peer(end), 
	// 			0,
	// 			DEFAULT_SUBNET_NODE_STAKE,
	// 			None,
	// 			None,
	// 			None,
	// 		) 
	// 	);

	// 	let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

	// 	let subnet_node = RegisteredSubnetNodesData::<T>::get(subnet_id, hotkey_subnet_node_id);
  //   let start_epoch = subnet_node.classification.start_epoch;

	// 	let epoch_length = T::EpochLength::get();
	// 	let current_epoch = get_current_block_as_u32::<T>() / epoch_length;

	// 	increase_epochs::<T>(start_epoch - current_epoch);

	// 	#[extrinsic_call]
	// 	activate_subnet_node(RawOrigin::Signed(subnet_node_account.clone()), subnet_id, hotkey_subnet_node_id);

	// 	assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), end+1);
	// }

	// // // #[benchmark]
	// // // fn deactivate_subnet_node() {
	// // // 	let end = 12;
	// // // 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// // // 	let subnet_node_account = funded_account::<T>("subnet_node_account", end+1);

	// // // 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// // // 	assert_ok!(
	// // // 		Network::<T>::register_subnet_node(
	// // // 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// // // 			subnet_id, 
	// // // 			peer(end+1), 
	// // // 			DEFAULT_SUBNET_NODE_STAKE,
	// // // 			None,
	// // // 			None,
	// // // 			None,
	// // // 		) 
	// // // 	);

	// // // 	assert_ok!(
	// // // 		Network::<T>::activate_subnet_node(
	// // // 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// // // 			subnet_id, 
	// // // 		) 
	// // // 	);

	// // // 	#[extrinsic_call]
	// // // 	deactivate_subnet_node(RawOrigin::Signed(subnet_node_account.clone()), subnet_id);

	// // // 	assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), end+1);
	// // // }

	// #[benchmark]
	// fn remove_subnet_node() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", 1);

	// 	let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

	// 	#[extrinsic_call]
	// 	remove_subnet_node(RawOrigin::Signed(subnet_node_account.clone()), subnet_id, hotkey_subnet_node_id);
		
	// 	assert_eq!(TotalSubnetNodes::<T>::get(subnet_id), end-1);

	// 	let subnet_node_id = HotkeySubnetNodeId::<T>::try_get(subnet_id, subnet_node_account.clone());
	// 	assert_eq!(subnet_node_id, Err(()));

	// 	let subnet_node_account = PeerIdSubnetNode::<T>::try_get(subnet_id, peer(end+1));
	// 	assert_eq!(subnet_node_account, Err(()));
	// }

	// #[benchmark]
	// fn add_to_stake() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", end+1);
	// 	assert_ok!(
	// 		Network::<T>::register_subnet_node(
	// 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 			subnet_id, 
	// 			subnet_node_account.clone(),
	// 			peer(end+1), 
	// 			peer(end+1), 
	// 			0,
	// 			DEFAULT_SUBNET_NODE_STAKE,
	// 			None,
	// 			None,
	// 			None,
  //     )
	// 	);

	// 	T::Currency::deposit_creating(&subnet_node_account, DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));

	// 	let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

	// 	#[extrinsic_call]
	// 	add_to_stake(
	// 		RawOrigin::Signed(subnet_node_account.clone()), 
	// 		subnet_id, 
	// 		hotkey_subnet_node_id,
	// 		subnet_node_account.clone(),
	// 		DEFAULT_STAKE_TO_BE_ADDED
	// 	);
		
	// 	let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node_account.clone(), subnet_id);
	// 	assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE + DEFAULT_STAKE_TO_BE_ADDED);
	// }

	// #[benchmark]
	// fn remove_stake() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let subnet_node_account: T::AccountId = funded_account::<T>("subnet_node_account", end+1);
	// 	assert_ok!(
	// 		Network::<T>::register_subnet_node(
	// 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 			subnet_id, 
	// 			subnet_node_account.clone(),
	// 			peer(end+1), 
	// 			peer(end+1), 
	// 			0,
	// 			DEFAULT_SUBNET_NODE_STAKE,
	// 			None,
	// 			None,
	// 			None,
  //     )
	// 	);

	// 	T::Currency::deposit_creating(&subnet_node_account, DEFAULT_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));
	// 	let hotkey_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, subnet_node_account.clone()).unwrap();

	// 	assert_ok!(
	// 		Network::<T>::add_to_stake(
	// 			RawOrigin::Signed(subnet_node_account.clone()).into(), 
	// 			subnet_id, 
	// 			hotkey_subnet_node_id,
	// 			subnet_node_account.clone(),
	// 			DEFAULT_STAKE_TO_BE_ADDED
	// 		)
	// 	);
	// 	let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node_account.clone(), subnet_id);
	// 	assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE + DEFAULT_STAKE_TO_BE_ADDED);

	// 	#[extrinsic_call]
	// 	remove_stake(RawOrigin::Signed(subnet_node_account.clone()), subnet_id, subnet_node_account.clone(), DEFAULT_STAKE_TO_BE_ADDED);
		
	// 	let account_subnet_stake = Network::<T>::account_subnet_stake(subnet_node_account.clone(), subnet_id);
	// 	assert_eq!(account_subnet_stake, DEFAULT_SUBNET_NODE_STAKE);
	// }

	// #[benchmark]
	// fn add_to_delegate_stake() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

  //   let _ = T::Currency::deposit_creating(&delegate_account.clone(), (DEFAULT_STAKE_TO_BE_ADDED + 500).try_into().ok().expect("REASON"));
  //   let starting_delegator_balance = T::Currency::free_balance(&delegate_account.clone());

	// 	#[extrinsic_call]
	// 	add_to_delegate_stake(RawOrigin::Signed(delegate_account.clone()), subnet_id, DEFAULT_DELEGATE_STAKE_TO_BE_ADDED);

  //   let post_delegator_balance = T::Currency::free_balance(&delegate_account.clone());
  //   assert_eq!(post_delegator_balance, starting_delegator_balance - DEFAULT_DELEGATE_STAKE_TO_BE_ADDED.try_into().ok().expect("REASON"));

  //   let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
  //   let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
  //   let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
  //   let delegate_balance = Network::<T>::convert_to_balance(
  //     delegate_shares,
  //     total_subnet_delegated_stake_shares,
  //     total_subnet_delegated_stake_balance
  //   );

  //   // Ensure balance is within <= 0.01% of deposited balance, and less than deposited balance
  //   assert!(
  //     (delegate_balance >= Network::<T>::percent_mul(DEFAULT_DELEGATE_STAKE_TO_BE_ADDED, 990000000)) &&
  //     (delegate_balance < DEFAULT_DELEGATE_STAKE_TO_BE_ADDED)
  //   );
	// }

	// #[benchmark]
	// fn swap_delegate_stake() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let from_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_PATH_2.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let to_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH_2.into()).unwrap();

	// 	let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

	// 	assert_ok!(
	// 		Network::<T>::add_to_delegate_stake(
	// 			RawOrigin::Signed(delegate_account.clone()).into(), 
	// 			from_subnet_id, 
	// 			DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
	// 		)
	// 	);

	// 	let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);
	// 	let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(from_subnet_id);
  //   let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id);

	// 	let from_delegate_balance = Network::<T>::convert_to_balance(
  //     delegate_shares,
  //     total_subnet_delegated_stake_shares,
  //     total_subnet_delegated_stake_balance
  //   );

	// 	#[extrinsic_call]
	// 	swap_delegate_stake(
	// 		RawOrigin::Signed(delegate_account.clone()), 
	// 		from_subnet_id, 
	// 		to_subnet_id, 
	// 		delegate_shares
	// 	);

  //   let from_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);
  //   assert_eq!(from_delegate_shares, 0);

  //   let to_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), to_subnet_id);
  //   assert_ne!(to_delegate_shares, 0);

  //   let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(to_subnet_id);
  //   let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(to_subnet_id);

  //   let to_delegate_balance = Network::<T>::convert_to_balance(
  //     to_delegate_shares,
  //     total_subnet_delegated_stake_shares,
  //     total_subnet_delegated_stake_balance
  //   );
  //   // The first depositor will lose a percentage of their deposit depending on the size
  //   // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
  //   // Will lose about .01% of the transfer value on first transfer into a pool
  //   // The balance should be about ~99% of the ``from`` subnet to the ``to`` subnet
  //   assert!(
  //     (to_delegate_balance >= Network::<T>::percent_mul(from_delegate_balance, 990000000)) &&
  //     (to_delegate_balance <= from_delegate_balance)
  //   );
	// }

	// #[benchmark]
	// fn remove_delegate_stake() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
	// 	assert_ok!(
	// 		Network::<T>::add_to_delegate_stake(
	// 			RawOrigin::Signed(delegate_account.clone()).into(), 
	// 			subnet_id, 
	// 			DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
	// 		)
	// 	);
	// 	let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);

	// 	let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
  //   let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

	// 	let delegate_balance = Network::<T>::convert_to_balance(
  //     delegate_shares,
  //     total_subnet_delegated_stake_shares,
  //     total_subnet_delegated_stake_balance
  //   );

	// 	let epoch_length = T::EpochLength::get();
	// 	let current_epoch = get_current_block_as_u32::<T>() / epoch_length;

	// 	#[extrinsic_call]
	// 	remove_delegate_stake(
	// 		RawOrigin::Signed(delegate_account.clone()), 
	// 		subnet_id, 
	// 		delegate_shares
	// 	);

  //   let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<T>::get(delegate_account.clone());
  //   assert_eq!(unbondings.len(), 1);

	// 	let (epoch, balance) = unbondings.iter().next().unwrap();
  //   assert_eq!(*epoch, current_epoch + T::DelegateStakeCooldownEpochs::get());
  //   assert_eq!(*balance, delegate_balance);
	// }

	// #[benchmark]
	// fn claim_unbondings() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
	// 	assert_ok!(
	// 		Network::<T>::add_to_delegate_stake(
	// 			RawOrigin::Signed(delegate_account.clone()).into(), 
	// 			subnet_id, 
	// 			DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
	// 		)
	// 	);
	// 	let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);

	// 	let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
  //   let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

	// 	let delegate_balance = Network::<T>::convert_to_balance(
  //     delegate_shares,
  //     total_subnet_delegated_stake_shares,
  //     total_subnet_delegated_stake_balance
  //   );

	// 	let epoch_length = T::EpochLength::get();
	// 	let current_epoch = get_current_block_as_u32::<T>() / epoch_length;

	// 	assert_ok!(
	// 		Network::<T>::remove_delegate_stake(
	// 			RawOrigin::Signed(delegate_account.clone()).into(), 
	// 			subnet_id, 
	// 			delegate_shares
	// 		)
	// 	);

	// 	let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<T>::get(delegate_account.clone());
  //   assert_eq!(unbondings.len(), 1);

	// 	let (epoch, balance) = unbondings.iter().next().unwrap();
  //   assert_eq!(*epoch, current_epoch + T::DelegateStakeCooldownEpochs::get());
  //   assert_eq!(*balance, delegate_balance);

	// 	let pre_delegator_balance: u128 = T::Currency::free_balance(&delegate_account.clone()).try_into().ok().expect("REASON");		

	// 	let current_block_number = get_current_block_as_u32::<T>();
	// 	frame_system::Pallet::<T>::set_block_number(u32_to_block::<T>(current_block_number + ((epoch_length  + 1) * T::DelegateStakeCooldownEpochs::get())));

	// 	#[extrinsic_call]
	// 	claim_unbondings(RawOrigin::Signed(delegate_account.clone()));

	// 	let post_delegator_balance: u128 = T::Currency::free_balance(&delegate_account.clone()).try_into().ok().expect("REASON");

	// 	assert_eq!(post_delegator_balance, pre_delegator_balance + balance);
	// }

	// #[benchmark]
	// fn increase_delegate_stake() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);
	// 	assert_ok!(
	// 		Network::<T>::add_to_delegate_stake(
	// 			RawOrigin::Signed(delegate_account.clone()).into(), 
	// 			subnet_id, 
	// 			DEFAULT_DELEGATE_STAKE_TO_BE_ADDED
	// 		)
	// 	);

	// 	let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
	// 	let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
  //   let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

	// 	let delegate_balance = Network::<T>::convert_to_balance(
  //     delegate_shares,
  //     total_subnet_delegated_stake_shares,
  //     total_subnet_delegated_stake_balance
  //   );

	// 	let funder = funded_account::<T>("funder", 0);

	// 	#[extrinsic_call]
	// 	increase_delegate_stake(RawOrigin::Signed(funder), subnet_id, DEFAULT_SUBNET_NODE_STAKE);
		
	// 	let increased_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), subnet_id);
	// 	let increased_total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
  //   let increased_total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

	// 	let increased_delegate_balance = Network::<T>::convert_to_balance(
  //     increased_delegate_shares,
  //     increased_total_subnet_delegated_stake_shares,
  //     increased_total_subnet_delegated_stake_balance
  //   );
	// 	assert_eq!(increased_total_subnet_delegated_stake_balance, total_subnet_delegated_stake_balance + DEFAULT_SUBNET_NODE_STAKE);

	// 	assert_ne!(increased_delegate_balance, delegate_balance);
	// 	assert!(increased_delegate_balance > delegate_balance);
	// }

	// #[benchmark]
	// fn add_to_node_delegate_stake() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	let subnet_node_id = end;

	// 	let delegate_node_account: T::AccountId = funded_account::<T>("delegate_node_account", 0);

	// 	#[extrinsic_call]
	// 	add_to_node_delegate_stake(RawOrigin::Signed(delegate_node_account.clone()), subnet_id, subnet_node_id, DEFAULT_SUBNET_NODE_STAKE);
		
  //   let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), subnet_id, subnet_node_id));
  //   let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);
  //   let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);

  //   let account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
  //     account_node_delegate_stake_shares,
  //     total_node_delegate_stake_shares,
  //     total_node_delegate_stake_balance
  //   );

  //   assert!(
  //     (account_node_delegate_stake_balance >= Network::<T>::percent_mul(DEFAULT_SUBNET_NODE_STAKE, 990000000)) &&
  //     (account_node_delegate_stake_balance <= DEFAULT_SUBNET_NODE_STAKE)
  //   );
	// }

	// #[benchmark]
	// fn swap_node_delegate_stake() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let from_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	let from_subnet_node_id = end;

	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_PATH_2.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let to_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH_2.into()).unwrap();
	// 	let to_subnet_node_id = end;

	// 	let delegate_node_account: T::AccountId = funded_account::<T>("delegate_node_account", 0);

	// 	assert_ok!(
	// 		Network::<T>::add_to_node_delegate_stake(
	// 			RawOrigin::Signed(delegate_node_account.clone()).into(), 
	// 			from_subnet_id, 
	// 			from_subnet_node_id, 
	// 			DEFAULT_SUBNET_NODE_STAKE
	// 		)
	// 	);

	// 	let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(from_subnet_id, from_subnet_node_id);
  //   let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(from_subnet_id, from_subnet_node_id);

	// 	let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), from_subnet_id, from_subnet_node_id));
  //   let account_node_delegate_stake_shares_to_be_removed = account_node_delegate_stake_shares / 2;

	// 	let expected_balance_to_be_removed = Network::<T>::convert_to_balance(
  //     account_node_delegate_stake_shares_to_be_removed,
  //     total_node_delegate_stake_shares,
  //     total_node_delegate_stake_balance
  //   );

	// 	let expected_post_balance = Network::<T>::convert_to_balance(
  //     account_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
  //     total_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
  //     total_node_delegate_stake_balance - expected_balance_to_be_removed
  //   );

	// 	#[extrinsic_call]
	// 	swap_node_delegate_stake(
	// 		RawOrigin::Signed(delegate_node_account.clone()), 
	// 		from_subnet_id,
	// 		from_subnet_node_id,
	// 		to_subnet_id,
	// 		to_subnet_node_id,
	// 		account_node_delegate_stake_shares_to_be_removed,
	// 	);
		
  //   let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), from_subnet_id, from_subnet_node_id));
  //   let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(from_subnet_id, from_subnet_node_id);
  //   let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(from_subnet_id, from_subnet_node_id);

  //   let account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
  //     account_node_delegate_stake_shares,
  //     total_node_delegate_stake_shares,
  //     total_node_delegate_stake_balance
  //   );

	// 	assert_eq!(account_node_delegate_stake_balance, expected_post_balance);




	// 	let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), to_subnet_id, to_subnet_node_id));
  //   let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(to_subnet_id, to_subnet_node_id);
  //   let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(to_subnet_id, to_subnet_node_id);

  //   let account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
  //     account_node_delegate_stake_shares,
  //     total_node_delegate_stake_shares,
  //     total_node_delegate_stake_balance
  //   );

  //   assert_ne!(account_node_delegate_stake_balance, 0);

  //   assert!(
  //     (account_node_delegate_stake_balance >= Network::<T>::percent_mul(expected_balance_to_be_removed, 990000000)) &&
  //     (account_node_delegate_stake_balance <= expected_balance_to_be_removed)
  //   );

  //   // Ensure the code didn't create an unbonding insert
  //   let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<T>::get(delegate_node_account.clone());
  //   assert_eq!(unbondings.len(), 0);
	// }

	// #[benchmark]
	// fn remove_node_delegate_stake() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	let subnet_node_id = end;

	// 	let delegate_node_account: T::AccountId = funded_account::<T>("delegate_node_account", 0);
	// 	assert_ok!(
	// 		Network::<T>::add_to_node_delegate_stake(
	// 			RawOrigin::Signed(delegate_node_account.clone()).into(), 
	// 			subnet_id, 
	// 			subnet_node_id, 
	// 			DEFAULT_SUBNET_NODE_STAKE
	// 		)
	// 	);

  //   let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);
  //   let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);

	// 	let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), subnet_id, subnet_node_id));
	// 	let account_node_delegate_stake_shares_to_be_removed = account_node_delegate_stake_shares / 2;

	// 	let expected_balance_to_be_removed = Network::<T>::convert_to_balance(
  //     account_node_delegate_stake_shares_to_be_removed,
  //     total_node_delegate_stake_shares,
  //     total_node_delegate_stake_balance
  //   );

  //   let expected_post_balance = Network::<T>::convert_to_balance(
  //     account_node_delegate_stake_shares_to_be_removed,
  //     total_node_delegate_stake_shares - account_node_delegate_stake_shares_to_be_removed,
  //     total_node_delegate_stake_balance - expected_balance_to_be_removed
  //   );

	// 	let epoch_length = T::EpochLength::get();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	#[extrinsic_call]
	// 	remove_node_delegate_stake(RawOrigin::Signed(delegate_node_account.clone()), subnet_id, subnet_node_id, account_node_delegate_stake_shares_to_be_removed);
		
  //   let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_node_account.clone(), subnet_id, subnet_node_id));
  //   let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);
  //   let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);

  //   assert_eq!(account_node_delegate_stake_shares, account_node_delegate_stake_shares_to_be_removed);

  //   let post_account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
  //     account_node_delegate_stake_shares,
  //     total_node_delegate_stake_shares,
  //     total_node_delegate_stake_balance
  //   );

  //   assert_eq!(expected_post_balance, post_account_node_delegate_stake_balance);

  //   let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<T>::get(delegate_node_account.clone());
  //   assert_eq!(unbondings.len(), 1);
  //   let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
  //   assert_eq!(*ledger_epoch, &epoch + T::NodeDelegateStakeCooldownEpochs::get());
  //   assert_eq!(*ledger_balance, expected_balance_to_be_removed);
	// }

	// #[benchmark]
	// fn increase_node_delegate_stake() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	let subnet_node_id = end;

	// 	let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

	// 	let pre_total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);

	// 	#[extrinsic_call]
	// 	increase_node_delegate_stake(RawOrigin::Signed(delegate_account), subnet_id, subnet_node_id, DEFAULT_SUBNET_NODE_STAKE);
		
	// 	let post_total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);

	// 	assert_eq!(pre_total_node_delegate_stake_balance + DEFAULT_SUBNET_NODE_STAKE, post_total_node_delegate_stake_balance);
	// }

	// #[benchmark]
	// fn transfer_from_node_to_subnet() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let from_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	let from_subnet_node_id = end;

	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_PATH_2.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let to_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH_2.into()).unwrap();

	// 	let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

  //   assert_ok!(
  //     Network::<T>::add_to_node_delegate_stake(
  //       RawOrigin::Signed(delegate_account.clone()).into(), 
	// 			from_subnet_id, 
	// 			from_subnet_node_id, 
	// 			DEFAULT_SUBNET_NODE_STAKE
  //     )
  //   );

	// 	let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_account.clone(), from_subnet_id, from_subnet_node_id));
  //   let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(from_subnet_id, from_subnet_node_id);
  //   let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(from_subnet_id, from_subnet_node_id);

  //   let account_node_delegate_stake_shares_to_be_removed = account_node_delegate_stake_shares / 2;

  //   let expected_balance_to_be_removed = Network::<T>::convert_to_balance(
  //     account_node_delegate_stake_shares_to_be_removed,
  //     total_node_delegate_stake_shares,
  //     total_node_delegate_stake_balance
  //   );

	// 	#[extrinsic_call]
	// 	transfer_from_node_to_subnet(
	// 		RawOrigin::Signed(delegate_account.clone()), 
	// 		from_subnet_id, 
	// 		from_subnet_node_id, 
	// 		to_subnet_id,
	// 		account_node_delegate_stake_shares_to_be_removed
	// 	);
		
  //   let to_delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), to_subnet_id);
  //   assert_ne!(to_delegate_shares, 0);

  //   let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(to_subnet_id);
  //   let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(to_subnet_id);

  //   let mut to_delegate_balance = Network::<T>::convert_to_balance(
  //     to_delegate_shares,
  //     total_subnet_delegated_stake_shares,
  //     total_subnet_delegated_stake_balance
  //   );

	// 	// The first depositor will lose a percentage of their deposit depending on the size
  //   // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
  //   // Will lose about .01% of the transfer value on first transfer into a pool
  //   // The balance should be about ~99% of the ``from`` subnet to the ``to`` subnet
  //   assert!(
  //     (to_delegate_balance >= Network::<T>::percent_mul(expected_balance_to_be_removed, 990000000)) &&
  //     (to_delegate_balance < expected_balance_to_be_removed)
  //   );
	// }

	// #[benchmark]
	// fn transfer_from_subnet_to_node() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let from_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();

	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_PATH_2.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let to_subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_PATH_2.into()).unwrap();
	// 	let to_subnet_node_id = end;

	// 	let delegate_account: T::AccountId = funded_account::<T>("delegate_account", 0);

  //   assert_ok!(
  //     Network::<T>::add_to_delegate_stake(
  //       RawOrigin::Signed(delegate_account.clone()).into(), 
	// 			from_subnet_id, 
	// 			DEFAULT_SUBNET_NODE_STAKE
  //     )
  //   );


	// 	let delegate_shares = AccountSubnetDelegateStakeShares::<T>::get(delegate_account.clone(), from_subnet_id);

	// 	let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(from_subnet_id);
  //   let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(from_subnet_id);

  //   let mut from_delegate_balance = Network::<T>::convert_to_balance(
  //     delegate_shares,
  //     total_subnet_delegated_stake_shares,
  //     total_subnet_delegated_stake_balance
  //   );

	// 	#[extrinsic_call]
	// 	transfer_from_subnet_to_node(
	// 		RawOrigin::Signed(delegate_account.clone()), 
	// 		from_subnet_id, 
	// 		to_subnet_id,
	// 		to_subnet_node_id, 
	// 		delegate_shares
	// 	);
		
  //   let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((delegate_account.clone(), to_subnet_id, to_subnet_node_id));
  //   let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(to_subnet_id, to_subnet_node_id);
  //   let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<T>::get(to_subnet_id, to_subnet_node_id);

  //   let account_node_delegate_stake_balance = Network::<T>::convert_to_balance(
  //     account_node_delegate_stake_shares,
  //     total_node_delegate_stake_shares,
  //     total_node_delegate_stake_balance
  //   );

  //   assert_ne!(account_node_delegate_stake_balance, 0);

  //   assert!(
  //     (account_node_delegate_stake_balance >= Network::<T>::percent_mul(from_delegate_balance, 990000000)) &&
  //     (account_node_delegate_stake_balance < from_delegate_balance)
  //   );
	// }

	// #[benchmark]
	// fn validate() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	let subnet = SubnetsData::<T>::get(subnet_id).unwrap();

	// 	let n_nodes: u32 = TotalSubnetNodes::<T>::get(subnet_id);

	// 	let epoch_length = T::EpochLength::get();

	// 	let current_block_number = get_current_block_as_u32::<T>();
	// 	let next_epoch_block = current_block_number - (current_block_number % epoch_length) + epoch_length;
	// 	frame_system::Pallet::<T>::set_block_number(u32_to_block::<T>(next_epoch_block));

	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

  //   Network::<T>::do_epoch_preliminaries(get_current_block_as_u32::<T>(), epoch as u32);

	// 	let validator_id = SubnetElectedValidator::<T>::get(subnet_id, epoch as u32);
  //   assert!(validator_id != None, "Validator is None");

	// 	let hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id.unwrap()).unwrap();

	// 	let subnet_node_data_vec = get_subnet_node_data(0, n_nodes);

	// 	#[extrinsic_call]
	// 	validate(RawOrigin::Signed(hotkey.clone()), subnet_id, subnet_node_data_vec.clone(), None);

	// 	let submission = SubnetConsensusSubmission::<T>::get(subnet_id, epoch as u32).unwrap();

  //   assert_eq!(submission.validator_id, validator_id.unwrap(), "Err: validator");
  //   assert_eq!(submission.data.len(), subnet_node_data_vec.clone().len(), "Err: data len");
  //   assert_eq!(submission.attests.len(), 1, "Err: attests"); // validator auto-attests
	// }

	// #[benchmark]
	// fn attest() {
	// 	let end = 12;
	// 	build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, end, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(DEFAULT_SUBNET_NAME.into()).unwrap();
	// 	let subnet = SubnetsData::<T>::get(subnet_id).unwrap();
	// 	let n_nodes: u32 = TotalSubnetNodes::<T>::get(subnet_id);

	// 	let epoch_length = T::EpochLength::get();

	// 	let current_block_number = get_current_block_as_u32::<T>();
	// 	let next_epoch_block = current_block_number - (current_block_number % epoch_length) + epoch_length;
	// 	frame_system::Pallet::<T>::set_block_number(u32_to_block::<T>(next_epoch_block));

	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

  //   Network::<T>::do_epoch_preliminaries(get_current_block_as_u32::<T>(), epoch as u32);

	// 	let validator_id = SubnetElectedValidator::<T>::get(subnet_id, epoch as u32);
  //   assert!(validator_id != None, "Validator is None");

	// 	let hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id.unwrap()).unwrap();

	// 	let subnet_node_data_vec = get_subnet_node_data(0, n_nodes);

	// 	assert_ok!(
	// 		Network::<T>::validate(
	// 			RawOrigin::Signed(hotkey.clone()).into(), 
	// 			subnet_id, 
	// 			subnet_node_data_vec.clone(),
	// 			None,
	// 		)
	// 	);
	
	// 	// Might be the same ID as validator_id
	// 	let attester = funded_account::<T>("subnet_node_account", 2);
  //   let attester_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, attester.clone()).unwrap();

	// 	let current_block_number = get_current_block_as_u32::<T>();

	// 	#[extrinsic_call]
	// 	attest(RawOrigin::Signed(attester.clone()), subnet_id);

	// 	let submission = SubnetConsensusSubmission::<T>::get(subnet_id, epoch as u32).unwrap();

	// 	// validator + attester
  //   assert_eq!(submission.attests.len(), 2 as usize);
  //   assert_eq!(submission.attests.get(&attester_subnet_node_id), Some(&current_block_number));
	// }

	// #[benchmark]
	// fn rewards_v2(x: Linear<1, 64>, p: Linear<1, 512>) {
	// 	/// x: subnets
	// 	/// p: nodes

	// 	let max_subnets: u32 = Network::<T>::max_subnets();
	// 	let n_nodes: u32 = Network::<T>::max_subnet_nodes();

	// 	// Activate subnets
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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

	// 		let validator_id: u32 = 1;
	// 		let validator_hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id).unwrap();

	// 		SubnetElectedValidator::<T>::insert(subnet_id, epoch, validator_id);
	// 		let subnet_node_data_vec = get_subnet_node_data(0, n_nodes);

	// 		assert_ok!(
	// 			Network::<T>::validate(
	// 				RawOrigin::Signed(validator_hotkey.clone()).into(), 
	// 				subnet_id, 
	// 				subnet_node_data_vec.clone(),
	// 				None,
	// 			)
	// 		);		
	// 	}

	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Attest
	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 1..p+1 {
	// 			if n == 1 {
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

	// 	let pre_total_issuance: u128 = Network::<T>::get_total_network_issuance();


	// 	#[block]
	// 	{
	// 		Network::<T>::reward_subnets_v2(0, 0);
	// 	}

	// 	let post_total_issuance: u128 = Network::<T>::get_total_network_issuance();

	// 	// assert!(post_total_issuance > pre_total_issuance);
	// 	assert!(true);
	// }

	// #[benchmark]
	// fn do_single_subnet_deactivation_ledger() {
	// 	let n_nodes: u32 = Network::<T>::max_subnet_nodes();

	// 	let path: Vec<u8> = DEFAULT_SUBNET_NAME.into();
	// 	build_activated_subnet::<T>(path.clone(), 0, n_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path.clone()).unwrap();

	// 	let epoch_length = T::EpochLength::get();
	// 	let block_number = get_current_block_as_u32::<T>();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Insert validator and validate
	// 	let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", 0);
	// 	SubnetElectedValidator::<T>::insert(subnet_id, epoch as u32, subnet_node_account.clone());
	// 	let subnet_node_data_vec = get_subnet_node_data(0, n_nodes);
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
	// 		assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Deactivated);		
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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", 0);
	// 		SubnetElectedValidator::<T>::insert(subnet_id, epoch as u32, subnet_node_account.clone());
	// 		let subnet_node_data_vec = get_subnet_node_data(0, n_nodes);
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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

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
	// 	// let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();
	// 	let mut i = 0;

	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..n_nodes {
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 			let subnet_node = SubnetNodesData::<T>::get(subnet_id, subnet_node_account.clone());
	// 			// assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Deactivated);		
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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", 0);
	// 		SubnetElectedValidator::<T>::insert(subnet_id, epoch as u32, subnet_node_account.clone());
	// 		let subnet_node_data_vec = get_subnet_node_data(0, n_nodes);
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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

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
	// 	// let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();
	// 	let mut i = 0;

	// 	for s in 0..x {
	// 		let path: Vec<u8> = format!("subnet-name-{s}").into(); 
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..p {
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 			let subnet_node = SubnetNodesData::<T>::get(subnet_id, subnet_node_account.clone());
	// 			// assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Deactivated);		
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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

	// 		let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", 0);
	// 		SubnetElectedValidator::<T>::insert(subnet_id, epoch as u32, subnet_node_account.clone());
	// 		let subnet_node_data_vec = get_subnet_node_data(0, n_nodes);
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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

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
	// 		let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path).unwrap();

	// 		for n in 0..n_nodes {
	// 			let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", n);
	// 			let subnet_node = SubnetNodesData::<T>::get(subnet_id, subnet_node_account.clone());
	// 			assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Deactivated);		
	// 		}
	// 	}
	// }

	// #[benchmark]
	// fn do_single_subnet_deactivation_ledger() {
	// 	let n_nodes: u32 = Network::<T>::max_subnet_nodes();

	// 	let path: Vec<u8> = DEFAULT_SUBNET_NAME.into();
	// 	build_activated_subnet::<T>(path.clone(), 0, n_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
	// 	let subnet_id = SubnetName::<T>::get::<Vec<u8>>(path.clone()).unwrap();

	// 	let epoch_length = T::EpochLength::get();
	// 	let block_number = get_current_block_as_u32::<T>();
	// 	let epoch = get_current_block_as_u32::<T>() / epoch_length as u32;

	// 	// Insert validator and validate
	// 	let subnet_node_account: T::AccountId = get_account::<T>("subnet_node_account", 0);
	// 	SubnetElectedValidator::<T>::insert(subnet_id, epoch as u32, subnet_node_account.clone());
	// 	let subnet_node_data_vec = get_subnet_node_data(0, n_nodes);
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
	// 		assert_eq!(subnet_node.classification.node_class, SubnetNodeClass::Deactivated);		
	// 	}
	// }

	// #[benchmark]
	// fn on_initialize_do_choose_validator_and_accountants() {
	// 	let max_subnets: u32 = Network::<T>::max_subnets();
	// 	let n_nodes: u32 = Network::<T>::max_subnet_nodes();

	// 	for s in 0..max_subnets {
	// 		build_activated_subnet::<T>(DEFAULT_SUBNET_NAME.into(), 0, n_nodes, DEFAULT_DEPOSIT_AMOUNT, DEFAULT_SUBNET_NODE_STAKE);
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
	// 		);
	// 	}

	// 	// ensure nodes were rewarded
	// 	for s in 0..max_subnets {
	// 		let subnet_id = s+1;

	// 		let validator = SubnetElectedValidator::<T>::get(subnet_id, epoch as u32);
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
