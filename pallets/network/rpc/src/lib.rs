use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{error::ErrorObject, ErrorObjectOwned},
};

use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

use sp_api::ProvideRuntimeApi;

use fp_account::AccountId20;
use frame_support::storage::bounded_vec::BoundedVec;
pub use network_custom_rpc_runtime_api::NetworkRuntimeApi;
use pallet_network::{DefaultMaxVectorLength, SubnetNodeClass};

#[rpc(client, server)]
pub trait NetworkCustomApi<BlockHash> {
    #[method(name = "network_getSubnetInfo")]
    fn get_subnet_info(&self, subnet_id: u32, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "network_getAllSubnetsInfo")]
    fn get_all_subnets_info(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "network_getSubnetNodeInfo")]
    fn get_subnet_node_info(
        &self,
        subnet_id: u32,
        subnet_node_id: u32,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "network_getSubnetNodesInfo")]
    fn get_subnet_nodes_info(&self, subnet_id: u32, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "network_getAllSubnetNodesInfo")]
    fn get_all_subnet_nodes_info(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "network_proofOfStake")]
    fn proof_of_stake(
        &self,
        subnet_id: u32,
        peer_id: Vec<u8>,
        min_class: u8,
        at: Option<BlockHash>,
    ) -> RpcResult<bool>;
    #[method(name = "network_getBootnodes")]
    fn get_bootnodes(&self, subnet_id: u32, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "network_getColdkeySubnetNodesInfo")]
    fn get_coldkey_subnet_nodes_info(
        &self,
        coldkey: AccountId20,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "network_getColdkeyStakes")]
    fn get_coldkey_stakes(&self, coldkey: AccountId20, at: Option<BlockHash>)
        -> RpcResult<Vec<u8>>;
    #[method(name = "network_getDelegateStakes")]
    fn get_delegate_stakes(
        &self,
        account_id: AccountId20,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "network_getNodeDelegateStakes")]
    fn get_node_delegate_stakes(
        &self,
        account_id: AccountId20,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "network_getOverwatchCommitsForEpochAndNode")]
    fn get_overwatch_commits_for_epoch_and_node(
        &self,
        epoch: u32,
        overwatch_node_id: u32,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "network_getOverwatchRevealsForEpochAndNode")]
    fn get_overwatch_reveals_for_epoch_and_node(
        &self,
        epoch: u32,
        overwatch_node_id: u32,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
}

/// A struct that implements the `NetworkCustomApi`.
pub struct NetworkCustom<C, Block> {
    // If you have more generics, no need to NetworkCustom<C, M, N, P, ...>
    // just use a tuple like NetworkCustom<C, (M, N, P, ...)>
    client: Arc<C>,
    _marker: std::marker::PhantomData<Block>,
}

impl<C, Block> NetworkCustom<C, Block> {
    /// Create new `NetworkCustom` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The call to runtime failed.
    RuntimeError(String),
}

impl From<Error> for ErrorObjectOwned {
    fn from(e: Error) -> Self {
        match e {
            Error::RuntimeError(e) => ErrorObject::owned(1, e, None::<()>),
        }
    }
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError(_) => 1,
        }
    }
}

impl<C, Block> NetworkCustomApiServer<<Block as BlockT>::Hash> for NetworkCustom<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: NetworkRuntimeApi<Block>,
{
    fn get_subnet_info(
        &self,
        subnet_id: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_subnet_info(at, subnet_id)
            .map_err(|e| Error::RuntimeError(format!("Unable to get subnet info: {:?}", e)).into())
    }

    fn get_all_subnets_info(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_all_subnets_info(at).map_err(|e| {
            Error::RuntimeError(format!("Unable to get all subnets info: {:?}", e)).into()
        })
    }

    fn get_subnet_node_info(
        &self,
        subnet_id: u32,
        subnet_node_id: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_subnet_node_info(at, subnet_id, subnet_node_id)
            .map_err(|e| {
                Error::RuntimeError(format!("Unable to get subnet node info: {:?}", e)).into()
            })
    }

    fn get_subnet_nodes_info(
        &self,
        subnet_id: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_subnet_nodes_info(at, subnet_id).map_err(|e| {
            Error::RuntimeError(format!("Unable to get subnet node info: {:?}", e)).into()
        })
    }

    fn get_all_subnet_nodes_info(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_all_subnet_nodes_info(at).map_err(|e| {
            Error::RuntimeError(format!("Unable to get all subnet node info: {:?}", e)).into()
        })
    }

    fn proof_of_stake(
        &self,
        subnet_id: u32,
        peer_id: Vec<u8>,
        min_class: u8,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<bool> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.proof_of_stake(at, subnet_id, peer_id, min_class)
            .map_err(|e| {
                Error::RuntimeError(format!(
                    "Unable to get subnet nodes by a parameter: {:?}",
                    e
                ))
                .into()
            })
    }
    fn get_bootnodes(
        &self,
        subnet_id: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_bootnodes(at, subnet_id)
            .map_err(|e| Error::RuntimeError(format!("Unable to get bootnodes: {:?}", e)).into())
    }

    fn get_coldkey_subnet_nodes_info(
        &self,
        coldkey: AccountId20,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_coldkey_subnet_nodes_info(at, coldkey).map_err(|e| {
            Error::RuntimeError(format!("Unable to get coldkey subnet nodes info: {:?}", e)).into()
        })
    }

    fn get_coldkey_stakes(
        &self,
        coldkey: AccountId20,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_coldkey_stakes(at, coldkey).map_err(|e| {
            Error::RuntimeError(format!("Unable to get coldkey stakes: {:?}", e)).into()
        })
    }

    fn get_delegate_stakes(
        &self,
        account_id: AccountId20,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_delegate_stakes(at, account_id).map_err(|e| {
            Error::RuntimeError(format!("Unable to get account delegate stakes: {:?}", e)).into()
        })
    }

    fn get_node_delegate_stakes(
        &self,
        account_id: AccountId20,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_node_delegate_stakes(at, account_id).map_err(|e| {
            Error::RuntimeError(format!(
                "Unable to get account node delegate stakes: {:?}",
                e
            ))
            .into()
        })
    }

    fn get_overwatch_commits_for_epoch_and_node(
        &self,
        epoch: u32,
        overwatch_node_id: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_overwatch_commits_for_epoch_and_node(at, epoch, overwatch_node_id)
            .map_err(|e| {
                Error::RuntimeError(format!("Unable to get overwatch node commits: {:?}", e)).into()
            })
    }

    fn get_overwatch_reveals_for_epoch_and_node(
        &self,
        epoch: u32,
        overwatch_node_id: u32,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_overwatch_reveals_for_epoch_and_node(at, epoch, overwatch_node_id)
            .map_err(|e| {
                Error::RuntimeError(format!("Unable to get overwatch node reveals: {:?}", e)).into()
            })
    }
}
