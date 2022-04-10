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

use crate::{
	self as ternoa_auction,
	types::{AuctionData, BidderList},
	Config,
};
use frame_support::{
	bounded_vec, parameter_types,
	traits::{ConstU32, Contains, GenesisBuild, OnFinalize, OnInitialize},
	PalletId,
};
use primitives::{
	marketplace::{MarketplaceData, MarketplaceType},
	nfts::{NFTData, NFTSeriesDetails},
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type BlockNumber = u64;
pub type AccountId = u64;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;

pub const MIN_AUCTION_DURATION: u64 = 100;
pub const MAX_AUCTION_DURATION: u64 = 1000;
pub const MAX_AUCTION_DELAY: u64 = 50;
pub const AUCTION_GRACE_PERIOD: u64 = 5;
pub const AUCTION_ENDING_PERIOD: u64 = 10;

pub const ALICE_NFT_ID: u32 = 1;
pub const ALICE_SERIES_ID: u8 = 1;
pub const ALICE_MARKET_ID: u32 = 1;

pub const BOB_NFT_ID: u32 = 10;
pub const BOB_SERIES_ID: u8 = 10;
pub const INVALID_NFT_ID: u32 = 404;
pub const MARKETPLACE_COMMISSION_FEE: u8 = 10;

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
		Auction: ternoa_auction,
	}
);

pub enum AuctionState {
	Before,
	InProgress,
	Extended,
}

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
	type BlockNumber = BlockNumber;
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

impl ternoa_marketplace::Config for Test {
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

parameter_types! {
	pub const MinAuctionDuration: BlockNumber = MIN_AUCTION_DURATION;
	pub const MaxAuctionDuration: BlockNumber = MAX_AUCTION_DURATION;
	pub const MaxAuctionDelay: BlockNumber = MAX_AUCTION_DELAY;
	pub const AuctionGracePeriod: BlockNumber = AUCTION_GRACE_PERIOD;
	pub const AuctionEndingPeriod: BlockNumber = AUCTION_ENDING_PERIOD;
	pub const AuctionsPalletId: PalletId = PalletId(*b"tauction");
	pub const BidderListLengthLimit: u32 = 3;
	pub const ParallelAuctionLimit: u32 = 10;
}

impl Config for Test {
	type Event = Event;
	type Currency = Balances;
	type NFTExt = NFT;
	type MarketplaceExt = Marketplace;
	type MaxAuctionDelay = MaxAuctionDelay;
	type MaxAuctionDuration = MaxAuctionDuration;
	type MinAuctionDuration = MinAuctionDuration;
	type AuctionGracePeriod = AuctionGracePeriod;
	type AuctionEndingPeriod = AuctionEndingPeriod;
	type PalletId = AuctionsPalletId;
	type WeightInfo = ();
	type BidderListLengthLimit = BidderListLengthLimit;
	type ParallelAuctionLimit = ParallelAuctionLimit;
}

pub struct ExtBuilder {
	balances: Vec<(u64, u128)>,
	state: Option<AuctionState>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder { balances: Vec::new(), state: None }
	}
}

impl ExtBuilder {
	pub fn new(balances: Vec<(u64, u128)>, state: Option<AuctionState>) -> Self {
		ExtBuilder { balances, state }
	}

	pub fn new_build(
		balances: Vec<(u64, u128)>,
		state: Option<AuctionState>,
	) -> sp_io::TestExternalities {
		Self::new(balances, state).build()
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		Self::build_nfts(&mut t);
		Self::build_market(&mut t);
		Self::build_auction(&mut t, self.state);

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	fn build_nfts(t: &mut sp_runtime::Storage) {
		let alice_nft: NFTData<AccountId, IPFSLengthLimit> =
			NFTData::new_default(ALICE, bounded_vec![10], vec![ALICE_SERIES_ID]);
		let bob_nft: NFTData<AccountId, IPFSLengthLimit> =
			NFTData::new_default(BOB, bounded_vec![10], vec![BOB_SERIES_ID]);

		let alice_series = NFTSeriesDetails::new(ALICE, false);
		let bob_series = NFTSeriesDetails::new(ALICE, false);

		let nfts = vec![alice_nft.to_raw(ALICE_NFT_ID), bob_nft.to_raw(BOB_NFT_ID)];
		let series = vec![
			alice_series.to_raw(vec![ALICE_SERIES_ID]),
			bob_series.to_raw(vec![BOB_SERIES_ID]),
		];

		ternoa_nft::GenesisConfig::<Test> { nfts, series, nft_mint_fee: 5 }
			.assimilate_storage(t)
			.unwrap();
	}

	fn build_market(t: &mut sp_runtime::Storage) {
		let alice_market: MarketplaceData<
			AccountId,
			AccountCountLimit,
			NameLengthLimit,
			URILengthLimit,
			DescriptionLengthLimit,
		> = MarketplaceData::new(
			MarketplaceType::Public,
			MARKETPLACE_COMMISSION_FEE,
			ALICE,
			bounded_vec![],
			bounded_vec![],
			bounded_vec![10],
			bounded_vec![],
			bounded_vec![],
			bounded_vec![],
		);
		let marketplaces = vec![alice_market.to_raw(ALICE_MARKET_ID)];

		ternoa_marketplace::GenesisConfig::<Test> {
			nfts: vec![],
			marketplaces,
			marketplace_mint_fee: 15,
		}
		.assimilate_storage(t)
		.unwrap();
	}

	fn build_auction(t: &mut sp_runtime::Storage, state: Option<AuctionState>) {
		pub const NFT_PRICE: u128 = 100;
		pub const NFT_BUY_PRICE: Option<u128> = Some(200);

		let mut auctions: Vec<(
			u32,
			AuctionData<AccountId, BlockNumber, u128, BidderListLengthLimit>,
		)> = vec![];
		if let Some(state) = state {
			let (start, end, extended) = match state {
				AuctionState::Before => (2, 2 + MAX_AUCTION_DURATION, false),
				AuctionState::InProgress => (1, 1 + MAX_AUCTION_DURATION, false),
				AuctionState::Extended => (1, 1 + MAX_AUCTION_DURATION, true),
			};

			let alice_data = AuctionData {
				creator: ALICE,
				start_block: start,
				end_block: end,
				start_price: NFT_PRICE,
				buy_it_price: NFT_BUY_PRICE.clone(),
				bidders: BidderList::new(),
				marketplace_id: ALICE_MARKET_ID,
				is_extended: extended,
			};

			let bob_data = AuctionData {
				creator: BOB,
				start_block: start,
				end_block: end,
				start_price: NFT_PRICE,
				buy_it_price: NFT_BUY_PRICE.clone(),
				bidders: BidderList::new(),
				marketplace_id: ALICE_MARKET_ID,
				is_extended: extended,
			};

			auctions = vec![(ALICE_NFT_ID, alice_data), (BOB_NFT_ID, bob_data)];
		}

		let auctions = auctions.iter().map(|x| x.1.to_raw(x.0)).collect();
		ternoa_auction::GenesisConfig::<Test> { auctions }
			.assimilate_storage(t)
			.unwrap();
	}
}

#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	ternoa_auction::GenesisConfig::<Test> { auctions: Default::default() }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		Auction::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		Auction::on_initialize(System::block_number());
	}
}
