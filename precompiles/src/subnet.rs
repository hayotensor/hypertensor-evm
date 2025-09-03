use pallet_evm::{AddressMapping, ExitError, PrecompileFailure, PrecompileHandle};
use precompile_utils::{EvmResult, prelude::*};
use sp_core::{H160, H256, OpaquePeerId, U256};
use sp_runtime::{
    Vec,
    traits::{Dispatchable, StaticLookup, UniqueSaturatedInto},
};
use sp_std::vec;
// use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::traits::ConstU32;
use frame_support::{
    dispatch::{GetDispatchInfo, PostDispatchInfo},
    storage::bounded_vec::BoundedVec,
};
use frame_system::RawOrigin;
use pallet_network::{KeyType, NodeRemovalSystem, RegistrationSubnetData};
use sp_std::collections::btree_set::BTreeSet;

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> =
    <Runtime as pallet_balances::Config<Instance>>::Balance;

pub(crate) struct SubnetPrecompile<R>(PhantomData<R>);

impl<R> SubnetPrecompile<R>
where
    // R: frame_system::Config
    //   + pallet_evm::Config
    //   + pallet_network::Config
    //   + pallet_balances::Config,
    // <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
    //   + From<pallet_balances::Call<R>>
    //   + GetDispatchInfo
    //   + Dispatchable<PostInfo = PostDispatchInfo>,
    // <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    // <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    // <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
    R: frame_system::Config + pallet_evm::Config + pallet_network::Config,
    R::AccountId: From<[u8; 20]> + Into<[u8; 20]>,
    <R as frame_system::Config>::RuntimeCall:
        From<pallet_network::Call<R>> + GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    pub const HASH_N: u64 = 2049;
}

