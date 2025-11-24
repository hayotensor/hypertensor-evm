use core::marker::PhantomData;
use frame_support::traits::ConstU32;
use frame_support::{
    dispatch::{GetDispatchInfo, PostDispatchInfo},
    storage::bounded_vec::BoundedVec,
};
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, ExitError, PrecompileFailure, PrecompileHandle};
use pallet_network::{
    DefaultMaxSocialIdLength, DefaultMaxUrlLength, DefaultMaxVectorLength, KeyType,
};
use precompile_utils::{EvmResult, prelude::*};
use sp_core::{H160, H256, OpaquePeerId, U256};
use sp_runtime::{
    Vec,
    traits::{Dispatchable, StaticLookup, UniqueSaturatedInto},
};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::vec;

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> =
    <Runtime as pallet_balances::Config<Instance>>::Balance;

pub(crate) struct SubnetPrecompile<R>(PhantomData<R>);

impl<R> SubnetPrecompile<R>
where
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
    R: frame_system::Config + pallet_evm::Config + pallet_network::Config,
    R::AccountId: From<[u8; 20]> + Into<[u8; 20]>,
    <R as frame_system::Config>::RuntimeCall:
        From<pallet_network::Call<R>> + GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    #[precompile::public(
        "registerSubnet(uint256,string,string,string,string,uint256,uint256,uint256,(address,uint256)[],uint256[],string[])"
    )]
    #[precompile::payable]
    fn register_subnet(
        handle: &mut impl PrecompileHandle,
        max_cost: U256,
        name: BoundedString<ConstU32<256>>,
        repo: BoundedString<ConstU32<1024>>,
        description: BoundedString<ConstU32<1024>>,
        misc: BoundedString<ConstU32<1024>>,
        min_stake: U256,
        max_stake: U256,
        delegate_stake_percentage: U256,
        initial_coldkeys: Vec<(Address, U256)>,
        key_types: Vec<U256>,
        bootnodes: Vec<BoundedString<ConstU32<1024>>>,
    ) -> EvmResult<()> {
        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let max_cost: u128 = max_cost.unique_saturated_into();
        let min_stake: u128 = min_stake.unique_saturated_into();
        let max_stake: u128 = max_stake.unique_saturated_into();
        let delegate_stake_percentage: u128 = delegate_stake_percentage.unique_saturated_into();
        let initial_coldkeys: BTreeMap<R::AccountId, u32> = initial_coldkeys
            .into_iter()
            .map(|(addr, count)| {
                Ok::<_, PrecompileFailure>((
                    R::AddressMapping::into_account_id(addr.into()),
                    try_u256_to_u32(count)?,
                ))
            })
            .collect::<Result<_, _>>()?;
        let key_types: BTreeSet<KeyType> = key_types
            .into_iter()
            .map(|val| key_type_from_u256(val).ok_or_else(|| revert("Invalid KeyType value")))
            .collect::<Result<_, _>>()?;
        let bootnodes: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>> = bootnodes
            .into_iter()
            .map(|bootnode_string| {
                let bootnode_bytes = bootnode_string.as_bytes();
                BoundedVec::try_from(bootnode_bytes.to_vec())
                    .map_err(|_| revert("Bootnode address too long"))
            })
            .collect::<Result<_, _>>()?;

        let subnet_data = pallet_network::RegistrationSubnetData {
            name: name.into(),
            repo: repo.into(),
            description: description.into(),
            misc: misc.into(),
            min_stake,
            max_stake,
            delegate_stake_percentage,
            initial_coldkeys,
            key_types,
            bootnodes,
        };

        let call = pallet_network::Call::<R>::register_subnet {
            max_cost,
            subnet_data: subnet_data,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("getCurrentRegistrationCost(uint256)")]
    #[precompile::view]
    fn get_current_registration_cost(
        handle: &mut impl PrecompileHandle,
        block: U256,
    ) -> EvmResult<u128> {
        let block = try_u256_to_u32(block)?;
        let cost = pallet_network::Pallet::<R>::get_current_registration_cost(block);
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
            0,
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

    #[precompile::public("getMinSubnetDelegateStakeBalance(uint256)")]
    #[precompile::view]
    fn get_min_subnet_delegate_stake_balance(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<u128> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let subnet_id = try_u256_to_u32(subnet_id)?;

        let result = pallet_network::Pallet::<R>::get_min_subnet_delegate_stake_balance(subnet_id);

        Ok(result)
    }

    #[precompile::public(
        "registerSubnetNode(uint256,address,string,string,string,string,uint256,uint256,string,string,uint256)"
    )]
    #[precompile::payable]
    fn register_subnet_node(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        hotkey: Address,
        peer_id: BoundedString<ConstU32<64>>,
        bootnode_peer_id: BoundedString<ConstU32<64>>,
        client_peer_id: BoundedString<ConstU32<64>>,
        bootnode: BoundedString<ConstU32<1024>>,
        delegate_reward_rate: U256,
        stake_to_be_added: U256,
        unique: BoundedString<ConstU32<1024>>,
        non_unique: BoundedString<ConstU32<1024>>,
        max_burn_amount: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let peer_id = OpaquePeerId(peer_id.as_bytes().to_vec());
        let bootnode_peer_id = OpaquePeerId(bootnode_peer_id.as_bytes().to_vec());
        let client_peer_id = OpaquePeerId(client_peer_id.as_bytes().to_vec());
        let delegate_reward_rate: u128 = delegate_reward_rate.unique_saturated_into();
        let stake_to_be_added: u128 = stake_to_be_added.unique_saturated_into();
        let unique: Option<BoundedVec<u8, DefaultMaxVectorLength>> =
            bounded_string_to_option_bounded_vec::<1024, DefaultMaxVectorLength>(&unique)?;
        let bootnode: Option<BoundedVec<u8, DefaultMaxVectorLength>> =
            bounded_string_to_option_bounded_vec::<1024, DefaultMaxVectorLength>(&bootnode)?;
        let non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>> =
            bounded_string_to_option_bounded_vec::<1024, DefaultMaxVectorLength>(&non_unique)?;
        let max_burn_amount: u128 = max_burn_amount.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let call = pallet_network::Call::<R>::register_subnet_node {
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
            max_burn_amount,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
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
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updateDelegateRewardRate(uint256,uint256,uint256)")]
    #[precompile::payable]
    fn update_node_delegate_reward_rate(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        new_delegate_reward_rate: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
        let new_delegate_reward_rate = new_delegate_reward_rate.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_node_delegate_reward_rate {
            subnet_id,
            subnet_node_id,
            new_delegate_reward_rate,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updateUnique(uint256,uint256,string)")]
    #[precompile::payable]
    fn update_unique(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        unique: BoundedString<ConstU32<1024>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
        let unique: Option<BoundedVec<u8, DefaultMaxVectorLength>> =
            bounded_string_to_option_bounded_vec::<1024, DefaultMaxVectorLength>(&unique)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_unique {
            subnet_id,
            subnet_node_id,
            unique,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updateNonUnique(uint256,uint256,string)")]
    #[precompile::payable]
    fn update_non_unique(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        non_unique: BoundedString<ConstU32<1024>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
        let non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>> =
            bounded_string_to_option_bounded_vec::<1024, DefaultMaxVectorLength>(&non_unique)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_non_unique {
            subnet_id,
            subnet_node_id,
            non_unique,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updateColdkey(address,address)")]
    #[precompile::payable]
    fn update_coldkey(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
        new_coldkey: Address,
    ) -> EvmResult<()> {
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let new_coldkey = R::AddressMapping::into_account_id(new_coldkey.into());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_coldkey {
            hotkey,
            new_coldkey,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updateHotkey(address,address)")]
    #[precompile::payable]
    fn update_hotkey(
        handle: &mut impl PrecompileHandle,
        old_hotkey: Address,
        new_hotkey: Address,
    ) -> EvmResult<()> {
        let old_hotkey = R::AddressMapping::into_account_id(old_hotkey.into());
        let new_hotkey = R::AddressMapping::into_account_id(new_hotkey.into());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_hotkey {
            old_hotkey,
            new_hotkey,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updatePeerId(uint256,uint256,string)")]
    #[precompile::payable]
    fn update_peer_id(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        new_peer_id: BoundedString<ConstU32<64>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
        let new_peer_id = OpaquePeerId(new_peer_id.as_bytes().to_vec());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_peer_id {
            subnet_id,
            subnet_node_id,
            new_peer_id,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updateBootnode(uint256,uint256,string)")]
    #[precompile::payable]
    fn update_bootnode(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        new_bootnode: BoundedString<ConstU32<1024>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
        let new_bootnode: Option<BoundedVec<u8, DefaultMaxVectorLength>> =
            bounded_string_to_option_bounded_vec::<1024, DefaultMaxVectorLength>(&new_bootnode)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_bootnode {
            subnet_id,
            subnet_node_id,
            new_bootnode,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updateBootnodePeerId(uint256,uint256,string)")]
    #[precompile::payable]
    fn update_bootnode_peer_id(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        new_bootnode_peer_id: BoundedString<ConstU32<64>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
        let new_bootnode_peer_id = OpaquePeerId(new_bootnode_peer_id.as_bytes().to_vec());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_bootnode_peer_id {
            subnet_id,
            subnet_node_id,
            new_bootnode_peer_id,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updateClientPeerId(uint256,uint256,string)")]
    #[precompile::payable]
    fn update_client_peer_id(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
        new_client_peer_id: BoundedString<ConstU32<64>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
        let new_client_peer_id = OpaquePeerId(new_client_peer_id.as_bytes().to_vec());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_client_peer_id {
            subnet_id,
            subnet_node_id,
            new_client_peer_id,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public(
        "registerOrUpdateIdentity(address,string,string,string,string,string,string,string,string,string,string)"
    )]
    #[precompile::payable]
    fn register_or_update_identity(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
        name: BoundedString<ConstU32<1024>>,
        url: BoundedString<ConstU32<1024>>,
        image: BoundedString<ConstU32<1024>>,
        discord: BoundedString<ConstU32<255>>,
        x: BoundedString<ConstU32<255>>,
        telegram: BoundedString<ConstU32<255>>,
        github: BoundedString<ConstU32<1024>>,
        hugging_face: BoundedString<ConstU32<1024>>,
        description: BoundedString<ConstU32<1024>>,
        misc: BoundedString<ConstU32<1024>>,
    ) -> EvmResult<()> {
        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let hotkey = R::AddressMapping::into_account_id(hotkey.into());

        let name: BoundedVec<u8, DefaultMaxVectorLength> =
            bounded_string_to_bounded_vec::<1024, DefaultMaxVectorLength>(&name)?;
        let url: BoundedVec<u8, DefaultMaxUrlLength> =
            bounded_string_to_bounded_vec::<1024, DefaultMaxUrlLength>(&url)?;
        let image: BoundedVec<u8, DefaultMaxUrlLength> =
            bounded_string_to_bounded_vec::<1024, DefaultMaxUrlLength>(&image)?;
        let discord: BoundedVec<u8, DefaultMaxSocialIdLength> =
            bounded_string_to_bounded_vec::<255, DefaultMaxSocialIdLength>(&discord)?;
        let x: BoundedVec<u8, DefaultMaxSocialIdLength> =
            bounded_string_to_bounded_vec::<255, DefaultMaxSocialIdLength>(&x)?;
        let telegram: BoundedVec<u8, DefaultMaxSocialIdLength> =
            bounded_string_to_bounded_vec::<255, DefaultMaxSocialIdLength>(&telegram)?;
        let github: BoundedVec<u8, DefaultMaxUrlLength> =
            bounded_string_to_bounded_vec::<1024, DefaultMaxUrlLength>(&github)?;
        let hugging_face: BoundedVec<u8, DefaultMaxUrlLength> =
            bounded_string_to_bounded_vec::<1024, DefaultMaxUrlLength>(&hugging_face)?;
        let description: BoundedVec<u8, DefaultMaxVectorLength> =
            bounded_string_to_bounded_vec::<1024, DefaultMaxVectorLength>(&description)?;
        let misc: BoundedVec<u8, DefaultMaxVectorLength> =
            bounded_string_to_bounded_vec::<1024, DefaultMaxVectorLength>(&misc)?;

        let call = pallet_network::Call::<R>::register_or_update_identity {
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
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("removeIdentity()")]
    #[precompile::payable]
    fn remove_identity(handle: &mut impl PrecompileHandle) -> EvmResult<()> {
        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let call = pallet_network::Call::<R>::remove_identity {};

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerPauseSubnet(uint256)")]
    fn owner_pause_subnet(handle: &mut impl PrecompileHandle, subnet_id: U256) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_pause_subnet { subnet_id };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUnpauseSubnet(uint256)")]
    fn owner_unpause_subnet(handle: &mut impl PrecompileHandle, subnet_id: U256) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_unpause_subnet { subnet_id };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerSetEmergencyValidatorSet(uint256,uint256[])")]
    fn owner_set_emergency_validator_set(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_ids: Vec<U256>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_ids = subnet_node_ids
            .into_iter()
            .map(try_u256_to_u32)
            .collect::<Result<Vec<u32>, _>>()?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_set_emergency_validator_set {
            subnet_id,
            subnet_node_ids,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerRevertEmergencyValidatorSet(uint256)")]
    fn owner_revert_emergency_validator_set(handle: &mut impl PrecompileHandle, subnet_id: U256) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_revert_emergency_validator_set { subnet_id };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerDeactivateSubnet(uint256)")]
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
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateName(uint256,string)")]
    fn owner_update_name(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: BoundedString<ConstU32<256>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_name {
            subnet_id,
            value: value.into(),
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateRepo(uint256,string)")]
    fn owner_update_repo(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: BoundedString<ConstU32<256>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_repo {
            subnet_id,
            value: value.into(),
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateDescription(uint256,string)")]
    fn owner_update_description(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: BoundedString<ConstU32<256>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_description {
            subnet_id,
            value: value.into(),
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateMisc(uint256,string)")]
    fn owner_update_misc(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: BoundedString<ConstU32<256>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_misc {
            subnet_id,
            value: value.into(),
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateChurnLimit(uint256,uint256)")]
    fn owner_update_churn_limit(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = try_u256_to_u32(value)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_churn_limit { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateRegistrationQueueEpochs(uint256,uint256)")]
    fn owner_update_registration_queue_epochs(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = try_u256_to_u32(value)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_registration_queue_epochs { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateIdleClassificationEpochs(uint256,uint256)")]
    fn owner_update_idle_classification_epochs(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = try_u256_to_u32(value)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_idle_classification_epochs { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateIncludedClassificationEpochs(uint256,uint256)")]
    fn owner_update_included_classification_epochs(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = try_u256_to_u32(value)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_included_classification_epochs {
            subnet_id,
            value,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerAddOrUpdateInitialColdkeys(uint256,(address,uint256)[])")]
    fn owner_add_or_update_initial_coldkeys(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        coldkeys: Vec<(Address, U256)>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let coldkeys: BTreeMap<R::AccountId, u32> = coldkeys
            .into_iter()
            .map(|(addr, count)| {
                Ok::<_, PrecompileFailure>((
                    R::AddressMapping::into_account_id(addr.into()),
                    try_u256_to_u32(count)?,
                ))
            })
            .collect::<Result<_, _>>()?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_add_or_update_initial_coldkeys {
            subnet_id,
            coldkeys,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerRemoveInitialColdkeys(uint256,address[])")]
    fn owner_remove_initial_coldkeys(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        coldkeys: Vec<Address>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let coldkeys: BTreeSet<R::AccountId> = coldkeys
            .into_iter()
            .map(|a| R::AddressMapping::into_account_id(a.into()))
            .collect();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_remove_initial_coldkeys {
            subnet_id,
            coldkeys,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateKeyTypes(uint256,uint256[])")]
    fn owner_update_key_types(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        key_types: Vec<U256>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let key_types: BTreeSet<KeyType> = key_types
            .into_iter()
            .map(|val| key_type_from_u256(val).ok_or_else(|| revert("Invalid KeyType value")))
            .collect::<Result<_, _>>()?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_key_types {
            subnet_id,
            key_types,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateMinMaxStake(uint256,uint256,uint256)")]
    fn owner_update_min_max_stake(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        min: U256,
        max: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let min: u128 = min.unique_saturated_into();
        let max: u128 = max.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_min_max_stake {
            subnet_id,
            min,
            max,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateDelegateStakePercentage(uint256,uint256)")]
    fn owner_update_delegate_stake_percentage(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value: u128 = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_delegate_stake_percentage { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateMaxRegisteredNodes(uint256,uint256)")]
    fn owner_update_max_registered_nodes(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = try_u256_to_u32(value)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_max_registered_nodes { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("transferSubnetOwnership(uint256,address)")]
    fn transfer_subnet_ownership(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        new_owner: Address,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let new_owner = R::AddressMapping::into_account_id(new_owner.into());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::transfer_subnet_ownership {
            subnet_id,
            new_owner,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("acceptSubnetOwnership(uint256)")]
    fn accept_subnet_ownership(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<()> {
        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let subnet_id = try_u256_to_u32(subnet_id)?;
        let call = pallet_network::Call::<R>::accept_subnet_ownership { subnet_id };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateTargetNodeRegistrationsPerEpoch(uint256,uint256)")]
    fn owner_update_target_node_registrations_per_epoch(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = try_u256_to_u32(value)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_target_node_registrations_per_epoch {
            subnet_id,
            value,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateNodeBurnRateAlpha(uint256,uint256)")]
    fn owner_update_node_burn_rate_alpha(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value: u128 = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_node_burn_rate_alpha { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateQueueImmunityEpochs(uint256,uint256)")]
    fn owner_update_queue_immunity_epochs(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = try_u256_to_u32(value)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_queue_immunity_epochs { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateMinSubnetNodeReputation(uint256,uint256)")]
    fn owner_update_min_subnet_node_reputation(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_min_subnet_node_reputation { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public(
        "ownerUpdateSubnetNodeMinWeightDecreaseReputationThreshold(uint256,uint256)"
    )]
    fn owner_update_subnet_node_min_weight_decrease_reputation_threshold(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_subnet_node_min_weight_decrease_reputation_threshold {
            subnet_id,
            value,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateAbsentDecreaseReputationFactor(uint256,uint256)")]
    fn owner_update_absent_decrease_reputation_factor(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_absent_decrease_reputation_factor {
            subnet_id,
            value,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateIncludedIncreaseReputationFactor(uint256,uint256)")]
    fn owner_update_included_increase_reputation_factor(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_update_included_increase_reputation_factor {
            subnet_id,
            value,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdatBelowMinWeightDecreaseReputationFactor(uint256,uint256)")]
    fn owner_update_below_min_weight_decrease_reputation_factor(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_below_min_weight_decrease_reputation_factor {
                subnet_id,
                value,
            };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdatNonAttestorDecreaseReputationFactor(uint256,uint256)")]
    fn owner_update_non_attestor_decrease_reputation_factor(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_non_attestor_decrease_reputation_factor {
                subnet_id,
                value,
            };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdatNonConsensusAttestorDecreaseReputationFactor(uint256,uint256)")]
    fn owner_update_non_consensus_attestor_decrease_reputation_factor(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_non_consensus_attestor_decrease_reputation_factor { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateValidatorAbsentDecreaseReputationFactor(uint256,uint256)")]
    fn owner_update_validator_absent_decrease_reputation_factor(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_validator_absent_decrease_reputation_factor { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerUpdateValidatorNonConsensusDecreaseReputationFactor(uint256,uint256)")]
    fn owner_update_validator_non_consensus_decrease_reputation_factor(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        value: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let value = value.unique_saturated_into();

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call =
            pallet_network::Call::<R>::owner_update_validator_non_consensus_decrease_reputation_factor { subnet_id, value };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("updateBootnodes(uint256,string[],string[])")]
    fn update_bootnodes(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        add: Vec<BoundedString<ConstU32<1024>>>,
        remove: Vec<BoundedString<ConstU32<1024>>>,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;

        let add: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>> = add
            .into_iter()
            .map(|add| -> Result<_, PrecompileFailure> {
                let value: BoundedVec<u8, DefaultMaxVectorLength> =
                    bounded_string_to_bounded_vec::<1024, DefaultMaxVectorLength>(&add)?;
                Ok(value)
            })
            .collect::<Result<_, _>>()?;

        let remove: BTreeSet<BoundedVec<u8, DefaultMaxVectorLength>> = remove
            .into_iter()
            .map(|remove| -> Result<_, PrecompileFailure> {
                let value: BoundedVec<u8, DefaultMaxVectorLength> =
                    bounded_string_to_bounded_vec::<1024, DefaultMaxVectorLength>(&remove)?;
                Ok(value)
            })
            .collect::<Result<_, _>>()?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::update_bootnodes {
            subnet_id,
            add,
            remove,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerAddBootnodeAccess(uint256,address)")]
    fn owner_add_bootnode_access(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        new_account: Address,
    ) -> EvmResult<()> {
        let origin = R::AddressMapping::into_account_id(handle.context().caller);

        let subnet_id = try_u256_to_u32(subnet_id)?;
        let new_account = R::AddressMapping::into_account_id(new_account.into());

        let call = pallet_network::Call::<R>::owner_add_bootnode_access {
            subnet_id,
            new_account,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("ownerRemoveBootnodeAccess(uint256,address)")]
    fn owner_remove_bootnode_access(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        remove_account: Address,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let remove_account = R::AddressMapping::into_account_id(remove_account.into());

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::owner_remove_bootnode_access {
            subnet_id,
            remove_account,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("getSubnetName(uint256)")]
    #[precompile::view]
    fn get_subnet_name(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<BoundedString<ConstU32<256>>> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let subnet_data = pallet_network::SubnetsData::<R>::try_get(subnet_id)
            .map_err(|_| revert("SubnetsData not found for subnet"))?;

        Ok(subnet_data.name.into())
    }

    #[precompile::public("getSubnetRepo(uint256)")]
    #[precompile::view]
    fn get_subnet_repo(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<BoundedString<ConstU32<1024>>> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let subnet_data = pallet_network::SubnetsData::<R>::try_get(subnet_id)
            .map_err(|_| revert("SubnetsData not found for subnet"))?;

        Ok(subnet_data.repo.into())
    }

    #[precompile::public("getSubnetDescription(uint256)")]
    #[precompile::view]
    fn get_subnet_description(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<BoundedString<ConstU32<1024>>> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let subnet_data = pallet_network::SubnetsData::<R>::try_get(subnet_id)
            .map_err(|_| revert("SubnetsData not found for subnet"))?;

        Ok(subnet_data.description.into())
    }

    #[precompile::public("getSubnetMisc(uint256)")]
    #[precompile::view]
    fn get_subnet_misc(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<BoundedString<ConstU32<256>>> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let subnet_data = pallet_network::SubnetsData::<R>::try_get(subnet_id)
            .map_err(|_| revert("SubnetsData not found for subnet"))?;

        Ok(subnet_data.misc.into())
    }

    #[precompile::public("getChurnLimit(uint256)")]
    #[precompile::view]
    fn get_churn_limit(handle: &mut impl PrecompileHandle, subnet_id: U256) -> EvmResult<u32> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let result = pallet_network::ChurnLimit::<R>::get(subnet_id);

        Ok(result)
    }

    #[precompile::public("getRegistrationQueueEpochs(uint256)")]
    #[precompile::view]
    fn get_registration_queue_epochs(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<u32> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let result = pallet_network::SubnetNodeQueueEpochs::<R>::get(subnet_id);

        Ok(result)
    }

    #[precompile::public("getIdleClassificationEpochs(uint256)")]
    #[precompile::view]
    fn get_idle_classification_epochs(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<u32> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let result = pallet_network::IdleClassificationEpochs::<R>::get(subnet_id);

        Ok(result)
    }

    #[precompile::public("getIncludedClassificationEpochs(uint256)")]
    #[precompile::view]
    fn get_included_classification_epochs(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<u32> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let result = pallet_network::IncludedClassificationEpochs::<R>::get(subnet_id);

        Ok(result)
    }

    #[precompile::public("getInitialColdkeys(uint256)")]
    #[precompile::view]
    fn get_initial_coldkeys(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<Vec<(Address, U256)>> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let result = pallet_network::SubnetRegistrationInitialColdkeys::<R>::try_get(subnet_id)
            .map_err(|_| revert("SubnetRegistrationInitialColdkeys not found for subnet"))?;

        let coldkeys: Vec<(Address, U256)> = result
            .into_iter()
            .map(|(acc, c)| {
                let address = Address(sp_core::H160::from(acc.into()));
                let count = U256::from(c);
                (address, count)
            })
            .collect();

        Ok(coldkeys)
    }

    #[precompile::public("getKeyTypes(uint256)")]
    #[precompile::view]
    fn get_key_types(handle: &mut impl PrecompileHandle, subnet_id: U256) -> EvmResult<Vec<U256>> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let result = pallet_network::SubnetKeyTypes::<R>::get(subnet_id);

        let key_types: Vec<U256> = result.into_iter().map(|k| U256::from(k as u32)).collect();

        Ok(key_types)
    }

    #[precompile::public("getMinStake(uint256)")]
    #[precompile::view]
    fn get_min_stake(handle: &mut impl PrecompileHandle, subnet_id: U256) -> EvmResult<u128> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let result = pallet_network::SubnetMinStakeBalance::<R>::get(subnet_id);

        Ok(result)
    }

    #[precompile::public("getMaxStake(uint256)")]
    #[precompile::view]
    fn get_max_stake(handle: &mut impl PrecompileHandle, subnet_id: U256) -> EvmResult<u128> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let result = pallet_network::SubnetMaxStakeBalance::<R>::get(subnet_id);

        Ok(result)
    }

    #[precompile::public("getDelegateStakePercentage(uint256)")]
    fn get_delegate_stake_percentage(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<u128> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let result = pallet_network::SubnetDelegateStakeRewardsPercentage::<R>::get(subnet_id);

        Ok(result)
    }

    #[precompile::public("getMaxRegisteredNodes(uint256)")]
    #[precompile::view]
    fn get_max_registered_nodes(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
    ) -> EvmResult<u32> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let result = pallet_network::MaxRegisteredNodes::<R>::get(subnet_id);

        Ok(result)
    }
}

fn try_u256_to_u32(value: U256) -> Result<u32, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("u32 out of bounds".into()),
    })
}

fn try_u32_to_u256(value: u32) -> Result<U256, PrecompileFailure> {
    value.try_into().map_err(|_| PrecompileFailure::Error {
        exit_status: ExitError::Other("u32 out of bounds".into()),
    })
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

fn bounded_string_to_option_bounded_vec<const N: u32, T>(
    s: &BoundedString<ConstU32<N>>,
) -> Result<Option<BoundedVec<u8, T>>, PrecompileFailure>
where
    T: sp_runtime::traits::Get<u32>,
{
    if s.as_bytes().is_empty() {
        Ok(None)
    } else {
        let vec: BoundedVec<u8, T> = s
            .as_bytes()
            .to_vec()
            .try_into()
            .map_err(|_| revert("String too long"))?;
        Ok(Some(vec))
    }
}

fn bounded_string_to_bounded_vec<const N: u32, T>(
    s: &BoundedString<ConstU32<N>>,
) -> Result<BoundedVec<u8, T>, PrecompileFailure>
where
    T: sp_runtime::traits::Get<u32>,
{
    let vec: BoundedVec<u8, T> = s
        .as_bytes()
        .to_vec()
        .try_into()
        .map_err(|_| revert("String too long"))?;
    Ok(vec)
}
