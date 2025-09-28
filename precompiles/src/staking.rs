use core::marker::PhantomData;
use fp_account::AccountId20;
use fp_evm::Log;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, ExitError, PrecompileFailure, PrecompileHandle};
use pallet_network::QueuedSwapCall;
use precompile_utils::{EvmResult, prelude::*, solidity::Codec};
use sp_core::Decode;
use sp_core::{H160, H256, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup, UniqueSaturatedInto};
use sp_std::vec;

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> =
    <Runtime as pallet_balances::Config<Instance>>::Balance;

pub(crate) struct StakingPrecompile<R>(PhantomData<R>);

impl<R> StakingPrecompile<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_network::Config,
    R::AccountId: From<[u8; 20]> + Into<[u8; 20]>,
    <R as frame_system::Config>::RuntimeCall:
        From<pallet_network::Call<R>> + GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    pub const HASH_N: u64 = 2048;
}

#[precompile_utils::precompile]
impl<R> StakingPrecompile<R>
where
    R: frame_system::Config + pallet_evm::Config + pallet_network::Config,
    R::AccountId: From<[u8; 20]> + Into<[u8; 20]>,
    <R as frame_system::Config>::RuntimeCall:
        From<pallet_network::Call<R>> + GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    #[precompile::public("addToStake(uint256,uint256,address,uint256)")]
    #[precompile::payable]
    fn add_stake(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        hotkey: Address,
        stake_to_be_added: U256,
    ) -> EvmResult<()> {
        let stake_to_be_added = stake_to_be_added.unique_saturated_into();
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::add_stake {
            subnet_id,
            subnet_node_id,
            hotkey,
            stake_to_be_added,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;
        // RuntimeHelper::<R>::try_dispatch(handle, Some(origin.clone()).into(), call, 0)?;

        Ok(())
    }

    #[precompile::public("removeStake(uint256,address,uint256)")]
    #[precompile::payable]
    fn remove_stake(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        hotkey: Address,
        stake_to_be_removed: U256,
    ) -> EvmResult<()> {
        let stake_to_be_removed = stake_to_be_removed.unique_saturated_into();
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::remove_stake {
            subnet_id,
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

    #[precompile::public("claimUnbondings()")]
    #[precompile::payable]
    fn claim_unbondings(handle: &mut impl PrecompileHandle) -> EvmResult<()> {
        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::claim_unbondings {};

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("addToDelegateStake(uint256,uint256)")]
    #[precompile::payable]
    fn add_to_delegate_stake(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        stake_to_be_added: U256,
    ) -> EvmResult<()> {
        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let subnet_id = try_u256_to_u32(subnet_id)?;
        let stake_to_be_added: u128 = stake_to_be_added.unique_saturated_into();

        let value_sent: U256 = handle.context().apparent_value;
        let value_balance: u128 = value_sent.unique_saturated_into();

        let call = pallet_network::Call::<R>::add_to_delegate_stake {
            subnet_id,
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

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("transferDelegateStake(uint256,address,uint256)")]
    #[precompile::payable]
    fn transfer_delegate_stake(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        to_account_id: Address,
        delegate_stake_shares_to_transfer: U256,
    ) -> EvmResult<()> {
        let delegate_stake_shares_to_transfer =
            delegate_stake_shares_to_transfer.unique_saturated_into();
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let to_account_id = R::AddressMapping::into_account_id(to_account_id.into());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::transfer_delegate_stake {
            subnet_id,
            to_account_id,
            delegate_stake_shares_to_transfer,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("removeDelegateStake(uint256,uint256)")]
    #[precompile::payable]
    fn remove_delegate_stake(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        shares_to_be_removed: U256,
    ) -> EvmResult<()> {
        let shares_to_be_removed = shares_to_be_removed.unique_saturated_into();
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::remove_delegate_stake {
            subnet_id,
            shares_to_be_removed,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("increaseDelegateStake(uint256,uint256)")]
    #[precompile::payable]
    fn donate_delegate_stake(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        amount: U256,
    ) -> EvmResult<()> {
        let amount = amount.unique_saturated_into();
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::donate_delegate_stake { subnet_id, amount };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

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
        let node_delegate_stake_to_be_added =
            node_delegate_stake_to_be_added.unique_saturated_into();
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::add_to_node_delegate_stake {
            subnet_id,
            subnet_node_id,
            node_delegate_stake_to_be_added,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("swapNodeDelegateStake(uint256,uint256,uint256,uint256,uint256)")]
    #[precompile::payable]
    fn swap_node_delegate_stake(
        handle: &mut impl PrecompileHandle,
        from_subnet_id: U256,
        from_subnet_node_id: U256,
        to_subnet_id: U256,
        to_subnet_node_id: U256,
        node_delegate_stake_shares_to_swap: U256,
    ) -> EvmResult<()> {
        let node_delegate_stake_shares_to_swap =
            node_delegate_stake_shares_to_swap.unique_saturated_into();
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

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("transferNodeDelegateStake(uint256,uint256,address,uint256)")]
    #[precompile::payable]
    fn transfer_node_delegate_stake(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        to_account_id: Address,
        node_delegate_stake_shares_to_transfer: U256,
    ) -> EvmResult<()> {
        let node_delegate_stake_shares_to_transfer =
            node_delegate_stake_shares_to_transfer.unique_saturated_into();
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
        let to_account_id = R::AddressMapping::into_account_id(to_account_id.into());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::transfer_node_delegate_stake {
            subnet_id,
            subnet_node_id,
            to_account_id,
            node_delegate_stake_shares_to_transfer,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

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
        let node_delegate_stake_shares_to_be_removed =
            node_delegate_stake_shares_to_be_removed.unique_saturated_into();
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::remove_node_delegate_stake {
            subnet_id,
            subnet_node_id,
            node_delegate_stake_shares_to_be_removed,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("increaseNodeDelegateStake(uint256,uint256,uint256)")]
    #[precompile::payable]
    fn donate_node_delegate_stake(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        amount: U256,
    ) -> EvmResult<()> {
        let amount = amount.unique_saturated_into();
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::donate_node_delegate_stake {
            subnet_id,
            subnet_node_id,
            amount,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("transferFromNodeToSubnet(uint256,uint256,uint256,uint256)")]
    #[precompile::payable]
    fn swap_from_node_to_subnet(
        handle: &mut impl PrecompileHandle,
        from_subnet_id: U256,
        from_subnet_node_id: U256,
        to_subnet_id: U256,
        node_delegate_stake_shares_to_swap: U256,
    ) -> EvmResult<()> {
        let node_delegate_stake_shares_to_swap =
            node_delegate_stake_shares_to_swap.unique_saturated_into();
        let from_subnet_id = try_u256_to_u32(from_subnet_id)?;
        let from_subnet_node_id = try_u256_to_u32(from_subnet_node_id)?;
        let to_subnet_id = try_u256_to_u32(to_subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::swap_from_node_to_subnet {
            from_subnet_id,
            from_subnet_node_id,
            to_subnet_id,
            node_delegate_stake_shares_to_swap,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("transferFromSubnetToNode(uint256,uint256,uint256,uint256)")]
    #[precompile::payable]
    fn swap_from_subnet_to_node(
        handle: &mut impl PrecompileHandle,
        from_subnet_id: U256,
        to_subnet_id: U256,
        to_subnet_node_id: U256,
        delegate_stake_shares_to_swap: U256,
    ) -> EvmResult<()> {
        let delegate_stake_shares_to_swap = delegate_stake_shares_to_swap.unique_saturated_into();
        let from_subnet_id = try_u256_to_u32(from_subnet_id)?;
        let to_subnet_id = try_u256_to_u32(to_subnet_id)?;
        let to_subnet_node_id = try_u256_to_u32(to_subnet_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::swap_from_subnet_to_node {
            from_subnet_id,
            to_subnet_id,
            to_subnet_node_id,
            delegate_stake_shares_to_swap,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updateSwapQueue(uint256,uint256,uint256,uint256)")]
    #[precompile::payable]
    fn update_swap_queue(
        handle: &mut impl PrecompileHandle,
        id: U256,
        call_type: U256,
        to_subnet_id: U256,
        to_subnet_node_id: U256,
    ) -> EvmResult<()> {
        let id = try_u256_to_u32(id)?;
        let call_type = try_u256_to_u32(call_type)?;
        let to_subnet_id = try_u256_to_u32(to_subnet_id)?;
        let to_subnet_node_id = try_u256_to_u32(to_subnet_node_id)?;
        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let new_call = match call_type {
            0 => QueuedSwapCall::SwapToSubnetDelegateStake {
                account_id: origin.clone(),
                to_subnet_id,
                balance: 0,
            },
            1 => QueuedSwapCall::SwapToNodeDelegateStake {
                account_id: origin.clone(),
                to_subnet_id,
                to_subnet_node_id,
                balance: 0,
            },
            _ => {
                return Err(revert(
                    "Invalid call type. Must be 0 (SwapToSubnetDelegateStake) or 1 (SwapToNodeDelegateStake)",
                ));
            }
        };

        let call = pallet_network::Call::<R>::update_swap_queue { id, new_call };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("getQueuedSwapCall(uint256)")]
    #[precompile::view]
    fn get_queued_swap_call(
        handle: &mut impl PrecompileHandle, 
        queue_id: U256
    ) -> EvmResult<(u32, Address, u8, u32, u32, u128, u32, u32)> {
        // Returns: (id, account_id, call_type, to_subnet_id, to_subnet_node_id, balance, queued_at_block, execute_after_blocks)
        let queue_id = try_u256_to_u32(queue_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        
        let queued_item = pallet_network::SwapCallQueue::<R>::get(queue_id);
        
        match queued_item {
            Some(item) => {
                let (call_type, account_id, to_subnet_id, to_subnet_node_id, balance) = match &item.call {
                    QueuedSwapCall::SwapToSubnetDelegateStake { 
                        account_id, 
                        to_subnet_id, 
                        balance 
                    } => {
                        (0u8, account_id, *to_subnet_id, 0u32, *balance)
                    },
                    QueuedSwapCall::SwapToNodeDelegateStake { 
                        account_id, 
                        to_subnet_id, 
                        to_subnet_node_id, 
                        balance 
                    } => {
                        (1u8, account_id, *to_subnet_id, *to_subnet_node_id, *balance)
                    }
                };
                
                let account_address = Address(sp_core::H160::from((account_id.clone()).into()));

                Ok((
                    item.id,                    // id
                    account_address,            // account_id (as Address)
                    call_type,                  // type (0=subnet, 1=node)
                    to_subnet_id,               // to_subnet_id
                    to_subnet_node_id,          // to_subnet_node_id (0 for subnet swaps)
                    balance,                    // balance
                    item.queued_at_block,       // queued_at_block
                    item.execute_after_blocks   // execute_after_blocks
                ))
            },
            None => Err(revert("Queue item not found"))
        }
    }

    #[precompile::public("totalSubnetStake(uint256)")]
    #[precompile::view]
    fn total_subnet_stake(handle: &mut impl PrecompileHandle, subnet_id: U256) -> EvmResult<u128> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let total_subnet_stake: u128 = pallet_network::TotalSubnetStake::<R>::get(subnet_id);

        Ok(total_subnet_stake)
    }

    #[precompile::public("accountSubnetStake(address,uint256)")]
    #[precompile::view]
    fn account_subnet_stake(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
        subnet_id: U256,
    ) -> EvmResult<u128> {
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let subnet_id = try_u256_to_u32(subnet_id)?;

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let account_subnet_stake: u128 =
            pallet_network::AccountSubnetStake::<R>::get(&hotkey, subnet_id);

        Ok(account_subnet_stake)
    }

    #[precompile::public("totalSubnetDelegateStakeBalance(uint256)")]
    #[precompile::view]
    fn total_subnet_delegate_stake_balance(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<u128> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let total_subnet_delegate_stake_balance: u128 =
            pallet_network::TotalSubnetDelegateStakeBalance::<R>::get(subnet_id);

        Ok(total_subnet_delegate_stake_balance)
    }

    #[precompile::public("totalSubnetDelegateStakeShares(uint256)")]
    #[precompile::view]
    fn total_subnet_delegate_stake_shares(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<u128> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let total_subnet_delegate_stake_shares: u128 =
            pallet_network::TotalSubnetDelegateStakeBalance::<R>::get(subnet_id);

        Ok(total_subnet_delegate_stake_shares)
    }

    #[precompile::public("accountSubnetDelegateStakeShares(address,uint256)")]
    #[precompile::view]
    fn account_subnet_delegate_stake_shares(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
        subnet_id: U256,
    ) -> EvmResult<u128> {
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let account_subnet_delegate_stake_shares: u128 =
            pallet_network::AccountSubnetDelegateStakeShares::<R>::get(&hotkey, subnet_id);
        Ok(account_subnet_delegate_stake_shares)
    }

    #[precompile::public("accountSubnetDelegateStakeBalance(address,uint256)")]
    #[precompile::view]
    fn account_subnet_delegate_stake_balance(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
        subnet_id: U256,
    ) -> EvmResult<u128> {
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());

        let subnet_id = try_u256_to_u32(subnet_id)?;

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let account_delegate_stake_shares: u128 =
            pallet_network::AccountSubnetDelegateStakeShares::<R>::get(&hotkey, subnet_id);
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let total_subnet_delegated_stake_shares =
            pallet_network::TotalSubnetDelegateStakeShares::<R>::get(subnet_id);
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let total_subnet_delegated_stake_balance =
            pallet_network::TotalSubnetDelegateStakeBalance::<R>::get(subnet_id);

        let balance: u128 = pallet_network::Pallet::<R>::convert_to_balance(
            account_delegate_stake_shares,
            total_subnet_delegated_stake_shares,
            total_subnet_delegated_stake_balance,
        );

        Ok(balance)
    }

    #[precompile::public("accountNodeDelegateStakeShares(address,uint256,uint256)")]
    #[precompile::view]
    fn account_node_delegate_stake_shares(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
        subnet_id: U256,
        subnet_node_id: U256,
    ) -> EvmResult<u128> {
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let account_node_delegate_stake_shares: u128 =
            pallet_network::AccountNodeDelegateStakeShares::<R>::get((
                &hotkey,
                subnet_id,
                subnet_node_id,
            ));
        Ok(account_node_delegate_stake_shares)
    }

    #[precompile::public("accountNodeDelegateStakeBalance(address,uint256,uint256)")]
    #[precompile::view]
    fn account_node_delegate_stake_balance(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
        subnet_id: U256,
        subnet_node_id: U256,
    ) -> EvmResult<u128> {
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());

        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let account_node_delegate_stake_shares: u128 =
            pallet_network::AccountNodeDelegateStakeShares::<R>::get((
                &hotkey,
                subnet_id,
                subnet_node_id,
            ));
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let total_node_delegated_stake_shares =
            pallet_network::TotalNodeDelegateStakeShares::<R>::get(subnet_id, subnet_node_id);
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;
        let total_node_delegated_stake_balance =
            pallet_network::NodeDelegateStakeBalance::<R>::get(subnet_id, subnet_node_id);

        let balance: u128 = pallet_network::Pallet::<R>::convert_to_balance(
            account_node_delegate_stake_shares,
            total_node_delegated_stake_shares,
            total_node_delegated_stake_balance,
        );

        Ok(balance)
    }
}

fn try_u256_to_u32(value: U256) -> Result<u32, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("u32 out of bounds".into()),
    })
}
