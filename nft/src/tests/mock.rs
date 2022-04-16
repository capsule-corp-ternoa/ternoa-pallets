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

use crate::{self as ternoa_nft, weights, Config, NegativeImbalanceOf};
use frame_support::{
	bounded_vec, parameter_types,
	traits::{ConstU32, Contains, Currency, GenesisBuild},
};
use primitives::nfts::{NFTData, NFTId, NFTSeriesDetails};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Do not use the `0` account id since this would be the default value
// for our account id. This would mess with some tests.
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const COLLECTOR: u64 = 99;

pub const ALICE_NFT_ID: u32 = 1;
pub const ALICE_SERIES_ID: u8 = 1;

pub const BOB_NFT_ID: u32 = 2;
pub const BOB_SERIES_ID: u8 = 2;

pub const NFT_MINT_FEE: Balance = 10;
pub const INVALID_NFT_ID: NFTId = 1001;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		NFT: ternoa_nft,
	}
);

pub struct TestBaseCallFilter;
impl Contains<Call> for TestBaseCallFilter {
	fn contains(c: &Call) -> bool {
		match *c {
			// Transfer works. Use `transfer_keep_alive` for a call that doesn't pass the filter.
			Call::Balances(pallet_balances::Call::transfer { .. }) => true,
			// For benchmarking, this acts as a noop call
			Call::System(frame_system::Call::remark { .. }) => true,
			// For tests
			_ => false,
		}
	}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

pub type Balance = u64;

impl frame_system::Config for Test {
	type BaseCallFilter = TestBaseCallFilter;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}
impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const IPFSLengthLimit: u32 = 5;
}

impl Config for Test {
	type Event = Event;
	type WeightInfo = weights::TernoaWeight<Test>;
	type Currency = Balances;
	type FeesCollector = MockFeeCollector;
	type IPFSLengthLimit = IPFSLengthLimit;
}

pub struct MockFeeCollector;
impl frame_support::traits::OnUnbalanced<NegativeImbalanceOf<Test>> for MockFeeCollector {
	fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<Test>) {
		Balances::resolve_creating(&COLLECTOR, amount);
	}
}

pub struct ExtBuilder {
	balances: Vec<(u64, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balances: Vec::new() }
	}
}

impl ExtBuilder {
	pub fn caps(mut self, accounts: Vec<(u64, Balance)>) -> Self {
		for account in accounts {
			self.balances.push(account);
		}
		self
	}

	pub fn new(balances: Vec<(u64, Balance)>) -> Self {
		Self { balances }
	}

	pub fn new_build(balances: Vec<(u64, Balance)>) -> sp_io::TestExternalities {
		Self::new(balances).build()
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		Self::build_nfts(&mut t);

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	fn build_nfts(t: &mut sp_runtime::Storage) {
		let alice_nft: NFTData<_, IPFSLengthLimit> =
			NFTData::new_default(ALICE, bounded_vec![100], vec![ALICE_SERIES_ID]);
		let bob_nft: NFTData<_, IPFSLengthLimit> =
			NFTData::new_default(BOB, bounded_vec![101], vec![BOB_SERIES_ID]);

		let alice_series = NFTSeriesDetails::new(ALICE, true);
		let bob_series = NFTSeriesDetails::new(BOB, true);

		let nfts = vec![alice_nft.to_raw(ALICE_NFT_ID), bob_nft.to_raw(BOB_NFT_ID)];
		let series = vec![
			alice_series.to_raw(vec![ALICE_SERIES_ID]),
			bob_series.to_raw(vec![BOB_SERIES_ID]),
		];

		ternoa_nft::GenesisConfig::<Test> { nfts, series, nft_mint_fee: NFT_MINT_FEE }
			.assimilate_storage(t)
			.unwrap();
	}
}

#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	t.into()
}