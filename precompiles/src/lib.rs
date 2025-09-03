#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use pallet_evm::{
    AddressMapping, IsPrecompileResult, Precompile, PrecompileHandle, PrecompileResult,
    PrecompileSet,
};
// use sp_core::H160;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use sp_core::{H160, U256, crypto::ByteArray};
use sp_runtime::traits::{Dispatchable, StaticLookup};

use crate::balance::*;
use crate::staking::*;
use crate::subnet::*;
use crate::template::*;

mod balance;
mod staking;
mod subnet;
mod template;

pub struct FrontierPrecompiles<R>(PhantomData<R>);

// impl<R> FrontierPrecompiles<R>
// where
//   R: frame_system::Config
//     + pallet_evm::Config
//     + pallet_balances::Config
//     + pallet_network::Config,
//   // R::AccountId: From<[u8; 32]> + ByteArray + Into<[u8; 32]>,
//   <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
//     + From<pallet_balances::Call<R>>
//     + GetDispatchInfo
//     + Dispatchable<PostInfo = PostDispatchInfo>,
//   <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
//   <R as pallet_balances::Config>::Balance: TryFrom<U256>,
//   <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
// {
// 	pub fn new() -> Self {
// 		Self(Default::default())
// 	}
// 	pub fn used_addresses() -> [H160; 9] {
// 		[
// 			hash(1),
// 			hash(2),
// 			hash(3),
// 			hash(4),
// 			hash(5),
// 			hash(1024),
// 			hash(1025),
// 			hash(StakingPrecompile::<R>::HASH_N),
// 			hash(SubnetPrecompile::<R>::HASH_N),
// 		]
// 	}
// }
// impl<R> PrecompileSet for FrontierPrecompiles<R>
// where
//   R: frame_system::Config
//     + pallet_evm::Config
//     + pallet_balances::Config
//     + pallet_network::Config,
//   <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
//     + From<pallet_balances::Call<R>>
//     + GetDispatchInfo
//     + Dispatchable<PostInfo = PostDispatchInfo>,
//   <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
//   <R as pallet_balances::Config>::Balance: TryFrom<U256>,
//   <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
// {
// 	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
// 		match handle.code_address() {
// 			// Ethereum precompiles :
// 			a if a == hash(1) => Some(ECRecover::execute(handle)),
// 			a if a == hash(2) => Some(Sha256::execute(handle)),
// 			a if a == hash(3) => Some(Ripemd160::execute(handle)),
// 			a if a == hash(4) => Some(Identity::execute(handle)),
// 			a if a == hash(5) => Some(Modexp::execute(handle)),
// 			// Non-Frontier specific nor Ethereum precompiles :
// 			a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
// 			a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
// 			// Hypertensor
// 			a if a == hash(2048) => Some(StakingPrecompile::<R>::execute(handle)),
// 			a if a == hash(2049) => Some(SubnetPrecompile::<R>::execute(handle)),
// 			_ => None,
// 		}
// 	}

// 	fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
// 		IsPrecompileResult::Answer {
// 			is_precompile: Self::used_addresses().contains(&address),
// 			extra_cost: 0,
// 		}
// 	}
// }

impl<R> Default for FrontierPrecompiles<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_balances::Config + pallet_network::Config,
    R::AccountId: From<[u8; 20]> + Into<[u8; 20]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
        + From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R> FrontierPrecompiles<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_balances::Config + pallet_network::Config,
    R::AccountId: From<[u8; 20]> + Into<[u8; 20]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
        + From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    pub fn new() -> Self {
        Self(Default::default())
    }
    pub fn used_addresses() -> [H160; 9] {
        [
            hash(1),
            hash(2),
            hash(3),
            hash(4),
            hash(5),
            hash(1024),
            hash(1025),
            hash(StakingPrecompile::<R>::HASH_N),
            hash(SubnetPrecompile::<R>::HASH_N),
            // hash(ERC20BalancePrecompile::<R>::HASH_N),
            // hash(TemplatePrecompile::<R>::HASH_N),
        ]
    }
}
impl<R> PrecompileSet for FrontierPrecompiles<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_balances::Config + pallet_network::Config,
    R::AccountId: From<[u8; 20]> + Into<[u8; 20]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
        + From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
        match handle.code_address() {
            // Ethereum precompiles :
            a if a == hash(1) => Some(ECRecover::execute(handle)),
            a if a == hash(2) => Some(Sha256::execute(handle)),
            a if a == hash(3) => Some(Ripemd160::execute(handle)),
            a if a == hash(4) => Some(Identity::execute(handle)),
            a if a == hash(5) => Some(Modexp::execute(handle)),
            // Non-Frontier specific nor Ethereum precompiles :
            a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
            a if a == hash(1025) => Some(ECRecoverPublicKey::execute(handle)),
            // Hypertensor
            a if a == hash(StakingPrecompile::<R>::HASH_N) => {
                Some(StakingPrecompile::<R>::execute(handle))
            }
            a if a == hash(SubnetPrecompile::<R>::HASH_N) => {
                Some(SubnetPrecompile::<R>::execute(handle))
            }
            // a if a == hash(ERC20BalancePrecompile::<R>::HASH_N) => Some(ERC20BalancePrecompile::<R>::execute(handle)),
            // a if a == hash(TemplatePrecompile::<R>::HASH_N) => Some(TemplatePrecompile::<R>::execute(handle)),
            _ => None,
        }
    }

    fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
        IsPrecompileResult::Answer {
            is_precompile: Self::used_addresses().contains(&address),
            extra_cost: 0,
        }
    }
}

fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}
