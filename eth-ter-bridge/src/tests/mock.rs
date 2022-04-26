// Copyright 2021 Centrifuge Foundation (centrifuge.io).
// This file is part of Centrifuge chain project.

// Centrifuge is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version (see http://www.gnu.org/licenses).

// Centrifuge is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

//! Mocking runtime for testing the Substrate/Ethereum chains bridging pallet.
//!
//! The main components implemented in this mock module is a mock runtime
//! and some helper functions.

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

// Import crate types, traits and constants
use crate::{
	self as pallet_chainbridge, tests::constants::*, ChainId, Config as ChainBridgePalletConfig,
	NegativeImbalanceOf, WeightInfo,
};

// Import Substrate primitives and components
use frame_support::{
	assert_ok, parameter_types,
	traits::{ConstU32, Currency, Everything, SortedMembers},
	weights::Weight,
	BoundedVec, PalletId,
};

use frame_system::{
	mocking::{MockBlock, MockUncheckedExtrinsic},
	EnsureRoot,
};

use sp_core::H256;

use sp_io::TestExternalities;

use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

// ----------------------------------------------------------------------------
// Types and constants declaration
// ----------------------------------------------------------------------------

pub type AccountId = u64;
pub type Balance = u64;

// Runtime mocking types definition
type UncheckedExtrinsic = MockUncheckedExtrinsic<MockRuntime>;
type Block = MockBlock<MockRuntime>;

// ----------------------------------------------------------------------------
// Weights
// ----------------------------------------------------------------------------

// Implement testing extrinsic weights for the pallet
pub struct MockWeightInfo;
impl WeightInfo for MockWeightInfo {
	fn set_threshold() -> Weight {
		0 as Weight
	}

	fn whitelist_chain() -> Weight {
		0 as Weight
	}

	fn set_relayers() -> Weight {
		0 as Weight
	}

	fn vote_for_proposal() -> Weight {
		0 as Weight
	}

	fn deposit() -> Weight {
		0 as Weight
	}

	fn set_bridge_fee() -> Weight {
		0 as Weight
	}
}

// Constants definition
pub(crate) const RELAYER_A: u64 = 0x2;
pub(crate) const RELAYER_B: u64 = 0x3;
pub(crate) const RELAYER_C: u64 = 0x4;
pub(crate) const ENDOWED_BALANCE: u64 = 100_000_000;
pub const COLLECTOR: u64 = 99;

// ----------------------------------------------------------------------------
// Mock runtime configuration
// ----------------------------------------------------------------------------

// Build mock runtime
frame_support::construct_runtime!(

	pub enum MockRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system,
		Balances: pallet_balances,
		ChainBridge: pallet_chainbridge,
	}
);

// Parameterize default test user identifier (with id 1)
parameter_types! {
	pub const TestUserId: u64 = 1;
}

impl SortedMembers<u64> for TestUserId {
	fn sorted_members() -> Vec<u64> {
		vec![1]
	}
}

// Parameterize FRAME system pallet
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const MaxLocks: u32 = 100;
}

// Implement FRAME system pallet configuration trait for the mock runtime
impl frame_system::Config for MockRuntime {
	type BaseCallFilter = Everything;
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type PalletInfo = PalletInfo;
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

// Parameterize FRAME balances pallet
parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

// Implement FRAME balances pallet configuration trait for the mock runtime
impl pallet_balances::Config for MockRuntime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type WeightInfo = ();
}

// Parameterize chainbridge pallet
parameter_types! {
	pub const MockChainId: ChainId = 5;
	pub const ChainBridgePalletId: PalletId = PalletId(*b"cb/bridg");
	pub const ProposalLifetime: u64 = 10;
	pub const RelayerVoteThreshold: u32 = DEFAULT_RELAYER_VOTE_THRESHOLD;
	pub const RelayerCountLimit: u32 = DEFAULT_RELAYER_COUNT_LIMIT;
	pub const InitialBridgeFee: u32 = DEFAULT_INITIAL_BRIDGE_FEE;
}

pub struct MockFeeCollector;
impl frame_support::traits::OnUnbalanced<NegativeImbalanceOf<MockRuntime>> for MockFeeCollector {
	fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<MockRuntime>) {
		Balances::resolve_creating(&COLLECTOR, amount);
	}
}

// Implement chainbridge pallet configuration trait for the mock runtime
impl ChainBridgePalletConfig for MockRuntime {
	type Event = Event;
	type WeightInfo = MockWeightInfo;
	type Currency = Balances;
	type FeesCollector = MockFeeCollector;
	type ExternalOrigin = EnsureRoot<Self::AccountId>;
	type ChainId = MockChainId;
	type PalletId = ChainBridgePalletId;
	type ProposalLifetime = ProposalLifetime;
	type RelayerVoteThreshold = RelayerVoteThreshold;
	type RelayerCountLimit = RelayerCountLimit;
	type InitialBridgeFee = InitialBridgeFee;
}

// ----------------------------------------------------------------------------
// Test externalities
// ----------------------------------------------------------------------------

// Test externalities builder type declaraction.
//
// This type is mainly used for mocking storage in tests. It is the type alias
// for an in-memory, hashmap-based externalities implementation.
pub struct TestExternalitiesBuilder {}

// Default trait implementation for test externalities builder
impl Default for TestExternalitiesBuilder {
	fn default() -> Self {
		Self {}
	}
}

impl TestExternalitiesBuilder {
	// Build a genesis storage key/value store
	pub(crate) fn build(self) -> TestExternalities {
		let bridge_id = ChainBridge::account_id();

		let mut storage =
			frame_system::GenesisConfig::default().build_storage::<MockRuntime>().unwrap();

		// pre-fill balances
		pallet_balances::GenesisConfig::<MockRuntime> {
			balances: vec![
				(bridge_id, ENDOWED_BALANCE),
				(RELAYER_A, ENDOWED_BALANCE),
				(RELAYER_B, ENDOWED_BALANCE),
				(RELAYER_C, ENDOWED_BALANCE),
			],
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		let mut externalities = TestExternalities::new(storage);
		externalities.execute_with(|| System::set_block_number(1));
		externalities
	}

	// Build a genesis storage with a pre-configured chainbridge
	pub(crate) fn build_with(self, src_id: ChainId, threshold: u32) -> TestExternalities {
		let mut externalities = Self::build(self);

		externalities.execute_with(|| {
			// Set and check threshold
			assert_ok!(ChainBridge::set_threshold(Origin::root(), threshold));
			assert_eq!(ChainBridge::relayer_vote_threshold(), threshold);
			// Add relayers
			assert_ok!(ChainBridge::set_relayers(
				Origin::root(),
				BoundedVec::try_from(vec![RELAYER_A, RELAYER_B, RELAYER_C]).unwrap()
			));
			// Whitelist chain
			assert_ok!(ChainBridge::whitelist_chain(Origin::root(), src_id));
		});

		externalities
	}
}

#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<MockRuntime>().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
