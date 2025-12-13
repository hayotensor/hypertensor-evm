//! # Template Pallet
//!
//! A pallet with minimal functionality to help developers understand the essential components of
//! writing a FRAME pallet. It is typically used in beginner tutorials or in Substrate template
//! nodes as a starting point for creating a new pallet and **not meant to be used in production**.
//!
//! ## Overview
//!
//! This template pallet contains basic examples of:
//! - declaring a storage item that stores a single `u32` value
//! - declaring and using events
//! - declaring and using errors
//! - a dispatchable function that allows a user to set a new value to storage and emits an event
//!   upon success
//! - another dispatchable function that causes a custom error to be thrown
//!
//! Each pallet section is annotated with an attribute using the `#[pallet::...]` procedural macro.
//! This macro generates the necessary code for a pallet to be aggregated into a FRAME runtime.
//!
//! Learn more about FRAME macros [here](https://docs.substrate.io/reference/frame-macros/).
//!
//! ### Pallet Sections
//!
//! The pallet sections in this template are:
//!
//! - A **configuration trait** that defines the types and parameters which the pallet depends on
//!   (denoted by the `#[pallet::config]` attribute). See: [`Config`].
//! - A **means to store pallet-specific data** (denoted by the `#[pallet::storage]` attribute).
//!   See: [`storage_types`].
//! - A **declaration of the events** this pallet emits (denoted by the `#[pallet::event]`
//!   attribute). See: [`Event`].
//! - A **declaration of the errors** that this pallet can throw (denoted by the `#[pallet::error]`
//!   attribute). See: [`Error`].
//! - A **set of dispatchable functions** that define the pallet's functionality (denoted by the
//!   `#[pallet::call]` attribute). See: [`dispatchables`].
//!
//! Run `cargo doc --package pallet-template --open` to view this pallet's documentation.

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

// extern crate alloc;

// Re-export pallet items so that they can be accessed from the crate namespace.
use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult,
    ensure,
    storage::bounded_vec::BoundedVec,
    traits::{
        tokens::WithdrawReasons, Currency, EnsureOrigin, ExistenceRequirement, Get, Randomness,
        ReservableCurrency,
    },
    weights::WeightMeter,
    PalletId,
};
use frame_system::pallet_prelude::OriginFor;
use frame_system::{self as system, ensure_signed};
pub use pallet::*;
use scale_info::prelude::vec::Vec;
use sp_core::OpaquePeerId as PeerId;
use sp_runtime::traits::TrailingZeroInput;
use sp_runtime::Saturating;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use strum_macros::{EnumIter, FromRepr};

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
// #[cfg(test)]
// mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// ./target/release/hypertensor-node --dev
// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

pub mod utilities;
pub use utilities::*;
pub mod stake;
pub use stake::*;
pub mod rpc_info;
pub use rpc_info::*;
pub mod admin;
pub use admin::*;
pub mod supply;
pub use supply::*;
pub mod consensus;
pub use consensus::*;
pub mod overwatch_nodes;
pub use overwatch_nodes::*;
pub mod bank;
pub use bank::*;

