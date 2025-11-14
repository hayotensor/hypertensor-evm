use core::marker::PhantomData;
use fp_evm::Log;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::ConstU32;
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, ExitError, PrecompileFailure, PrecompileHandle};
use pallet_network::QueuedSwapCall;
use pallet_network::{OverwatchCommit, OverwatchReveal};
use precompile_utils::{EvmResult, prelude::*, solidity::Codec};
use sp_core::Decode;
use sp_core::Get;
use sp_core::{H160, H256, OpaquePeerId, U256};
use sp_runtime::SaturatedConversion;
use sp_runtime::Vec;
use sp_runtime::traits::{Dispatchable, StaticLookup, UniqueSaturatedInto};
use sp_std::vec;

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> =
    <Runtime as pallet_balances::Config<Instance>>::Balance;

pub(crate) struct OverwatchNodePrecompile<R>(PhantomData<R>);

impl<R> OverwatchNodePrecompile<R>
where
    R: frame_system::Config<Hash = H256> + pallet_evm::Config + pallet_network::Config,
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
    R: frame_system::Config<Hash = H256> + pallet_evm::Config + pallet_network::Config,
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

    // #[precompile::public("commitOverwatchSubnetWeights(uint256,uint256[],bytes32[])")]
    // fn commit_overwatch_subnet_weights(
    //     handle: &mut impl PrecompileHandle,
    //     overwatch_node_id: U256,
    //     subnet_ids: Vec<U256>,
    //     weight_hashes: Vec<H256>,
    // ) -> EvmResult<()> {
    //     handle.record_cost(RuntimeHelper::<R>::db_write_gas_cost())?;

    //     let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;

    //     // Validate arrays have same length
    //     if subnet_ids.len() != weight_hashes.len() {
    //         return Err(revert("subnet_ids and weight_hashes length mismatch"));
    //     }

    //     let mut commit_weights: Vec<OverwatchCommit<R::Hash>> = Vec::new();
    //     for (subnet_id_u256, weight_hash) in subnet_ids.iter().zip(weight_hashes.iter()) {
    //         let subnet_id = try_u256_to_u32(*subnet_id_u256)?;
    //         commit_weights.push(OverwatchCommit {
    //             subnet_id,
    //             weight: *weight_hash,
    //         });
    //     }

    //     // Get the caller
    //     let origin = R::AddressMapping::into_account_id(handle.context().caller);

    //     // Create the pallet call
    //     let call = pallet_network::Call::<R>::commit_overwatch_subnet_weights {
    //         overwatch_node_id,
    //         commit_weights,
    //     };

    //     // Dispatch the call
    //     RuntimeHelper::<R>::try_dispatch(
    //         handle,
    //         RawOrigin::Signed(origin.clone()).into(),
    //         call,
    //         0,
    //     )?;

    //     Ok(())
    // }

    #[precompile::public("commitOverwatchSubnetWeights(uint256,(uint256,bytes32)[])")]
    fn commit_overwatch_subnet_weights(
        handle: &mut impl PrecompileHandle,
        overwatch_node_id: U256,
        commits: Vec<(U256, H256)>,
    ) -> EvmResult {
        handle.record_cost(RuntimeHelper::<R>::db_write_gas_cost())?;

        let overwatch_node_id: u32 = try_u256_to_u32(overwatch_node_id)?;
        let commit_weights: Vec<OverwatchCommit<R::Hash>> = commits
            .into_iter()
            .map(|(subnet_id, weight)| {
                Ok::<_, PrecompileFailure>(OverwatchCommit::<R::Hash> {
                    subnet_id: try_u256_to_u32(subnet_id)?,
                    weight,
                })
            })
            .collect::<Result<_, _>>()?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let call = pallet_network::Call::<R>::commit_overwatch_subnet_weights {
            overwatch_node_id,
            commit_weights,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("revealOverwatchSubnetWeights(uint256,(uint256,uint256,uint8[])[])")]
    fn reveal_overwatch_subnet_weights(
        handle: &mut impl PrecompileHandle,
        overwatch_node_id: U256,
        reveals: Vec<(U256, U256, Vec<u8>)>,
    ) -> EvmResult {
        handle.record_cost(RuntimeHelper::<R>::db_write_gas_cost())?;

        let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;

        let reveals: Vec<OverwatchReveal> = reveals
            .into_iter()
            .map(|(subnet_id, weight, salt)| {
                Ok::<_, PrecompileFailure>(OverwatchReveal {
                    subnet_id: try_u256_to_u32(subnet_id)?,
                    weight: try_u256_to_u128(weight)?,
                    salt,
                })
            })
            .collect::<Result<_, _>>()?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let call = pallet_network::Call::<R>::reveal_overwatch_subnet_weights {
            overwatch_node_id,
            reveals: reveals,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

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

    #[precompile::public("overwatchNodeBlacklist(address)")]
    #[precompile::view]
    fn overwatch_node_blacklist(
        handle: &mut impl PrecompileHandle,
        coldkey: Address,
    ) -> EvmResult<bool> {
        let coldkey = R::AddressMapping::into_account_id(coldkey.into());

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let blacklisted = pallet_network::OverwatchNodeBlacklist::<R>::get(coldkey);

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

    #[precompile::public("overwatchNodes(uint256)")]
    #[precompile::view]
    fn overwatch_nodes(
        handle: &mut impl PrecompileHandle,
        overwatch_node_id: U256,
    ) -> EvmResult<(U256, Address)> {
        let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let overwatch_node = pallet_network::OverwatchNodes::<R>::get(overwatch_node_id)
            .ok_or(revert("Overwatch node not found"))?;

        // Convert AccountId to Address
        let hotkey = Address(sp_core::H160::from(overwatch_node.hotkey.into()));
        let overwatch_node_id = try_u32_to_u256(overwatch_node.id)?;

        Ok((overwatch_node_id, hotkey))
    }

    #[precompile::public("overwatchNodeIdHotkey(uint256)")]
    #[precompile::view]
    fn overwatch_node_id_hotkey(
        handle: &mut impl PrecompileHandle,
        overwatch_node_id: U256,
    ) -> EvmResult<Address> {
        let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let overwatch_node_hotkey =
            pallet_network::OverwatchNodeIdHotkey::<R>::get(overwatch_node_id)
                .ok_or(revert("Overwatch node ID hotkey not found"))?;

        // Convert AccountId to Address
        let hotkey = Address(sp_core::H160::from(overwatch_node_hotkey.into()));

        Ok(hotkey)
    }

    #[precompile::public("hotkeyOverwatchNodeId(address)")]
    #[precompile::view]
    fn hotkey_overwatch_node_id(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
    ) -> EvmResult<U256> {
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let overwatch_node_id = pallet_network::HotkeyOverwatchNodeId::<R>::get(hotkey)
            .ok_or(revert("Hotkey overwatch node ID not found"))?;

        let overwatch_node_id_u256 = try_u32_to_u256(overwatch_node_id)?;

        Ok(overwatch_node_id_u256)
    }

    #[precompile::public("peerIdOverwatchNode(uint256,string)")]
    #[precompile::view]
    fn peer_id_overwatch_node(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        peer_id: BoundedString<ConstU32<64>>,
    ) -> EvmResult<U256> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let peer_id = OpaquePeerId(peer_id.as_bytes().to_vec());

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let overwatch_node_id = pallet_network::PeerIdOverwatchNodeId::<R>::get(subnet_id, peer_id);

        let overwatch_node_id_u256 = try_u32_to_u256(overwatch_node_id)?;

        Ok(overwatch_node_id_u256)
    }

    #[precompile::public("overwatchCommits(uint256,uint256,uint256)")]
    #[precompile::view]
    fn overwatch_commits(
        handle: &mut impl PrecompileHandle,
        overwatch_epoch: U256,
        overwatch_node_id: U256,
        subnet_id: U256,
    ) -> EvmResult<H256> {
        let overwatch_epoch = try_u256_to_u32(overwatch_epoch)?;
        let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;
        let subnet_id = try_u256_to_u32(subnet_id)?;

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let commit = pallet_network::OverwatchCommits::<R>::get((
            overwatch_epoch,
            overwatch_node_id,
            subnet_id,
        ))
        .ok_or(revert("Peer ID overwatch node ID not found"))?;

        let hash_bytes = commit.as_ref();
        let commit_as_h256 = H256::from_slice(hash_bytes);

        Ok(commit_as_h256)
    }

    #[precompile::public("overwatchReveals(uint256,uint256,uint256)")]
    #[precompile::view]
    fn overwatch_reveals(
        handle: &mut impl PrecompileHandle,
        overwatch_epoch: U256,
        subnet_id: U256,
        overwatch_node_id: U256,
    ) -> EvmResult<U256> {
        let overwatch_epoch = try_u256_to_u32(overwatch_epoch)?;
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let reveal = pallet_network::OverwatchReveals::<R>::get((
            overwatch_epoch,
            subnet_id,
            overwatch_node_id,
        ))
        .ok_or(revert("Peer ID overwatch node ID not found"))?;

        let reveal_as_U256 = try_u128_to_u256(reveal)?;

        Ok(reveal_as_U256)
    }

    #[precompile::public("overwatchSubnetWeights(uint256,uint256)")]
    #[precompile::view]
    fn overwatch_subnet_weights(
        handle: &mut impl PrecompileHandle,
        overwatch_epoch: U256,
        subnet_id: U256,
    ) -> EvmResult<U256> {
        let overwatch_epoch = try_u256_to_u32(overwatch_epoch)?;
        let subnet_id = try_u256_to_u32(subnet_id)?;

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let overwatch_subnet_weight =
            pallet_network::OverwatchSubnetWeights::<R>::get(overwatch_epoch, subnet_id)
                .ok_or(revert("Overwatch subnet weights not found"))?;

        let overwatch_subnet_weight = try_u128_to_u256(overwatch_subnet_weight)?;

        Ok(overwatch_subnet_weight)
    }

    #[precompile::public("overwatchNodeWeights(uint256,uint256)")]
    #[precompile::view]
    fn overwatch_node_weights(
        handle: &mut impl PrecompileHandle,
        overwatch_epoch: U256,
        overwatch_node_id: U256,
    ) -> EvmResult<U256> {
        let overwatch_epoch = try_u256_to_u32(overwatch_epoch)?;
        let overwatch_node_id = try_u256_to_u32(overwatch_node_id)?;

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let overwatch_node_weight =
            pallet_network::OverwatchNodeWeights::<R>::get(overwatch_epoch, overwatch_node_id)
                .ok_or(revert("Overwatch node weights not found"))?;

        let overwatch_node_weight = try_u128_to_u256(overwatch_node_weight)?;

        Ok(overwatch_node_weight)
    }

    #[precompile::public("overwatchMinDiversificationRatio()")]
    #[precompile::view]
    fn overwatch_min_diversification_ratio(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let value = pallet_network::OverwatchMinDiversificationRatio::<R>::get();

        let value = try_u128_to_u256(value)?;

        Ok(value)
    }

    #[precompile::public("overwatchMinRepScore()")]
    #[precompile::view]
    fn overwatch_min_rep_score(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let value = pallet_network::OverwatchMinRepScore::<R>::get();

        let value = try_u128_to_u256(value)?;

        Ok(value)
    }

    #[precompile::public("overwatchMinAvgAttestationRatio()")]
    #[precompile::view]
    fn overwatch_min_avg_attestation_ratio(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let value = pallet_network::OverwatchMinAvgAttestationRatio::<R>::get();

        let value = try_u128_to_u256(value)?;

        Ok(value)
    }

    #[precompile::public("overwatchMinAge()")]
    #[precompile::view]
    fn overwatch_min_age(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let value = pallet_network::OverwatchMinAge::<R>::get();

        let value = try_u32_to_u256(value)?;

        Ok(value)
    }

    #[precompile::public("overwatchMinStakeBalance()")]
    #[precompile::view]
    fn overwatch_min_stake_balance(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let value: u128 = pallet_network::OverwatchMinStakeBalance::<R>::get();

        let value = try_u128_to_u256(value)?;

        Ok(value)
    }

    #[precompile::public("getCurrentOverwatchEpoch()")]
    #[precompile::view]
    fn get_current_overwatch_epoch_as_u32(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let current_block: u32 = frame_system::Pallet::<R>::block_number().saturated_into::<u32>();
        let epoch_length: u32 = R::EpochLength::get();
        let multiplier = pallet_network::OverwatchEpochLengthMultiplier::<R>::get();
        let overwatch_epoch = current_block.saturating_div(epoch_length.saturating_mul(multiplier));

        let value = try_u32_to_u256(overwatch_epoch)?;

        Ok(value)
    }
}

fn try_u256_to_u32(value: U256) -> Result<u32, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("u32 out of bounds".into()),
    })
}

fn try_u256_to_u128(value: U256) -> Result<u128, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("u128 out of bounds".into()),
    })
}

fn try_u32_to_u256(value: u32) -> Result<U256, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("u32 out of bounds".into()),
    })
}

fn try_u128_to_u256(value: u128) -> Result<U256, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("u128 out of bounds".into()),
    })
}