#[precompile_utils::precompile]
impl<R> SubnetPrecompile<R>
where
    // R: frame_system::Config
    //   + pallet_evm::Config
    //   + pallet_network::Config
    //   + pallet_balances::Config,
    // <R as frame_system::Config>::RuntimeCall: From<pallet_network::Call<R>>
    //   + From<pallet_balances::Call<R>>
    //   + GetDispatchInfo
    //   + Dispatchable<PostInfo = PostDispatchInfo>,
    // <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    // <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    // <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
    R: frame_system::Config + pallet_evm::Config + pallet_network::Config,
    R::AccountId: From<[u8; 20]> + Into<[u8; 20]>,
    <R as frame_system::Config>::RuntimeCall:
        From<pallet_network::Call<R>> + GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    // #[precompile::public("registerSubnet(RegistrationSubnetData)")]
    // #[precompile::payable]
    // fn register_subnet(
    //   handle: &mut impl PrecompileHandle,
    //   subnet_data: RegistrationSubnetData,
    // ) -> EvmResult<()> {
    //   let origin = R::AddressMapping::into_account_id(handle.context().caller);
    //   let call = pallet_network::Call::<R>::register_subnet {
    //     subnet_data
    //   };

    //   RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    //   Ok(())
    // }

    // #[precompile::public("registerSubnet(address,string,string,string,string,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,address[],uint256,uint256,uint256[])")]
    // #[precompile::payable]
    // fn register_subnet(
    //   handle: &mut impl PrecompileHandle,
    //   hotkey: Address,
    //   name: BoundedString<ConstU32<256>>,
    //   repo: BoundedString<ConstU32<1024>>,
    //   description: BoundedString<ConstU32<1024>>,
    //   misc: BoundedString<ConstU32<1024>>,
    // 	churn_limit: U256,
    //   min_stake: U256,
    //   max_stake: U256,
    //   delegate_stake_percentage: U256,
    // 	registration_queue_epochs: U256,
    // 	activation_grace_epochs: U256,
    // 	queue_classification_epochs: U256,
    // 	included_classification_epochs: U256,
    // 	max_node_penalties: U256,
    // 	initial_coldkeys: Vec<Address>,
    //   max_registered_nodes: U256,
    //   node_removal_system: U256,
    //   key_types: Vec<U256>,
    // ) -> EvmResult<()> {
    //   handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

    //   let hotkey = R::AddressMapping::into_account_id(hotkey.into());
    //   let churn_limit = try_u256_to_u32(churn_limit)?;
    //   let min_stake: u128 = min_stake.unique_saturated_into();
    //   let max_stake: u128 = max_stake.unique_saturated_into();
    //   let delegate_stake_percentage: u128 = delegate_stake_percentage.unique_saturated_into();
    //   let registration_queue_epochs = try_u256_to_u32(registration_queue_epochs)?;
    //   let activation_grace_epochs = try_u256_to_u32(activation_grace_epochs)?;
    //   let queue_classification_epochs = try_u256_to_u32(queue_classification_epochs)?;
    //   let included_classification_epochs = try_u256_to_u32(included_classification_epochs)?;
    //   let max_node_penalties = try_u256_to_u32(max_node_penalties)?;
    //   let initial_coldkeys: BTreeSet<R::AccountId> = initial_coldkeys
    //     .into_iter()
    //     .map(|a| R::AddressMapping::into_account_id(a.into()))
    //     .collect();
    //   let max_registered_nodes = try_u256_to_u32(max_registered_nodes)?;

    //   // let node_removal_system = node_removal_from_u256(node_removal_system)
    //   //   .ok_or_else(|| revert("Invalid NodeRemovalSystem value"))?;

    //   let key_types: BTreeSet<KeyType> = key_types
    //     .into_iter()
    //     .map(|val| key_type_from_u256(val).ok_or_else(|| revert("Invalid KeyType value")))
    //     .collect::<Result<_, _>>()?;

    //   let subnet_data = pallet_network::RegistrationSubnetData::<R::AccountId> {
    //     name: name.into(),
    //     repo: repo.into(),
    // 		description: description.into(),
    // 		misc: misc.into(),
    //     min_stake,
    //     max_stake,
    //     delegate_stake_percentage,
    //     churn_limit,
    //     registration_queue_epochs,
    //     activation_grace_epochs,
    //     queue_classification_epochs,
    //     included_classification_epochs,
    //     max_node_penalties,
    //     initial_coldkeys,
    //     max_registered_nodes,
    //     // node_removal_system: None,
    //     key_types
    //   };

    //   let origin = R::AddressMapping::into_account_id(handle.context().caller);
    //   let call = pallet_network::Call::<R>::register_subnet {
    //     hotkey,
    //     subnet_data
    //   };

    //   RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 0)?;

    //   Ok(())
    // }

    #[precompile::public("canSubnetRegister(uint256)")]
    #[precompile::view]
    fn can_subnet_register(handle: &mut impl PrecompileHandle, epoch: U256) -> EvmResult<bool> {
        let epoch = try_u256_to_u32(epoch)?;
        let can_register = pallet_network::Pallet::<R>::can_subnet_register(epoch);
        Ok(can_register)
    }

    #[precompile::public("registrationCost(uint256)")]
    #[precompile::view]
    fn registration_cost(handle: &mut impl PrecompileHandle, epoch: U256) -> EvmResult<u128> {
        let epoch = try_u256_to_u32(epoch)?;
        let cost = pallet_network::Pallet::<R>::registration_cost(epoch);
        Ok(cost)
    }

    #[precompile::public("activateSubnet(uint256)")]
    #[precompile::payable]
    fn activate_subnet(handle: &mut impl PrecompileHandle, subnet_id: U256) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::activate_subnet { subnet_id };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            148,
        )?;

        Ok(())
    }

    #[precompile::public("removeSubnet(uint256)")]
    #[precompile::payable]
    fn remove_subnet(handle: &mut impl PrecompileHandle, subnet_id: U256) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::remove_subnet { subnet_id };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            148,
        )?;

        Ok(())
    }

    #[precompile::public("ownerDeactivateSubnet(uint256)")]
    #[precompile::payable]
    fn owner_deactivate_subnet(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_deactivate_subnet { subnet_id };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            148,
        )?;

        Ok(())
    }

    // #[precompile::public("ownerUpdateRegistrationInterval(uint256,uint256)")]
    // #[precompile::payable]
    // fn owner_update_registration_interval(
    //   handle: &mut impl PrecompileHandle,
    //   subnet_id: U256,
    //   value: U256,
    // ) -> EvmResult<()> {
    //   let subnet_id = try_u256_to_u32(subnet_id)?;
    //   let value = try_u256_to_u32(value)?;

    //   let origin = R::AddressMapping::into_account_id(handle.context().caller);
    //   let call = pallet_network::Call::<R>::owner_update_registration_interval {
    //     subnet_id,
    //     value
    //   };

    //   RuntimeHelper::<R>::try_dispatch(handle, RawOrigin::Signed(origin.clone()).into(), call, 148)?;

    //   Ok(())
    // }

    #[precompile::public("ownerRemoveSubnetNode(uint256,uint256)")]
    #[precompile::payable]
    fn owner_remove_subnet_node(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_remove_subnet_node {
            subnet_id,
            subnet_node_id,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            148,
        )?;

        Ok(())
    }

    #[precompile::public("getSubnetId(string)")]
    #[precompile::view]
    fn get_subnet_id(
        handle: &mut impl PrecompileHandle,
        name: BoundedString<ConstU32<256>>,
    ) -> EvmResult<u32> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let subnet_id = match pallet_network::SubnetName::<R>::try_get::<Vec<u8>>(name.into()) {
            Ok(subnet_id) => subnet_id,
            Err(()) => 0,
        };

        Ok(subnet_id)
    }

    #[precompile::public("addSubnetNode(uint256,address,string,string,string,uint256,uint256)")]
    #[precompile::payable]
    fn add_subnet_node(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        hotkey: Address,
        peer_id: BoundedString<ConstU32<64>>,
        bootnode_peer_id: BoundedString<ConstU32<64>>,
        client_peer_id: BoundedString<ConstU32<64>>,
        delegate_reward_rate: U256,
        stake_to_be_added: U256,
        // unique: BoundedVec<u8, DefaultMaxVectorLength>,
        // non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let peer_id = OpaquePeerId(peer_id.as_bytes().to_vec());
        let bootnode_peer_id = OpaquePeerId(bootnode_peer_id.as_bytes().to_vec());
        let client_peer_id = OpaquePeerId(client_peer_id.as_bytes().to_vec());
        let delegate_reward_rate: u128 = delegate_reward_rate.unique_saturated_into();
        let stake_to_be_added: u128 = stake_to_be_added.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let call = pallet_network::Call::<R>::add_subnet_node {
            subnet_id,
            hotkey,
            peer_id,
            bootnode_peer_id,
            client_peer_id,
            bootnode: None,
            delegate_reward_rate,
            stake_to_be_added,
            unique: None,
            non_unique: None,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            148,
        )?;

        Ok(())
    }

    #[precompile::public("registerSubnetNode(uint256,address,string,string,string,uint256,uint256)")]
    #[precompile::payable]
    fn register_subnet_node(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        hotkey: Address,
        peer_id: BoundedString<ConstU32<64>>,
        bootnode_peer_id: BoundedString<ConstU32<64>>,
        client_peer_id: BoundedString<ConstU32<64>>,
        delegate_reward_rate: U256,
        stake_to_be_added: U256,
        // unique: BoundedVec<u8, DefaultMaxVectorLength>,
        // non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let peer_id = OpaquePeerId(peer_id.as_bytes().to_vec());
        let bootnode_peer_id = OpaquePeerId(bootnode_peer_id.as_bytes().to_vec());
        let client_peer_id = OpaquePeerId(client_peer_id.as_bytes().to_vec());
        let delegate_reward_rate: u128 = delegate_reward_rate.unique_saturated_into();
        let stake_to_be_added: u128 = stake_to_be_added.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let call = pallet_network::Call::<R>::register_subnet_node {
            subnet_id,
            hotkey,
            peer_id,
            bootnode_peer_id,
            client_peer_id,
            bootnode: None,
            delegate_reward_rate,
            stake_to_be_added,
            unique: None,
            non_unique: None,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            148,
        )?;

        Ok(())
    }

    #[precompile::public("activateSubnetNode(uint256,uint256)")]
    #[precompile::payable]
    fn activate_subnet_node(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::activate_subnet_node {
            subnet_id,
            subnet_node_id,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            148,
        )?;

        Ok(())
    }

    #[precompile::public("deactivateSubnetNode(uint256,uint256)")]
    #[precompile::payable]
    fn deactivate_subnet_node(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::deactivate_subnet_node {
            subnet_id,
            subnet_node_id,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            148,
        )?;

        Ok(())
    }

    #[precompile::public("removeSubnetNode(uint256,uint256)")]
    #[precompile::payable]
    fn remove_subnet_node(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::remove_subnet_node {
            subnet_id,
            subnet_node_id,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            148,
        )?;

        Ok(())
    }

    #[precompile::public("updateDelegateRewardRate(uint256,uint256,uint256)")]
    #[precompile::payable]
    fn update_delegate_reward_rate(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        new_delegate_reward_rate: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
        let new_delegate_reward_rate = new_delegate_reward_rate.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_delegate_reward_rate {
            subnet_id,
            subnet_node_id,
            new_delegate_reward_rate,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            148,
        )?;

        Ok(())
    }
}

fn try_u256_to_u32(value: U256) -> Result<u32, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("u32 out of bounds".into()),
    })
}

fn node_removal_from_u256(val: U256) -> Option<NodeRemovalSystem> {
    match val.as_u32() {
        0 => Some(NodeRemovalSystem::Consensus),
        1 => Some(NodeRemovalSystem::Stake),
        2 => Some(NodeRemovalSystem::Reputation),
        _ => None,
    }
}

fn key_type_from_u256(val: U256) -> Option<KeyType> {
    match val.as_u32() {
        0 => Some(KeyType::Rsa),
        1 => Some(KeyType::Ed25519),
        2 => Some(KeyType::Secp256k1),
        3 => Some(KeyType::Ecdsa),
        _ => None,
    }
}
