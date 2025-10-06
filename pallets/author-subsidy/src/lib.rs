// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

use frame_support::{
    sp_runtime::SaturatedConversion,
    traits::{Currency, FindAuthor, Get},
    weights::Weight,
};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;
use pallet_evm::AddressMapping;
use sp_core::H160;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
    // Import various useful types required by all FRAME pallets.
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
    // (`Call`s) in this pallet.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The pallet's configuration trait.
    ///
    /// All our types and constants a pallet depends on must be declared here.
    /// These types are defined generically and made concrete when the pallet is declared in the
    /// `runtime/src/lib.rs` file of your chain.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        type FindAuthor: FindAuthor<H160>;

        type AddressMapping: AddressMapping<Self::AccountId>;

        #[pallet::constant]
        type AuthorBlockEmissions: Get<u128>;

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;
    }

    /// Events that functions in this pallet can emit.
    ///
    /// Events are a simple means of indicating to the outside world (such as dApps, chain explorers
    /// or other users) that some notable update in the runtime has occurred. In a FRAME pallet, the
    /// documentation for each event field and its parameters is added to a node's metadata so it
    /// can be used by external interfaces or tools.
    ///
    ///	The `generate_deposit` macro generates a function on `Pallet` called `deposit_event` which
    /// will convert the event type of your pallet into `RuntimeEvent` (declared in the pallet's
    /// [`Config`] trait) and deposit it using [`frame_system::Pallet::deposit_event`].
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AuthorSubsidy { who: T::AccountId, subsidy: u128 },
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
            let digest = frame_system::Pallet::<T>::digest();
            let pre_runtime_digests = digest.logs.iter().filter_map(|d| d.as_pre_runtime());
            let author = T::FindAuthor::find_author(pre_runtime_digests).unwrap_or_default();
            let account_id = T::AddressMapping::into_account_id(author);

            let block_reward_as_u128 = T::AuthorBlockEmissions::get();
            let block_reward = block_reward_as_u128.saturated_into::<BalanceOf<T>>();

            T::Currency::deposit_creating(&account_id, block_reward);

            Self::deposit_event(Event::AuthorSubsidy {
                who: account_id,
                subsidy: block_reward_as_u128,
            });

            Weight::from_parts(0, 0)
            // T::WeightInfo::on_initialize()
        }
    }
}
