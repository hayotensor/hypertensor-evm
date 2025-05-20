use precompile_utils::{EvmResult, prelude::*};
use pallet_evm::{AddressMapping, ExitError, PrecompileFailure, PrecompileHandle};
use sp_core::{H256, U256, H160, OpaquePeerId};
use sp_runtime::{Vec, traits::{Dispatchable, StaticLookup, UniqueSaturatedInto}};
use sp_std::vec;
// use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_system::RawOrigin;
use frame_support::{
  dispatch::{GetDispatchInfo, PostDispatchInfo},
	storage::bounded_vec::BoundedVec,
};
use pallet_network::RegistrationSubnetData;
use frame_support::traits::ConstU32;
use sp_std::collections::btree_set::BTreeSet;

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> =
	<Runtime as pallet_balances::Config<Instance>>::Balance;

pub(crate) struct SubnetPrecompile<R>(PhantomData<R>);

impl<R> SubnetPrecompile<R> 
where
  // R: frame_system::Config
  //   + pallet_evm::Config
  //   + pallet_network::Config
  //   + pallet_balances::Config,
  // <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
  //   + From<pallet_balances::Call<R>>
  //   + GetDispatchInfo
  //   + Dispatchable<PostInfo = PostDispatchInfo>,
  // <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
  // <R as pallet_balances::Config>::Balance: TryFrom<U256>,
  // <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
  R: frame_system::Config
      + pallet_evm::Config
      + pallet_network::Config,
  <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
      + GetDispatchInfo
      + Dispatchable<PostInfo = PostDispatchInfo>,
  <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
  <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
  pub const HASH_N: u64 = 2049;
}

