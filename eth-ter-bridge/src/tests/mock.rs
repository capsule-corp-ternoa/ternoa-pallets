// Copyright 2022 Capsule Corp (France) SAS.
// This file is part of Ternoa.

// Ternoa is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Ternoa is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Ternoa.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::{
	assert_ok, parameter_types,
	traits::{ConstU32, Currency, Everything},
	weights::Weight,
	BoundedVec, PalletId,
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

use crate::{self as ternoa_bridge, ChainId, Config, NegativeImbalanceOf};

pub type AccountId = u64;
pub type Balance = u64;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const DEFAULT_RELAYER_VOTE_THRESHOLD: u32 = 1;
pub const DEFAULT_RELAYER_COUNT_LIMIT: u32 = 3;
pub const DEFAULT_INITIAL_BRIDGE_FEE: u32 = 1;
pub const RELAYER_A: u64 = 0x2;
pub const RELAYER_B: u64 = 0x3;
pub const RELAYER_C: u64 = 0x4;
pub const ENDOWED_BALANCE: u64 = 100_000_000;
pub const COLLECTOR: u64 = 99;

// Build mock runtime
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system,
		Balances: pallet_balances,
		Bridge: ternoa_bridge,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const MaxLocks: u32 = 100;
}

impl frame_system::Config for Test {
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

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
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

parameter_types! {
	pub const MockChainId: ChainId = 5;
	pub const ChainBridgePalletId: PalletId = PalletId(*b"cb/bridg");
	pub const ProposalLifetime: u64 = 10;
	pub const RelayerVoteThreshold: u32 = DEFAULT_RELAYER_VOTE_THRESHOLD;
	pub const RelayerCountLimit: u32 = DEFAULT_RELAYER_COUNT_LIMIT;
	pub const InitialBridgeFee: u32 = DEFAULT_INITIAL_BRIDGE_FEE;
}

pub struct MockFeeCollector;
impl frame_support::traits::OnUnbalanced<NegativeImbalanceOf<Test>> for MockFeeCollector {
	fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<Test>) {
		Balances::resolve_creating(&COLLECTOR, amount);
	}
}

impl Config for Test {
	type Event = Event;
	type WeightInfo = ternoa_bridge::weights::TernoaWeight<Test>;
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

pub struct ExtBuilder {}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {}
	}
}

impl ExtBuilder {
	pub fn build() -> TestExternalities {
		let bridge_id = Bridge::account_id();

		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> {
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

	pub fn build_with(src_id: ChainId, threshold: u32) -> TestExternalities {
		let mut externalities = Self::build();

		externalities.execute_with(|| {
			// Set and check threshold
			assert_ok!(Bridge::set_threshold(Origin::root(), threshold));
			assert_eq!(Bridge::relayer_vote_threshold(), threshold);
			// Add relayers
			assert_ok!(Bridge::set_relayers(
				Origin::root(),
				BoundedVec::try_from(vec![RELAYER_A, RELAYER_B, RELAYER_C]).unwrap()
			));
			// Whitelist chain
			assert_ok!(Bridge::add_chain(Origin::root(), src_id));
		});

		externalities
	}
}

#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
