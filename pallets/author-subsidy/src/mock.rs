// Copyright (C) Hypertensor.
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

use crate as pallet_author_subsidy;
use crate::*;
use core::str::FromStr;
use fp_account::EthereumSignature;
use frame_support::weights::constants::WEIGHT_REF_TIME_PER_MILLIS;
use frame_support::ConsensusEngineId;
use frame_support::{
    derive_impl, parameter_types,
    traits::{
        tokens::{PayFromAccount, UnityAssetBalanceConversion},
        Everything,
    },
    weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
    PalletId,
};
use frame_system as system;
pub use frame_system::{EnsureRoot, EnsureRootWithSuccess};
use pallet_evm::IdentityAddressMapping;
use sp_core::H160;
use sp_core::{ConstU128, ConstU32, ConstU64, H256, U256};
use sp_runtime::traits::{AccountIdLookup, BlakeTwo256, IdentifyAccount, IdentityLookup, Verify};
use sp_runtime::BuildStorage;
use sp_runtime::Perbill;
use sp_runtime::Permill;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlockU32<Test>;

frame_support::construct_runtime!(
    pub enum Test
    {
      System: system,
      Balances: pallet_balances,
      AuthorSubsidy: pallet_author_subsidy,
    }
);

/// A hash of some data used by the chain.
pub type Hash = H256;

/// The hashing algorithm used by the chain.
pub type Hashing = BlakeTwo256;

// An index to a block.
pub type BlockNumber = u32;

pub type BalanceCall = pallet_balances::Call<Test>;

pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
pub const YEAR: BlockNumber = DAYS * 365;
pub const BLOCKS_PER_HALVING: BlockNumber = YEAR * 2;
pub const TARGET_MAX_TOTAL_SUPPLY: u128 = 2_800_000_000_000_000_000_000_000;
pub const INITIAL_REWARD_PER_BLOCK: u128 =
    (TARGET_MAX_TOTAL_SUPPLY / 2) / BLOCKS_PER_HALVING as u128;

pub const SECS_PER_BLOCK: u32 = 6000 / 1000;

pub const EPOCH_LENGTH: u32 = 100;
pub const BLOCKS_PER_EPOCH: u32 = SECS_PER_BLOCK * EPOCH_LENGTH;
pub const EPOCHS_PER_YEAR: u32 = (YEAR as u32) / BLOCKS_PER_EPOCH;

pub const OVERWATCH_YEARLY_EMISSIONS: u128 = 10_000_000_000_000_000_000_000; // 10,000
pub const OVERWATCH_EPOCH_EMISSIONS: u128 = OVERWATCH_YEARLY_EMISSIONS / (EPOCHS_PER_YEAR as u128);

pub const AUTHOR_YEARLY_EMISSIONS: u128 = 1_000_000_000_000_000_000_000; // 1,000
pub const AUTHOR_BLOCK_EMISSIONS: u128 = AUTHOR_YEARLY_EMISSIONS / (YEAR as u128);

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2000ms of compute with a 6 second average block time.
pub const WEIGHT_MILLISECS_PER_BLOCK: u64 = 2000;
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
    WEIGHT_MILLISECS_PER_BLOCK * WEIGHT_REF_TIME_PER_MILLIS,
    u64::MAX,
);

parameter_types! {
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
        ::with_sensible_defaults(MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);
}

parameter_types! {
  pub const BlockHashCount: BlockNumber = 250;
  pub const SS58Prefix: u8 = 42;
}

pub type Signature = EthereumSignature;

pub type AccountPublic = <Signature as Verify>::Signer;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// The address format for describing accounts.
pub type Address = AccountId;

// Balance of an account.
pub type Balance = u128;

pub const EXISTENTIAL_DEPOSIT: u128 = 500;

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type Balance = Balance;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Block = Block;
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u32;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const AuthorBlockEmissions: u128 = AUTHOR_BLOCK_EMISSIONS;
}

pub struct FindAuthorTruncated;
impl FindAuthor<H160> for FindAuthorTruncated {
    fn find_author<'a, I>(_digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        Some(H160::from_str("1234500000000000000000000000000000000000").unwrap())
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type FindAuthor = FindAuthorTruncated;
    type AddressMapping = IdentityAddressMapping;
    type WeightInfo = ();
    type AuthorBlockEmissions = AuthorBlockEmissions;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
