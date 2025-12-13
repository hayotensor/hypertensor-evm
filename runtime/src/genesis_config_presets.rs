// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
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

use hex_literal::hex;
use scale_info::prelude::collections::BTreeMap;
// Substrate
use core::str::FromStr;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
#[allow(unused_imports)]
use sp_core::ecdsa;
use sp_core::{OpaquePeerId, Pair, Public, H160, U256};
use sp_runtime::traits::{IdentifyAccount, Verify};
// Frontier
use crate::{AccountId, Balance, SS58Prefix, Signature};
use alloc::{format, vec, vec::Vec};
use frame_support::build_struct_json_patch;
use serde_json::Value;
use sp_genesis_builder::{self, PresetId};

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

fn peer(id: u8) -> OpaquePeerId {
    let peer_id = format!("12D{id}KooWGFuUunX1AzAzjs3CgyqTXtPWX3AqRhJFbesGPGYHJQTP");
    OpaquePeerId(peer_id.into())
}

#[allow(dead_code)]
type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
/// For use with `AccountId32`, `dead_code` if `AccountId20`.
#[allow(dead_code)]
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

const UNITS: Balance = 1_000_000_000_000_000_000;

// Returns the genesis config presets populated with given parameters.
fn testnet_genesis(
    sudo_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    chain_id: u64,
    enable_manual_seal: bool,
) -> serde_json::Value {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let mut peer_index: u8 = 0;

    let evm_accounts = {
        let mut map = BTreeMap::new();
        map.insert(
            // H160 address of Alice dev account
            // Derived from SS58 (42 prefix) address
            // SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
            // hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
            // Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
            H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
                .expect("internal H160 is valid; qed"),
            fp_evm::GenesisAccount {
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                code: Default::default(),
                nonce: Default::default(),
                storage: Default::default(),
            },
        );
        map.insert(
            // H160 address of CI test runner account
            H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
                .expect("internal H160 is valid; qed"),
            fp_evm::GenesisAccount {
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                code: Default::default(),
                nonce: Default::default(),
                storage: Default::default(),
            },
        );
        map.insert(
            // H160 address for benchmark usage
            H160::from_str("1000000000000000000000000000000000000001")
                .expect("internal H160 is valid; qed"),
            fp_evm::GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: vec![0x00],
            },
        );
        map
    };

    serde_json::json!({
        "sudo": { "key": Some(sudo_key) },
        "balances": {
            "balances": endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1_000_000 * UNITS))
                .collect::<Vec<_>>()
        },
        "aura": { "authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>() },
        "grandpa": { "authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>() },
        "evmChainId": { "chainId": chain_id },
        "evm": { "accounts": evm_accounts },
        "manualSeal": { "enable": enable_manual_seal },
        "network": {
            "subnetName": subnet_name,
            "subnetNodes": endowed_accounts.iter().cloned().map(|k| {
                peer_index += 1;
                (
                    k,
                    peer(peer_index),
                )
            }).collect::<Vec<_>>(),
        },
    })
}

// Development testing mainly for ts-tests
fn ethereum_testnet_genesis(
    sudo_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    chain_id: u64,
    enable_manual_seal: bool,
) -> serde_json::Value {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let mut peer_index: u8 = 0;

    let evm_accounts = {
        let mut map = BTreeMap::new();
        map.insert(
            // H160 address of Alice dev account
            // Derived from SS58 (42 prefix) address
            // SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
            // hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
            // Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
            H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
                .expect("internal H160 is valid; qed"),
            fp_evm::GenesisAccount {
                balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                    .expect("internal U256 is valid; qed"),
                code: Default::default(),
                nonce: Default::default(),
                storage: Default::default(),
            },
        );
        map.insert(
            // H160 address of CI test runner account
            H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
                .expect("internal H160 is valid; qed"),
            fp_evm::GenesisAccount {
                balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                    .expect("internal U256 is valid; qed"),
                code: Default::default(),
                nonce: Default::default(),
                storage: Default::default(),
            },
        );
        map.insert(
            // H160 address for benchmark usage
            H160::from_str("1000000000000000000000000000000000000001")
                .expect("internal H160 is valid; qed"),
            fp_evm::GenesisAccount {
                nonce: U256::from(1),
                balance: U256::from(1_000_000_000_000_000_000_000_000u128),
                storage: Default::default(),
                code: vec![0x00],
            },
        );
        map
    };

    serde_json::json!({
        "sudo": { "key": Some(sudo_key) },
        "balances": {
            "balances": endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1_000_000 * UNITS))
                .collect::<Vec<_>>()
        },
        "aura": { "authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>() },
        "grandpa": { "authorities": initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>() },
        "evmChainId": { "chainId": chain_id },
        "evm": { "accounts": evm_accounts },
        "manualSeal": { "enable": enable_manual_seal },
        "network": {
            "subnetName": subnet_name,
            "subnetNodes": endowed_accounts.iter().cloned().map(|k| {
                peer_index += 1;
                (
                    k,
                    peer(peer_index),
                )
            }).collect::<Vec<_>>(),
        },
    })
}

