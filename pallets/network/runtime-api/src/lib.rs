// This file is part of Hypertensor.

// Copyright (C) 2023 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Runtime API definition for the network pallet.

#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::BoundedVec;
use pallet_network::{
    ConsensusData, DefaultMaxVectorLength, SubnetInfo, SubnetNode, SubnetNodeInfo,
};
use sp_std::vec::Vec;
// use fp_account::AccountId20;

sp_api::decl_runtime_apis! {
  pub trait NetworkRuntimeApi {
    fn get_subnet_info(subnet_id: u32) -> Vec<u8>;
    fn get_all_subnets_info() -> Vec<u8>;
    fn get_subnet_nodes(subnet_id: u32) -> Vec<u8>;
    fn get_min_class_subnet_nodes(subnet_id: u32, subnet_epoch: u32, min_class: u8) -> Vec<u8>;
    fn get_subnet_nodes_included(subnet_id: u32) -> Vec<u8>;
    fn get_subnet_nodes_validator(subnet_id: u32) -> Vec<u8>;
    fn get_consensus_data(subnet_id: u32, epoch: u32) -> Vec<u8>;
    fn get_subnet_nodes_info(subnet_id: u32) -> Vec<u8>;
    fn is_subnet_node_by_peer_id(subnet_id: u32, peer_id: Vec<u8>) -> bool;
    fn is_subnet_node_by_unique(subnet_id: u32, unique: BoundedVec<u8, DefaultMaxVectorLength>) -> bool;
    fn proof_of_stake(subnet_id: u32, peer_id: Vec<u8>, min_class: u8) -> bool;

    // fn get_subnet_info(subnet_id: u32) -> Option<SubnetInfo<AccountId20>>;
    // fn get_all_subnets_info() -> Vec<SubnetInfo<AccountId20>>;
    // fn get_subnet_nodes(subnet_id: u32) -> Vec<SubnetNode<AccountId20>>;
    // fn get_subnet_nodes_included(subnet_id: u32) -> Vec<SubnetNode<AccountId20>>;
    // fn get_subnet_nodes_validator(subnet_id: u32) -> Vec<SubnetNode<AccountId20>>;
    // fn get_consensus_data(subnet_id: u32, epoch: u32) -> Option<ConsensusData<AccountId20>>;
    // fn get_subnet_nodes_info(subnet_id: u32) -> Vec<SubnetNodeInfo<AccountId20>>;
    // fn is_subnet_node_by_peer_id(subnet_id: u32, peer_id: Vec<u8>) -> bool;
    // fn is_subnet_node_by_unique(subnet_id: u32, unique: BoundedVec<u8, DefaultMaxVectorLength>) -> bool;
    // fn proof_of_stake(subnet_id: u32, peer_id: Vec<u8>, min_class: u8) -> bool;
  }
}
