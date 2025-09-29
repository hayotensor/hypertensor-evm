use core::marker::PhantomData;
use fp_evm::Log;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::ConstU32;
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, ExitError, PrecompileFailure, PrecompileHandle};
use pallet_network::QueuedSwapCall;
use precompile_utils::{EvmResult, prelude::*, solidity::Codec};
use sp_core::Decode;
use sp_core::{H160, H256, OpaquePeerId, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup, UniqueSaturatedInto};
use sp_std::vec;

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> =
    <Runtime as pallet_balances::Config<Instance>>::Balance;

pub(crate) struct OverwatchNodePrecompile<R>(PhantomData<R>);

impl<R> OverwatchNodePrecompile<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_network::Config,
    R::AccountId: From<[u8; 20]> + Into<[u8; 20]>,
    <R as frame_system::Config>::RuntimeCall:
        From<pallet_network::Call<R>> + GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    pub const HASH_N: u64 = 2050;
}

#[precompile_utils::precompile]
impl<R> OverwatchNodePrecompile<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_network::Config,
    R::AccountId: From<[u8; 20]> + Into<[u8; 20]>,
    <R as frame_system::Config>::RuntimeCall:
        From<pallet_network::Call<R>> + GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    #[precompile::public("registerOverwatchNode(address,uint256)")]
    #[precompile::payable]
    fn register_overwatch_node(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
        stake_to_be_added: U256,
    ) -> EvmResult<()> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let stake_to_be_added = stake_to_be_added.unique_saturated_into();
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::register_overwatch_node {
            hotkey,
            stake_to_be_added,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("removeOverwatchNode(uint256)")]
    fn remove_overwatch_node(
        handle: &mut impl PrecompileHandle,
        overwatch_node_id: U256,
    ) -> EvmResult<()> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::remove_overwatch_node { overwatch_node_id };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("anyoneRemoveOverwatchNode(uint256)")]
    fn anyone_remove_overwatch_node(
        handle: &mut impl PrecompileHandle,
        overwatch_node_id: U256,
    ) -> EvmResult<()> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::anyone_remove_overwatch_node { overwatch_node_id };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("setOverwatchNodePeerId(uint256,uint256,string)")]
    fn set_overwatch_node_peer_id(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        overwatch_node_id: U256,
        peer_id: BoundedString<ConstU32<64>>,
    ) -> EvmResult<()> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;
        let peer_id = OpaquePeerId(peer_id.as_bytes().to_vec());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::set_overwatch_node_peer_id {
            subnet_id,
            overwatch_node_id,
            peer_id,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    // #[precompile::public("commitOverwatchSubnetWeights(uint256,uint256,string)")]
    // fn commit_overwatch_subnet_weights(
    //     handle: &mut impl PrecompileHandle,
    //     overwatch_node_id: U256,
    //     commit_weights: &mut Vec<OverwatchCommit<T::Hash>>
    // ) -> EvmResult<()> {
    //     handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
    //     let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;

    //     let origin = R::AddressMapping::into_account_id(handle.context().caller);
    //     let call = pallet_network::Call::<R>::commit_overwatch_subnet_weights {
    //         subnet_id,
    //         overwatch_node_id,
    //         peer_id
    //     };

    //     RuntimeHelper::<R>::try_dispatch(
    //         handle,
    //         RawOrigin::Signed(origin.clone()).into(),
    //         call,
    //         0,
    //     )?;

    //     Ok(())
    // }

    // #[precompile::public("revealOverwatchSubnetWeights(uint256,uint256,string)")]
    // fn reveal_overwatch_subnet_weights(
    //     handle: &mut impl PrecompileHandle,
    //     overwatch_node_id: U256,
    //     commit_weights: &mut Vec<OverwatchCommit<T::Hash>>
    // ) -> EvmResult<()> {
    //     handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
    //     let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;

    //     let origin = R::AddressMapping::into_account_id(handle.context().caller);
    //     let call = pallet_network::Call::<R>::reveal_overwatch_subnet_weights {
    //         subnet_id,
    //         overwatch_node_id,
    //         peer_id
    //     };

    //     RuntimeHelper::<R>::try_dispatch(
    //         handle,
    //         RawOrigin::Signed(origin.clone()).into(),
    //         call,
    //         0,
    //     )?;

    //     Ok(())
    // }

    #[precompile::public("addToOverwatchStake(uint256,address,uint256)")]
    #[precompile::payable]
    fn add_to_overwatch_stake(
        handle: &mut impl PrecompileHandle,
        overwatch_node_id: U256,
        hotkey: Address,
        stake_to_be_added: U256,
    ) -> EvmResult<()> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let stake_to_be_added = stake_to_be_added.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::add_to_overwatch_stake {
            overwatch_node_id,
            hotkey,
            stake_to_be_added,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("removeOverwatchStake(address,uint256)")]
    fn remove_overwatch_stake(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
        stake_to_be_removed: U256,
    ) -> EvmResult<()> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let stake_to_be_removed = stake_to_be_removed.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::remove_overwatch_stake {
            hotkey,
            stake_to_be_removed,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("accountOverwatchStake(address)")]
    #[precompile::view]
    fn account_overwatch_stake(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
    ) -> EvmResult<u128> {
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let account_stake: u128 = pallet_network::AccountOverwatchStake::<R>::get(hotkey);

        Ok(account_stake)
    }

    #[precompile::public("totalOverwatchStake()")]
    #[precompile::view]
    fn total_overwatch_stake(handle: &mut impl PrecompileHandle) -> EvmResult<u128> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let total_stake: u128 = pallet_network::TotalOverwatchStake::<R>::get();

        Ok(total_stake)
    }

    #[precompile::public("overwatchBlacklist(address)")]
    #[precompile::view]
    fn overwatch_blacklist(
        handle: &mut impl PrecompileHandle,
        coldkey: Address,
    ) -> EvmResult<bool> {
        let coldkey = R::AddressMapping::into_account_id(coldkey.into());

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let blacklisted: bool = pallet_network::OverwatchNodeBlacklist::<R>::get(coldkey);

        Ok(blacklisted)
    }

    #[precompile::public("maxOverwatchNodes()")]
    #[precompile::view]
    fn max_overwatch_nodes(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let max_overwatch_nodes = pallet_network::MaxOverwatchNodes::<R>::get();

        Ok(max_overwatch_nodes)
    }

    #[precompile::public("totalOverwatchNodes()")]
    #[precompile::view]
    fn total_overwatch_nodes(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let total_overwatch_nodes = pallet_network::TotalOverwatchNodes::<R>::get();

        Ok(total_overwatch_nodes)
    }

    #[precompile::public("totalOverwatchNodeUids()")]
    #[precompile::view]
    fn total_overwatch_node_uids(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let uids = pallet_network::TotalOverwatchNodeUids::<R>::get();

        Ok(uids)
    }

    #[precompile::public("overwatchEpochLengthMultiplier()")]
    #[precompile::view]
    fn overwatch_epoch_length_multiplier(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let epoch_multiplier = pallet_network::OverwatchEpochLengthMultiplier::<R>::get();

        Ok(epoch_multiplier)
    }

    #[precompile::public("overwatchCommitCutoffPercent()")]
    #[precompile::view]
    fn overwatch_commit_cutoff_percent(handle: &mut impl PrecompileHandle) -> EvmResult<u128> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let percent: u128 = pallet_network::OverwatchCommitCutoffPercent::<R>::get();

        Ok(percent)
    }

    // #[pallet::storage]
    // pub type OverwatchNodes<T: Config> =
    //     StorageMap<_, Identity, u32, OverwatchNode<T::AccountId>, OptionQuery>;

    // #[pallet::storage]
    // pub type OverwatchNodeIdHotkey<T: Config> =
    //     StorageMap<_, Identity, u32, T::AccountId, OptionQuery>;

    // #[pallet::storage]
    // pub type HotkeyOverwatchNodeId<T: Config> =
    //     StorageMap<_, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    // #[pallet::storage]
    // pub type PeerIdOverwatchNode<T> = StorageDoubleMap<
    //     _,
    //     Identity,
    //     u32,
    //     Blake2_128Concat,
    //     PeerId,
    //     u32,
    //     ValueQuery,
    //     DefaultZeroU32,
    // >;

    // #[pallet::storage]
    // pub type OverwatchNodeIndex<T> = StorageMap<
    //     _,
    //     Identity,
    //     u32, // overwatch_node_id
    //     BTreeMap<u32, PeerId>,
    //     ValueQuery,
    // >;

    // #[pallet::storage]
    // pub type OverwatchCommits<T: Config> = StorageNMap<
    //     _,
    //     (
    //         NMapKey<Identity, u32>, // Epoch
    //         NMapKey<Identity, u32>, // Overwatch ID
    //         NMapKey<Identity, u32>, // Subnet ID
    //     ),
    //     T::Hash, // Commit
    //     OptionQuery,
    // >;

    // #[pallet::storage]
    // pub type OverwatchReveals<T> = StorageNMap<
    //     _,
    //     (
    //         NMapKey<Identity, u32>, // Epoch
    //         NMapKey<Identity, u32>, // Subnet ID
    //         NMapKey<Identity, u32>, // Overwatch ID
    //     ),
    //     u128, // Reveal
    //     OptionQuery,
    // >;

    // #[pallet::storage]
    // pub type OverwatchNodePenalties<T> = StorageMap<_, Identity, u32, u32, OptionQuery>;

    // #[pallet::storage]
    // pub type MaxOverwatchNodePenalties<T> =
    //     StorageValue<_, u32, ValueQuery, DefaultMaxOverwatchNodePenalties>;

    // #[pallet::storage]
    // pub type OverwatchSubnetWeights<T> = StorageDoubleMap<
    //     _,
    //     Identity,
    //     u32, // Epoch
    //     Identity,
    //     u32,  // Subnet ID
    //     u128, // Weight
    //     OptionQuery,
    // >;

    // #[pallet::storage]
    // pub type OverwatchNodeWeights<T> = StorageDoubleMap<
    //     _,
    //     Identity,
    //     u32, // Epoch
    //     Identity,
    //     u32,  // Node ID
    //     u128, // Weight
    //     OptionQuery,
    // >;

    // #[pallet::storage]
    // pub type OverwatchMinDiversificationRatio<T> =
    //     StorageValue<_, u128, ValueQuery, DefaultOverwatchMinDiversificationRatio>;

    // #[pallet::storage]
    // pub type OverwatchMinRepScore<T> =
    //     StorageValue<_, u128, ValueQuery, DefaultOverwatchMinRepScore>;

    // #[pallet::storage]
    // pub type OverwatchMinAvgAttestationRatio<T> =
    //     StorageValue<_, u128, ValueQuery, DefaultOverwatchMinAvgAttestationRatio>;

    // #[pallet::storage]
    // pub type OverwatchMinAge<T> = StorageValue<_, u32, ValueQuery, DefaultOverwatchMinAge<T>>;

    // #[pallet::storage]
    // pub type OverwatchMinStakeBalance<T> =
    //     StorageValue<_, u128, ValueQuery, DefaultOverwatchMinStakeBalance>;
}

fn try_u256_to_u32(value: U256) -> Result<u32, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("u32 out of bounds".into()),
    })
}
