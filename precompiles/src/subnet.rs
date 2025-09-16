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
use pallet_network::{
    DefaultMaxSocialIdLength, DefaultMaxUrlLength, DefaultMaxVectorLength, KeyType,
    RegistrationSubnetData,
};
use sp_std::collections::btree_set::BTreeSet;

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
        "registerSubnet(address,string,string,string,string,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,address[],uint256,uint256[],string[])"
    )]
    #[precompile::payable]
    fn register_subnet(
        handle: &mut impl PrecompileHandle,
        hotkey: Address,
        name: BoundedString<ConstU32<256>>,
        repo: BoundedString<ConstU32<1024>>,
        description: BoundedString<ConstU32<1024>>,
        misc: BoundedString<ConstU32<1024>>,
        churn_limit: U256,
        min_stake: U256,
        max_stake: U256,
        delegate_stake_percentage: U256,
        subnet_node_queue_epochs: U256,
        activation_grace_epochs: U256,
        queue_classification_epochs: U256,
        included_classification_epochs: U256,
        max_node_penalties: U256,
        initial_coldkeys: Vec<Address>,
        max_registered_nodes: U256,
        key_types: Vec<U256>,
        bootnodes: Vec<BoundedString<ConstU32<1024>>>,
    ) -> EvmResult<()> {
        handle.record_cost(RuntimeHelper::<R>::db_read_gas_cost())?;

        let hotkey = R::AddressMapping::into_account_id(hotkey.into());
        let churn_limit = try_u256_to_u32(churn_limit)?;
        let min_stake: u128 = min_stake.unique_saturated_into();
        let max_stake: u128 = max_stake.unique_saturated_into();
        let delegate_stake_percentage: u128 = delegate_stake_percentage.unique_saturated_into();
        let subnet_node_queue_epochs = try_u256_to_u32(subnet_node_queue_epochs)?;
        let activation_grace_epochs = try_u256_to_u32(activation_grace_epochs)?;
        let queue_classification_epochs = try_u256_to_u32(queue_classification_epochs)?;
        let included_classification_epochs = try_u256_to_u32(included_classification_epochs)?;
        let max_node_penalties = try_u256_to_u32(max_node_penalties)?;
        let initial_coldkeys: BTreeSet<R::AccountId> = initial_coldkeys
            .into_iter()
            .map(|a| R::AddressMapping::into_account_id(a.into()))
            .collect();
        let max_registered_nodes = try_u256_to_u32(max_registered_nodes)?;

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

        let subnet_data = pallet_network::RegistrationSubnetData::<R::AccountId> {
            name: name.into(),
            repo: repo.into(),
            description: description.into(),
            misc: misc.into(),
            min_stake,
            max_stake,
            delegate_stake_percentage,
            churn_limit,
            subnet_node_queue_epochs,
            activation_grace_epochs,
            queue_classification_epochs,
            included_classification_epochs,
            max_node_penalties,
            initial_coldkeys,
            max_registered_nodes,
            key_types,
            bootnodes: bootnodes,
        };

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::register_subnet {
            hotkey,
            subnet_data,
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("registrationCost(uint256)")]
    #[precompile::view]
    fn registration_cost(handle: &mut impl PrecompileHandle, block: U256) -> EvmResult<u128> {
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

    #[precompile::public(
        "registerSubnetNode(uint256,address,string,string,string,string,uint256,uint256,string,string)"
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
        };

        RuntimeHelper::<R>::try_dispatch(
            handle,
            RawOrigin::Signed(origin.clone()).into(),
            call,
            0,
        )?;

        Ok(())
    }

    #[precompile::public("deactivateSubnetNode(uint256,uint256)")]
    #[precompile::payable]
    fn pause_subnet_node(
        handle: &mut impl PrecompileHandle,
        subnet_id: U256,
        subnet_node_id: U256,
    ) -> EvmResult<()> {
        let subnet_id = try_u256_to_u32(subnet_id)?;
        let subnet_node_id = try_u256_to_u32(subnet_node_id)?;

        let origin = R::AddressMapping::into_account_id(handle.context().caller);
        let call = pallet_network::Call::<R>::pause_subnet_node {
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
            0,
        )?;

        Ok(())
    }

    // #[precompile::public("updateUnique(uint256,uint256,string)")]
    // #[precompile::payable]
    // fn update_unique(
    //     handle: &mut impl PrecompileHandle,
    //     subnet_id: U256,
    //     subnet_node_id: U256,
    //     unique: BoundedString<ConstU32<1024>>,
    // ) -> EvmResult<()> {
    //     let subnet_id = try_u256_to_u32(subnet_id)?;
    //     let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
    //     let unique: Option<BoundedVec<u8, DefaultMaxVectorLength>> = bounded_string_to_option_bounded_vec::<1024, DefaultMaxVectorLength>(&unique)?;

    //     let origin = R::AddressMapping::into_account_id(handle.context().caller);
    //     let call = pallet_network::Call::<R>::update_unique {
    //         subnet_id,
    //         subnet_node_id,
    //         unique,
    //     };

    //     RuntimeHelper::<R>::try_dispatch(
    //         handle,
    //         RawOrigin::Signed(origin.clone()).into(),
    //         call,
    //         0,
    //     )?;

    //     Ok(())
    // }

    // #[precompile::public("updateNonUnique(uint256,uint256,string)")]
    // #[precompile::payable]
    // fn update_non_unique(
    //     handle: &mut impl PrecompileHandle,
    //     subnet_id: U256,
    //     subnet_node_id: U256,
    //     non_unique: BoundedString<ConstU32<1024>>,
    // ) -> EvmResult<()> {
    //     let subnet_id = try_u256_to_u32(subnet_id)?;
    //     let subnet_node_id = try_u256_to_u32(subnet_node_id)?;
    //     let non_unique: Option<BoundedVec<u8, DefaultMaxVectorLength>> = bounded_string_to_option_bounded_vec::<1024, DefaultMaxVectorLength>(&non_unique)?;

    //     let origin = R::AddressMapping::into_account_id(handle.context().caller);
    //     let call = pallet_network::Call::<R>::update_non_unique {
    //         subnet_id,
    //         subnet_node_id,
    //         non_unique,
    //     };

    //     RuntimeHelper::<R>::try_dispatch(
    //         handle,
    //         RawOrigin::Signed(origin.clone()).into(),
    //         call,
    //         0,
    //     )?;

    //     Ok(())
    // }

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
}

fn try_u256_to_u32(value: U256) -> Result<u32, PrecompileFailure> {
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
