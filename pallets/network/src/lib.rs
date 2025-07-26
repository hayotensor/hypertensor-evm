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

// extern crate alloc;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;
use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchResult},
	traits::{tokens::WithdrawReasons, Get, Currency, ReservableCurrency, ExistenceRequirement, Randomness, EnsureOrigin},
	PalletId,
	ensure,
	fail,
	storage::bounded_vec::BoundedVec,
};
use frame_system::{self as system, ensure_signed};
use scale_info::prelude::string::String;
use scale_info::prelude::vec::Vec;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use sp_core::OpaquePeerId as PeerId;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, FromRepr};
use sp_runtime::traits::TrailingZeroInput;
use frame_system::pallet_prelude::OriginFor;
use sp_std::ops::BitAnd;
use sp_runtime::Saturating;
use scale_info::prelude::boxed::Box;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
// #[cfg(test)]
// mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// ./target/release/solochain-template-node --dev
// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;
// use crate::weights::WeightInfo;

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

// mod rewards;
mod rewards_v4;

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

		#[pallet::constant]
		type EpochLength: Get<u32>;

		#[pallet::constant]
		type EpochsPerYear: Get<u32>;

		#[pallet::constant]
		type StringLimit: Get<u32>;
	
		#[pallet::constant] // Initial transaction rate limit.
		type InitialTxRateLimit: Get<u32>;
			
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type DelegateStakeCooldownEpochs: Get<u32>;

		#[pallet::constant]
		type NodeDelegateStakeCooldownEpochs: Get<u32>;

		#[pallet::constant]
		type StakeCooldownEpochs: Get<u32>;

		#[pallet::constant]
		type DelegateStakeEpochsRemovalWindow: Get<u32>;

		#[pallet::constant]
		type MaxDelegateStakeUnlockings: Get<u32>;
		
		#[pallet::constant]
		type MaxStakeUnlockings: Get<u32>;

		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

		#[pallet::constant]
		type MinProposalStake: Get<u128>;

		#[pallet::constant]
		type TreasuryAccount: Get<Self::AccountId>;

		#[pallet::constant]
		type OverwatchEpochEmissions: Get<u128>;
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
		SubnetRegistered { account_id: T::AccountId, name: Vec<u8>, subnet_id: u32 },
		SubnetActivated { subnet_id: u32 },
		SubnetDeactivated { subnet_id: u32, reason: SubnetRemovalReason },

		// Subnet Nodes
		SubnetNodeRegistered { 
			subnet_id: u32, 
			subnet_node_id: u32, 
			coldkey: T::AccountId, 
			hotkey: T::AccountId, 
			peer_id: PeerId, 
		},
		SubnetNodeActivated { subnet_id: u32, subnet_node_id: u32 },
		SubnetNodeDeactivated { subnet_id: u32, subnet_node_id: u32 },
		SubnetNodeRemoved { subnet_id: u32, subnet_node_id: u32 },
		SubnetNodeReactivated { subnet_id: u32, subnet_node_id: u32 },

		// Stake
		StakeAdded(u32, T::AccountId, T::AccountId, u128),
		StakeRemoved(u32, T::AccountId, T::AccountId, u128),

		SubnetDelegateStakeAdded(u32, T::AccountId, u128),
		SubnetDelegateStakeRemoved(u32, T::AccountId, u128),
		SubnetDelegateStakeSwapped(u32, u32, T::AccountId, u128),

		DelegateNodeStakeAdded { account_id: T::AccountId, subnet_id: u32, subnet_node_id: u32, amount: u128 },
		DelegateNodeStakeRemoved { account_id: T::AccountId, subnet_id: u32, subnet_node_id: u32, amount: u128 },
		DelegateNodeStakeSwapped { 
			account_id: T::AccountId, 
			from_subnet_id: u32, 
			from_subnet_node_id: u32, 
			to_subnet_id: u32, 
			to_subnet_node_id: u32, 
			amount: u128 
		},
		DelegateNodeToSubnetDelegateStakeSwapped { 
			account_id: T::AccountId, 
			from_subnet_id: u32, 
			from_subnet_node_id: u32, 
			to_subnet_id: u32, 
			amount: u128 
		},
		SubnetDelegateToNodeDelegateStakeSwapped { 
			account_id: T::AccountId, 
			from_subnet_id: u32, 
			to_subnet_id: u32, 
			to_subnet_node_id: u32, 
			amount: u128 
		},

		// Admin 
    SetMaxSubnets(u32),
    SetMinSubnetNodes(u32),
    SetMaxSubnetNodes(u32),
    SetMinStakeBalance(u128),
    SetTxRateLimit(u32),
		SetSubnetInflationFactor(u128),
		SetMinSubnetDelegateStakeFactor(u128),

		// Proposals
		Proposal { subnet_id: u32, proposal_id: u32, epoch: u32, plaintiff: T::AccountId, defendant: T::AccountId, plaintiff_data: Vec<u8> },
		ProposalChallenged { subnet_id: u32, proposal_id: u32, defendant: T::AccountId, defendant_data: Vec<u8> },
		ProposalAttested { subnet_id: u32, proposal_id: u32, account_id: T::AccountId, attestor_data: Vec<u8> },
		ProposalVote { subnet_id: u32, proposal_id: u32, account_id: T::AccountId, vote: VoteType },
		ProposalFinalized { subnet_id: u32, proposal_id: u32 },
		ProposalCanceled { subnet_id: u32, proposal_id: u32 },

		// Validation and Attestation
		ValidatorSubmission { subnet_id: u32, account_id: T::AccountId, epoch: u32},
		Attestation { subnet_id: u32, account_id: T::AccountId, epoch: u32},

		Slashing { subnet_id: u32, account_id: T::AccountId, amount: u128},

		// Rewards data
		RewardResult { subnet_id: u32, attestation_percentage: u128 },

		// Subnet owners
		SubnetNameUpdate { subnet_id: u32, owner: T::AccountId, prev_value: Vec<u8>, value: Vec<u8> },
		SubnetRepoUpdate { subnet_id: u32, owner: T::AccountId, prev_value: Vec<u8>, value: Vec<u8> },
		SubnetDescriptionUpdate { subnet_id: u32, owner: T::AccountId, prev_value: Vec<u8>, value: Vec<u8> },
		SubnetMiscUpdate { subnet_id: u32, owner: T::AccountId, prev_value: Vec<u8>, value: Vec<u8> },
		ChurnLimitUpdate { subnet_id: u32, owner: T::AccountId, value: u32 },
		RegistrationQueueEpochsUpdate { subnet_id: u32, owner: T::AccountId, value: u32 },
		ActivationGraceEpochsUpdate { subnet_id: u32, owner: T::AccountId, value: u32 },
		IdleClassificationEpochsUpdate { subnet_id: u32, owner: T::AccountId, value: u32 },
		IncludedClassificationEpochsUpdate { subnet_id: u32, owner: T::AccountId, value: u32 },
		MaxSubnetNodePenaltiesUpdate { subnet_id: u32, owner: T::AccountId, value: u32 },
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
		/// Errors should have helpful documentation associated with them.

		InvalidChurnLimit,
		InvalidRegistrationQueueEpochs,
		InvalidActivationGraceEpochs,
		InvalidIdleClassificationEpochs,
		InvalidIncludedClassificationEpochs,

		/// Subnet must be registering or activated, this error usually occurs during the enactment period
		SubnetMustBeRegisteringOrActivated,
		/// Subnet must be registering to perform this action
		SubnetMustBeRegistering,
		/// Node hasn't been initialized for required epochs to be an accountant
		NodeAccountantEpochNotReached,
		/// Maximum subnets reached
		MaxSubnets,
		/// Subnet registration require a coldkey whitelist for whitelisted nodes during the registration period
		ColdkeyWhitelistRequired,
		/// Account has subnet peer under subnet already
		InvalidSubnetNodeId,
		/// Not subnet owner
		NotSubnetOwner,
		/// Not pending subnet owner
		NotPendingSubnetOwner,
		/// No oending subnet owner exists
		NoPendingSubnetOwner,
		/// Subnet owner not exist, check the subnet ID is correct
		SubnetOwnerNotExist,
		/// Not Uid owner
		NotUidOwner,
		/// Must activate the designated epoch assigned
		NotStartEpoch,
		/// Subnet node already activated
		SubnetNodeAlreadyActivated,
		///
		SubnetNodeNotActivated,
		/// Peer ID already in use in subnet
		PeerIdExist,
		MaxSubnetNodes,
		/// Bootstrap peer ID already in use in subnet
		BootstrapPeerIdExist,
		/// Client peer ID already in use in subnet
		ClientPeerIdExist,
		/// Invalid client peer ID
		InvalidClientPeerId,
		/// Node ID already in use
		PeerIdNotExist,
		/// Subnet peer doesn't exist
		SubnetNodeNotExist,
		/// Subnet already exists
		SubnetExist,
		/// Subnet name already exists
		SubnetNameExist,
		/// Subnet repository already exists
		SubnetRepoExist,
		/// Subnet registration cooldown period not met
		SubnetRegistrationCooldown,
		/// Invalid registration block
		InvalidSubnetRegistrationBlocks,
		/// Subnet node must be unstaked to re-register to use the same balance
		InvalidSubnetRegistrationCooldown,
		/// Subnet doesn't exist
		InvalidSubnet,
		/// Minimum required subnet nodes not reached
		SubnetNodesMin,
		/// Maximum allowed subnet nodes reached
		SubnetNodesMax,
		/// Subnet state must be active to perform this action
		SubnetMustBeActive,
		/// Subnet state must be paused to perform this action
		SubnetMustBePaused,
		/// Subnet is paused, cannot perform this action
		SubnetIsPaused,
		/// Transaction rate limiter exceeded
		TxRateLimitExceeded,
		///
		HotkeySubnetNodeIdNotFound,
		/// PeerId format invalid
		InvalidPeerId,
		/// PeerId format invalid
		InvalidBootstrapPeerId,
		/// PeerId and Bootstrap PeerId must not match
		UniquePeerIdsRequired,
		/// The provided signature is incorrect.
		WrongSignature,
		InvalidSubnetId,
		/// Coldkey not whitelisted to register
		ColdkeyRegistrationWhitelist,
		MaxRegisteredNodes,
		/// Wallet doesn't have enough balance to register subnet
		NotEnoughBalanceToRegisterSubnet,

		/// Cannot deactivate node if current epochs validator
		IsValidatorCannotDeactivate,

		/// NodeRemovalStakePercentageDelta is above 100.0%
		InvalidNodeRemovalStakePercentageDelta,
		///
		InvalidNodeRemovalReputationScorePercentageDelta,

		/// Maximum amount of subnet registrations surpassed, see subnet `node_registration_interval` for more information
		MaxSubnetRegistrationReached,
		/// Maximum `node_registration_interval` parameter entered during subnet registration
		MaxSubnetRegistration,

		/// Maximum amount of subnet activations surpassed, see subnet `node_activation_interval` for more information
		MaxSubnetActivationReached,
		/// Maximum `node_activation_interval` parameter entered during subnet activation
		MaxSubnetActivation,

		/// This subnet only allows removal via penalties, not programmatic replacement.
		ConsensusRemovalSystemInUse,

		///
		InvalidDelegateStakeRewardsPercentage,
		InvalidMaxRegisteredNodes,
		InvalidSubnetRegistrationInitialColdkeys,
		InvalidSubnetMinStake,
		InvalidSubnetMaxStake,
		// Min stake must be lesser than the max stake
		InvalidSubnetStakeParameters,
		InvalidMinDelegateStakePercentage,
		
		DelegateStakeTransferPeriodExceeded,
		MustUnstakeToRegister,
		// Admin
		/// Consensus block epoch_length invalid, must reach minimum
		InvalidEpochLengthsInterval,
		/// Invalid maximimum subnets, must not exceed maximum allowable
		InvalidMaxSubnets,
		NoAvailableSlots,
		/// Invalid min subnet nodes, must not be less than minimum allowable
		InvalidMinSubnetNodes,
		/// Invalid maximimum subnet nodes, must not exceed maximimum allowable
		InvalidMaxSubnetNodes,
		/// Invalid minimum stake balance, must be greater than or equal to minimim required stake balance
		InvalidMinStakeBalance,
		/// Invalid percent number, must be in 1e4 format. Used for elements that only require correct format
		InvalidPercent,

		// Staking
		/// u128 -> BalanceOf conversion error
		CouldNotConvertToBalance,
		/// Not enough balance on Account to stake and keep alive
		NotEnoughBalanceToStake,
		NotEnoughBalance,
		/// Required unstake epochs not met based on
		RequiredUnstakeEpochsNotMet,
		/// Amount will kill account
		BalanceWithdrawalError,
		/// Not enough stake to withdraw
		NotEnoughStakeToWithdraw,
		MaxStakeReached,
		// if min stake not met on both stake and unstake
		MinStakeNotReached,
		// delegate staking
		CouldNotConvertToShares,
		// 
		MaxDelegatedStakeReached,
		//
		InsufficientCooldown,
		//
		UnstakeWindowFinished,
		//
		MaxUnlockingsPerEpochReached,
		//
		MaxUnlockingsReached,
		//
		NoDelegateStakeUnbondingsOrCooldownNotMet,
		NoStakeUnbondingsOrCooldownNotMet,
		// Conversion to balance was zero
		InsufficientBalanceToSharesConversion,
		MinDelegateStake,
		/// Elected validator on current epoch cannot unstake to ensure they are able to be penalized
		ElectedValidatorCannotUnstake,
		MinActiveNodeStakeEpochs,

		/// Not enough balance to withdraw bid for proposal
		NotEnoughBalanceToBid,

		QuorumNotReached,

		/// Dishonest propsal type
		PropsTypeInvalid,

		PartiesCannotVote,

		ProposalNotExist,
		ProposalNotChallenged,
		ProposalChallenged,
		ProposalChallengePeriodPassed,
		PropsalAlreadyChallenged,
		NotChallenger,
		NotEligible,
		AlreadyVoted,
		VotingPeriodInvalid,
		ChallengePeriodPassed,
		DuplicateVote,
		PlaintiffIsDefendant,

		InvalidSubnetConsensusSubmission,
		SubnetInitializing,
		SubnetActivatedAlready,
		InvalidSubnetRemoval,

		// Validation and Attestation
		/// Subnet rewards data already submitted by validator
		SubnetRewardsAlreadySubmitted,
		/// Not epoch validator
		InvalidValidator,
		/// Already attested validator data
		AlreadyAttested,
		/// Invalid rewards data length
		InvalidRewardsDataLength,
		/// Score overflow
		ScoreOverflow,


		ProposalInvalid,
		NotDefendant,
		NotPlaintiff,
		ProposalUnchallenged,
		ProposalComplete,
		/// Subnet node as defendant has proposal activated already
		NodeHasActiveProposal,
		/// Not the key owner
		NotKeyOwner,
		/// Not owner of hotkey that owns Subnet Node
		NotSubnetNodeOwner,
		/// Subnet Node param A must be unique
		SubnetNodeUniqueParamTaken,
		/// Subnet node param A is already set
		SubnetNodeUniqueParamIsSet,
		/// Subnet node params must be Some
		SubnetNodeNonUniqueParamMustBeSome,
		/// Non unique Subnet Node parameters can be updated once per SubnetNodeNonUniqueParamUpdateInterval
		SubnetNodeNonUniqueParamUpdateIntervalNotReached,
		/// Key owner taken
		KeyOwnerTaken,
		// Hotkey already registered to coldkey
		HotkeyAlreadyRegisteredToColdkey,
		// Hotkey not registered to coldkey
		OldHotkeyNotRegistered,
		/// Invalid identity, this error occurs if the vector is empty
		InvalidIdentity,
		/// Identity is taken by another coldkey
		IdentityTaken,
		/// No pending identity under given identity
		NoPendingIdentityOwner,
		/// No pending identity owner under given identity
		NotPendingIdentityOwner,
		/// No change between current and new delegate reward rate, make sure to increase or decrease it
		NoDelegateRewardRateChange,
		/// Invalid delegate reward rate above 100%
		InvalidDelegateRewardRate,
		/// Rate of change to great for decreasing reward rate, see MaxRewardRateDecrease
		SurpassesMaxRewardRateDecrease,
		/// Too many updates to reward rate in the RewardRateUpdatePeriod
		MaxRewardRateUpdates,
		/// Invalid curve parameters
		InvalidCurveParameters,
		/// Transactions are paused
		Paused,
		SubnetNotActive,

		// Keys
		/// Hotkey has an owner and hotkeys must be unique to each node. If you're the owner, use a fresh hotkey
		HotkeyHasOwner,
		ColdkeyMatchesHotkey,

		NoCommitFound,
		NoCommitsFound,
		RevealMismatch,
		CommitsEmpty,
		AlreadyCommitted,
		/// Invalid subnet weight, must be below percentage factor 1e18
		InvalidWeight,
		InvalidOverwatchNode,
		MaxOverwatchNodes,
	}
	
	/// Subnet data
	///
	/// # Arguments
	///
	/// * `id` - Unique identifier.
	/// * `name` - Path to download the model, this can be HuggingFace, IPFS, anything.
	/// * `state` - Registered or Active.
	/// * `registered` - Epoch subnet registered.
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetData {
		pub id: u32,
		pub name: Vec<u8>,
		pub repo: Vec<u8>,
		pub description: Vec<u8>,
		pub misc: Vec<u8>,
		pub state: SubnetState,
		pub start_epoch: u32,
	}

	#[derive(Default, EnumIter, FromRepr, Copy, Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, Ord, scale_info::TypeInfo)]
  pub enum SubnetState {
		#[default] Registered,
    Active,
		Paused,
  }

	#[derive(Default, EnumIter, FromRepr, Copy, Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, Ord, scale_info::TypeInfo)]
  pub enum KeyType {
		#[default] Rsa,
		Ed25519,
		Secp256k1,
		Ecdsa,
  }

	/// Subnet data used before activation
	///
	/// # Arguments
	///
	/// * name: Unique name of subnet
	/// * repo: Repository URL to open-sourced subnet code base
	/// * description: Description of subnet
	/// * misc: Miscillanous information 
	/// * churn_limit: Number of subnet activations per epoch
	/// * min_stake: Minimum stake balance to register a Subnet Node in the subnet
	/// * max_stake: Maximum stake balance to register a Subnet Node in the subnet
	/// * delegate_stake_percentage: Percentage of emissions that are allocated to delegate stakers
	/// * registration_queue_epochs: Number of epochs for registered nodes to be in queue before activation
	/// * activation_grace_epochs: Grace period following `registration_queue_epochs` to activate
	/// * queue_classification_epochs: Number of epochs in "Idle" classification (See SubnetNodeClass)
	/// * included_classification_epochs: Number of epochs in "Included" classification (See SubnetNodeClass)
	/// * max_node_penalties: Number of penalties to be removed
	/// * initial_coldkeys: List of initial coldkeys that can register while subnet is in registration
	/// * max_registered_nodes: Maximum number of nodes that can be registered at any time
	///
	///
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RegistrationSubnetData<AccountId> {
		pub name: Vec<u8>,
		pub repo: Vec<u8>,
		pub description: Vec<u8>,
		pub misc: Vec<u8>,
		pub churn_limit: u32,
		pub min_stake: u128,
		pub max_stake: u128,
		pub delegate_stake_percentage: u128,
		pub registration_queue_epochs: u32,
		pub activation_grace_epochs: u32,
		pub queue_classification_epochs: u32,
		pub included_classification_epochs: u32,
		pub max_node_penalties: u32,
		pub initial_coldkeys: BTreeSet<AccountId>,
		pub max_registered_nodes: u32,
		pub node_removal_system: NodeRemovalSystem,
		pub key_types: BTreeSet<KeyType>,
	}

	/// Removal system
	///
	/// How activating nodes remove other nodes when the subnet is at its max node count.
	/// 
	/// # Arguments
	///
	/// *Consensus: Nodes can only be removed based on consensus, i.e., by hitting max penalties
	/// *Stake: Lowest staked & newest node is removed if activating node has a higher stake balance
	///		- If using this, there should be a min - max range for stake balance to allow balance competing.
	/// *Reputation: Lowest rated & newest node is removed if activating node has a higher reputation and age.
	#[derive(Default, EnumIter, FromRepr, Copy, Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, Ord, scale_info::TypeInfo)]
	pub enum NodeRemovalSystem {
		#[default] Consensus,
    Stake,
    Reputation,
  }

	/// Reputation node removal system parameters
	///
	/// How activating nodes remove other nodes when the subnet is at its max node count.
	/// 
	/// # Arguments
	///
	/// *percentage_score_delta: Score delta in relation to activating node a potential removable node must be below to be removed
	/// *min_score_threshold: The minimum score to be immune from being removed
	/// *average_attestation_threshold: The minimum average attetation ratio to be immune from being removed
	pub struct ReputationNodeRemovalParameters {
		pub percentage_score_delta: u128,
		pub min_score_threshold: u128,
		pub average_attestation_threshold: u128,
	}

	/// Condition types
	///
	/// # Logic expression for node removal policy
	///
	/// # Options
	///
	/// Below are the logic expression options that can be used to qualify a node to be removed
	///
	/// # Hard
	///
	/// HardBelowScore: Below score (see Reputation)
	/// HardBelowAverageAttestation: Below average attestation ratio (see Reputation)
	/// HardBelowNodeDelegateStakeRate: Below node delegate stake rate (see stake/delegate_staking)
	///
	/// # Deltas
	/// - Note: Percent uses 1e18 as u128 (see math.rs)
	///
	/// DeltaBelowScore: Delta % below activating node score
	/// DeltaBelowAverageAttestation: Delta % below activating node average attestation ratio
	/// DeltaBelowNodeDelegateStakeRate: Delta % below activating node delegate stake rate
	/// DeltaBelowNodeDelegateStakeBalance: Delta % below activating node delegate stake balance (users staked to node) (see stake/delegate_staking)
	/// DeltaBelowStakeBalance: Delta % below activating node stake balance (only works if subnet has a min-max stake range for nodes)
	#[derive(Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, Ord, scale_info::TypeInfo)]
	pub enum NodeRemovalConditionType {
		// hard
		HardBelowScore(u128),
		HardBelowAverageAttestation(u128),
		HardBelowNodeDelegateStakeRate(u128),

		// deltas
		DeltaBelowScore(u128),
		DeltaBelowAverageAttestation(u128),
		DeltaBelowNodeDelegateStakeRate(u128),
		DeltaBelowNodeDelegateStakeBalance(u128),
		DeltaBelowStakeBalance(u128),
	}

	/// A tree-based logic system where node removal conditions are expressed as composable Boolean 
	/// logic expressions (AND, OR, XOR, NOT) over measurable properties
	#[derive(Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, Ord, scale_info::TypeInfo)]
	pub enum LogicExpr {
		And(Box<LogicExpr>, Box<LogicExpr>),
		Or(Box<LogicExpr>, Box<LogicExpr>),
		Xor(Box<LogicExpr>, Box<LogicExpr>),
		Not(Box<LogicExpr>),
		Condition(NodeRemovalConditionType),
	}

	#[derive(Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, Ord, scale_info::TypeInfo)]
	pub struct NodeRemovalPolicy {
		pub logic: LogicExpr,
	}

	pub struct SubnetInfo<AccountId> {
		pub id: u32,
		pub name: Vec<u8>,
		pub repo: Vec<u8>,
		pub description: Vec<u8>,
		pub misc: Vec<u8>,
		pub state: SubnetState,
		pub start_epoch: u32,
		pub churn_limit: u32,
		pub min_stake: u128,
		pub max_stake: u128,
		pub delegate_stake_percentage: u128,
		pub registration_queue_epochs: u32,
		pub activation_grace_epochs: u32,
		pub queue_classification_epochs: u32,
		pub included_classification_epochs: u32,
		pub max_node_penalties: u32,
		pub initial_coldkeys: Option<BTreeSet<AccountId>>,
		pub max_registered_nodes: u32,
		pub owner: AccountId,
		pub registration_epoch: u32,
		pub node_removal_system: NodeRemovalSystem,
		pub key_types: BTreeSet<KeyType>,
		pub slot_index: u32,
		// TODO: Add ID (non-uid between min-max subnets)
	}

	/// Subnet Node
	/// 
	/// # Arguments
	///
	/// * id: Subnet node ID
	/// * hotkey: Unique hotkey
	/// * peer_id: Unique peer ID (used for subnet communication/proof-of-stake)
	/// * bootstrap_peer_id: Unique bootstrap peer ID (used as a bootnode)
	/// * client_peer_id: Unique client peer ID (used for client-only node)
	/// * classification: SubnetNodeClassification
	/// * delegate_reward_rate: Delegate stakers reward rate
	/// * last_delegate_reward_rate_update: Last block rate was updated
	/// * a: Miscellaneous data
	/// * b: Miscellaneous data
	/// * c: Miscellaneous data
	///
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, scale_info::TypeInfo)]
	pub struct SubnetNode<AccountId> {
		pub id: u32,
		pub hotkey: AccountId,
		pub peer_id: PeerId,
		pub bootstrap_peer_id: PeerId,
		pub client_peer_id: PeerId,
		pub classification: SubnetNodeClassification,
		pub delegate_reward_rate: u128,
		pub last_delegate_reward_rate_update: u32,
		pub a: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
		pub b: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
		pub c: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNodeInfo<AccountId> {
		pub subnet_node_id: u32,
		pub coldkey: AccountId,
		pub hotkey: AccountId,
		pub peer_id: PeerId,
		pub bootstrap_peer_id: PeerId,
		pub client_peer_id: PeerId,
		pub identity: ColdkeyIdentityData,
		pub classification: SubnetNodeClassification,
		pub delegate_reward_rate: u128,
		pub last_delegate_reward_rate_update: u32,
		pub a: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
		pub b: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
		pub c: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
		pub stake_balance: u128,
	}

	/// Subnet node classes
	/// 
	/// # Arguments
	///
	/// *Deactivated: Subnet node is temporarily activated (done manually). Available to Validator class only.
	/// *Registered: Subnet node registered, not included in consensus
	/// *Idle: Subnet node is activated as queue, unless subnet is registering, and automatically updates on the first successful consensus epoch
	/// *Included: Subnet node automatically updates to Included from Idle on the first successful consensus epoch after being Idle
	/// *Validator: Subnet node updates to Submittble from Included on the first successful consensus epoch they are included in consensus data
	#[derive(Default, EnumIter, FromRepr, Copy, Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, Ord, scale_info::TypeInfo)]
	pub enum SubnetNodeClass {
		Deactivated,
		#[default] Registered,
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

    /// Decrements the node class, but if already at the lowest level, stays at Deactivated.
    pub fn previous(&self) -> Self {
			if *self == Self::Deactivated {
					return Self::Deactivated; // Stay at the lowest level
			}
			let new_value = (*self as usize) - 1; // Decrement the enum value
			Self::from_repr(new_value).unwrap_or(*self) // If out of bounds, return the current value
    }
	}
	
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Ord, PartialOrd, scale_info::TypeInfo)]
	pub struct SubnetNodeClassification {
		pub node_class: SubnetNodeClass,
		pub start_epoch: u32,
	}

	impl<AccountId> SubnetNode<AccountId> {
		pub fn has_classification(&self, required: &SubnetNodeClass, epoch: u32) -> bool {
			self.classification.node_class >= *required && self.classification.start_epoch <= epoch
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

	/// Incentives protocol format V2 (not in use)
	///
	/// Scoring is calculated off-chain between subnet nodes hosting AI subnets together
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNodeIncentives {
		pub uid: u32,
		pub score: u128,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, scale_info::TypeInfo)]
	pub struct ColdkeyIdentityData {
		pub name: BoundedVec<u8, DefaultMaxUrlLength>,
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

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct ConsensusSubmissionData<AccountId> {
    pub validator_subnet_node_id: u32,
    pub attestation_ratio: u128,
    pub weight_sum: u128,
    pub data_length: u32,
		pub data: Vec<SubnetNodeConsensusData>,
		pub subnet_nodes: Vec<SubnetNode<AccountId>>,
  }

	/// Reasons for a subnets removal
	///
	/// # Enums
	///
	/// *MaxPenalties: Subnet has surpasses the maximum penalties.
	/// *MinSubnetNodes: Subnet has went under the minumum subnet nodes required.
	/// *MinSubnetDelegateStake: Subnet delegate stake balance went under minimum required supply.
	/// *Council: Removed by council.
	/// *EnactmentPeriod: Subnet registered but never activated within the enactment period.
	/// *MaxSubnets: Lowest rated subnet removed if there are maximum subnets
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum SubnetRemovalReason {
    MaxPenalties,
		MinSubnetNodes,
		MinSubnetDelegateStake,
		Council,
		EnactmentPeriod,
		MaxSubnets,
		Owner,
		PauseExpired,
  }

	/// Attests format for consensus
	/// ``u64`` is the block number of the accounts attestation for subnets to utilize to measure attestation speed
	/// The blockchain itself doesn't utilize this data
	// pub type Attests<AccountId> = BTreeMap<AccountId, u64>;

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct ConsensusData<AccountId> {
		pub validator_id: u32, // Chosen validator of the epoch
		pub attests: BTreeMap<u32, u32>, // Count of attestations of the submitted data
		pub subnet_nodes: Vec<SubnetNode<AccountId>>,
		pub data: Vec<SubnetNodeConsensusData>, // Data submitted by chosen validator
		pub args: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>, // Optional arguements to pass for subnet to validate
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RewardsData {
    pub overall_subnet_reward: u128,
    pub subnet_owner_reward: u128,
    pub subnet_rewards: u128,
    pub delegate_stake_rewards: u128,
    pub subnet_node_rewards: u128,
  }

	// Overwatch nodes

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, scale_info::TypeInfo)]
	pub struct OverwatchNode<AccountId> {
		pub id: u32,
		pub hotkey: AccountId,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, scale_info::TypeInfo)]
	pub struct OverwatchCommit<Hash> {
		pub subnet_id: u32,
		pub weight: Hash,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, scale_info::TypeInfo)]
	pub struct OverwatchReveal {
		pub subnet_id: u32,
		pub weight: u128,
		pub salt: Vec<u8>,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, scale_info::TypeInfo)]
	pub struct OverwatchRevealData<AccountId> {
		pub subnet_id: u32,
		pub hotkey: AccountId,
		pub stake_balance: u128,
		pub weight: u128,
	}

	// #[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, scale_info::TypeInfo)]
	// pub struct AllOverwatchReveals {
	// 	pub total_stake: u128,
	// 	pub total_weight: u128,
	// 	pub weights: Vec<OverwatchReveal>,
	// }

	/// Vote types
	///
	/// # Enums
	///
	/// *Yay: Vote yay.
	/// *Nay: Vote nay.
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum VoteType {
    Yay,
    Nay,
  }

	/// Subnet node deactivation parameters
	///
	/// # Arguments
	///
	/// * `subnet_id` - Subnet ID.
	/// * `subnet_node_id` - Subnet node ID
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo, PartialOrd, Ord)]
	pub struct SubnetNodeDeactivation {
		pub subnet_id: u32,
		pub subnet_node_id: u32,
	}

	/// A moving average of the nodes count
	/// This is used against the minimum delegate staker as a multiplier
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	pub struct SubnetNodeEmaState {
		pub ema_nodes: u32,
		pub smoothing: u32, 
		pub last_update: u32,
	}

	/// Mapping of votes of a subnet proposal
	///
	/// # Arguments
	///
	/// * `yay` - Mapping of Subnet Node IDs voted yay.
	/// * `nay` - Mapping of Subnet Node IDs voted nay.
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct VoteParams {
		pub yay: BTreeSet<u32>,
		pub nay: BTreeSet<u32>,
	}

	/// Proposal parameters
	///
	/// # Arguments
	///
	/// * `subnet_id` - Subnet ID.
	/// * `plaintiff_id` - Proposers Subnet Node ID.
	/// * `defendant_id` - Defendants Subnet Node ID.
	/// * `plaintiff_bond` - Plaintiffs bond to create proposal (minimum proposal bond at time of proposal).
	/// * `defendant_bond` - Defendants bond to create proposal (matches plaintiffs bond).
	/// * `eligible_voters` - Mapping of Subnet Node IDs eligible to vote at time of proposal.
	/// * `votes` - Mapping of votes (`yay` and `nay`).
	/// * `start_block` - Block when proposer proposes proposal.
	/// * `challenge_block` - Block when defendant disputes proposal.
	/// * `plaintiff_data` - Proposers data to prove removal. Data is based on subnet removal reasons off-chain.
	/// * `defendant_data` - Defedants data to prove dispute.
	/// * `complete` - If proposal is complete.
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct ProposalParams {
		pub subnet_id: u32,
		pub plaintiff_id: u32,
		pub defendant_id: u32,
		pub plaintiff_bond: u128,
		pub defendant_bond: u128,
		pub eligible_voters: BTreeSet<u32>, // Those eligible to vote at time of the proposal
		pub votes: VoteParams,
		pub start_block: u32,
		pub challenge_block: u32,
		pub plaintiff_data: Vec<u8>,
		pub defendant_data: Vec<u8>,
		pub complete: bool,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
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

	#[pallet::type_value]
	pub fn DefaultZeroU32() -> u32 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultZeroU64() -> u64 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultZeroU128() -> u128 {
		0
	}
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
	#[pallet::type_value]
	pub fn DefaultTxRateLimit<T: Config>() -> u32 {
		T::InitialTxRateLimit::get()
	}
	#[pallet::type_value]
	pub fn DefaultLastTxBlock() -> u32 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultTxPause() -> bool {
		false
	}
	#[pallet::type_value]
	pub fn DefaultIdentityFee() -> u128 {
		// 1 TENSOR
		1000000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetPenaltyCount() -> u32 {
		16
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeRegistrationEpochs() -> u32 {
		16
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeQueuePeriod() -> u32 {
		1
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNode<T: Config>() -> SubnetNode<T::AccountId> {
		return SubnetNode {
			id: 0,
			hotkey: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
			peer_id: PeerId(Vec::new()),
			bootstrap_peer_id: PeerId(Vec::new()),
			client_peer_id: PeerId(Vec::new()),
			classification: SubnetNodeClassification {
				node_class: SubnetNodeClass::Registered,
				start_epoch: 0,
			},
			delegate_reward_rate: 0,
			last_delegate_reward_rate_update: 0,
			a: Some(BoundedVec::new()),
			b: Some(BoundedVec::new()),
      c: Some(BoundedVec::new()),
		};
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetNodes() -> u32 {
		512
	}
	#[pallet::type_value]
	pub fn DefaultMaxOverwatchNodes() -> u32 {
		64
	}
	#[pallet::type_value]
	pub fn DefaultMaxOverwatchNodePenalties() -> u32 {
		10
	}
	#[pallet::type_value]
	pub fn DefaultAccountTake() -> u128 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultRegisteredStakeCooldownEpochs<T: Config>() -> u32 {
		4
	}
	#[pallet::type_value]
	pub fn DefaultNetworkMaxStakeBalance() -> u128 {
		280000000000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultNetworkMinStakeBalance() -> u128 {
		1000e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMinActiveNodeStakeEpochs() -> u32 {
		100
	}
	#[pallet::type_value]
	pub fn DefaultSubnetMinStakeBalance() -> u128 {
		100+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultSubnetMaxStakeBalance() -> u128 {
		10000e+18 as u128
	}
	// #[pallet::type_value]
	// pub fn DefaultNetworkMaxStakeBalance() -> u128 {
	// 	1000e+18 as u128
	// }
	#[pallet::type_value]
	pub fn DefaultMinSubnetDelegateStakeFactor() -> u128 {
		// 0.1%

		1_000_000_000_000_000 // 1e18
	}
	#[pallet::type_value]
	pub fn DefaultMinDelegateStakeBalance() -> u128 {
		1000
	}
	#[pallet::type_value]
	pub fn DefaultMaxDelegateStakeBalance() -> u128 {
		1_000_000_000_000_000_000_000_000
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeTransferPeriod() -> u32 {
		1000
	}
	// #[pallet::type_value]
	// pub fn DefaultSubnetMinStakeBalance() -> u128 {
	// 	1000
	// }
	// #[pallet::type_value]
	// pub fn DefaultSubnetMaxStakeBalance() -> u128 {
	// 	1000
	// }
	#[pallet::type_value]
	pub fn DefaultDelegateStakeRewardsPercentage() -> u128 {
		// 1100
		// 110000000
		110000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeCooldown() -> u32 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeUnbondingLedger() -> BTreeMap<u32, u128> {
		// We use epochs because cooldowns are based on epochs
		// {
		// 	epoch_start: u32, // cooldown begin epoch (+ cooldown duration for unlock)
		// 	balance: u128,
		// }
		BTreeMap::new()
	}
	#[pallet::type_value]
	pub fn DefaultStakeUnbondingLedger() -> BTreeMap<u32, u128> {
		// We use epochs because cooldowns are based on epochs
		// {
		// 	epoch_start: u32, // cooldown begin epoch (+ cooldown duration for unlock)
		// 	balance: u128,
		// }
		BTreeMap::new()
	}
	#[pallet::type_value]
	pub fn DefaultBaseValidatorReward() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetNodePenalties() -> u32 {
		3
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeScorePenaltyThreshold() -> u32 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultSlashPercentage() -> u128 {
		31250000000000000
	}
	#[pallet::type_value]
	pub fn DefaultMaxSlashAmount() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMinAttestationPercentage() -> u128 {
		// 2/3
		660000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultMinVastMajorityAttestationPercentage() -> u128 {
		// 7/8
		875000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultSuperMajorityAttestationRatio() -> u128 {
		// 7/8
		875000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetNodes() -> u32 {
		// development and mainnet
		// 3
		// local && testnet
		1
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetRegistrationBlocks() -> u32 {
		// 9 days at 6s blocks
		// 129_600
		
		// Testnet && Local 150 blocks ||| 15 minutes
		150
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetRegistrationBlocks() -> u32 {
		// 21 days at 6s blocks
		// 302_400

		// Testnet 3 days
		43200
	}
	#[pallet::type_value]
	pub fn DefaultSubnetActivationEnactmentPeriod() -> u32 {
		// 3 days at 6s blocks
		43_200
	}
	#[pallet::type_value]
	pub fn DefaultSubnetRegistrationEpochs<T: Config>() -> u32 {
		T::EpochsPerYear::get() / 52
	}
	#[pallet::type_value]
	pub fn DefaultSubnetActivationEnactmentEpochs<T: Config>() -> u32 {
		// T::EpochsPerYear::get() / 52
		T::EpochsPerYear::get() / 104
	}	
	#[pallet::type_value]
	pub fn DefaultMaxSubnets() -> u32 {
		64
	}
	#[pallet::type_value]
	pub fn DefaultMaxVectorLength() -> u32 {
		1024
	}
	#[pallet::type_value]
	pub fn DefaultMaxUrlLength() -> u32 {
		1024
	}
	#[pallet::type_value]
	pub fn DefaultMaxSocialIdLength() -> u32 {
		255
	}
	#[pallet::type_value]
	pub fn DefaultValidatorArgsLimit() -> u32 {
		4096
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetRegistrationFee() -> u128 {
		100e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetRegistrationFee() -> u128 {
		1000e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultSubnetRegistrationInterval() -> u32 {
		// Based on blocks
		// 1 week based on 6s blocks using epochs
		// 1008
		// Testnet:
		// * 1 hour, 600 blocks, 6 epochs
		100
	}
	#[pallet::type_value]
	pub fn DefaultMaxRegisteredSubnetNodes() -> u32 {
		8
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetRegistrationInterval() -> u32 {
		// ~1 month
		438_290
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetActivationInterval() -> u32 {
		// ~1 month
		438_290
	}
	#[pallet::type_value]
	pub fn DefaultSubnetOwnerPercentage() -> u128 {
		// 23%
		230000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultSubnetInflationFactor() -> u128 {
		// 40.0%
		400000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultInflationAdjFactor() -> u128 {
		// 15%
		150000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultSubnetInflationAdjFactor() -> u128 {
		// Utilization steepness as `k`
		// 200%
		2000000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeInflationAdjFactor() -> u128 {
		// Utilization steepness as `k`
		// 200%
		2000000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultTerminalInflationFactor() -> u128 {
		// 15%
		150000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultUtilizationLowerBound() -> u128 {
		// 10.0%
		100000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultUtilizationUpperBound() -> u128 {
		// 50.0%
		500000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultSigmoidMidpoint() -> u128 {
		// 50.0%
		500000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultSigmoidSteepness() -> u128 {
		7
	}
	// #[pallet::type_value]
	// pub fn DefaultDeactivationLedger<T: Config>() -> BTreeSet<SubnetNodeDeactivation> {
	// 	BTreeSet::new()
	// }
	#[pallet::type_value]
	pub fn DefaultMaxDeactivations() -> u32 {
		512
	}
	#[pallet::type_value]
	pub fn DefaultChurnDenominator() -> u32 {
		4
	}
	#[pallet::type_value]
	pub fn DefaultChurnLimit() -> u32 {
		4
	}
	#[pallet::type_value]
	pub fn DefaultMinChurnLimit() -> u32 {
		// Must allow at least one node activation per epoch
		1
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetPauseEpochs<T: Config>() -> u32 {
		// 3 days
		T::EpochsPerYear::get() / 120
	}
	#[pallet::type_value]
	pub fn DefaultMaxChurnLimit() -> u32 {
		// Must only enable up to 64 node activations per epoch
		64
	}
	#[pallet::type_value]
	pub fn DefaultMinRegistrationQueueEpochs() -> u32 {
		// Require at least one epoch in registration queue
		1
	}
	#[pallet::type_value]
	pub fn DefaultMaxRegistrationQueueEpochs<T: Config>() -> u32 {
		// Max queue of 1 month
		T::EpochsPerYear::get() / 12
	}
	#[pallet::type_value]
	pub fn DefaultMinActivationGraceEpochs() -> u32 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultMaxActivationGraceEpochs<T: Config>() -> u32 {
		// Max grace period of one week
		T::EpochsPerYear::get() / 52
	}
	#[pallet::type_value]
	pub fn DefaultMinIdleClassificationEpochs() -> u32 {
		// Require at least one epoch in the queue classification
		1
	}
	#[pallet::type_value]
	pub fn DefaultMaxIdleClassificationEpochs<T: Config>() -> u32 {
		// Max queue classification of one week
		T::EpochsPerYear::get() / 52
	}
	#[pallet::type_value]
	pub fn DefaultMinIncludedClassificationEpochs() -> u32 {
		// Require at least one epoch in the included classification
		1
	}
	#[pallet::type_value]
	pub fn DefaultMaxIncludedClassificationEpochs<T: Config>() -> u32 {
		// Max queue classification of one week
		T::EpochsPerYear::get() / 52
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetMinStake() -> u128 {
		100e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetMinStake() -> u128 {
		1000e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetMaxStake() -> u128 {
		// Matches the DefaultMinSubnetMinStake
		100e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetMaxStake() -> u128 {
		10000e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMinDelegateStakePercentage() -> u128 {
		// 2.0%
		20000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultMaxDelegateStakePercentage() -> u128 {
		// 100.0%
		1000000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultActivationGraceEpochs() -> u32 {
		4
	}
	#[pallet::type_value]
	pub fn DefaultMaxRegisteredNodes() -> u32 {
		9
	}
	#[pallet::type_value]
	pub fn DefaultMinMaxRegisteredNodes() -> u32 {
		1
	}
	#[pallet::type_value]
	pub fn DefaultRegistrationQueueEpochs() -> u32 {
		4
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeRemovalSystem() -> NodeRemovalSystem {
		NodeRemovalSystem::Consensus
	}
	#[pallet::type_value]
	pub fn DefaultNodeRemovalStakePercentageDelta() -> u128 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultNodeRemovalReputationScorePercentageDelta() -> u128 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultNodeRemovalStakeBalancePercentageDelta() -> u128 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultNodeRemovalDelegateStakeRatePercentageDelta() -> u128 {
		0
	}	
	#[pallet::type_value]
	pub fn DefaultNodeRemovalReputationScoreMin() -> u128 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultNodeRemovalImmunityEpochs() -> u32 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultIdleClassificationEpochs() -> u32 {
		4
	}
	#[pallet::type_value]
	pub fn DefaultIncludedClassificationEpochs() -> u32 {
		4
	}
	#[pallet::type_value]
	pub fn DefaultMaxDeactivationEpochs() -> u32 {
		4
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeNonUniqueParamUpdateInterval() -> u32 {
		1
	}
	#[pallet::type_value]
	pub fn DefaultRewardRateUpdatePeriod() -> u32 {
		// 1 day at 6 seconds a block (86,000s per day)
		14400
	}
	#[pallet::type_value]
	pub fn DefaultMaxRewardRateDecrease() -> u128 {
		// 1%
		10_000_000
	}
	#[pallet::type_value]
	pub fn DefaultNodeAttestationRemovalThreshold() -> u128 {
		// 8500
		850000000
	}
	#[pallet::type_value]
	pub fn DefaultProposalParams<T: Config>() -> ProposalParams {
		return ProposalParams {
			subnet_id: 0,
			plaintiff_id: 0,
			defendant_id: 0,
			plaintiff_bond: 0,
			defendant_bond: 0,
			eligible_voters: BTreeSet::new(),
			votes: VoteParams {
				yay: BTreeSet::new(),
				nay: BTreeSet::new(),
			},
			start_block: 0,
			challenge_block: 0,
			plaintiff_data: Vec::new(),
			defendant_data: Vec::new(),
			complete: false,
		};
	}
	#[pallet::type_value]
	pub fn DefaultProposalMinSubnetNodes() -> u32 {
		16
	}
	#[pallet::type_value]
	pub fn DefaultVotingPeriod() -> u32 {
		// 7 days
		100800
	}
	#[pallet::type_value]
	pub fn DefaultChallengePeriod() -> u32 {
		// 7 days in blocks
		100800
	}
	#[pallet::type_value]
	pub fn DefaultProposalQuorum() -> u128 {
		// 75.0%
		750000000
	}
	#[pallet::type_value]
	pub fn DefaultProposalConsensusThreshold() -> u128 {
		// 66.0%
		660000000
	}
	#[pallet::type_value]
	pub fn DefaultProposalsCount() -> u32 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultProposalBidAmount() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultReputationIncreaseFactor() -> u128 {
		// 0.5
		500000000000000
	}
	#[pallet::type_value]
	pub fn DefaultReputationDecreaseFactor() -> u128 {
		// 0.5
		500000000000000
	}
	#[pallet::type_value]
	pub fn DefaultColdkeyReputation() -> Reputation {
		return Reputation {
			start_epoch: 0,
			score: 500_000_000_000_000, // 0.5 / 50%
			lifetime_node_count: 0,
			total_active_nodes: 0,
			total_increases: 0,
			total_decreases: 0,
			average_attestation: 0,
			last_validator_epoch: 0,
			ow_score: 500_000_000_000_000 // 0.5 / 50%
		}
	}
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
		}
	}
	#[pallet::type_value]
	pub fn MaxSlots<T: Config>() -> u32 {
		T::EpochLength::get()
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeCountEMA() -> u128 {
		// Min nodes * 1e18
		// Mainnet
		// 3000000000000000000
		// Local
		// 1000000000000000000
		0
	}
	#[pallet::type_value]
	pub fn DefaultOverwatchMinStakeBalance() -> u128 {
		100e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaulMaxMinDelegateStakeMultiplier() -> u128 {
		// 400%
		4000000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeWeightFactor() -> u128 {
		// 90.0%
		900000000000000000
	}



	
	// 
	// Subnet elements
	//

	/// Count of subnets
	#[pallet::storage]
	pub type TotalSubnetUids<T> = StorageValue<_, u32, ValueQuery>;
	
	/// Slots for each subnet based no max subnets
	#[pallet::storage]
	pub type FriendlySubnetUids<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage] // friendly_subnet_id => subnet_id
	pub type FriendlyUidToUid<T> = StorageMap<_, Identity, u32, u32, ValueQuery>;

	/// Count of active subnets
	#[pallet::storage]
	pub type TotalActiveSubnets<T> = StorageValue<_, u32, ValueQuery>;

	// Max subnets in the network
	#[pallet::storage]
	#[pallet::getter(fn max_subnets)]
	pub type MaxSubnets<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnets>;

	// Mapping of each subnet stored by ID, uniqued by `SubnetName`
	// Stores subnet data by a unique id
	#[pallet::storage] // subnet_id => data struct
	pub type SubnetsData<T> = StorageMap<_, Identity, u32, SubnetData>;

	#[pallet::storage] // subnet_id => data struct
	pub type SubnetBootnodes<T> = StorageMap<_, Identity, u32, BoundedVec<u32, DefaultMaxVectorLength>, ValueQuery>;

	// Ensures no duplicate subnet paths within the network at one time
	// If a subnet name is voted out, it can be voted up later on and any
	// stakes attached to the subnet_id won't impact the re-initialization
	// of the subnet name.
	#[pallet::storage]
	#[pallet::getter(fn subnet_name)]
	pub type SubnetName<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32>;

	// Repository to subnet codebase
	#[pallet::storage]
	#[pallet::getter(fn subnet_repo)]
	pub type SubnetRepo<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32>;

	// Epoch subnet registered on
	#[pallet::storage] // subnet_id => blocks
	pub type SubnetRegistrationEpoch<T> = StorageMap<_, Identity, u32, u32>;

	// Owner of subnet (defaulted to registerer of subnet)
	#[pallet::storage] // subnet_id => AccountId
	pub type SubnetOwner<T: Config> = StorageMap<_, Identity, u32, T::AccountId>;

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetOwnerKeys<AccountId> {
		pub coldkey: AccountId,
		pub hotkey: AccountId,
	}

	// Owner of subnet (defaulted to registerer of subnet)
	#[pallet::storage] // subnet_id => AccountId
	pub type SubnetOwnerV2<T: Config> = StorageMap<_, Identity, u32, SubnetOwnerKeys<T::AccountId>>;

	// Owner of subnet (defaulted to registerer of subnet)
	#[pallet::storage] // subnet_id => AccountId
	pub type SubnetOwnerHotkey<T: Config> = StorageMap<_, Identity, u32, T::AccountId>;

	// Pending owner of subnet
	#[pallet::storage] // subnet_id => AccountId
	pub type PendingSubnetOwner<T: Config> = StorageMap<_, Identity, u32, T::AccountId>;
	
	// Percentage of rewards that allocates to subnet owners
	#[pallet::storage] // subnet_id => AccountId
	pub type SubnetOwnerPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultSubnetOwnerPercentage>;

	/// Subnet registration blocks
	/// Total blocks subnet is in registration to reach conditions to activate
	#[pallet::storage]
	pub type SubnetRegistrationEpochs<T: Config> = StorageValue<_, u32, ValueQuery, DefaultSubnetRegistrationEpochs<T>>;

	/// Time period allowable for subnet activation following registration period
	#[pallet::storage]
	pub type SubnetActivationEnactmentEpochs<T: Config> = StorageValue<_, u32, ValueQuery, DefaultSubnetActivationEnactmentEpochs<T>>;

	// Max epochs where consensus isn't formed before subnet being removed
	#[pallet::storage]
	pub type MaxSubnetPenaltyCount<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnetPenaltyCount>;
	
	// Count of epochs a subnet has consensus errors
	#[pallet::storage] // subnet_id => count
	pub type SubnetPenaltyCount<T> = StorageMap<
		_,
		Identity,
		u32,
		u32,
		ValueQuery,
	>;

	/// Min ChurnLimit
	#[pallet::storage]
	pub type MaxSubnetPauseEpochs<T: Config> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnetPauseEpochs<T>>;

	// Lower bound of registration fee
	#[pallet::storage]
	pub type MinSubnetRegistrationFee<T> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetRegistrationFee>;

	// Upper bound of registration fee
	#[pallet::storage]
	pub type MaxSubnetRegistrationFee<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSubnetRegistrationFee>;

	// Last epoch a subnet was registered
	#[pallet::storage]
	pub type LastSubnetRegistrationEpoch<T> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

	// Epochs per subnet registration
	// Also used for calculating the fee between the max and min registration fee
	// e.g. Amount of epochs required to go by after a subnet registers before another can
	#[pallet::storage]
	pub type SubnetRegistrationInterval<T> = StorageValue<_, u32, ValueQuery, DefaultSubnetRegistrationInterval>;

	//
	// Subnet reward slots
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
		
	// Election slots
	#[pallet::storage]
	pub type SubnetNodeElectionSlots<T> = StorageMap<
		_,
		Identity,
		u32,
		BoundedVec<u32, DefaultMaxSubnetNodes>,
		ValueQuery,
	>;

	// Subnet count of electable nodes
	#[pallet::storage]
	pub type TotalSubnetElectableNodes<T> = StorageMap<
		_,
		Identity,
		u32,
		u32,
		ValueQuery,
	>;

	// Network wide count of electable nodes
	#[pallet::storage]
	pub type TotalElectableNodes<T> = StorageValue<_, u32, ValueQuery>;

	// Track election slots to avoid iterating
	#[pallet::storage]
	pub type NodeSlotIndex<T> = StorageDoubleMap<
		_,
		Identity,
		u32,         // subnet_id
		Identity,
		u32,         // node_id
		u32,         // index in slot vec
		OptionQuery,
	>;

	// Minimum amount of nodes required per subnet
	// required for subnet activity
	#[pallet::storage]
	#[pallet::getter(fn min_subnet_nodes)]
	pub type MinSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMinSubnetNodes>;

	// Maximim nodes in a subnet at any given time
	#[pallet::storage]
	#[pallet::getter(fn max_subnet_nodes)]
	pub type MaxSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnetNodes>;

	// Count of nodes in the network
	#[pallet::storage]
	pub type TotalNodes<T> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

	// Count of active nodes in the network
	#[pallet::storage]
	pub type TotalActiveNodes<T> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

	// Count of total nodes in a subnet
	#[pallet::storage] // subnet_uid --> u32
	#[pallet::getter(fn total_subnet_nodes)]
	pub type TotalSubnetNodes<T> =
		StorageMap<_, Identity, u32, u32, ValueQuery>;

	// Count of active nodes in a subnet
	#[pallet::storage] // subnet_uid --> u32
	pub type TotalActiveSubnetNodes<T> =
		StorageMap<_, Identity, u32, u32, ValueQuery>;
	
	#[pallet::storage]
	pub type MaxMinDelegateStakeMultiplier<T> = StorageValue<_, u128, ValueQuery, DefaulMaxMinDelegateStakeMultiplier>;

	#[pallet::storage]
	pub type SubnetNodeCountEMA<T> = StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery, DefaultSubnetNodeCountEMA>;

	// subnet_id -> last block updated
	#[pallet::storage]
	pub type SubnetNodeCountEMALastUpdated<T> =
    StorageMap<_, Blake2_128Concat, u32, u32, ValueQuery, DefaultZeroU32>;

	/// Min ChurnLimit
	#[pallet::storage]
	pub type MinChurnLimit<T> = StorageValue<_, u32, ValueQuery, DefaultMinChurnLimit>;
	
	/// Max ChurnLimit
	#[pallet::storage]
	pub type MaxChurnLimit<T> = StorageValue<_, u32, ValueQuery, DefaultMaxChurnLimit>;

	/// Min RegistrationQueueEpochs
	#[pallet::storage]
	pub type MinRegistrationQueueEpochs<T> = StorageValue<_, u32, ValueQuery, DefaultMinRegistrationQueueEpochs>;

	/// Max RegistrationQueueEpochs
	#[pallet::storage]
	pub type MaxRegistrationQueueEpochs<T: Config> = StorageValue<_, u32, ValueQuery, DefaultMaxRegistrationQueueEpochs<T>>;
	
	/// Min ActivationGraceEpochs
	#[pallet::storage]
	pub type MinActivationGraceEpochs<T> = StorageValue<_, u32, ValueQuery, DefaultMinActivationGraceEpochs>;
	
	/// Max ActivationGraceEpochs
	#[pallet::storage]
	pub type MaxActivationGraceEpochs<T: Config> = StorageValue<_, u32, ValueQuery, DefaultMaxActivationGraceEpochs<T>>;

	/// Min IdleClassificationEpochs
	#[pallet::storage]
	pub type MinIdleClassificationEpochs<T> = StorageValue<_, u32, ValueQuery, DefaultMinIdleClassificationEpochs>;
	
	/// Max IdleClassificationEpochs
	#[pallet::storage]
	pub type MaxIdleClassificationEpochs<T: Config> = StorageValue<_, u32, ValueQuery, DefaultMaxIdleClassificationEpochs<T>>;

	/// Min IncludedClassificationEpochs
	#[pallet::storage]
	pub type MinIncludedClassificationEpochs<T> = StorageValue<_, u32, ValueQuery, DefaultMinIncludedClassificationEpochs>;

	/// Max IncludedClassificationEpochs
	#[pallet::storage]
	pub type MaxIncludedClassificationEpochs<T: Config> = StorageValue<_, u32, ValueQuery, DefaultMaxIncludedClassificationEpochs<T>>;

	#[pallet::storage]
	pub type MinSubnetMinStake<T> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetMinStake>;
	
	#[pallet::storage]
	pub type MaxSubnetMinStake<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSubnetMinStake>;
	
	#[pallet::storage]
	pub type MinSubnetMaxStake<T> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetMaxStake>;
	
	#[pallet::storage]
	pub type MaxSubnetMaxStake<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSubnetMaxStake>;
	
	#[pallet::storage]
	pub type MinDelegateStakePercentage<T> = StorageValue<_, u128, ValueQuery, DefaultMinDelegateStakePercentage>;
	
	#[pallet::storage]
	pub type MaxDelegateStakePercentage<T> = StorageValue<_, u128, ValueQuery, DefaultMaxDelegateStakePercentage>;

	/// Minimum MaxRegisteredNodes
	#[pallet::storage]
	pub type MinMaxRegisteredNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMinMaxRegisteredNodes>;

	/// Max amount of nodes that can activate per epoch
	#[pallet::storage] // subnet_uid --> u32
	pub type ChurnLimit<T> =
		StorageMap<_, Identity, u32, u32, ValueQuery, DefaultChurnLimit>;

	/// Length of epochs a node must be in the registration queue before they can activate
	#[pallet::storage] // subnet_uid --> u32
	pub type RegistrationQueueEpochs<T> =
		StorageMap<_, Identity, u32, u32, ValueQuery, DefaultRegistrationQueueEpochs>;

	/// Length of epochs a Idle classified node must be in that class for
	#[pallet::storage] // subnet_uid --> u32
	pub type IdleClassificationEpochs<T> =
		StorageMap<_, Identity, u32, u32, ValueQuery, DefaultIdleClassificationEpochs>;

	// Min required stake balance for a Subnet Node in a specified subnet
	#[pallet::storage]
	pub type SubnetMinStakeBalance<T> = StorageMap<_, Identity, u32, u128, ValueQuery, DefaultSubnetMinStakeBalance>;
		
	// Max stake balance for a Subnet Node in a specified subnet
	// A node can go over this amount as a balance but cannot add more above it
	#[pallet::storage]
	pub type SubnetMaxStakeBalance<T> = StorageMap<_, Identity, u32, u128, ValueQuery, DefaultSubnetMaxStakeBalance>;

	#[pallet::storage]
	pub type SubnetDelegateStakeRewardsPercentage<T> = StorageMap<_, Identity, u32, u128, ValueQuery, DefaultDelegateStakeRewardsPercentage>;

	/// Length of epochs an Included classified node must be in that class for
	/// This can be used in tandem with SubnetNodePenalties to ensure a node is included
	/// in consensus data before they are activated instead of automatically being upgraded
	/// to validator. (see rewards.rs to see how a node is upgraded to the Validator class)
	#[pallet::storage] // subnet_uid --> u32
	pub type IncludedClassificationEpochs<T> =
		StorageMap<_, Identity, u32, u32, ValueQuery, DefaultIncludedClassificationEpochs>;

	// Max epochs a subnet can be deactivated for until they are either able to be cleaned up and remove
	// or unable to reactivate
	#[pallet::storage] // subnet_uid --> u32
	pub type MaxDeactivationEpochs<T> =
		StorageMap<_, Identity, u32, u32, ValueQuery, DefaultMaxDeactivationEpochs>;

	// Epochs a registered & queued node can activate themselves in addition to their start_epoch
	#[pallet::storage] // subnet_uid --> u32
	pub type ActivationGraceEpochs<T> =
		StorageMap<_, Identity, u32, u32, ValueQuery, DefaultActivationGraceEpochs>;
	
	// Whitelist of coldkeys that nodes can register to a subnet during its registration period
	// Afterwards on subnet activation, this list is deleted and the subnet is now public
	// Because all subnets are expected to be P2P, each subnet starts as a blockchain would with
	// trusting nodes to ensure no malicious nodes can enter at the start.
	#[pallet::storage] // subnet_id => {..., AccountId, ...}
	pub type SubnetRegistrationInitialColdkeys<T: Config> = StorageMap<_, Identity, u32, BTreeSet<T::AccountId>>;
	
	#[pallet::storage] // subnet_uid --> u32
	pub type MaxRegisteredNodes<T> =
		StorageMap<_, Identity, u32, u32, ValueQuery, DefaultMaxRegisteredNodes>;

	#[pallet::storage] // subnet_uid --> u32
	pub type SubnetNodeRemovalSystem<T> =
		StorageMap<_, Identity, u32, NodeRemovalSystem, ValueQuery, DefaultSubnetNodeRemovalSystem>;

	#[pallet::storage]
	pub type NodeRemovalSystemV2<T> = StorageMap<_, Identity, u32, NodeRemovalPolicy, OptionQuery>;

	#[pallet::storage]
	pub type SubnetKeyTypes<T> = StorageMap<_, Identity, u32, BTreeSet<KeyType>, ValueQuery>;

	/// When subnet utilizes the node removal system Stake
	/// The percentage delta of the activating node versus the removal node minimum delta
	/// e.g. If the NodeRemovalStakePercentageDelta is 10% and the activating node has 100, 
	/// 		 the removing node must have a stake of less than 90
	#[pallet::storage]
	pub type NodeRemovalStakePercentageDelta<T> =
		StorageMap<_, Identity, u32, u128, ValueQuery, DefaultNodeRemovalStakePercentageDelta>;

	// The delta between the activating node and potential removable nodes
	#[pallet::storage]
	pub type NodeRemovalReputationScorePercentageDelta<T> =
		StorageMap<_, Identity, u32, u128, ValueQuery, DefaultNodeRemovalReputationScorePercentageDelta>;

	#[pallet::storage]
	pub type NodeRemovalStakeBalancePercentageDelta<T> =
		StorageMap<_, Identity, u32, u128, ValueQuery, DefaultNodeRemovalStakeBalancePercentageDelta>;

	#[pallet::storage]
	pub type NodeRemovalDelegateStakeRatePercentageDelta<T> =
		StorageMap<_, Identity, u32, u128, ValueQuery, DefaultNodeRemovalDelegateStakeRatePercentageDelta>;

	#[pallet::storage]
	pub type NodeRemovalDelegateStakeBalancePercentageDelta<T> =
		StorageMap<_, Identity, u32, u128, ValueQuery, DefaultZeroU128>;

	// The minimum score a node must have to NOT qualify to be removed
	#[pallet::storage]
	pub type NodeRemovalReputationScoreMin<T> =
		StorageMap<_, Identity, u32, u128, ValueQuery, DefaultNodeRemovalReputationScoreMin>;

	#[pallet::storage]
	pub type NodeRemovalImmunityEpochs<T> =
		StorageMap<_, Identity, u32, u32, ValueQuery, DefaultNodeRemovalImmunityEpochs>;

	#[pallet::storage] // subnet_id --> u32
	pub type TotalSubnetNodeUids<T: Config> = StorageMap<_, Identity, u32, u32, ValueQuery>;

	/// Coldkey identities

	/// Coldkey => Public Identity
	#[pallet::storage] 
	pub type ColdkeyIdentity<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ColdkeyIdentityData, ValueQuery, DefaultColdkeyIdentity>;

	// /// Public Identity => Pending Owner
	// #[pallet::storage] 
	// pub type PendingIdentityOwner<T: Config> = StorageMap<_, Identity, Vec<u8>, T::AccountId, ValueQuery, DefaultAccountId<T>>;

	// Hotkey => Coldkey
	#[pallet::storage]
	pub type HotkeyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultAccountId<T>>;

	// Coldkey => {Hotkeys}
	// This conditions unique hotkeys over the entire network and enables tracking hotkeys to coldkeys
	#[pallet::storage]
	pub type ColdkeyHotkeys<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BTreeSet<T::AccountId>, ValueQuery>;

	// Subnet ID => Hotkey => Subnet Node ID
	#[pallet::storage]
	pub type HotkeySubnetNodeId<T: Config> = StorageDoubleMap<_, Identity, u32, Blake2_128Concat, T::AccountId, u32, OptionQuery>;
	
	// Subnet ID => Subnet Node ID => Hotkey
	#[pallet::storage]
	pub type SubnetNodeIdHotkey<T: Config> = StorageDoubleMap<_, Identity, u32, Identity, u32, T::AccountId, OptionQuery>;
	
	// // Subnet ID => PeerId => Hotkey
	// #[pallet::storage]
	// pub type PeerIdHotkey<T: Config> = StorageDoubleMap<_, Identity, u32, Blake2_128Concat, PeerId, T::AccountId, OptionQuery>;

	#[pallet::storage] // subnet_id --> uid --> data
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

	/// Subnets that are deactivated are stored here before reactivation
	#[pallet::storage] // subnet_id --> uid --> data
	pub type DeactivatedSubnetNodesData<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Identity,
		u32,
		SubnetNode<T::AccountId>,
		ValueQuery,
		DefaultSubnetNode<T>,
	>;
	
	#[pallet::storage] // subnet_id --> peer_id --> subnet_node_id
	pub type PeerIdSubnetNodeId<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Blake2_128Concat,
		PeerId,
		u32,
		ValueQuery,
		DefaultZeroU32,
	>;

	#[pallet::storage] // subnet_id --> bootstrap_peer_id --> subnet_node_id
	pub type BootstrapPeerIdSubnetNodeId<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Blake2_128Concat,
		PeerId,
		u32,
		ValueQuery,
		DefaultZeroU32,
	>;

	#[pallet::storage] // subnet_id --> client_peer_id --> subnet_node_id
	pub type ClientPeerIdSubnetNode<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Blake2_128Concat,
		PeerId,
		u32,
		ValueQuery,
		DefaultZeroU32,
	>;

	// Used for unique parameters
	#[pallet::storage] // subnet_id --> param --> peer_id
	pub type SubnetNodeUniqueParam<T> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Blake2_128Concat,
		BoundedVec<u8, DefaultMaxVectorLength>,
		PeerId,
		ValueQuery,
		DefaultPeerId,
	>;
	
	#[pallet::storage]
	pub type SubnetNodeNonUniqueParamUpdateInterval<T> = 
		StorageValue<_, u32, ValueQuery, DefaultSubnetNodeNonUniqueParamUpdateInterval>;

	#[pallet::storage]
	pub type SubnetNodeNonUniqueParamLastSet<T> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Identity,
		u32,
		u32,
		ValueQuery,
		DefaultZeroU32,
	>;

	//
	// Network utility elements
	//

	#[pallet::storage] // ( tx_rate_limit )
	pub type TxRateLimit<T> = StorageValue<_, u32, ValueQuery, DefaultTxRateLimit<T>>;

	// Last transaction on rate limited functions
	#[pallet::storage] // key --> last_block
	pub type LastTxBlock<T: Config> =
		StorageMap<_, Identity, T::AccountId, u32, ValueQuery, DefaultLastTxBlock>;

	// Pause the network
	#[pallet::storage]
	pub type TxPause<T> = StorageValue<_, bool, ValueQuery, DefaultTxPause>;

	//
	// Validate / Attestation
	//

	#[pallet::storage] // subnet ID => epoch  => Subnet Node ID
	pub type SubnetElectedValidator<T> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Identity,
		u32,
		u32,
	>;

	#[pallet::storage] // subnet ID => epoch  => data
	pub type SubnetConsensusSubmission<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Identity,
		u32,
		ConsensusData<T::AccountId>,
	>;

	#[pallet::storage]
	pub type MinAttestationPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultMinAttestationPercentage>;

	#[pallet::storage]
	pub type MinVastMajorityAttestationPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultMinVastMajorityAttestationPercentage>;

	#[pallet::storage]
	pub type SuperMajorityAttestationRatio<T> = StorageValue<_, u128, ValueQuery, DefaultSuperMajorityAttestationRatio>;

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct DistributionData {
		pub validator_emissions: u128,
		pub weights: BTreeMap<u32, u128>,
	}

	// Epoch -> {total_issuance, (subnet_id, weight)}
	#[pallet::storage]
	pub type FinalSubnetEmissionWeights<T> = StorageMap<
		_,
		Identity,
		u32,
		DistributionData,
		ValueQuery,
	>;

	//
	// Rewards (validator, incentives)
	//

	// Base reward per epoch for validators
	// This is the base reward to subnet validators on successful attestation
	#[pallet::storage]
	pub type BaseValidatorReward<T> = StorageValue<_, u128, ValueQuery, DefaultBaseValidatorReward>;

	#[pallet::storage]
	pub type BaseSlashPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultSlashPercentage>;

	#[pallet::storage]
	pub type MaxSlashAmount<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSlashAmount>;
	
	// Max penalties a node can hae in a specified subnet, set by registrar & owner
	#[pallet::storage]
	pub type MaxSubnetNodePenalties<T> = StorageMap<
		_,
		Identity,
		u32,
		u32,
		ValueQuery,
		DefaultMaxSubnetNodePenalties
	>;

	#[pallet::storage]
	pub type SubnetNodeScorePenaltyThreshold<T> = StorageMap<
		_,
		Identity,
		u32,
		u32,
		ValueQuery,
		DefaultSubnetNodeScorePenaltyThreshold
	>;

	// A subnet nodes penalties count
	#[pallet::storage]
	pub type SubnetNodePenalties<T> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Identity,
		u32,
		u32,
		ValueQuery,
		DefaultZeroU32,
	>;

	// Tracking Subnet Node reputation based on their validator activity
	// This is used for the gateway to become a validator node to ensure they
	// are trustworthy

	/// Weight used to increase a subnet validator nodes reputation
	#[pallet::storage]
	pub type ReputationIncreaseFactor<T> = StorageValue<_, u128, ValueQuery, DefaultReputationIncreaseFactor>;	

	/// Weight used to decrease a subnet validator nodes reputation
	#[pallet::storage]
	pub type ReputationDecreaseFactor<T> = StorageValue<_, u128, ValueQuery, DefaultReputationDecreaseFactor>;	

	/// Tracks a coldkeys reputation using numerous data points
	#[pallet::storage]
	pub type ColdkeyReputation<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Reputation, ValueQuery, DefaultColdkeyReputation>;

	/// Tracks how many unique subnets a coldkey is in
	/// This is not 100% accurate is a subnet is removed and deleted
	/// This storage requires confirming the subnet is still active, not removed on subnet removal
	#[pallet::storage]
	pub type ColdkeySubnets<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BTreeSet<u32>, ValueQuery>;

	// Attestion percentage required to increment a nodes penalty count up
	#[pallet::storage]
	pub type NodeAttestationRemovalThreshold<T> = StorageValue<_, u128, ValueQuery, DefaultNodeAttestationRemovalThreshold>;

	//
	// Staking
	// 

	#[pallet::storage] // ( total_stake )
	#[pallet::getter(fn total_stake)]
	pub type TotalStake<T> = StorageValue<_, u128, ValueQuery>;

	// Total stake sum of all nodes in specified subnet
	#[pallet::storage] // subnet_uid --> peer_data
	#[pallet::getter(fn total_subnet_stake)]
	pub type TotalSubnetStake<T> =
		StorageMap<_, Identity, u32, u128, ValueQuery>;

	// An accounts stake per subnet
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
		DefaultAccountTake,
	>;
	
	#[pallet::storage]
	pub type StakeUnbondingLedger<T: Config> = 
		StorageMap<_, Blake2_128Concat, T::AccountId, BTreeMap<u32, u128>, ValueQuery, DefaultStakeUnbondingLedger>;

	// Network maximum stake balance per Subnet Node
	// Only checked on `do_add_stake` and ``
	// A subnet staker can have greater than the max stake balance although any rewards
	// they would receive based on their stake balance will only account up to the max stake balance allowed
	#[pallet::storage]
	pub type NetworkMaxStakeBalance<T> = StorageValue<_, u128, ValueQuery, DefaultNetworkMaxStakeBalance>;

	// Network minimum required Subnet Node stake balance per subnet
	#[pallet::storage]
	pub type NetworkMinStakeBalance<T> = StorageValue<_, u128, ValueQuery, DefaultNetworkMinStakeBalance>;

	// The number of epochs a node must stay staked as a node from start_epoch
	#[pallet::storage]
	pub type MinActiveNodeStakeEpochs<T> = StorageValue<_, u32, ValueQuery, DefaultMinActiveNodeStakeEpochs>;

	// The maximum balance a subnet can require a node to stake to become a node in Hypertensor
	// #[pallet::storage]
	// pub type NetworkMaxStakeBalance<T> = StorageValue<_, u128, ValueQuery, DefaultNetworkMaxStakeBalance>;

	//
	// Delegate Staking
	// 

	// Minimum delegate stake balance for all subnets as a factor
	// Measured against the total network supply as a percentage
	#[pallet::storage]
	pub type MinSubnetDelegateStakeFactor<T> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetDelegateStakeFactor>;
	
	// Min delegate stake deposit amount
	// Mitigates against inflation attacks
	#[pallet::storage]
	pub type MinDelegateStakeDeposit<T> = StorageValue<_, u128, ValueQuery, DefaultMinDelegateStakeBalance>;

	#[pallet::storage] // ( total_stake )
	#[pallet::getter(fn total_delegate_stake)]
	pub type TotalDelegateStake<T> = StorageValue<_, u128, ValueQuery>;

	// Total stake sum of all nodes in specified subnet
	#[pallet::storage] // subnet_uid --> u128
	pub type TotalSubnetDelegateStakeShares<T> =
		StorageMap<_, Identity, u32, u128, ValueQuery>;

	// Total stake sum of all nodes in specified subnet
	#[pallet::storage] // subnet_uid --> u128
	pub type TotalSubnetDelegateStakeBalance<T> =
		StorageMap<_, Identity, u32, u128, ValueQuery>;

	// Exponent used to get subnet emissions based on overall network delegate stake weight 
	#[pallet::storage]
	pub type DelegateStakeWeightExponent<T> = StorageValue<_, u128, ValueQuery>;

	// An accounts delegate stake per subnet
	#[pallet::storage] // account --> subnet_id --> u128
	pub type AccountSubnetDelegateStakeShares<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Identity,
		u32,
		u128,
		ValueQuery,
		DefaultAccountTake,
	>;
	
	//
	// Node Delegate Stake
	//

	// Time between Subnet Node updating node delegate staking rate
	#[pallet::storage]
	pub type RewardRateUpdatePeriod<T> = StorageValue<_, u32, ValueQuery, DefaultRewardRateUpdatePeriod>;

	// Max nominal percentage decrease of Subnet Node delegate reward rate
	#[pallet::storage]
	pub type MaxRewardRateDecrease<T> = StorageValue<_, u128, ValueQuery, DefaultMaxRewardRateDecrease>;

	#[pallet::storage] // ( total_stake )
	#[pallet::getter(fn total_node_delegate_stake)]
	pub type TotalNodeDelegateStake<T> = StorageValue<_, u128, ValueQuery>;

	// Total stake sum of shares in specified Subnet Node
	#[pallet::storage]
	pub type TotalNodeDelegateStakeShares<T> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Identity,
		u32,
		u128,
		ValueQuery,
		DefaultAccountTake,
	>;

	// Total stake sum of balance in specified Subnet Node
	#[pallet::storage]
	pub type NodeDelegateStakeBalance<T> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Identity,
		u32,
		u128,
		ValueQuery,
		DefaultAccountTake,
	>;
	
	// account_id -> subnet_id -> subnet_node_id -> shares
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
	// Props
	//

	#[pallet::storage] // subnet => proposal_id => proposal
	pub type Proposals<T> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Identity,
		u32,
		ProposalParams,
		ValueQuery,
		DefaultProposalParams<T>,
	>;
	

	/// The minimum subnet nodes for a subnet to have to be able to use the proposal mechanism
	// Because of slashing of funds is possible, we ensure the subnet is well decentralized
	// If a subnet is under this amount, it's best to have logic in the subnet to have them absent
	// from the incentives consensus data and have them removed after the required consecutive epochs
	#[pallet::storage] 
	pub type ProposalMinSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultProposalMinSubnetNodes>;

	#[pallet::storage] 
	pub type ProposalsCount<T> = StorageValue<_, u32, ValueQuery, DefaultProposalsCount>;

	// Amount required to put up as a proposer and challenger
	#[pallet::storage] 
	pub type ProposalBidAmount<T> = StorageValue<_, u128, ValueQuery, DefaultProposalBidAmount>;

	#[pallet::storage] // Period in blocks for votes after challenge
	pub type VotingPeriod<T> = StorageValue<_, u32, ValueQuery, DefaultVotingPeriod>;

	#[pallet::storage] // Period in blocks after proposal to challenge proposal
	pub type ChallengePeriod<T> = StorageValue<_, u32, ValueQuery, DefaultChallengePeriod>;

	#[pallet::storage] // How many voters are needed in a subnet proposal
	pub type ProposalQuorum<T> = StorageValue<_, u128, ValueQuery, DefaultProposalQuorum>;

	// Consensus required to pass proposal
	#[pallet::storage]
	pub type ProposalConsensusThreshold<T> = StorageValue<_, u128, ValueQuery, DefaultProposalConsensusThreshold>;

	//
	// Weight helpers
	//
	#[pallet::storage]
	pub type DelegateStakeWeightFactor<T> = StorageValue<_, u128, ValueQuery, DefaultDelegateStakeWeightFactor>;

	// 
	// Inflation helpers elements
	//

	// // Factor of subnet utilization helper to get the overall inflation on an epoch
	// // *This works alongside Subnet Node utilization factor
	// //		*Subnet node utilization will be 1.0-SubnetInflationFactor
	// #[pallet::storage]
	// pub type SubnetInflationFactor<T> = StorageValue<_, u128, ValueQuery, DefaultSubnetInflationFactor>;

	// // Factor that is used as the pow against the utilization factors `SubnetInflationFactor` and subet node inflation factor
	// #[pallet::storage]
	// pub type InflationAdjFactor<T> = StorageValue<_, u128, ValueQuery, DefaultInflationAdjFactor>;

	// // Exponent used for subnet utilization
	// #[pallet::storage]
	// pub type SubnetInflationAdjFactor<T> = StorageValue<_, u128, ValueQuery, DefaultSubnetInflationAdjFactor>;

	// // Exponent used for Subnet Node utilization
	// #[pallet::storage]
	// pub type SubnetNodeInflationAdjFactor<T> = StorageValue<_, u128, ValueQuery, DefaultSubnetNodeInflationAdjFactor>;

	// #[pallet::storage]
	// pub type TerminalInflationFactor<T> = StorageValue<_, u128, ValueQuery, DefaultTerminalInflationFactor>;

	// /// Lower bound of inflation factor
	// #[pallet::storage]
	// pub type UtilizationLowerBound<T> = StorageValue<_, u128, ValueQuery, DefaultUtilizationLowerBound>;

	// /// Upper bound of inflation factor
	// #[pallet::storage]
	// pub type UtilizationUpperBound<T> = StorageValue<_, u128, ValueQuery, DefaultUtilizationUpperBound>;

	/// Inflation grpah midpoint (sigmoid)
	#[pallet::storage]
	pub type SigmoidMidpoint<T> = StorageValue<_, u128, ValueQuery, DefaultSigmoidMidpoint>;

	/// Inflation grpah midpoint (sigmoid)
	#[pallet::storage]
	pub type SigmoidSteepness<T> = StorageValue<_, u128, ValueQuery, DefaultSigmoidSteepness>;

	//
	// Overwatch Nodes
	//
	
	#[pallet::storage]
	pub type MaxOverwatchNodes<T: Config> = StorageValue<_, u32, ValueQuery, DefaultMaxOverwatchNodes>;

	#[pallet::storage]
	pub type TotalOverwatchNodes<T: Config> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

	#[pallet::storage]
	pub type TotalOverwatchNodeUids<T: Config> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

	// Hotkey => OverwatchNode
	#[pallet::storage]
	pub type OverwatchNodes<T: Config> = StorageMap<
		_,
		Identity,
		u32,
		OverwatchNode<T::AccountId>,
		OptionQuery
	>;

	#[pallet::storage]
	pub type OverwatchNodeIdHotkey<T: Config> = StorageMap<
		_,
		Identity,
		u32,
		T::AccountId,
		OptionQuery
	>;
	
	// Hotkey => Subnet Node ID
	#[pallet::storage]
	pub type HotkeyOverwatchNodeId<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		u32,
		OptionQuery
	>;

	#[pallet::storage] // subnet_id --> peer_id --> overwatch_node_id
	pub type PeerIdOverwatchNode<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Blake2_128Concat,
		PeerId,
		u32,
		ValueQuery,
		DefaultZeroU32,
	>;

	#[pallet::storage]
	pub type OverwatchNodeIndex<T: Config> = StorageMap<
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
		OptionQuery
	>;

	#[pallet::storage]
	pub type OverwatchReveals<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Identity, u32>, // Epoch
			NMapKey<Identity, u32>, // Subnet ID
			NMapKey<Identity, u32>, // Overwatch ID
		),
		u128, // Reveal
		OptionQuery
	>;
	
	// Epoch >>> Subnet ID
	#[pallet::storage]
	pub type OverwatchWeights<T: Config> = StorageDoubleMap<
    _, 
    Identity, 
		u32, 
    Identity, 
		u32,
    Vec<u8>, 
		OptionQuery
	>;
	
	// ow ID -> penalties count
	#[pallet::storage]
	pub type OverwatchNodePenalties<T: Config> = StorageMap<
		_,
		Identity,
		u32,
		u32,
		OptionQuery
	>;
	
	#[pallet::storage]
	pub type MaxOverwatchNodePenalties<T: Config> = StorageValue<_, u32, ValueQuery, DefaultMaxOverwatchNodePenalties>;

	/// Finalized calculated subnet weights from overwatch nodes
	#[pallet::storage]
	pub type OverwatchSubnetWeights<T: Config> = StorageMap<
		_,
		Identity,
		u32,
		u128,
		OptionQuery
	>;
	
	/// Overwatch node scores
	#[pallet::storage]
	pub type OverwatchNodeWeights<T: Config> = StorageDoubleMap<
    _, 
    Identity, 
		u32,  	// Epoch
    Identity, 
		u32, 		// Node ID
    u128, 	// Weight
		OptionQuery
	>;

	//
	// Overwatch staking
	//

	#[pallet::storage]
	pub type TotalOverwatchStake<T> = StorageValue<_, u128, ValueQuery>;

	// Overwatch hotkey stake balance
	#[pallet::storage] // subnet_uid --> peer_data
	pub type AccountOverwatchStake<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u128, ValueQuery, DefaultAccountTake>;

	#[pallet::storage]
	pub type OverwatchMinStakeBalance<T> = StorageValue<_, u128, ValueQuery, DefaultOverwatchMinStakeBalance>;
		
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
			hotkey: T::AccountId,
			subnet_data: RegistrationSubnetData<T::AccountId>,
		) -> DispatchResult {
			let owner: T::AccountId = ensure_signed(origin)?;
	
			Self::is_paused()?;

			Self::do_register_subnet(
				owner,
				hotkey,
				subnet_data,
			)
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
		pub fn activate_subnet(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin)?;
	
			Self::is_paused()?;

			ensure!(
				Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
				Error::<T>::NotSubnetOwner
			);

			Self::do_activate_subnet(subnet_id)
		}

		/// Try removing a subnet.
		///
		/// This can be useful if there is no network activity on a span of epochs
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		///
		/// # Requirements
		/// 
		/// * `SubnetPenaltyCount` surpasses `MaxSubnetPenaltyCount`
		/// * Subnet delegate stake balance is below the required balance
		/// 
		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		pub fn remove_subnet(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResult {
			ensure_signed(origin)?;

			Self::is_paused()?;

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::InvalidSubnet.into()),
			};
			
			// --- Ensure the subnet has passed it's required period to begin consensus submissions
			ensure!(
				subnet.state != SubnetState::Registered,
				Error::<T>::SubnetInitializing
			);

			let penalties = SubnetPenaltyCount::<T>::get(subnet_id);

			let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
			// let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance();
			let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance_v2(subnet_id);

			if penalties > MaxSubnetPenaltyCount::<T>::get() {
				// --- If the subnet has reached max penalty, remove it
        Self::do_remove_subnet(
          // subnet.name,
					subnet_id,
          SubnetRemovalReason::MaxPenalties,
        ).map_err(|e| e)?;
			} else if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
				// --- If the delegate stake balance is below minimum threshold, remove it
        Self::do_remove_subnet(
          // subnet.name,
					subnet_id,
          SubnetRemovalReason::MinSubnetDelegateStake,
        ).map_err(|e| e)?;
			}

			// --- If we make it to here, fail the extrinsic
			Err(Error::<T>::InvalidSubnetRemoval.into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({0})]
		pub fn owner_pause_subnet(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_pause_subnet(origin, subnet_id)
		}

		// #[pallet::call_index(3)]
		// #[pallet::weight({0})]
		// pub fn owner_unpause_subnet(
		// 	origin: OriginFor<T>, 
		// 	subnet_id: u32,
		// ) -> DispatchResult {
		// 	Self::is_paused()?;
		// 	Self::do_owner_pause_subnet(origin, subnet_id)
		// }

		#[pallet::call_index(4)]
		#[pallet::weight({0})]
		pub fn owner_deactivate_subnet(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_deactivate_subnet(origin, subnet_id)
		}

		#[pallet::call_index(5)]
		#[pallet::weight({0})]
		pub fn owner_update_name(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: Vec<u8>
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_name(origin, subnet_id, value)
		}

		#[pallet::call_index(6)]
		#[pallet::weight({0})]
		pub fn owner_update_repo(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: Vec<u8>
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_repo(origin, subnet_id, value)
		}

		#[pallet::call_index(7)]
		#[pallet::weight({0})]
		pub fn owner_update_description(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: Vec<u8>
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_description(origin, subnet_id, value)
		}

		#[pallet::call_index(8)]
		#[pallet::weight({0})]
		pub fn owner_update_misc(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: Vec<u8>
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_misc(origin, subnet_id, value)
		}

		#[pallet::call_index(9)]
		#[pallet::weight({0})]
		pub fn owner_update_churn_limit(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: u32
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_churn_limit(origin, subnet_id, value)
		}

		#[pallet::call_index(10)]
		#[pallet::weight({0})]
		pub fn owner_update_registration_queue_epochs(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: u32
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_registration_queue_epochs(origin, subnet_id, value)
		}

		#[pallet::call_index(11)]
		#[pallet::weight({0})]
		pub fn owner_update_activation_grace_epochs(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: u32
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_activation_grace_epochs(origin, subnet_id, value)
		}

		#[pallet::call_index(12)]
		#[pallet::weight({0})]
		pub fn owner_update_idle_classification_epochs(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: u32
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_idle_classification_epochs(origin, subnet_id, value)
		}

		#[pallet::call_index(13)]
		#[pallet::weight({0})]
		pub fn owner_update_included_classification_epochs(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: u32
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_included_classification_epochs(origin, subnet_id, value)
		}

		#[pallet::call_index(14)]
		#[pallet::weight({0})]
		pub fn owner_update_max_node_penalties(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: u32
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_max_node_penalties(origin, subnet_id, value)
		}

		#[pallet::call_index(15)]
		#[pallet::weight({0})]
		pub fn owner_add_initial_coldkeys(
			origin: OriginFor<T>, 
			subnet_id: u32,
			coldkeys: BTreeSet<T::AccountId>
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_add_initial_coldkeys(origin, subnet_id, coldkeys)
		}

		#[pallet::call_index(16)]
		#[pallet::weight({0})]
		pub fn owner_remove_initial_coldkeys(
			origin: OriginFor<T>, 
			subnet_id: u32,
			coldkeys: BTreeSet<T::AccountId>
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_remove_initial_coldkeys(origin, subnet_id, coldkeys)
		}

		#[pallet::call_index(17)]
		#[pallet::weight({0})]
		pub fn owner_remove_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_remove_subnet_node(origin, subnet_id, subnet_node_id)
		}

		#[pallet::call_index(18)]
		#[pallet::weight({0})]
		pub fn owner_update_min_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: u128
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_min_stake(origin, subnet_id, value)
		}

		#[pallet::call_index(19)]
		#[pallet::weight({0})]
		pub fn owner_update_max_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: u128
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_max_stake(origin, subnet_id, value)
		}

		#[pallet::call_index(20)]
		#[pallet::weight({0})]
		pub fn owner_update_delegate_stake_percentage(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: u128
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_owner_update_delegate_stake_percentage(origin, subnet_id, value)
		}

		#[pallet::call_index(21)]
		#[pallet::weight({0})]
		pub fn owner_update_max_registered_nodes(
			origin: OriginFor<T>, 
			subnet_id: u32,
			value: u32
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
		pub fn accept_subnet_ownership(
			origin: OriginFor<T>, 
			subnet_id: u32, 
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_accept_subnet_ownership(origin, subnet_id)
		}

		/// Add a Subnet Node to the subnet by registering and activating in one call
		///
		/// The Subnet Node will be assigned a class (`SubnetNodeClass`)
		/// * If the subnet is in its registration period it will be assigned the Validator class
		/// * If the subnet is active, it will be assigned as `Registered` and must be inducted by consensus
		/// - See `SubnetNodeClass` for more information on class levels
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `hotkey` - Hotkey of the Subnet Node.
		/// * `peer_id` - The Peer ID of the Subnet Node within the subnet P2P network.
		/// * `stake_to_be_added` - The balance to add to stake.
		/// * `a` - A Subnet Node parameter unique to each subnet.
		/// * `b` - A non-unique parameter.
		/// * `c` - A non-unique parameter.
		///
		/// # Requirements
		/// 
		/// * `stake_to_be_added` must be the minimum required stake balance
		/// 
		#[pallet::call_index(24)]
		// #[pallet::weight(T::WeightInfo::add_subnet_node())]
		#[pallet::weight({0})]
		pub fn add_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			hotkey: T::AccountId,
			peer_id: PeerId, 
			bootstrap_peer_id: PeerId,
			delegate_reward_rate: u128,
			stake_to_be_added: u128,
			a: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
			b: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
			c: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
		) -> DispatchResult {
			Self::is_paused()?;

			Self::do_register_subnet_node(
				origin.clone(),
				subnet_id,
				hotkey.clone(),
				peer_id,
				bootstrap_peer_id,
				// client_peer_id,
				delegate_reward_rate,
				stake_to_be_added,
				a,
				b,
				c,
			).map_err(|e| e)?;

			let subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone())
				.ok_or(Error::<T>::NotUidOwner)?;

			Self::do_activate_subnet_node(
				origin,
				subnet_id,
				subnet_node_id,
			).map(|_| ()).map_err(|e| e.error)
		}

		/// Register a Subnet Node to the subnet
		///
		/// A registered Subnet Node will not be included in consensus data, therefor no incentives until
		/// the Subnet Node activates itself (see `activate_subnet_node`)
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
		/// * `c` - A non-unique parameter.
		///
		/// # Requirements
		/// 
		/// * `stake_to_be_added` must be the minimum required stake balance
		/// 
		#[pallet::call_index(25)]
		#[pallet::weight({0})]
		pub fn register_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			hotkey: T::AccountId,
			peer_id: PeerId, 
			bootstrap_peer_id: PeerId,
			delegate_reward_rate: u128,
			stake_to_be_added: u128,
			a: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
			b: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
			c: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
		) -> DispatchResult {
			Self::is_paused()?;

			Self::do_register_subnet_node(
				origin,
				subnet_id,
				hotkey,
				peer_id,
				bootstrap_peer_id,
				delegate_reward_rate,
				stake_to_be_added,
				a,
				b,
				c,
			)
		}

		/// Activate a Subnet Node
		///
		/// Subnet nodes should activate their Subnet Node once they are in the subnet and have completed any
		/// steps the subnet requires, such as any consensus mechanisms.
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Subnet node ID assigned during registration
		/// 
		#[pallet::call_index(26)]
		#[pallet::weight({0})]
		pub fn activate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResultWithPostInfo {
			// Self::is_paused()?;
			Self::is_paused().map_err(|e| e)?;

			Self::do_activate_subnet_node(
				origin,
				subnet_id,
				subnet_node_id
			)
		}	

		/// Deactivate a Subnet Node temporarily
		///
		/// A Subnet Node can deactivate themselves temporarily up to the MaxDeactivationEpochs
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Subnet node ID assigned during registration
		///
		/// # Requirements
		/// 
		/// * Must be a Validator class to deactivate, otherwise the Subnet Node must
		/// 
		#[pallet::call_index(27)]
		#[pallet::weight({0})]
		pub fn deactivate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResult {
			Self::is_paused()?;

			Self::do_deactivate_subnet_node_new(
				origin,
				subnet_id,
				subnet_node_id
			)
		}
		
		/// Reactivate Subnet Node that is currently deactivated
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
		#[pallet::call_index(28)]
		#[pallet::weight({0})]
		pub fn reactivate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResult {
			Self::is_paused()?;

			Self::do_reactivate_subnet_node(origin, subnet_id, subnet_node_id)
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
		#[pallet::call_index(29)]
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
				Self::is_subnet_node_keys_owner(
					subnet_id, 
					subnet_node_id, 
					key, 
				),
				Error::<T>::NotKeyOwner
			);

			Self::do_remove_subnet_node(subnet_id, subnet_node_id)
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
		#[pallet::call_index(30)]
		#[pallet::weight({0})]
		pub fn register_identity(
			origin: OriginFor<T>, 
			hotkey: T::AccountId,
			name: BoundedVec<u8, DefaultMaxUrlLength>,
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

			Self::do_register_identity(
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

		#[pallet::call_index(31)]
		#[pallet::weight({0})]
		pub fn transfer_identity(origin: OriginFor<T>, new_owner: T::AccountId) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin)?;

			let coldkey_identity = ColdkeyIdentity::<T>::get(&coldkey);
			// PendingIdentityOwner::<T>::insert(coldkey_identity, new_owner);

			Ok(())
		}

		#[pallet::call_index(32)]
		#[pallet::weight({0})]
		pub fn accept_identity(origin: OriginFor<T>, identity: Vec<u8>) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin)?;

			// let pending_owner: T::AccountId = match PendingIdentityOwner::<T>::try_get(&identity) {
			// 	Ok(pending_owner) => pending_owner,
			// 	Err(()) => return Err(Error::<T>::NoPendingIdentityOwner.into()),
			// };

			// ensure!(
			// 	coldkey == pending_owner,
			// 	Error::<T>::NotPendingIdentityOwner
			// );

			// --- Update coldkeys identity
			// ColdkeyIdentity::<T>::insert(&coldkey, &identity);
			// // --- Update identities coldkey
			// IdentityColdkey::<T>::insert(&identity, &coldkey);
			// // --- Remove pending state
			// PendingIdentityOwner::<T>::remove(identity);

			Ok(())
		}

		#[pallet::call_index(33)]
		#[pallet::weight({0})]
		pub fn clean_expired_registered_subnet_nodes(
			origin: OriginFor<T>, 
			subnet_id: u32
		) -> DispatchResult {
			ensure_signed(origin.clone())?;
			
			// let epoch = Self::get_current_epoch_as_u32();
			let epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);

			if subnet_id == 0 {
				for (subnet_id, subnet_node_id, subnet_node) in RegisteredSubnetNodesData::<T>::iter() {
					let grace_epochs = ActivationGraceEpochs::<T>::get(subnet_id);
					if epoch > subnet_node.classification.start_epoch + grace_epochs {
						RegisteredSubnetNodesData::<T>::remove(subnet_id, subnet_node_id);
					}
				}
			} else {
				let grace_epochs = ActivationGraceEpochs::<T>::get(subnet_id);
				for (subnet_node_id, subnet_node) in RegisteredSubnetNodesData::<T>::iter_prefix(subnet_id) {
					if epoch > subnet_node.classification.start_epoch + grace_epochs {
						RegisteredSubnetNodesData::<T>::remove(subnet_id, subnet_node_id);
					}
				}
			}

			Ok(())
		}

		#[pallet::call_index(34)]
		#[pallet::weight({0})]
		pub fn clean_expired_deactivated_subnet_nodes(
			origin: OriginFor<T>, 
			subnet_id: u32
		) -> DispatchResult {
			ensure_signed(origin.clone())?;
			
			// let epoch = Self::get_current_epoch_as_u32();
			let epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);

			if subnet_id == 0 {
				for (subnet_id, subnet_node_id, subnet_node) in DeactivatedSubnetNodesData::<T>::iter() {
					let max_deactivation_epochs = MaxDeactivationEpochs::<T>::get(subnet_id);
					if epoch > subnet_node.classification.start_epoch + max_deactivation_epochs {
						DeactivatedSubnetNodesData::<T>::remove(subnet_id, subnet_node_id);
					}
				}
			} else {
				let max_deactivation_epochs = MaxDeactivationEpochs::<T>::get(subnet_id);
				for (subnet_node_id, subnet_node) in DeactivatedSubnetNodesData::<T>::iter_prefix(subnet_id) {
					if epoch > subnet_node.classification.start_epoch + max_deactivation_epochs {
						DeactivatedSubnetNodesData::<T>::remove(subnet_id, subnet_node_id);
					}
				}
			}

			Ok(())
		}

		// #[pallet::call_index(35)]
		// #[pallet::weight({0})]
		// pub fn clean_removed_subnets(
		// 	origin: OriginFor<T>, 
		// 	max_cleans: u32
		// ) -> DispatchResult {
		// 	ensure_signed(origin.clone())?;
			
		// 	let epoch = Self::get_current_epoch_as_u32();

		// 	if subnet_id == 0 {
		// 		for (subnet_id, subnet_node_id, subnet_node) in DeactivatedSubnetNodesData::<T>::iter() {
		// 			let max_deactivation_epochs = MaxDeactivationEpochs::<T>::get(subnet_id);
		// 			if epoch > subnet_node.classification.start_epoch + max_deactivation_epochs {
		// 				DeactivatedSubnetNodesData::<T>::remove(subnet_id, subnet_node_id);
		// 			}
		// 		}
		// 	} else {
		// 		let max_deactivation_epochs = MaxDeactivationEpochs::<T>::get(subnet_id);
		// 		for (subnet_node_id, subnet_node) in DeactivatedSubnetNodesData::<T>::iter_prefix(subnet_id) {
		// 			if epoch > subnet_node.classification.start_epoch + max_deactivation_epochs {
		// 				DeactivatedSubnetNodesData::<T>::remove(subnet_id, subnet_node_id);
		// 			}
		// 		}
		// 	}

		// 	Ok(())
		// }

		/// Remove Subnet Node of caller
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
		#[pallet::call_index(35)]
		#[pallet::weight({0})]
		pub fn update_delegate_reward_rate(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			new_delegate_reward_rate: u128
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin)?;

			Self::is_paused()?;

			ensure!(
				Self::is_subnet_node_coldkey(
					subnet_id, 
					subnet_node_id, 
					coldkey, 
				),
				Error::<T>::NotKeyOwner
			);

			// --- Ensure rate doesn't surpass 100%
			ensure!(
				new_delegate_reward_rate <= Self::percentage_factor_as_u128(),
				Error::<T>::InvalidDelegateRewardRate
			);

			let block: u32 = Self::get_current_block_as_u32();
			let max_reward_rate_decrease = MaxRewardRateDecrease::<T>::get();
			let reward_rate_update_period = RewardRateUpdatePeriod::<T>::get();

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetNodeId)?;
					let mut curr_delegate_reward_rate = params.delegate_reward_rate;

					// --- Ensure rate change surpasses minimum update period
					ensure!(
						block - params.last_delegate_reward_rate_update >= reward_rate_update_period,
						Error::<T>::MaxRewardRateUpdates
					);
					
					// --- Ensure rate is being updated redundantly
					ensure!(
						new_delegate_reward_rate != curr_delegate_reward_rate,
						Error::<T>::NoDelegateRewardRateChange
					);

					let mut delegate_reward_rate = params.delegate_reward_rate;

					if new_delegate_reward_rate > curr_delegate_reward_rate {
						// Freely increase reward rate
						delegate_reward_rate = new_delegate_reward_rate;
					} else {
						// Ensure reward rate decrease doesn't surpass max rate of change
						let delta = curr_delegate_reward_rate - new_delegate_reward_rate;
						ensure!(
							delta <= max_reward_rate_decrease,
							Error::<T>::SurpassesMaxRewardRateDecrease
						);
						delegate_reward_rate = new_delegate_reward_rate
					}

					params.last_delegate_reward_rate_update = block;
					params.delegate_reward_rate = delegate_reward_rate;
					Ok(())
				}
			)?;

			Ok(())
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
		#[pallet::call_index(36)]
		// #[pallet::weight(T::WeightInfo::add_to_stake())]
		#[pallet::weight({0})]
		pub fn add_to_stake(
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
				Error::<T>::InvalidSubnet
			);

			// --- Ensure coldkey owns the hotkey
			ensure!(
				HotkeyOwner::<T>::get(&hotkey) == coldkey,
				Error::<T>::NotKeyOwner
			);

			// --- Ensure hotkey owns the subnet_node_id
			// ensure!(
			// 	Self::is_subnet_node_owner(subnet_id, subnet_node_id, hotkey.clone()),
			// 	Error::<T>::NotSubnetNodeOwner
			// );
						
			Self::do_add_stake(
				origin, 
				subnet_id,
				hotkey,
				stake_to_be_added,
			)
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
		#[pallet::call_index(37)]
		#[pallet::weight({0})]
		pub fn remove_stake(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			hotkey: T::AccountId,
			stake_to_be_removed: u128
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
			let is_subnet_node: bool = match HotkeySubnetNodeId::<T>::try_get(subnet_id, &hotkey) {
				Ok(subnet_node_id) => {
					let subnet_epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);
					let is_validator: bool = Self::is_validator(subnet_id, subnet_node_id, subnet_epoch);

					// --- Check if current epochs validator, can't unstake if so
					ensure!(
						!is_validator,
						Error::<T>::ElectedValidatorCannotUnstake
					);

					if let Some(subnet_node) = Self::get_activated_subnet_node(subnet_id, subnet_node_id) {
						let min_stake_epochs = MinActiveNodeStakeEpochs::<T>::get();
						// --- Ensure activated nodes minimum stake epochs are complete to remove any balances
						ensure!(
							subnet_node.classification.start_epoch + min_stake_epochs < subnet_epoch,
							Error::<T>::MinActiveNodeStakeEpochs
						);
					}
					true
				},
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
		#[pallet::call_index(38)]
		#[pallet::weight({0})]
		pub fn claim_unbondings(
			origin: OriginFor<T>, 
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin)?;

			Self::is_paused()?;

			let successful_unbondings: u32 = Self::do_claim_unbondings(&coldkey);

			// Give error if there is no unbondings
			// TODO: Give more info
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
		#[pallet::call_index(39)]
		// #[pallet::weight(T::WeightInfo::add_to_delegate_stake())]
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
				Error::<T>::InvalidSubnet
			);

			Self::do_add_delegate_stake(
				origin, 
				subnet_id,
				stake_to_be_added,
			)
		}
		
		/// Increase subnet delegate stake
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
		#[pallet::call_index(40)]
		#[pallet::weight({0})]
		pub fn swap_delegate_stake(
			origin: OriginFor<T>, 
			from_subnet_id: u32, 
			to_subnet_id: u32, 
			delegate_stake_shares_to_swap: u128
		) -> DispatchResult {
			Self::is_paused()?;

			// --- Ensure ``to`` subnet exists
			ensure!(
				SubnetsData::<T>::contains_key(to_subnet_id),
				Error::<T>::InvalidSubnet
			);

			// TODO: Ensure users aren't hopping from one subnet to another to get both rewards
			// Check if both subnets have generated rewards
			// --- Only allow one ``swap_delegate_stake`` per epoch or every other epoch

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
		#[pallet::call_index(41)]
		#[pallet::weight({0})]
		pub fn transfer_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			to_account_id: T::AccountId,
			delegate_stake_shares_to_transfer: u128
		) -> DispatchResult {
			Self::is_paused()?;

			// TODO: Ensure users aren't hopping from one subnet to another to get both rewards
			// Check if both subnets have generated rewards
			// --- Only allow one ``swap_delegate_stake`` per epoch or every other epoch

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
		#[pallet::call_index(42)]
		// #[pallet::weight(T::WeightInfo::remove_delegate_stake())]
		#[pallet::weight({0})]
		pub fn remove_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			shares_to_be_removed: u128
		) -> DispatchResult {
			Self::is_paused()?;

			Self::do_remove_delegate_stake(
				origin, 
				subnet_id,
				shares_to_be_removed,
			)
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
		/// TODO: Change name of function to avoid delegate staking confusions
		#[pallet::call_index(43)]
		#[pallet::weight({0})]
		pub fn increase_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			amount: u128,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;

			Self::is_paused()?;

			// --- Ensure subnet exists, otherwise at risk of burning tokens
			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::InvalidSubnet
			);

			ensure!(
				amount >= MinDelegateStakeDeposit::<T>::get(),
				Error::<T>::MinDelegateStake
			);

			let amount_as_balance = Self::u128_to_balance(amount);

			ensure!(
				amount_as_balance.is_some(),
				Error::<T>::CouldNotConvertToBalance
			);
	
			// --- Ensure the callers account_id has enough balance to perform the transaction.
			ensure!(
				Self::can_remove_balance_from_coldkey_account(&account_id, amount_as_balance.unwrap()),
				Error::<T>::NotEnoughBalance
			);
	
			// --- Ensure the remove operation from the account_id is a success.
			ensure!(
				Self::remove_balance_from_coldkey_account(&account_id, amount_as_balance.unwrap()) == true,
				Error::<T>::BalanceWithdrawalError
			);
			
			Self::do_increase_delegate_stake(
				subnet_id,
				amount,
			);

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
		#[pallet::call_index(44)]
		#[pallet::weight({0})]
		pub fn add_to_node_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			node_delegate_stake_to_be_added: u128
		) -> DispatchResult {
			Self::is_paused()?;

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
		#[pallet::call_index(45)]
		#[pallet::weight({0})]
		pub fn swap_node_delegate_stake(
			origin: OriginFor<T>, 
			from_subnet_id: u32,
			from_subnet_node_id: u32, 
			to_subnet_id: u32, 
			to_subnet_node_id: u32, 
			node_delegate_stake_shares_to_swap: u128
		) -> DispatchResult {
			Self::is_paused()?;

			// --- Ensure ``to`` Subnet Node exists
			ensure!(
				SubnetNodesData::<T>::contains_key(to_subnet_id, to_subnet_node_id),
				Error::<T>::SubnetNodeNotExist
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
		#[pallet::call_index(46)]
		#[pallet::weight({0})]
		pub fn transfer_node_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32, 
			to_account_id: T::AccountId, 
			node_delegate_stake_shares_to_transfer: u128
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
		#[pallet::call_index(47)]
		#[pallet::weight({0})]
		pub fn remove_node_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			node_delegate_stake_shares_to_be_removed: u128
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
		#[pallet::call_index(48)]
		#[pallet::weight({0})]
		pub fn increase_node_delegate_stake(
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
				Error::<T>::SubnetNodeNotExist
			);
			
			ensure!(
				amount >= MinDelegateStakeDeposit::<T>::get(),
				Error::<T>::MinDelegateStake
			);

			let amount_as_balance = Self::u128_to_balance(amount);

			ensure!(
				amount_as_balance.is_some(),
				Error::<T>::CouldNotConvertToBalance
			);
	
			// --- Ensure the callers account_id has enough balance to perform the transaction.
			ensure!(
				Self::can_remove_balance_from_coldkey_account(&account_id, amount_as_balance.unwrap()),
				Error::<T>::NotEnoughBalance
			);
	
			// --- Ensure the remove operation from the account_id is a success.
			ensure!(
				Self::remove_balance_from_coldkey_account(&account_id, amount_as_balance.unwrap()) == true,
				Error::<T>::BalanceWithdrawalError
			);
			
			Self::do_increase_node_delegate_stake(
				subnet_id,
				subnet_node_id,
				amount,
			);

			Ok(())
		}

		/// Transfer stake from a Subnet Node to a subnet
		///
		/// # Arguments
		///
		/// * `from_subnet_id` - From subnet ID to remove delegate stake from.
		/// * `from_subnet_node_id` - From Subnet Node ID to remove delegate stake from.
		/// * `to_subnet_id` - To subnet ID to add delegate stake to
		/// * `node_delegate_stake_shares_to_swap` - Shares to remove from delegate pool and add balance to subnet
		///
		#[pallet::call_index(49)]
		#[pallet::weight({0})]
		pub fn transfer_from_node_to_subnet(
			origin: OriginFor<T>, 
			from_subnet_id: u32,
			from_subnet_node_id: u32,
			to_subnet_id: u32,
			node_delegate_stake_shares_to_swap: u128,
		) -> DispatchResult {
			Self::is_paused()?;

			Self::do_transfer_from_node_to_subnet(
				origin,
				from_subnet_id,
				from_subnet_node_id,
				to_subnet_id,
				node_delegate_stake_shares_to_swap,
			)		
		}

		/// Transfer stake from a subnet to a Subnet Node
		///
		/// # Arguments
		///
		/// * `from_subnet_id` - From subnet ID to remove delegate stake from.
		/// * `to_subnet_id` - To subnet ID to add delegate stake to.
		/// * `to_subnet_node_id` - To Subnet Node ID to add delegate stake to
		/// * `delegate_stake_shares_to_swap` - Shares to remove from delegate pool and add balance to node
		///
		#[pallet::call_index(50)]
		#[pallet::weight({0})]
		pub fn transfer_from_subnet_to_node(
			origin: OriginFor<T>, 
			from_subnet_id: u32,
			to_subnet_id: u32,
			to_subnet_node_id: u32,
			delegate_stake_shares_to_swap: u128,
		) -> DispatchResult {
			Self::is_paused()?;

			Self::do_transfer_from_subnet_to_node(
				origin,
				from_subnet_id,
				to_subnet_id,
				to_subnet_node_id,
				delegate_stake_shares_to_swap,
			)
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
		#[pallet::call_index(51)]
		#[pallet::weight({0})]
		pub fn validate(
			origin: OriginFor<T>, 
			subnet_id: u32,
			data: Vec<SubnetNodeConsensusData>,
			args: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
		) -> DispatchResultWithPostInfo {
			let hotkey: T::AccountId = ensure_signed(origin)?;

			Self::is_paused()?;

			Self::do_validate(
				subnet_id, 
				hotkey,
				data,
				args,
			)
		}

		/// Attest validators view of the subnet
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID to increase delegate pool balance of.
		/// 
		#[pallet::call_index(52)]
		#[pallet::weight({0})]
		pub fn attest(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResultWithPostInfo {
			let hotkey: T::AccountId = ensure_signed(origin)?;

			Self::is_paused()?;

			Self::do_attest(
				subnet_id, 
				hotkey,
			)
		}

		/// Register unique Subnet Node parameter if not already added
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Callers Subnet Node ID
		/// * `a` - The unique parameter
		/// 
		#[pallet::call_index(53)]
		#[pallet::weight({0})]
		pub fn register_subnet_node_a_parameter(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			a: BoundedVec<u8, DefaultMaxVectorLength>,
		) -> DispatchResult {
			Self::is_paused()?;

			let key: T::AccountId = ensure_signed(origin)?;

			ensure!(
				Self::is_subnet_node_keys_owner(
					subnet_id, 
					subnet_node_id, 
					key, 
				),
				Error::<T>::NotKeyOwner
			);

			ensure!(
				!SubnetNodeUniqueParam::<T>::contains_key(subnet_id, &a),
				Error::<T>::SubnetNodeUniqueParamTaken
			);

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetNodeId)?;
					ensure!(
						params.a.is_none(),
						Error::<T>::SubnetNodeUniqueParamIsSet
					);
					SubnetNodeUniqueParam::<T>::insert(subnet_id, &a, &params.peer_id);
					params.a = Some(a);
					Ok(())
				}
			)
		}

		/// Register non-unique Subnet Node parameter `b` or `c`
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Callers Subnet Node ID
		/// * `b` (Optional) - The non-unique parameter
		/// * `c` (Optional) - The non-unique parameter
		/// 
		#[pallet::call_index(54)]
		#[pallet::weight({0})]
		pub fn set_subnet_node_non_unique_parameter(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			b: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
			c: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
		) -> DispatchResult {
			let key: T::AccountId = ensure_signed(origin)?;

			Self::is_paused()?;

			ensure!(
				Self::is_subnet_node_keys_owner(
					subnet_id, 
					subnet_node_id, 
					key, 
				),
				Error::<T>::NotKeyOwner
			);

			let epoch: u32 = Self::get_current_epoch_as_u32();

			let last_update_epoch = SubnetNodeNonUniqueParamLastSet::<T>::get(subnet_id, subnet_node_id);
			let interval = SubnetNodeNonUniqueParamUpdateInterval::<T>::get();

			ensure!(
				last_update_epoch.saturating_add(interval) <= epoch,
				Error::<T>::SubnetNodeNonUniqueParamUpdateIntervalNotReached
			);

			ensure!(
				b.is_some() || c.is_some(),
				Error::<T>::SubnetNodeNonUniqueParamMustBeSome
			);

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetNodeId)?;

					if b.is_some() {
						params.b = Some(b.clone().unwrap());
					}

					if c.is_some() {
						params.c = Some(c.clone().unwrap());
					}

					SubnetNodeNonUniqueParamLastSet::<T>::insert(subnet_id, subnet_node_id, epoch);

					Ok(())
				}
			)
		}

		/// Update coldkey
		///
		/// # Arguments
		///
		/// * `hotkey` - Current hotkey.
		/// * `new_coldkey` - New coldkey
		/// * `subnet_id` - Optional parameter used for subnet owners
		/// 
		#[pallet::call_index(55)]
		#[pallet::weight({0})]
		pub fn update_coldkey(
			origin: OriginFor<T>,
			hotkey: T::AccountId,
			new_coldkey: T::AccountId,
		) -> DispatchResult {
			let curr_coldkey: T::AccountId = ensure_signed(origin)?;

			Self::is_paused()?;

			ensure!(
				&hotkey != &new_coldkey,
				Error::<T>::ColdkeyMatchesHotkey
			);

			HotkeyOwner::<T>::try_mutate_exists(hotkey, |maybe_coldkey| -> DispatchResult {
        match maybe_coldkey {
					Some(coldkey) if *coldkey == curr_coldkey => {
						// Condition met, update or remove
						*maybe_coldkey = Some(new_coldkey.clone());
						// Update StakeUnbondingLedger
						StakeUnbondingLedger::<T>::swap(&curr_coldkey, &new_coldkey);

						// Update coldkeys list of hotkeys
						ColdkeyHotkeys::<T>::swap(&curr_coldkey, &new_coldkey);
						
						// Identity is not required so we ensure it exists first
						match ColdkeyIdentity::<T>::try_get(&curr_coldkey) {
							Ok(identity) => {
								ColdkeyIdentity::<T>::swap(&curr_coldkey, &new_coldkey);
							},
							// Has no identity, pass
							Err(()) => (),
						};

						ColdkeyReputation::<T>::swap(&curr_coldkey, &new_coldkey);
						ColdkeySubnets::<T>::swap(&curr_coldkey, &new_coldkey);

						Ok(())
					},
					// --- Revert from here if not exist
					Some(_) => {
						Err(Error::<T>::NotKeyOwner.into())
					},
					None => {
						Err(Error::<T>::NotKeyOwner.into())
					}
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
		/// # Arguments
		///
		/// * `old_hotkey` - Old hotkey to be replaced.
		/// * `new_hotkey` - New hotkey to replace the old hotkey.
		/// 
		#[pallet::call_index(56)]
		#[pallet::weight({0})]
		pub fn update_hotkey(
			origin: OriginFor<T>, 
			old_hotkey: T::AccountId,
			new_hotkey: T::AccountId,
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin.clone())?;

			Self::is_paused()?;
			// TODO: Make a pause function unique to subnet_id's

			ensure!(
				&coldkey != &new_hotkey,
				Error::<T>::ColdkeyMatchesHotkey
			);

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

			HotkeyOwner::<T>::remove(&old_hotkey);
			HotkeyOwner::<T>::insert(&new_hotkey, &coldkey);

			// --- Update overwatch node hotkey
			if let Some(overwatch_node_id) = HotkeyOverwatchNodeId::<T>::take(&old_hotkey) {
				OverwatchNodeIdHotkey::<T>::insert(overwatch_node_id, &new_hotkey);
				HotkeyOverwatchNodeId::<T>::insert(&new_hotkey, overwatch_node_id);
				OverwatchNodes::<T>::try_mutate_exists(
					overwatch_node_id,
					|maybe_params| -> DispatchResult {
						let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetNodeId)?;
						params.hotkey = new_hotkey.clone();
						Ok(())
					}
				);
			};

			// --- Update overwatch node stake (outside of above `if` incase removed but has balance)
			let account_overwatch_stake: u128 = AccountOverwatchStake::<T>::get(&old_hotkey);
			if account_overwatch_stake != 0 {
				Self::do_swap_overwatch_hotkey_balance(
					&old_hotkey, 
					&new_hotkey, 
				);
			}

			// Iterate each subnet and update node and stake balance
			for (subnet_id, _) in SubnetsData::<T>::iter() {
				let subnet_node_owner: (bool, u32) = match HotkeySubnetNodeId::<T>::try_get(subnet_id, &old_hotkey) {
					Ok(subnet_node_id) => (true, subnet_node_id),
					Err(()) => (false, 0),
				};

				// --- Swap hotkey node IDs and storage elements
				if subnet_node_owner.0 {
					// --- Update nodes hotkey
					SubnetNodeIdHotkey::<T>::insert(subnet_id, subnet_node_owner.1, &new_hotkey);
					// --- Update Subnet Nodes Data's hotkey (Active, Registered, or Deactivated)
					Self::update_subnet_node_hotkey(subnet_id, subnet_node_owner.1, new_hotkey.clone());
					// Swap hotkeys -> node_id
					HotkeySubnetNodeId::<T>::swap(subnet_id, &old_hotkey, subnet_id, &new_hotkey);
				}
			}

			// --- Swap stake balance
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

			Ok(())
		}

		#[pallet::call_index(57)]
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
				Self::is_subnet_node_coldkey(
					subnet_id, 
					subnet_node_id, 
					coldkey, 
				),
				Error::<T>::NotKeyOwner
			);

			ensure!(
				Self::validate_peer_id(&new_peer_id),
				Error::<T>::InvalidPeerId
			);

			ensure!(
				Self::is_owner_of_peer_or_ownerless(subnet_id, 0, 0, &new_peer_id),
				Error::<T>::PeerIdExist
			);

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetNodeId)?;

					PeerIdSubnetNodeId::<T>::remove(subnet_id, &params.peer_id);
					PeerIdSubnetNodeId::<T>::insert(subnet_id, &new_peer_id, subnet_node_id);

					params.peer_id = new_peer_id;
					Ok(())
				}
			)?;

			// TODO: Must update Rewards Submissions to use SN-UID instead of PeerId to allow updating PeerId


			Ok(())
		}

		#[pallet::call_index(58)]
		#[pallet::weight({0})]
		pub fn update_bootstrap_peer_id(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			new_bootstrap_peer_id: PeerId,
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin.clone())?;

			Self::is_paused()?;

			ensure!(
				Self::is_subnet_node_coldkey(
					subnet_id, 
					subnet_node_id, 
					coldkey, 
				),
				Error::<T>::NotKeyOwner
			);

			ensure!(
				Self::validate_peer_id(&new_bootstrap_peer_id),
				Error::<T>::InvalidBootstrapPeerId
			);

			ensure!(
				Self::is_owner_of_peer_or_ownerless(subnet_id, 0, 0, &new_bootstrap_peer_id),
				Error::<T>::BootstrapPeerIdExist
			);

			// Subnet node PeerIds and bootstrap PeerIds can match only if they are under the same Subnet Node ID
			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetNodeId)?;

					BootstrapPeerIdSubnetNodeId::<T>::remove(subnet_id, &params.bootstrap_peer_id);
					BootstrapPeerIdSubnetNodeId::<T>::insert(subnet_id, &new_bootstrap_peer_id, subnet_node_id);
		
					params.bootstrap_peer_id = new_bootstrap_peer_id;
					Ok(())
				}
			)?;

			Ok(())
		}

		#[pallet::call_index(59)]
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
				Self::is_subnet_node_coldkey(
					subnet_id, 
					subnet_node_id, 
					coldkey, 
				),
				Error::<T>::NotKeyOwner
			);

			ensure!(
				Self::validate_peer_id(&new_client_peer_id),
				Error::<T>::InvalidClientPeerId
			);

			ensure!(
				Self::is_owner_of_peer_or_ownerless(subnet_id, subnet_node_id, 0, &new_client_peer_id),
				Error::<T>::ClientPeerIdExist
			);

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetNodeId)?;

					ClientPeerIdSubnetNode::<T>::remove(subnet_id, &params.client_peer_id);
					ClientPeerIdSubnetNode::<T>::insert(subnet_id, &new_client_peer_id, subnet_node_id);
		
					params.client_peer_id = new_client_peer_id;
					Ok(())
				}
			)?;

			Ok(())
		}

		#[pallet::call_index(60)]
		#[pallet::weight({0})]
		pub fn register_overwatch_node(
			origin: OriginFor<T>,
			hotkey: T::AccountId,
			stake_to_be_added: u128,
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_register_ow(
				origin,
				hotkey,
				stake_to_be_added,
			)
		}

		#[pallet::call_index(61)]
		#[pallet::weight({0})]
		pub fn remove_overwatch_node(
			origin: OriginFor<T>,
			overwatch_node_id: u32,
			stake_to_be_added: u128,
		) -> DispatchResult {
			let key: T::AccountId = ensure_signed(origin.clone())?;

			Self::is_paused()?;

			ensure!(
				Self::is_overwatch_node_keys_owner(
					overwatch_node_id, 
					key, 
				),
				Error::<T>::NotKeyOwner
			);

			Ok(())
		}

		#[pallet::call_index(62)]
		#[pallet::weight({0})]
		pub fn set_overwatch_peer_id(
			origin: OriginFor<T>,
			subnet_id: u32,
			overwatch_node_id: u32,
			peer_id: PeerId
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_set_ow_peer_id(
				origin,
				subnet_id,
				overwatch_node_id,
				peer_id
			)
		}

		#[pallet::call_index(63)]
		#[pallet::weight({0})]
		pub fn commit_ow_weights(
			origin: OriginFor<T>,
			overwatch_node_id: u32,
			mut commit_weights: Vec<OverwatchCommit<T::Hash>>,
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_commit_ow_weights(
				origin,
				overwatch_node_id,
				commit_weights,
			)
		}

		#[pallet::call_index(64)]
		#[pallet::weight({0})]
		pub fn reveal_ow_weights(
			origin: OriginFor<T>,
			overwatch_node_id: u32,
			reveals: Vec<OverwatchReveal>,
		) -> DispatchResult {
			Self::is_paused()?;
			Self::do_reveal_ow_weights(
				origin,
				overwatch_node_id,
				reveals,
			)
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
		#[pallet::call_index(65)]
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
				Error::<T>::InvalidSubnet
			);

			// --- Ensure coldkey owns the hotkey
			ensure!(
				HotkeyOwner::<T>::get(&hotkey) == coldkey,
				Error::<T>::NotKeyOwner
			);

			Self::do_add_overwatch_stake(
				coldkey,
				hotkey,
				stake_to_be_added,
			)
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
		#[pallet::call_index(66)]
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
		#[pallet::call_index(67)]
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
		#[pallet::call_index(68)]
		#[pallet::weight({0})]
		pub fn unpause(origin: OriginFor<T>) -> DispatchResult {
			T::MajorityCollectiveOrigin::ensure_origin(origin)?;
			Self::do_unpause()
		}

		#[pallet::call_index(69)]
		#[pallet::weight({0})]
		pub fn collective_remove_subnet(
			origin: OriginFor<T>,
			subnet_id: u32
		) -> DispatchResult {
			T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
			Self::do_collective_remove_subnet(subnet_id)
		}

		#[pallet::call_index(70)]
		#[pallet::weight({0})]
		pub fn collective_remove_subnet_node(
			origin: OriginFor<T>,
			subnet_id: u32,
			subnet_node_id: u32
		) -> DispatchResult {
			T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
			Self::do_collective_remove_subnet_node(subnet_id, subnet_node_id)
		}

		#[pallet::call_index(71)]
		#[pallet::weight({0})]
		pub fn collective_remove_overwatch_node(
			origin: OriginFor<T>,
			overwatch_node_id: u32
		) -> DispatchResult {
			T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
			Self::do_collective_remove_overwatch_node(overwatch_node_id)
		}

		/// Set new max subnet nodes (per subnet)
		///
		/// # Requirements
		///
		/// Requires majority vote
		/// 
		#[pallet::call_index(72)]
		#[pallet::weight({0})]
		pub fn set_max_subnet_nodes(
			origin: OriginFor<T>, 
			value: u32
		) -> DispatchResult {
			T::MajorityCollectiveOrigin::ensure_origin(origin)?;
			Self::do_set_max_subnet_nodes(value)
		}

		/// Set new minimum subnet delegate stake factor
		///
		/// # Requirements
		///
		/// Requires super majority vote
		/// 
		#[pallet::call_index(73)]
		#[pallet::weight({0})]
		pub fn set_min_subnet_delegate_stake_factor(
			origin: OriginFor<T>, 
			value: u128
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
		#[pallet::call_index(74)]
		#[pallet::weight({0})]
		pub fn set_subnet_owner_percentage(
			origin: OriginFor<T>, 
			value: u128
		) -> DispatchResult {
			T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;
			Self::do_set_subnet_owner_percentage(value)
		}

		#[pallet::call_index(75)]
		#[pallet::weight({0})]
		pub fn set_sigmoid_midpoint(
			origin: OriginFor<T>, 
			value: u128
		) -> DispatchResult {
			T::MajorityCollectiveOrigin::ensure_origin(origin)?;
			Self::do_set_sigmoid_midpoint(value)
		}

		#[pallet::call_index(76)]
		#[pallet::weight({0})]
		pub fn set_sigmoid_steepness(
			origin: OriginFor<T>, 
			value: u128
		) -> DispatchResult {
			T::MajorityCollectiveOrigin::ensure_origin(origin)?;
			Self::do_set_sigmoid_steepness(value)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Register subnet
		pub fn do_register_subnet(
			owner: T::AccountId,
			owner_hotkey: T::AccountId,
			subnet_registration_data: RegistrationSubnetData<T::AccountId>,
		) -> DispatchResult {
			// Ensure name is unique
			ensure!(
				!SubnetName::<T>::contains_key(&subnet_registration_data.name),
				Error::<T>::SubnetNameExist
			);

			// Ensure name is unique
			ensure!(
				!SubnetRepo::<T>::contains_key(&subnet_registration_data.repo),
				Error::<T>::SubnetRepoExist
			);

			let epoch = Self::get_current_epoch_as_u32();

			ensure!(
				Self::can_subnet_register(epoch),
				Error::<T>::SubnetRegistrationCooldown
			);
	
			// TODO: Add conditionals for subnet_registration_data
			ensure!(
				subnet_registration_data.churn_limit >= MinChurnLimit::<T>::get() &&
				subnet_registration_data.churn_limit <= MaxChurnLimit::<T>::get(),
				Error::<T>::InvalidChurnLimit
			);

			ensure!(
				subnet_registration_data.registration_queue_epochs >= MinRegistrationQueueEpochs::<T>::get() &&
				subnet_registration_data.registration_queue_epochs <= MaxRegistrationQueueEpochs::<T>::get(),
				Error::<T>::InvalidRegistrationQueueEpochs
			);

			ensure!(
				subnet_registration_data.activation_grace_epochs >= MinActivationGraceEpochs::<T>::get() &&
				subnet_registration_data.activation_grace_epochs <= MaxActivationGraceEpochs::<T>::get(),
				Error::<T>::InvalidActivationGraceEpochs
			);

			ensure!(
				subnet_registration_data.queue_classification_epochs >= MinIdleClassificationEpochs::<T>::get() &&
				subnet_registration_data.queue_classification_epochs <= MaxIdleClassificationEpochs::<T>::get(),
				Error::<T>::InvalidIdleClassificationEpochs
			);

			ensure!(
				subnet_registration_data.included_classification_epochs >= MinIncludedClassificationEpochs::<T>::get() &&
				subnet_registration_data.included_classification_epochs <= MaxIncludedClassificationEpochs::<T>::get(),
				Error::<T>::InvalidIncludedClassificationEpochs
			);

			ensure!(
				subnet_registration_data.min_stake >= MinSubnetMinStake::<T>::get() &&
				subnet_registration_data.min_stake <= MaxSubnetMinStake::<T>::get(),
				Error::<T>::InvalidSubnetMinStake
			);

			ensure!(
				subnet_registration_data.max_stake >= MinSubnetMaxStake::<T>::get() &&
				subnet_registration_data.max_stake <= MaxSubnetMaxStake::<T>::get(),
				Error::<T>::InvalidSubnetMaxStake
			);

			ensure!(
				subnet_registration_data.min_stake <= subnet_registration_data.max_stake,
				Error::<T>::InvalidSubnetStakeParameters
			);

			ensure!(
				subnet_registration_data.delegate_stake_percentage >= MinDelegateStakePercentage::<T>::get() &&
				subnet_registration_data.delegate_stake_percentage <= MaxDelegateStakePercentage::<T>::get() &&
				subnet_registration_data.delegate_stake_percentage <= Self::percentage_factor_as_u128(),
				Error::<T>::InvalidMinDelegateStakePercentage
			);

			ensure!(
				subnet_registration_data.max_registered_nodes >= MinMaxRegisteredNodes::<T>::get(),
				Error::<T>::InvalidMaxRegisteredNodes
			);

			// --- Must have at least min subnet nodes as initial coldkeys
			// While initial coldkeys can register multiple nodes, this forces
			// further decentalizataion and diversification.
			ensure!(
				subnet_registration_data.initial_coldkeys.len() as u32 >= MinSubnetNodes::<T>::get(),
				Error::<T>::InvalidSubnetRegistrationInitialColdkeys
			);

			let subnet_fee: u128 = Self::registration_cost(epoch);

			if subnet_fee > 0 {
				let subnet_fee_as_balance = Self::u128_to_balance(subnet_fee);

				// Ensure user has the funds, give accurate information on errors
				ensure!(
					Self::can_remove_balance_from_coldkey_account(&owner, subnet_fee_as_balance.unwrap()),
					Error::<T>::NotEnoughBalanceToRegisterSubnet
				);
				
				// Send funds to Treasury and revert if failed
				Self::send_to_treasury(&owner, subnet_fee_as_balance.unwrap())?;
			}

			// Get total subnets ever
			let subnet_uids: u32 = TotalSubnetUids::<T>::get();

			// Start the subnet_ids at 1
			let subnet_id = subnet_uids.saturating_add(1);
			// Increase total subnets. This is used for unique Subnet IDs
			TotalSubnetUids::<T>::mutate(|n: &mut u32| *n += 1);

			let subnet_data = SubnetData {
				id: subnet_id,
				name: subnet_registration_data.name,
				repo: subnet_registration_data.repo,
				description: subnet_registration_data.description,
				misc: subnet_registration_data.misc,
				state: SubnetState::Registered,
				start_epoch: u32::MAX,
			};

			ensure!(
				&owner != &owner_hotkey,
				Error::<T>::ColdkeyMatchesHotkey
			);

			// Store owner
			SubnetOwner::<T>::insert(subnet_id, &owner);

			// SubnetOwnerV2::<T>::insert(
			// 	subnet_id,
			// 	SubnetOwnerKeys {
			// 		coldkey: owner.clone(),
			// 		hotkey: owner_hotkey.clone(),
			// 	}
			// );
			// // SubnetOwnerHotkey::<T>::insert(subnet_id, &owner_hotkey);
			// // Give owner HotkeyOwner for rewards
			// // Enables to give owner rewards from staking and require them to use the unbonding ledger
			// HotkeyOwner::<T>::insert(&owner_hotkey, &owner);

			// // --- Ensure owner is not using the coldkey has a hotkey anywhere else
			// let mut hotkeys = ColdkeyHotkeys::<T>::get(&owner);
			// ensure!(
			// 	!hotkeys.contains(&owner_hotkey),
			// 	Error::<T>::HotkeyAlreadyRegisteredToColdkey
			// );
			// hotkeys.insert(owner_hotkey.clone());
			// ColdkeyHotkeys::<T>::insert(&owner, hotkeys);

			// Store the stake balance range
			SubnetMinStakeBalance::<T>::insert(subnet_id, subnet_registration_data.min_stake);
			SubnetMaxStakeBalance::<T>::insert(subnet_id, subnet_registration_data.max_stake);

			// Add delegate state ratio
			SubnetDelegateStakeRewardsPercentage::<T>::insert(subnet_id, subnet_registration_data.delegate_stake_percentage);

			// Add classification epochs
			RegistrationQueueEpochs::<T>::insert(subnet_id, subnet_registration_data.registration_queue_epochs);
			IdleClassificationEpochs::<T>::insert(subnet_id, subnet_registration_data.queue_classification_epochs);
			IncludedClassificationEpochs::<T>::insert(subnet_id, subnet_registration_data.included_classification_epochs);

			// Add queue variables
			ActivationGraceEpochs::<T>::insert(subnet_id, subnet_registration_data.activation_grace_epochs);
			ChurnLimit::<T>::insert(subnet_id, subnet_registration_data.churn_limit);

			// Store max node penalties
			MaxSubnetNodePenalties::<T>::insert(subnet_id, subnet_registration_data.max_node_penalties);

			// Store whitelisted coldkeys for registration period
			SubnetRegistrationInitialColdkeys::<T>::insert(
				subnet_id, 
				subnet_registration_data.initial_coldkeys
			);

			MaxRegisteredNodes::<T>::insert(subnet_id, subnet_registration_data.max_registered_nodes);

			// Store unique name
			SubnetName::<T>::insert(&subnet_data.name, subnet_id);
			SubnetRepo::<T>::insert(&subnet_data.repo, subnet_id);
			SubnetNodeRemovalSystem::<T>::insert(subnet_id, subnet_registration_data.node_removal_system);
			SubnetKeyTypes::<T>::insert(subnet_id, subnet_registration_data.key_types);
			// Store subnet data
			SubnetsData::<T>::insert(subnet_id, &subnet_data);
			// Update latest registration epoch for all subnets
			// This is used for one subnet per registration phase
			LastSubnetRegistrationEpoch::<T>::set(epoch);
			// Store registration epoch temporarily
			SubnetRegistrationEpoch::<T>::insert(subnet_id, epoch);

			Self::deposit_event(Event::SubnetRegistered { 
				account_id: owner, 
				name: subnet_data.name, 
				subnet_id: subnet_id 
			});

			Ok(())
		}

		/// Activate subnet or remove registering subnet if doesn't meet requirements
		pub fn do_activate_subnet(subnet_id: u32) -> DispatchResult {
			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::InvalidSubnetId.into()),
			};

			ensure!(
				subnet.state == SubnetState::Registered,
				Error::<T>::SubnetActivatedAlready
			);

			let epoch: u32 = Self::get_current_epoch_as_u32();

			let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
	
			// --- Ensure the subnet has passed it's required period to begin consensus submissions
			// --- Ensure the subnet is within the enactment period
			ensure!(
				Self::is_subnet_registering(subnet_id, subnet.state, epoch) == false,
				Error::<T>::SubnetInitializing
			);

			// --- If subnet not activated yet and is outside the enactment period, remove subnet
			if Self::is_subnet_in_enactment(subnet_id, subnet.state, epoch) == false {
				return Self::do_remove_subnet(
					subnet_id,
					SubnetRemovalReason::EnactmentPeriod,
				)
			}

			// --- 1. Ensure minimum nodes are activated
			let total_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);

			if total_nodes < MinSubnetNodes::<T>::get() {
				return Self::do_remove_subnet(
					subnet_id,
					SubnetRemovalReason::MinSubnetNodes,
				)
			}

			// --- 2. Ensure minimum delegate stake achieved 
			let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
			let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance();

			// --- Ensure delegate stake balance is below minimum threshold required
			if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
				return Self::do_remove_subnet(
					subnet_id,
					SubnetRemovalReason::MinSubnetDelegateStake,
				)
			}

			// ===============
			// Gauntlet passed
			// ===============

			// --- Activate subnet
			SubnetsData::<T>::try_mutate(
				subnet_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;
					params.state = SubnetState::Active;
					// Start consensus after 1 fresh epoch.
					// Consensus starts once epoch > start_epoch
					params.start_epoch = epoch + 1;
					Ok(())
				}
			)?;

			TotalActiveSubnets::<T>::mutate(|n: &mut u32| *n += 1);

			// --- Remove registration whitelist
			SubnetRegistrationInitialColdkeys::<T>::remove(subnet_id);

			Self::assign_subnet_slot(subnet_id)?;

      Self::deposit_event(Event::SubnetActivated { subnet_id: subnet_id });
	
			Ok(())
		}

		pub fn do_remove_subnet(
			subnet_id: u32,
			reason: SubnetRemovalReason,
		) -> DispatchResult {
			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::InvalidSubnet.into()),
			};

			// Remove unique name
			SubnetName::<T>::remove(&subnet.name);
			SubnetRepo::<T>::remove(&subnet.repo);
			// Remove subnet data
			SubnetsData::<T>::remove(subnet_id);

			SubnetRegistrationEpoch::<T>::remove(subnet_id);
			SubnetRegistrationInitialColdkeys::<T>::remove(subnet_id);

			Self::free_slot_of_subnet(subnet_id);

			if subnet.state == SubnetState::Active {
				// Dec total active subnets
				TotalActiveSubnets::<T>::mutate(|n: &mut u32| n.saturating_dec());
			}

			// Remove all subnet nodes data
			let _ = SubnetNodesData::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let total_nodes = TotalActiveSubnetNodes::<T>::take(subnet_id);
			TotalActiveNodes::<T>::mutate(|n: &mut u32| n.saturating_reduce(total_nodes));

			// We have removed all of the data required to assist in blockchain logic
			// `clean_subnet_nodes` cleans up non-required data
			Self::clean_subnet_nodes(subnet_id);
	
			Self::deposit_event(Event::SubnetDeactivated { subnet_id: subnet_id, reason: reason });

			Ok(())
		}

		pub fn clean_subnet_nodes(
			subnet_id: u32,
		) -> DispatchResult {
			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::InvalidSubnet.into()),
			};

			let _ = TotalSubnetNodes::<T>::remove(subnet_id);
			let _ = TotalSubnetNodeUids::<T>::remove(subnet_id);
			let _ = PeerIdSubnetNodeId::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = BootstrapPeerIdSubnetNodeId::<T>::clear_prefix(subnet_id, u32::MAX, None);			
			let _ = SubnetNodeUniqueParam::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = HotkeySubnetNodeId::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = SubnetNodeIdHotkey::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = SubnetNodeNonUniqueParamLastSet::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = SubnetNodePenalties::<T>::clear_prefix(subnet_id, u32::MAX, None);

			let electable_nodes = SubnetNodeElectionSlots::<T>::get(subnet_id).len() as u32;
			TotalElectableNodes::<T>::mutate(|mut n| n.saturating_sub(electable_nodes));

			// Remove all subnet consensus data
			let _ = SubnetPenaltyCount::<T>::remove(subnet_id);

			// Remove consensus data
			let _ = SubnetElectedValidator::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = SubnetConsensusSubmission::<T>::clear_prefix(subnet_id, u32::MAX, None);

			Ok(())
		}

		pub fn do_remove_subnet_node(
			subnet_id: u32,
			subnet_node_id: u32,
		) -> DispatchResult {
			Self::perform_remove_subnet_node(subnet_id, subnet_node_id);
			Ok(())
		}

		// Get the start epoch
		// This epoch will be the epoch (+ grace period) where the node can activate
		// after the queue (from Registration class (in queue) to Idle class)
		pub fn get_registration_queue_start_epoch(
			subnet_id: u32,
			subnet_node_id: u32,
			epoch: u32
		) -> u32 {
			let churn_limit = ChurnLimit::<T>::get(subnet_id).max(1);
			let queue_epochs = RegistrationQueueEpochs::<T>::get(subnet_id);
			let grace_epochs = ActivationGraceEpochs::<T>::get(subnet_id);

			let position = RegisteredSubnetNodesData::<T>::iter_prefix(subnet_id)
				.filter(|(_, node)| epoch <= node.classification.start_epoch + grace_epochs)
				.count() as u32;
			let additional_epochs = position / churn_limit;
			let start_epoch = epoch + queue_epochs + additional_epochs;

			start_epoch
		}

		pub fn do_register_subnet_node(
			origin: OriginFor<T>,
			subnet_id: u32,
			hotkey: T::AccountId,
			peer_id: PeerId,
			bootstrap_peer_id: PeerId,
			delegate_reward_rate: u128,
			stake_to_be_added: u128,
			a: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
			b: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
			c: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin.clone())?;

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::InvalidSubnet.into()),
			};

			ensure!(
				&coldkey != &hotkey,
				Error::<T>::ColdkeyMatchesHotkey
			);

			ensure!(
				subnet.state != SubnetState::Paused,
				Error::<T>::SubnetIsPaused
			);

			// Unique network-wide hotkey
			ensure!(
				!Self::hotkey_has_owner(hotkey.clone()),
				Error::<T>::HotkeyHasOwner
			);

			let epoch: u32 = Self::get_current_epoch_as_u32();

			// If in enactment period, registering is disabled
			// Nodes must enter in the registration period or activation period
			// Once we are in the enactment period, only delegate staking is enabled to reach the qualifications
			ensure!(
				!Self::is_subnet_in_enactment(subnet_id, subnet.state, epoch),
				Error::<T>::SubnetMustBeRegisteringOrActivated
			);

			// --- If in registration period, check if there is a whitelist and coldkey is in the whitelist
			// `SubnetRegistrationInitialColdkeys` is removed on activation
			match SubnetRegistrationInitialColdkeys::<T>::try_get(subnet_id) {
				Ok(key_tree) => {
					ensure!(
						key_tree.contains(&coldkey),
						Error::<T>::ColdkeyRegistrationWhitelist
					);
				},
				// Has no initial coldkeys list, pass because subnet is activated
				Err(()) => {
					let grace_epochs = ActivationGraceEpochs::<T>::get(subnet_id);
					let total_registered_nodes = RegisteredSubnetNodesData::<T>::iter_prefix(subnet_id)
						.filter(|(_, node)| epoch <= node.classification.start_epoch + grace_epochs)
						.count() as u32;
					ensure!(
						total_registered_nodes <= MaxRegisteredNodes::<T>::get(subnet_id),
						Error::<T>::MaxRegisteredNodes
					);
				},
			};

			// Unique ``a``
			// [here]
			if a.is_some() {
				ensure!(
					!SubnetNodeUniqueParam::<T>::contains_key(subnet_id, a.clone().unwrap()),
					Error::<T>::SubnetNodeUniqueParamTaken
				);
				SubnetNodeUniqueParam::<T>::insert(subnet_id, a.clone().unwrap(), &peer_id);
			}

			// Validate peer_id
			ensure!(
				Self::validate_peer_id(&peer_id),
				Error::<T>::InvalidPeerId
			);

			ensure!(
				Self::validate_peer_id(&bootstrap_peer_id),
				Error::<T>::InvalidBootstrapPeerId
			);	

			// Ensure peer and boostrap peer ID doesn't already exist within subnet regardless of coldkey

			// Unique subnet_id -> PeerId
			ensure!(
				Self::is_owner_of_peer_or_ownerless(subnet_id, 0, 0, &peer_id),
				Error::<T>::PeerIdExist
			);

			// Unique subnet_id -> Bootstrap PeerId
			ensure!(
				Self::is_owner_of_peer_or_ownerless(subnet_id, 0, 0, &bootstrap_peer_id),
				Error::<T>::BootstrapPeerIdExist
			);

			// --- Ensure they have no stake on registration
			// If a Subnet Node deregisters, then they must fully unstake its stake balance to register again using that same balance
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

			// ====================
			// Initiate stake logic
			// ====================
			Self::do_add_stake(
				origin.clone(), 
				subnet_id,
				hotkey.clone(),
				stake_to_be_added,
			).map_err(|e| e)?;
			
			let block: u32 = Self::get_current_block_as_u32();

			// --- Only use block for last_delegate_reward_rate_update is rate is greater than zero
			let mut last_delegate_reward_rate_update = 0;
			if delegate_reward_rate > 0 {
				last_delegate_reward_rate_update = block;
			}

			// --- Start the UIDs at 1
			// TODO: Make one UID element for all subnets
			TotalSubnetNodeUids::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
			let current_uid = TotalSubnetNodeUids::<T>::get(subnet_id);

			HotkeySubnetNodeId::<T>::insert(subnet_id, &hotkey, current_uid);

			// Insert Subnet Node ID -> hotkey
			SubnetNodeIdHotkey::<T>::insert(subnet_id, current_uid, &hotkey);

			// Insert hotkey -> coldkey
			HotkeyOwner::<T>::insert(&hotkey, &coldkey);
			
			// Insert coldkey -> hotkeys
			hotkeys.insert(hotkey.clone());
			ColdkeyHotkeys::<T>::insert(&coldkey, hotkeys);

			// To ensure the AccountId that owns the PeerId, the subnet should use signature authentication
			// This ensures others cannot claim to own a PeerId they are not the owner of

			// Insert subnet peer and bootstrap peer to keep peer_ids unique within subnets
			PeerIdSubnetNodeId::<T>::insert(subnet_id, &peer_id, current_uid);
			BootstrapPeerIdSubnetNodeId::<T>::insert(subnet_id, &bootstrap_peer_id, current_uid);

			// Add to registration queue
			let subnet_epoch: u32 = Self::get_current_subnet_epoch_as_u32(subnet_id);
			let start_epoch = Self::get_registration_queue_start_epoch(
				subnet_id,
				current_uid,
				subnet_epoch
			);

			// ========================
			// Insert peer into storage
			// ========================
			let classification: SubnetNodeClassification = SubnetNodeClassification {
				node_class: SubnetNodeClass::Registered,
				start_epoch: start_epoch,
			};

			let subnet_node: SubnetNode<T::AccountId> = SubnetNode {
				id: current_uid,
				hotkey: hotkey.clone(),
				peer_id: peer_id.clone(),
				bootstrap_peer_id: bootstrap_peer_id.clone(),
				client_peer_id: bootstrap_peer_id.clone(),
				classification: classification,
				delegate_reward_rate: delegate_reward_rate,
				last_delegate_reward_rate_update: last_delegate_reward_rate_update,
				a: a,
				b: b,
				c: c,
			};

			// Insert RegisteredSubnetNodesData
			RegisteredSubnetNodesData::<T>::insert(subnet_id, current_uid, subnet_node);

			// Increase total subnet nodes
			TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
			TotalNodes::<T>::mutate(|n: &mut u32| *n += 1);

			// Add subnet Id to coldkey subnets set
			ColdkeySubnets::<T>::mutate(&coldkey, |subnets| {
				subnets.insert(subnet_id);
			});

			Self::deposit_event(
				Event::SubnetNodeRegistered { 
					subnet_id: subnet_id, 
					subnet_node_id: current_uid,
					coldkey: coldkey,
					hotkey: hotkey, 
					peer_id: peer_id,
				}
			);

			Ok(())
		}

		pub fn do_activate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResultWithPostInfo {
			// Activate with hotkey from the Subnet Node server
			let key: T::AccountId = ensure_signed(origin)?;

			let (hotkey, coldkey) = match Self::get_subnet_node_hotkey_coldkey(subnet_id, subnet_node_id) {
				Some((hotkey, coldkey)) => {
					(hotkey, coldkey)
				}
				None => {
					return Err(Error::<T>::NotKeyOwner.into())
				}
			};

			ensure!(
				key == hotkey || key == coldkey,
				Error::<T>::NotKeyOwner
			);

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::InvalidSubnet.into()),
			};

			ensure!(
				HotkeySubnetNodeId::<T>::get(subnet_id, &hotkey) == Some(subnet_node_id),
				Error::<T>::NotUidOwner
			);

			// --- Ensure they are a registered node
			ensure!(
				RegisteredSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id),
				Error::<T>::NotUidOwner
			);

			// --- Check stake balance
			// This can only revert if the owner updated the required minimum balance
			ensure!(
				AccountSubnetStake::<T>::get(&hotkey, subnet_id) >= SubnetMinStakeBalance::<T>::get(subnet_id),
				Error::<T>::MinStakeNotReached
			);

			// --- Remove from RegisteredSubnetNodesData
			let mut subnet_node = RegisteredSubnetNodesData::<T>::take(subnet_id, subnet_node_id);
			let start_epoch = subnet_node.classification.start_epoch;
			let grace_epochs = ActivationGraceEpochs::<T>::get(subnet_id);

			// let epoch: u32 = Self::get_current_epoch_as_u32();
			let subnet_epoch: u32 = Self::get_current_subnet_epoch_as_u32(subnet_id);

			// Node must register within the grace period if subnet activated
			ensure!(
				subnet.state == SubnetState::Registered ||
				subnet_epoch >= start_epoch && subnet_epoch <= start_epoch + grace_epochs,
				Error::<T>::NotStartEpoch
			);

			let total_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
			let max_nodes = MaxSubnetNodes::<T>::get();

			// Push node into queue again if paused
			if subnet.state == SubnetState::Paused {
				Self::defer_subnet_node(subnet_id, subnet_node, subnet_epoch);
				return Ok(Pays::No.into())
			}

			// --- Use the subnets node removal system
			// If node can't be removed, push the node back into the queue
			if total_nodes >= max_nodes {
				let mut maybe_remove_node: Option<u32> = None;
				let mut should_defer = false;

				match Self::get_removing_node(
					subnet_id, 
					&coldkey, 
					&hotkey, 
					&subnet_node
				) {
					Some(remove_subnet_node_id) => {
						maybe_remove_node = Some(remove_subnet_node_id);
					},
					None => {
						should_defer = true;
					},
				}

				// Redundant
				// Ensure either maybe_remove_node is Some or should_defer is true
				ensure!(maybe_remove_node.is_some() || should_defer, Error::<T>::SubnetNodesMax);

				if let Some(remove_subnet_node_id) = maybe_remove_node {
					// Remove node and add activating node in `perform_activate_subnet_node`
					Self::perform_remove_subnet_node(subnet_id, remove_subnet_node_id);
				} else if should_defer {
					// --- Defer the start_epoch
					// Add node back to the queue
					Self::defer_subnet_node(subnet_id, subnet_node, subnet_epoch);
					return Ok(Pays::No.into())
				}
			}
			
			Self::perform_activate_subnet_node(
				coldkey, 
				subnet_id, 
				subnet.state,
				subnet_node,
				subnet_epoch,
			).map_err(|e| e)?;

			Ok(().into())
		}

		fn defer_subnet_node(subnet_id: u32, mut subnet_node: SubnetNode<T::AccountId>, subnet_epoch: u32) {
			let new_start_epoch = Self::get_registration_queue_start_epoch(
				subnet_id,
				subnet_node.id,
				subnet_epoch,
			);
			subnet_node.classification.start_epoch = new_start_epoch;
			RegisteredSubnetNodesData::<T>::insert(subnet_id, subnet_node.id, &subnet_node);
		}

		pub fn perform_activate_subnet_node(
			coldkey: T::AccountId, 
			subnet_id: u32, 
			subnet_state: SubnetState,
			mut subnet_node: SubnetNode<T::AccountId>,
			subnet_epoch: u32,
		) -> DispatchResult {
			// --- If subnet activated, activate the node starting at `Idle`
			subnet_node.classification.node_class = SubnetNodeClass::Idle;
			// --- Increase epoch by one to ensure node starts on a fresh epoch unless subnet is still registering
			subnet_node.classification.start_epoch = subnet_epoch + 1;

			// --- If subnet in registration, activate node at `Validator` to start off subnet consensus
			// --- Initial nodes before activation are entered as ``Validator`` nodes
			// They initiate the first consensus epoch and are responsible for increasing classifications
			// of other nodes that come in post activation
			if subnet_state == SubnetState::Registered {
				subnet_node.classification.node_class = SubnetNodeClass::Validator;
				// --- Start node on current epoch for the next era
				// subnet_node.classification.start_epoch = epoch;
				subnet_node.classification.start_epoch = subnet_epoch;

				// --- Insert into election slot if entering as Validator class
				// The only other way to enter the election slots is by being graduated by consensus
				ensure!(
					Self::insert_node_into_election_slot(subnet_id, subnet_node.id),
					Error::<T>::MaxSubnetNodes
				)
			}

			// --- Enter node into the Idle class
			SubnetNodesData::<T>::insert(subnet_id, subnet_node.id, &subnet_node);
			// Increase total active nodes in subnet
			TotalActiveSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

			// Increase total active nodes
			TotalActiveNodes::<T>::mutate(|n: &mut u32| *n += 1);

			ColdkeyReputation::<T>::mutate(&coldkey, |rep| {
				rep.lifetime_node_count = rep.lifetime_node_count.saturating_add(1);
				rep.total_active_nodes = rep.total_active_nodes.saturating_add(1);
			});

			Self::deposit_event(
				Event::SubnetNodeActivated { 
					subnet_id: subnet_id, 
					subnet_node_id: subnet_node.id, 
				}
			);

			Ok(())
		}


		/// Called by extrinsic ``deactivate_subnet_node``
		pub fn do_deactivate_subnet_node_new(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResult {
			let key: T::AccountId = ensure_signed(origin)?;

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::InvalidSubnet.into()),
			};

			// can deactivate using both keys
			let (hotkey, coldkey) = match Self::get_subnet_node_hotkey_coldkey(subnet_id, subnet_node_id) {
				Some((hotkey, coldkey)) => {
					(hotkey, coldkey)
				}
				None => {
					return Err(Error::<T>::NotUidOwner.into())
				}
			};

			ensure!(
				key == hotkey.clone() || key == coldkey,
				Error::<T>::NotKeyOwner
			);

			// let epoch: u32 = Self::get_current_epoch_as_u32();

			// Ensure node is a current Validator node
			// Otherise they cannot deactivate and must remove
			let mut subnet_node = match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
				Ok(subnet_node) => {
					ensure!(
						subnet_node.classification.node_class >= SubnetNodeClass::Validator,
            Error::<T>::SubnetNodeNotActivated
					);
					SubnetNodesData::<T>::take(subnet_id, subnet_node_id)
				},
				Err(()) => return Err(Error::<T>::SubnetNodeNotExist.into()),
			};

			let subnet_epoch: u32 = Self::get_current_subnet_epoch_as_u32(subnet_id);
			let is_validator: bool = Self::is_validator(subnet_id, subnet_node_id, subnet_epoch);

			ensure!(
				!is_validator,
				Error::<T>::IsValidatorCannotDeactivate
			);

			subnet_node.classification.node_class = SubnetNodeClass::Deactivated;
			// --- Increase epoch by one to ensure node starts on a fresh epoch unless subnet is still registering
			// subnet_node.classification.start_epoch = epoch + 1;
			subnet_node.classification.start_epoch = subnet_epoch + 1;

			DeactivatedSubnetNodesData::<T>::insert(subnet_id, subnet_node_id, subnet_node);

			// Decrease total subnet nodes
			TotalActiveSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());
			TotalActiveNodes::<T>::mutate(|n: &mut u32| n.saturating_dec());

			// Must already be validator to get this far, so we remove from the election slots
			Self::remove_node_from_election_slot(subnet_id, subnet_node_id);

			Self::deposit_event(
				Event::SubnetNodeDeactivated { 
					subnet_id: subnet_id, 
					subnet_node_id: subnet_node_id, 
				}
			);

			Ok(())
		}
		
		pub fn do_reactivate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResult {
			let key: T::AccountId = ensure_signed(origin)?;

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::InvalidSubnet.into()),
			};

			// can deactivate using both keys
			let (hotkey, coldkey) = match Self::get_subnet_node_hotkey_coldkey(subnet_id, subnet_node_id) {
				Some((hotkey, coldkey)) => {
					(hotkey, coldkey)
				}
				None => {
					return Err(Error::<T>::NotUidOwner.into())
				}
			};

			ensure!(
				key == hotkey.clone() || key == coldkey,
				Error::<T>::NotKeyOwner
			);

			ensure!(
				HotkeySubnetNodeId::<T>::get(subnet_id, &hotkey) == Some(subnet_node_id),
				Error::<T>::NotUidOwner
			);

			// --- Ensure they are a registered node
			ensure!(
				DeactivatedSubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id),
				Error::<T>::NotUidOwner
			);
			
			// --- Remove from DeactivatedSubnetNodesData
			let mut subnet_node = DeactivatedSubnetNodesData::<T>::take(subnet_id, subnet_node_id);
			let max_deactivation_epochs = MaxDeactivationEpochs::<T>::get(subnet_id);
			let start_epoch = subnet_node.classification.start_epoch;

			// let epoch: u32 = Self::get_current_epoch_as_u32();
			let subnet_epoch: u32 = Self::get_current_subnet_epoch_as_u32(subnet_id);

			// Node must register within the grace period if subnet activated
			ensure!(
				subnet_epoch >= start_epoch && subnet_epoch <= start_epoch + max_deactivation_epochs,
				Error::<T>::NotStartEpoch
			);
			// ensure!(
			// 	epoch >= start_epoch && epoch <= start_epoch + max_deactivation_epochs,
			// 	Error::<T>::NotStartEpoch
			// );
			
			ensure!(
				Self::insert_node_into_election_slot(subnet_id, subnet_node_id),
				Error::<T>::MaxSubnetNodes
			);

			// Add SubnetNodesData
			// --- If subnet reactivated, activate the node starting at `Validator`
			// All nodes that deactivate must be Validator class, so we start them back where they deactivated from
			subnet_node.classification.node_class = SubnetNodeClass::Validator;
			// --- Increase epoch by one to ensure node starts on a fresh epoch unless subnet is still registering
			// subnet_node.classification.start_epoch = epoch + 1;
			subnet_node.classification.start_epoch = subnet_epoch + 1;

			// --- Enter node into the Idle class
			SubnetNodesData::<T>::insert(subnet_id, subnet_node_id, subnet_node);

			// Increase total subnet nodes
			TotalActiveSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
			TotalActiveNodes::<T>::mutate(|n: &mut u32| *n += 1);
	
			Self::deposit_event(
				Event::SubnetNodeReactivated { 
					subnet_id: subnet_id, 
					subnet_node_id: subnet_node_id, 
				}
			);

			Ok(())
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
		/// 1. Reward subnet nodes from the previous epochs validators data
		/// 2. Run deactivation ledger
		/// 3. Do epoch preliminaries
		///		* Remove subnets if needed
		/// 	* Randomly choose subnet validators
		///
		/// # Arguments
		///
		/// * `block_number` - Current block number.
		/// 
		fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
			let mut weight = Weight::zero();
			if Self::is_paused().is_err() {
				return weight
			}
			let db_weight = T::DbWeight::get();

			let block: u32 = Self::convert_block_as_u32(block_number);
			let epoch_length: u32 = T::EpochLength::get();
			let epoch_slot = block % epoch_length;
			let current_epoch = block.saturating_div(epoch_length);

			if block >= epoch_length && block % epoch_length == 0 {
				// Remove unqualified subnets
				let step_weight = Self::do_epoch_preliminaries(block, current_epoch);
				return weight.saturating_add(step_weight)
			} else if (block - 1) >= epoch_length && (block - 1) % epoch_length == 0 {
				// Calculate rewards
				// Distribute foundation
				// Calculate emissions based on subnet weights (delegate stake based)
				// We calculate based on delegate stake weights on all subnets
				let step_weight = Self::handle_subnet_emission_weights(current_epoch);
				return weight.saturating_add(step_weight)
			} else if let Some(subnet_id) = SlotAssignment::<T>::get(epoch_slot) {
				weight = weight.saturating_add(db_weight.reads(1));
				let step_weight = Self::emission_step(block, current_epoch, subnet_id);
				return weight.saturating_add(step_weight)
			} else {
				// If we made it this far, SlotAssignment was read
				weight = weight.saturating_add(db_weight.reads(1));
			}
		
			// for EVM tests (Weights in on_initialize change the block weight/gas)
			Weight::from_parts(0, 0)
		}

		fn on_finalize(block_number: BlockNumberFor<T>) {
			// let block: u32 = Self::convert_block_as_u32(block_number);
			// Self::do_queue(block);
		}

		fn on_idle(block_number: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
			// let block: u32 = Self::convert_block_as_u32(block_number);

			// if remaining_weight.any_lt(T::DbWeight::get().reads(2)) {
			// 	return Weight::from_parts(0, 0)
			// }

			return Weight::from_parts(0, 0)

			// Self::do_on_idle(remaining_weight)
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn do_on_idle(remaining_weight: Weight) -> Weight {
			// any weight that is unaccounted for
			let mut unaccounted_weight = Weight::from_parts(0, 0);

			unaccounted_weight
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
			// use sp_core::U256;
			// use sp_core::crypto::Ss58Codec;

			// if self.subnet_name.last().is_none() {
			// 	return
			// }
			
			// let subnet_id = 1;
				
			// let subnet_data = SubnetData {
			// 	id: subnet_id,
			// 	name: self.subnet_name.clone(),
			// 	repo: Vec::new(),
			// 	description: Vec::new(),
			// 	misc: Vec::new(),
			// 	state: SubnetState::Active,
			// };
			
			// SubnetRegistrationEpoch::<T>::insert(subnet_id, 1);
			// // Store unique name
			// SubnetName::<T>::insert(self.subnet_name.clone(), subnet_id);
			// // Store subnet data
			// SubnetsData::<T>::insert(subnet_id, subnet_data.clone());
			// // Increase total subnets count
			// TotalSubnetUids::<T>::mutate(|n: &mut u32| *n += 1);

			// LastSubnetRegistrationEpoch::<T>::set(1);

			// // Increase delegate stake to allow activation of subnet model
			// let min_stake_balance = NetworkMinStakeBalance::<T>::get();
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
			// 	.saturating_add(total_staked)
			// 	.saturating_add(total_delegate_staked)
			// 	.saturating_add(total_node_delegate_staked);
	
			// let factor: u128 = MinSubnetDelegateStakeFactor::<T>::get();

			// let x = U256::from(total_network_issuance);
			// let y = U256::from(factor);

			// // x * y / 100.0

			// let result = x * y / U256([0xde0b6b3a7640000, 0x0, 0x0, 0x0]);
			// log::info!("result                               {:?}", result);

			// let min_subnet_delegate_stake_balance: u128 = result.try_into().unwrap_or(u128::MAX);

			// log::info!("min_subnet_delegate_stake_balance    {:?}", min_subnet_delegate_stake_balance);

			// // --- Mitigate inflation attack
			// TotalSubnetDelegateStakeShares::<T>::mutate(subnet_id, |mut n| n.saturating_accrue(1000));

			// // =================
			// // convert_to_shares
			// // =================
			// let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
			// log::info!("total_subnet_delegated_stake_balance {:?}", total_subnet_delegated_stake_balance);

			// let balance = U256::from(min_subnet_delegate_stake_balance);
			// let total_shares = U256::from(0) + U256::from(10_u128.pow(1));
			// let total_balance = U256::from(total_subnet_delegated_stake_balance) + U256::from(1);
		
			// let shares = balance * total_shares / total_balance;
			// let shares: u128 = shares.try_into().unwrap_or(u128::MAX);

			// // =====================================
			// // increase_account_delegate_stake
			// // =====================================
			// // -- increase total subnet delegate stake balance
			// TotalSubnetDelegateStakeBalance::<T>::mutate(subnet_id, |mut n| n.saturating_accrue(min_subnet_delegate_stake_balance));
			// // -- increase total subnet delegate stake shares
			// TotalSubnetDelegateStakeShares::<T>::mutate(subnet_id, |mut n| n.saturating_accrue(shares));
			// TotalDelegateStake::<T>::set(min_subnet_delegate_stake_balance);

			// // --- Initialize subnet nodes
			// // Only initialize to test using subnet nodes
			// // If testing using subnet nodes in a subnet, comment out the ``for`` loop

			// let mut stake_amount: u128 = NetworkMinStakeBalance::<T>::get();
			
			// let mut count = 0;
			// for (account_id, peer_id) in &self.subnet_nodes {
			// 	// Redundant
			// 	// Unique subnet_id -> PeerId
			// 	// Ensure peer ID doesn't already exist within subnet regardless of account_id
			// 	let peer_exists: bool = match PeerIdSubnetNodeId::<T>::try_get(subnet_id, peer_id.clone()) {
			// 		Ok(_) => true,
			// 		Err(()) => false,
			// 	};

			// 	if peer_exists {
			// 		continue;
			// 	}

			// 	// ====================
			// 	// Initiate stake logic
			// 	// ====================
			// 	// T::Currency::withdraw(
			// 	// 	&account_id,
			// 	// 	stake_amount,
			// 	// 	WithdrawReasons::except(WithdrawReasons::TIP),
			// 	// 	ExistenceRequirement::KeepAlive,
			// 	// );

			// 	// -- increase account subnet staking balance
			// 	AccountSubnetStake::<T>::insert(
			// 		account_id,
			// 		subnet_id,
			// 		AccountSubnetStake::<T>::get(account_id, subnet_id).saturating_add(stake_amount),
			// 	);

			// 	// -- increase total subnet stake
			// 	TotalSubnetStake::<T>::mutate(subnet_id, |mut n| *n += stake_amount);

			// 	// -- increase total stake overall
			// 	TotalStake::<T>::mutate(|mut n| *n += stake_amount);

			// 	// To ensure the AccountId that owns the PeerId, they must sign the PeerId for others to verify
			// 	// This ensures others cannot claim to own a PeerId they are not the owner of
			// 	// Self::validate_signature(&Encode::encode(&peer_id), &signature, &signer)?;

			// 	// ========================
			// 	// Insert peer into storage
			// 	// ========================
			// 	let classification = SubnetNodeClassification {
			// 		node_class: SubnetNodeClass::Validator,
			// 		start_epoch: 0,
			// 	};

			// 	let bounded_peer_id: BoundedVec<u8, DefaultMaxVectorLength> = BoundedVec::try_from(peer_id.clone().0)
			// 		.expect("Vec is within bounds");

			// 	TotalSubnetNodeUids::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
			// 	let current_uid = TotalSubnetNodeUids::<T>::get(subnet_id);
	
			// 	HotkeySubnetNodeId::<T>::insert(subnet_id, account_id.clone(), current_uid);
	
			// 	// Insert Subnet Node ID -> hotkey
			// 	SubnetNodeIdHotkey::<T>::insert(subnet_id, current_uid, account_id.clone());
	
			// 	// Insert hotkey -> coldkey
			// 	HotkeyOwner::<T>::insert(account_id.clone(), account_id.clone());
				
			// 	let subnet_node: SubnetNode<T::AccountId> = SubnetNode {
			// 		id: current_uid,
			// 		hotkey: account_id.clone(),
			// 		peer_id: peer_id.clone(),
			// 		bootstrap_peer_id: peer_id.clone(),
			// 		client_peer_id: peer_id.clone(),
			// 		classification: classification,
			// 		delegate_reward_rate: 0,
			// 		last_delegate_reward_rate_update: 0,
			// 		a: Some(bounded_peer_id),
			// 		b: Some(BoundedVec::new()),
			// 		c: Some(BoundedVec::new()),
			// 	};
	

			// 	// Insert SubnetNodesData
			// 	SubnetNodesData::<T>::insert(subnet_id, current_uid, subnet_node);
	
			// 	// Insert subnet peer account to keep peer_ids unique within subnets
			// 	PeerIdSubnetNodeId::<T>::insert(subnet_id, peer_id.clone(), current_uid);
		
			// 	// Increase total subnet nodes
			// 	TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
	
			// 	count += 1;
			// }
		}
	}
}