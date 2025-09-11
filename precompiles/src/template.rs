use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::RawOrigin;
use pallet_evm::AddressMapping;
use pallet_evm::{ExitError, PrecompileFailure, PrecompileHandle};
use precompile_utils::EvmResult;
use precompile_utils::prelude::RuntimeHelper;
use sp_core::U256;
use sp_runtime::traits::{Dispatchable, StaticLookup, UniqueSaturatedInto};

pub(crate) struct TemplatePrecompile<R>(PhantomData<R>);

impl<R> TemplatePrecompile<R>
where
    R: frame_system::Config + pallet_template::Config + pallet_evm::Config,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_template::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    pub const HASH_N: u64 = 10000;
}

#[precompile_utils::precompile]
impl<R> TemplatePrecompile<R>
where
    R: frame_system::Config + pallet_template::Config + pallet_evm::Config,
    <R as frame_system::Config>::RuntimeCall:
        GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_template::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    #[precompile::public("doSomething(uint256)")]
    #[precompile::payable]
    fn do_something(handle: &mut impl PrecompileHandle, something: U256) -> EvmResult<()> {
        let something = try_u256_to_u32(something)?;

        let call = pallet_template::Call::<R>::do_something { something };

        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }
}

fn try_u256_to_u32(value: U256) -> Result<u32, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("u32 out of bounds".into()),
    })
}
