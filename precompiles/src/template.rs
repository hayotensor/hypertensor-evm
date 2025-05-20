use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::RawOrigin;
use pallet_evm::PrecompileHandle;
use precompile_utils::EvmResult;
use sp_core::{H256, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup, UniqueSaturatedInto};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::RuntimeHelper;

pub(crate) struct TemplatePrecompile<R>(PhantomData<R>);


impl<R> TemplatePrecompile<R> 
where
    R: frame_system::Config + pallet_balances::Config + pallet_evm::Config,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
{
  pub const HASH_N: u64 = 10000;
}

#[precompile_utils::precompile]
impl<R> TemplatePrecompile<R>
where
    R: frame_system::Config + pallet_balances::Config + pallet_evm::Config,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
{
  #[precompile::public("doSomething(uint256)")]
  #[precompile::payable]
  fn do_something(handle: &mut impl PrecompileHandle, something: U256) -> EvmResult<()> {
    let amount = handle.context().apparent_value;

    if amount.is_zero() {
        return Ok(());
    }

    let dest = R::AddressMapping::into_account_id(address.into());

    let call = pallet_balances::Call::<R>::transfer_allow_death {
      dest: dest.into(),
      value: amount.unique_saturated_into(),
    };

    let origin = R::AddressMapping::into_account_id(handle.context().caller);

    RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    Ok(())
  }
}