#[precompile_utils::precompile]
impl<R> SubnetPrecompile<R> 
where 
  // R: frame_system::Config
  //   + pallet_evm::Config
  //   + pallet_network::Config
  //   + pallet_balances::Config,
  // <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
  //   + From<pallet_balances::Call<R>>
  //   + GetDispatchInfo
  //   + Dispatchable<PostInfo = PostDispatchInfo>,
  // <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
  // <R as pallet_balances::Config>::Balance: TryFrom<U256>,
  // <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
  R: frame_system::Config
      + pallet_evm::Config
      + pallet_network::Config,
  <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
      + GetDispatchInfo
      + Dispatchable<PostInfo = PostDispatchInfo>,
  <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
  <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
  // #[precompile::public("registerSubnet(RegistrationSubnetData)")]
  // #[precompile::payable]
  // fn register_subnet(
  //   handle: &mut impl PrecompileHandle,
  //   subnet_data: RegistrationSubnetData,
  // ) -> EvmResult<()> {
  //   let origin = R::AddressMapping::into_account_id(handle.context().caller);
  //   let call = pallet_network::Call::<R>::register_subnet {
  //     subnet_data
  //   };

  //   RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

  //   Ok(())
  // }

  #[precompile::public("registerSubnet(string,uint256,uint256,uint256,uint256,uint256,bytes32[])")]
  #[precompile::payable]
  fn register_subnet(
    handle: &mut impl PrecompileHandle,
    name: BoundedString<ConstU32<256>>,
    max_node_registration_epochs: U256,
    node_registration_interval: U256,
    node_activation_interval: U256,
    node_queue_period: U256,
    max_node_penalties: U256,
    coldkeys: Vec<H256>,
  ) -> EvmResult<()> {
    let max_node_registration_epochs = try_u256_to_u32(max_node_registration_epochs)?;
    let node_registration_interval = try_u256_to_u32(node_registration_interval)?;
    let node_activation_interval = try_u256_to_u32(node_activation_interval)?;
    let node_queue_period = try_u256_to_u32(node_queue_period)?;
    let max_node_penalties = try_u256_to_u32(max_node_penalties)?;
    let coldkey_whitelist: BTreeSet<R::AccountId> = coldkeys
      .into_iter()
      .map(|a| R::AddressMapping::into_account_id(a.into()))
      .collect();

    let subnet_data = pallet_network::RegistrationSubnetData::<R::AccountId> {
      name: name.into(),
      repo: Vec::new(),
			description: Vec::new(),
			misc: Vec::new(),
      max_node_registration_epochs,
      node_registration_interval,
      node_activation_interval,
      node_queue_period,
      max_node_penalties,
      coldkey_whitelist,
    };

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::register_subnet {
      subnet_data
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("activateSubnet(uint256)")]
  #[precompile::payable]
  fn activate_subnet(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::activate_subnet {
      subnet_id
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("removeSubnet(uint256)")]
  #[precompile::payable]
  fn remove_subnet(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::remove_subnet {
      subnet_id
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("ownerDeactivateSubnet(uint256,string)")]
  #[precompile::payable]
  fn owner_deactivate_subnet(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    name: BoundedString<ConstU32<256>>,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::owner_deactivate_subnet {
      subnet_id,
      name: name.into()
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("ownerUpdateRegistrationInterval(uint256,uint256)")]
  #[precompile::payable]
  fn owner_update_registration_interval(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    value: U256,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let value = try_u256_to_u32(value)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::owner_update_registration_interval {
      subnet_id,
      value
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("ownerRemoveSubnetNode(uint256,uint256)")]
  #[precompile::payable]
  fn owner_remove_subnet_node(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    subnet_node_id: U256,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::owner_remove_subnet_node {
      subnet_id,
      subnet_node_id
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("getSubnetId(string)")]
	#[precompile::view]
	fn get_subnet_id(
    handle: &mut impl PrecompileHandle,
    name: BoundedString<ConstU32<256>>,
  ) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let subnetId = match pallet_network::SubnetPaths::<R>::try_get::<Vec<u8>>(name.into()) {
      Ok(subnet_id) => subnet_id,
      Err(()) => 0,
    };

		Ok(subnetId)
	}


  #[precompile::public("addSubnetNode(uint256,bytes32,bytes32,bytes32,uint256,uint256)")]
  #[precompile::payable]
  fn add_subnet_node(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    hotkey: H256,
    peer_id: H256, 
    bootstrap_peer_id: H256,
    delegate_reward_rate: U256,
    stake_to_be_added: U256,
    // a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>,
    // b: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
    // c: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let hotkey = R::AddressMapping::into_account_id(hotkey.into());
    let peer_id = OpaquePeerId(peer_id.as_bytes().to_vec());
    let bootstrap_peer_id = OpaquePeerId(bootstrap_peer_id.as_bytes().to_vec());
    let delegate_reward_rate: u128 = delegate_reward_rate.unique_saturated_into();
    let stake_to_be_added: u128 = stake_to_be_added.unique_saturated_into();

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    // let call = pallet_network::Call::<R>::add_to_delegate_stake {
    //   subnet_id,
    //   stake_to_be_added: delegate_reward_rate,
    // };

    // let call = pallet_network::Call::<R>::add_subnet_node {
    //   subnet_id,
    //   hotkey,
    //   peer_id,
    //   bootstrap_peer_id,
    //   delegate_reward_rate,
    //   stake_to_be_added,
    //   a,
    //   b,
    //   c,
    // };

    // RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("registerSubnetNode(uint256,bytes32,bytes32,bytes32,uint256,uint256)")]
  #[precompile::payable]
  fn register_subnet_node(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    hotkey: H256,
    peer_id: H256, 
    bootstrap_peer_id: H256,
    delegate_reward_rate: U256,
    stake_to_be_added: U256,
    // a: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
    // b: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
    // c: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let hotkey = R::AddressMapping::into_account_id(hotkey.into());
    let peer_id = OpaquePeerId(peer_id.as_bytes().to_vec());
    let bootstrap_peer_id = OpaquePeerId(bootstrap_peer_id.as_bytes().to_vec());
    let delegate_reward_rate: u128 = delegate_reward_rate.unique_saturated_into();
    let stake_to_be_added: u128 = stake_to_be_added.unique_saturated_into();

    let origin = R::AddressMapping::into_account_id(handle.context().caller);

    // let call = pallet_network::Call::<R>::register_subnet_node {
    //   subnet_id,
    //   hotkey,
    //   peer_id,
    //   bootstrap_peer_id,
    //   delegate_reward_rate,
    //   stake_to_be_added,
    //   a,
    //   b,
    //   c,
    // };

    // RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("activateSubnetNode(uint256,uint256)")]
  #[precompile::payable]
  fn activate_subnet_node(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    subnet_node_id: U256,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::activate_subnet_node {
      subnet_id,
      subnet_node_id,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("deactivateSubnetNode(uint256,uint256)")]
  #[precompile::payable]
  fn deactivate_subnet_node(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    subnet_node_id: U256,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::deactivate_subnet_node {
      subnet_id,
      subnet_node_id,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("removeSubnetNode(uint256,uint256)")]
  #[precompile::payable]
  fn remove_subnet_node(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    subnet_node_id: U256,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::remove_subnet_node {
      subnet_id,
      subnet_node_id,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("updateDelegateRewardRate(uint256,uint256,uint256)")]
  #[precompile::payable]
  fn update_delegate_reward_rate(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    subnet_node_id: U256,
    new_delegate_reward_rate: U256,
  ) -> EvmResult<()> {
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
    let new_delegate_reward_rate = new_delegate_reward_rate.unique_saturated_into();

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::update_delegate_reward_rate {
      subnet_id,
      subnet_node_id,
      new_delegate_reward_rate,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }
}

fn try_u256_to_u32(value: U256) -> Result<u32, PrecompileFailure> {
  value.try_into().map_err(|_| PrecompileFailure::Error {
    exit_status: ExitError::Other("u32 out of bounds".into()),
  })
}