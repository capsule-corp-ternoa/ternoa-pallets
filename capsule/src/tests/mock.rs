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
	parameter_types,
	traits::{ConstU32, Contains, GenesisBuild},
	PalletId,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use crate::{self as ternoa_capsule, Config};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

type AccountId = u64;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		NFT: ternoa_nft,
		Capsule: ternoa_capsule,
		TEE: ternoa_tee,
	}
);

pub struct TestBaseCallFilter;
impl Contains<RuntimeCall> for TestBaseCallFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match *c {
			// Transfer works. Use `transfer_keep_alive` for a call that doesn't pass the filter.
			RuntimeCall::Balances(pallet_balances::Call::transfer { .. }) => true,
			// For benchmarking, this acts as a noop call
			RuntimeCall::System(frame_system::Call::remark { .. }) => true,
			// For tests
			_ => false,
		}
	}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(frame_support::weights::Weight::from_ref_time(1024));
}
impl frame_system::Config for Test {
	type BaseCallFilter = TestBaseCallFilter;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type Balance = u128;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const ClusterSize: u32 = 5;
	pub const MaxUriLen: u32 = 12;
	pub const ListSizeLimit: u32 = 10;
}

impl ternoa_tee::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type FeesCollector = ();
	type ClusterSize = ClusterSize;
	type MaxUriLen = MaxUriLen;
	type ListSizeLimit = ListSizeLimit;
}

parameter_types! {
	pub const IPFSLengthLimit: u32 = 5;
	pub const CapsuleCountLimit: u32 = 2;
	pub const CapsulePalletId: PalletId = PalletId(*b"mockcaps");
}

impl ternoa_nft::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ternoa_nft::weights::TernoaWeight<Test>;
	type Currency = Balances;
	type FeesCollector = ();
	type IPFSLengthLimit = IPFSLengthLimit;
	type TEEExt = TEE;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type NFTExt = NFT;
	type PalletId = CapsulePalletId;
	type CapsuleCountLimit = CapsuleCountLimit;
}

// Do not use the `0` account id since this would be the default value
// for our account id. This would mess with some tests.
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;

pub struct ExtBuilder {
	endowed_accounts: Vec<(u64, u128)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder { endowed_accounts: Vec::new() }
	}
}

impl ExtBuilder {
	pub fn caps(mut self, accounts: Vec<(u64, u128)>) -> Self {
		for account in accounts {
			self.endowed_accounts.push(account);
		}
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: self.endowed_accounts }
			.assimilate_storage(&mut t)
			.unwrap();

		ternoa_nft::GenesisConfig::<Test> {
			nfts: Default::default(),
			series: Default::default(),
			nft_mint_fee: 10,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		ternoa_capsule::GenesisConfig::<Test> { capsule_mint_fee: 1000, ..Default::default() }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub mod help {
	use super::*;
	use frame_support::{assert_ok, bounded_vec, BoundedVec};
	use primitives::nfts::{NFTId, NFTSeriesId};

	pub fn create_capsule_fast(owner: Origin) -> NFTId {
		let nft_id = create_nft(owner.clone(), bounded_vec![50], None);
		assert_ok!(Capsule::create_from_nft(owner, nft_id, bounded_vec![60]));
		nft_id
	}

	pub fn create_nft_fast(owner: Origin) -> NFTId {
		create_nft(owner, bounded_vec![50], None)
	}

	pub fn create_nft(
		owner: Origin,
		ipfs_reference: BoundedVec<u8, IPFSLengthLimit>,
		series_id: Option<NFTSeriesId>,
	) -> NFTId {
		assert_ok!(NFT::create(owner, ipfs_reference, series_id));
		NFT::nft_id_generator() - 1
	}
}

#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	t.into()
}