// mod rewards;
// mod rewards_v4;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
    // Import various useful types required by all FRAME pallets.
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec;
    use sp_std::vec::Vec;

    // The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
    // (`Call`s) in this pallet.
    #[pallet::pallet]
    #[pallet::without_storage_info]
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

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + Send + Sync;

        /// Majority council 2/3s
        type MajorityCollectiveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Majority council 4/5s - Used in functions that include tokenization
        type SuperMajorityCollectiveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Number of blocks in an epoch
        #[pallet::constant]
        type EpochLength: Get<u32>;

        /// Number of epochs per year
        #[pallet::constant]
        type EpochsPerYear: Get<u32>;

        /// Initial transaction rate limit.
        #[pallet::constant]
        type InitialTxRateLimit: Get<u32>;

        /// Used in Randomness
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        /// Total overwatch emissions per epoch
        #[pallet::constant]
        type OverwatchEpochEmissions: Get<u128>;

        /// Maximum weight to consume in hooks functions
        #[pallet::constant]
        type MaximumHooksWeight: Get<Weight>;

        /// Epoch slots (see `on_initialize`)
        #[pallet::constant]
        type DesignatedEpochSlots: Get<u32>;
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
    /// Events for the pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // Subnets
        SubnetRegistered {
            owner: T::AccountId,
            name: Vec<u8>,
            subnet_id: u32,
        },
        SubnetActivated {
            subnet_id: u32,
        },
        SubnetDeactivated {
            subnet_id: u32,
            reason: SubnetRemovalReason,
        },

        // Subnet Nodes
        SubnetNodeRegistered {
            subnet_id: u32,
            subnet_node_id: u32,
            coldkey: T::AccountId,
            hotkey: T::AccountId,
            data: SubnetNode<T::AccountId>,
        },
        SubnetNodeActivated {
            subnet_id: u32,
            subnet_node_id: u32,
        },
        SubnetNodeRemoved {
            subnet_id: u32,
            subnet_node_id: u32,
        },
        SubnetNodeUpdateDelegateRewardRate {
            subnet_id: u32,
            subnet_node_id: u32,
            delegate_reward_rate: u128,
        },
        SubnetNodeUpdatePeerId {
            subnet_id: u32,
            subnet_node_id: u32,
            peer_id: PeerId,
        },
        SubnetNodeUpdateBootnode {
            subnet_id: u32,
            subnet_node_id: u32,
            bootnode: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        },
        SubnetNodeUpdateBootnodePeerId {
            subnet_id: u32,
            subnet_node_id: u32,
            bootnode_peer_id: PeerId,
        },
        SubnetNodeUpdateClientPeerId {
            subnet_id: u32,
            subnet_node_id: u32,
            client_peer_id: PeerId,
        },
        SubnetNodeUpdateUnique {
            subnet_id: u32,
            subnet_node_id: u32,
            unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        },
        SubnetNodeUpdateNonUnique {
            subnet_id: u32,
            subnet_node_id: u32,
            non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        },
        UpdateColdkey {
            coldkey: T::AccountId,
            new_coldkey: T::AccountId,
        },
        UpdateHotkey {
            hotkey: T::AccountId,
            new_hotkey: T::AccountId,
        },
        QueuedNodePrioritized {
            subnet_id: u32,
            subnet_node_id: u32,
        },
        QueuedNodeRemoved {
            subnet_id: u32,
            subnet_node_id: u32,
        },
        NodeClassGraduation {
            subnet_id: u32,
            subnet_node_id: u32,
            classification: SubnetNodeClassification,
        },

        // Stake
        StakeAdded(u32, T::AccountId, T::AccountId, u128),
        StakeRemoved(u32, T::AccountId, T::AccountId, u128),
        SubnetDelegateStakeAdded(u32, T::AccountId, u128),
        SubnetDelegateStakeRemoved(u32, T::AccountId, u128),
        SubnetDelegateStakeSwapped(u32, u32, T::AccountId, u128),
        DelegateNodeStakeAdded {
            account_id: T::AccountId,
            subnet_id: u32,
            subnet_node_id: u32,
            amount: u128,
        },
        DelegateNodeStakeRemoved {
            account_id: T::AccountId,
            subnet_id: u32,
            subnet_node_id: u32,
            amount: u128,
        },
        DelegateNodeStakeSwapped {
            account_id: T::AccountId,
            from_subnet_id: u32,
            from_subnet_node_id: u32,
            to_subnet_id: u32,
            to_subnet_node_id: u32,
            amount: u128,
        },
        DelegateNodeToSubnetDelegateStakeSwapped {
            account_id: T::AccountId,
            from_subnet_id: u32,
            from_subnet_node_id: u32,
            to_subnet_id: u32,
            amount: u128,
        },
        SubnetDelegateToNodeDelegateStakeSwapped {
            account_id: T::AccountId,
            from_subnet_id: u32,
            to_subnet_id: u32,
            to_subnet_node_id: u32,
            amount: u128,
        },

        // Admin
        SetMaxSubnets(u32),
        SetMaxBootnodes(u32),
        SetMaxSubnetBootnodeAccess(u32),
        SetSubnetEnactmentEpochs(u32),
        SetMaxSubnetPauseEpochs(u32),
        SetMinSubnetRegistrationFee(u128),
        SetMaxSubnetRegistrationFee(u128),
        SetMinRegistrationCost(u128),
        SetRegistrationCostDecayBlocks(u32),
        SetRegistrationCostAlpha(u128),
        SetNewRegistrationCostMultiplier(u128),
        SetMaxMinDelegateStakeMultiplier(u128),
        SetChurnLimits(u32, u32),
        SetChurnLimitMultipliers(u32, u32),
        SetQueueEpochs(u32, u32),
        SetMinActivationGraceEpochs(u32),
        SetMaxActivationGraceEpochs(u32),
        SetMinIdleClassificationEpochs(u32),
        SetMaxIdleClassificationEpochs(u32),
        SetIncludedClassificationEpochs(u32, u32),
        SetSubnetStakesLimits(u128, u128),
        SetDelegateStakePercentages(u128, u128),
        SetMinMaxRegisteredNodes(u32, u32),
        SetMaxSubnetDelegateStakeRewardsPercentageChange(u128),
        SetSubnetDelegateStakeRewardsUpdatePeriod(u32),
        SetMinAttestationPercentage(u128),
        SetSuperMajorityAttestationRatio(u128),
        SetBaseValidatorReward(u128),
        SetBaseSlashPercentage(u128),
        SetMaxSlashAmount(u128),
        SetColdkeyReputationIncreaseFactor(u128),
        SetColdkeyReputationDecreaseFactor(u128),
        SetNetworkMinStakeBalance(u128),
        SetNetworkMaxStakeBalance(u128),
        SetMinActiveNodeStakeEpochs(u32),
        SetMinDelegateStakeDeposit(u128),
        SetNodeRewardRateUpdatePeriod(u32),
        SetMaxRewardRateDecrease(u128),
        SetSubnetDistributionPower(u128),
        SetDelegateStakeWeightFactor(u128),
        SetInflationSigmoidMidpoint(u128),
        SetMaximumHooksWeight(u32),
        SetBaseNodeBurnAmount(u128),
        SetNodeBurnRates(u128, u128),
        SetDelegateStakeSubnetRemovalInterval(u32),
        SetSubnetRemovalIntervals(u32, u32),
        SetMinSubnetNodeConsecutiveIncludedEpochs(u32),
        SetMaxSubnetNodeConsecutiveIncludedEpochs(u32),
        SetSubnetPauseCooldownEpochs(u32),
        SetMaxSwapQueueCallsPerBlock(u32),
        SetMaxSubnetNodeMinWeightDecreaseReputationThreshold(u128),
        SetValidatorRewardK(u64),
        SetAttestorRewardExponent(u64),
        SetAttestorMinRewardFactor(u128),
        SetNodeReputationLimits(u128, u128),
        SetNodeReputationFactors(u128, u128),
        SetMinSubnetReputation(u128),
        SetNotInConsensusSubnetReputationFactor(u128),
        SetMaxPauseEpochsSubnetReputationFactor(u128),
        SetLessThanMinNodesSubnetReputationFactor(u128),
        SetValidatorAbsentSubnetReputationFactor(u128),
        SetInConsensusSubnetReputationFactor(u128),
        SetOverwatchWeightFactor(u128),
        SetMaxEmergencyValidatorEpochsMultiplier(u128),
        SetMaxEmergencySubnetNodes(u32),
        SetOverwatchStakeWeightFactor(u128),
        SetSubnetWeightFactors(SubnetWeightFactorsData),
        SetValidatorRewardMidpoint(u128),
        OverwatchNodeBlacklist(T::AccountId, bool),
        SetSigmoidSteepness(u128),
        SetMaxOverwatchNodes(u32),
        SetOverwatchEpochLengthMultiplier(u32),
        SetOverwatchCommitCutoffPercent(u128),
        SetOverwatchMinDiversificationRatio(u128),
        SetOverwatchMinRepScore(u128),
        SetOverwatchMinAvgAttestationRatio(u128),
        SetOverwatchMinAge(u32),
        SetOverwatchMinStakeBalance(u128),
        SetTxPause(),
        SetTxUnpause(),
        SetSubnetOwnerPercentage(u128),
        SetDelegateStakeCooldownEpochs(u32),
        SetNodeDelegateStakeCooldownEpochs(u32),
        SetStakeCooldownEpochs(u32),
        SetMaxUnbondings(u32),
        CollectiveRemoveSubnetNode(u32, u32),
        CollectiveRemoveOverwatchNode(u32),
        SetMinSubnetRegistrationEpochs(u32),
        SetSubnetRegistrationEpochs(u32),
        SetMinMaxSubnetNodes(u32, u32),
        SetMinStakeBalance(u128),
        SetTxRateLimit(u32),
        SetMinSubnetDelegateStakeFactor(u128),

        // Consensus / Validation and Attestation
        ValidatorSubmission {
            subnet_id: u32,
            account_id: T::AccountId,
            epoch: u32,
        },
        Attestation {
            subnet_id: u32,
            subnet_node_id: u32,
            epoch: u32,
        },
        Slashing {
            subnet_id: u32,
            account_id: T::AccountId,
            amount: u128,
        },

        // Rewards data
        SubnetRewards {
            subnet_id: u32,
            node_rewards: Vec<(u32, u128)>,
            delegate_stake_reward: u128,
            node_delegate_stake_rewards: Vec<(u32, u128)>,
        },
        OverwatchRewards {
            node_rewards: Vec<(u32, u128)>,
        },
        SubnetReputationUpdate {
            subnet_id: u32,
            prev_reputation: u128,
            new_reputation: u128,
        },
        NodeReputationUpdate {
            subnet_id: u32,
            subnet_node_id: u32,
            prev_reputation: u128,
            new_reputation: u128,
        },

        // Subnet owners
        SubnetNameUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            prev_value: Vec<u8>,
            value: Vec<u8>,
        },
        SubnetRepoUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            prev_value: Vec<u8>,
            value: Vec<u8>,
        },
        SubnetDescriptionUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            prev_value: Vec<u8>,
            value: Vec<u8>,
        },
        SubnetMiscUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            prev_value: Vec<u8>,
            value: Vec<u8>,
        },
        ChurnLimitUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u32,
        },
        ChurnLimitMultiplierUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u32,
        },
        RegistrationQueueEpochsUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u32,
        },
        IdleClassificationEpochsUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u32,
        },
        IncludedClassificationEpochsUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u32,
        },
        BootnodesUpdated {
            subnet_id: u32,
            added: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>,
            removed: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>,
        },
        SubnetPaused {
            subnet_id: u32,
            owner: T::AccountId,
        },
        SubnetUnpaused {
            subnet_id: u32,
            owner: T::AccountId,
        },
        SubnetForked {
            subnet_id: u32,
            owner: T::AccountId,
            subnet_node_ids: Vec<u32>,
        },
        SubnetForkRevert {
            subnet_id: u32,
            owner: T::AccountId,
        },
        AddSubnetRegistrationInitialColdkeys {
            subnet_id: u32,
            owner: T::AccountId,
            coldkeys: BTreeMap<T::AccountId, u32>,
        },
        RemoveSubnetRegistrationInitialColdkeys {
            subnet_id: u32,
            owner: T::AccountId,
            coldkeys: BTreeSet<T::AccountId>,
        },
        SubnetKeyTypesUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: BTreeSet<KeyType>,
        },
        SubnetMinMaxStakeBalanceUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            min: u128,
            max: u128,
        },
        SubnetDelegateStakeRewardsPercentageUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        MaxRegisteredNodesUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u32,
        },
        TransferPendingSubnetOwner {
            subnet_id: u32,
            owner: T::AccountId,
            new_owner: T::AccountId,
        },
        AcceptPendingSubnetOwner {
            subnet_id: u32,
            new_owner: T::AccountId,
        },
        AddSubnetBootnodeAccess {
            subnet_id: u32,
            owner: T::AccountId,
            new_account: T::AccountId,
        },
        RemoveSubnetBootnodeAccess {
            subnet_id: u32,
            owner: T::AccountId,
            remove_account: T::AccountId,
        },
        TargetNodeRegistrationsPerEpochUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u32,
        },
        NodeBurnRateAlphaUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        QueueImmunityEpochsUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u32,
        },
        SubnetNodeMinWeightDecreaseReputationThresholdUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        MinSubnetNodeReputationUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        AbsentDecreaseReputationFactorUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        IncludedIncreaseReputationFactorUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        BelowMinWeightDecreaseReputationFactorUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        NonAttestorDecreaseReputationFactorUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        NonConsensusAttestorDecreaseReputationFactorUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        ValidatorAbsentSubnetNodeReputationFactorUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        ValidatorNonConsensusSubnetNodeReputationFactorUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u128,
        },
        SubnetNodeConsecutiveIncludedEpochsUpdate {
            subnet_id: u32,
            owner: T::AccountId,
            value: u32,
        },
        IdentityRegistered {
            coldkey: T::AccountId,
            identity: ColdkeyIdentityData,
        },
        IdentityRemoved {
            coldkey: T::AccountId,
            identity: ColdkeyIdentityData,
        },
        SwapCallQueued {
            id: u32,
            account_id: T::AccountId,
            call: QueuedSwapCall<T::AccountId>,
        },
        SwapCallQueueUpdated {
            id: u32,
            account_id: T::AccountId,
            call: QueuedSwapCall<T::AccountId>,
        },
    }

    /// Errors that can be returned by this pallet.
    ///
    /// Errors tell users that something went wrong so it's important that their naming is
    /// informative. Similar to events, error documentation is added to a node's metadata so it's
    /// equally important that they have helpful documentation associated with them.
    ///
    /// This type of runtime error can be up to 4 bytes in size should you want to return additional
    /// information.
    #[pallet::error]
    pub enum Error<T> {
        InvalidChurnLimit,
        InvalidChurnLimitMultiplier,
        InvalidRegistrationQueueEpochs,
        InvalidIdleClassificationEpochs,
        InvalidIncludedClassificationEpochs,
        InvalidSubnetNodeConsecutiveIncludedEpochs,
        InvalidOverwatchEpochLengthMultiplier,

        /// Subnet must be registering or activated, this error usually occurs during the enactment period
        SubnetMustBeRegisteringOrActivated,
        /// Subnet must be registering to perform this action
        SubnetMustBeRegistering,
        /// Maximum subnets reached
        MaxSubnets,
        /// Account has subnet peer under subnet already
        InvalidSubnetNodeId,
        InvalidEmergencySubnetNodeId,
        /// Not subnet owner
        NotSubnetOwner,
        /// Not pending subnet owner
        NotPendingSubnetOwner,
        /// No pending subnet owner exists
        NoPendingSubnetOwner,
        /// Cannot pause again until pause cooldown epochs is reached
        SubnetPauseCooldownActive,
        /// Must be less than maximum registrations per epoch
        InvalidTargetNodeRegistrationsPerEpoch,
        /// Peer ID already in use in subnet
        PeerIdExist,
        /// Max subnet nodes reached
        MaxSubnetNodes,
        /// Bootnode peer ID already in use in subnet
        BootnodePeerIdExist,
        /// Client peer ID already in use in subnet
        ClientPeerIdExist,
        /// Invalid client peer ID
        InvalidClientPeerId,
        /// Bootnode already in use in subnet
        BootnodeExist,
        /// Hotkey doesn't have a subnet node
        InvalidHotkeySubnetNodeId,
        /// Subnet name already exists
        SubnetNameExist,
        /// Subnet repository already exists
        SubnetRepoExist,
        /// Subnet doesn't exist
        InvalidSubnetId,
        /// Subnet state must be active to perform this action
        SubnetMustBeActive,
        /// Subnet state must be paused to perform this action
        SubnetMustBePaused,
        InvalidMinEmergencySubnetNodes,
        InvalidMaxEmergencySubnetNodes,
        /// Subnet is paused, cannot perform this action
        SubnetIsPaused,
        /// Transaction rate limiter exceeded
        TxRateLimitExceeded,
        /// PeerId format invalid
        InvalidPeerId,
        /// PeerId format invalid
        InvalidBootnodePeerId,
        /// Coldkey not whitelisted to register
        ColdkeyRegistrationWhitelist,
        MaxRegisteredNodes,
        /// Wallet doesn't have enough balance to register subnet
        NotEnoughBalanceToRegisterSubnet,
        UniqueParameterTaken,
        /// Conditions to activate subnet no reached, see documentation
        SubnetActivationConditionsNotMetYet,
        /// Subnet registration cost is greater than max cost value
        CostGreaterThanMaxCost,
        /// Activation opens passed the MinSubnetRegistrationEpochs from the time of registration
        MinSubnetRegistrationEpochsNotMet,
        InvalidMaxRegisteredNodes,
        /// The number of initial coldkeys must be greater than or equal to the minimum nodes requirement
        InvalidSubnetRegistrationInitialColdkeys,
        /// Bootnodes is empty
        BootnodesEmpty,
        InvalidSubnetMinStake,
        InvalidSubnetMaxStake,
        // Min stake must be lesser than the max stake
        InvalidSubnetStakeParameters,
        InvalidMinDelegateStakePercentage,
        InvalidDelegateStakePercentage,
        DelegateStakePercentageUpdateTooSoon,
        /// The distance between the current rate and new rate is too large, see MaxSubnetDelegateStakeRewardsPercentageChange
        DelegateStakePercentageAbsDiffTooLarge,
        /// Must unstake to register
        MustUnstakeToRegister,

        // Admin
        /// Invalid maximimum subnets, must not exceed maximum allowable
        InvalidMaxSubnets,
        InvalidMinDelegateStakeDeposit,
        InvalidMaxBootnodes,
        InvalidMaxSubnetBootnodeAccess,
        InvalidMaxSubnetPauseEpochs,
        NoAvailableSlots,
        /// Invalid min subnet nodes, must not be less than minimum allowable
        InvalidMinSubnetNodes,
        /// Invalid maximimum subnet nodes, must not exceed maximimum allowable
        InvalidMaxSubnetNodes,
        /// Invalid percent number, must be in 1e18 format. Used for elements that only require correct format
        InvalidPercent,
        /// Emergency validators are set, can't update this value
        EmergencyValidatorsSet,
        MinSubnetNodeReputation,
        InvalidAbsentDecreaseReputationFactor,
        InvalidIncludedIncreaseReputationFactor,
        InvalidNonConsensusAttestorDecreaseReputationFactor,
        InvalidNonValidatorAbsentSubnetNodeReputationFactor,
        InvalidValidatorNonConsensusSubnetNodeReputationFactor,
        InvalidBelowMinWeightDecreaseReputationFactor,
        InvalidNonAttestorDecreaseReputationFactor,
        InvalidValidatorRewardK,
        InvalidAttestorRewardExponent,
        InvalidSuperMajorityAttestationRatio,
        /// Invalid values
        InvalidValues,
        /// Invalid percent number, must be in 1e2 format. Used for elements that only require correct format
        InvalidPerbillPercent,
        InvalidMinNodeBurnRate,
        InvalidMaxNodeBurnRate,
        InvalidDelegateStakeSubnetRemovalInterval,
        InvalidMaxSubnetRemovalInterval,
        InvalidMinSubnetRegistrationEpochs,
        InvalidSubnetRegistrationEpochs,
        InvalidStakeCooldownEpochs,
        InvalidMaxUnbondings,
        InvalidDelegateStakeCooldownEpochs,
        InvalidNodeDelegateStakeCooldownEpochs,

        // Staking
        /// u128 -> BalanceOf conversion error
        CouldNotConvertToBalance,
        InvalidAmount,
        /// Not enough balance on Account to stake and keep alive
        NotEnoughBalanceToStake,
        /// Not enough balance on Account to remove balance and keep alive
        NotEnoughBalance,
        /// Amount will kill account
        BalanceWithdrawalError,
        /// Burn failed, amount will kill account
        BalanceBurnError,
        /// Not enough stake to withdraw
        NotEnoughStakeToWithdraw,
        MaxStakeReached,
        MinDelegateStakeDepositNotReached,
        MinNodeDelegateStakeDepositNotReached,
        // if min stake not met on both stake and unstake
        MinStakeNotReached,
        // delegate staking
        CouldNotConvertToShares,
        // Maximum unlockings reached for the unbonding ledger, see MaxUnbondings
        MaxUnlockingsReached,
        NoStakeUnbondingsOrCooldownNotMet,
        MinDelegateStake,
        /// Elected validator on current epoch cannot unstake to ensure they are able to be rewarded or penalized
        ElectedValidatorCannotUnstake,
        /// Elected validator on current epoch cannot remove to ensure they are able to be rewarded or penalized
        ElectedValidatorCannotRemove,
        MinActiveNodeStakeEpochs,
        /// Shares entered is zero, must be greater than
        SharesZero,
        /// Amount entered is zero, must be greater than
        AmountZero,

        /// Consensus submission doesn't exist on this epoch
        InvalidSubnetConsensusSubmission,
        SubnetActivatedAlready,
        /// Subnet not qualified for removal
        InvalidSubnetRemoval,

        // Validation and Attestation
        /// Subnet rewards data already submitted by validator
        SubnetRewardsAlreadySubmitted,
        SubnetEpochDataIsNone,
        /// Not epoch validator
        InvalidValidator,
        /// Validator not elected on subnet epoch
        NoElectedValidator,
        /// Already attested validator data
        AlreadyAttested,
        /// Score overflow
        ScoreOverflow,
        ElectionSlotInsertFail,
        /// Not the key owner
        NotKeyOwner,
        /// Subnet Node param A must be unique
        SubnetNodeUniqueParamTaken,
        // Hotkey already registered to coldkey
        HotkeyAlreadyRegisteredToColdkey,
        /// Burn amount exceeds maximum burn amount allowable
        MaxBurnAmountExceeded,
        // Hotkey not registered to coldkey
        OldHotkeyNotRegistered,
        /// Identity is taken by another coldkey
        IdentityTaken,
        /// Identity field cannot be empty
        IdentityFieldEmpty,
        /// No change between current and new delegate reward rate, make sure to increase or decrease it
        NoDelegateRewardRateChange,
        /// Invalid delegate reward rate above 100%
        InvalidDelegateRewardRate,
        /// Rate of change to great for decreasing reward rate, see MaxRewardRateDecrease
        SurpassesMaxRewardRateDecrease,
        /// Too many updates to reward rate in the NodeRewardRateUpdatePeriod
        MaxRewardRateUpdates,
        /// Transactions are paused
        Paused,

        // Keys
        /// Hotkey has an owner and hotkeys must be unique to each node. If you're the owner, use a fresh hotkey
        HotkeyHasOwner,
        ColdkeyMatchesHotkey,
        PeerIdsMustBeUnique,
        NoCommitFound,
        /// Reveal doesn't match commit or no commit
        RevealMismatch,
        /// Commits vector is empty
        CommitsEmpty,
        /// Already committed on this epoch and subnet ID
        AlreadyCommitted,
        /// Invalid subnet weight, must be below percentage factor 1e18
        InvalidWeight,
        /// Maximum overwatch nodes reached
        MaxOverwatchNodes,
        /// Overwatch scores are based on the previous epoch, therefor a node cannot begin commiting until overwatch epoch 1 to avoid underflow
        OverwatchEpochIsZero,
        /// Account already in bootnode access list
        InBootnodeAccessList,
        /// Account not in bootnode access list
        NotInAccessList,
        /// Maximum bootnodes reached, see MaxBootnodes
        TooManyBootnodes,
        /// Caller cannot access this function
        InvalidAccess,
        /// Not in the commit period of the epoch
        NotCommitPeriod,
        /// Not in the reveal period of the epoch
        NotRevealPeriod,
        /// Not qualified to be an overwatch node, see ColdkeyReputation
        ColdkeyNotOverwatchQualified,
        /// Is qualified to be an overwatch node, see ColdkeyReputation
        ColdkeyOverwatchQualified,
        /// Overwatch node ID doesn't exist
        InvalidOverwatchNodeId,
        /// Maximum number of accounts for bootnode update access
        MaxSubnetBootnodeAccess,
        /// Swap call not found under ID
        SwapCallNotFound,
        /// Coldkey is blacklisted from being an Overwatch Node
        ColdkeyBlacklisted,
    }

    /// Subnet data
    ///
    /// # Fields
    ///
    /// * `id` - Unique identifier.
    /// * `friendly_id` - Friendly ID always between 1-max subnets.
    /// * `name` - Unique name of the subnet.
    /// * `repo` - Unique repository of the subnet.
    /// * `description` - Description of what the subnet does and use cases.
    /// * `misc` - Misc data.
    /// * `state` - Registered, Active, or Paused.
    /// * `start_epoch` - Epoch subnet registered.
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct SubnetData {
        pub id: u32,
        pub friendly_id: u32,
        pub name: Vec<u8>,
        pub repo: Vec<u8>,
        pub description: Vec<u8>,
        pub misc: Vec<u8>,
        pub state: SubnetState,
        pub start_epoch: u32,
    }

    /// Operational states of a subnet in its lifecycle.
    ///
    /// This enum tracks the current state of a subnet from registration through activation
    /// and potential pausing. Subnets progress through these states based on time, owner
    /// actions, and network conditions.
    ///
    /// # Variants
    ///
    /// * `Registered` - The subnet has been registered but not yet activated. During this
    ///   state, the subnet is in its enactment period where only whitelisted coldkeys from
    ///   `initial_coldkeys` can register nodes and users can delegate stake before the
    ///   enactment period.
    ///
    /// * `Active` - The subnet is fully operational and participating in consensus epochs.
    ///
    /// * `Paused` - The subnet has been temporarily suspended by the owner. While paused,
    ///   no consensus operations occur and no new nodes can register, but existing nodes
    ///   and stake remain in place. The owner can resume the subnet to return it to `Active`
    ///   state, or it may be automatically removed if the pause period expires without resumption.
    ///
    /// # State Transitions
    ///
    /// The typical lifecycle is:
    /// ```text
    /// Registered → Active ⇄ Paused
    ///            ↓        ↓       ↓
    ///          Removed (terminal state, not in enum)
    /// ```
    #[derive(
        Default,
        EnumIter,
        FromRepr,
        Copy,
        Encode,
        Decode,
        Clone,
        PartialOrd,
        PartialEq,
        Eq,
        RuntimeDebug,
        Ord,
        scale_info::TypeInfo,
    )]
    pub enum SubnetState {
        #[default]
        Registered,
        Active,
        Paused,
    }

    /// All key types a subnet can support
    #[derive(
        Default,
        EnumIter,
        FromRepr,
        Copy,
        Encode,
        Decode,
        Clone,
        PartialOrd,
        PartialEq,
        Eq,
        RuntimeDebug,
        Ord,
        scale_info::TypeInfo,
    )]
    pub enum KeyType {
        #[default]
        Rsa,
        Ed25519,
        Secp256k1,
        Ecdsa,
    }

    /// Configuration data for a subnet during its registration phase before activation.
    ///
    /// This struct contains all the parameters and metadata needed to register a new subnet.
    /// Once the subnet completes its enactment period and becomes active, this data is used
    /// to initialize the operational subnet configuration.
    ///
    /// # Fields
    ///
    /// ## Identity (Unique)
    ///
    /// * `name` - Unique name of the subnet. No two subnets can share the same name.
    /// * `repo` - Unique repository URL pointing to the open-source subnet codebase.
    ///   Must be distinct from all other subnet repositories.
    /// * `description` - Text description explaining the subnet's purpose, goals, and
    ///   functionality. Multiple subnets can have similar descriptions.
    /// * `misc` - Miscellaneous metadata or additional information that doesn't fit
    ///   other categories. Non-unique across subnets.
    /// * `min_stake` - Minimum stake balance required for a subnet node to register and
    ///   participate in this subnet. Creates an economic barrier to entry.
    /// * `max_stake` - Maximum stake balance a single subnet node can hold. Promotes
    ///   decentralization by preventing excessive stake concentration.
    /// * `delegate_stake_percentage` - Percentage of subnet emissions allocated to delegate
    ///   stakers rather than node operators, represented as a fixed-point number (where
    ///   1e18 = 100%). Balances rewards between operators and supporters.
    /// * `initial_coldkeys` - Set of coldkey accounts with the maximum number of nodes they can register
    //    while subnet is registering, granted permission to register nodes.
    ///   during the subnet's registration phase. After activation, registration typically
    ///   opens to all eligible participants.
    /// * `key_types` - Set of cryptographic key types (signature algorithms) that the subnet
    ///   accepts for node registration. This is informational metadata not enforced onchain.
    /// * `bootnodes` - Set of multiaddresses or connection information for official bootnodes
    ///   that help new nodes discover and connect to the subnet network. Can be updated by
    ///   the subnet owner and whitelisted accounts. This is informational metadata for
    ///   network coordination.
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct RegistrationSubnetData<AccountId> {
        pub name: Vec<u8>,
        pub repo: Vec<u8>,
        pub description: Vec<u8>,
        pub misc: Vec<u8>,
        pub min_stake: u128,
        pub max_stake: u128,
        pub delegate_stake_percentage: u128,
        pub initial_coldkeys: BTreeMap<AccountId, u32>,
        pub key_types: BTreeSet<KeyType>,
        pub bootnodes: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>,
    }

    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct RegistrationSubnetDataV2<AccountId> {
        pub name: Vec<u8>,
        pub repo: Vec<u8>,
        pub description: Vec<u8>,
        pub misc: Vec<u8>,
        pub min_stake: u128,
        pub max_stake: u128,
        pub delegate_stake_percentage: u128,
        pub initial_coldkeys: Vec<(AccountId, u32)>,
        pub key_types: BTreeSet<KeyType>,
        pub bootnodes: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>,
    }

    /// Complete subnet information aggregated for RPC queries.
    ///
    /// This struct provides a comprehensive view of a subnet's configuration, state, and
    /// statistics. It is designed as an RPC helper to allow external clients to query all
    /// relevant subnet data in a single call, avoiding multiple separate queries.
    ///
    /// # Fields
    ///
    /// * `id` - The unique subnet ID assigned at registration.
    /// * `name` - The human-readable name of the subnet.
    /// * `repo` - Repository URL where the subnet's code or documentation is hosted.
    /// * `description` - A text description explaining the subnet's purpose and functionality.
    /// * `misc` - Miscellaneous metadata that doesn't fit other categories.
    /// * `state` - The current operational state of the subnet (e.g., active, paused, removed).
    ///   See `SubnetState` for possible values.
    /// * `start_epoch` - The epoch when the subnet became active and began operations.
    /// * `churn_limit` - Maximum number of nodes that can change classification (join/leave
    ///   active participation) per epoch, preventing network instability from rapid turnover.
    /// * `churn_limit_multiplier` - The multiplier for the ChurnLimit
    /// * `min_stake` - Minimum stake required for a node to register and participate in this subnet.
    /// * `max_stake` - Maximum stake a single node can hold in this subnet, promoting
    ///   decentralization.
    /// * `queue_immunity_epochs` - Number of epochs a newly queued node is protected from
    ///   removal, giving them time to establish themselves.
    /// * `target_node_registrations_per_epoch` - Target number of new node registrations per
    ///   epoch, used to dynamically adjust burn rates.
    /// * `subnet_node_queue_epochs` - Number of epochs nodes spend in the queue before being
    ///   eligible for activation.
    /// * `idle_classification_epochs` - Number of epochs a node remains in `Idle` classification
    ///   before transitioning to `Included`.
    /// * `included_classification_epochs` - Number of epochs a node remains in `Included`
    ///   classification before becoming eligible for `Validator` status.
    /// * `delegate_stake_percentage` - Percentage of subnet rewards allocated to delegate stakers,
    ///   represented as a fixed-point number (where 1e18 = 100%).
    /// * `node_burn_rate_alpha` - Smoothing factor (alpha) used in the exponential moving average
    ///   calculation for dynamic burn rate adjustments. Must be <= 1e18 (100%).
    /// * `initial_coldkeys` - Optional set of coldkey accounts that were granted initial access
    ///   or privileges when the subnet was created.
    /// * `max_registered_nodes` - Maximum total number of nodes allowed to be registered in
    ///   this subnet simultaneously.
    /// * `owner` - Optional account ID of the subnet owner who created and manages the subnet.
    /// * `pending_owner` - Optional account ID of a pending new owner during ownership transfer.
    /// * `registration_epoch` - Optional epoch when the subnet was registered. Present during
    ///   the enactment period before activation.
    /// * `key_types` - Set of key types (cryptographic algorithms) accepted for node registration
    ///   in this subnet.
    /// * `slot_index` - Optional slot index if the subnet is scheduled for consensus operations
    ///   in a specific slot rotation.
    /// * `bootnode_access` - Set of coldkey accounts that are authorized to register and operate
    ///   bootnodes for this subnet.
    /// * `bootnodes` - Set of multiaddresses or connection information for the subnet's bootnodes,
    ///   allowing new nodes to discover and join the network.
    /// * `total_nodes` - Total count of all registered nodes in the subnet, regardless of
    ///   classification or activity status.
    /// * `total_active_nodes` - Count of nodes that are actively participating in consensus
    ///   (nodes with `Idle`, `Included`, or `Validator` classification).
    /// * `total_electable_nodes` - Count of nodes eligible to be elected as validators
    ///   (nodes with `Validator` classification).
    /// * `current_min_delegate_stake` - The current minimum required subnet delegate stake balance.
    #[derive(
        Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, Ord, scale_info::TypeInfo,
    )]
    pub struct SubnetInfo<AccountId> {
        pub id: u32,
        pub friendly_id: Option<u32>,
        pub name: Vec<u8>,
        pub repo: Vec<u8>,
        pub description: Vec<u8>,
        pub misc: Vec<u8>,
        pub state: SubnetState,
        pub start_epoch: u32,
        pub churn_limit: u32,
        pub churn_limit_multiplier: u32,
        pub min_stake: u128,
        pub max_stake: u128,
        pub queue_immunity_epochs: u32,
        pub target_node_registrations_per_epoch: u32,
        pub node_registrations_this_epoch: u32,
        pub subnet_node_queue_epochs: u32,
        pub idle_classification_epochs: u32,
        pub included_classification_epochs: u32,
        pub delegate_stake_percentage: u128,
        pub last_delegate_stake_rewards_update: u32,
        pub node_burn_rate_alpha: u128,
        pub current_node_burn_rate: u128,
        pub initial_coldkeys: Option<BTreeMap<AccountId, u32>>,
        pub initial_coldkey_data: Option<BTreeMap<AccountId, u32>>,
        pub max_registered_nodes: u32,
        pub owner: Option<AccountId>,
        pub pending_owner: Option<AccountId>,
        pub registration_epoch: Option<u32>,
        pub prev_pause_epoch: u32,
        pub key_types: BTreeSet<KeyType>,
        pub slot_index: Option<u32>,
        pub slot_assignment: Option<u32>,
        pub subnet_node_min_weight_decrease_reputation_threshold: u128,
        pub reputation: u128,
        pub min_subnet_node_reputation: u128,
        pub absent_decrease_reputation_factor: u128,
        pub included_increase_reputation_factor: u128,
        pub below_min_weight_decrease_reputation_factor: u128,
        pub non_attestor_decrease_reputation_factor: u128,
        pub non_consensus_attestor_decrease_reputation_factor: u128,
        pub validator_absent_subnet_node_reputation_factor: u128,
        pub validator_non_consensus_subnet_node_reputation_factor: u128,
        pub bootnode_access: BTreeSet<AccountId>,
        pub bootnodes: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>,
        pub total_nodes: u32,
        pub total_active_nodes: u32,
        pub total_electable_nodes: u32,
        pub current_min_delegate_stake: u128,
        pub total_subnet_stake: u128,
        pub total_subnet_delegate_stake_shares: u128,
        pub total_subnet_delegate_stake_balance: u128,
    }

    /// A subnet node representing a participant in a subnet.
    ///
    /// This struct contains all the identity, network, and operational data for a node
    /// participating in a subnet. Nodes progress through different classification levels
    /// and can earn rewards based on their performance and delegate stake.
    ///
    /// # Fields
    ///
    /// * `id` - The unique subnet node ID assigned when the node registers to the subnet.
    ///   This ID is used throughout consensus operations to reference the node.
    /// * `hotkey` - The unique hotkey account associated with this node. The hotkey is used
    ///   for signing transactions and identifying the node operator.
    /// * `peer_id` - The libp2p peer ID used for subnet communication and peer-to-peer
    ///   networking. This identifier is used during proof-of-stake operations and network
    ///   consensus.
    /// * `bootnode_peer_id` - The libp2p peer ID used when this node operates as a bootnode,
    ///   helping new nodes discover and connect to the subnet network.
    /// * `client_peer_id` - The libp2p peer ID used when this node operates in client-only
    ///   mode, allowing it to participate in the network without full consensus responsibilities.
    /// * `bootnode` - Optional multiaddress or connection information for the bootnode.
    ///   Contains network addressing details needed for other nodes to connect to this
    ///   bootnode. Limited to `DefaultMaxVectorLength` bytes.
    /// * `classification` - The current classification status of the node, tracking its
    ///   progression through registration, activation, and validator eligibility stages.
    ///   See `SubnetNodeClassification` for details on lifecycle stages.
    /// * `delegate_reward_rate` - The percentage of rewards that delegate stakers receive
    ///   from this node's earnings, represented as a fixed-point number (where 1e18 = 100%).
    ///   This rate determines how rewards are split between the node operator and their delegates.
    /// * `last_delegate_reward_rate_update` - The block number when the delegate reward rate
    ///   was last modified. This prevents frequent rate changes and provides transparency
    ///   about rate stability.
    /// * `unique` - Optional field for storing unique, node-specific data that must be distinct
    ///   across all nodes in the subnet. Can be used for custom node identifiers or properties.
    ///   Limited to `DefaultMaxVectorLength` bytes.
    /// * `non_unique` - Optional field for storing miscellaneous data that doesn't need to be
    ///   unique across nodes. Can be used for metadata, configuration, or other supplementary
    ///   information. Limited to `DefaultMaxVectorLength` bytes.
    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        PartialOrd,
        Ord,
        scale_info::TypeInfo,
    )]
    pub struct SubnetNode<AccountId> {
        pub id: u32,
        pub hotkey: AccountId,
        pub peer_id: PeerId,
        pub bootnode_peer_id: PeerId,
        pub client_peer_id: PeerId,
        pub bootnode: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        pub classification: SubnetNodeClassification,
        pub delegate_reward_rate: u128,
        pub last_delegate_reward_rate_update: u32,
        pub unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        pub non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
    }

    /// Subnet Node Info
    /// RPC helper
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct SubnetNodeInfo<AccountId> {
        pub subnet_id: u32,
        pub subnet_node_id: u32,
        pub coldkey: AccountId,
        pub hotkey: AccountId,
        pub peer_id: PeerId,
        pub bootnode_peer_id: PeerId,
        pub client_peer_id: PeerId,
        pub bootnode: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        pub identity: ColdkeyIdentityData,
        pub classification: SubnetNodeClassification,
        pub delegate_reward_rate: u128,
        pub last_delegate_reward_rate_update: u32,
        pub unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        pub non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        pub stake_balance: u128,
        pub total_node_delegate_stake_shares: u128,
        pub node_delegate_stake_balance: u128,
        pub coldkey_reputation: Reputation,
        pub subnet_node_reputation: u128,
        pub node_slot_index: Option<u32>,
        pub consecutive_idle_epochs: u32,
        pub consecutive_included_epochs: u32,
    }

    /// RPC helper for node stakes
    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        PartialOrd,
        Ord,
        scale_info::TypeInfo,
    )]
    pub struct SubnetNodeStakeInfo<AccountId> {
        pub subnet_id: Option<u32>,
        pub subnet_node_id: Option<u32>,
        pub hotkey: AccountId,
        pub balance: u128,
    }

    /// RPC helper for delegate stakes
    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        PartialOrd,
        Ord,
        scale_info::TypeInfo,
    )]
    pub struct DelegateStakeInfo {
        pub subnet_id: u32,
        pub shares: u128,
        pub balance: u128,
    }

    /// RPC helper for node delegate stakes
    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        PartialOrd,
        Ord,
        scale_info::TypeInfo,
    )]
    pub struct NodeDelegateStakeInfo {
        pub subnet_id: u32,
        pub subnet_node_id: u32,
        pub shares: u128,
        pub balance: u128,
    }

    /// Classification levels for subnet nodes, representing their participation status.
    ///
    /// This enum defines the lifecycle stages of a subnet node, from initial registration
    /// through active participation in consensus. Nodes progress through these classes
    /// based on their participation and performance, with automatic transitions occurring
    /// during consensus epochs.
    ///
    /// The ordering of variants is significant for the `has_classification` function,
    /// which checks if a node has reached at least a certain classification level.
    ///
    /// # Variants
    ///
    /// * `Registered` - The subnet node has been registered but is not yet included in
    ///   consensus operations. Nodes in this class are waiting in the queue to become active.
    ///   This is the default initial state for new nodes.
    /// * `Idle` - The subnet node is activated and waiting for their first consensus epoch.
    ///   Nodes remain in this class until they appear in a successful consensus submission,
    ///   at which point they automatically transition to `Included`.
    /// * `Included` - The subnet node has been included in at least one successful consensus
    ///   epoch. This indicates the node is actively participating and being scored by validators.
    ///   Nodes automatically transition from `Idle` to `Included` after their first appearance
    ///   in consensus data.
    /// * `Validator` - The subnet node is eligible to become a validator and submit consensus
    ///   data. Nodes automatically transition from `Included` to `Validator` after appearing
    ///   in a successful consensus epoch while in the `Included` class.
    ///
    /// # State Transitions
    ///
    /// The typical progression is:
    /// ```text
    /// Registered → Idle → Included → Validator
    /// ```
    #[derive(
        Default,
        EnumIter,
        FromRepr,
        Copy,
        Encode,
        Decode,
        Clone,
        PartialOrd,
        PartialEq,
        Eq,
        RuntimeDebug,
        Ord,
        scale_info::TypeInfo,
    )]
    pub enum SubnetNodeClass {
        #[default]
        Registered,
        Idle,
        Included,
        Validator,
    }

    impl SubnetNodeClass {
        /// Increments the node class, but if already at the highest level, stays at Validator.
        pub fn next(&self) -> Self {
            let new_value = (*self as usize) + 1; // Increment the enum value
            Self::from_repr(new_value).unwrap_or(*self) // If out of bounds, return the current value
        }

        /// Decrements the node class, but if already at the lowest level, stays at Registered.
        pub fn previous(&self) -> Self {
            if *self == Self::Registered {
                return Self::Registered; // Stay at the lowest level
            }
            let new_value = (*self as usize) - 1; // Decrement the enum value
            Self::from_repr(new_value).unwrap_or(*self) // If out of bounds, return the current value
        }
    }

    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        Ord,
        PartialOrd,
        scale_info::TypeInfo,
    )]
    pub struct SubnetNodeClassification {
        pub node_class: SubnetNodeClass,
        pub start_epoch: u32,
    }

    impl<AccountId> SubnetNode<AccountId> {
        pub fn has_classification(&self, required: &SubnetNodeClass, subnet_epoch: u32) -> bool {
            self.classification.node_class >= *required
                && self.classification.start_epoch <= subnet_epoch
        }
    }

    /// Incentives protocol format
    ///
    /// Scoring is calculated off-chain between subnet nodes hosting AI subnets together
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct SubnetNodeConsensusData {
        pub subnet_node_id: u32,
        pub score: u128,
    }

    /// Identity data associated with a coldkey account.
    ///
    /// This struct stores public identity information that coldkey holders can set to
    /// identify themselves. It provides a standardized way for subnet
    /// owners, validators, and other participants to share contact information and
    /// branding with the community.
    ///
    /// # Fields
    ///
    /// * `name` - The display name or project name associated with this coldkey.
    ///   Limited to `DefaultMaxVectorLength` bytes.
    /// * `url` - A website URL for the project or entity. This could be a personal
    ///   website, project homepage, or documentation site. Limited to `DefaultMaxUrlLength` bytes.
    /// * `image` - A URL to an avatar, logo, or profile image. Limited to
    ///   `DefaultMaxUrlLength` bytes.
    /// * `discord` - Discord username or server identifier for community contact.
    ///   Limited to `DefaultMaxSocialIdLength` bytes.
    /// * `x` - X (formerly Twitter) handle or profile URL for social media presence.
    ///   Limited to `DefaultMaxSocialIdLength` bytes.
    /// * `telegram` - Telegram username or group identifier for direct communication.
    ///   Limited to `DefaultMaxSocialIdLength` bytes.
    /// * `github` - GitHub username or repository URL for open source projects and code.
    ///   Limited to `DefaultMaxUrlLength` bytes.
    /// * `hugging_face` - Hugging Face profile or model repository URL for AI/ML projects.
    ///   Limited to `DefaultMaxUrlLength` bytes.
    /// * `description` - A text description of the project, entity, or purpose of this coldkey.
    ///   Limited to `DefaultMaxVectorLength` bytes.
    /// * `misc` - Miscellaneous additional information that doesn't fit other categories.
    ///   Can be used for custom metadata or supplementary details. Limited to
    ///   `DefaultMaxVectorLength` bytes.
    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        PartialOrd,
        Ord,
        scale_info::TypeInfo,
    )]
    pub struct ColdkeyIdentityData {
        pub name: BoundedVec<u8, DefaultMaxVectorLength>,
        pub url: BoundedVec<u8, DefaultMaxUrlLength>,
        pub image: BoundedVec<u8, DefaultMaxUrlLength>,
        pub discord: BoundedVec<u8, DefaultMaxSocialIdLength>,
        pub x: BoundedVec<u8, DefaultMaxSocialIdLength>,
        pub telegram: BoundedVec<u8, DefaultMaxSocialIdLength>,
        pub github: BoundedVec<u8, DefaultMaxUrlLength>,
        pub hugging_face: BoundedVec<u8, DefaultMaxUrlLength>,
        pub description: BoundedVec<u8, DefaultMaxVectorLength>,
        pub misc: BoundedVec<u8, DefaultMaxVectorLength>,
    }

    /// Attestation entry for validator consensus submissions.
    ///
    /// This struct records when a validator attested to a consensus submission and
    /// allows them to include optional arbitrary data. It is stored in the `attests`
    /// map of `ConsensusData` to track which validators have reviewed and approved
    /// the consensus data.
    ///
    /// # Fields
    ///
    /// * `block` - The block number at which this attestation was made. This provides
    ///   a timestamp for when the validator attested and can be used to track attestation
    ///   timing relative to the consensus submission.
    /// * `attestor_progress` - The percentage progress from the validator proposal to when attestor attests
    /// * `reward_factor` - The percentage factor against the reward the attestor receives
    /// * `data` - Optional arbitrary attestation data that the validator can include.
    ///   This data is not used in any onchain logic but allows validators to attach
    ///   metadata, signatures, or other information for off-chain verification or
    ///   coordination purposes.
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct AttestEntry {
        pub block: u32,
        pub attestor_progress: u128,
        pub reward_factor: u128,
        pub data: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
    }

    /// This struct represents the processed consensus submission. It is generated
    /// during the `precheck_subnet_consensus_submission` process and contains all
    /// the information needed for reward distribution and subnet state updates.
    ///
    /// # Fields
    ///
    /// * `validator_subnet_node_id` - The subnet node ID of the validator who originally
    ///   proposed this consensus data.
    /// * `validator_epoch_progress` - The percent process of the epoch when the validator submitted
    ///   consensus data, represented as 1e18.
    /// * `attestation_ratio` - The ratio of validators who have attested to this consensus
    ///   submission, represented as a fixed-point number (where 1e18 = 100%). This indicates
    ///   the level of agreement among validators for this submission.
    /// * `weight_sum` - The total sum of all scores in the consensus data. This is used
    ///   for normalization during reward distribution and helps prevent overflow issues.
    /// * `data_length` - The number of peers included in the consensus data. This provides
    ///   quick access to the size of the consensus without iterating through the data vector.
    /// * `data` - A vector of consensus data entries, where each entry contains a subnet
    ///   node ID and their corresponding score. Only peers with `Included` classification
    ///   are present in this vector.
    /// * `attests` - A map of subnet node IDs to their attestation entries. Each entry
    ///   contains the block number when the attestation was made and optional attestation
    ///   data. This tracks which validators have attested to this consensus submission.
    /// * `subnet_nodes` - A vector of all active subnet nodes that are eligible for
    ///   consensus and rewards. This includes nodes with `Idle` classification and above,
    ///   retrieved at the time of consensus submission for efficient processing.
    /// * `prioritize_queue_node_id` - Optional node ID from the registration queue to
    ///   move to the front of the queue. This is set by the proposing validator and
    ///   executed during consensus finalization if the submission is accepted.
    /// * `remove_queue_node_id` - Optional node ID from the registration queue to remove.
    ///   This is set by the proposing validator and executed during consensus finalization
    ///   if the submission is accepted and the node has passed its immunity period.
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct ConsensusSubmissionData<AccountId> {
        pub validator_subnet_node_id: u32,
        pub validator_epoch_progress: u128,
        pub validator_reward_factor: u128,
        pub attestation_ratio: u128,
        pub weight_sum: u128,
        pub data_length: u32,
        pub data: Vec<SubnetNodeConsensusData>,
        pub attests: BTreeMap<u32, AttestEntry>, // subnet_node_id: AttestEntry
        pub subnet_nodes: Vec<SubnetNode<AccountId>>,
        pub prioritize_queue_node_id: Option<u32>,
        pub remove_queue_node_id: Option<u32>,
    }

    /// Reasons for a subnet's removal from the network.
    ///
    /// This enum tracks why a subnet was removed, which is important for
    /// transparency and debugging. The reason is recorded when a subnet is
    /// deactivated or removed from the network.
    ///
    /// # Variants
    ///
    /// * `MinReputation` - The subnet has went below the minimum reputation value
    ///   allowed and is being removed as a consequence of repeated violations or failures.
    /// * `MinSubnetNodes` - The subnet's active node count has fallen below the minimum
    ///   required threshold, making it unable to maintain consensus and operation.
    /// * `MinSubnetDelegateStake` - The subnet's delegate stake balance has fallen below
    ///   the minimum required supply, indicating insufficient economic support to continue.
    /// * `Council` - The subnet was manually removed by council governance decision,
    ///   typically for policy violations or network health reasons.
    /// * `EnactmentPeriod` - The subnet was registered but never activated within the
    ///   required enactment period, indicating abandonment or inability to launch.
    /// * `MaxSubnets` - The network has reached maximum subnet capacity, and this subnet
    ///   was removed as the lowest-rated subnet to make room for new registrations.
    /// * `Owner` - The subnet owner voluntarily removed their own subnet.
    /// * `PauseExpired` - The subnet was paused and the pause period expired without
    ///   being resumed, resulting in automatic removal.
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub enum SubnetRemovalReason {
        MinReputation,
        MinSubnetNodes,
        MinSubnetDelegateStake,
        Council,
        EnactmentPeriod,
        MaxSubnets,
        Owner,
        PauseExpired,
    }

    /// Consensus data for a subnet epoch, storing the validator's submission and attestations.
    ///
    /// This struct represents the complete consensus state for a subnet during a specific epoch.
    /// It is initially created and stored in `propose_attestation` when the elected validator
    /// submits their consensus data, and is subsequently updated in `attest` as other validators
    /// attest to the submission.
    ///
    /// # Fields
    ///
    /// * `validator_id` - The subnet node ID of the elected validator for this epoch who
    ///   proposed the consensus data. This validator is responsible for submitting the initial
    ///   scores and queue management decisions.
    /// * `block` - Block proposed.
    /// * `validator_epoch_progress` - The percent process of the epoch when the validator submitted
    ///   consensus data, represented as 1e18.
    /// * `attests` - A map of subnet node IDs to their attestation entries, tracking which
    ///   validators have attested to this consensus submission. Each entry contains the block
    ///   number when the attestation was made and optional attestation data. The proposing
    ///   validator automatically attests to their own submission upon proposal.
    /// * `subnet_nodes` - A vector of all active subnet nodes that are eligible for consensus
    ///   and rewards at the time of submission. This includes nodes with `Idle` classification
    ///   and above, captured during proposal for efficient processing during reward distribution.
    /// * `prioritize_queue_node_id` - Optional node ID from the registration queue to move
    ///   to the front of the queue. Set by the proposing validator and executed during consensus
    ///   finalization if the submission achieves sufficient attestation. The node must exist
    ///   in the queue for this operation to be applied.
    /// * `remove_queue_node_id` - Optional node ID from the registration queue to remove.
    ///   Set by the proposing validator and executed during consensus finalization if the
    ///   submission achieves sufficient attestation and the node has passed its immunity period.
    /// * `data` - A vector of consensus data entries submitted by the elected validator,
    ///   where each entry contains a subnet node ID and their corresponding performance score.
    ///   Only peers with `Included` classification are retained, and duplicates are automatically
    ///   removed based on subnet node ID.
    /// * `args` - Optional arbitrary arguments for subnet-specific validation and coordination.
    ///   This data is not used in any onchain logic but allows subnets to pass custom parameters
    ///   that validators can use for off-chain validation or coordination purposes.
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct ConsensusData<AccountId> {
        pub validator_id: u32, // Chosen validator of the epoch
        pub block: u32,
        pub validator_epoch_progress: u128,
        pub validator_reward_factor: u128,
        pub attests: BTreeMap<u32, AttestEntry>, // Count of attestations of the submitted data (node ID, (block, data))
        pub subnet_nodes: Vec<SubnetNode<AccountId>>,
        pub prioritize_queue_node_id: Option<u32>,
        pub remove_queue_node_id: Option<u32>,
        pub data: Vec<SubnetNodeConsensusData>, // Data submitted by chosen validator
        pub args: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>, // Optional arguements to pass for subnet to validate
    }

    /// Subnet epoch data
    ///
    /// # Fields
    ///
    /// * `subnet_epoch` - The subnet epoch
    /// * `subnet_epoch_progression` - The subnet epoch progression as a percentage using 1e18 as 1.0
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct SubnetEpochData {
        pub subnet_epoch: u32,
        pub subnet_epoch_progression: u128,
    }

    /// This struct represents the breakdown of how total subnet rewards are allocated
    /// across different participants in the subnet ecosystem. It is generated during
    /// the `calculate_rewards` process and provides a complete view of reward distribution
    /// for a single epoch.
    ///
    /// # Fields
    ///
    /// * `overall_subnet_reward` - The total rewards allocated to the entire subnet for
    ///   this epoch. This is the sum of all other reward components and represents the
    ///   complete reward pool before any distribution.
    /// * `subnet_owner_reward` - The portion of rewards allocated to the subnet owner.
    ///   This is calculated first and taken from the overall subnet reward as compensation
    ///   for creating and maintaining the subnet.
    /// * `subnet_rewards` - The remaining rewards for the subnet after the owner's portion
    ///   has been deducted. This amount is further divided between delegate stakers and
    ///   subnet nodes. Equals `overall_subnet_reward - subnet_owner_reward`.
    /// * `delegate_stake_rewards` - The portion of subnet rewards designated for distribution
    ///   to delegate stakers. These are participants who have staked tokens to support the
    ///   subnet and share in its rewards.
    /// * `subnet_node_rewards` - The portion of subnet rewards allocated for distribution
    ///   among all active subnet nodes based on their consensus scores. This represents
    ///   the reward pool that will be split according to node performance.
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct RewardsData {
        pub overall_subnet_reward: u128,
        pub subnet_owner_reward: u128,
        pub subnet_rewards: u128,
        pub delegate_stake_rewards: u128,
        pub subnet_node_rewards: u128,
    }

    // Overwatch nodes

    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct OverwatchNodeInfo<AccountId> {
        pub overwatch_node_id: u32,
        pub coldkey: AccountId,
        pub hotkey: AccountId,
        pub peer_ids: Vec<PeerId>,
        pub reputation: Reputation,
    }

    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        PartialOrd,
        Ord,
        scale_info::TypeInfo,
    )]
    pub struct OverwatchNode<AccountId> {
        pub id: u32,
        pub hotkey: AccountId,
    }

    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        PartialOrd,
        Ord,
        scale_info::TypeInfo,
    )]
    pub struct OverwatchCommit<Hash> {
        pub subnet_id: u32,
        pub weight: Hash,
    }

    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        PartialOrd,
        Ord,
        scale_info::TypeInfo,
    )]
    pub struct OverwatchReveal {
        pub subnet_id: u32,
        pub weight: u128,
        pub salt: Vec<u8>,
    }

    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct Reputation {
        /// Epoch when the node first elected subnet validator node to submit consensus.
        pub start_epoch: u32,

        /// Current reputation weight.
        pub score: u128,

        /// Track total nodes under a coldkey ever, this can only increase.
        pub lifetime_node_count: u32,

        /// Track total nodes under a coldkey.
        pub total_active_nodes: u32,

        /// Number of times the node's weight increased (i.e., successful validation).
        pub total_increases: u32,

        /// Number of times the node's weight decreased (i.e., failed validation).
        pub total_decreases: u32,

        /// Average attestation rate.
        pub average_attestation: u128,

        /// Last epoch the node was selected as validator.
        pub last_validator_epoch: u32,

        /// Current overwatch node reputation weight.
        pub ow_score: u128,
    }

    struct WeightAccumulator<T: Config> {
        total_reads: u64,
        total_writes: u64,
        computational_ops: u64,
        _phantom: core::marker::PhantomData<T>,
    }

    impl<T: Config> WeightAccumulator<T> {
        fn new() -> Self {
            Self {
                total_reads: 0,
                total_writes: 0,
                computational_ops: 0,
                _phantom: Default::default(),
            }
        }

        fn add_computational_weight(&mut self, ops: u64) {
            self.computational_ops += ops;
        }

        fn add_clear_prefix(&mut self, removed_count: u32) {
            self.total_writes += removed_count as u64;
        }

        fn add_remove(&mut self) {
            self.total_writes += 1;
        }

        fn add_take(&mut self) {
            self.total_reads += 1;
            self.total_writes += 1;
        }

        fn add_mutate(&mut self) {
            self.total_reads += 1;
            self.total_writes += 1;
        }

        fn add_reads(&mut self, count: u64) {
            self.total_reads += count;
        }

        fn add_writes(&mut self, count: u64) {
            self.total_writes += count;
        }

        fn finalize(self) -> Weight {
            let mut weight = Weight::zero();
            weight = weight.saturating_add(T::DbWeight::get().reads(self.total_reads));
            weight = weight.saturating_add(T::DbWeight::get().writes(self.total_writes));
            // Add computational overhead
            weight = weight
                .saturating_add(Weight::from_parts(1000, 0).saturating_mul(self.computational_ops));
            weight
        }
    }

    /// RPC helper for getting subnet bootnodes
    ///
    /// bootnodes: List of official subnet bootnodes
    /// node_bootnodes: List of all node bootnodes
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct AllSubnetBootnodes {
        pub bootnodes: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>,
        pub node_bootnodes: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>,
    }

    ///
    ///
    #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
    pub struct DistributionData {
        pub validator_emissions: u128,
        pub weights: BTreeMap<u32, u128>,
    }

    /// This type value is referenced in:
    /// - PreviousSubnetPauseEpoch
    /// - PrevSubnetActivationEpoch
    /// - TotalNodes
    /// - TotalActiveNodes
    /// - LastSubnetDelegateStakeRewardsUpdate
    /// - SubnetNodeConsecutiveIncludedEpochs
    /// - PeerIdSubnetNodeId
    /// - BootnodePeerIdSubnetNodeId
    /// - ClientPeerIdSubnetNodeId
    /// - BootnodeSubnetNodeId
    /// - UniqueParamSubnetNodeId
    /// - TotalOverwatchNodes
    /// - TotalOverwatchNodeUids
    /// - PeerIdOverwatchNodeId
    /// - LastTxBlock
    #[pallet::type_value]
    pub fn DefaultZeroU32() -> u32 {
        0
    }
    #[pallet::type_value]
    pub fn DefaultZeroU64() -> u64 {
        0
    }
    /// This type value is referenced in:
    /// - SubnetNodeMinWeightDecreaseReputationThreshold
    /// - AccountSubnetStake
    /// - AccountSubnetDelegateStakeShares
    /// - TotalNodeDelegateStakeShares
    /// - TotalNodeDelegateStakeBalance
    /// - AccountOverwatchStake
    #[pallet::type_value]
    pub fn DefaultZeroU128() -> u128 {
        0
    }
    /// This type value is referenced in:
    /// - SubnetNodeReputation
    #[pallet::type_value]
    pub fn DefaultPercentageFactorU128() -> u128 {
        1_000_000_000_000_000_000
    }
    #[pallet::type_value]
    pub fn DefaultHalfPercentageFactorU128() -> u128 {
        1_000_000_000_000_000_000
    }
    /// This type value is referenced in:
    /// - ColdkeyIdentityNameOwner
    /// - HotkeyOwner
    #[pallet::type_value]
    pub fn DefaultAccountId<T: Config>() -> T::AccountId {
        T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap()
    }
    #[pallet::type_value]
    pub fn DefaultEmptyVec() -> Vec<u8> {
        Vec::new()
    }
    #[pallet::type_value]
    pub fn DefaultPeerId() -> PeerId {
        PeerId(Vec::new())
    }
    /// This type value is referenced in:
    /// - TxRateLimit
    #[pallet::type_value]
    pub fn DefaultTxRateLimit<T: Config>() -> u32 {
        T::InitialTxRateLimit::get()
    }
    #[pallet::type_value]
    pub fn DefaultBoolTrue() -> bool {
        true
    }
    /// This type value is referenced in:
    /// - TxPause
    /// - OverwatchNodeBlacklist
    #[pallet::type_value]
    pub fn DefaultBoolFalse() -> bool {
        false
    }
    /// This type value is referenced in:
    /// - SubnetNodesData
    /// - RegisteredSubnetNodesData
    #[pallet::type_value]
    pub fn DefaultSubnetNode<T: Config>() -> SubnetNode<T::AccountId> {
        return SubnetNode {
            id: 0,
            hotkey: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
            peer_id: PeerId(Vec::new()),
            bootnode_peer_id: PeerId(Vec::new()),
            client_peer_id: PeerId(Vec::new()),
            bootnode: None,
            classification: SubnetNodeClassification {
                node_class: SubnetNodeClass::Registered,
                start_epoch: 0,
            },
            delegate_reward_rate: 0,
            last_delegate_reward_rate_update: 0,
            unique: None,
            non_unique: None,
        };
    }
    /// This type value is referenced in:
    /// - MaxSubnetNodes
    #[pallet::type_value]
    pub fn DefaultMaxSubnetNodes() -> u32 {
        256
    }
    /// This type value is referenced in:
    /// - SubnetMinStakeBalance
    #[pallet::type_value]
    pub fn DefaultSubnetMinStakeBalance() -> u128 {
        100e+18 as u128
    }
    /// This type value is referenced in:
    /// - NetworkMaxStakeBalance
    /// - SubnetMaxStakeBalance
    #[pallet::type_value]
    pub fn DefaultNetworkMaxStakeBalance() -> u128 {
        1000e+18 as u128
    }
    /// This type value is referenced in:
    /// - MinActiveNodeStakeEpochs
    #[pallet::type_value]
    pub fn DefaultMinActiveNodeStakeEpochs<T: Config>() -> u32 {
        // 1 day
        T::EpochsPerYear::get() / 365
    }
    /// This type value is referenced in:
    /// - MinSubnetDelegateStakeFactor
    #[pallet::type_value]
    pub fn DefaultMinSubnetDelegateStakeFactor() -> u128 {
        // 0.1%
        1_000_000_000_000_000 // 1e18
    }
    /// This type value is referenced in:
    /// - MinDelegateStakeDeposit
    #[pallet::type_value]
    pub fn DefaultMinDelegateStakeDeposit() -> u128 {
        1000
    }
    /// This type value is referenced in:
    /// - SubnetDelegateStakeRewardsPercentage
    #[pallet::type_value]
    pub fn DefaultDelegateStakeRewardsPercentage() -> u128 {
        // 10.0%
        100000000000000000
    }
    /// This type value is referenced in:
    /// - MaxSubnetDelegateStakeRewardsPercentageChange
    #[pallet::type_value]
    pub fn DefaultMaxSubnetDelegateStakeRewardsPercentageChange() -> u128 {
        // 2%
        20000000000000000
    }
    /// This type value is referenced in:
    /// - SubnetDelegateStakeRewardsUpdatePeriod
    #[pallet::type_value]
    pub fn DefaultSubnetDelegateStakeRewardsUpdatePeriod() -> u32 {
        // 1 day at 6 seconds a block (86,000s per day)
        14400
    }
    /// This type value is referenced in:
    /// - StakeUnbondingLedger
    #[pallet::type_value]
    pub fn DefaultStakeUnbondingLedger() -> BTreeMap<u32, u128> {
        // {
        // 	block: u32, // cooldown begin epoch (+ cooldown duration for unlock)
        // 	balance: u128,
        // }
        BTreeMap::new()
    }
    /// This type value is referenced in:
    /// - BaseValidatorReward
    #[pallet::type_value]
    pub fn DefaultBaseValidatorReward() -> u128 {
        1e+18 as u128
    }
    /// This type value is referenced in:
    /// - ValidatorRewardK
    #[pallet::type_value]
    pub fn DefaultValidatorRewardK() -> u64 {
        20
    }
    #[pallet::type_value]
    pub fn DefaultValidatorRewardMidpoint() -> u128 {
        // 50.0%
        500000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultAttestorRewardExponent() -> u64 {
        10
    }
    #[pallet::type_value]
    pub fn DefaultAttestorMinRewardFactor() -> u128 {
        // 33.0%
        330000000000000000
    }
    /// This type value is referenced in:
    /// - BaseSlashPercentage
    #[pallet::type_value]
    pub fn DefaultBaseSlashPercentage() -> u128 {
        // 3.125%
        31250000000000000
    }
    /// This type value is referenced in:
    /// - MaxSlashAmount
    #[pallet::type_value]
    pub fn DefaultMaxSlashAmount() -> u128 {
        1e+18 as u128
    }
    /// This type value is referenced in:
    /// - MinAttestationPercentage
    #[pallet::type_value]
    pub fn DefaultMinAttestationPercentage() -> u128 {
        // 2/3
        660000000000000000
    }
    /// This type value is referenced in:
    /// - SuperMajorityAttestationRatio
    #[pallet::type_value]
    pub fn DefaultSuperMajorityAttestationRatio() -> u128 {
        // 7/8
        875000000000000000
    }
    /// This type value is referenced in:
    /// - MinSubnetNodes
    #[pallet::type_value]
    pub fn DefaultMinSubnetNodes() -> u32 {
        3
    }
    /// This type value is referenced in:
    /// - SubnetRegistrationEpochs
    #[pallet::type_value]
    pub fn DefaultSubnetRegistrationEpochs<T: Config>() -> u32 {
        T::EpochsPerYear::get() / 52
    }
    /// This type value is referenced in:
    /// - MinSubnetRegistrationEpochs
    #[pallet::type_value]
    pub fn DefaultMinSubnetRegistrationEpochs<T: Config>() -> u32 {
        T::EpochsPerYear::get() / 365
    }
    /// This type value is referenced in:
    /// - SubnetEnactmentEpochs
    #[pallet::type_value]
    pub fn DefaultSubnetEnactmentEpochs<T: Config>() -> u32 {
        T::EpochsPerYear::get() / 104
    }
    /// This type value is referenced in:
    /// - MaxSubnets
    #[pallet::type_value]
    pub fn DefaultMaxSubnets() -> u32 {
        64
    }
    /// This type value is referenced in:
    /// - MaxBootnodes
    #[pallet::type_value]
    pub fn DefaultMaxBootnodes() -> u32 {
        32
    }
    /// This type value is referenced in:
    /// - MaxSubnetBootnodeAccess
    #[pallet::type_value]
    pub fn DefaultMaxSubnetBootnodeAccess() -> u32 {
        21
    }
    /// This type value is referenced in:
    /// - SubnetBootnodes
    /// - BootnodeSubnetNodeId
    /// - UniqueParamSubnetNodeId
    #[pallet::type_value]
    pub fn DefaultMaxVectorLength() -> u32 {
        1024
    }
    /// This type value is referenced in:
    /// - register_or_update_identity
    #[pallet::type_value]
    pub fn DefaultMaxUrlLength() -> u32 {
        1024
    }
    /// This type value is referenced in:
    /// - register_or_update_identity
    #[pallet::type_value]
    pub fn DefaultMaxSocialIdLength() -> u32 {
        255
    }
    /// This type value is referenced in:
    /// - propose_attestation
    /// - attest
    #[pallet::type_value]
    pub fn DefaultValidatorArgsLimit() -> u32 {
        4096
    }
    /// This type value is referenced in:
    /// - SubnetOwnerPercentage
    #[pallet::type_value]
    pub fn DefaultSubnetOwnerPercentage() -> u128 {
        // 23%
        230000000000000000
    }
    /// This type value is referenced in:
    /// - InflationSigmoidMidpoint
    #[pallet::type_value]
    pub fn DefaultSigmoidMidpoint() -> u128 {
        // 50.0%
        500000000000000000
    }
    /// This type value is referenced in:
    /// - InflationSigmoidSteepness
    #[pallet::type_value]
    pub fn DefaultSigmoidSteepness() -> u128 {
        7
    }
    /// This type value is referenced in:
    /// - ChurnLimit
    #[pallet::type_value]
    pub fn DefaultChurnLimit() -> u32 {
        4
    }
    #[pallet::type_value]
    pub fn DefaultChurnLimitMultiplier() -> u32 {
        1
    }
    /// This type value is referenced in:
    /// - MinChurnLimit
    #[pallet::type_value]
    pub fn DefaultMinChurnLimit() -> u32 {
        // Must allow at least one node activation per epoch
        1
    }
    /// This type value is referenced in:
    /// - MaxChurnLimit
    #[pallet::type_value]
    pub fn DefaultMaxChurnLimit() -> u32 {
        // Must only enable up to 64 node activations per epoch
        64
    }
    #[pallet::type_value]
    pub fn DefaultMinChurnLimitMultiplier() -> u32 {
        1
    }
    #[pallet::type_value]
    pub fn DefaultMaxChurnLimitMultiplier<T: Config>() -> u32 {
        // Multiplier must allow at least one node per week
        T::EpochsPerYear::get() / 52
    }
    /// This type value is referenced in:
    /// - MaxSubnetPauseEpochs
    #[pallet::type_value]
    pub fn DefaultMaxSubnetPauseEpochs<T: Config>() -> u32 {
        // 3 days
        T::EpochsPerYear::get() / 120
    }
    /// This type value is referenced in:
    /// - MinQueueEpochs
    /// - SubnetNodeQueueEpochs
    /// - QueueImmunityEpochs
    #[pallet::type_value]
    pub fn DefaultMinRegistrationQueueEpochs() -> u32 {
        // Require at least one epoch in registration queue
        4
    }
    /// This type value is referenced in:
    /// - MaxQueueEpochs
    #[pallet::type_value]
    pub fn DefaultMaxRegistrationQueueEpochs<T: Config>() -> u32 {
        // Max queue of 1 month
        T::EpochsPerYear::get() / 12
    }
    /// This type value is referenced in:
    /// - MinIdleClassificationEpochs
    #[pallet::type_value]
    pub fn DefaultMinIdleClassificationEpochs() -> u32 {
        // Require at least one epoch in the queue classification
        1
    }
    /// This type value is referenced in:
    /// - MaxIdleClassificationEpochs
    #[pallet::type_value]
    pub fn DefaultMaxIdleClassificationEpochs<T: Config>() -> u32 {
        // Max queue classification of one week
        T::EpochsPerYear::get() / 52
    }
    /// This type value is referenced in:
    /// - MinIncludedClassificationEpochs
    #[pallet::type_value]
    pub fn DefaultMinIncludedClassificationEpochs() -> u32 {
        // Require at least one epoch in the included classification
        1
    }
    /// This type value is referenced in:
    /// - MaxIncludedClassificationEpochs
    #[pallet::type_value]
    pub fn DefaultMaxIncludedClassificationEpochs<T: Config>() -> u32 {
        // Max queue classification of one week
        T::EpochsPerYear::get() / 52
    }
    /// This type value is referenced in:
    /// - MinSubnetNodeConsecutiveIncludedEpochs
    #[pallet::type_value]
    pub fn DefaultMinSubnetNodeConsecutiveIncludedEpochs() -> u32 {
        // Require at least 3 epoch in the included classification
        3
    }
    /// This type value is referenced in:
    /// - MinSubnetNodeConsecutiveIncludedEpochs
    #[pallet::type_value]
    pub fn DefaultMaxSubnetNodeConsecutiveIncludedEpochs() -> u32 {
        // Require at least 3 epoch in the included classification
        16
    }
    /// This type value is referenced in:
    /// - MinSubnetMinStake
    #[pallet::type_value]
    pub fn DefaultMinSubnetMinStake() -> u128 {
        100e+18 as u128
    }
    /// This type value is referenced in:
    /// - MaxSubnetMinStake
    #[pallet::type_value]
    pub fn DefaultMaxSubnetMinStake() -> u128 {
        250e+18 as u128
    }
    /// This type value is referenced in:
    /// - MinDelegateStakePercentage
    #[pallet::type_value]
    pub fn DefaultMinDelegateStakePercentage() -> u128 {
        // 5.0%
        50000000000000000
    }
    /// This type value is referenced in:
    /// - MaxDelegateStakePercentage
    #[pallet::type_value]
    pub fn DefaultMaxDelegateStakePercentage() -> u128 {
        // 95.0%
        950000000000000000
    }
    /// This type value is referenced in:
    /// - MinMaxRegisteredNodes
    #[pallet::type_value]
    pub fn DefaultMinMaxRegisteredNodes() -> u32 {
        1
    }
    /// This type value is referenced in:
    /// - MaxMaxRegisteredNodes
    #[pallet::type_value]
    pub fn DefaultMaxMaxRegisteredNodes() -> u32 {
        64
    }
    /// This type value is referenced in:
    /// - IdleClassificationEpochs
    #[pallet::type_value]
    pub fn DefaultIdleClassificationEpochs() -> u32 {
        4
    }
    /// This type value is referenced in:
    /// - IncludedClassificationEpochs
    #[pallet::type_value]
    pub fn DefaultIncludedClassificationEpochs() -> u32 {
        7
    }
    /// This type value is referenced in:
    /// - NodeRewardRateUpdatePeriod
    #[pallet::type_value]
    pub fn DefaultNodeRewardRateUpdatePeriod() -> u32 {
        // 1 day at 6 seconds a block (86,000s per day)
        14400
    }
    /// This type value is referenced in:
    /// - MaxRewardRateDecrease
    #[pallet::type_value]
    pub fn DefaultMaxRewardRateDecrease() -> u128 {
        // 1%
        10_000_000
    }
    /// This type value is referenced in:
    /// - SubnetDistributionPower
    #[pallet::type_value]
    pub fn DefaultSubnetDistributionPower() -> u128 {
        // 0.75
        750000000000000000
    }
    /// This type value is referenced in:
    /// - ColdkeyReputationIncreaseFactor
    #[pallet::type_value]
    pub fn DefaultColdkeyReputationIncreaseFactor() -> u128 {
        // 0.5
        500000000000000
    }
    /// This type value is referenced in:
    /// - ColdkeyReputationDecreaseFactor
    #[pallet::type_value]
    pub fn DefaultColdkeyReputationDecreaseFactor() -> u128 {
        // 50%
        500000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMinSubnetNodeReputation() -> u128 {
        // 10%
        100000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultAbsentDecreaseReputationFactor() -> u128 {
        // 10%
        100000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultIncludedIncreaseReputationFactor() -> u128 {
        // 10%
        100000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultBelowMinWeightDecreaseReputationFactor() -> u128 {
        // 10%
        100000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultNonAttestorDecreaseReputationFactor() -> u128 {
        // 10%
        100000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultNonConsensusAttestorDecreaseReputationFactor() -> u128 {
        // 10%
        100000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultValidatorAbsentSubnetNodeReputationFactor() -> u128 {
        // 5%
        50000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultValidatorNonConsensusSubnetNodeReputationFactor() -> u128 {
        // 5%
        50000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMinNodeReputationFactor() -> u128 {
        // 5.0%
        50000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMaxNodeReputationFactor() -> u128 {
        // 50%
        500000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMinMinSubnetNodeReputation() -> u128 {
        // 5.0%
        50000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMinNonConsensusAttestorDecreaseReputationFactorn() -> u128 {
        // 50%
        500000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMaxNonConsensusAttestorDecreaseReputationFactor() -> u128 {
        // 5.0%
        50000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMaxMinSubnetNodeReputation() -> u128 {
        // 50%
        500000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMinSubnetReputation() -> u128 {
        // 10%
        100000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultNotInConsensusSubnetReputationFactor() -> u128 {
        // 12.5%
        125000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMaxPauseEpochsSubnetReputationFactor() -> u128 {
        // 12.5%
        125000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultLessThanMinNodesSubnetReputationFactor() -> u128 {
        // 12.5%
        125000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultValidatorAbsentSubnetReputationFactor() -> u128 {
        // 5.0%
        50000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultInConsensusSubnetReputationFactor() -> u128 {
        // 12.5%
        125000000000000000
    }
    /// This type value is referenced in:
    /// - MaxSubnetRemovalInterval
    #[pallet::type_value]
    pub fn DefaultMaxSubnetRemovalInterval() -> u32 {
        10
    }
    /// This type value is referenced in:
    /// - MinSubnetRemovalInterval
    #[pallet::type_value]
    pub fn DefaultMinSubnetRemovalInterval() -> u32 {
        10
    }
    /// This type value is referenced in:
    /// - DelegateStakeSubnetRemovalInterval
    #[pallet::type_value]
    pub fn DefaultDelegateStakeSubnetRemovalInterval() -> u32 {
        2
    }
    /// This type value is referenced in:
    /// - ColdkeyReputation
    #[pallet::type_value]
    pub fn DefaultColdkeyReputation() -> Reputation {
        return Reputation {
            start_epoch: 0,
            score: 500_000_000_000_000_000, // 0.5 / 50%
            lifetime_node_count: 0,
            total_active_nodes: 0,
            total_increases: 0,
            total_decreases: 0,
            average_attestation: 0,
            last_validator_epoch: 0,
            ow_score: 500_000_000_000_000, // 0.5 / 50%
        };
    }
    /// This type value is referenced in:
    /// - ColdkeyIdentity
    #[pallet::type_value]
    pub fn DefaultColdkeyIdentity() -> ColdkeyIdentityData {
        return ColdkeyIdentityData {
            name: BoundedVec::new(),
            url: BoundedVec::new(),
            image: BoundedVec::new(),
            discord: BoundedVec::new(),
            x: BoundedVec::new(),
            telegram: BoundedVec::new(),
            github: BoundedVec::new(),
            hugging_face: BoundedVec::new(),
            description: BoundedVec::new(),
            misc: BoundedVec::new(),
        };
    }
    /// This type value is referenced in:
    /// - MaxOverwatchNodes
    #[pallet::type_value]
    pub fn DefaultMaxOverwatchNodes() -> u32 {
        64
    }
    /// This type value is referenced in:
    /// - OverwatchEpochLengthMultiplier
    #[pallet::type_value]
    pub fn DefaultOverwatchEpochLengthMultiplier() -> u32 {
        16
    }
    /// This type value is referenced in:
    /// - OverwatchCommitCutoffPercent
    #[pallet::type_value]
    pub fn DefaultOverwatchCommitCutoffPercent() -> u128 {
        // 80%
        800000000000000000
    }
    /// This type value is referenced in:
    /// - OverwatchStakeWeightFactor
    #[pallet::type_value]
    pub fn DefaultOverwatchStakeWeightFactor() -> u128 {
        900000000000000000
    }
    /// This type value is referenced in:
    /// - OverwatchMinStakeBalance
    #[pallet::type_value]
    pub fn DefaultOverwatchMinStakeBalance() -> u128 {
        100e+18 as u128
    }
    /// This type value is referenced in:
    /// - OverwatchMinDiversificationRatio
    #[pallet::type_value]
    pub fn DefaultOverwatchMinDiversificationRatio() -> u128 {
        // 25%
        250000000000000000
    }
    /// This type value is referenced in:
    /// - OverwatchMinRepScore
    #[pallet::type_value]
    pub fn DefaultOverwatchMinRepScore() -> u128 {
        // 75%
        750000000000000000
    }
    /// This type value is referenced in:
    /// - OverwatchMinAvgAttestationRatio
    #[pallet::type_value]
    pub fn DefaultOverwatchMinAvgAttestationRatio() -> u128 {
        // 75%
        720000000000000000
    }
    /// This type value is referenced in:
    /// - OverwatchMinAge
    #[pallet::type_value]
    pub fn DefaultOverwatchMinAge<T: Config>() -> u32 {
        // ~3 months
        T::EpochLength::get() / 4
    }
    /// This type value is referenced in:
    /// - MaxMinDelegateStakeMultiplier
    #[pallet::type_value]
    pub fn DefaulMaxMinDelegateStakeMultiplier() -> u128 {
        // 400%
        4000000000000000000
    }
    /// This type value is referenced in:
    /// - DelegateStakeWeightFactor
    #[pallet::type_value]
    pub fn DefaultDelegateStakeWeightFactor() -> u128 {
        // 40.0%
        400000000000000000
    }
    /// This type value is referenced in:
    /// - DelegateStakeCooldownEpochs
    #[pallet::type_value]
    pub fn DefaultDelegateStakeCooldownEpochs() -> u32 {
        1
    }
    /// This type value is referenced in:
    /// - NodeDelegateStakeCooldownEpochs
    #[pallet::type_value]
    pub fn DefaultNodeDelegateStakeCooldownEpochs() -> u32 {
        1
    }
    /// This type value is referenced in:
    /// - StakeCooldownEpochs
    #[pallet::type_value]
    pub fn DefaultStakeCooldownEpochs() -> u32 {
        4
    }
    /// This type value is referenced in:
    /// - MaxUnbondings
    #[pallet::type_value]
    pub fn DefaultMaxUnbondings() -> u32 {
        32
    }
    /// This type value is referenced in:
    /// - CurrentNodeBurnRate
    /// - MinNodeBurnRate
    #[pallet::type_value]
    pub fn DefaultCurrentNodeBurnRate() -> u128 {
        // 100%
        1000000000000000000
    }
    /// This type value is referenced in:
    /// - MaxNodeBurnRate
    #[pallet::type_value]
    pub fn DefaultMaxNodeBurnRate() -> u128 {
        // 500%
        5_000_000_000_000_000_000
    }
    /// This type value is referenced in:
    /// - BaseNodeBurnAmount
    #[pallet::type_value]
    pub fn DefaulBaseNodeBurnAmount() -> u128 {
        // 0.00001
        10000000000000
    }
    /// This type value is referenced in:
    /// - NodeBurnRateAlpha
    #[pallet::type_value]
    pub fn DefaultNodeBurnRateAlpha() -> u128 {
        // 50%
        500_000_000_000_000_000
    }
    /// This type value is referenced in:
    /// - SubnetPauseCooldownEpochs
    #[pallet::type_value]
    pub fn DefaultSubnetPausePeriodDelta<T: Config>() -> u32 {
        // 1 month / 30 days
        T::EpochsPerYear::get() / 12
    }
    /// This type value is referenced in:
    /// - LastRegistrationCost
    #[pallet::type_value]
    pub fn DefaultLastRegistrationCost() -> u128 {
        1000000000000000000
    }
    /// This type value is referenced in:
    /// - MinRegistrationCost
    #[pallet::type_value]
    pub fn DefaultMinRegistrationCost() -> u128 {
        // Always should be less than `LastRegistrationCost`
        100000000000000000
    }
    /// This type value is referenced in:
    /// - RegistrationCostDecayBlocks
    #[pallet::type_value]
    pub fn DefaultRegistrationCostDecayBlocks() -> u32 {
        // 3 months
        // 1314871
        // 1 months
        438290
    }
    /// This type value is referenced in:
    /// - RegistrationCostAlpha
    #[pallet::type_value]
    pub fn DefaultRegistrationCostAlpha() -> u128 {
        // 50%
        500000000000000000
    }
    /// This type value is referenced in:
    /// - NewRegistrationCostMultiplier
    #[pallet::type_value]
    pub fn DefaultNewRegistrationCostMultiplier() -> u128 {
        // 200%
        2000000000000000000
    }
    /// This type value is referenced in:
    /// - MaxSwapQueueCallsPerBlock
    #[pallet::type_value]
    pub fn DefaultMaxSwapQueueCallsPerBlock() -> u32 {
        16
    }
    /// This type value is referenced in:
    /// - MaximumHooksWeightV2
    #[pallet::type_value]
    pub fn DefaultMaximumHooksWeightV2<T: Config>() -> Weight {
        sp_runtime::Perbill::from_percent(50) * T::BlockWeights::get().max_block
    }
    /// This type value is referenced in:
    /// - MaxSubnetNodeMinWeightDecreaseReputationThreshold
    #[pallet::type_value]
    pub fn DefaultMaxSubnetNodeMinWeightDecreaseReputationThreshold() -> u128 {
        100000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMaxEmergencyValidatorEpochsMultiplier() -> u128 {
        // 200%, doubles the target seudo fork epochs
        2000000000000000000
    }
    #[pallet::type_value]
    pub fn DefaultMaxEmergencySubnetNodes() -> u32 {
        64
    }

    //
    // Subnet elements
    //

    /// Count of subnets
    #[pallet::storage]
    pub type TotalSubnetUids<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    pub type SubnetNetFlow<T: Config> = StorageMap<_, Identity, u32, i128, ValueQuery>;

    /// For informational purposes only, not used in conditinal logic
    /// Subnet Id -> Friendly UID
    #[pallet::storage] // subnet_id => count
    pub type SubnetIdFriendlyUid<T> = StorageMap<_, Identity, u32, u32, OptionQuery>;

    /// For informational purposes only, not used in conditinal logic
    /// Friendly UID -> Subnet Id
    #[pallet::storage] // subnet_id => count
    pub type FriendlyUidSubnetId<T> = StorageMap<_, Identity, u32, u32, OptionQuery>;

    /// Count of active subnets
    #[pallet::storage]
    pub type TotalActiveSubnets<T> = StorageValue<_, u32, ValueQuery>;

    /// Mapping of each subnet stored by ID, uniqued by `SubnetName`
    /// Stores subnet data by a unique id
    #[pallet::storage] // subnet_id => data struct
    pub type SubnetsData<T> = StorageMap<_, Identity, u32, SubnetData>;

    /// Epoch subnet registered on
    #[pallet::storage] // subnet_id => blocks
    pub type SubnetRegistrationEpoch<T> = StorageMap<_, Identity, u32, u32>;

    /// Owner of subnet (defaulted to registerer of subnet)
    #[pallet::storage] // subnet_id => AccountId
    pub type SubnetOwner<T: Config> = StorageMap<_, Identity, u32, T::AccountId>;

    /// Minimum time between subnet pauses
    #[pallet::storage]
    pub type SubnetPauseCooldownEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultSubnetPausePeriodDelta<T>>;

    /// Most recent re-activation (from paused)
    #[pallet::storage]
    pub type PreviousSubnetPauseEpoch<T> =
        StorageMap<_, Identity, u32, u32, ValueQuery, DefaultZeroU32>;

    /// Most recent epoch a subnet was activated on
    /// Used to calculate subnet removal intervals
    #[pallet::storage]
    pub type PrevSubnetActivationEpoch<T> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

    //
    // Registration fees
    //

    /// The last fee paid to register a subnet
    /// Default is 1.0, also the starting fee on genesis
    #[pallet::storage]
    pub type LastRegistrationCost<T> =
        StorageValue<_, u128, ValueQuery, DefaultLastRegistrationCost>;

    /// The minimum subnet registration fee
    #[pallet::storage]
    pub type MinRegistrationCost<T> = StorageValue<_, u128, ValueQuery, DefaultMinRegistrationCost>;

    /// Last block the price was updated
    #[pallet::storage]
    pub type LastSubnetRegistrationBlock<T> = StorageValue<_, u32, ValueQuery>;

    //
    // Subnet slots
    //

    /// Maps subnet_id => slot (block offset in epoch)
    #[pallet::storage]
    pub type SubnetSlot<T> = StorageMap<_, Identity, u32, u32, OptionQuery>;

    /// Reverse: slot => subnet_id (to track occupied slots)
    #[pallet::storage]
    pub type SlotAssignment<T> = StorageMap<_, Identity, u32, u32, OptionQuery>;

    /// Keep track of currently assigned slots for quick lookup
    #[pallet::storage]
    pub type AssignedSlots<T> = StorageValue<_, BTreeSet<u32>, ValueQuery>;

    //
    // Subnet node elements
    //

    /// Election slots
    /// List of every subnet node ID that can be elected as validator
    #[pallet::storage]
    pub type SubnetNodeElectionSlots<T> = StorageMap<_, Identity, u32, Vec<u32>, ValueQuery>;

    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        PartialOrd,
        Ord,
        scale_info::TypeInfo,
    )]
    pub struct EmergencySubnetValidatorData {
        pub subnet_node_ids: Vec<u32>,
        pub target_emergency_validators_epochs: u32,
        pub max_emergency_validators_epoch: u32,
        pub total_epochs: u32,
    }

    #[pallet::storage]
    pub type EmergencySubnetNodeElectionData<T> =
        StorageMap<_, Identity, u32, EmergencySubnetValidatorData, OptionQuery>;

    #[pallet::storage]
    pub type MaxEmergencySubnetNodes<T> =
        StorageValue<_, u32, ValueQuery, DefaultMaxEmergencySubnetNodes>;

    /// The multiplier used to get the EmergencySubnetValidatorData.max_emergency_validators_epoch
    /// Example: `current subnet epoch + target_emergency_validators_epochs * MaxEmergencyValidatorEpochsMultiplier`
    #[pallet::storage]
    pub type MaxEmergencyValidatorEpochsMultiplier<T> =
        StorageValue<_, u128, ValueQuery, DefaultMaxEmergencyValidatorEpochsMultiplier>;

    /// Subnet count of electable nodes
    #[pallet::storage]
    pub type TotalSubnetElectableNodes<T> = StorageMap<_, Identity, u32, u32, ValueQuery>;

    /// Network wide count of electable nodes
    #[pallet::storage]
    pub type TotalElectableNodes<T> = StorageValue<_, u32, ValueQuery>;

    /// Track election slots to avoid iterating
    #[pallet::storage]
    pub type NodeSlotIndex<T> = StorageDoubleMap<
        _,
        Identity,
        u32, // subnet_id
        Identity,
        u32, // node_id
        u32, // index in slot vec
        OptionQuery,
    >;

    /// Count of nodes in the network
    #[pallet::storage]
    pub type TotalNodes<T> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

    /// Count of active nodes in the network (non-registered nodes)
    #[pallet::storage]
    pub type TotalActiveNodes<T> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

    /// Count of total nodes in a subnet
    #[pallet::storage] // subnet_uid --> u32
    #[pallet::getter(fn total_subnet_nodes)]
    pub type TotalSubnetNodes<T> = StorageMap<_, Identity, u32, u32, ValueQuery>;

    /// Count of active nodes in a subnet
    #[pallet::storage] // subnet_uid --> u32
    pub type TotalActiveSubnetNodes<T> = StorageMap<_, Identity, u32, u32, ValueQuery>;

    // ============================================
    // Collective
    // ============================================

    /// The blocks until the fee decays to `MinRegistrationCost`
    #[pallet::storage]
    pub type RegistrationCostDecayBlocks<T> =
        StorageValue<_, u32, ValueQuery, DefaultRegistrationCostDecayBlocks>;

    #[pallet::storage]
    pub type RegistrationCostAlpha<T> =
        StorageValue<_, u128, ValueQuery, DefaultRegistrationCostAlpha>;

    /// The multiplier applied to the next subnet registration fee from the current fee
    #[pallet::storage]
    pub type NewRegistrationCostMultiplier<T> =
        StorageValue<_, u128, ValueQuery, DefaultNewRegistrationCostMultiplier>;

    /// Max epochs a subnet can be in the pause state for
    #[pallet::storage]
    pub type MaxSubnetPauseEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultMaxSubnetPauseEpochs<T>>;

    /// Number of epoch post PrevSubnetActivationEpoch to be able to remove a subnet
    /// Allows newly activated subnets time to increase delegate stake
    #[pallet::storage]
    pub type MinSubnetRemovalInterval<T> =
        StorageValue<_, u32, ValueQuery, DefaultMinSubnetRemovalInterval>;

    /// Count of every epochs we attempt to remove a subnet if there are > max subnets
    #[pallet::storage]
    pub type MaxSubnetRemovalInterval<T> =
        StorageValue<_, u32, ValueQuery, DefaultMaxSubnetRemovalInterval>;

    /// Count of every epochs we attempt to remove a subnet if there are > max subnets
    #[pallet::storage]
    pub type DelegateStakeSubnetRemovalInterval<T> =
        StorageValue<_, u32, ValueQuery, DefaultDelegateStakeSubnetRemovalInterval>;

    /// Subnet registration epochs
    /// This is the total epochs in the registration period before the enactment period
    /// In this period, users can register subnet nodes and delegate stake
    #[pallet::storage]
    pub type SubnetRegistrationEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultSubnetRegistrationEpochs<T>>;

    /// The minimum epochs a subnet much be registered for
    #[pallet::storage]
    pub type MinSubnetRegistrationEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultMinSubnetRegistrationEpochs<T>>;

    /// Time period allowable for subnet activation following registration period
    /// In this period, users can no longer register subnet nodes (and can continue to delegate stake)
    #[pallet::storage]
    pub type SubnetEnactmentEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultSubnetEnactmentEpochs<T>>;

    /// Max number of accounts that can access updating the main bootnodes for a subnet
    #[pallet::storage]
    pub type MaxSubnetBootnodeAccess<T> =
        StorageValue<_, u32, ValueQuery, DefaultMaxSubnetBootnodeAccess>;

    /// Max subnets in the network
    #[pallet::storage]
    #[pallet::getter(fn max_subnets)]
    pub type MaxSubnets<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnets>;

    /// Max bootnodes for a subnet to manage
    #[pallet::storage]
    pub type MaxBootnodes<T> = StorageValue<_, u32, ValueQuery, DefaultMaxBootnodes>;

    /// Minimum amount of nodes required per subnet
    /// required for subnet activity
    #[pallet::storage]
    pub type MinSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMinSubnetNodes>;

    /// Maximim nodes in a subnet at any given time
    #[pallet::storage]
    #[pallet::getter(fn max_subnet_nodes)]
    pub type MaxSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnetNodes>;

    /// Maximum minimum delegate stake multiplier for getting the minimum delegate stake balance required for a subnet
    #[pallet::storage]
    pub type MaxMinDelegateStakeMultiplier<T> =
        StorageValue<_, u128, ValueQuery, DefaulMaxMinDelegateStakeMultiplier>;

    /// Min ChurnLimit a subnet can set
    #[pallet::storage]
    pub type MinChurnLimit<T> = StorageValue<_, u32, ValueQuery, DefaultMinChurnLimit>;

    /// Max ChurnLimit a subnet can set
    #[pallet::storage]
    pub type MaxChurnLimit<T> = StorageValue<_, u32, ValueQuery, DefaultMaxChurnLimit>;

    #[pallet::storage]
    pub type MinChurnLimitMultiplier<T> =
        StorageValue<_, u32, ValueQuery, DefaultMinChurnLimitMultiplier>;

    #[pallet::storage]
    pub type MaxChurnLimitMultiplier<T> =
        StorageValue<_, u32, ValueQuery, DefaultMaxChurnLimitMultiplier<T>>;

    //
    // Registered classification
    //

    /// Min SubnetNodeQueueEpochs a subnet can set
    #[pallet::storage]
    pub type MinQueueEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultMinRegistrationQueueEpochs>;

    /// Max SubnetNodeQueueEpochs a subnet can set
    #[pallet::storage]
    pub type MaxQueueEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultMaxRegistrationQueueEpochs<T>>;

    //
    // Idle classification
    //

    /// Min IdleClassificationEpochs a subnet can set
    #[pallet::storage]
    pub type MinIdleClassificationEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultMinIdleClassificationEpochs>;

    /// Max IdleClassificationEpochs a subnet can set
    #[pallet::storage]
    pub type MaxIdleClassificationEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultMaxIdleClassificationEpochs<T>>;

    //
    // Included classification
    //

    /// Min IncludedClassificationEpochs a subnet can set
    #[pallet::storage]
    pub type MinIncludedClassificationEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultMinIncludedClassificationEpochs>;

    /// Max IncludedClassificationEpochs a subnet can set
    #[pallet::storage]
    pub type MaxIncludedClassificationEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultMaxIncludedClassificationEpochs<T>>;

    #[pallet::storage]
    pub type MaxSubnetNodeMinWeightDecreaseReputationThreshold<T> =
        StorageValue<_, u128, ValueQuery, DefaultMaxSubnetNodeMinWeightDecreaseReputationThreshold>;

    //
    // Subnet stake parameters
    //

    /// Min value the SubnetMinStake a subnet can set
    #[pallet::storage]
    pub type MinSubnetMinStake<T> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetMinStake>;

    /// Max value the SubnetMinStake a subnet can set
    #[pallet::storage]
    pub type MaxSubnetMinStake<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSubnetMinStake>;

    /// Network maximum stake balance per Subnet Node
    /// A subnet staker can have greater than the max stake balance.
    /// Subnets can't set max stake above this value
    #[pallet::storage]
    pub type NetworkMaxStakeBalance<T> =
        StorageValue<_, u128, ValueQuery, DefaultNetworkMaxStakeBalance>;

    /// The MinDelegateStakePercentage a subnet can set
    #[pallet::storage]
    pub type MinDelegateStakePercentage<T> =
        StorageValue<_, u128, ValueQuery, DefaultMinDelegateStakePercentage>;

    /// The maximum value the delegate stake percentage can be
    /// i.e., the percentage of rewards that can go to delegate stakers
    /// This is used universally for subnet delegate stake (updated by owners), and node delegate stake (updated by nodes)
    #[pallet::storage]
    pub type MaxDelegateStakePercentage<T> =
        StorageValue<_, u128, ValueQuery, DefaultMaxDelegateStakePercentage>;

    /// Minimum MaxRegisteredNodes for a given subnet (controlled by owner)
    #[pallet::storage]
    pub type MinMaxRegisteredNodes<T> =
        StorageValue<_, u32, ValueQuery, DefaultMinMaxRegisteredNodes>;

    /// Maximum MaxRegisteredNodes for a given subnet (controlled by owner)
    #[pallet::storage]
    pub type MaxMaxRegisteredNodes<T> =
        StorageValue<_, u32, ValueQuery, DefaultMaxMaxRegisteredNodes>;

    /// Maximum delegate stake rate change bidirectionally, by subnet owner
    #[pallet::storage]
    pub type MaxSubnetDelegateStakeRewardsPercentageChange<T> =
        StorageValue<_, u128, ValueQuery, DefaultMaxSubnetDelegateStakeRewardsPercentageChange>;

    /// Minimum blocks between delegate stake reward updates
    #[pallet::storage]
    pub type SubnetDelegateStakeRewardsUpdatePeriod<T> =
        StorageValue<_, u32, ValueQuery, DefaultSubnetDelegateStakeRewardsUpdatePeriod>;

    /// Cooldown epochs for unstaking from a delegate stake position to unstake from the unbonding ledger
    #[pallet::storage]
    pub type DelegateStakeCooldownEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultDelegateStakeCooldownEpochs>;

    /// Cooldown epochs for unstaking from a node delegate stake position to unstake from the unbonding ledger
    #[pallet::storage]
    pub type NodeDelegateStakeCooldownEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultNodeDelegateStakeCooldownEpochs>;

    /// Cooldown epochs for unstaking as a node to unstake from the unbonding ledger
    #[pallet::storage]
    pub type StakeCooldownEpochs<T> = StorageValue<_, u32, ValueQuery, DefaultStakeCooldownEpochs>;

    /// Maximum unbondings in the unbonding ledger per account
    #[pallet::storage]
    pub type MaxUnbondings<T> = StorageValue<_, u32, ValueQuery, DefaultMaxUnbondings>;

    //
    // Rewards (validator, incentives)
    //

    /// Base reward per epoch for validators
    /// This is the base reward to subnet validators on successful attestation
    #[pallet::storage]
    pub type BaseValidatorReward<T> = StorageValue<_, u128, ValueQuery, DefaultBaseValidatorReward>;

    /// Sigmoid steepness for getting factor of what percentage of the base reward a validator receives
    /// Used in `get_validator_reward_multiplier`
    #[pallet::storage]
    pub type ValidatorRewardK<T> = StorageValue<_, u64, ValueQuery, DefaultValidatorRewardK>;

    /// Sigmoid midpoint for getting factor of what percentage of the base reward a validator receives
    /// Used in `get_validator_reward_multiplier`
    #[pallet::storage]
    pub type ValidatorRewardMidpoint<T> =
        StorageValue<_, u128, ValueQuery, DefaultValidatorRewardMidpoint>;

    #[pallet::storage]
    pub type AttestorRewardExponent<T> =
        StorageValue<_, u64, ValueQuery, DefaultAttestorRewardExponent>;

    #[pallet::storage]
    pub type AttestorMinRewardFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultAttestorMinRewardFactor>;

    /// Base percentage slash is based off of
    #[pallet::storage]
    pub type BaseSlashPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultBaseSlashPercentage>;

    /// Maximum amount any validator can be slashed
    #[pallet::storage]
    pub type MaxSlashAmount<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSlashAmount>;

    //
    // Weight helpers
    //

    /// The power before normalizing subnet weights for distributing emissions to subnets
    #[pallet::storage]
    pub type SubnetDistributionPower<T> =
        StorageValue<_, u128, ValueQuery, DefaultSubnetDistributionPower>;

    /// Factor used (0-1.0) for getting a subnet weight between delegate stake and electable node count
    /// see `calculate_subnet_weights`
    #[pallet::storage]
    pub type DelegateStakeWeightFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultDelegateStakeWeightFactor>;

    #[derive(
        Default,
        Encode,
        Decode,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        PartialOrd,
        Ord,
        scale_info::TypeInfo,
    )]
    pub struct SubnetWeightFactorsData {
        pub delegate_stake: u128,
        pub node_count: u128,
        pub net_flow: u128,
    }

    #[pallet::type_value]
    pub fn DefaultSubnetWeightFactors() -> SubnetWeightFactorsData {
        return SubnetWeightFactorsData {
            delegate_stake: 400000000000000000,
            node_count: 400000000000000000,
            net_flow: 200000000000000000,
        };
    }

    #[pallet::storage]
    pub type SubnetWeightFactors<T: Config> =
        StorageValue<_, SubnetWeightFactorsData, ValueQuery, DefaultSubnetWeightFactors>;
    //
    // Inflation helpers elements
    //

    /// Inflation grpah midpoint (sigmoid)
    #[pallet::storage]
    pub type InflationSigmoidMidpoint<T> =
        StorageValue<_, u128, ValueQuery, DefaultSigmoidMidpoint>;

    /// Inflation grpah midpoint (sigmoid)
    #[pallet::storage]
    pub type InflationSigmoidSteepness<T> =
        StorageValue<_, u128, ValueQuery, DefaultSigmoidSteepness>;

    //
    // Subnet owner
    //

    /// Percentage of rewards that allocates to subnet owners
    #[pallet::storage] // subnet_id => AccountId
    pub type SubnetOwnerPercentage<T> =
        StorageValue<_, u128, ValueQuery, DefaultSubnetOwnerPercentage>;

    // ============================================
    // Owner
    // ============================================

    /// Pending owner of subnet
    #[pallet::storage] // subnet_id => AccountId
    pub type PendingSubnetOwner<T: Config> = StorageMap<_, Identity, u32, T::AccountId>;

    /// List of bootnodes updated by the subnet owner
    #[pallet::storage]
    pub type SubnetBootnodes<T> =
        StorageMap<_, Identity, u32, BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>, ValueQuery>;

    /// Set of accounts that have access to add bootnodes
    #[pallet::storage]
    pub type SubnetBootnodeAccess<T: Config> =
        StorageMap<_, Identity, u32, BTreeSet<T::AccountId>, ValueQuery>;

    /// Ensures no duplicate subnet paths within the network at one time
    /// If a subnet name is voted out, it can be voted up later on and any
    /// stakes attached to the subnet_id won't impact the re-initialization
    /// of the subnet name.
    #[pallet::storage]
    #[pallet::getter(fn subnet_name)]
    pub type SubnetName<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32>;

    /// Repository to subnet codebase
    #[pallet::storage]
    #[pallet::getter(fn subnet_repo)]
    pub type SubnetRepo<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32>;

    /// Max amount of nodes that can activate per epoch
    #[pallet::storage] // subnet_uid --> u32
    pub type ChurnLimit<T> = StorageMap<_, Identity, u32, u32, ValueQuery, DefaultChurnLimit>;

    /// The epoch multiplier for the ChurnLimit
    /// If the churn limit is 4 and the multiplier is 2, 4 nodes can be activated every 4 epochs
    /// If the churn limit is 4 and the multiplier is 1, 4 nodes can be activated every 1 epoch
    #[pallet::storage]
    pub type ChurnLimitMultiplier<T> =
        StorageMap<_, Identity, u32, u32, ValueQuery, DefaultChurnLimitMultiplier>;

    /// Length of epochs a node must be in the registration queue before they can activate
    /// Note: Registration classification epochs
    #[pallet::storage] // subnet_uid --> u32
    pub type SubnetNodeQueueEpochs<T> =
        StorageMap<_, Identity, u32, u32, ValueQuery, DefaultMinRegistrationQueueEpochs>;

    /// Length of epochs a Idle classified node must be in that class for
    #[pallet::storage] // subnet_uid --> u32
    pub type IdleClassificationEpochs<T> =
        StorageMap<_, Identity, u32, u32, ValueQuery, DefaultIdleClassificationEpochs>;

    /// Length of epochs an Included classified node must be consecutively in that class for
    /// This can be used in tandem with SubnetNodeReputation to ensure a node is included
    /// in consensus data before they are activated instead of automatically being upgraded
    /// to validator. (see rewards.rs to see how a node is upgraded to the Validator class)
    #[pallet::storage] // subnet_uid --> u32
    pub type IncludedClassificationEpochs<T> =
        StorageMap<_, Identity, u32, u32, ValueQuery, DefaultIncludedClassificationEpochs>;

    /// Count of epochs an Idle node has been active in this class
    /// subnet_id --> uid --> count of epochs in a row
    #[pallet::storage]
    pub type SubnetNodeIdleConsecutiveEpochs<T> =
        StorageDoubleMap<_, Identity, u32, Identity, u32, u32, ValueQuery, DefaultZeroU32>;

    /// Count of epochs in a row an Included node has been in consensus
    /// subnet_id --> uid --> count of epochs in a row
    #[pallet::storage]
    pub type SubnetNodeConsecutiveIncludedEpochs<T> =
        StorageDoubleMap<_, Identity, u32, Identity, u32, u32, ValueQuery, DefaultZeroU32>;

    /// Immunity period for queued nodes to not be removed from queue
    /// See `remove_queue_node_id` parameter in `propose_attestation`
    #[pallet::storage]
    pub type QueueImmunityEpochs<T: Config> =
        StorageMap<_, Identity, u32, u32, ValueQuery, DefaultMinRegistrationQueueEpochs>;

    /// Whitelist of coldkeys that nodes can register to a subnet during its registration period
    /// Afterwards on subnet activation, this list is deleted and the subnet is now public
    /// Because all subnets are expected to be P2P, each subnet starts as a blockchain would with
    /// trusting nodes to ensure no malicious nodes can enter at the start.
    /// u32 is the number of nodes the coldkey can register while subnet is in registration
    /// Verified via InitialColdkeyData
    /// subnet_id => { AccountId, max registrations}
    #[pallet::storage]
    pub type SubnetRegistrationInitialColdkeys<T: Config> =
        StorageMap<_, Identity, u32, BTreeMap<T::AccountId, u32>>;

    /// Keytypes the subnet accepts, set by the subnet
    #[pallet::storage]
    pub type SubnetKeyTypes<T> = StorageMap<_, Identity, u32, BTreeSet<KeyType>, ValueQuery>;

    /// Min required stake balance for a Subnet Node in a specified subnet
    #[pallet::storage]
    pub type SubnetMinStakeBalance<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultSubnetMinStakeBalance>;

    /// Max stake balance for a Subnet Node in a specified subnet
    /// A node can go over this amount as a balance but cannot add more above it
    #[pallet::storage]
    pub type SubnetMaxStakeBalance<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultNetworkMaxStakeBalance>;

    #[pallet::storage]
    pub type SubnetDelegateStakeRewardsPercentage<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultDelegateStakeRewardsPercentage>;

    /// The last block the subent set SubnetDelegateStakeRewardsPercentage
    #[pallet::storage]
    pub type LastSubnetDelegateStakeRewardsUpdate<T> =
        StorageMap<_, Identity, u32, u32, ValueQuery, DefaultZeroU32>;

    /// Count of registrations per coldkey in SubnetRegistrationInitialColdkeys
    /// Removed when subnet is activated
    #[pallet::storage]
    pub type InitialColdkeyData<T: Config> =
        StorageMap<_, Identity, u32, BTreeMap<T::AccountId, u32>>;

    /// Maximum registered nodes in a subnet set by the subnet
    #[pallet::storage] // subnet_uid --> u32
    pub type MaxRegisteredNodes<T> =
        StorageMap<_, Identity, u32, u32, ValueQuery, DefaultMaxMaxRegisteredNodes>;

    /// The subnet node weight (score ratio) threshold where a node will get its reputation decreased
    /// i.e. if the nodes score (and in consensus) is under this value, it will decresae their reputation
    /// This is updated by the owner
    /// This can be logically unused if value is 0_u128
    #[pallet::storage]
    pub type SubnetNodeMinWeightDecreaseReputationThreshold<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultZeroU128>;

    /// Total subnet UIDs. Used to get each nodes UID
    #[pallet::storage] // subnet_id --> u32
    pub type TotalSubnetNodeUids<T> = StorageMap<_, Identity, u32, u32, ValueQuery>;

    // ====================
    // Nodes
    // ====================

    /// Coldkey => Public Identity
    #[pallet::storage]
    pub type ColdkeyIdentity<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ColdkeyIdentityData,
        ValueQuery,
        DefaultColdkeyIdentity,
    >;

    /// Owner of a coldkey identity name, used to keep names unique
    #[pallet::storage]
    pub type ColdkeyIdentityNameOwner<T: Config> =
        StorageMap<_, Blake2_128Concat, Vec<u8>, T::AccountId, ValueQuery, DefaultAccountId<T>>;

    /// Owner of a hotkey, Hotkey => Coldkey
    /// This is only removed if the node has a stake of 0 (not on node removal), see `do_remove_stake`
    #[pallet::storage]
    pub type HotkeyOwner<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        T::AccountId,
        ValueQuery,
        DefaultAccountId<T>,
    >;

    /// Hotkey => Subnet ID
    /// This is useful for updating hotkeys in `update_hotkey`
    /// This is only removed if the node has a stake of 0 (not on node removal), see `do_remove_stake`
    #[pallet::storage]
    pub type HotkeySubnetId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    /// Coldkey => {Hotkeys}
    /// This conditions unique hotkeys over the entire network and enables tracking hotkeys to coldkeys
    /// This is only removed if the node has a stake of 0 (not on node removal), see `do_remove_stake`
    #[pallet::storage]
    pub type ColdkeyHotkeys<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BTreeSet<T::AccountId>, ValueQuery>;

    /// Coldkey => {SID: SNID}
    /// This is used mainly for overwatch node qualification because it allows easily getting
    /// a count of how many unique subnets a coldkey is in.
    /// This is cleaned up on `perform_remove_subnet_node` and `clean_coldkey_subnet_nodes`
    #[pallet::storage]
    pub type ColdkeySubnetNodes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BTreeMap<u32, BTreeSet<u32>>, ValueQuery>;

    /// Mapping of each hotkeys subnet node ID in each subnet
    /// Subnet ID => Hotkey => Subnet Node ID
    #[pallet::storage]
    pub type HotkeySubnetNodeId<T: Config> =
        StorageDoubleMap<_, Identity, u32, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    /// Mapping of each subnet node ID's hotkey
    /// Subnet ID => Subnet Node ID => Hotkey
    #[pallet::storage]
    pub type SubnetNodeIdHotkey<T: Config> =
        StorageDoubleMap<_, Identity, u32, Identity, u32, T::AccountId, OptionQuery>;

    /// Mapping of subnet nodes
    /// subnet_id --> uid --> data
    #[pallet::storage]
    pub type SubnetNodesData<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u32,
        Identity,
        u32,
        SubnetNode<T::AccountId>,
        ValueQuery,
        DefaultSubnetNode<T>,
    >;

    /// Subnets that are registered, not yet activated, are stored here before activation
    /// This is used to allow nodes in the queue (registered) to update itself
    #[pallet::storage] // subnet_id --> uid --> data
    pub type RegisteredSubnetNodesData<T: Config> = StorageDoubleMap<
        _,
        Identity,
        u32,
        Identity,
        u32,
        SubnetNode<T::AccountId>,
        ValueQuery,
        DefaultSubnetNode<T>,
    >;

    /// Subnet node queue to be activated
    /// This if for registered nodes
    #[pallet::storage]
    pub type SubnetNodeQueue<T: Config> =
        StorageMap<_, Identity, u32, Vec<SubnetNode<T::AccountId>>, ValueQuery>;

    /// Each subnet nodes peer_id, conditions uniqueness
    /// subnet_id --> peer_id --> subnet_node_id
    #[pallet::storage]
    pub type PeerIdSubnetNodeId<T> = StorageDoubleMap<
        _,
        Identity,
        u32,
        Blake2_128Concat,
        PeerId,
        u32,
        ValueQuery,
        DefaultZeroU32,
    >;

    /// Each subnet nodes bootnode peer_id, conditions uniqueness
    /// subnet_id --> bootnode_peer_id --> subnet_node_id
    #[pallet::storage]
    pub type BootnodePeerIdSubnetNodeId<T> = StorageDoubleMap<
        _,
        Identity,
        u32,
        Blake2_128Concat,
        PeerId,
        u32,
        ValueQuery,
        DefaultZeroU32,
    >;

    /// Each subnet nodes client peer_id, conditions uniqueness
    /// subnet_id --> client_peer_id --> subnet_node_id
    #[pallet::storage]
    pub type ClientPeerIdSubnetNodeId<T> = StorageDoubleMap<
        _,
        Identity,
        u32,
        Blake2_128Concat,
        PeerId,
        u32,
        ValueQuery,
        DefaultZeroU32,
    >;

    /// Each subnet nodes bootnode peer_id, conditions uniqueness
    /// subnet_id --> bootnode --> subnet_node_id
    #[pallet::storage]
    pub type BootnodeSubnetNodeId<T> = StorageDoubleMap<
        _,
        Identity,
        u32,
        Blake2_128Concat,
        BoundedVec<u8, DefaultMaxVectorLength>,
        u32,
        ValueQuery,
        DefaultZeroU32,
    >;

    /// Used for unique parameters
    #[pallet::storage] // subnet_id --> param --> node ID
    pub type UniqueParamSubnetNodeId<T> = StorageDoubleMap<
        _,
        Identity,
        u32,
        Blake2_128Concat,
        BoundedVec<u8, DefaultMaxVectorLength>,
        u32,
        ValueQuery,
        DefaultZeroU32,
    >;

    //
    // Node burn
    //

    /// Subnet ID => current rate
    /// Set each epoch
    #[pallet::storage]
    pub type CurrentNodeBurnRate<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultCurrentNodeBurnRate>;

    /// Base network burn amount
    /// Base burn amount (what 100% rate represents)
    #[pallet::storage]
    #[pallet::getter(fn base_burn_amount)]
    pub type BaseNodeBurnAmount<T: Config> =
        StorageValue<_, u128, ValueQuery, DefaulBaseNodeBurnAmount>;

    /// Minimum burn rate as a percentage (using 1e18 precision)
    /// e.g., 1e18 = 100% minimum rate
    #[pallet::storage]
    pub type MinNodeBurnRate<T: Config> =
        StorageValue<_, u128, ValueQuery, DefaultCurrentNodeBurnRate>;

    /// Maximum burn rate as a percentage (using 1e18 precision)  
    /// e.g., 5e18 = 500% maximum rate
    #[pallet::storage]
    pub type MaxNodeBurnRate<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMaxNodeBurnRate>;

    /// Alpha value for burn rate adjustment (using 1e18 precision)
    /// Higher alpha = faster adjustment to target
    /// Subnet ID => alpha (set by subnet owner)
    #[pallet::storage]
    pub type NodeBurnRateAlpha<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultNodeBurnRateAlpha>;

    /// Target number of registrations per epoch
    /// Subnet ID => alpha (set by subnet owner)
    #[pallet::storage]
    pub type TargetNodeRegistrationsPerEpoch<T> =
        StorageMap<_, Identity, u32, u32, ValueQuery, DefaultMaxMaxRegisteredNodes>;

    /// Count of registrations in current epoch
    #[pallet::storage]
    pub type NodeRegistrationsThisEpoch<T> = StorageMap<_, Identity, u32, u32, ValueQuery>;

    //
    // Network utility elements
    //

    #[pallet::storage] // ( tx_rate_limit )
    pub type TxRateLimit<T> = StorageValue<_, u32, ValueQuery, DefaultTxRateLimit<T>>;

    /// Last transaction on rate limited functions
    #[pallet::storage] // key --> last_block
    pub type LastTxBlock<T: Config> =
        StorageMap<_, Identity, T::AccountId, u32, ValueQuery, DefaultZeroU32>;

    /// Pause the network
    #[pallet::storage]
    pub type TxPause<T> = StorageValue<_, bool, ValueQuery, DefaultBoolFalse>;

    //
    // Validate / Attestation
    //

    // subnet ID => epoch  => Subnet Node ID
    #[pallet::storage]
    pub type SubnetElectedValidator<T> =
        StorageDoubleMap<_, Identity, u32, Identity, u32, u32, OptionQuery>;

    /// Consensus submissions (attestation proposals by elected validator)
    #[pallet::storage] // subnet ID => epoch  => data
    pub type SubnetConsensusSubmission<T: Config> =
        StorageDoubleMap<_, Identity, u32, Identity, u32, ConsensusData<T::AccountId>>;

    /// Minimum attestation ratio to form consensus
    #[pallet::storage]
    pub type MinAttestationPercentage<T> =
        StorageValue<_, u128, ValueQuery, DefaultMinAttestationPercentage>;

    /// Minimum attestation ratio for mechanisms that require super majority
    #[pallet::storage]
    pub type SuperMajorityAttestationRatio<T> =
        StorageValue<_, u128, ValueQuery, DefaultSuperMajorityAttestationRatio>;

    /// Epoch -> {total_issuance, (subnet_id, weight)}
    /// Set each epoch
    #[pallet::storage]
    pub type FinalSubnetEmissionWeights<T> =
        StorageMap<_, Identity, u32, DistributionData, ValueQuery>;

    //
    // Reputation
    //

    // Node Reputation

    /// Maximum value for the factor that decreases or increases a nodes reputation (set by collective)
    #[pallet::storage]
    pub type MinNodeReputationFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultMinNodeReputationFactor>;

    /// Maximum value for the factor that decreases or increases a nodes reputation (set by collective)
    #[pallet::storage]
    pub type MaxNodeReputationFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultMaxNodeReputationFactor>;

    /// Maximum value for a node to be removed based on its `SubnetNodeReputation` (set by collective)
    #[pallet::storage]
    pub type MinMinSubnetNodeReputation<T> =
        StorageValue<_, u128, ValueQuery, DefaultMinMinSubnetNodeReputation>;

    /// Maximum value for a node to be removed based on its `SubnetNodeReputation` (set by collective)
    #[pallet::storage]
    pub type MaxMinSubnetNodeReputation<T> =
        StorageValue<_, u128, ValueQuery, DefaultMaxMinSubnetNodeReputation>;

    /// Node reputation factor when a node is absent for decreasing node reputation (set by subnet owner)
    #[pallet::storage]
    pub type MinSubnetNodeReputation<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultMinSubnetNodeReputation>;

    #[pallet::storage]
    pub type SubnetNodeReputation<T> = StorageDoubleMap<
        _,
        Identity,
        u32, // subnet ID
        Identity,
        u32,  // subnet node ID
        u128, // Reputation
        ValueQuery,
        DefaultPercentageFactorU128,
    >;

    /// Node reputation factor when a node is absent from consensus for decreasing node reputation
    #[pallet::storage]
    pub type AbsentDecreaseReputationFactor<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultAbsentDecreaseReputationFactor>;

    /// Node reputation factor when a node is included in consensus data for increasing node reputation
    #[pallet::storage]
    pub type IncludedIncreaseReputationFactor<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultIncludedIncreaseReputationFactor>;

    /// Node reputation factor when a node is below minimum weight for decreasing node reputation
    #[pallet::storage]
    pub type BelowMinWeightDecreaseReputationFactor<T> = StorageMap<
        _,
        Identity,
        u32,
        u128,
        ValueQuery,
        DefaultBelowMinWeightDecreaseReputationFactor,
    >;

    /// Node reputation factor when a node hasn't attested in vast majority consensus for decreasing node reputation
    #[pallet::storage]
    pub type NonAttestorDecreaseReputationFactor<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultNonAttestorDecreaseReputationFactor>;

    /// Node reputation factor when a node hasn't attested in vast majority consensus for decreasing node reputation
    #[pallet::storage]
    pub type NonConsensusAttestorDecreaseReputationFactor<T> = StorageMap<
        _,
        Identity,
        u32,
        u128,
        ValueQuery,
        DefaultNonConsensusAttestorDecreaseReputationFactor,
    >;

    /// Node reputation factor when an elected node hasn't proposed
    #[pallet::storage]
    pub type ValidatorAbsentSubnetNodeReputationFactor<T> = StorageMap<
        _,
        Identity,
        u32,
        u128,
        ValueQuery,
        DefaultValidatorAbsentSubnetNodeReputationFactor,
    >;

    /// Node reputation factor when an elected node is not in consensus
    /// This is applied against the attestation delta as `factor * (1.0 - attestation ratio / min attestation ratio)`
    #[pallet::storage]
    pub type ValidatorNonConsensusSubnetNodeReputationFactor<T> = StorageMap<
        _,
        Identity,
        u32,
        u128,
        ValueQuery,
        DefaultValidatorNonConsensusSubnetNodeReputationFactor,
    >;

    // Subnet Reputation

    #[pallet::storage]
    pub type SubnetReputation<T> =
        StorageMap<_, Identity, u32, u128, ValueQuery, DefaultPercentageFactorU128>;

    #[pallet::storage]
    pub type MinSubnetReputation<T> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetReputation>;

    /// Subnet reputation factor when a subnet is not in consensus (set by collective)
    #[pallet::storage]
    pub type NotInConsensusSubnetReputationFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultNotInConsensusSubnetReputationFactor>;

    /// Subnet reputation factor when a paused subnet is passed the max pause epochs (set by collective)
    #[pallet::storage]
    pub type MaxPauseEpochsSubnetReputationFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultMaxPauseEpochsSubnetReputationFactor>;

    /// Subnet reputation factor when a subnet has less than the required minimum electable nodes (set by collective)
    #[pallet::storage]
    pub type LessThanMinNodesSubnetReputationFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultLessThanMinNodesSubnetReputationFactor>;

    /// Subnet reputation factor when a subnet validator node doesn't submit consensus data (set by collective)
    #[pallet::storage]
    pub type ValidatorAbsentSubnetReputationFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultValidatorAbsentSubnetReputationFactor>;

    /// Subnet reputation factor when a subnet validator node doesn't submit consensus data (set by collective)
    #[pallet::storage]
    pub type InConsensusSubnetReputationFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultInConsensusSubnetReputationFactor>;

    // Coldkey Reputation (used for Overwatch Nodes)

    /// Weight used to increase a subnet validator nodes reputation
    #[pallet::storage]
    pub type ColdkeyReputationIncreaseFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultColdkeyReputationIncreaseFactor>;

    /// Weight used to decrease a subnet validator nodes reputation
    #[pallet::storage]
    pub type ColdkeyReputationDecreaseFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultColdkeyReputationDecreaseFactor>;

    /// Tracks a coldkeys reputation using numerous data points
    #[pallet::storage]
    pub type ColdkeyReputation<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Reputation,
        ValueQuery,
        DefaultColdkeyReputation,
    >;

    //
    // Staking
    //

    #[pallet::storage] // ( total_stake )
    #[pallet::getter(fn total_stake)]
    pub type TotalStake<T> = StorageValue<_, u128, ValueQuery>;

    /// Total stake sum of all nodes in specified subnet
    #[pallet::storage] // subnet_uid --> peer_data
    #[pallet::getter(fn total_subnet_stake)]
    pub type TotalSubnetStake<T> = StorageMap<_, Identity, u32, u128, ValueQuery>;

    /// An accounts stake per subnet
    #[pallet::storage] // account--> subnet_id --> u128
    #[pallet::getter(fn account_subnet_stake)]
    pub type AccountSubnetStake<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        u32,
        u128,
        ValueQuery,
        DefaultZeroU128,
    >;

    /// account => { block: balance }
    #[pallet::storage]
    pub type StakeUnbondingLedger<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BTreeMap<u32, u128>,
        ValueQuery,
        DefaultStakeUnbondingLedger,
    >;

    /// The number of epochs a node must stay staked as a node from its start_epoch
    /// This only applies to activated nodes
    /// If a node never activates, they can unstake under the standard cooldown period
    #[pallet::storage]
    pub type MinActiveNodeStakeEpochs<T> =
        StorageValue<_, u32, ValueQuery, DefaultMinActiveNodeStakeEpochs<T>>;

    //
    // Delegate Staking
    //

    /// Minimum delegate stake balance for all subnets as a factor
    /// Measured against the total network supply as a percentage
    /// Used to result the subnet minimum delegate stake balance
    #[pallet::storage]
    pub type MinSubnetDelegateStakeFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultMinSubnetDelegateStakeFactor>;

    /// Min delegate stake deposit amount
    /// Mitigates against inflation attacks
    #[pallet::storage]
    pub type MinDelegateStakeDeposit<T> =
        StorageValue<_, u128, ValueQuery, DefaultMinDelegateStakeDeposit>;

    /// Total network delegate stake balance
    #[pallet::storage]
    #[pallet::getter(fn total_delegate_stake)]
    pub type TotalDelegateStake<T> = StorageValue<_, u128, ValueQuery>;

    /// Total stake sum of all nodes in specified subnet
    #[pallet::storage] // subnet_uid --> u128
    pub type TotalSubnetDelegateStakeShares<T> = StorageMap<_, Identity, u32, u128, ValueQuery>;

    /// Total stake sum of all nodes in specified subnet
    #[pallet::storage] // subnet_uid --> u128
    pub type TotalSubnetDelegateStakeBalance<T> = StorageMap<_, Identity, u32, u128, ValueQuery>;

    /// An accounts delegate stake sharesper subnet
    #[pallet::storage] // account --> subnet_id --> u128
    pub type AccountSubnetDelegateStakeShares<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Identity,
        u32,
        u128,
        ValueQuery,
        DefaultZeroU128,
    >;

    //
    // Node Delegate Stake
    //

    /// Time between Subnet Node updating node delegate staking rate
    #[pallet::storage]
    pub type NodeRewardRateUpdatePeriod<T> =
        StorageValue<_, u32, ValueQuery, DefaultNodeRewardRateUpdatePeriod>;

    /// Max nominal percentage decrease of Subnet Node delegate reward rate
    #[pallet::storage]
    pub type MaxRewardRateDecrease<T> =
        StorageValue<_, u128, ValueQuery, DefaultMaxRewardRateDecrease>;

    /// Total network node delegate stake balance
    #[pallet::storage]
    pub type TotalNodeDelegateStake<T> = StorageValue<_, u128, ValueQuery>;

    /// Total stake sum of shares in specified Subnet Node
    /// subnet_id -> subnet node ID -> shares
    #[pallet::storage]
    pub type TotalNodeDelegateStakeShares<T> =
        StorageDoubleMap<_, Identity, u32, Identity, u32, u128, ValueQuery, DefaultZeroU128>;

    /// Total stake sum of balance in specified Subnet Node
    /// subnet_id -> subnet node ID -> balance
    #[pallet::storage]
    pub type TotalNodeDelegateStakeBalance<T> =
        StorageDoubleMap<_, Identity, u32, Identity, u32, u128, ValueQuery, DefaultZeroU128>;

    /// Shares a user has under a node it delegate staked to
    /// account_id -> subnet_id -> subnet_node_id -> shares
    #[pallet::storage]
    pub type AccountNodeDelegateStakeShares<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>,
            NMapKey<Identity, u32>,
            NMapKey<Identity, u32>,
        ),
        u128,
        ValueQuery,
    >;

    //
    // Overwatch Nodes
    //

    /// If a coldkey is blacklisted from being an overwatch node
    #[pallet::storage]
    pub type OverwatchNodeBlacklist<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery, DefaultBoolFalse>;

    #[pallet::storage]
    pub type MaxOverwatchNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMaxOverwatchNodes>;

    #[pallet::storage]
    pub type TotalOverwatchNodes<T> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

    #[pallet::storage]
    pub type TotalOverwatchNodeUids<T> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

    /// Overwatch epoch multipler vs T::EpochLength
    /// i.e. Overwatch nodes submit data every /x/ epochs
    #[pallet::storage]
    pub type OverwatchEpochLengthMultiplier<T> =
        StorageValue<_, u32, ValueQuery, DefaultOverwatchEpochLengthMultiplier>;

    /// The percent progress of the overwatch interval where the node can:
    /// - no longer commit
    /// - can reveal
    /// i.e. Node can commit for 80% of the period, and reveal in the latter 20% of the period
    #[pallet::storage]
    pub type OverwatchCommitCutoffPercent<T> =
        StorageValue<_, u128, ValueQuery, DefaultOverwatchCommitCutoffPercent>;

    // Overwatch Node ID => OverwatchNode
    #[pallet::storage]
    pub type OverwatchNodes<T: Config> =
        StorageMap<_, Identity, u32, OverwatchNode<T::AccountId>, OptionQuery>;

    /// Mapping overwatch node ID to hotkey
    /// Overwatch node ID => Hotkey
    #[pallet::storage]
    pub type OverwatchNodeIdHotkey<T: Config> =
        StorageMap<_, Identity, u32, T::AccountId, OptionQuery>;

    /// Mapping overwatch hotkey to overwatch node ID
    /// Hotkey => Overewatch Node ID
    #[pallet::storage]
    pub type HotkeyOverwatchNodeId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    /// Mapping overwatch node peer IDs for each subnet
    /// subnet_id --> peer_id --> overwatch_node_id
    #[pallet::storage]
    pub type PeerIdOverwatchNodeId<T> = StorageDoubleMap<
        _,
        Identity,
        u32,
        Blake2_128Concat,
        PeerId,
        u32,
        ValueQuery,
        DefaultZeroU32,
    >;

    /// Overwatch node peer Ids mapping {subnet ID: PeerId}
    #[pallet::storage]
    pub type OverwatchNodeIndex<T> = StorageMap<
        _,
        Identity,
        u32, // overwatch_node_id
        BTreeMap<u32, PeerId>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type OverwatchCommits<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Identity, u32>, // Epoch
            NMapKey<Identity, u32>, // Overwatch ID
            NMapKey<Identity, u32>, // Subnet ID
        ),
        T::Hash, // Commit
        OptionQuery,
    >;

    #[pallet::storage]
    pub type OverwatchReveals<T> = StorageNMap<
        _,
        (
            NMapKey<Identity, u32>, // Epoch
            NMapKey<Identity, u32>, // Subnet ID
            NMapKey<Identity, u32>, // Overwatch ID
        ),
        u128, // Reveal
        OptionQuery,
    >;

    /// The percentage factor applied to the final overwatch weights for its impact on economic weights (dstake weight and node weight)
    /// Example: `weight = overwatch_weight * factor`
    #[pallet::storage]
    pub type OverwatchWeightFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultHalfPercentageFactorU128>;

    #[pallet::storage]
    pub type OverwatchStakeWeightFactor<T> =
        StorageValue<_, u128, ValueQuery, DefaultOverwatchStakeWeightFactor>;

    /// Finalized calculated subnet weights from overwatch nodes
    /// Epoch => Subnet ID => Weight
    #[pallet::storage]
    pub type OverwatchSubnetWeights<T> = StorageDoubleMap<
        _,
        Identity,
        u32, // Epoch
        Identity,
        u32,  // Subnet ID
        u128, // Weight
        OptionQuery,
    >;

    /// Overwatch node scores
    #[pallet::storage]
    pub type OverwatchNodeWeights<T> = StorageDoubleMap<
        _,
        Identity,
        u32, // Epoch
        Identity,
        u32,  // Node ID
        u128, // Weight
        OptionQuery,
    >;

    //
    // Overwatch reputation conditional requirements
    //

    /// The percentage of subnets a coldkey must be in  to become an Overwatch Node
    /// i.e. if there are 100 subnets and the ratio is 51%, they must be in at least 51 subnets as a subnet node
    #[pallet::storage]
    pub type OverwatchMinDiversificationRatio<T> =
        StorageValue<_, u128, ValueQuery, DefaultOverwatchMinDiversificationRatio>;

    /// The minimum coldkey reputation score
    #[pallet::storage]
    pub type OverwatchMinRepScore<T> =
        StorageValue<_, u128, ValueQuery, DefaultOverwatchMinRepScore>;

    /// The minimum coldkey reputation attestation ratio
    #[pallet::storage]
    pub type OverwatchMinAvgAttestationRatio<T> =
        StorageValue<_, u128, ValueQuery, DefaultOverwatchMinAvgAttestationRatio>;

    /// The minimum coldkey reputation time in network based on general blockchain epochs
    #[pallet::storage]
    pub type OverwatchMinAge<T> = StorageValue<_, u32, ValueQuery, DefaultOverwatchMinAge<T>>;

    //
    // Overwatch staking
    //

    #[pallet::storage]
    pub type TotalOverwatchStake<T> = StorageValue<_, u128, ValueQuery>;

    /// Overwatch hotkey stake balance
    #[pallet::storage] // subnet_uid --> peer_data
    pub type AccountOverwatchStake<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery, DefaultZeroU128>;

    #[pallet::storage]
    pub type OverwatchMinStakeBalance<T> =
        StorageValue<_, u128, ValueQuery, DefaultOverwatchMinStakeBalance>;

    //
    // Swap queue
    //

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub enum QueuedSwapCall<AccountId> {
        // swap_delegate_stake
        SwapToSubnetDelegateStake {
            account_id: AccountId,
            to_subnet_id: u32,
            balance: u128,
        },
        // swap_node_delegate_stake
        SwapToNodeDelegateStake {
            account_id: AccountId,
            to_subnet_id: u32,
            to_subnet_node_id: u32,
            balance: u128,
        },
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
    pub struct QueuedSwapItem<AccountId> {
        pub id: u32,
        pub call: QueuedSwapCall<AccountId>,
        pub queued_at_block: u32,
        pub execute_after_blocks: u32, // How many blocks to wait to execute
    }

    impl<AccountId> QueuedSwapCall<AccountId> {
        pub fn get_queue_balance(&self) -> u128 {
            match self {
                QueuedSwapCall::SwapToSubnetDelegateStake { balance, .. } => *balance,
                QueuedSwapCall::SwapToNodeDelegateStake { balance, .. } => *balance,
            }
        }
    }

    /// List of current swaps in order
    #[pallet::storage]
    pub type SwapQueueOrder<T> = StorageValue<_, BoundedVec<u32, ConstU32<1000>>, ValueQuery>;

    /// Queue to swap between nodes and subnet delegate staking
    #[pallet::storage]
    pub type SwapCallQueue<T: Config> = StorageMap<
        _,
        Identity,
        u32, // queue_id
        QueuedSwapItem<T::AccountId>,
        OptionQuery,
    >;

    /// Tracks queue swap IDs
    #[pallet::storage]
    pub type NextSwapQueueId<T> = StorageValue<_, u32, ValueQuery>;

    /// Maximum number of queued swap executions per block
    #[pallet::storage]
    pub type MaxSwapQueueCallsPerBlock<T> =
        StorageValue<_, u32, ValueQuery, DefaultMaxSwapQueueCallsPerBlock>;

    #[pallet::storage]
    pub type MaximumHooksWeightV2<T> =
        StorageValue<_, Weight, ValueQuery, DefaultMaximumHooksWeightV2<T>>;

    /// The pallet's dispatchable functions ([`Call`]s).
    ///
    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// They must always return a `DispatchResult` and be annotated with a weight and call index.
    ///
    /// The [`call_index`] macro is used to explicitly
    /// define an index for calls in the [`Call`] enum. This is useful for pallets that may
    /// introduce new dispatchables over time. If the order of a dispatchable changes, its index
    /// will also change which will break backwards compatibility.
    ///
    /// The [`weight`] macro is used to assign a weight to each call.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new subnet.
        ///
        /// # Arguments
        ///
        /// * `subnet_data` - Subnet registration data `RegistrationSubnetData`.
        ///
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn register_subnet(
            origin: OriginFor<T>,
            max_cost: u128,
            subnet_data: RegistrationSubnetData<T::AccountId>,
        ) -> DispatchResult {
            let owner: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            Self::do_register_subnet(owner, max_cost, subnet_data)
        }

        /// Try activation a registered subnet.
        ///
        /// # Requirements
        ///
        /// * Can be either the coldkey or hotkey of the owner
        /// - The owner is responsible for activating to ensure all subnet functionalities are a-go
        ///
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID assigned on registration.
        /// * `subnet_node_id` - Subnet node ID of activator.
        ///
        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn activate_subnet(origin: OriginFor<T>, subnet_id: u32) -> DispatchResultWithPostInfo {
            let coldkey: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            ensure!(
                Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
                Error::<T>::NotSubnetOwner
            );

            Self::do_activate_subnet(subnet_id)
        }

        #[pallet::call_index(2)]
        #[pallet::weight({0})]
        pub fn owner_pause_subnet(origin: OriginFor<T>, subnet_id: u32) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_pause_subnet(origin, subnet_id)
        }

        #[pallet::call_index(3)]
        #[pallet::weight({0})]
        pub fn owner_unpause_subnet(origin: OriginFor<T>, subnet_id: u32) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_unpause_subnet(origin, subnet_id)
        }

        #[pallet::call_index(4)]
        #[pallet::weight({0})]
        pub fn owner_deactivate_subnet(origin: OriginFor<T>, subnet_id: u32) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_deactivate_subnet(origin, subnet_id)
        }

        #[pallet::call_index(5)]
        #[pallet::weight({0})]
        pub fn owner_update_name(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: Vec<u8>,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_name(origin, subnet_id, value)
        }

        #[pallet::call_index(6)]
        #[pallet::weight({0})]
        pub fn owner_update_repo(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: Vec<u8>,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_repo(origin, subnet_id, value)
        }

        #[pallet::call_index(7)]
        #[pallet::weight({0})]
        pub fn owner_update_description(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: Vec<u8>,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_description(origin, subnet_id, value)
        }

        #[pallet::call_index(8)]
        #[pallet::weight({0})]
        pub fn owner_update_misc(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: Vec<u8>,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_misc(origin, subnet_id, value)
        }

        #[pallet::call_index(9)]
        #[pallet::weight({0})]
        pub fn owner_update_churn_limit(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u32,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_churn_limit(origin, subnet_id, value)
        }

        #[pallet::call_index(10)]
        #[pallet::weight({0})]
        pub fn owner_update_registration_queue_epochs(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u32,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_registration_queue_epochs(origin, subnet_id, value)
        }

        #[pallet::call_index(11)]
        #[pallet::weight({0})]
        pub fn owner_update_idle_classification_epochs(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u32,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_idle_classification_epochs(origin, subnet_id, value)
        }

        #[pallet::call_index(12)]
        #[pallet::weight({0})]
        pub fn owner_update_included_classification_epochs(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u32,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_included_classification_epochs(origin, subnet_id, value)
        }

        #[pallet::call_index(13)]
        #[pallet::weight({0})]
        pub fn owner_update_non_consensus_attestor_decrease_reputation_factor(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_non_consensus_attestor_decrease_reputation_factor(
                origin, subnet_id, value,
            )
        }

        #[pallet::call_index(14)]
        #[pallet::weight({0})]
        pub fn owner_add_or_update_initial_coldkeys(
            origin: OriginFor<T>,
            subnet_id: u32,
            coldkeys: BTreeMap<T::AccountId, u32>,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_add_or_update_initial_coldkeys(origin, subnet_id, coldkeys)
        }

        #[pallet::call_index(15)]
        #[pallet::weight({0})]
        pub fn owner_remove_initial_coldkeys(
            origin: OriginFor<T>,
            subnet_id: u32,
            coldkeys: BTreeSet<T::AccountId>,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_remove_initial_coldkeys(origin, subnet_id, coldkeys)
        }

        #[pallet::call_index(16)]
        #[pallet::weight({0})]
        pub fn owner_update_key_types(
            origin: OriginFor<T>,
            subnet_id: u32,
            key_types: BTreeSet<KeyType>,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_key_types(origin, subnet_id, key_types)
        }

        #[pallet::call_index(17)]
        #[pallet::weight({0})]
        pub fn owner_set_emergency_validator_set(
            origin: OriginFor<T>,
            subnet_id: u32,
            mut subnet_node_ids: Vec<u32>,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_set_emergency_validator_set(origin, subnet_id, subnet_node_ids)
        }

        #[pallet::call_index(18)]
        #[pallet::weight({0})]
        pub fn owner_revert_emergency_validator_set(
            origin: OriginFor<T>,
            subnet_id: u32,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_revert_emergency_validator_set(origin, subnet_id)
        }

        #[pallet::call_index(19)]
        #[pallet::weight({0})]
        pub fn owner_update_min_max_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            min: u128,
            max: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_min_max_stake(origin, subnet_id, min, max)
        }

        #[pallet::call_index(20)]
        #[pallet::weight({0})]
        pub fn owner_update_delegate_stake_percentage(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_delegate_stake_percentage(origin, subnet_id, value)
        }

        #[pallet::call_index(21)]
        #[pallet::weight({0})]
        pub fn owner_update_max_registered_nodes(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u32,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_max_registered_nodes(origin, subnet_id, value)
        }

        /// Swap subnet owner
        ///
        /// - First step in a 2 step owner transfer, see `accept_subnet_ownership` for 2nd step
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `new_owner` - Account of new owner.
        ///
        /// # Requirements
        ///
        /// * Must be owner
        ///
        #[pallet::call_index(22)]
        #[pallet::weight({0})]
        pub fn transfer_subnet_ownership(
            origin: OriginFor<T>,
            subnet_id: u32,
            new_owner: T::AccountId,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_transfer_subnet_ownership(origin, subnet_id, new_owner)
        }

        /// Accept subnet owner transfer
        ///
        /// - Step 2 in subnet owner transfer, see `transfer_subnet_ownership` for step 1.
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        ///
        /// # Requirements
        ///
        /// * Must be pending owner
        ///
        #[pallet::call_index(23)]
        #[pallet::weight({0})]
        pub fn accept_subnet_ownership(origin: OriginFor<T>, subnet_id: u32) -> DispatchResult {
            Self::is_paused()?;
            Self::do_accept_subnet_ownership(origin, subnet_id)
        }

        #[pallet::call_index(24)]
        #[pallet::weight({0})]
        pub fn owner_add_bootnode_access(
            origin: OriginFor<T>,
            subnet_id: u32,
            new_account: T::AccountId,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_add_bootnode_access(origin, subnet_id, new_account)
        }

        #[pallet::call_index(25)]
        #[pallet::weight({0})]
        pub fn owner_remove_bootnode_access(
            origin: OriginFor<T>,
            subnet_id: u32,
            remove_account: T::AccountId,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_remove_bootnode_access(origin, subnet_id, remove_account)
        }

        #[pallet::call_index(26)]
        #[pallet::weight({0})]
        pub fn owner_update_target_node_registrations_per_epoch(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u32,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_target_node_registrations_per_epoch(origin, subnet_id, value)
        }

        #[pallet::call_index(27)]
        #[pallet::weight({0})]
        pub fn owner_update_node_burn_rate_alpha(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_node_burn_rate_alpha(origin, subnet_id, value)
        }

        #[pallet::call_index(28)]
        #[pallet::weight({0})]
        pub fn owner_update_queue_immunity_epochs(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u32,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_queue_immunity_epochs(origin, subnet_id, value)
        }

        #[pallet::call_index(29)]
        #[pallet::weight({0})]
        pub fn owner_update_subnet_node_min_weight_decrease_reputation_threshold(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_subnet_node_min_weight_decrease_reputation_threshold(
                origin, subnet_id, value,
            )
        }

        #[pallet::call_index(30)]
        #[pallet::weight({0})]
        pub fn owner_update_min_subnet_node_reputation(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_min_subnet_node_reputation(origin, subnet_id, value)
        }

        #[pallet::call_index(31)]
        #[pallet::weight({0})]
        pub fn owner_update_absent_decrease_reputation_factor(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_absent_decrease_reputation_factor(origin, subnet_id, value)
        }

        #[pallet::call_index(32)]
        #[pallet::weight({0})]
        pub fn owner_update_included_increase_reputation_factor(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_included_increase_reputation_factor(origin, subnet_id, value)
        }

        #[pallet::call_index(33)]
        #[pallet::weight({0})]
        pub fn owner_update_below_min_weight_decrease_reputation_factor(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_below_min_weight_decrease_reputation_factor(
                origin, subnet_id, value,
            )
        }

        #[pallet::call_index(34)]
        #[pallet::weight({0})]
        pub fn owner_update_non_attestor_decrease_reputation_factor(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_non_attestor_decrease_reputation_factor(origin, subnet_id, value)
        }

        #[pallet::call_index(35)]
        #[pallet::weight({0})]
        pub fn owner_update_validator_absent_decrease_reputation_factor(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_validator_absent_decrease_reputation_factor(
                origin, subnet_id, value,
            )
        }

        #[pallet::call_index(36)]
        #[pallet::weight({0})]
        pub fn owner_update_validator_non_consensus_decrease_reputation_factor(
            origin: OriginFor<T>,
            subnet_id: u32,
            value: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_owner_update_validator_non_consensus_decrease_reputation_factor(
                origin, subnet_id, value,
            )
        }

        #[pallet::call_index(37)]
        #[pallet::weight({0})]
        pub fn update_bootnodes(
            origin: OriginFor<T>,
            subnet_id: u32,
            add: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>,
            remove: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>>,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_update_bootnodes(origin, subnet_id, add, remove)
        }

        /// Register a Subnet Node to the subnet
        ///
        /// A registered Subnet Node will not be included in consensus data, therefor no incentives until
        /// the Subnet Node is activatated from the queue
        ///
        /// Subnet nodes register by staking the minimum required balance to pass authentication in any P2P
        /// networks, such as a subnet.
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `hotkey` - Hotkey of the Subnet Node.
        /// * `peer_id` - The Peer ID of the Subnet Node within the subnet P2P network.
        /// * `stake_to_be_added` - The balance to add to stake.
        /// * `a` - A Subnet Node parameter unique to each subnet.
        /// * `b` - A non-unique parameter.
        ///
        /// # Requirements
        ///
        /// * `stake_to_be_added` must be the minimum required stake balance
        ///
        #[pallet::call_index(38)]
        #[pallet::weight({0})]
        pub fn register_subnet_node(
            origin: OriginFor<T>,
            subnet_id: u32,
            hotkey: T::AccountId,
            peer_id: PeerId,
            bootnode_peer_id: PeerId,
            client_peer_id: PeerId,
            bootnode: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
            delegate_reward_rate: u128,
            stake_to_be_added: u128,
            unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
            non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
            max_burn_amount: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_register_subnet_node(
                origin,
                subnet_id,
                hotkey,
                peer_id,
                bootnode_peer_id,
                client_peer_id,
                bootnode,
                delegate_reward_rate,
                stake_to_be_added,
                unique,
                non_unique,
                u128::MAX,
            )
        }

        /// Remove Subnet Node of caller
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `subnet_node_id` - Subnet node ID assigned during registration
        ///
        /// # Requirements
        ///
        /// * Caller must be owner of Subnet Node, hotkey or coldkey
        ///
        #[pallet::call_index(39)]
        // #[pallet::weight(T::WeightInfo::remove_subnet_node())]
        #[pallet::weight({0})]
        pub fn remove_subnet_node(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
        ) -> DispatchResult {
            let key: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            ensure!(
                Self::is_subnet_node_keys_owner(subnet_id, subnet_node_id, key),
                Error::<T>::NotKeyOwner
            );

            // Check if validator
            let subnet_epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);
            let is_chosen_validator: bool =
                Self::is_chosen_validator(subnet_id, subnet_node_id, subnet_epoch);
            ensure!(
                !is_chosen_validator,
                Error::<T>::ElectedValidatorCannotRemove
            );

            Self::do_remove_subnet_node(subnet_id, subnet_node_id)
        }

        #[pallet::call_index(40)]
        #[pallet::weight({0})]
        pub fn update_peer_id(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            new_peer_id: PeerId,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            ensure!(
                Self::is_subnet_node_coldkey(subnet_id, subnet_node_id, coldkey),
                Error::<T>::NotKeyOwner
            );

            Self::do_update_peer_id(subnet_id, subnet_node_id, new_peer_id)
        }

        #[pallet::call_index(41)]
        #[pallet::weight({0})]
        pub fn update_bootnode(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            new_bootnode: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            ensure!(
                Self::is_subnet_node_coldkey(subnet_id, subnet_node_id, coldkey),
                Error::<T>::NotKeyOwner
            );

            Self::do_update_bootnode(subnet_id, subnet_node_id, new_bootnode)
        }

        #[pallet::call_index(42)]
        #[pallet::weight({0})]
        pub fn update_bootnode_peer_id(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            new_bootnode_peer_id: PeerId,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            ensure!(
                Self::is_subnet_node_coldkey(subnet_id, subnet_node_id, coldkey),
                Error::<T>::NotKeyOwner
            );

            Self::do_update_bootnode_peer_id(subnet_id, subnet_node_id, new_bootnode_peer_id)
        }

        #[pallet::call_index(43)]
        #[pallet::weight({0})]
        pub fn update_client_peer_id(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            new_client_peer_id: PeerId,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            ensure!(
                Self::is_subnet_node_coldkey(subnet_id, subnet_node_id, coldkey),
                Error::<T>::NotKeyOwner
            );

            Self::do_update_client_peer_id(subnet_id, subnet_node_id, new_client_peer_id)
        }

        /// Update node delegate reward rate
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `subnet_node_id` - Subnet node ID.
        /// * `new_delegate_reward_rate` - New delegate reward rate.
        ///
        /// # Requirements
        ///
        /// * Caller must be coldkey owner of Subnet Node ID.
        /// * If decreasing rate, new rate must not be more than a 1% decrease nominally
        ///
        #[pallet::call_index(44)]
        #[pallet::weight({0})]
        pub fn update_node_delegate_reward_rate(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            new_delegate_reward_rate: u128,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            ensure!(
                Self::is_subnet_node_coldkey(subnet_id, subnet_node_id, coldkey),
                Error::<T>::NotKeyOwner
            );

            Self::do_update_node_delegate_reward_rate(
                subnet_id,
                subnet_node_id,
                new_delegate_reward_rate,
            )
        }

        /// Register unique Subnet Node parameter if not already added
        ///
        /// Can be updatd by coldkey or hotkey
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `subnet_node_id` - Callers Subnet Node ID
        /// * `a` - The unique parameter
        ///
        #[pallet::call_index(45)]
        #[pallet::weight({0})]
        pub fn update_unique(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        ) -> DispatchResult {
            Self::is_paused()?;

            let key: T::AccountId = ensure_signed(origin)?;

            ensure!(
                Self::is_subnet_node_keys_owner(subnet_id, subnet_node_id, key),
                Error::<T>::NotKeyOwner
            );

            Self::do_update_unique(subnet_id, subnet_node_id, unique)
        }

        /// Register non-unique Subnet Node parameter
        ///
        /// Can be updatd by coldkey or hotkey
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `subnet_node_id` - Callers Subnet Node ID
        /// * `non_unique` - The non-unique parameter
        ///
        #[pallet::call_index(46)]
        #[pallet::weight({0})]
        pub fn update_non_unique(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
        ) -> DispatchResult {
            let key: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            ensure!(
                Self::is_subnet_node_keys_owner(subnet_id, subnet_node_id, key),
                Error::<T>::NotKeyOwner
            );

            Self::do_update_non_unique(subnet_id, subnet_node_id, non_unique)
        }

        /// Add to Subnet Node stake
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `subnet_node_id` - Subnet node ID assigned during registration
        /// * `hotkey` - Hotkey of Subnet Node
        /// * `stake_to_be_added` - Amount to add to stake
        ///
        /// # Requirements
        ///
        /// * Coldkey caller only
        /// * Subnet must exist
        /// * Must have amount free in wallet
        ///
        #[pallet::call_index(47)]
        // #[pallet::weight(T::WeightInfo::add_stake())]
        #[pallet::weight({0})]
        pub fn add_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            hotkey: T::AccountId,
            stake_to_be_added: u128,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            // Each account can only have one peer
            // Staking is accounted for per account_id per subnet_id
            // We only check that origin exists within SubnetNodesData

            // --- Ensure subnet exists to add to stake
            ensure!(
                SubnetsData::<T>::contains_key(subnet_id),
                Error::<T>::InvalidSubnetId
            );

            // --- Ensure coldkey owns the hotkey
            ensure!(
                HotkeyOwner::<T>::get(&hotkey) == coldkey,
                Error::<T>::NotKeyOwner
            );

            Self::do_add_stake(origin, subnet_id, hotkey, stake_to_be_added)
        }

        /// Remove from Subnet Node stake and add to unstaking ledger
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `hotkey` - Hotkey of Subnet Node
        /// * `stake_to_be_removed` - Amount to remove from stake
        ///
        /// # Requirements
        ///
        /// * Coldkey caller only
        /// * If Subnet Node, must have available staked balance greater than minimum required stake balance
        ///
        #[pallet::call_index(48)]
        #[pallet::weight({0})]
        pub fn remove_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            hotkey: T::AccountId,
            stake_to_be_removed: u128,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            // --- Ensure the hotkey stake owner is owned by the caller
            ensure!(
                HotkeyOwner::<T>::get(&hotkey) == coldkey,
                Error::<T>::NotKeyOwner
            );

            // If account is a Subnet Node they can remove stake up to minimum required stake balance
            // Else they can remove entire balance because they are not validating subnets
            //		They are removed in `do_remove_subnet_node()` when self or consensus removed
            // This includes registered, active, and deactivated subnet nodes
            // Note that `HotkeySubnetNodeId` is cleaned when subnet is removed so we don't check if the
            // subnet exists
            let is_subnet_node: bool = match HotkeySubnetNodeId::<T>::try_get(subnet_id, &hotkey) {
                Ok(subnet_node_id) => {
                    let subnet_epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);
                    let is_chosen_validator: bool =
                        Self::is_chosen_validator(subnet_id, subnet_node_id, subnet_epoch);

                    // --- Check if current epochs validator, can't unstake if so
                    ensure!(
                        !is_chosen_validator,
                        Error::<T>::ElectedValidatorCannotUnstake
                    );

                    // --- Check if activated node
                    // If activated, must stay staked for minimum time
                    if let Some(subnet_node) =
                        Self::get_activated_subnet_node(subnet_id, subnet_node_id)
                    {
                        let min_stake_epochs = MinActiveNodeStakeEpochs::<T>::get();
                        // --- Ensure activated nodes minimum stake epochs are complete to remove any balances
                        ensure!(
                            subnet_node.classification.start_epoch + min_stake_epochs
                                <= subnet_epoch,
                            Error::<T>::MinActiveNodeStakeEpochs
                        );
                    }
                    true
                }
                Err(()) => false,
            };

            // Remove stake
            // 		is_subnet_node: cannot remove stake below minimum required stake
            // 		else: can remove total stake balance
            Self::do_remove_stake(
                origin,
                subnet_id,
                hotkey,
                is_subnet_node,
                stake_to_be_removed,
            )
        }

        /// Transfer unstaking ledger balance to coldkey
        ///
        /// # Requirements
        ///
        /// * Coldkey caller only
        /// * Must be owner of stake balance
        ///
        #[pallet::call_index(49)]
        #[pallet::weight({0})]
        pub fn claim_unbondings(origin: OriginFor<T>) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            let successful_unbondings: u32 = Self::do_claim_unbondings(&coldkey);

            // Give error if there is no unbondings
            ensure!(
                successful_unbondings > 0,
                Error::<T>::NoStakeUnbondingsOrCooldownNotMet
            );
            Ok(())
        }

        /// Increase subnet delegate stake
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `stake_to_be_added` - Amount of add to delegate stake
        ///
        /// # Requirements
        ///
        /// * Subnet must exist
        ///
        #[pallet::call_index(50)]
        #[pallet::weight({0})]
        pub fn add_to_delegate_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            stake_to_be_added: u128,
        ) -> DispatchResult {
            let account_id: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            // --- Ensure subnet exists
            ensure!(
                SubnetsData::<T>::contains_key(subnet_id),
                Error::<T>::InvalidSubnetId
            );

            Self::do_add_delegate_stake(origin, subnet_id, stake_to_be_added)
        }

        /// Swap subnet delegate stake
        ///
        /// * Swaps delegate stake from one subnet to another subnet in one call
        ///
        /// # Arguments
        ///
        /// * `from_subnet_id` - from subnet ID.
        /// * `to_subnet_id` - To subnet ID
        /// * `delegate_stake_shares_to_swap` - Shares of `from_subnet_id` to swap to `to_subnet_id`
        ///
        /// # Requirements
        ///
        /// * `to_subnet_id` subnet must exist
        ///
        #[pallet::call_index(51)]
        #[pallet::weight({0})]
        pub fn swap_delegate_stake(
            origin: OriginFor<T>,
            from_subnet_id: u32,
            to_subnet_id: u32,
            delegate_stake_shares_to_swap: u128,
        ) -> DispatchResult {
            Self::is_paused()?;

            // --- Ensure ``to`` subnet exists
            ensure!(
                SubnetsData::<T>::contains_key(to_subnet_id),
                Error::<T>::InvalidSubnetId
            );

            // Handles ``ensure_signed``
            Self::do_swap_delegate_stake(
                origin,
                from_subnet_id,
                to_subnet_id,
                delegate_stake_shares_to_swap,
            )
        }

        /// Transfer delegate stake balance (via shares) to a new account
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID staked to
        /// * `to_account_id` - Account ID to transfer shares to
        /// * `delegate_stake_shares_to_transfer` - Shares to transfer
        ///
        /// # Requirements
        ///
        /// * `to_subnet_id` subnet must exist
        ///
        #[pallet::call_index(52)]
        #[pallet::weight({0})]
        pub fn transfer_delegate_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            to_account_id: T::AccountId,
            delegate_stake_shares_to_transfer: u128,
        ) -> DispatchResult {
            Self::is_paused()?;

            // Handles ``ensure_signed``
            Self::do_transfer_delegate_stake(
                origin,
                subnet_id,
                to_account_id,
                delegate_stake_shares_to_transfer,
            )
        }

        /// Remove subnet delegate stake balance and add to unstaking ledger.
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `shares_to_be_removed` - Shares to remove
        ///
        /// # Requirements
        ///
        /// * Must have balance
        ///
        #[pallet::call_index(53)]
        // #[pallet::weight(T::WeightInfo::remove_delegate_stake())]
        #[pallet::weight({0})]
        pub fn remove_delegate_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            shares_to_be_removed: u128,
        ) -> DispatchResult {
            Self::is_paused()?;

            Self::do_remove_delegate_stake(origin, subnet_id, shares_to_be_removed)
        }

        /// * DONATION FUNCTION*
        ///
        /// Increase the delegate stake pool balance of a subnet
        ///
        /// * Anyone can perform this action as a donation
        ///
        /// # Notes
        ///
        /// *** THIS DOES ''NOT'' INCREASE A USERS BALANCE ***
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID to increase delegate pool balance of.
        /// * `amount` - Amount TENSOR to add to pool
        ///
        ///
        #[pallet::call_index(54)]
        #[pallet::weight({0})]
        pub fn donate_delegate_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            amount: u128,
        ) -> DispatchResult {
            let account_id: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            // --- Ensure subnet exists, otherwise at risk of burning tokens
            ensure!(
                SubnetsData::<T>::contains_key(subnet_id),
                Error::<T>::InvalidSubnetId
            );

            ensure!(
                amount >= MinDelegateStakeDeposit::<T>::get(),
                Error::<T>::MinDelegateStake
            );

            let amount_as_balance = match Self::u128_to_balance(amount) {
                Some(b) => b,
                None => return Err(Error::<T>::CouldNotConvertToBalance.into()),
            };

            // --- Ensure the callers account_id has enough balance to perform the transaction.
            ensure!(
                Self::can_remove_balance_from_coldkey_account(&account_id, amount_as_balance),
                Error::<T>::NotEnoughBalance
            );

            // --- Ensure the remove operation from the account_id is a success.
            ensure!(
                Self::remove_balance_from_coldkey_account(&account_id, amount_as_balance) == true,
                Error::<T>::BalanceWithdrawalError
            );

            Self::do_increase_delegate_stake(subnet_id, amount);

            Ok(())
        }

        /// Delegate stake to a Subnet Node
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID
        /// * `node_account_id` - Subnet node ID
        /// * `node_delegate_stake_to_be_added` - Amount TENSOR to delegate stake
        ///
        #[pallet::call_index(55)]
        #[pallet::weight({0})]
        pub fn add_to_node_delegate_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            node_delegate_stake_to_be_added: u128,
        ) -> DispatchResult {
            Self::is_paused()?;

            ensure!(
                Self::get_subnet_node(subnet_id, subnet_node_id,).is_some(),
                Error::<T>::InvalidSubnetNodeId
            );

            Self::do_add_node_delegate_stake(
                origin,
                subnet_id,
                subnet_node_id,
                node_delegate_stake_to_be_added,
            )
        }

        /// Transfer Subnet Node delegate stake between any Subnet Node across all subnets
        ///
        /// * Swaps delegate stake from one subnet to another subnet in one call
        ///
        /// # Arguments
        ///
        /// * `from_subnet_id` - From subnet ID.
        /// * `from_subnet_node_id` - From Subnet Node ID
        /// * `to_subnet_id` - To subnet ID.
        /// * `to_subnet_node_id` - To Subnet Node ID
        /// * `node_delegate_stake_shares_to_swap` - Shares of `from_subnet_id` to swap to `to_subnet_id`
        ///
        /// # Requirements
        ///
        /// * `to_subnet_id` subnet must exist
        ///
        #[pallet::call_index(56)]
        #[pallet::weight({0})]
        pub fn swap_node_delegate_stake(
            origin: OriginFor<T>,
            from_subnet_id: u32,
            from_subnet_node_id: u32,
            to_subnet_id: u32,
            to_subnet_node_id: u32,
            node_delegate_stake_shares_to_swap: u128,
        ) -> DispatchResult {
            Self::is_paused()?;

            // --- Ensure ``to`` Subnet Node exists
            ensure!(
                Self::get_subnet_node(to_subnet_id, to_subnet_node_id,).is_some(),
                Error::<T>::InvalidSubnetNodeId
            );

            Self::do_swap_node_delegate_stake(
                origin,
                from_subnet_id,
                from_subnet_node_id,
                to_subnet_id,
                to_subnet_node_id,
                node_delegate_stake_shares_to_swap,
            )
        }

        /// Transfer node delegate stake balance (via shares) to a new account
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `from_subnet_node_id` - Subnet node ID
        /// * `to_account_id` - Account ID to transfer shares to
        /// * `node_delegate_stake_shares_to_transfer` - Shares to transfer
        ///
        /// # Requirements
        ///
        /// * `to_subnet_id` subnet must exist
        ///
        #[pallet::call_index(57)]
        #[pallet::weight({0})]
        pub fn transfer_node_delegate_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            to_account_id: T::AccountId,
            node_delegate_stake_shares_to_transfer: u128,
        ) -> DispatchResult {
            Self::is_paused()?;

            Self::do_transfer_node_delegate_stake(
                origin,
                subnet_id,
                subnet_node_id,
                to_account_id,
                node_delegate_stake_shares_to_transfer,
            )
        }

        /// Remove delegate stake from a Subnet Node and add to unbonding ledger.
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID
        /// * `node_account_id` - Subnet node ID
        /// * `node_delegate_stake_shares_to_be_removed` - Pool shares to remove
        ///
        #[pallet::call_index(58)]
        #[pallet::weight({0})]
        pub fn remove_node_delegate_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            node_delegate_stake_shares_to_be_removed: u128,
        ) -> DispatchResult {
            Self::is_paused()?;

            Self::do_remove_node_delegate_stake(
                origin,
                subnet_id,
                subnet_node_id,
                node_delegate_stake_shares_to_be_removed,
            )
        }

        /// * DONATION FUNCTION*
        ///
        /// Increase the node delegate stake pool balance of a Subnet Node
        ///
        /// * Anyone can perform this action as a donation
        ///
        /// # Notes
        ///
        /// *** THIS DOES ''NOT'' INCREASE A USERS BALANCE ***
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID to increase delegate pool balance of.
        /// * `subnet_node_id` - Subnet node ID.
        /// * `amount` - Amount TENSOR to add to pool
        ///
        #[pallet::call_index(59)]
        #[pallet::weight({0})]
        pub fn donate_node_delegate_stake(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
            amount: u128,
        ) -> DispatchResult {
            let account_id: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            // --- Ensure Subnet Node exists, otherwise at risk of burning tokens
            ensure!(
                SubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id),
                Error::<T>::InvalidSubnetNodeId
            );

            ensure!(
                amount >= MinDelegateStakeDeposit::<T>::get(),
                Error::<T>::MinDelegateStake
            );

            let amount_as_balance = match Self::u128_to_balance(amount) {
                Some(b) => b,
                None => return Err(Error::<T>::CouldNotConvertToBalance.into()),
            };

            // --- Ensure the callers account_id has enough balance to perform the transaction.
            ensure!(
                Self::can_remove_balance_from_coldkey_account(&account_id, amount_as_balance),
                Error::<T>::NotEnoughBalance
            );

            // --- Ensure the remove operation from the account_id is a success.
            ensure!(
                Self::remove_balance_from_coldkey_account(&account_id, amount_as_balance) == true,
                Error::<T>::BalanceWithdrawalError
            );

            Self::do_increase_node_delegate_stake(subnet_id, subnet_node_id, amount);

            Ok(())
        }

        /// Swap stake from a Subnet Node to a subnet
        ///
        /// # Arguments
        ///
        /// * `from_subnet_id` - From subnet ID to remove delegate stake from.
        /// * `from_subnet_node_id` - From Subnet Node ID to remove delegate stake from.
        /// * `to_subnet_id` - To subnet ID to add delegate stake to
        /// * `node_delegate_stake_shares_to_swap` - Shares to remove from delegate pool and add balance to subnet
        ///
        #[pallet::call_index(60)]
        #[pallet::weight({0})]
        pub fn swap_from_node_to_subnet(
            origin: OriginFor<T>,
            from_subnet_id: u32,
            from_subnet_node_id: u32,
            to_subnet_id: u32,
            node_delegate_stake_shares_to_swap: u128,
        ) -> DispatchResult {
            Self::is_paused()?;

            ensure!(
                SubnetsData::<T>::contains_key(to_subnet_id),
                Error::<T>::InvalidSubnetId
            );

            Self::do_swap_from_node_to_subnet(
                origin,
                from_subnet_id,
                from_subnet_node_id,
                to_subnet_id,
                node_delegate_stake_shares_to_swap,
            )
        }

        /// Swap stake from a subnet to a Subnet Node
        ///
        /// # Arguments
        ///
        /// * `from_subnet_id` - From subnet ID to remove delegate stake from.
        /// * `to_subnet_id` - To subnet ID to add delegate stake to.
        /// * `to_subnet_node_id` - To Subnet Node ID to add delegate stake to
        /// * `delegate_stake_shares_to_swap` - Shares to remove from delegate pool and add balance to node
        ///
        #[pallet::call_index(61)]
        #[pallet::weight({0})]
        pub fn swap_from_subnet_to_node(
            origin: OriginFor<T>,
            from_subnet_id: u32,
            to_subnet_id: u32,
            to_subnet_node_id: u32,
            delegate_stake_shares_to_swap: u128,
        ) -> DispatchResult {
            Self::is_paused()?;

            ensure!(
                Self::get_subnet_node(to_subnet_id, to_subnet_node_id,).is_some(),
                Error::<T>::InvalidSubnetNodeId
            );

            Self::do_swap_from_subnet_to_node(
                origin,
                from_subnet_id,
                to_subnet_id,
                to_subnet_node_id,
                delegate_stake_shares_to_swap,
            )
        }

        #[pallet::call_index(62)]
        #[pallet::weight({0})]
        pub fn update_swap_queue(
            origin: OriginFor<T>,
            id: u32,
            new_call: QueuedSwapCall<T::AccountId>,
        ) -> DispatchResult {
            let account_id: T::AccountId = ensure_signed(origin)?;
            Self::do_update_swap_queue(account_id, id, new_call)
        }

        /// Register onchain identity
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `subnet_node_id` - Subnet node ID assigned during registration
        ///
        /// # Requirements
        ///
        /// * Caller must be owner of Subnet Node, hotkey or coldkey
        ///
        #[pallet::call_index(63)]
        #[pallet::weight({0})]
        pub fn register_or_update_identity(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            name: BoundedVec<u8, DefaultMaxVectorLength>,
            url: BoundedVec<u8, DefaultMaxUrlLength>,
            image: BoundedVec<u8, DefaultMaxUrlLength>,
            discord: BoundedVec<u8, DefaultMaxSocialIdLength>,
            x: BoundedVec<u8, DefaultMaxSocialIdLength>,
            telegram: BoundedVec<u8, DefaultMaxSocialIdLength>,
            github: BoundedVec<u8, DefaultMaxUrlLength>,
            hugging_face: BoundedVec<u8, DefaultMaxUrlLength>,
            description: BoundedVec<u8, DefaultMaxVectorLength>,
            misc: BoundedVec<u8, DefaultMaxVectorLength>,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            Self::do_register_or_update_identity(
                coldkey,
                hotkey,
                name,
                url,
                image,
                discord,
                x,
                telegram,
                github,
                hugging_face,
                description,
                misc,
            )
        }

        #[pallet::call_index(64)]
        #[pallet::weight({0})]
        pub fn remove_identity(origin: OriginFor<T>) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin)?;
            Self::do_remove_identity(coldkey);
            Ok(())
        }

        /// Validator extrinsic for submitting incentives protocol data of the validators view of of the subnet
        /// This is used t oscore each Subnet Node for allocation of emissions
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID to increase delegate pool balance of.
        /// * `data` - Vector of SubnetNodeConsensusData on each Subnet Node for scoring each
        /// * `args` (Optional) - Data that can be used by the subnet
        ///
        /// Returns Ok(Pays::No.into()) on success
        ///
        #[pallet::call_index(65)]
        #[pallet::weight({0})]
        pub fn propose_attestation(
            origin: OriginFor<T>,
            subnet_id: u32,
            data: Vec<SubnetNodeConsensusData>,
            prioritize_queue_node_id: Option<u32>,
            remove_queue_node_id: Option<u32>,
            args: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
            attest_data: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
        ) -> DispatchResultWithPostInfo {
            let hotkey: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            Self::do_propose_attestation(
                subnet_id,
                hotkey,
                data,
                prioritize_queue_node_id,
                remove_queue_node_id,
                args,
                attest_data,
            )
        }

        /// Attest validators view of the subnet
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID to increase delegate pool balance of.
        ///
        /// Returns Ok(Pays::No.into()) on success
        ///
        #[pallet::call_index(66)]
        #[pallet::weight({0})]
        pub fn attest(
            origin: OriginFor<T>,
            subnet_id: u32,
            attest_data: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
        ) -> DispatchResultWithPostInfo {
            let hotkey: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            Self::do_attest(subnet_id, hotkey, attest_data)
        }

        /// Update coldkey
        ///
        /// # Arguments
        ///
        /// * `hotkey` - Current hotkey.
        /// * `new_coldkey` - New coldkey
        /// * `subnet_id` - Optional parameter used for subnet owners
        ///
        #[pallet::call_index(67)]
        #[pallet::weight({0})]
        pub fn update_coldkey(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            new_coldkey: T::AccountId,
        ) -> DispatchResult {
            let curr_coldkey: T::AccountId = ensure_signed(origin)?;

            Self::is_paused()?;

            ensure!(&hotkey != &new_coldkey, Error::<T>::ColdkeyMatchesHotkey);

            HotkeyOwner::<T>::try_mutate_exists(hotkey, |maybe_coldkey| -> DispatchResult {
                match maybe_coldkey {
                    Some(coldkey) if *coldkey == curr_coldkey => {
                        // Condition met, update or remove
                        *maybe_coldkey = Some(new_coldkey.clone());
                        // Update StakeUnbondingLedger
                        // StakeUnbondingLedger::<T>::swap(&curr_coldkey, &new_coldkey);

                        StakeUnbondingLedger::<T>::swap(&curr_coldkey, &new_coldkey);

                        // Update coldkeys list of hotkeys
                        ColdkeyHotkeys::<T>::swap(&curr_coldkey, &new_coldkey);

                        // Identity is not required so we ensure it exists first
                        if let Ok(coldkey_identity) = ColdkeyIdentity::<T>::try_get(&curr_coldkey) {
                            ColdkeyIdentity::<T>::swap(&curr_coldkey, &new_coldkey);
                            ColdkeyIdentityNameOwner::<T>::insert(
                                coldkey_identity.name.clone(),
                                &new_coldkey,
                            );
                        };

                        ColdkeyReputation::<T>::swap(&curr_coldkey, &new_coldkey);

                        ColdkeySubnetNodes::<T>::swap(&curr_coldkey, &new_coldkey);

                        Self::deposit_event(Event::UpdateColdkey {
                            coldkey: curr_coldkey,
                            new_coldkey: new_coldkey,
                        });

                        Ok(())
                    }
                    // --- Revert from here if not exist
                    Some(_) => Err(Error::<T>::NotKeyOwner.into()),
                    None => Err(Error::<T>::NotKeyOwner.into()),
                }
            })
        }

        /// Update hotkey (subnet node, overwatch node)
        ///
        /// # Requirements
        ///
        /// * Coldkey caller only
        /// * New hotkey must be already exist
        ///		- No merging logic
        ///
        /// # Note
        ///
        /// This only updates node related storage elements and not user elements like node/delegating staking keys
        ///
        /// # Arguments
        ///
        /// * `old_hotkey` - Old hotkey to be replaced.
        /// * `new_hotkey` - New hotkey to replace the old hotkey.
        ///
        #[pallet::call_index(68)]
        #[pallet::weight({0})]
        pub fn update_hotkey(
            origin: OriginFor<T>,
            old_hotkey: T::AccountId,
            new_hotkey: T::AccountId,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            ensure!(&coldkey != &new_hotkey, Error::<T>::ColdkeyMatchesHotkey);

            // Ensure `old_hotkey` is owned by caller
            ensure!(
                Self::is_hotkey_owner(&old_hotkey, &coldkey),
                Error::<T>::NotKeyOwner
            );

            // --- Ensure new_hotkey not taken
            // Hotkeys must be unique across network
            ensure!(
                !Self::hotkey_has_owner(new_hotkey.clone()),
                Error::<T>::HotkeyHasOwner
            );

            let mut hotkeys = ColdkeyHotkeys::<T>::get(&coldkey);
            // Redundant
            ensure!(
                !hotkeys.contains(&new_hotkey),
                Error::<T>::HotkeyAlreadyRegisteredToColdkey
            );
            // Redundant
            ensure!(
                hotkeys.contains(&old_hotkey),
                Error::<T>::OldHotkeyNotRegistered
            );
            // Replace
            hotkeys.remove(&old_hotkey);
            hotkeys.insert(new_hotkey.clone());
            ColdkeyHotkeys::<T>::insert(&coldkey, hotkeys);

            // Update hotkey owner
            HotkeyOwner::<T>::swap(&old_hotkey, &new_hotkey);

            // --- Update overwatch node hotkey
            if let Some(overwatch_node_id) = HotkeyOverwatchNodeId::<T>::take(&old_hotkey) {
                OverwatchNodeIdHotkey::<T>::insert(overwatch_node_id, &new_hotkey);
                HotkeyOverwatchNodeId::<T>::insert(&new_hotkey, overwatch_node_id);
                OverwatchNodes::<T>::try_mutate_exists(
                    overwatch_node_id,
                    |maybe_params| -> DispatchResult {
                        let params = maybe_params
                            .as_mut()
                            .ok_or(Error::<T>::InvalidOverwatchNodeId)?;
                        params.hotkey = new_hotkey.clone();
                        Ok(())
                    },
                );
            };

            // --- Update overwatch node stake (outside of above `if` incase removed but has balance)
            let account_overwatch_stake: u128 = AccountOverwatchStake::<T>::get(&old_hotkey);
            if account_overwatch_stake != 0 {
                Self::do_swap_overwatch_hotkey_balance(&old_hotkey, &new_hotkey);
            }

            // Check if hotkey has a subnet ID and update node and stake balance
            // *Note: Each hotkey is unique to each subnet
            if let Some(subnet_id) = HotkeySubnetId::<T>::take(&old_hotkey) {
                // HotkeySubnetNodeId is removed when node and subnet is removed
                if let Some(subnet_node_id) = HotkeySubnetNodeId::<T>::get(&subnet_id, &old_hotkey)
                {
                    // --- Update nodes hotkey by inserting and overriding
                    SubnetNodeIdHotkey::<T>::insert(subnet_id, subnet_node_id, &new_hotkey);
                    // --- Update Subnet Nodes Data's hotkey (Active, Registered)
                    Self::update_subnet_node_hotkey(subnet_id, subnet_node_id, new_hotkey.clone());
                    // Swap hotkeys -> node_id
                    HotkeySubnetNodeId::<T>::swap(subnet_id, &old_hotkey, subnet_id, &new_hotkey);
                }

                // Note: This is never removed to allow the RPC to get a coldkeys stake data
                // --- Insert new hotkey (we `take` it earlier)
                HotkeySubnetId::<T>::insert(&new_hotkey, subnet_id);
            }

            // --- Swap stake balance
            // Note: Iterating is redundant here since hotkeys are unique but we do anyway
            // Iterate each hotkey stake
            // If a Subnet Node or subnet is no longer active, the stake can still be available for unstaking
            for (subnet_id, balance) in AccountSubnetStake::<T>::iter_prefix(&old_hotkey) {
                if balance != 0 {
                    Self::do_swap_hotkey_stake_balance(
                        subnet_id,
                        &old_hotkey, // from
                        &new_hotkey, // to
                    );
                }
            }

            Self::deposit_event(Event::UpdateHotkey {
                hotkey: old_hotkey,
                new_hotkey: new_hotkey,
            });

            Ok(())
        }

        #[pallet::call_index(69)]
        #[pallet::weight({0})]
        pub fn register_overwatch_node(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            stake_to_be_added: u128,
        ) -> DispatchResult {
            Self::is_paused()?;
            Self::do_register_overwatch_node(origin, hotkey, stake_to_be_added)
        }

        #[pallet::call_index(70)]
        #[pallet::weight({0})]
        pub fn remove_overwatch_node(
            origin: OriginFor<T>,
            overwatch_node_id: u32,
        ) -> DispatchResult {
            let key: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            Self::do_remove_overwatch_node(key, overwatch_node_id)
        }

        #[pallet::call_index(71)]
        #[pallet::weight({0})]
        pub fn anyone_remove_overwatch_node(
            origin: OriginFor<T>,
            overwatch_node_id: u32,
        ) -> DispatchResult {
            ensure_signed(origin.clone())?;

            Self::is_paused()?;

            // Ensure overwatch node exists
            let overwatch_node_hotkey = match OverwatchNodeIdHotkey::<T>::get(overwatch_node_id) {
                Some(hotkey) => hotkey,
                None => return Err(Error::<T>::InvalidOverwatchNodeId.into()),
            };

            let overwatch_node_coldkey = HotkeyOwner::<T>::get(&overwatch_node_hotkey);

            // Ensure overwatch node is not qualified
            ensure!(
                !Self::is_overwatch_node_qualified(&overwatch_node_coldkey),
                Error::<T>::ColdkeyOverwatchQualified
            );

            Self::perform_remove_overwatch_node(overwatch_node_id);

            Ok(())
        }

        /// Update hotkey (subnet node, overwatch node)
        ///
        /// # Requirements
        ///
        /// * Must be overwatch node
        /// * Subnet ID must exist
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - Subnet ID.
        /// * `overwatch_node_id` - Overwatch node ID of caller.
        /// * `peer_id` - New peer ID for subnet.
        ///
        /// Returns Ok(Pays::No.into()) on success
        ///
        #[pallet::call_index(72)]
        #[pallet::weight({0})]
        pub fn set_overwatch_node_peer_id(
            origin: OriginFor<T>,
            subnet_id: u32,
            overwatch_node_id: u32,
            peer_id: PeerId,
        ) -> DispatchResultWithPostInfo {
            Self::is_paused()?;
            Self::do_set_overwatch_node_peer_id(origin, subnet_id, overwatch_node_id, peer_id)
        }

        /// Commit overwatch subnet weight weight
        ///
        /// # Requirements
        ///
        /// * Must be an overwatch node
        /// * Must be in the commit period/phase of the overwatch epoch
        ///
        /// # Note
        ///
        /// This can be called multiple times in the commit phase of the overwatch epoch
        ///
        /// # Arguments
        ///
        /// * `overwatch_node_id` - Caller Overwatch Node ID.
        /// * `mut commit_weights` - Vector of hashed subnet commits, see `OverwatchCommit`.
        ///
        /// Returns Ok(Pays::No.into()) on success
        ///
        #[pallet::call_index(73)]
        #[pallet::weight({0})]
        pub fn commit_overwatch_subnet_weights(
            origin: OriginFor<T>,
            overwatch_node_id: u32,
            mut commit_weights: Vec<OverwatchCommit<T::Hash>>,
        ) -> DispatchResultWithPostInfo {
            Self::is_paused()?;
            Self::do_commit_overwatch_subnet_weights(origin, overwatch_node_id, commit_weights)
        }

        /// Reveal overwatch subnet weight weight
        ///
        /// # Requirements
        ///
        /// * Must be an overwatch node
        /// * Every reveal must match every commit
        ///
        /// # Note
        ///
        /// This can be called multiple times in the reveal phase of the overwatch epoch
        ///
        /// # Arguments
        ///
        /// * `overwatch_node_id` - Caller Overwatch Node ID.
        /// * `reveals` - Vector of commit reveals, see `OverwatchReveal`.
        ///
        /// Returns Ok(Pays::No.into()) on success
        ///
        #[pallet::call_index(74)]
        #[pallet::weight({0})]
        pub fn reveal_overwatch_subnet_weights(
            origin: OriginFor<T>,
            overwatch_node_id: u32,
            reveals: Vec<OverwatchReveal>,
        ) -> DispatchResultWithPostInfo {
            Self::is_paused()?;
            Self::do_reveal_overwatch_subnet_weights(origin, overwatch_node_id, reveals)
        }

        /// Add to Overwatch Node stake
        ///
        /// # Arguments
        ///
        /// * `overwatch_node_id` - Overwatch Node ID assigned during registration
        /// * `hotkey` - Hotkey of Overwatch Node
        /// * `stake_to_be_added` - Amount to add to stake
        ///
        /// # Requirements
        ///
        /// * Coldkey caller only
        /// * Must have amount free in wallet
        ///
        #[pallet::call_index(75)]
        #[pallet::weight({0})]
        pub fn add_to_overwatch_stake(
            origin: OriginFor<T>,
            overwatch_node_id: u32,
            hotkey: T::AccountId,
            stake_to_be_added: u128,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            // --- Ensure overwatch node
            ensure!(
                OverwatchNodes::<T>::contains_key(overwatch_node_id),
                Error::<T>::InvalidSubnetId
            );

            // --- Ensure coldkey owns the hotkey
            ensure!(
                HotkeyOwner::<T>::get(&hotkey) == coldkey,
                Error::<T>::NotKeyOwner
            );

            Self::do_add_overwatch_stake(coldkey, hotkey, stake_to_be_added)
        }

        /// Remove from Overwatch Node stake and add to unstaking ledger
        ///
        /// # Arguments
        ///
        /// * `hotkey` - Hotkey of Overwatch Node
        /// * `stake_to_be_removed` - Amount to remove from stake
        ///
        /// # Requirements
        ///
        /// * Coldkey caller only
        /// * If Overwatch Node, must have available staked balance greater than minimum required stake balance
        ///
        #[pallet::call_index(76)]
        #[pallet::weight({0})]
        pub fn remove_overwatch_stake(
            origin: OriginFor<T>,
            hotkey: T::AccountId,
            stake_to_be_removed: u128,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin.clone())?;

            Self::is_paused()?;

            // --- Ensure the hotkey stake owner is owned by the caller
            ensure!(
                HotkeyOwner::<T>::get(&hotkey) == coldkey,
                Error::<T>::NotKeyOwner
            );

            // If account is an overwatch node they can remove stake up to minimum required stake balance
            let is_overwatch_node: bool = match HotkeyOverwatchNodeId::<T>::try_get(&hotkey) {
                Ok(_) => true,
                Err(()) => false,
            };

            Self::do_remove_overwatch_stake(
                origin.clone(),
                hotkey,
                is_overwatch_node,
                stake_to_be_removed,
            )
        }

        /// Collective functions
        ///
        /// These are a set of functions designated for the collective to manage the network
        ///

        /// Pause network
        ///
        /// # Requirements
        ///
        /// Requires majority vote
        ///
        #[pallet::call_index(77)]
        #[pallet::weight({0})]
        pub fn pause(origin: OriginFor<T>) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_pause()
        }

        /// Unpause network
        ///
        /// # Requirements
        ///
        /// Requires majority vote
        ///
        #[pallet::call_index(78)]
        #[pallet::weight({0})]
        pub fn unpause(origin: OriginFor<T>) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_unpause()
        }

        #[pallet::call_index(79)]
        #[pallet::weight({0})]
        pub fn collective_remove_subnet(
            origin: OriginFor<T>,
            subnet_id: u32,
        ) -> DispatchResultWithPostInfo {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_collective_remove_subnet(subnet_id)
        }

        #[pallet::call_index(80)]
        #[pallet::weight({0})]
        pub fn collective_remove_subnet_node(
            origin: OriginFor<T>,
            subnet_id: u32,
            subnet_node_id: u32,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_collective_remove_subnet_node(subnet_id, subnet_node_id)
        }

        #[pallet::call_index(81)]
        #[pallet::weight({0})]
        pub fn collective_remove_overwatch_node(
            origin: OriginFor<T>,
            overwatch_node_id: u32,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_collective_remove_overwatch_node(overwatch_node_id)
        }

        /// Set new minimum subnet delegate stake factor
        ///
        /// # Requirements
        ///
        /// Requires super majority vote
        ///
        #[pallet::call_index(82)]
        #[pallet::weight({0})]
        pub fn set_min_subnet_delegate_stake_factor(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_subnet_delegate_stake_factor(value)
        }

        /// Set new subnet owner percentage of emissions
        ///
        /// # Requirements
        ///
        /// Requires super majority vote
        ///
        #[pallet::call_index(83)]
        #[pallet::weight({0})]
        pub fn set_subnet_owner_percentage(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_subnet_owner_percentage(value)
        }

        #[pallet::call_index(84)]
        #[pallet::weight({0})]
        pub fn set_max_subnets(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_subnets(value)
        }

        #[pallet::call_index(85)]
        #[pallet::weight({0})]
        pub fn set_max_bootnodes(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_bootnodes(value)
        }

        #[pallet::call_index(86)]
        #[pallet::weight({0})]
        pub fn set_max_subnet_bootnodes_access(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_subnet_bootnodes_access(value)
        }

        #[pallet::call_index(87)]
        #[pallet::weight({0})]
        pub fn set_max_pause_epochs(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_pause_epochs(value)
        }

        #[pallet::call_index(88)]
        #[pallet::weight({0})]
        pub fn set_delegate_stake_subnet_removal_interval(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_delegate_stake_subnet_removal_interval(value)
        }

        #[pallet::call_index(89)]
        #[pallet::weight({0})]
        pub fn set_subnet_removal_intervals(
            origin: OriginFor<T>,
            min: u32,
            max: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_subnet_removal_intervals(min, max)
        }

        #[pallet::call_index(90)]
        #[pallet::weight({0})]
        pub fn set_subnet_pause_cooldown_epochs(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_subnet_pause_cooldown_epochs(value)
        }

        #[pallet::call_index(91)]
        #[pallet::weight({0})]
        pub fn set_min_registration_cost(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_registration_cost(value)
        }

        #[pallet::call_index(92)]
        #[pallet::weight({0})]
        pub fn set_registration_cost_delay_blocks(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_registration_cost_delay_blocks(value)
        }

        #[pallet::call_index(93)]
        #[pallet::weight({0})]
        pub fn set_registration_cost_alpha(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_registration_cost_alpha(value)
        }

        #[pallet::call_index(94)]
        #[pallet::weight({0})]
        pub fn set_new_registration_cost_multiplier(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_new_registration_cost_multiplier(value)
        }

        #[pallet::call_index(95)]
        #[pallet::weight({0})]
        pub fn set_max_min_delegate_stake_multiplier(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_min_delegate_stake_multiplier(value)
        }

        #[pallet::call_index(96)]
        #[pallet::weight({0})]
        pub fn set_churn_limits(origin: OriginFor<T>, min: u32, max: u32) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_churn_limits(min, max)
        }

        #[pallet::call_index(97)]
        #[pallet::weight({0})]
        pub fn set_queue_epochs(origin: OriginFor<T>, min: u32, max: u32) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_queue_epochs(min, max)
        }

        #[pallet::call_index(98)]
        #[pallet::weight({0})]
        pub fn set_max_swap_queue_calls_per_block(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_swap_queue_calls_per_block(value)
        }

        #[pallet::call_index(99)]
        #[pallet::weight({0})]
        pub fn set_min_idle_classification_epochs(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_idle_classification_epochs(value)
        }

        #[pallet::call_index(100)]
        #[pallet::weight({0})]
        pub fn set_max_idle_classification_epochs(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_idle_classification_epochs(value)
        }

        #[pallet::call_index(101)]
        #[pallet::weight({0})]
        pub fn set_subnet_activation_enactment_epochs(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_subnet_activation_enactment_epochs(value)
        }

        #[pallet::call_index(102)]
        #[pallet::weight({0})]
        pub fn set_included_classification_epochs(
            origin: OriginFor<T>,
            min: u32,
            max: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_included_classification_epochs(min, max)
        }

        #[pallet::call_index(103)]
        #[pallet::weight({0})]
        pub fn set_subnet_stakes(origin: OriginFor<T>, min: u128, max: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_subnet_stakes(min, max)
        }

        #[pallet::call_index(104)]
        #[pallet::weight({0})]
        pub fn set_delegate_stake_percentages(
            origin: OriginFor<T>,
            min: u128,
            max: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_delegate_stake_percentages(min, max)
        }

        #[pallet::call_index(105)]
        #[pallet::weight({0})]
        pub fn set_min_max_registered_nodes(
            origin: OriginFor<T>,
            min: u32,
            max: u32,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_max_registered_nodes(min, max)
        }

        #[pallet::call_index(106)]
        #[pallet::weight({0})]
        pub fn set_max_subnet_delegate_stake_rewards_percentage_change(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_subnet_delegate_stake_rewards_percentage_change(value)
        }

        #[pallet::call_index(107)]
        #[pallet::weight({0})]
        pub fn set_subnet_delegate_stake_rewards_update_period(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_subnet_delegate_stake_rewards_update_period(value)
        }

        #[pallet::call_index(108)]
        #[pallet::weight({0})]
        pub fn set_min_attestation_percentage(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_attestation_percentage(value)
        }

        #[pallet::call_index(109)]
        #[pallet::weight({0})]
        pub fn set_super_majority_attestation_ratio(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_super_majority_attestation_ratio(value)
        }

        #[pallet::call_index(110)]
        #[pallet::weight({0})]
        pub fn set_base_validator_reward(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_base_validator_reward(value)
        }

        #[pallet::call_index(111)]
        #[pallet::weight({0})]
        pub fn set_base_slash_percentage(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_base_slash_percentage(value)
        }

        #[pallet::call_index(112)]
        #[pallet::weight({0})]
        pub fn set_max_slash_amount(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_slash_amount(value)
        }

        #[pallet::call_index(113)]
        #[pallet::weight({0})]
        pub fn set_reputation_increase_factor(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_reputation_increase_factor(value)
        }

        #[pallet::call_index(114)]
        #[pallet::weight({0})]
        pub fn set_reputation_decrease_factor(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_reputation_decrease_factor(value)
        }

        #[pallet::call_index(115)]
        #[pallet::weight({0})]
        pub fn set_network_max_stake_balance(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_network_max_stake_balance(value)
        }

        #[pallet::call_index(116)]
        #[pallet::weight({0})]
        pub fn set_min_delegate_stake_deposit(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_delegate_stake_deposit(value)
        }

        #[pallet::call_index(117)]
        #[pallet::weight({0})]
        pub fn set_node_reward_rate_update_period(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_node_reward_rate_update_period(value)
        }

        #[pallet::call_index(118)]
        #[pallet::weight({0})]
        pub fn set_max_reward_rate_decrease(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_reward_rate_decrease(value)
        }

        #[pallet::call_index(119)]
        #[pallet::weight({0})]
        pub fn set_subnet_distribution_power(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_subnet_distribution_power(value)
        }

        #[pallet::call_index(120)]
        #[pallet::weight({0})]
        pub fn set_delegate_stake_weight_factor(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_delegate_stake_weight_factor(value)
        }

        #[pallet::call_index(121)]
        #[pallet::weight({0})]
        pub fn set_inflation_sigmoid_steepness(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_inflation_sigmoid_steepness(value)
        }

        #[pallet::call_index(122)]
        #[pallet::weight({0})]
        pub fn set_max_overwatch_nodes(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_overwatch_nodes(value)
        }

        #[pallet::call_index(123)]
        #[pallet::weight({0})]
        pub fn set_overwatch_epoch_length_multiplier(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_overwatch_epoch_length_multiplier(value)
        }

        #[pallet::call_index(124)]
        #[pallet::weight({0})]
        pub fn set_overwatch_commit_cutoff_percent(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_overwatch_commit_cutoff_percent(value)
        }

        #[pallet::call_index(125)]
        #[pallet::weight({0})]
        pub fn set_overwatch_min_diversification_ratio(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_overwatch_min_diversification_ratio(value)
        }

        #[pallet::call_index(126)]
        #[pallet::weight({0})]
        pub fn set_overwatch_min_rep_score(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_overwatch_min_rep_score(value)
        }

        #[pallet::call_index(127)]
        #[pallet::weight({0})]
        pub fn set_overwatch_min_avg_attestation_ratio(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_overwatch_min_avg_attestation_ratio(value)
        }

        #[pallet::call_index(128)]
        #[pallet::weight({0})]
        pub fn set_overwatch_min_age(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_overwatch_min_age(value)
        }

        #[pallet::call_index(129)]
        #[pallet::weight({0})]
        pub fn set_overwatch_min_stake_balance(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_overwatch_min_stake_balance(value)
        }

        #[pallet::call_index(130)]
        #[pallet::weight({0})]
        pub fn set_min_max_subnet_node(origin: OriginFor<T>, min: u32, max: u32) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_max_subnet_node(min, max)
        }

        #[pallet::call_index(131)]
        #[pallet::weight({0})]
        pub fn set_tx_rate_limit(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_tx_rate_limit(value)
        }

        #[pallet::call_index(132)]
        #[pallet::weight({0})]
        pub fn collective_set_coldkey_overwatch_node_eligibility(
            origin: OriginFor<T>,
            coldkey: T::AccountId,
            value: bool,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_collective_set_coldkey_overwatch_node_eligibility(coldkey, value)
        }

        #[pallet::call_index(133)]
        #[pallet::weight({0})]
        pub fn set_min_subnet_registration_epochs(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_subnet_registration_epochs(value)
        }

        #[pallet::call_index(134)]
        #[pallet::weight({0})]
        pub fn set_subnet_registration_epochs(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_subnet_registration_epochs(value)
        }

        #[pallet::call_index(135)]
        #[pallet::weight({0})]
        pub fn set_min_active_node_stake_epochs(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_active_node_stake_epochs(value)
        }

        #[pallet::call_index(136)]
        #[pallet::weight({0})]
        pub fn set_delegate_stake_cooldown_epochs(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_delegate_stake_cooldown_epochs(value)
        }

        #[pallet::call_index(137)]
        #[pallet::weight({0})]
        pub fn set_node_delegate_stake_cooldown_epochs(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_node_delegate_stake_cooldown_epochs(value)
        }

        #[pallet::call_index(138)]
        #[pallet::weight({0})]
        pub fn set_min_stake_cooldown_epochs(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_stake_cooldown_epochs(value)
        }

        #[pallet::call_index(139)]
        #[pallet::weight({0})]
        pub fn set_max_unbondings(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_unbondings(value)
        }

        /// Set midpoint on sigmoid for inflation mech
        #[pallet::call_index(140)]
        #[pallet::weight({0})]
        pub fn set_sigmoid_midpoint(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_sigmoid_midpoint(value)
        }

        #[pallet::call_index(141)]
        #[pallet::weight({0})]
        pub fn set_maximum_hooks_weight(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_maximum_hooks_weight(value)
        }

        #[pallet::call_index(142)]
        #[pallet::weight({0})]
        pub fn set_base_node_burn_amount(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_base_node_burn_amount(value)
        }

        #[pallet::call_index(143)]
        #[pallet::weight({0})]
        pub fn set_node_burn_rates(origin: OriginFor<T>, min: u128, max: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_node_burn_rates(min, max)
        }

        #[pallet::call_index(144)]
        #[pallet::weight({0})]
        pub fn set_max_subnet_node_min_weight_decrease_reputation_threshold(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_subnet_node_min_weight_decrease_reputation_threshold(value)
        }

        #[pallet::call_index(145)]
        #[pallet::weight({0})]
        pub fn set_validator_reward_k(origin: OriginFor<T>, value: u64) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_validator_reward_k(value)
        }

        #[pallet::call_index(146)]
        #[pallet::weight({0})]
        pub fn set_validator_reward_midpoint(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_validator_reward_midpoint(value)
        }

        #[pallet::call_index(147)]
        #[pallet::weight({0})]
        pub fn set_attestor_reward_exponent(origin: OriginFor<T>, value: u64) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_attestor_reward_exponent(value)
        }

        #[pallet::call_index(148)]
        #[pallet::weight({0})]
        pub fn set_attestor_min_reward_factor(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_attestor_min_reward_factor(value)
        }

        #[pallet::call_index(149)]
        #[pallet::weight({0})]
        pub fn set_min_max_node_reputation(
            origin: OriginFor<T>,
            min: u128,
            max: u128,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_max_node_reputation(min, max)
        }

        #[pallet::call_index(150)]
        #[pallet::weight({0})]
        pub fn set_min_max_node_reputation_factor(
            origin: OriginFor<T>,
            min: u128,
            max: u128,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_max_node_reputation_factor(min, max)
        }

        #[pallet::call_index(151)]
        #[pallet::weight({0})]
        pub fn set_min_subnet_reputation(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_min_subnet_reputation(value)
        }

        #[pallet::call_index(152)]
        #[pallet::weight({0})]
        pub fn set_not_in_consensus_subnet_reputation_factor(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_not_in_consensus_subnet_reputation_factor(value)
        }

        #[pallet::call_index(153)]
        #[pallet::weight({0})]
        pub fn set_max_pause_epochs_subnet_reputation_factor(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_pause_epochs_subnet_reputation_factor(value)
        }

        #[pallet::call_index(154)]
        #[pallet::weight({0})]
        pub fn set_less_than_min_nodes_subnet_reputation_factor(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_less_than_min_nodes_subnet_reputation_factor(value)
        }

        #[pallet::call_index(155)]
        #[pallet::weight({0})]
        pub fn set_validator_proposal_absent_subnet_reputation_factor(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_validator_proposal_absent_subnet_reputation_factor(value)
        }

        #[pallet::call_index(156)]
        #[pallet::weight({0})]
        pub fn set_in_consensus_subnet_reputation_factor(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_in_consensus_subnet_reputation_factor(value)
        }

        #[pallet::call_index(157)]
        #[pallet::weight({0})]
        pub fn set_overwatch_weight_factor(origin: OriginFor<T>, value: u128) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_overwatch_weight_factor(value)
        }

        #[pallet::call_index(158)]
        #[pallet::weight({0})]
        pub fn set_max_emergency_validator_epochs_multiplier(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_emergency_validator_epochs_multiplier(value)
        }

        #[pallet::call_index(159)]
        #[pallet::weight({0})]
        pub fn set_max_emergency_subnet_nodes(origin: OriginFor<T>, value: u32) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_max_emergency_subnet_nodes(value)
        }

        #[pallet::call_index(160)]
        #[pallet::weight({0})]
        pub fn set_overwatch_stake_weight_factor(
            origin: OriginFor<T>,
            value: u128,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_overwatch_stake_weight_factor(value)
        }

        #[pallet::call_index(161)]
        #[pallet::weight({0})]
        pub fn set_subnet_weight_factors(
            origin: OriginFor<T>,
            value: SubnetWeightFactorsData,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_subnet_weight_factors(value)
        }

        #[pallet::call_index(162)]
        #[pallet::weight({0})]
        pub fn set_churn_limit_multipliers(
            origin: OriginFor<T>,
            min: u32,
            max: u32,
        ) -> DispatchResult {
            T::MajorityCollectiveOrigin::ensure_origin(origin)?;
            Self::do_set_churn_limit_multipliers(min, max)
        }
    }

    impl<T: Config> Pallet<T> {
        /// Register a new subnet on the network
        ///
        /// This function creates a new subnet and enters it into the registration phase.
        /// After registration, the subnet must accumulate sufficient nodes and delegate stake
        /// before it can be activated (see `do_activate_subnet`).
        ///
        /// # Arguments
        ///
        /// * `owner` - The account that owns and controls the subnet
        /// * `max_cost` - Maximum registration cost the owner is willing to pay (slippage protection)
        /// * `subnet_registration_data` - Subnet configuration data (see `RegistrationSubnetData`)
        ///
        /// # Registration Requirements
        ///
        /// ## 1. Uniqueness Constraints
        ///
        /// - **Unique Name**: Subnet name must not already exist in `SubnetName` storage
        /// - **Unique Repository**: Repository URL must not already exist in `SubnetRepo` storage
        ///
        /// ## 2. Network Capacity
        ///
        /// - **Max Subnets**: Total registered subnets must be less than `MaxSubnets + 1`
        ///   - Network allows one extra subnet (n+1) to facilitate subnet rotation
        ///   - If exceeded, subnets are removed during epoch preliminaries (see `do_epoch_preliminaries`)
        ///
        /// ## 3. Bootnode Configuration
        ///
        /// - **Non-Empty**: At least one bootnode must be provided
        /// - **Count Limit**: Number of bootnodes must not exceed `MaxBootnodes`
        /// - Bootnodes are P2P network entry points for subnet nodes and overwatchers
        ///
        /// ## 4. Stake Balance Configuration
        ///
        /// ### Minimum Stake Per Node
        /// - Must be >= `MinSubnetMinStake` (network-wide minimum)
        /// - Must be <= `MaxSubnetMinStake` (network-wide maximum)
        /// - This is the minimum stake required for each subnet node
        ///
        /// ### Maximum Stake Per Node
        /// - Must be <= `NetworkMaxStakeBalance` (prevents stake concentration)
        /// - Must be >= minimum stake (logical consistency)
        ///
        /// ## 5. Delegate Stake Rewards Configuration
        ///
        /// - **Percentage Range**: Must be between `MinDelegateStakePercentage` and `MaxDelegateStakePercentage`
        /// - **Percentage Cap**: Must not exceed 100% (`percentage_factor_as_u128()`)
        /// - Determines what percentage of subnet emissions go to delegate stakers vs node operators
        ///
        /// ## 6. Initial Node Operators (Whitelist)
        ///
        /// - **Minimum Count**: At least `MinSubnetNodes` coldkeys must be whitelisted
        /// - **Registration Slots**: Each coldkey must be allocated at least 1 registration slot
        /// - During registration period, only these coldkeys can register nodes
        /// - Removed upon activation (see `do_activate_subnet`)
        ///
        /// ## 7. Registration Cost (Dynamic Pricing)
        ///
        /// - **Cost Calculation**: Uses `get_current_registration_cost(block)` with:
        ///   - Exponential decay based on time since last registration
        ///   - Alpha parameter for concave decay curve
        ///   - Minimum floor price `MinRegistrationCost`
        /// - **Slippage Protection**: Actual cost must be <= `max_cost` parameter
        /// - **Cost Update**: New cost multiplied by `NewRegistrationCostMultiplier` for next registration
        /// - **Payment**: Cost sent to Treasury, reverts on failure
        ///
        /// # Slot Assignment
        ///
        /// - **Unique Slot**: Each subnet is assigned a unique slot in the epoch schedule via `assign_subnet_slot()`
        /// - **Friendly UID**: Human-readable ID calculated as: `slot - DesignatedEpochSlots + 1`
        /// - **Designated Slots**: First 3 slots reserved for:
        ///   - Slot 0: Validator elections
        ///   - Slot 1: Overwatch weight calculations
        ///   - Slot 2: Emission weight calculations
        ///
        /// # Storage Updates
        ///
        /// The following storage items are initialized:
        ///
        /// - `SubnetsData` - Core subnet metadata (name, repo, description, state, etc.)
        /// - `SubnetOwner` - Owner account
        /// - `SubnetMinStakeBalance` / `SubnetMaxStakeBalance` - Stake limits
        /// - `SubnetDelegateStakeRewardsPercentage` - Reward split configuration
        /// - `LastSubnetDelegateStakeRewardsUpdate` - Timestamp for reward calculations
        /// - `SubnetRegistrationInitialColdkeys` - Whitelisted node operators (temporary)
        /// - `SubnetBootnodes` - P2P network entry points
        /// - `SubnetName` / `SubnetRepo` - Reverse lookups for uniqueness
        /// - `SubnetKeyTypes` - Cryptographic key types allowed
        /// - `SubnetRegistrationEpoch` - Registration timestamp (temporary, removed on activation)
        /// - `TotalSubnetUids` - Incremented to generate unique subnet ID
        ///
        /// # Post-Registration Flow
        ///
        /// After registration, the subnet enters the **Registration Period**:
        ///
        /// 1. **Registration Phase** (`SubnetRegistrationEpochs` duration):
        ///    - Whitelisted coldkeys register nodes
        ///    - Users delegate stake to the subnet
        ///    - Must wait at least `MinSubnetRegistrationEpochs` before activation attempt, but can
        ///      activate after the `MinSubnetRegistrationEpochs` in the registration phase.
        ///
        /// 2. **Enactment Phase** (`SubnetEnactmentEpochs` duration):
        ///    - Grace period after registration phase
        ///    - No new node registrations allowed
        ///    - Delegate staking continues
        ///    - Subnet must activate before this period ends
        ///
        /// 3. **Activation** (see `do_activate_subnet`):
        ///    - Must meet minimum requirements:
        ///      - Sufficient delegate stake (`get_min_subnet_delegate_stake_balance`)
        ///      - Minimum node count (`MinSubnetNodes`)
        ///      - Minimum reputation (`MinSubnetReputation`)
        ///    - If requirements not met by end of enactment period, subnet is removed
        ///
        /// # Events
        ///
        /// Emits `SubnetRegistered` event with:
        /// - `owner` - Subnet owner account
        /// - `name` - Subnet name
        /// - `subnet_id` - Unique subnet identifier
        ///
        /// # Errors
        ///
        /// - `SubnetNameExist` - Name already taken
        /// - `SubnetRepoExist` - Repository URL already taken
        /// - `MaxSubnets` - Network at capacity (> MaxSubnets + 1)
        /// - `BootnodesEmpty` - No bootnodes provided
        /// - `TooManyBootnodes` - Exceeds `MaxBootnodes`
        /// - `InvalidSubnetMinStake` - Min stake outside allowed range
        /// - `InvalidSubnetMaxStake` - Max stake exceeds network maximum
        /// - `InvalidSubnetStakeParameters` - Min stake > max stake
        /// - `InvalidMinDelegateStakePercentage` - Delegate percentage out of range
        /// - `InvalidSubnetRegistrationInitialColdkeys` - Invalid whitelist (too few coldkeys or invalid slot counts)
        /// - `CostGreaterThanMaxCost` - Registration cost exceeds slippage tolerance
        /// - `NotEnoughBalanceToRegisterSubnet` - Owner lacks funds
        /// - `CouldNotConvertToBalance` - Balance conversion overflow
        /// - `NoAvailableSlots` - No epoch slots available for assignment
        ///
        pub fn do_register_subnet(
            owner: T::AccountId,
            max_cost: u128,
            subnet_registration_data: RegistrationSubnetData<T::AccountId>,
        ) -> DispatchResult {
            // Ensure name is unique
            ensure!(
                !SubnetName::<T>::contains_key(&subnet_registration_data.name),
                Error::<T>::SubnetNameExist
            );

            // Ensure there aren't max+1 subnets already registered
            // Network allows n+1 subnets
            // See `do_epoch_preliminaries` to learn how subnets are removed if
            // maximum subnets is exceeded
            ensure!(
                Self::get_total_subnets() < MaxSubnets::<T>::get().saturating_add(1),
                Error::<T>::MaxSubnets
            );

            // Ensure name is unique
            ensure!(
                !SubnetRepo::<T>::contains_key(&subnet_registration_data.repo),
                Error::<T>::SubnetRepoExist
            );

            // Ensure bootnodes is not empty
            ensure!(
                !subnet_registration_data.bootnodes.is_empty(),
                Error::<T>::BootnodesEmpty
            );

            ensure!(
                subnet_registration_data.bootnodes.len() as u32 <= MaxBootnodes::<T>::get(),
                Error::<T>::TooManyBootnodes
            );

            // Min stake must be between min-max min stake allowable
            ensure!(
                subnet_registration_data.min_stake >= MinSubnetMinStake::<T>::get()
                    && subnet_registration_data.min_stake <= MaxSubnetMinStake::<T>::get(),
                Error::<T>::InvalidSubnetMinStake
            );

            // Max stake must be below the network max
            ensure!(
                subnet_registration_data.max_stake <= NetworkMaxStakeBalance::<T>::get(),
                Error::<T>::InvalidSubnetMaxStake
            );

            // Min stake must be less than or equal to max stake
            ensure!(
                subnet_registration_data.min_stake <= subnet_registration_data.max_stake,
                Error::<T>::InvalidSubnetStakeParameters
            );

            ensure!(
                subnet_registration_data.delegate_stake_percentage
                    >= MinDelegateStakePercentage::<T>::get()
                    && subnet_registration_data.delegate_stake_percentage
                        <= MaxDelegateStakePercentage::<T>::get()
                    && subnet_registration_data.delegate_stake_percentage
                        <= Self::percentage_factor_as_u128(),
                Error::<T>::InvalidMinDelegateStakePercentage
            );

            // --- Must have at least min subnet nodes as initial coldkeys
            // Each coldkey must have at least 1 available registration slot
            ensure!(
                subnet_registration_data
                    .initial_coldkeys
                    .values()
                    .all(|&value| value >= 1)
                    && subnet_registration_data.initial_coldkeys.len() as u32
                        >= MinSubnetNodes::<T>::get(),
                Error::<T>::InvalidSubnetRegistrationInitialColdkeys
            );

            let block: u32 = Self::get_current_block_as_u32();
            let cost = Self::get_current_registration_cost(block);

            ensure!(max_cost >= cost, Error::<T>::CostGreaterThanMaxCost);

            Self::update_last_registration_cost(cost, block);

            if cost > 0 {
                let cost_as_balance = match Self::u128_to_balance(cost) {
                    Some(balance) => balance,
                    None => return Err(Error::<T>::CouldNotConvertToBalance.into()),
                };

                // Ensure user has the funds, give accurate information on errors
                ensure!(
                    Self::can_remove_balance_from_coldkey_account(&owner, cost_as_balance),
                    Error::<T>::NotEnoughBalanceToRegisterSubnet
                );

                // Send funds to Treasury and revert if failed
                Self::send_to_treasury(&owner, cost_as_balance)?;
            }

            // Get total subnets ever
            let subnet_uids: u32 = TotalSubnetUids::<T>::get();

            // Start the subnet_ids at 1
            let subnet_id = subnet_uids.saturating_add(1);
            // Increase total subnets. This is used for unique Subnet IDs
            TotalSubnetUids::<T>::put(subnet_id);

            let slot = Self::assign_subnet_slot(subnet_id)?;
            let friendly_uid = slot.saturating_sub(T::DesignatedEpochSlots::get()) + 1;

            SubnetIdFriendlyUid::<T>::insert(subnet_id, friendly_uid);
            FriendlyUidSubnetId::<T>::insert(friendly_uid, subnet_id);

            let subnet_data = SubnetData {
                id: subnet_id,
                friendly_id: friendly_uid,
                name: subnet_registration_data.name,
                repo: subnet_registration_data.repo,
                description: subnet_registration_data.description,
                misc: subnet_registration_data.misc,
                state: SubnetState::Registered,
                start_epoch: u32::MAX, // updates on activation
            };

            // Store subnet data
            SubnetsData::<T>::insert(subnet_id, &subnet_data);

            // Store owner
            SubnetOwner::<T>::insert(subnet_id, &owner);

            // Store the stake balance range
            SubnetMinStakeBalance::<T>::insert(subnet_id, subnet_registration_data.min_stake);
            SubnetMaxStakeBalance::<T>::insert(subnet_id, subnet_registration_data.max_stake);

            // Add delegate state ratio
            SubnetDelegateStakeRewardsPercentage::<T>::insert(
                subnet_id,
                subnet_registration_data.delegate_stake_percentage,
            );

            LastSubnetDelegateStakeRewardsUpdate::<T>::insert(subnet_id, block);

            // Store whitelisted coldkeys for registration period
            SubnetRegistrationInitialColdkeys::<T>::insert(
                subnet_id,
                subnet_registration_data.initial_coldkeys,
            );

            // Add bootnodes
            SubnetBootnodes::<T>::insert(subnet_id, subnet_registration_data.bootnodes);

            // Store unique name
            SubnetName::<T>::insert(&subnet_data.name, subnet_id);
            SubnetRepo::<T>::insert(&subnet_data.repo, subnet_id);
            SubnetKeyTypes::<T>::insert(subnet_id, subnet_registration_data.key_types);
            // Update latest registration epoch for all subnets
            // This is used for one subnet per registration phase

            // Store registration epoch temporarily
            // This is removed on activation
            SubnetRegistrationEpoch::<T>::insert(subnet_id, Self::get_current_epoch_as_u32());

            Self::deposit_event(Event::SubnetRegistered {
                owner: owner,
                name: subnet_data.name,
                subnet_id: subnet_id,
            });

            Ok(())
        }

        /// Activate a registered subnet or remove it if activation requirements are not met
        ///
        /// This function transitions a subnet from `Registered` state to `Active` state, enabling
        /// it to participate in consensus, earn emissions, and operate normally. The subnet must
        /// meet specific timing and quality requirements to activate.
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - The unique identifier of the subnet to activate
        ///
        /// # Activation State Machine
        ///
        /// ## Timeline Overview
        ///
        /// ```text
        /// Registration (t=0)
        ///     ↓
        ///     ├─── MinSubnetRegistrationEpochs ───┤ ← Too early to activate
        ///     │                                   │
        ///     │   ✓ Accumulate nodes              │
        ///     │   ✓ Accumulate delegate stake     │
        ///     │                                   │
        ///     ├───────────────────────────────────┤ ← Can start activating
        ///     │   REGISTRATION PERIOD             │
        ///     │   (SubnetRegistrationEpochs)      │
        ///     │   - Nodes can register            │
        ///     │   - Delegate staking allowed      │
        ///     │   - Activation attempts allowed   │
        ///     │   - Failures return error         │
        ///     └───────────────────────────────────┘
        ///             ↓
        ///     ┌─── ENACTMENT PERIOD ───────────────┐
        ///     │   (SubnetEnactmentEpochs)          │
        ///     │   ⚠️  NO new node registrations    │
        ///     │   ✓  Delegate staking continues    │
        ///     │   ✓  Activation attempts allowed   │
        ///     │   ⚠️  Failures remove subnet       │
        ///     └────────────────────────────────────┘
        ///             ↓
        ///         ❌ TOO LATE
        ///         Subnet removed (EnactmentPeriod)
        /// ```
        ///
        /// # Timing Requirements
        ///
        /// ## 1. Minimum Registration Epochs
        ///
        /// - **Check**: `current_epoch >= registered_epoch + MinSubnetRegistrationEpochs`
        /// - **Purpose**: Prevents premature activation before subnet has had time to accumulate resources
        /// - **Note**: `MinSubnetRegistrationEpochs` < `SubnetRegistrationEpochs` (always less than full registration period)
        /// - **Error**: `MinSubnetRegistrationEpochsNotMet` if attempted too early
        ///
        /// ## 2. Valid Activation Period
        ///
        /// Must be in one of two valid periods:
        ///
        /// ### Registration Period
        /// - **Duration**: From registration to `registered_epoch + SubnetRegistrationEpochs`
        /// - **Checked by**: `is_subnet_registering(subnet_id, state, epoch)`
        /// - **Behavior**: Failed activation attempts return **error** (not removal)
        /// - **Allows**: Node registrations + delegate staking
        ///
        /// ### Enactment Period (Grace Period)
        /// - **Duration**: From end of registration to `registered_epoch + SubnetRegistrationEpochs + SubnetEnactmentEpochs`
        /// - **Checked by**: `is_subnet_in_enactment(subnet_id, state, epoch)`
        /// - **Behavior**: Failed activation attempts **remove subnet**
        /// - **Allows**: Delegate staking only (NO new node registrations)
        ///
        /// # Activation Requirements
        ///
        /// Checked by `can_subnet_be_active(subnet_id)` - ALL must be satisfied:
        ///
        /// ## 1. Minimum Subnet Reputation (Redundant)
        ///
        /// - **Check**: `SubnetReputation >= MinSubnetReputation`
        /// - **Default**: Usually initialized to minimum value on registration
        /// - **Removal Reason**: `SubnetRemovalReason::MinReputation`
        ///
        /// ## 2. Minimum Active Nodes
        ///
        /// - **Check**: `TotalActiveSubnetNodes >= MinSubnetNodes`
        /// - **Typical**: 3 nodes minimum
        /// - **Note**: Counts active nodes, not just registered nodes
        /// - **Removal Reason**: `SubnetRemovalReason::MinSubnetNodes`
        ///
        /// ## 3. Minimum Delegate Stake Balance
        ///
        /// - **Check**: `TotalSubnetDelegateStakeBalance >= get_min_subnet_delegate_stake_balance(subnet_id)`
        /// - **Calculation**: Base percentage of total network issuance, scaled by node count
        ///   - Base: `total_issuance * MinSubnetDelegateStakeFactor` (e.g., 0.1%)
        ///   - Multiplier: Linear scale from 100% (at MinSubnetNodes) to MaxMinDelegateStakeMultiplier (at MaxSubnetNodes)
        /// - **Dynamic**: Minimum increases as subnet adds more nodes
        /// - **Removal Reason**: `SubnetRemovalReason::MinSubnetDelegateStake`
        ///
        /// # Activation Outcomes (4 Cases)
        ///
        /// ## Case 1: Outside All Valid Periods
        ///
        /// - **Condition**: `!in_registration_period && !in_enactment_period`
        /// - **Action**: Remove subnet
        /// - **Reason**: `SubnetRemovalReason::EnactmentPeriod`
        /// - **Rationale**: Owner missed the activation deadline
        /// - **Returns**: `Ok(weight)` after cleanup
        ///
        /// ## Case 2: In Registration Period, Can't Activate
        ///
        /// - **Condition**: `!can_subnet_be_active && in_registration_period`
        /// - **Action**: Return error (no removal)
        /// - **Error**: `SubnetActivationConditionsNotMetYet`
        /// - **Rationale**: Still time to accumulate resources, allow retry
        /// - **User Action**: Add more nodes or delegate stake, retry later
        ///
        /// ## Case 3: In Enactment Period, Can't Activate
        ///
        /// - **Condition**: `!can_subnet_be_active && in_enactment_period`
        /// - **Action**: Remove subnet
        /// - **Reason**: Specific failure reason (MinReputation, MinSubnetNodes, or MinSubnetDelegateStake)
        /// - **Rationale**: Grace period expired, subnet failed to meet requirements
        /// - **Returns**: `Ok(weight)` after cleanup
        ///
        /// ## Case 4: Can Activate (Success Path)
        ///
        /// - **Condition**: `can_subnet_be_active && (in_registration_period || in_enactment_period)`
        /// - **Action**: Activate subnet
        /// - **Returns**: `Ok(weight)` after activation
        ///
        /// # Activation Process
        ///
        /// When activation succeeds, the following occurs:
        ///
        /// ## State Transitions
        ///
        /// 1. **Subnet State**: `SubnetState::Registered` → `SubnetState::Active`
        /// 2. **Start Epoch**: Set to `current_epoch + 1` (consensus begins next epoch)
        /// 3. **Total Active Subnets**: Incremented by 1
        ///
        /// ## Storage Cleanup (Temporary Registration Data)
        ///
        /// - `SubnetRegistrationEpoch` - Removed (no longer needed)
        /// - `SubnetRegistrationInitialColdkeys` - Removed (whitelist not needed after activation)
        /// - `InitialColdkeyData` - Removed (registration tracking data)
        ///
        /// ## Initialization of Active Subnet State
        ///
        /// - `LastSubnetDelegateStakeRewardsUpdate` - Set to current block
        /// - `PreviousSubnetPauseEpoch` - Set to current epoch (for pause logic)
        /// - `PrevSubnetActivationEpoch` - Set to current epoch (network-wide tracking)
        ///
        /// # Subnet Removal
        ///
        /// When a subnet is removed (Cases 1 or 3), `do_remove_subnet()` is called which:
        ///
        /// - Removes all subnet data and configuration
        /// - Cleans up all registered nodes (see `clean_subnet_nodes`)
        /// - Frees the epoch slot for reuse
        /// - Emits `SubnetDeactivated` event with removal reason
        ///
        /// **Important**: Delegate stakers must unstake manually after removal as they won't receive rewards
        ///
        /// # Weight Accounting
        ///
        /// This function carefully tracks database reads/writes for proper weight calculation:
        ///
        /// - **Reads**: SubnetsData(1), SubnetRegistrationEpoch(2), MinSubnetRegistrationEpochs(1), others(2) = ~6+
        /// - **Writes**: SubnetsData(1), TotalActiveSubnets(1), removal of 3 items, initialization of 3 items = 7
        /// - **Additional**: Weight from `do_remove_subnet` if removal occurs
        ///
        /// # Events
        ///
        /// - **Success**: `SubnetActivated { subnet_id }`
        /// - **Removal**: `SubnetDeactivated { subnet_id, reason }` (emitted by `do_remove_subnet`)
        ///
        /// # Errors
        ///
        /// - `InvalidSubnetId` - Subnet does not exist
        /// - `SubnetActivatedAlready` - Subnet is already in Active or Paused state (not Registered)
        /// - `MinSubnetRegistrationEpochsNotMet` - Too early to activate (before minimum registration period)
        /// - `SubnetActivationConditionsNotMetYet` - Requirements not met but still in registration period (retry allowed)
        ///
        /// # Notes
        ///
        /// - This function can only be called by the owner by `activate_subnet()`
        /// - Can be called multiple times during registration period (retries allowed)
        /// - After successful activation, subnet enters consensus on the next epoch (`start_epoch + 1`)
        /// - Removal is permanent - owner must re-register and pay registration cost again and restart the process
        ///
        pub fn do_activate_subnet(subnet_id: u32) -> DispatchResultWithPostInfo {
            let mut weight = Weight::zero();
            let db_weight = T::DbWeight::get();

            let subnet = match SubnetsData::<T>::try_get(subnet_id) {
                Ok(subnet) => subnet,
                Err(()) => return Err(Error::<T>::InvalidSubnetId.into()),
            };
            weight = weight.saturating_add(db_weight.reads(1));

            ensure!(
                subnet.state == SubnetState::Registered,
                Error::<T>::SubnetActivatedAlready
            );

            let epoch: u32 = Self::get_current_epoch_as_u32();

            // SubnetRegistrationEpoch
            weight = weight.saturating_add(db_weight.reads(1));

            let past_min_registration_epochs =
                if let Ok(registered_epoch) = SubnetRegistrationEpoch::<T>::try_get(subnet_id) {
                    let min_epochs = MinSubnetRegistrationEpochs::<T>::get();
                    // MinSubnetRegistrationEpochs
                    weight = weight.saturating_add(db_weight.reads(1));
                    let min_registration_epoch = registered_epoch.saturating_add(min_epochs);
                    min_registration_epoch <= epoch
                } else {
                    false
                };

            // --- Minimum registration epochs not met yet
            // This is the period within the registration period where the subnet CANNOT attempt to activate yet
            // Note: There can be a minimum registration epochs that must be met before the subnet can attempt to activate
            //       This is always less than the registration phase period (see `is_subnet_registering`)
            ensure!(
                past_min_registration_epochs,
                Error::<T>::MinSubnetRegistrationEpochsNotMet
            );

            // If in the registration period
            let in_registration_period =
                Self::is_subnet_registering(subnet_id, subnet.state, epoch);

            // Can subnet activate:
            // - Min delegate stake
            // - Min node count
            let (can_subnet_be_active, reason) = Self::can_subnet_be_active(subnet_id);

            // If in the enactment period
            let in_enactment_period = Self::is_subnet_in_enactment(subnet_id, subnet.state, epoch);

            // Case 1: Outside all valid periods (passed enactment period)
            if !in_registration_period && !in_enactment_period {
                weight = weight.saturating_add(Self::do_remove_subnet(
                    subnet_id,
                    SubnetRemovalReason::EnactmentPeriod,
                ));
                return Ok(Some(weight).into());
            }

            // Case 2: Can't activate while in registration period
            // We allow them to attempt to activate without removing the subnet
            if !can_subnet_be_active && in_registration_period {
                return Err(Error::<T>::SubnetActivationConditionsNotMetYet.into());
            }

            // Case 3: Remove subnet if in enactment period and can't activate
            if let (true, Some(removal_reason)) =
                (!can_subnet_be_active && in_enactment_period, reason.clone())
            {
                weight = weight.saturating_add(Self::do_remove_subnet(subnet_id, removal_reason));
                return Ok(Some(weight).into());
            }

            // Case 4: Can activate (implicit: can_subnet_be_active = true, in valid period)

            // ===============
            // Gauntlet passed
            // ===============

            // --- Activate subnet
            // Subnet start_epoch uses general blockchain epoch (not subnet epoch)
            SubnetsData::<T>::try_mutate(subnet_id, |maybe_params| -> DispatchResult {
                let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;
                params.state = SubnetState::Active;
                // Start consensus after 1 fresh epoch.
                // Consensus starts once epoch > start_epoch
                params.start_epoch = epoch + 1;
                Ok(())
            })?;

            TotalActiveSubnets::<T>::mutate(|n: &mut u32| *n += 1);

            // --- Remove SubnetRegistrationEpoch as it is no longer required
            // Used in:
            //  - do_epoch_preliminaries
            //  - is_subnet_registering
            //  - is_subnet_in_enactment
            SubnetRegistrationEpoch::<T>::remove(subnet_id);

            // --- Remove registration whitelist
            SubnetRegistrationInitialColdkeys::<T>::remove(subnet_id);
            InitialColdkeyData::<T>::remove(subnet_id);

            // --- Set most recent block
            LastSubnetDelegateStakeRewardsUpdate::<T>::insert(
                subnet_id,
                Self::get_current_block_as_u32(),
            );

            // --- Set pause epoch now to abide by pause logic
            PreviousSubnetPauseEpoch::<T>::insert(subnet_id, epoch);

            // --- Set most recent activation epoch
            PrevSubnetActivationEpoch::<T>::set(epoch);

            // SubnetsData | TotalActiveSubnets | SubnetRegistrationEpoch |
            // SubnetRegistrationInitialColdkeys | LastSubnetDelegateStakeRewardsUpdate
            // PreviousSubnetPauseEpoch | PrevSubnetActivationEpoch
            weight = weight.saturating_add(db_weight.writes(7));
            // SubnetsData | TotalActiveSubnets
            weight = weight.saturating_add(db_weight.reads(2));

            Self::deposit_event(Event::SubnetActivated {
                subnet_id: subnet_id,
            });

            Ok(Some(weight).into())
        }

        /// Try to remove a subnet
        /// This is called by `do_epoch_preliminaries`
        /// We ensure there is enough block weight to call to remove a subnet
        pub fn try_do_remove_subnet(
            weight_meter: &mut WeightMeter,
            subnet_id: u32,
            reason: SubnetRemovalReason,
        ) {
            // TotalSubnetNodes
            weight_meter.consume(T::DbWeight::get().reads(1));
            // See if we can call to remove a subnet
            if !weight_meter.can_consume(T::WeightInfo::do_remove_subnet(
                TotalSubnetNodes::<T>::get(subnet_id),
            )) {
                return;
            }

            let weight = Self::do_remove_subnet(subnet_id, reason);
            weight_meter.consume(weight);
        }

        /// Remove a subnet and clean up all associated storage
        ///
        /// This function permanently removes a subnet from the network, cleaning up all configuration,
        /// node data, and associated storage. This is called when a subnet fails to meet activation
        /// requirements or is explicitly removed by governance.
        ///
        /// # Arguments
        ///
        /// * `subnet_id` - The unique identifier of the subnet to remove
        /// * `reason` - The reason for removal (see `SubnetRemovalReason`)
        ///
        /// # Removal Reasons
        ///
        /// ## Activation-Related Removals
        ///
        /// - **`EnactmentPeriod`**: Subnet owner missed the activation deadline (passed enactment period end)
        /// - **`MinReputation`**: Subnet failed to maintain minimum reputation requirement
        /// - **`MinSubnetNodes`**: Subnet failed to meet minimum node count (typically 3)
        /// - **`MinSubnetDelegateStake`**: Insufficient delegate stake balance
        ///
        /// ## Operational Removals
        ///
        /// - **`MaxPauseEpochs`**: Subnet was paused for too long (exceeded maximum pause duration)
        /// - **`NotInConsensus`**: Subnet consistently failed to achieve consensus
        /// - **`LessThanMinNodes`**: Active node count dropped below minimum threshold
        /// - **`ValidatorProposalAbsent`**: Validator failed to submit proposals consistently
        ///
        /// ## Governance Removals
        ///
        /// - **`Owner`**: Subnet owner chose to remove their subnet
        /// - **`Collective`**: Governance collective voted to remove the subnet
        ///
        /// # Removal Process
        ///
        /// The subnet is removed in the following order:
        ///
        /// ## 1. Core Subnet Data (6 items)
        ///
        /// - `SubnetsData` - Core metadata (name, repo, description, state, etc.)
        /// - `SubnetName` - Reverse lookup by name (uniqueness constraint)
        /// - `SubnetRepo` - Reverse lookup by repository URL (uniqueness constraint)
        /// - `SubnetOwner` - Subnet owner account
        /// - `PendingSubnetOwner` - Pending ownership transfer (if any)
        /// - `SubnetRegistrationEpoch` - Registration timestamp (if still registered)
        ///
        /// ## 2. Subnet Configuration Parameters (27 items)
        ///
        /// ### Operational Parameters
        /// - `ChurnLimit` - Maximum nodes that can enter/exit per epoch
        /// - `ChurnLimitMultiplier` - Churn limit scaling factor
        /// - `SubnetNodeQueueEpochs` - Epochs nodes must wait in queue
        /// - `IdleClassificationEpochs` - Epochs before marking nodes idle
        /// - `IncludedClassificationEpochs` - Epochs for inclusion classification
        /// - `QueueImmunityEpochs` - Protection period for queued nodes
        /// - `MaxRegisteredNodes` - Maximum allowed registered nodes
        /// - `TargetNodeRegistrationsPerEpoch` - Target registration rate
        /// - `NodeBurnRateAlpha` - Registration cost decay parameter
        /// - `CurrentNodeBurnRate` - Current dynamic registration cost
        /// - `NodeRegistrationsThisEpoch` - Tracking counter for this epoch
        ///
        /// ### Stake Configuration
        /// - `SubnetMinStakeBalance` - Minimum stake per node
        /// - `SubnetMaxStakeBalance` - Maximum stake per node
        /// - `SubnetDelegateStakeRewardsPercentage` - Delegate reward split
        /// - `LastSubnetDelegateStakeRewardsUpdate` - Last reward distribution timestamp
        ///
        /// ### Registration Data (Temporary)
        /// - `SubnetRegistrationInitialColdkeys` - Whitelisted node operators
        /// - `InitialColdkeyData` - Registration tracking data
        ///
        /// ### Network Configuration
        /// - `SubnetBootnodes` - P2P network entry points
        /// - `SubnetBootnodeAccess` - Accounts allowed to update bootnodes
        /// - `SubnetKeyTypes` - Allowed cryptographic key types
        ///
        /// ### Reputation System
        /// - `SubnetReputation` - Overall subnet reputation score
        /// - `MinSubnetNodeReputation` - Minimum required node reputation
        /// - `SubnetNodeMinWeightDecreaseReputationThreshold` - Weight threshold for penalties
        /// - `AbsentDecreaseReputationFactor` - Penalty for absent nodes
        /// - `IncludedIncreaseReputationFactor` - Reward for included nodes
        /// - `BelowMinWeightDecreaseReputationFactor` - Penalty for low-weight nodes
        /// - `NonAttestorDecreaseReputationFactor` - Penalty for non-attesting nodes
        /// - `NonConsensusAttestorDecreaseReputationFactor` - Penalty for incorrect attestations
        /// - `ValidatorAbsentSubnetNodeReputationFactor` - Validator absence impact
        /// - `ValidatorNonConsensusSubnetNodeReputationFactor` - Validator consensus failure impact
        ///
        /// ### State Tracking
        /// - `PreviousSubnetPauseEpoch` - Last pause timestamp
        /// - `EmergencySubnetNodeElectionData` - Emergency validator election state
        ///
        /// ## 3. Subnet Identifiers (2 items)
        ///
        /// - `SubnetIdFriendlyUid` - Human-readable ID mapping
        /// - `FriendlyUidSubnetId` - Reverse lookup for friendly ID
        ///
        /// ## 4. Slot Assignment (3 items via `free_slot_of_subnet`)
        ///
        /// - `SubnetSlot` - Assigned epoch slot
        /// - `SlotAssignment` - Reverse lookup (slot → subnet)
        /// - `AssignedSlots` - Set of currently assigned slots (freed for reuse)
        ///
        /// ## 5. Active Subnet Counter
        ///
        /// - `TotalActiveSubnets` - Decremented if subnet was in Active state
        ///
        /// ## 6. All Node Data (via `clean_subnet_nodes`)
        ///
        /// Removes all subnet node data including:
        /// - `SubnetNodesData` - All node metadata (cleared via prefix)
        /// - `RegisteredSubnetNodesData` - Registration data (cleared via prefix)
        /// - `TotalActiveSubnetNodes` - Active node counter
        /// - `TotalSubnetNodes` - Total node counter
        /// - `TotalSubnetNodeUids` - Node ID counter
        /// - `TotalActiveNodes` - Global active node counter (decremented)
        /// - `PeerIdSubnetNodeId` - Peer ID mappings (cleared via prefix)
        /// - `BootnodePeerIdSubnetNodeId` - Bootnode peer mappings (cleared via prefix)
        /// - `ClientPeerIdSubnetNodeId` - Client peer mappings (cleared via prefix)
        /// - `BootnodeSubnetNodeId` - Bootnode node mappings (cleared via prefix)
        /// - `UniqueParamSubnetNodeId` - Unique parameter tracking (cleared via prefix)
        /// - `HotkeySubnetNodeId` - Hotkey to node ID mappings (cleared via prefix)
        /// - `SubnetNodeIdHotkey` - Reverse hotkey mappings (cleared via prefix)
        /// - `SubnetNodeReputation` - Individual node reputations (cleared via prefix)
        /// - `SubnetNodeConsecutiveIncludedEpochs` - Inclusion streaks (cleared via prefix)
        /// - `SubnetElectedValidator` - Validator election results (cleared via prefix)
        /// - `NodeSlotIndex` - Slot index mappings (cleared via prefix)
        /// - `SubnetNodeElectionSlots` - Election slot arrays
        /// - `SubnetNodeQueue` - Node queue
        /// - `TotalElectableNodes` - Global electable node counter (decremented)
        /// - `TotalSubnetElectableNodes` - Subnet electable node counter
        ///
        /// # Important: Data NOT Removed
        ///
        /// ## Consensus Submission Data (Preserved)
        ///
        /// - **`SubnetConsensusSubmission`**: Historical consensus data is **NOT** removed
        ///   - Preserves consensus history for auditing and analysis
        ///   - Does not impact blockchain logic for active subnets
        ///   - Can be queried for historical subnet performance
        ///
        /// ## Stake Data (Preserved - User Action Required)
        ///
        /// - **Node Stake Balances**: Individual node stakes are NOT automatically removed
        ///   - Node operators must call `remove_stake()` to reclaim stake
        ///   - Stake remains locked until manually unstaked
        ///   - Enters unbonding period upon unstaking
        ///
        /// - **Delegate Stake Balances**: Delegate stakes are NOT automatically removed
        ///   - Delegate stakers must call `remove_account_delegate_stake()` to reclaim stake
        ///   - Important: No rewards will be earned after subnet removal
        ///   - Users should monitor for subnet removals and unstake promptly
        ///
        /// - **Account Mappings**: Some account-related storage persists
        ///   - `ColdkeyHotkeys` - Coldkey to hotkey relationships
        ///   - `HotkeyOwner` - Hotkey ownership records
        ///   - `AccountSubnetStake` - Stake balances by account
        ///   - `TotalSubnetStake` - Total stake counters
        ///   - Cleaned up when stake is removed to zero
        ///
        /// # Weight Calculation
        ///
        /// This function carefully accounts for database operations:
        ///
        /// - **Base Reads**: 2 (SubnetsData check + state check)
        /// - **Base Writes**: 26 (core subnet data + configurations)
        /// - **Conditional Writes**: +1 (FriendlyUidSubnetId if exists)
        /// - **Slot Cleanup**: +1 read, +3 writes (via `free_slot_of_subnet`)
        /// - **Active Counter**: +1 read, +1 write (if subnet was active)
        /// - **Node Cleanup**: Variable (see `clean_subnet_nodes` - scales with node count)
        ///
        /// Total approximate weight: ~4 reads + ~30 writes + node cleanup weight
        ///
        /// # Events
        ///
        /// Emits `SubnetDeactivated` event with:
        /// - `subnet_id` - The removed subnet identifier
        /// - `reason` - The reason for removal (see `SubnetRemovalReason`)
        ///
        /// # Errors
        ///
        /// This function does not return errors. If the subnet doesn't exist, it returns
        /// early with minimal weight accounting (1 read). This is safe because:
        /// - Called from contexts that already validated subnet existence
        /// - Idempotent - safe to call multiple times
        /// - Weight is always returned for proper accounting
        ///
        /// # Usage Contexts
        ///
        /// This function is called from several contexts:
        ///
        /// ## Manual Removal (Immediate)
        ///
        /// ### 1. Owner Removal
        /// - **Path**: `owner_deactivate_subnet` extrinsic → `do_owner_deactivate_subnet` → `do_remove_subnet`
        /// - **Who**: Subnet owner only
        /// - **Reason**: `SubnetRemovalReason::Owner`
        /// - **When**: Any time owner chooses to shut down their subnet
        ///
        /// ### 2. Collective/Governance Removal
        /// - **Path**: `collective_remove_subnet` extrinsic → `do_collective_remove_subnet` → `do_remove_subnet`
        /// - **Who**: Governance collective (requires vote/approval)
        /// - **Reason**: `SubnetRemovalReason::Collective`
        /// - **When**: Governance decides subnet should be removed (e.g., malicious behavior, policy violation)
        ///
        /// ## Automatic Removal via Activation Process
        ///
        /// ### 3. Activation Failure During Enactment
        /// - **Path**: `activate_subnet` extrinsic → `do_activate_subnet` → `do_remove_subnet`
        /// - **Who**: Called by owner attempting to activate
        /// - **Reason**: `SubnetRemovalReason::MinReputation`, `MinSubnetNodes`, or `MinSubnetDelegateStake`
        /// - **When**: Owner attempts activation during enactment period but requirements not met
        ///
        /// ### 4. Missed Activation Deadline
        /// - **Path**: `activate_subnet` extrinsic → `do_activate_subnet` → `do_remove_subnet`
        /// - **Who**: Called by owner attempting to activate
        /// - **Reason**: `SubnetRemovalReason::EnactmentPeriod`
        /// - **When**: Owner attempts activation after enactment period has expired
        ///
        /// ## Automatic Removal via Epoch Preliminaries
        ///
        /// The `do_epoch_preliminaries` function runs at the start of each epoch and performs
        /// several health checks. Removal is triggered via `try_do_remove_subnet` (which calls
        /// this function with weight metering) in the following scenarios:
        ///
        /// ### 5. Enactment Period - Insufficient Nodes
        /// - **State**: Subnet in `Registered` state, within enactment period
        /// - **Condition**: `TotalActiveSubnetNodes < MinSubnetNodes` (typically < 3 nodes)
        /// - **Reason**: `SubnetRemovalReason::MinSubnetNodes`
        /// - **Rationale**: Even in grace period, must maintain minimum node count
        /// - **Note**: Delegate stake is NOT checked during enactment (users can still stake)
        ///
        /// ### 6. Failed to Activate (Post-Enactment)
        /// - **State**: Subnet in `Registered` state, past enactment period
        /// - **Condition**: `epoch > registered_epoch + SubnetRegistrationEpochs + SubnetEnactmentEpochs`
        /// - **Reason**: `SubnetRemovalReason::EnactmentPeriod`
        /// - **Rationale**: Owner failed to activate subnet within allowed time window
        ///
        /// ### 7. Paused Too Long with Low Reputation
        /// - **State**: Subnet in `Paused` state
        /// - **Condition**:
        ///   1. `start_epoch + MaxSubnetPauseEpochs < current_epoch` (paused beyond limit)
        ///   2. Reputation decreased by `MaxPauseEpochsSubnetReputationFactor`
        ///   3. Resulting reputation < `MinSubnetReputation`
        /// - **Reason**: `SubnetRemovalReason::PauseExpired`
        /// - **Rationale**: Subnet paused too long and reputation dropped too low
        /// - **Note**: Reputation is decreased first, then checked; subnet may survive if reputation stays above minimum
        ///
        /// ### 8. Insufficient Delegate Stake (Active Subnets)
        /// - **State**: Subnet in `Active` state
        /// - **Condition**:
        ///   1. `TotalSubnetDelegateStakeBalance < get_min_subnet_delegate_stake_balance(subnet_id)`
        ///   2. `epoch % DelegateStakeSubnetRemovalInterval == 0` (only on designated epochs)
        /// - **Reason**: `SubnetRemovalReason::MinSubnetDelegateStake`
        /// - **Rationale**: Insufficient community stake support, checked periodically to give time to recover
        /// - **Note**: Not checked every epoch - only on interval epochs to allow recovery time
        ///
        /// ### 9. Low Reputation (Active Subnets)
        /// - **State**: Subnet in `Active` state
        /// - **Condition**: `SubnetReputation < MinSubnetReputation`
        /// - **Reason**: `SubnetRemovalReason::MinReputation`
        /// - **Rationale**: Subnet consistently underperforming or failing health checks
        /// - **Note**: Reputation can be decreased by multiple factors:
        ///   - Low electable node count (decreases reputation but doesn't immediately remove)
        ///   - Consensus failures
        ///   - Validator absence
        ///   - Other runtime logic
        ///
        /// ### 10. Excess Subnets (Lowest Stake Removal)
        /// - **State**: Subnet in `Active` state
        /// - **Condition**:
        ///   1. `total_subnets > MaxSubnets`
        ///   2. `epoch % MaxSubnetRemovalInterval == 0` (designated removal epochs)
        ///   3. `epoch >= PrevSubnetActivationEpoch + MinSubnetRemovalInterval` (cooldown period)
        ///   4. Subnet has lowest `TotalSubnetDelegateStakeBalance` among all active subnets
        /// - **Reason**: `SubnetRemovalReason::MaxSubnets`
        /// - **Rationale**: Network at capacity, least-supported subnet removed
        /// - **Note**: Network allows `MaxSubnets + 1` to facilitate rotation; weakest removed periodically
        ///
        /// ## Epoch Preliminaries Removal Logic Summary
        ///
        /// ```text
        /// For each subnet:
        ///   IF state == Registered:
        ///     IF in registration period → Skip (no checks)
        ///     IF in enactment period:
        ///       IF active_nodes < min → Remove (MinSubnetNodes)
        ///     IF past enactment period → Remove (EnactmentPeriod)
        ///   
        ///   IF state == Paused:
        ///     IF paused_too_long:
        ///       Decrease reputation
        ///       IF reputation < min → Remove (PauseExpired)
        ///   
        ///   IF state == Active:
        ///     IF delegate_stake < min AND is_dstake_epoch → Remove (MinSubnetDelegateStake)
        ///     IF electable_nodes < min → Decrease reputation (NOT removed)
        ///     IF reputation < min → Remove (MinReputation)
        ///     IF excess_subnets AND is_removal_epoch AND can_remove → Track for removal
        ///
        /// IF excess subnets:
        ///   Remove subnet with lowest delegate stake (MaxSubnets)
        /// ```
        ///
        /// # Post-Removal Actions Required
        ///
        /// After a subnet is removed, users must take action:
        ///
        /// 1. **Node Operators**: Call `remove_stake()` to unstake and reclaim funds
        /// 2. **Delegate Stakers**: Call `remove_account_delegate_stake()` to reclaim delegate stake
        /// 3. **Owner**: Can re-register subnet (must pay registration cost again)
        ///
        /// # Notes
        ///
        /// - Removal is **permanent** and **immediate** (no grace period)
        /// - Subnet ID can be reused after sufficient time
        /// - Epoch slot is freed for assignment to new subnets
        /// - Historical consensus data is preserved for analysis
        /// - Total subnet count is not decremented (only active count)
        /// - Removal during registration period returns all registration costs (governance decision)
        ///
        pub fn do_remove_subnet(subnet_id: u32, reason: SubnetRemovalReason) -> Weight {
            let mut weight = Weight::zero();
            let db_weight = T::DbWeight::get();

            weight = weight.saturating_add(db_weight.reads(1));
            let subnet = match SubnetsData::<T>::try_get(subnet_id) {
                Ok(subnet) => subnet,
                Err(()) => return weight,
            };

            // Remove unique name
            SubnetName::<T>::remove(&subnet.name);
            SubnetRepo::<T>::remove(&subnet.repo);
            // Remove subnet data
            SubnetsData::<T>::remove(subnet_id);

            // Remove owner elements
            SubnetOwner::<T>::remove(subnet_id);
            PendingSubnetOwner::<T>::remove(subnet_id);

            SubnetRegistrationEpoch::<T>::remove(subnet_id);
            PreviousSubnetPauseEpoch::<T>::remove(subnet_id);

            // Subnet parameters
            ChurnLimit::<T>::remove(subnet_id);
            ChurnLimitMultiplier::<T>::remove(subnet_id);
            SubnetNodeQueueEpochs::<T>::remove(subnet_id);
            IdleClassificationEpochs::<T>::remove(subnet_id);
            IncludedClassificationEpochs::<T>::remove(subnet_id);
            SubnetMinStakeBalance::<T>::remove(subnet_id);
            SubnetMaxStakeBalance::<T>::remove(subnet_id);
            SubnetDelegateStakeRewardsPercentage::<T>::remove(subnet_id);
            LastSubnetDelegateStakeRewardsUpdate::<T>::remove(subnet_id);
            SubnetRegistrationInitialColdkeys::<T>::remove(subnet_id);
            InitialColdkeyData::<T>::remove(subnet_id);
            MaxRegisteredNodes::<T>::remove(subnet_id);
            SubnetKeyTypes::<T>::remove(subnet_id);
            TargetNodeRegistrationsPerEpoch::<T>::remove(subnet_id);
            NodeBurnRateAlpha::<T>::remove(subnet_id);
            CurrentNodeBurnRate::<T>::remove(subnet_id);
            QueueImmunityEpochs::<T>::remove(subnet_id);
            SubnetBootnodeAccess::<T>::remove(subnet_id);
            SubnetBootnodes::<T>::remove(subnet_id);
            EmergencySubnetNodeElectionData::<T>::remove(subnet_id);
            SubnetReputation::<T>::remove(subnet_id);
            MinSubnetNodeReputation::<T>::remove(subnet_id);
            NodeRegistrationsThisEpoch::<T>::remove(subnet_id);
            SubnetNodeMinWeightDecreaseReputationThreshold::<T>::remove(subnet_id);
            AbsentDecreaseReputationFactor::<T>::remove(subnet_id);
            IncludedIncreaseReputationFactor::<T>::remove(subnet_id);
            BelowMinWeightDecreaseReputationFactor::<T>::remove(subnet_id);
            NonAttestorDecreaseReputationFactor::<T>::remove(subnet_id);
            NonConsensusAttestorDecreaseReputationFactor::<T>::remove(subnet_id);
            ValidatorAbsentSubnetNodeReputationFactor::<T>::remove(subnet_id);
            ValidatorNonConsensusSubnetNodeReputationFactor::<T>::remove(subnet_id);

            if let Some(friendly_uid) = SubnetIdFriendlyUid::<T>::take(subnet_id) {
                FriendlyUidSubnetId::<T>::remove(friendly_uid);
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }

            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 26));

            // Remove from slot
            Self::free_slot_of_subnet(subnet_id);
            // Add weight here of `free_slot_of_subnet`
            // reads:
            // AssignedSlots
            // writes:
            // SubnetSlot | SlotAssignment | AssignedSlots
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 3));

            if subnet.state == SubnetState::Active {
                // Dec total active subnets, if active
                // Note: We don't have a TotalSubnets storage elements
                //       When counting how many subnets there are, we iter() `SubnetsData`
                TotalActiveSubnets::<T>::mutate(|n: &mut u32| n.saturating_dec());
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            }

            // We have removed all of the data required to assist in blockchain logic
            // `clean_subnet_nodes` cleans up non-required data
            // let _ = Self::clean_subnet_nodes(subnet_id);
            weight = weight.saturating_add(Self::clean_subnet_nodes(subnet_id));

            Self::deposit_event(Event::SubnetDeactivated {
                subnet_id: subnet_id,
                reason: reason,
            });

            weight
        }

        // Only called from `do_remove_subnet`
        // If we call this anywhere else, must include a way to ensure subnet exists
        // Note that `HotkeySubnetId` `ColdkeyHotkeys` `HotkeyOwner` are removed when
        // the node stake balance hits 0, plus `ColdkeySubnetNodes` is filtered.
        // `ColdkeySubnetNodes` is filtered on each node registration as well via
        // `clean_coldkey_subnet_nodes`
        pub fn clean_subnet_nodes(subnet_id: u32) -> Weight {
            let mut weight_acc = WeightAccumulator::<T>::new();

            // Remove all subnet nodes data
            let removed_subnet_nodes_data =
                SubnetNodesData::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(removed_subnet_nodes_data.unique);

            let total_nodes = TotalActiveSubnetNodes::<T>::take(subnet_id);
            weight_acc.add_take();

            TotalActiveNodes::<T>::mutate(|n: &mut u32| n.saturating_reduce(total_nodes));
            weight_acc.add_mutate();

            let _ = TotalSubnetNodes::<T>::remove(subnet_id);
            weight_acc.add_remove();

            let _ = TotalSubnetNodeUids::<T>::remove(subnet_id);
            weight_acc.add_remove();

            let peer_id_subnet_node_id_removed =
                PeerIdSubnetNodeId::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(peer_id_subnet_node_id_removed.unique);

            let bootnode_peer_id_subnet_node_id_removed =
                BootnodePeerIdSubnetNodeId::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(bootnode_peer_id_subnet_node_id_removed.unique);

            let client_peer_id_subnet_node_id_removed =
                ClientPeerIdSubnetNodeId::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(client_peer_id_subnet_node_id_removed.unique);

            let bootnode_subnet_node_id_removed =
                BootnodeSubnetNodeId::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(bootnode_subnet_node_id_removed.unique);

            let subnet_node_unique_param_removed =
                UniqueParamSubnetNodeId::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(subnet_node_unique_param_removed.unique);

            let hotkey_subnet_node_id_removed =
                HotkeySubnetNodeId::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(hotkey_subnet_node_id_removed.unique);

            let subnet_node_is_hotkey_removed =
                SubnetNodeIdHotkey::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(subnet_node_is_hotkey_removed.unique);

            let subnet_node_reputations =
                SubnetNodeReputation::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(subnet_node_reputations.unique);

            let subnet_node_consecutive_included_epochs_removed =
                SubnetNodeConsecutiveIncludedEpochs::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(subnet_node_consecutive_included_epochs_removed.unique);
            let registered_subnet_nodes_data_removed =
                RegisteredSubnetNodesData::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(registered_subnet_nodes_data_removed.unique);

            let subnet_elected_validator =
                SubnetElectedValidator::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(subnet_elected_validator.unique);

            let node_slot_index_removed =
                NodeSlotIndex::<T>::clear_prefix(subnet_id, u32::MAX, None);
            weight_acc.add_clear_prefix(node_slot_index_removed.unique);

            let electable_nodes = SubnetNodeElectionSlots::<T>::take(subnet_id).len() as u32;
            weight_acc.add_take();

            // Vec size impacts weight, add extra based on length
            let vec_len = electable_nodes as u64;
            if vec_len > 1 {
                // Add extra computational weight for processing larger vectors
                weight_acc.add_computational_weight(vec_len * 1000);
            }

            let queue = SubnetNodeQueue::<T>::take(subnet_id).len() as u32;
            weight_acc.add_take();

            // Vec size impacts weight, add extra based on length
            let vec_len = queue as u64;
            if vec_len > 1 {
                // Add extra computational weight for processing larger vectors
                weight_acc.add_computational_weight(vec_len * 1000);
            }

            // Sub total electable nodes network wide
            TotalElectableNodes::<T>::mutate(|mut n| n.saturating_sub(electable_nodes));
            weight_acc.add_mutate();

            TotalSubnetElectableNodes::<T>::remove(subnet_id);
            weight_acc.add_remove();

            let final_weight = weight_acc.finalize();
            final_weight
        }

        pub fn do_remove_subnet_node(subnet_id: u32, subnet_node_id: u32) -> DispatchResult {
            Self::perform_remove_subnet_node(subnet_id, subnet_node_id);
            Ok(())
        }

        /// Register a new subnet node (validator/miner) to a subnet
        ///
        /// This function registers a new node to a subnet, enabling it to participate in consensus,
        /// validation, or compute work. The node must meet numerous requirements and pay a dynamic
        /// burn fee. The registration process differs depending on whether the subnet is in the
        /// registration period (auto-activation) or active state (queued entry).
        ///
        /// # Arguments
        ///
        /// * `origin` - The coldkey account registering the node
        /// * `subnet_id` - The subnet to register the node to
        /// * `hotkey` - Unique hotkey account for this node (must be network-wide unique)
        /// * `peer_id` - Libp2p peer ID for P2P communication
        /// * `bootnode_peer_id` - Libp2p peer ID for bootnode connections
        /// * `client_peer_id` - Libp2p peer ID for client connections
        /// * `bootnode` - Optional bootnode multiaddr for network discovery
        /// * `delegate_reward_rate` - Percentage of rewards shared with delegators (0-100%)
        /// * `stake_to_be_added` - Initial stake amount (must meet minimum requirements)
        /// * `unique` - Optional unique parameter for node identification (subnet-wide unique)
        /// * `non_unique` - Optional non-unique parameter for node metadata
        /// * `max_burn_amount` - Maximum registration burn fee willing to pay (fee change protection)
        ///
        /// # Registration Requirements
        ///
        /// ## 1. Subnet State Validation
        ///
        /// - **Valid Subnet**: Subnet ID must exist in `SubnetsData`
        /// - **Not Paused**: Subnet must not be in `Paused` state
        /// - **Valid Registration Window**: Must NOT be in enactment period
        ///   - ✅ **Allowed**: Registration period (`SubnetState::Registered`, before enactment)
        ///   - ✅ **Allowed**: Active state (`SubnetState::Active`)
        ///   - ❌ **Blocked**: Enactment period (checked via `is_subnet_in_enactment`)
        ///   - **Rationale**: Enactment is grace period for delegate staking only, no new nodes
        ///
        /// ## 2. Coldkey and Hotkey Validation
        ///
        /// ### Distinct Keys
        /// - **Check**: `coldkey != hotkey`
        /// - **Error**: `ColdkeyMatchesHotkey`
        /// - **Rationale**: Security - keys must have separate purposes. Keep all subnets isolated
        ///
        /// ### Unique Hotkey (Network-Wide)
        /// - **Check**: Hotkey must not exist in `HotkeyOwner` (network-wide uniqueness)
        /// - **Error**: `HotkeyHasOwner`
        /// - **Rationale**: One hotkey can only operate one node across entire network
        ///
        /// ### Not Already Registered to Coldkey
        /// - **Check**: Hotkey not in `ColdkeyHotkeys[coldkey]`
        /// - **Error**: `HotkeyAlreadyRegisteredToColdkey`
        /// - **Note**: Redundant after network-wide check, but defensive
        ///
        /// ### No Existing Stake
        /// - **Check**: `AccountSubnetStake[hotkey, subnet_id] == 0`
        /// - **Error**: `MustUnstakeToRegister`
        /// - **Note**: Redundant after hotkey uniqueness check
        ///
        /// ## 3. Peer ID Validation
        ///
        /// ### All Peer IDs Must Be Unique
        /// - **Check**: `peer_id != bootnode_peer_id != client_peer_id`
        /// - **Error**: `PeerIdsMustBeUnique`
        /// - **Rationale**: Each peer ID serves different network functions
        ///
        /// ### Loosely validate Libp2p Peer ID Format
        /// - **Checks**: All three peer IDs validated via `validate_peer_id()`
        /// - **Errors**: `InvalidPeerId`, `InvalidBootnodePeerId`, `InvalidClientPeerId`
        /// - **Format**: Must be valid base58-encoded libp2p peer IDs
        ///
        /// ### Subnet-Wide Peer ID Uniqueness
        /// - **Checks**: Each peer ID must not exist in subnet via:
        ///   - `PeerIdSubnetNodeId[subnet_id, peer_id]` - Standard peer ID
        ///   - `BootnodePeerIdSubnetNodeId[subnet_id, bootnode_peer_id]` - Bootnode peer ID
        ///   - `ClientPeerIdSubnetNodeId[subnet_id, client_peer_id]` - Client peer ID
        /// - **Errors**: `PeerIdExist`, `BootnodePeerIdExist`, `ClientPeerIdExist`
        /// - **Rationale**: Prevents impersonation and network conflicts within subnet
        ///
        /// ### Bootnode Uniqueness (If Provided)
        /// - **Check**: If `bootnode` is `Some`, must not exist in `BootnodeSubnetNodeId[subnet_id]`
        /// - **Error**: `BootnodeExist`
        /// - **Rationale**: Each bootnode multiaddr must be unique within subnet
        ///
        /// ## 4. Registration Whitelist (During Registration Period Only)
        ///
        /// - **When**: Only checked if `SubnetRegistrationInitialColdkeys` exists (subnet in registration)
        /// - **Condition 1**: Coldkey must be in the whitelist
        ///   - **Error**: `ColdkeyRegistrationWhitelist` if not whitelisted
        /// - **Condition 2**: Coldkey must not have exhausted registration slots
        ///   - **Check**: `InitialColdkeyData[subnet_id][coldkey] < whitelist_max_registrations`
        ///   - **Error**: `MaxRegisteredNodes` if quota exceeded
        /// - **Post-Activation**: Whitelist removed, check skipped (anyone can register)
        ///
        /// ## 5. Subnet Capacity
        ///
        /// - **Check**: `SubnetNodeQueue.len() <= MaxRegisteredNodes`
        /// - **Error**: `MaxRegisteredNodes`
        /// - **Rationale**: Subnet has maximum node capacity to maintain performance
        ///
        /// ## 6. Unique Parameter Validation (If Provided)
        ///
        /// - **Check**: If `unique` is `Some`, must not exist in `UniqueParamSubnetNodeId[subnet_id]`
        /// - **Error**: `SubnetNodeUniqueParamTaken`
        /// - **Rationale**: Allows subnets to enforce custom uniqueness constraints (e.g., unique IP addresses)
        ///
        /// # Registration Cost (Burn Fee)
        ///
        /// ## Dynamic Pricing Mechanism
        ///
        /// - **Calculation**: `calculate_burn_amount(subnet_id)`
        ///   - Uses exponential decay based on registration rate
        ///   - Formula: `current_rate * alpha^(-registrations_this_epoch / target_rate)`
        ///   - `NodeBurnRateAlpha` - Decay parameter (0-100%, typically ~50%)
        ///   - `TargetNodeRegistrationsPerEpoch` - Target registration rate
        ///   - `NodeRegistrationsThisEpoch` - Counter for this epoch
        ///
        /// - **Slippage Protection**: `burn_amount <= max_burn_amount`
        ///   - **Error**: `MaxBurnAmountExceeded` if cost exceeds tolerance
        ///   - Protects against front-running and rapid cost changes
        ///
        /// - **Payment**: Tokens permanently burned (sent to zero address)
        ///   - **Error**: `BalanceBurnError` if coldkey lacks funds
        ///
        /// - **Recording**: Registration counted via `record_registration()` for next epoch's pricing
        ///
        /// # Stake Requirements
        ///
        /// - **Minimum Stake**: `stake_to_be_added >= SubnetMinStakeBalance[subnet_id]`
        /// - **Maximum Stake**: `stake_to_be_added <= SubnetMaxStakeBalance[subnet_id]`
        /// - **Process**: Calls `do_add_stake()` which:
        ///   - Transfers tokens from coldkey to node stake account
        ///   - Validates stake amount against subnet limits
        ///   - Updates `AccountSubnetStake`, `TotalSubnetStake`, and related counters
        /// - **Error**: Propagated from `do_add_stake` (e.g., insufficient balance, stake out of range)
        ///
        /// # Delegate Reward Rate
        ///
        /// - **Range**: 0-100% (`0` to `percentage_factor_as_u128()`)
        /// - **Purpose**: Percentage of node rewards shared with delegators
        /// - **Timestamp**: `last_delegate_reward_rate_update` set to current block if rate > 0
        ///   - Cooldown period may apply for rate changes (enforced in update function)
        ///
        /// # Registration Process
        ///
        /// ## Storage Updates (All Registrations)
        ///
        /// The following storage is initialized for every node:
        ///
        /// ### Node Identity
        /// - `TotalSubnetNodeUids[subnet_id]` - Incremented to generate unique node ID
        /// - `HotkeySubnetNodeId[subnet_id, hotkey]` - Hotkey → node ID mapping
        /// - `SubnetNodeIdHotkey[subnet_id, node_id]` - Reverse mapping (node ID → hotkey)
        /// - `HotkeySubnetId[hotkey]` - Hotkey → subnet ID (for cross-subnet operations)
        /// - `HotkeyOwner[hotkey]` - Hotkey → coldkey ownership
        /// - `ColdkeyHotkeys[coldkey]` - Set of all hotkeys owned by coldkey (updated)
        /// - `ColdkeySubnetNodes[coldkey][subnet_id]` - Set of node IDs owned by coldkey in subnet
        ///
        /// ### Peer ID Mappings
        /// - `PeerIdSubnetNodeId[subnet_id, peer_id]` - Peer ID → node ID
        /// - `BootnodePeerIdSubnetNodeId[subnet_id, bootnode_peer_id]` - Bootnode peer ID → node ID
        /// - `ClientPeerIdSubnetNodeId[subnet_id, client_peer_id]` - Client peer ID → node ID
        /// - `BootnodeSubnetNodeId[subnet_id, bootnode]` - Bootnode multiaddr → node ID (if provided)
        ///
        /// ### Custom Parameters
        /// - `UniqueParamSubnetNodeId[subnet_id, unique]` - Unique param → node ID (if provided)
        ///
        /// ### Counters
        /// - `TotalSubnetNodes[subnet_id]` - Incremented
        /// - `TotalNodes` - Global counter incremented
        ///
        /// ## Registration Period Behavior (Auto-Activation)
        ///
        /// **When**: `subnet.state == SubnetState::Registered`
        ///
        /// - **Node State**: Immediately activated (no queue)
        /// - **Process**: Calls `perform_activate_subnet_node()` which:
        ///   - Sets node classification to `Active`
        ///   - Inserts into `SubnetNodesData` (active nodes)
        ///   - Adds to `SubnetNodeElectionSlots` (validator election pool)
        ///   - Updates `TotalActiveSubnetNodes`, `TotalSubnetElectableNodes`
        ///   - Emits `SubnetNodeActivated` event
        /// - **Whitelist Counter**: `InitialColdkeyData[subnet_id][coldkey]` incremented
        /// - **Rationale**: Registration period = bootstrapping phase, nodes needed ASAP
        ///
        /// ## Active State Behavior (Queued Entry)
        ///
        /// **When**: `subnet.state == SubnetState::Active`
        ///
        /// - **Node State**: Enters queue as `Registered` (not active yet)
        /// - **Classification**: `SubnetNodeClass::Registered`
        /// - **Start Epoch**: `subnet_epoch + 1` (waits at least 1 epoch)
        /// - **Queue**: Added to `SubnetNodeQueue[subnet_id]`
        /// - **Storage**: Inserted into `RegisteredSubnetNodesData[subnet_id, node_id]`
        /// - **Emits**: `SubnetNodeRegistered` event (NOT activated yet)
        /// - **Activation**: Must wait for:
        ///   1. Queue immunity period (`QueueImmunityEpochs`)
        ///   2. Available election slots
        ///   3. Meets activation criteria (via churn logic in epoch processing)
        /// - **Rationale**: Active subnets enforce orderly entry to prevent disruption
        ///
        /// # Storage Cleanup
        ///
        /// - **`clean_coldkey_subnet_nodes(coldkey)`**: Removes stale node references for coldkey
        ///   - Cleans up `ColdkeySubnetNodes` for nodes that no longer exist
        ///   - Ensures data consistency
        ///
        /// # Events
        ///
        /// ## Registration Period
        /// - `SubnetNodeActivated { subnet_id, subnet_node_id, coldkey, hotkey, data }`
        ///   - Emitted via `perform_activate_subnet_node`
        ///
        /// ## Active State
        /// - `SubnetNodeRegistered { subnet_id, subnet_node_id, coldkey, hotkey, data }`
        ///   - Node queued, not yet active
        ///
        /// # Errors
        ///
        /// ## Subnet Validation
        /// - `InvalidSubnetId` - Subnet does not exist
        /// - `SubnetIsPaused` - Subnet is paused (cannot accept new nodes)
        /// - `SubnetMustBeRegisteringOrActivated` - In enactment period (registration blocked)
        ///
        /// ## Key Validation
        /// - `ColdkeyMatchesHotkey` - Coldkey and hotkey must be different
        /// - `HotkeyHasOwner` - Hotkey already in use (network-wide)
        /// - `HotkeyAlreadyRegisteredToColdkey` - Hotkey already owned by this coldkey
        ///
        /// ## Peer ID Validation
        /// - `PeerIdsMustBeUnique` - Peer IDs must be distinct from each other
        /// - `InvalidPeerId` / `InvalidBootnodePeerId` / `InvalidClientPeerId` - Invalid libp2p format
        /// - `PeerIdExist` / `BootnodePeerIdExist` / `ClientPeerIdExist` - Peer ID already used in subnet
        /// - `BootnodeExist` - Bootnode multiaddr already used in subnet
        ///
        /// ## Capacity and Whitelist
        /// - `ColdkeyRegistrationWhitelist` - Coldkey not whitelisted (registration period only)
        /// - `MaxRegisteredNodes` - Subnet at capacity OR coldkey exhausted whitelist quota
        ///
        /// ## Unique Parameters
        /// - `SubnetNodeUniqueParamTaken` - Unique parameter already in use
        ///
        /// ## Financial
        /// - `MaxBurnAmountExceeded` - Burn cost exceeds slippage tolerance
        /// - `BalanceBurnError` - Insufficient balance to pay burn fee
        /// - `MustUnstakeToRegister` - Hotkey has existing stake (should be impossible)
        ///
        /// ## Staking (from do_add_stake)
        /// - `StakeAmountBelowMinimum` - Stake below `SubnetMinStakeBalance`
        /// - `StakeAmountExceedsMaximum` - Stake exceeds `SubnetMaxStakeBalance`
        /// - `InsufficientBalance` - Coldkey lacks funds for stake
        ///
        /// # Important Notes
        ///
        /// ## Registration vs Activation
        /// - **Registration Period**: Nodes immediately active (bootstrap phase)
        /// - **Active State**: Nodes enter queue, activated later (orderly entry)
        ///
        /// ## Queue Activation (Active Subnets)
        /// Queued nodes are activated during epoch processing based on:
        /// - **Churn Limit**: Maximum nodes entering per epoch (`ChurnLimit`)
        /// - **Queue Immunity**: Protection period (`QueueImmunityEpochs`)
        /// - **Election Slots**: Available validator slots
        /// - **Queue Order**: FIFO with immunity considerations
        ///
        /// ## Burn Fee Dynamics
        /// - Cost increases as more nodes register in current epoch
        /// - Cost decays exponentially each epoch based on actual vs target registrations
        /// - Prevents spam while allowing legitimate growth
        ///
        /// ## Peer ID Security
        /// - Subnet-wide uniqueness prevents impersonation
        /// - Different peer IDs for different network functions (standard, bootnode, client)
        /// - Signature verification should be used for additional security
        ///
        /// ## Coldkey Node Limits
        /// - During registration: Limited by whitelist quota
        /// - Post-activation: No explicit limit, but constrained by:
        ///   - Subnet capacity (`MaxRegisteredNodes`)
        ///   - Economic feasibility (stake requirements × number of nodes)
        ///   - Queue wait times
        ///
        pub fn do_register_subnet_node(
            origin: OriginFor<T>,
            subnet_id: u32,
            hotkey: T::AccountId,
            peer_id: PeerId,
            bootnode_peer_id: PeerId,
            client_peer_id: PeerId,
            bootnode: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
            delegate_reward_rate: u128,
            stake_to_be_added: u128,
            unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
            non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
            max_burn_amount: u128,
        ) -> DispatchResult {
            let coldkey: T::AccountId = ensure_signed(origin.clone())?;

            let subnet = match SubnetsData::<T>::try_get(subnet_id) {
                Ok(subnet) => subnet,
                Err(()) => return Err(Error::<T>::InvalidSubnetId.into()),
            };

            ensure!(&coldkey != &hotkey, Error::<T>::ColdkeyMatchesHotkey);

            // Ensure subnet is not paused
            ensure!(
                subnet.state != SubnetState::Paused,
                Error::<T>::SubnetIsPaused
            );

            // Unique network-wide hotkey
            ensure!(
                !Self::hotkey_has_owner(hotkey.clone()),
                Error::<T>::HotkeyHasOwner
            );

            // Redundant, this is impossible after hotkey check
            ensure!(
                !HotkeySubnetId::<T>::contains_key(hotkey.clone()),
                Error::<T>::HotkeyHasOwner
            );

            // --- Ensure all peer IDs are unique
            ensure!(
                Self::are_all_unique(&vec![
                    peer_id.clone(),
                    bootnode_peer_id.clone(),
                    client_peer_id.clone()
                ]),
                Error::<T>::PeerIdsMustBeUnique
            );

            // - Get standard epoch to check if subnet can accept registrations
            let epoch: u32 = Self::get_current_epoch_as_u32();

            // If in enactment period, registering is disabled
            // Nodes must enter in the registration period or activation period
            // Once we are in the enactment period, only delegate staking is enabled to reach the qualifications
            ensure!(
                !Self::is_subnet_in_enactment(subnet_id, subnet.state, epoch),
                Error::<T>::SubnetMustBeRegisteringOrActivated
            );

            // - Get subnet epoch to check against node start_epochs
            let subnet_epoch: u32 = Self::get_current_subnet_epoch_as_u32(subnet_id);

            // --- If in registration period, check if there is a whitelist and coldkey is in the whitelist
            //     and if the coldkey hasn't registered too many nodes
            // There must be SubnetRegistrationInitialColdkeys if not active
            // `SubnetRegistrationInitialColdkeys` is removed on activation
            // Note: `SubnetRegistrationInitialColdkeys` is removed on activation
            if let Some(coldkey_map) = SubnetRegistrationInitialColdkeys::<T>::get(subnet_id) {
                if let Some(&max_registrations) = coldkey_map.get(&coldkey) {
                    let current_registrations = InitialColdkeyData::<T>::get(subnet_id)
                        .and_then(|map| map.get(&coldkey).copied())
                        .unwrap_or(0);

                    ensure!(
                        current_registrations < max_registrations,
                        Error::<T>::MaxRegisteredNodes
                    );
                } else {
                    // Coldkey doesn't exist in the mapping
                    return Err(Error::<T>::ColdkeyRegistrationWhitelist.into());
                }
            }

            // Ensure there are registered node slots available
            ensure!(
                SubnetNodeQueue::<T>::get(subnet_id).len() as u32
                    <= MaxRegisteredNodes::<T>::get(subnet_id),
                Error::<T>::MaxRegisteredNodes
            );

            // Validate peer IDs
            ensure!(Self::validate_peer_id(&peer_id), Error::<T>::InvalidPeerId);
            ensure!(
                Self::validate_peer_id(&client_peer_id),
                Error::<T>::InvalidClientPeerId
            );
            ensure!(
                Self::validate_peer_id(&bootnode_peer_id),
                Error::<T>::InvalidBootnodePeerId
            );

            // Ensure peer and boostrap peer ID doesn't already exist within subnet regardless of coldkey

            // Unique subnet_id -> PeerId
            ensure!(
                Self::is_owner_of_peer_or_ownerless(subnet_id, 0, 0, &peer_id),
                Error::<T>::PeerIdExist
            );

            // Unique subnet_id -> Bootnode PeerId
            ensure!(
                Self::is_owner_of_peer_or_ownerless(subnet_id, 0, 0, &bootnode_peer_id),
                Error::<T>::BootnodePeerIdExist
            );

            // Unique subnet_id -> Client PeerId
            ensure!(
                Self::is_owner_of_peer_or_ownerless(subnet_id, 0, 0, &client_peer_id),
                Error::<T>::ClientPeerIdExist
            );

            // Ensure bootnode is unique
            if let Some(bootnode) = &bootnode {
                ensure!(
                    Self::is_owner_of_bootnode_or_ownerless(subnet_id, 0, bootnode.clone()),
                    Error::<T>::BootnodeExist
                );
            }

            // --- Ensure they have no stake on registration
            // This is redundant since hotkeys can't be used twice
            ensure!(
                AccountSubnetStake::<T>::get(&hotkey, subnet_id) == 0,
                Error::<T>::MustUnstakeToRegister
            );

            // Redundant after hotkey_has_owner
            let mut hotkeys = ColdkeyHotkeys::<T>::get(&coldkey);
            ensure!(
                !hotkeys.contains(&hotkey),
                Error::<T>::HotkeyAlreadyRegisteredToColdkey
            );

            //
            // Burn fee
            //
            let burn_amount = Self::calculate_burn_amount(subnet_id);
            ensure!(
                burn_amount <= max_burn_amount,
                Error::<T>::MaxBurnAmountExceeded
            );

            let burn_amount_as_balance = Self::u128_to_balance(burn_amount);
            if let Some(burn_amount_as_balance) = burn_amount_as_balance {
                ensure!(
                    Self::burn(coldkey.clone(), burn_amount_as_balance),
                    Error::<T>::BalanceBurnError
                );
            }

            // --- Record node registration on this epoch for burn fee calculations
            Self::record_registration(subnet_id);

            // ====================
            // Initiate stake logic
            // ====================
            Self::do_add_stake(origin.clone(), subnet_id, hotkey.clone(), stake_to_be_added)
                .map_err(|e| e)?;

            let block: u32 = Self::get_current_block_as_u32();

            // --- Only use block for last_delegate_reward_rate_update is rate is greater than zero
            let mut last_delegate_reward_rate_update = 0;
            if delegate_reward_rate > 0 {
                last_delegate_reward_rate_update = block;
            }

            // --- Start the UIDs at 1
            TotalSubnetNodeUids::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
            let subnet_node_id = TotalSubnetNodeUids::<T>::get(subnet_id);

            // Unique ``unique``
            // [here]
            if let Some(unique_param) = unique.clone() {
                ensure!(
                    !UniqueParamSubnetNodeId::<T>::contains_key(subnet_id, &unique_param),
                    Error::<T>::SubnetNodeUniqueParamTaken
                );
                UniqueParamSubnetNodeId::<T>::insert(subnet_id, &unique_param, subnet_node_id);
            }
            HotkeySubnetNodeId::<T>::insert(subnet_id, &hotkey, subnet_node_id);

            // Insert Subnet Node ID -> hotkey
            SubnetNodeIdHotkey::<T>::insert(subnet_id, subnet_node_id, &hotkey);

            // Insert unique hotkey to subnet ID mapping
            // This is used for updating hotkeys that have subnet_id keys
            HotkeySubnetId::<T>::insert(&hotkey, subnet_id);

            // Insert hotkey -> coldkey
            HotkeyOwner::<T>::insert(&hotkey, &coldkey);

            // Insert coldkey -> hotkeys
            hotkeys.insert(hotkey.clone());
            ColdkeyHotkeys::<T>::insert(&coldkey, hotkeys);

            // Insert ColdkeySubnetNodes
            // Used in overwatch node subnet diversity
            ColdkeySubnetNodes::<T>::mutate(&coldkey, |node_map| {
                node_map
                    .entry(subnet_id)
                    .or_insert_with(BTreeSet::new)
                    .insert(subnet_node_id);
            });

            // To ensure the AccountId that owns the PeerId, the subnet should use signature authentication
            // This ensures others cannot claim to own a PeerId they are not the owner of

            // Insert subnet peer and bootnode peer to keep peer_ids unique within subnets
            PeerIdSubnetNodeId::<T>::insert(subnet_id, &peer_id, subnet_node_id);
            BootnodePeerIdSubnetNodeId::<T>::insert(subnet_id, &bootnode_peer_id, subnet_node_id);
            ClientPeerIdSubnetNodeId::<T>::insert(subnet_id, &client_peer_id, subnet_node_id);
            if let Some(bootnode) = &bootnode {
                BootnodeSubnetNodeId::<T>::insert(subnet_id, &bootnode, subnet_node_id);
            }

            // Add to registration queue
            // ========================
            // Insert peer into storage
            // ========================
            let classification: SubnetNodeClassification = SubnetNodeClassification {
                node_class: SubnetNodeClass::Registered,
                start_epoch: subnet_epoch + 1,
            };

            let subnet_node: SubnetNode<T::AccountId> = SubnetNode {
                id: subnet_node_id,
                hotkey: hotkey.clone(),
                peer_id: peer_id.clone(),
                bootnode_peer_id: bootnode_peer_id.clone(),
                client_peer_id: client_peer_id.clone(),
                bootnode: bootnode,
                classification: classification,
                delegate_reward_rate: delegate_reward_rate,
                last_delegate_reward_rate_update: last_delegate_reward_rate_update,
                unique: unique,
                non_unique: non_unique,
            };

            // Increase total subnet nodes
            TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
            TotalNodes::<T>::mutate(|n: &mut u32| *n += 1);

            Self::clean_coldkey_subnet_nodes(coldkey.clone());

            // Push into queue
            if subnet.state == SubnetState::Registered {
                // Activate subnet node automatically
                Self::perform_activate_subnet_node(
                    subnet_id,
                    subnet.state,
                    subnet_node,
                    subnet_epoch,
                )
                .map_err(|e| e)?;

                InitialColdkeyData::<T>::mutate(subnet_id, |maybe_map| {
                    let map = maybe_map.get_or_insert_with(BTreeMap::new);
                    map.entry(coldkey.clone())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                });
            } else {
                // Insert RegisteredSubnetNodesData
                RegisteredSubnetNodesData::<T>::insert(subnet_id, subnet_node_id, &subnet_node);

                SubnetNodeQueue::<T>::mutate(subnet_id, |nodes| {
                    nodes.push(subnet_node.clone());
                });

                Self::deposit_event(Event::SubnetNodeRegistered {
                    subnet_id: subnet_id,
                    subnet_node_id: subnet_node_id,
                    coldkey: coldkey,
                    hotkey: hotkey,
                    data: subnet_node,
                });
            }

            Ok(())
        }

        /// Activate subnet node if subnet is in registration
        /// This should only be called if the subnet is in registration
        /// when a node is registering to the subnet
        pub fn perform_activate_subnet_node(
            subnet_id: u32,
            subnet_state: SubnetState,
            mut subnet_node: SubnetNode<T::AccountId>,
            subnet_epoch: u32,
        ) -> DispatchResult {
            // We're about to call `insert_node_into_election_slot`. We must always check
            // max subnet nodes is not breached when calling this.
            ensure!(
                TotalActiveSubnetNodes::<T>::get(subnet_id) < MaxSubnetNodes::<T>::get(),
                Error::<T>::MaxRegisteredNodes
            );

            // This can only return false if electrion slot insertion fails
            ensure!(
                Self::do_activate_subnet_node(
                    &mut WeightMeter::new(),
                    subnet_id,
                    subnet_state,
                    subnet_node,
                    subnet_epoch,
                    false
                ),
                Error::<T>::ElectionSlotInsertFail
            );

            Ok(())
        }

        /// Activates a subnet node, transitioning it from a registered state to an active state.
        ///
        /// This function handles the logic for moving a node from the `RegisteredSubnetNodesData`
        /// storage to the `SubnetNodesData` storage, effectively making it an active participant
        /// in the subnet. It updates various counters and reputation metrics.
        ///
        /// # Logic
        ///
        /// 1. **Validation**: Checks if the subnet state and queue flag combination is valid.
        /// 2. **Weight Check**: Verifies if there is enough weight remaining to perform the operation.
        /// 3. **Queue Handling**: If called from the queue, ensures the node is in the `Registered` class.
        /// 4. **State Transition**:
        ///    - Moves the node data from `RegisteredSubnetNodesData` to `SubnetNodesData`.
        ///    - Sets the node class to `Idle` initially.
        ///    - If the subnet is in the `Registered` state and not queued, it attempts to promote
        ///      the node to `Validator` class and insert it into an election slot.
        /// 5. **Counters**: Updates `TotalActiveSubnetNodes`, `TotalActiveNodes`, and `ColdkeyReputation`.
        /// 6. **Event**: Deposits a `SubnetNodeActivated` event.
        ///
        /// # Parameters
        ///
        /// * `weight_meter` - WeightMeter reference to track execution weight.
        /// * `subnet_id` - ID of the subnet the node is activating on.
        /// * `subnet_state` - Current state of the subnet.
        /// * `subnet_node` - The node data to activate.
        /// * `subnet_epoch` - The current subnet epoch.
        /// * `queue` - Whether this activation is being processed from the queue.
        ///             If `true`, the node is coming from the queue.
        ///             If `false`, it's a direct activation (e.g. during subnet registration phase).
        ///
        /// # Returns
        ///
        /// * `bool` - `true` if activation was successful, `false` otherwise.
        pub fn do_activate_subnet_node(
            weight_meter: &mut WeightMeter,
            subnet_id: u32,
            subnet_state: SubnetState,
            mut subnet_node: SubnetNode<T::AccountId>,
            subnet_epoch: u32,
            queue: bool,
        ) -> bool {
            let mut weight = Weight::zero();
            let db_weight = T::DbWeight::get();

            // These combination should never be called
            if subnet_state == SubnetState::Registered && queue
                || subnet_state == SubnetState::Active && !queue
                || subnet_state == SubnetState::Paused
            {
                return false;
            }

            // writes:
            // RegisteredSubnetNodesData
            // SubnetNodesData
            // TotalActiveSubnetNodes
            // TotalActiveNodes
            // ColdkeyReputation
            //
            // reads:
            // HotkeyOwner
            if !weight_meter.can_consume(db_weight.reads_writes(5, 1)) {
                return false;
            }

            if queue && subnet_node.classification.node_class != SubnetNodeClass::Registered {
                // Because each queued node is always `SubnetNodeClass::Registered`
                // return true and pop them out of the queue
                return true;
            }

            // Try to take the RegisteredSubnetNodesData
            weight = weight.saturating_add(db_weight.reads_writes(1, 1));
            RegisteredSubnetNodesData::<T>::take(subnet_id, subnet_node.id);

            // Default use if subnet is active and not currently registering
            // --- If subnet activated, activate the node starting at `Idle`
            subnet_node.classification.node_class = SubnetNodeClass::Idle;
            // --- Increase subnet_epoch by one to ensure node starts on a fresh subnet_epoch unless subnet is still registering
            subnet_node.classification.start_epoch = subnet_epoch + 1;

            if subnet_state == SubnetState::Registered && !queue {
                subnet_node.classification.node_class = SubnetNodeClass::Validator;
                // --- Start node on current subnet_epoch for the next era
                subnet_node.classification.start_epoch = subnet_epoch;

                // --- Insert into election slot if entering as Validator class
                // The only other way to enter the election slots is by being graduated by consensus
                // This should not be possible due to registration checks max subnet nodes
                if !Self::insert_node_into_election_slot(subnet_id, subnet_node.id) {
                    return false;
                }
            }

            // --- Insert new active node
            SubnetNodesData::<T>::insert(subnet_id, subnet_node.id, &subnet_node);
            // Increase total active nodes in subnet
            TotalActiveSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

            // Increase total active nodes
            TotalActiveNodes::<T>::mutate(|n: &mut u32| *n += 1);

            let coldkey = HotkeyOwner::<T>::get(&subnet_node.hotkey);
            weight = weight.saturating_add(db_weight.reads(1));

            ColdkeyReputation::<T>::mutate(&coldkey, |rep| {
                rep.lifetime_node_count = rep.lifetime_node_count.saturating_add(1);
                rep.total_active_nodes = rep.total_active_nodes.saturating_add(1);
            });

            // (r/w)
            // TotalActiveSubnetNodes
            // TotalActiveNodes
            // ColdkeyReputation
            // (r)
            // HotkeyOwner
            // (w)
            // SubnetNodesData
            weight_meter.consume(db_weight.reads_writes(4, 4));

            Self::deposit_event(Event::SubnetNodeActivated {
                subnet_id: subnet_id,
                subnet_node_id: subnet_node.id,
            });

            true
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Run block functions
        ///
        /// # Flow
        ///
        /// At the start of each epoch
        ///
        /// 1. Epoch prelims (removing or penalizing subnets) (block)
        /// 2. Calculate overwatch subnet weights (block - 1) (called on overwatch epochs only)
        /// 3. Calculate subnet emissions distribution (block - 2)
        /// 4. Handle subnet slots (slot)
        ///		* Distribute rewards
        /// 	* Elect validator
        ///
        /// * Execute swap queue calls on all blocks with block weight remaining
        ///
        /// # Arguments
        ///
        /// * `block_number` - Current block number.
        ///
        fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
            let db_weight = T::DbWeight::get();

            let mut weight_meter = WeightMeter::with_limit(MaximumHooksWeightV2::<T>::get());

            // MaximumHooksWeightV2
            weight_meter.consume(db_weight.reads(1));

            if Self::is_paused().is_err() {
                return weight_meter.consumed();
            }

            // General epochs
            let block: u32 = Self::convert_block_as_u32(block_number);
            let epoch_length: u32 = T::EpochLength::get();
            let epoch_slot = block % epoch_length;
            let current_epoch = block.saturating_div(epoch_length);

            // Overwatch epochs
            let multiplier: u32 = OverwatchEpochLengthMultiplier::<T>::get();

            // OverwatchEpochLengthMultiplier
            weight_meter.consume(db_weight.reads(1));

            let overwatch_epoch_length = epoch_length.saturating_mul(multiplier);

            if block >= epoch_length && block % epoch_length == 0 {
                // Remove unqualified subnets
                // Note: This updates `weight_meter`
                //
                // The weight_meter is sent in to ensure we can remove a subnet without consuming too much
                // block weight. The maximum number of subnets being removed does not currently surpass the
                // maximum block weight, although, this is meant for future-proofing and optimizing
                Self::do_epoch_preliminaries(&mut weight_meter, block, current_epoch);
            } else if (block - 1) >= overwatch_epoch_length
                && (block - 1) % overwatch_epoch_length == 0
            {
                // Calculate Overwatch Node Weights
                let block_step_weight = Self::calculate_overwatch_rewards();
                // `consume(..)` saturates at zero
                weight_meter.consume(block_step_weight);
            } else if (block - 2) >= epoch_length && (block - 2) % epoch_length == 0 {
                // Calculate rewards
                // Distribute to foundation/treasury
                // Calculate emissions based on subnet weights (delegate stake/node count based)
                let block_step_weight = Self::handle_subnet_emission_weights(current_epoch);
                // `consume(..)` saturates at zero
                weight_meter.consume(block_step_weight);
            } else if let Some(subnet_id) = SlotAssignment::<T>::get(epoch_slot) {
                // SlotAssignment
                weight_meter.consume(db_weight.reads(1));

                let subnet_epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);

                // Uses `WeightMeter` so we don't consume after
                Self::emission_step(
                    &mut weight_meter,
                    block,
                    current_epoch,
                    subnet_epoch,
                    subnet_id,
                );
            } else {
                // If we make it here, SlotAssignment was read
                // SlotAssignment
                weight_meter.consume(db_weight.reads(1));
            }

            // Attempt stake swap queue on every block
            Self::execute_ready_swap_calls(block, &mut weight_meter);

            // for EVM tests (Weights in on_initialize change the block weight/gas)
            Weight::from_parts(0, 0)

            // weight_meter.consumed()
        }

        fn on_finalize(block_number: BlockNumberFor<T>) {}

        fn on_idle(block_number: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            return Weight::from_parts(0, 0);
        }
    }

    impl<T: Config> Pallet<T> {
        // Execute multiple calls at once (for block hooks)
        pub fn execute_ready_swap_calls(block_number: u32, weight_meter: &mut WeightMeter) {
            let db_weight = T::DbWeight::get();

            let max_executions = MaxSwapQueueCallsPerBlock::<T>::get();
            weight_meter.consume(db_weight.reads(1));

            let mut executed = 0;

            while executed < max_executions {
                // Loop iteration overhead
                weight_meter.consume(Weight::from_parts(1_000, 0));

                // SwapQueueOrder
                weight_meter.consume(db_weight.reads_writes(1, 1));
                let should_continue = SwapQueueOrder::<T>::mutate(|queue| -> bool {
                    // Check either stake functions can be called before we get there.
                    // `weight_meter.can_consume` is called in `execute_swap_call_internal` again.
                    // This is redundant but will save at least one DB read (which is what is expensive)
                    if queue.is_empty() {
                        return false;
                    }

                    let first_id = queue[0];

                    // SwapCallQueue
                    weight_meter.consume(db_weight.reads(1));

                    if let Some(item) = SwapCallQueue::<T>::get(&first_id) {
                        let blocks_passed = block_number.saturating_sub(item.queued_at_block);
                        if blocks_passed >= item.execute_after_blocks.into() {
                            // If the function can't be called, it will return before calling and the loop will break
                            let is_ok = Self::execute_swap_call_internal(&item.call, weight_meter);
                            // If not `is_ok`, the function was not called
                            if !is_ok {
                                // break if no weight left in WeightMeter
                                return false;
                            }

                            // The swap can only fail if the balance -> shares conversion fails
                            // Therefore, we always remove from the queue.
                            // If the conversion fails, the stake value is worthless or near worthless
                            queue.remove(0);
                            SwapCallQueue::<T>::remove(&first_id);
                            // SwapCallQueue
                            weight_meter.consume(db_weight.writes(1));
                            return true;
                        }
                    }

                    // If no elements in `SwapCallQueue`
                    false
                });

                if !should_continue {
                    break;
                }

                executed += 1;
            }
        }

        pub fn execute_swap_call_internal(
            queued_call: &QueuedSwapCall<T::AccountId>,
            weight_meter: &mut WeightMeter,
        ) -> bool {
            match queued_call {
                QueuedSwapCall::SwapToSubnetDelegateStake {
                    account_id,
                    to_subnet_id,
                    balance,
                } => {
                    if !weight_meter
                        .can_consume(T::WeightInfo::handle_increase_account_delegate_stake())
                    {
                        return false;
                    }
                    weight_meter.consume(T::WeightInfo::handle_increase_account_delegate_stake());
                    let (_, _, _) = Self::handle_increase_account_delegate_stake(
                        account_id,
                        *to_subnet_id,
                        *balance,
                    );
                }
                QueuedSwapCall::SwapToNodeDelegateStake {
                    account_id,
                    to_subnet_id,
                    to_subnet_node_id,
                    balance,
                } => {
                    if !weight_meter.can_consume(
                        T::WeightInfo::handle_increase_account_node_delegate_stake_shares(),
                    ) {
                        return false;
                    }
                    weight_meter.consume(
                        T::WeightInfo::handle_increase_account_node_delegate_stake_shares(),
                    );
                    let (_, _, _) = Self::handle_increase_account_node_delegate_stake_shares(
                        account_id,
                        *to_subnet_id,
                        *to_subnet_node_id,
                        *balance,
                    );
                }
            }

            true
        }
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub subnet_name: Vec<u8>,
        pub subnet_nodes: Vec<(T::AccountId, PeerId)>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // [TESTING: LOCAL]
            MinSubnetRegistrationEpochs::<T>::set(0);
            OverwatchEpochLengthMultiplier::<T>::set(2);
            OverwatchMinDiversificationRatio::<T>::set(0);
            OverwatchMinRepScore::<T>::set(0);
            OverwatchMinAvgAttestationRatio::<T>::set(0);
            OverwatchMinAge::<T>::set(0);

            // // [TESTING: BENCHMARKING && EVM TESTS]
            // // Enable subnets to register right when conditions are met
            // MinSubnetRegistrationEpochs::<T>::set(0);
            // // Enable testing overwatch nodes on each epoch
            // OverwatchEpochLengthMultiplier::<T>::set(1);
            // OverwatchMinDiversificationRatio::<T>::set(0);
            // OverwatchMinRepScore::<T>::set(0);
            // OverwatchMinAvgAttestationRatio::<T>::set(0);
            // OverwatchMinAge::<T>::set(0);
            // DelegateStakeCooldownEpochs::<T>::set(0);
            // NodeDelegateStakeCooldownEpochs::<T>::put(0);
            // StakeCooldownEpochs::<T>::put(0);
            // MinActiveNodeStakeEpochs::<T>::put(0);
            // SubnetDelegateStakeRewardsUpdatePeriod::<T>::put(0);
            // NodeRewardRateUpdatePeriod::<T>::put(0);
            // MinSubnetDelegateStakeFactor::<T>::put(0);
            // MaxMinDelegateStakeMultiplier::<T>::put(1000000000000000000); // 100%
            // SubnetPauseCooldownEpochs::<T>::put(0);

            // [TESTING: EVM TESTS]
            // Enable subnets to register right when conditions are met
            // MinSubnetRegistrationEpochs::<T>::set(0);
            // OverwatchEpochLengthMultiplier::<T>::set(1);
            // OverwatchMinDiversificationRatio::<T>::set(0);
            // OverwatchMinRepScore::<T>::set(0);
            // OverwatchMinAvgAttestationRatio::<T>::set(0);
            // OverwatchMinAge::<T>::set(0);
            // DelegateStakeCooldownEpochs::<T>::set(0);
            // NodeDelegateStakeCooldownEpochs::<T>::put(0);
            // StakeCooldownEpochs::<T>::put(0);
            // MinActiveNodeStakeEpochs::<T>::put(0);
            // SubnetDelegateStakeRewardsUpdatePeriod::<T>::put(0);
            // NodeRewardRateUpdatePeriod::<T>::put(0);
            // MinSubnetDelegateStakeFactor::<T>::put(0);
            // MaxMinDelegateStakeMultiplier::<T>::put(1000000000000000000); // 100%
            // SubnetPauseCooldownEpochs::<T>::put(0);

            // use fp_account::AccountId20;
            // use sp_core::H160;
            // use sp_core::U256;

            // if self.subnet_name.last().is_none() {
            //     return;
            // }

            // let subnet_id = 1;
            // let friendly_uid = 1;

            // SubnetIdFriendlyUid::<T>::insert(subnet_id, friendly_uid);
            // FriendlyUidSubnetId::<T>::insert(friendly_uid, subnet_id);

            // let subnet_data = SubnetData {
            //     id: subnet_id,
            //     friendly_id: subnet_id,
            //     name: self.subnet_name.clone(),
            //     repo: Vec::new(),
            //     description: Vec::new(),
            //     misc: Vec::new(),
            //     state: SubnetState::Active,
            //     start_epoch: 0,
            // };

            // SubnetRegistrationEpoch::<T>::insert(subnet_id, 1);
            // // Store unique name
            // SubnetName::<T>::insert(self.subnet_name.clone(), subnet_id);
            // // Store repo
            // SubnetRepo::<T>::insert(self.subnet_name.clone(), subnet_id);
            // // Store subnet data
            // SubnetsData::<T>::insert(subnet_id, subnet_data.clone());
            // // Increase total subnets count
            // TotalSubnetUids::<T>::mutate(|n: &mut u32| *n += 1);

            // // Add bootnodes
            // let raw_bootnode = b"p2p/127.0.0.1/33130".to_vec();
            // // Try converting to a bounded vec (panics if too long)
            // let bounded: BoundedVec<u8, DefaultMaxVectorLength> = raw_bootnode
            //     .try_into()
            //     .expect("bootnode string fits in bounded vec");
            // // Put it into a BTreeSet
            // let bootnodes = BTreeSet::from([bounded]);
            // SubnetBootnodes::<T>::insert(subnet_id, bootnodes);

            // // Increase delegate stake to allow activation of subnet model
            // let min_stake_balance = MinSubnetMinStake::<T>::get();
            // // --- Get minimum subnet stake balance
            // let min_subnet_stake_balance = min_stake_balance;

            // let total_issuance_as_balance = T::Currency::total_issuance();

            // let alith = &self.subnet_nodes.iter().next();

            // let alith_balance = T::Currency::free_balance(&alith.unwrap().0);

            // let total_issuance: u128 = total_issuance_as_balance.try_into().unwrap_or(0);

            // let total_staked: u128 = TotalStake::<T>::get();

            // let total_delegate_staked: u128 = TotalDelegateStake::<T>::get();

            // let total_node_delegate_staked: u128 = TotalNodeDelegateStake::<T>::get();

            // let total_network_issuance = total_issuance
            //     .saturating_add(total_staked)
            //     .saturating_add(total_delegate_staked)
            //     .saturating_add(total_node_delegate_staked);

            // let factor: u128 = MinSubnetDelegateStakeFactor::<T>::get();

            // let x = U256::from(total_network_issuance);
            // let y = U256::from(factor);

            // // x * y / 100.0

            // let result = x * y / U256([0xde0b6b3a7640000, 0x0, 0x0, 0x0]);

            // let min_subnet_delegate_stake_balance: u128 = result.try_into().unwrap_or(u128::MAX);

            // // --- Mitigate inflation attack
            // TotalSubnetDelegateStakeShares::<T>::mutate(subnet_id, |mut n| {
            //     n.saturating_accrue(1000)
            // });

            // // =================
            // // convert_to_shares
            // // =================
            // let total_subnet_delegated_stake_balance =
            //     TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

            // let balance = U256::from(min_subnet_delegate_stake_balance);
            // let total_shares = U256::from(0) + U256::from(10_u128.pow(1));
            // let total_balance = U256::from(total_subnet_delegated_stake_balance) + U256::from(1);

            // let shares = balance * total_shares / total_balance;
            // let shares: u128 = shares.try_into().unwrap_or(u128::MAX);

            // // =====================================
            // // increase_account_delegate_stake
            // // =====================================
            // // -- increase total subnet delegate stake balance
            // TotalSubnetDelegateStakeBalance::<T>::mutate(subnet_id, |mut n| {
            //     n.saturating_accrue(min_subnet_delegate_stake_balance)
            // });
            // // -- increase total subnet delegate stake shares
            // TotalSubnetDelegateStakeShares::<T>::mutate(subnet_id, |mut n| {
            //     n.saturating_accrue(shares)
            // });
            // TotalDelegateStake::<T>::set(min_subnet_delegate_stake_balance);

            // // Store subnet data
            // SubnetsData::<T>::insert(subnet_id, &subnet_data);

            // // Store owner
            // SubnetOwner::<T>::insert(subnet_id, &alith.unwrap().0);

            // // Store the stake balance range
            // SubnetMinStakeBalance::<T>::insert(subnet_id, 0);
            // SubnetMaxStakeBalance::<T>::insert(subnet_id, 0);

            // // Add delegate state ratio
            // SubnetDelegateStakeRewardsPercentage::<T>::insert(subnet_id, 100000000000000000);
            // LastSubnetDelegateStakeRewardsUpdate::<T>::insert(subnet_id, 0);

            // // Add classification epochs
            // SubnetNodeQueueEpochs::<T>::insert(subnet_id, 0);
            // IdleClassificationEpochs::<T>::insert(subnet_id, 0);
            // IncludedClassificationEpochs::<T>::insert(subnet_id, 0);

            // // Add queue variables
            // ChurnLimit::<T>::insert(subnet_id, 0);

            // // Store min nodes reputation
            // MinSubnetNodeReputation::<T>::insert(subnet_id, 100000000000000000);

            // // Store whitelisted coldkeys for registration period
            // // SubnetRegistrationInitialColdkeys::<T>::insert(
            // // 	subnet_id,
            // // 	BTreeSet::new()
            // // );

            // MaxRegisteredNodes::<T>::insert(subnet_id, 256);
            // let keytypes = BTreeSet::from([KeyType::Rsa]);
            // SubnetKeyTypes::<T>::insert(subnet_id, keytypes);
            // LastSubnetRegistrationBlock::<T>::set(0);
            // SubnetRegistrationEpoch::<T>::insert(subnet_id, 0);

            // //
            // //
            // //
            // // --- Initialize subnet nodes
            // // Only initialize to test using subnet nodes
            // // If testing using subnet nodes in a subnet, comment out the ``for`` loop
            // //
            // //
            // //

            // let mut stake_amount: u128 = MinSubnetMinStake::<T>::get();

            // let mut count = 0;
            // for (account_id, peer_id) in &self.subnet_nodes {
            //     // Redundant
            //     // Unique subnet_id -> PeerId
            //     // Ensure peer ID doesn't already exist within subnet regardless of account_id
            //     let peer_exists: bool =
            //         match PeerIdSubnetNodeId::<T>::try_get(subnet_id, peer_id.clone()) {
            //             Ok(_) => true,
            //             Err(()) => false,
            //         };

            //     if peer_exists {
            //         continue;
            //     }

            //     // ====================
            //     // Initiate stake logic
            //     // ====================
            //     // T::Currency::withdraw(
            //     // 	&account_id,
            //     // 	stake_amount,
            //     // 	WithdrawReasons::except(WithdrawReasons::TIP),
            //     // 	ExistenceRequirement::KeepAlive,
            //     // );

            //     // -- increase account subnet staking balance
            //     AccountSubnetStake::<T>::insert(
            //         account_id,
            //         subnet_id,
            //         AccountSubnetStake::<T>::get(account_id, subnet_id)
            //             .saturating_add(stake_amount),
            //     );

            //     // -- increase total subnet stake
            //     TotalSubnetStake::<T>::mutate(subnet_id, |mut n| *n += stake_amount);

            //     // -- increase total stake overall
            //     TotalStake::<T>::mutate(|mut n| *n += stake_amount);

            //     // To ensure the AccountId that owns the PeerId, they must sign the PeerId for others to verify
            //     // This ensures others cannot claim to own a PeerId they are not the owner of
            //     // Self::validate_signature(&Encode::encode(&peer_id), &signature, &signer)?;

            //     // ========================
            //     // Insert peer into storage
            //     // ========================
            //     let classification = SubnetNodeClassification {
            //         node_class: SubnetNodeClass::Validator,
            //         start_epoch: 0,
            //     };

            //     let bounded_peer_id: BoundedVec<u8, DefaultMaxVectorLength> =
            //         BoundedVec::try_from(peer_id.clone().0).expect("Vec is within bounds");

            //     TotalSubnetNodeUids::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
            //     let current_uid = TotalSubnetNodeUids::<T>::get(subnet_id);

            //     HotkeySubnetNodeId::<T>::insert(subnet_id, account_id.clone(), current_uid);

            //     // Insert Subnet Node ID -> hotkey
            //     SubnetNodeIdHotkey::<T>::insert(subnet_id, current_uid, account_id.clone());

            //     // Insert hotkey -> coldkey
            //     HotkeyOwner::<T>::insert(account_id.clone(), account_id.clone());

            //     let subnet_node: SubnetNode<T::AccountId> = SubnetNode {
            //         id: current_uid,
            //         hotkey: account_id.clone(),
            //         peer_id: peer_id.clone(),
            //         bootnode_peer_id: peer_id.clone(),
            //         client_peer_id: peer_id.clone(),
            //         bootnode: Some(BoundedVec::new()),
            //         classification: classification,
            //         delegate_reward_rate: 0,
            //         last_delegate_reward_rate_update: 0,
            //         unique: Some(bounded_peer_id),
            //         non_unique: Some(BoundedVec::new()),
            //     };

            //     ColdkeyHotkeys::<T>::insert(
            //         &account_id.clone(),
            //         BTreeSet::from([account_id.clone()]),
            //     );

            //     ColdkeySubnetNodes::<T>::mutate(&account_id.clone(), |node_map| {
            //         node_map
            //             .entry(subnet_id)
            //             .or_insert_with(BTreeSet::new)
            //             .insert(current_uid);
            //     });

            //     HotkeySubnetId::<T>::insert(&account_id.clone(), subnet_id);

            //     // Insert SubnetNodesData
            //     SubnetNodesData::<T>::insert(subnet_id, current_uid, subnet_node);

            //     // Insert subnet peer account to keep peer_ids unique within subnets
            //     PeerIdSubnetNodeId::<T>::insert(subnet_id, peer_id.clone(), current_uid);

            //     // Increase total subnet nodes
            //     TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
            //     TotalNodes::<T>::mutate(|n: &mut u32| *n += 1);

            //     // ===================================
            //     // Give delegate stake balance to each user
            //     // ===================================
            //     let delegate_stake_amount = 1000;

            //     // -- increase account subnet staking shares balance
            //     AccountSubnetDelegateStakeShares::<T>::mutate(
            //         account_id.clone(),
            //         subnet_id,
            //         |mut n| n.saturating_accrue(delegate_stake_amount),
            //     );

            //     // -- increase total subnet delegate stake balance
            //     TotalSubnetDelegateStakeBalance::<T>::mutate(subnet_id, |mut n| {
            //         n.saturating_accrue(delegate_stake_amount)
            //     });

            //     // -- increase total subnet delegate stake shares
            //     TotalSubnetDelegateStakeShares::<T>::mutate(subnet_id, |mut n| {
            //         n.saturating_accrue(delegate_stake_amount)
            //     });

            //     TotalDelegateStake::<T>::mutate(|mut n| n.saturating_accrue(delegate_stake_amount));

            //     // ===================================
            //     // Give node delegate stake balance to each user
            //     // ===================================
            //     let node_delegate_stake_amount = 1000;

            //     // -- increase account subnet staking shares balance
            //     AccountNodeDelegateStakeShares::<T>::mutate(
            //         (account_id.clone(), subnet_id, current_uid),
            //         |mut n| n.saturating_accrue(node_delegate_stake_amount),
            //     );

            //     // -- increase total subnet delegate stake balance
            //     TotalNodeDelegateStakeBalance::<T>::mutate(subnet_id, current_uid, |mut n| {
            //         n.saturating_accrue(node_delegate_stake_amount)
            //     });

            //     // -- increase total subnet delegate stake shares
            //     TotalNodeDelegateStakeShares::<T>::mutate(subnet_id, current_uid, |mut n| {
            //         n.saturating_accrue(node_delegate_stake_amount)
            //     });

            //     TotalNodeDelegateStake::<T>::mutate(|mut n| {
            //         n.saturating_accrue(node_delegate_stake_amount)
            //     });

            //     ColdkeyReputation::<T>::mutate(&account_id.clone(), |rep| {
            //         rep.lifetime_node_count = rep.lifetime_node_count.saturating_add(1);
            //         rep.total_active_nodes = rep.total_active_nodes.saturating_add(1);
            //     });

            //     let current_count = NodeRegistrationsThisEpoch::<T>::get(subnet_id);
            //     NodeRegistrationsThisEpoch::<T>::insert(subnet_id, current_count.saturating_add(1));

            //     count += 1;
            // }
        }
    }
}