/// Return the development genesis config.
pub fn development_config_genesis(enable_manual_seal: bool) -> Value {
    testnet_genesis(
        // Sudo account (Alith)
        AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
        // Pre-funded accounts
        vec![
            AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")), // Alith
            AccountId::from(hex!("317D7a5a2ba5787A99BE4693Eb340a10C71d680b")), // Alith hotkey
            AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")), // Baltathar
            AccountId::from(hex!("c30fE91DE91a3FA79E42Dfe7a01917d0D92D99D7")), // Baltathar hotkey
            AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")), // Charleth
            AccountId::from(hex!("2f7703Ba9953d422294079A1CB32f5d2B60E38EB")), // Charleth hotkey
            AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")), // Dorothy
            AccountId::from(hex!("294BFfC18b5321264f55c517Aca2963bEF9D29EA")), // Dorothy hotkey
            AccountId::from(hex!("Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB")), // Ethan
            AccountId::from(hex!("919a696741e5bEe48538D43CB8A34a95261E62fc")), // Ethan hotkey
            AccountId::from(hex!("C0F0f4ab324C46e55D02D0033343B4Be8A55532d")), // Faith
            AccountId::from(hex!("D4eb2503fA9F447CCa7b78D9a86F2fdbc964401e")), // Faith hotkey
        ],
        vec![authority_keys_from_seed("Alice")],
        SS58Prefix::get() as u64,
        enable_manual_seal,
    )
}

pub fn ethereum_development_config_genesis(enable_manual_seal: bool) -> Value {
    ethereum_testnet_genesis(
        // Sudo account (Alith)
        AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
        // Pre-funded accounts
        vec![
            AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")), // Alith
            AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")), // Baltathar
            AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")), // Charleth
            AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")), // Dorothy
            AccountId::from(hex!("Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB")), // Ethan
            AccountId::from(hex!("C0F0f4ab324C46e55D02D0033343B4Be8A55532d")), // Faith
        ],
        vec![authority_keys_from_seed("Alice")],
        SS58Prefix::get() as u64,
        enable_manual_seal,
    )
}

/// Return the local genesis config preset.
pub fn local_config_genesis() -> Value {
    testnet_genesis(
        // Sudo account (Alith)
        AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
        // Pre-funded accounts
        vec![
            AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")), // Alith
            AccountId::from(hex!("317D7a5a2ba5787A99BE4693Eb340a10C71d680b")), // Alith hotkey
            AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")), // Baltathar
            AccountId::from(hex!("c30fE91DE91a3FA79E42Dfe7a01917d0D92D99D7")), // Baltathar hotkey
            AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")), // Charleth
            AccountId::from(hex!("2f7703Ba9953d422294079A1CB32f5d2B60E38EB")), // Charleth hotkey
            AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")), // Dorothy
            AccountId::from(hex!("294BFfC18b5321264f55c517Aca2963bEF9D29EA")), // Dorothy hotkey
            AccountId::from(hex!("Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB")), // Ethan
            AccountId::from(hex!("919a696741e5bEe48538D43CB8A34a95261E62fc")), // Ethan hotkey
            AccountId::from(hex!("C0F0f4ab324C46e55D02D0033343B4Be8A55532d")), // Faith
            AccountId::from(hex!("D4eb2503fA9F447CCa7b78D9a86F2fdbc964401e")), // Faith hotkey
        ],
        vec![
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
        ],
        42,
        false,
    )
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
    let patch = match id.as_ref() {
        "ETHEREUM_DEV_RUNTIME_PRESET" => ethereum_development_config_genesis(true),
        sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(false),
        sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_config_genesis(),
        _ => return None,
    };
    Some(
        serde_json::to_string(&patch)
            .expect("serialization to json is expected to work. qed.")
            .into_bytes(),
    )
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
    vec![
        PresetId::from("ETHEREUM_DEV_RUNTIME_PRESET"),
        PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
        PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
    ]
}
