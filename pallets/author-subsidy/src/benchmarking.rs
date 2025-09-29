//! Benchmarking setup for pallet-authory-subsidy
// frame-omni-bencher v1 benchmark pallet --runtime target/release/wbuild/solochain-template-runtime/solochain_template_runtime.compact.compressed.wasm --extrinsic "" --pallet "pallet_author_subsidy" --output pallets/author-subsidy/src/weights.rs --template ./.maintain/frame-weight-template.hbs

// frame-omni-bencher v1 benchmark pallet --runtime target/release/wbuild/solochain-template-runtime/solochain_template_runtime.compact.compressed.wasm --extrinsic "" --pallet "pallet_author_subsidy"

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

const SEED: u32 = 0;

pub type BalanceOf<T> = <T as Config>::Currency;

fn get_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let caller: T::AccountId = account(name, index, SEED);
    caller
}

pub fn u128_to_balance<T: frame_system::Config + pallet::Config>(
    input: u128,
) -> Option<
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
> {
    input.try_into().ok()
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn on_initialize() {
        #[block]
        {
            let digest = frame_system::Pallet::<T>::digest();
            let pre_runtime_digests = digest.logs.iter().filter_map(|d| d.as_pre_runtime());
            let author = T::FindAuthor::find_author(pre_runtime_digests).unwrap_or_default();
            let account_id = T::AddressMapping::into_account_id(author);

            let block_reward_as_u128 = T::AuthorBlockEmissions::get();
            let block_reward = u128_to_balance::<T>(block_reward_as_u128);

            T::Currency::deposit_creating(&account_id, block_reward.unwrap());
        }
    }

    impl_benchmark_test_suite!(
        AuthorSubsidy,
        tests::mock::new_test_ext(),
        tests::mock::Test
    );
}
