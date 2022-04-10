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
	bounded_vec, parameter_types,
	traits::{ConstU32, Contains, GenesisBuild},
};
use primitives::marketplace::{MarketplaceType, MarketplacesGenesis};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use crate::{
	self as ternoa_marketplace, AccountVec, Config, DescriptionVec, MarketplaceData, NameVec,
	URIVec,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const DAVE: u64 = 3;

pub type AccountId = u64;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		NFT: ternoa_nft,
		Marketplace: ternoa_marketplace,
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
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
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
	pub const ExistentialDeposit: u64 = 0;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = u128;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
}

parameter_types! {
	pub const IPFSLengthLimit: u32 = 5;
	pub const AccountCountLimit: u32 = 5;
	pub const NameLengthLimit: u32 = 5;
	pub const URILengthLimit: u32 = 5;
	pub const DescriptionLengthLimit: u32 = 5;
}

impl ternoa_nft::Config for Test {
	type Event = Event;
	type WeightInfo = ternoa_nft::weights::TernoaWeight<Test>;
	type Currency = Balances;
	type FeesCollector = ();
	type IPFSLengthLimit = IPFSLengthLimit;
}

impl Config for Test {
	type Event = Event;
	type Currency = Balances;
	type NFTExt = NFT;
	type WeightInfo = ();
	type FeesCollector = ();
	type AccountCountLimit = AccountCountLimit;
	type NameLengthLimit = NameLengthLimit;
	type URILengthLimit = URILengthLimit;
	type DescriptionLengthLimit = DescriptionLengthLimit;
}

pub struct ExtBuilder {
	nfts: Vec<primitives::nfts::NFTsGenesis<u64>>,
	series: Vec<primitives::nfts::SeriesGenesis<u64>>,
	caps_endowed_accounts: Vec<(u64, u128)>,
	marketplaces: Vec<MarketplacesGenesis<u64>>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder {
			nfts: Vec::new(),
			series: Vec::new(),
			caps_endowed_accounts: Vec::new(),
			marketplaces: Vec::new(),
		}
	}
}

impl ExtBuilder {
	pub fn caps(mut self, accounts: Vec<(u64, u128)>) -> Self {
		for account in accounts {
			self.caps_endowed_accounts.push(account);
		}
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: self.caps_endowed_accounts }
			.assimilate_storage(&mut t)
			.unwrap();

		ternoa_nft::GenesisConfig::<Test> {
			nfts: self.nfts,
			series: self.series,
			nft_mint_fee: 10,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let empty_list: AccountVec<Test> = bounded_vec![];
		let name: NameVec<Test> = bounded_vec![50, 50, 50, 50];
		let uri: URIVec<Test> = bounded_vec![];
		let description: DescriptionVec<Test> = bounded_vec![];
		let market = MarketplaceData::new(
			MarketplaceType::Public,
			0,
			ALICE,
			empty_list.clone(),
			empty_list.clone(),
			name,
			uri.clone(),
			uri.clone(),
			description,
		);
		let mut marketplaces: Vec<MarketplacesGenesis<u64>> = vec![market.to_raw(0)];
		marketplaces.extend(self.marketplaces);

		ternoa_marketplace::GenesisConfig::<Test> {
			nfts: Default::default(),
			marketplaces,
			marketplace_mint_fee: 250,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	/*     pub fn build_v6_migration(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	} */
}

pub mod help {
	use crate::{MarketplaceId, NameVec};

	use super::*;
	use frame_support::assert_ok;
	use primitives::nfts::{NFTId, NFTSeriesId};

	pub fn create_nft(
		owner: Origin,
		ipfs_reference: primitives::nfts::IPFSReference<IPFSLengthLimit>,
		series_id: Option<NFTSeriesId>,
	) -> NFTId {
		assert_ok!(NFT::create(owner, ipfs_reference, series_id));
		return NFT::nft_id_generator() - 1
	}

	pub fn create_nft_and_lock_series(
		owner: Origin,
		ipfs_reference: primitives::nfts::IPFSReference<IPFSLengthLimit>,
		series_id: NFTSeriesId,
	) -> NFTId {
		let nft_id = help::create_nft(owner.clone(), ipfs_reference, Some(series_id.clone()));
		help::finish_series(owner.clone(), series_id.clone());

		nft_id
	}

	pub fn create_mkp(
		owner: Origin,
		kind: MarketplaceType,
		fee: u8,
		name: NameVec<Test>,
		list: Vec<u64>,
	) -> MarketplaceId {
		assert_ok!(Marketplace::create_marketplace(
			owner.clone(),
			kind,
			fee,
			name,
			bounded_vec![],
			bounded_vec![],
			bounded_vec![],
		));
		let mkp_id = Marketplace::marketplace_id_generator();

		for acc in list {
			match kind {
				MarketplaceType::Private => {
					let ok = Marketplace::add_account_to_allow_list(owner.clone(), mkp_id, acc);
					assert_ok!(ok);
				},
				MarketplaceType::Public => {
					let ok = Marketplace::add_account_to_disallow_list(owner.clone(), mkp_id, acc);
					assert_ok!(ok);
				},
			}
		}

		return Marketplace::marketplace_id_generator()
	}

	pub fn finish_series(owner: Origin, series_id: Vec<u8>) {
		assert_ok!(NFT::finish_series(owner, series_id));
	}
}

#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	t.into()
}
