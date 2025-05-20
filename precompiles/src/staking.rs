use precompile_utils::{EvmResult, prelude::*};
use pallet_evm::{AddressMapping, ExitError, PrecompileFailure, PrecompileHandle};
use sp_core::{H256, U256, H160};
use sp_runtime::traits::{Dispatchable, StaticLookup, UniqueSaturatedInto};
use sp_std::vec;
use core::marker::PhantomData;
use frame_system::RawOrigin;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use sp_core::Decode;
use fp_evm::Log;

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> =
	<Runtime as pallet_balances::Config<Instance>>::Balance;

pub(crate) struct StakingPrecompile<R>(PhantomData<R>);

impl<R> StakingPrecompile<R> 
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
  // R::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
  <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
      + GetDispatchInfo
      + Dispatchable<PostInfo = PostDispatchInfo>,
  <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
  <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
  pub const HASH_N: u64 = 2048;
}

#[precompile_utils::precompile]
impl<R> StakingPrecompile<R> 
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
  // R::AccountId: From<[u8; 32]> + Into<[u8; 32]>,
  <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
      + GetDispatchInfo
      + Dispatchable<PostInfo = PostDispatchInfo>,
  <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
  <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
  #[precompile::public("addToStake(uint256,uint256,bytes32,uint256)")]
  #[precompile::payable]
  fn add_to_stake(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    subnet_node_id: U256,
    hotkey: H256,
    stake_to_be_added: U256,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
    let stake_to_be_added = stake_to_be_added.unique_saturated_into();
    let hotkey = R::AddressMapping::into_account_id(hotkey.into());
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::add_to_stake {
      subnet_id,
      subnet_node_id,
      hotkey,
      stake_to_be_added,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;
    // RuntimeHelper::<R>::try_dispatch(handle, Some(origin.clone()).into(), call, 0)?;

    Ok(())
  }

  #[precompile::public("removeStake(uint256,bytes32,uint256)")]
  #[precompile::payable]
  fn remove_stake(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    hotkey: H256,
    stake_to_be_removed: U256,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let stake_to_be_removed = stake_to_be_removed.unique_saturated_into();
    let hotkey = R::AddressMapping::into_account_id(hotkey.into());
    let subnet_id = try_u256_to_u32(subnet_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::remove_stake {
      subnet_id,
      hotkey,
      stake_to_be_removed,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("claimUnbondings()")]
  #[precompile::payable]
  fn claim_unbondings(
    handle: &mut impl PrecompileHandle,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::claim_unbondings {};

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("addToDelegateStake(uint256,uint256)")]
  #[precompile::payable]
  fn add_to_delegate_stake(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    stake_to_be_added: U256,
  ) -> EvmResult<()> {
    // handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let subnet_id = try_u256_to_u32(subnet_id)?;
    let stake_to_be_added = stake_to_be_added.unique_saturated_into();

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::add_to_delegate_stake {
      subnet_id,
      stake_to_be_added,
    };

    let post_dispatch_info = RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 0)?;

    log::trace!(
      target: "precompile", 
      "subnet_id {:?}, stake_to_be_added {:?}", 
      subnet_id, 
      stake_to_be_added
    );

    Ok(())
  }

  #[precompile::public("swapDelegateStake(uint256,uint256,uint256)")]
  #[precompile::payable]
  fn swap_delegate_stake(
    handle: &mut impl PrecompileHandle,
    from_subnet_id: U256,
    to_subnet_id: U256,
    delegate_stake_shares_to_swap: U256,
  ) -> EvmResult<()> {
    let delegate_stake_shares_to_swap = delegate_stake_shares_to_swap.unique_saturated_into();
    let from_subnet_id = try_u256_to_u32(from_subnet_id)?;
    let to_subnet_id = try_u256_to_u32(to_subnet_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::swap_delegate_stake {
      from_subnet_id,
      to_subnet_id,
      delegate_stake_shares_to_swap,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("removeDelegateStake(uint256,uint256)")]
  #[precompile::payable]
  fn remove_delegate_stake(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    shares_to_be_removed: U256,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
    let shares_to_be_removed = shares_to_be_removed.unique_saturated_into();
    let subnet_id = try_u256_to_u32(subnet_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::remove_delegate_stake {
      subnet_id,
      shares_to_be_removed,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("increaseDelegateStake(uint256,uint256)")]
  #[precompile::payable]
  fn increase_delegate_stake(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    amount: U256,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
    let amount = amount.unique_saturated_into();
    let subnet_id = try_u256_to_u32(subnet_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::increase_delegate_stake {
      subnet_id,
      amount,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("addToNodeDelegateStake(uint256,uint256,uint256)")]
  #[precompile::payable]
  fn add_to_node_delegate_stake(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    subnet_node_id: U256,
    node_delegate_stake_to_be_added: U256,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
    let node_delegate_stake_to_be_added = node_delegate_stake_to_be_added.unique_saturated_into();
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::add_to_node_delegate_stake {
      subnet_id,
      subnet_node_id,
      node_delegate_stake_to_be_added,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 0)?;

    Ok(())
  }

  #[precompile::public("transferNodeDelegateStake(uint256,uint256,uint256,uint256,uint256)")]
  #[precompile::payable]
  fn swap_node_delegate_stake(
    handle: &mut impl PrecompileHandle,
    from_subnet_id: U256,
    from_subnet_node_id: U256,
    to_subnet_id: U256,
    to_subnet_node_id: U256,
    node_delegate_stake_shares_to_swap: U256,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
    let node_delegate_stake_shares_to_swap = node_delegate_stake_shares_to_swap.unique_saturated_into();
    let from_subnet_id = try_u256_to_u32(from_subnet_id)?;
    let from_subnet_node_id = try_u256_to_u32(from_subnet_node_id)?;
    let to_subnet_id = try_u256_to_u32(to_subnet_id)?;
    let to_subnet_node_id = try_u256_to_u32(to_subnet_node_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::swap_node_delegate_stake {
      from_subnet_id,
      from_subnet_node_id,
      to_subnet_id,
      to_subnet_node_id,
      node_delegate_stake_shares_to_swap,    
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("removeNodeDelegateStake(uint256,uint256,uint256)")]
  #[precompile::payable]
  fn remove_node_delegate_stake(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    subnet_node_id: U256,
    node_delegate_stake_shares_to_be_removed: U256,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let node_delegate_stake_shares_to_be_removed = node_delegate_stake_shares_to_be_removed.unique_saturated_into();
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::remove_node_delegate_stake {
      subnet_id,
      subnet_node_id,
      node_delegate_stake_shares_to_be_removed,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("increaseNodeDelegateStake(uint256,uint256,uint256)")]
  #[precompile::payable]
  fn increase_node_delegate_stake(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
    subnet_node_id: U256,
    amount: U256,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let amount = amount.unique_saturated_into();
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::increase_node_delegate_stake {
      subnet_id,
      subnet_node_id,
      amount,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("transferFromNodeToSubnet(uint256,uint256,uint256,uint256)")]
  #[precompile::payable]
  fn transfer_from_node_to_subnet(
    handle: &mut impl PrecompileHandle,
    from_subnet_id: U256,
    from_subnet_node_id: U256,
    to_subnet_id: U256,
    node_delegate_stake_shares_to_swap: U256,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
    let node_delegate_stake_shares_to_swap = node_delegate_stake_shares_to_swap.unique_saturated_into();
    let from_subnet_id = try_u256_to_u32(from_subnet_id)?;
    let from_subnet_node_id = try_u256_to_u32(from_subnet_node_id)?;
    let to_subnet_id = try_u256_to_u32(to_subnet_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::transfer_from_node_to_subnet {
      from_subnet_id,
      from_subnet_node_id,
      to_subnet_id,
      node_delegate_stake_shares_to_swap,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("transferFromSubnetToNode(uint256,uint256,uint256,uint256)")]
  #[precompile::payable]
  fn transfer_from_subnet_to_node(
    handle: &mut impl PrecompileHandle,
    from_subnet_id: U256,
    to_subnet_id: U256,
    to_subnet_node_id: U256,
    delegate_stake_shares_to_swap: U256,
  ) -> EvmResult<()> {
    handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
    let delegate_stake_shares_to_swap = delegate_stake_shares_to_swap.unique_saturated_into();
    let from_subnet_id = try_u256_to_u32(from_subnet_id)?;
    let to_subnet_id = try_u256_to_u32(to_subnet_id)?;
    let to_subnet_node_id = try_u256_to_u32(to_subnet_node_id)?;

    let origin = R::AddressMapping::into_account_id(handle.context().caller);
    let call = pallet_network::Call::<R>::transfer_from_subnet_to_node {
      from_subnet_id,
      to_subnet_id,
      to_subnet_node_id,
      delegate_stake_shares_to_swap,
    };

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }

  #[precompile::public("totalSubnetStake(uint256)")]
	#[precompile::view]
	fn total_subnet_stake(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
  ) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let subnet_id = try_u256_to_u32(subnet_id)?;
    let total_subnet_stake: u128 = pallet_network::TotalSubnetStake::<R>::get(subnet_id);

		Ok(total_subnet_stake)
	}

  #[precompile::public("accountSubnetStake(bytes32,uint256)")]
	#[precompile::view]
	fn account_subnet_stake(
    handle: &mut impl PrecompileHandle,
    hotkey: H256,
    subnet_id: U256,
  ) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let hotkey = R::AddressMapping::into_account_id(hotkey.into());
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let account_subnet_stake: u128 = pallet_network::AccountSubnetStake::<R>::get(&hotkey, subnet_id);

		Ok(account_subnet_stake)
	}

  #[precompile::public("totalSubnetDelegateStakeBalance(uint256)")]
	#[precompile::view]
	fn total_subnet_delegate_stake_balance(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
  ) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let subnet_id = try_u256_to_u32(subnet_id)?;
    let total_subnet_delegate_stake_balance: u128 = pallet_network::TotalSubnetDelegateStakeBalance::<R>::get(subnet_id);

		Ok(total_subnet_delegate_stake_balance)
	}

  #[precompile::public("totalSubnetDelegateStakeShares(uint256)")]
	#[precompile::view]
	fn total_subnet_delegate_stake_shares(
    handle: &mut impl PrecompileHandle,
    subnet_id: U256,
  ) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let subnet_id = try_u256_to_u32(subnet_id)?;
    let total_subnet_delegate_stake_shares: u128 = pallet_network::TotalSubnetDelegateStakeBalance::<R>::get(subnet_id);

		Ok(total_subnet_delegate_stake_shares)
	}

  #[precompile::public("accountSubnetDelegateStakeShares(bytes32,uint256)")]
	#[precompile::view]
	fn account_subnet_delegate_stake_shares(
    handle: &mut impl PrecompileHandle,
    hotkey: H256,
    subnet_id: U256,
  ) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let hotkey = R::AddressMapping::into_account_id(hotkey.into());
    let subnet_id = try_u256_to_u32(subnet_id)?;
    let account_subnet_delegate_stake_shares: u128 = pallet_network::AccountSubnetDelegateStakeShares::<R>::get(&hotkey, subnet_id);

		Ok(account_subnet_delegate_stake_shares)
	}

  #[precompile::public("accountSubnetDelegateStakeBalance(bytes32,uint256)")]
	#[precompile::view]
	fn account_subnet_delegate_stake_balance(
    handle: &mut impl PrecompileHandle,
    hotkey: H256,
    subnet_id: U256,
  ) -> EvmResult<u128> {
		handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    let hotkey = R::AddressMapping::into_account_id(hotkey.into());
    let subnet_id = try_u256_to_u32(subnet_id)?;

    let account_delegate_stake_shares = pallet_network::AccountSubnetDelegateStakeShares::<R>::get(&hotkey, subnet_id);
    if account_delegate_stake_shares == 0 {
      return Ok(0)
    }
    let total_subnet_delegated_stake_shares = pallet_network::TotalSubnetDelegateStakeShares::<R>::get(subnet_id);
    let total_subnet_delegated_stake_balance = pallet_network::TotalSubnetDelegateStakeBalance::<R>::get(subnet_id);

    let shares = U256::from(account_delegate_stake_shares);
    let total_balance = U256::from(total_subnet_delegated_stake_balance) + U256::from(1);
    let total_shares = U256::from(total_subnet_delegated_stake_shares) + U256::from(10_u128.pow(1));
  
    let balance = shares * total_balance / total_shares;
    
    Ok(balance.try_into().unwrap_or(u128::MAX))
	}
}

fn try_u256_to_u32(value: U256) -> Result<u32, PrecompileFailure> {
  value.try_into().map_err(|_| PrecompileFailure::Error {
    exit_status: ExitError::Other("u32 out of bounds".into()),
  })
}