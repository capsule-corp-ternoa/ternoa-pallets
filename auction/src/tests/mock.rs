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
	traits::{ConstU32, Contains, OnFinalize, OnInitialize},
	PalletId,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use crate::{self as ternoa_auction, Config};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type BlockNumber = u64;
pub type AccountId = u64;
pub type Balance = u64;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;

pub const PARALLEL_AUCTION_LIMIT: u32 = 20;
pub const MIN_AUCTION_DURATION: u64 = 100;
pub const MAX_AUCTION_DURATION: u64 = 1000;
pub const MAX_AUCTION_DELAY: u64 = 50;
pub const AUCTION_GRACE_PERIOD: u64 = 5;
pub const AUCTION_ENDING_PERIOD: u64 = 10;
pub const NFT_MINT_FEE: Balance = 10;
pub const SECRET_NFT_MINT_FEE: Balance = 75;
pub const MARKETPLACE_MINT_FEE: Balance = 100;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		NFT: ternoa_nft,
		Auction: ternoa_auction,
		Marketplace: ternoa_marketplace,
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
	type BlockNumber = BlockNumber;
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
	pub const ExistentialDeposit: u64 = 0;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = u128;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
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
	type ClusterSize = ClusterSize;
	type MaxUriLen = MaxUriLen;
	type ListSizeLimit = ListSizeLimit;
}

parameter_types! {
	// NFT parameter types
	pub const NFTInitialMintFee: Balance = NFT_MINT_FEE;
	pub const NFTOffchainDataLimit: u32 = 10;
	pub const CollectionOffchainDataLimit: u32 = 10;
	pub const CollectionSizeLimit: u32 = 10;
	pub const InitialSecretMintFee: Balance = SECRET_NFT_MINT_FEE;
	pub const ShardsNumber: u32 = 5;
	// Marketplace parameter types
	pub const MarketplaceInitialMintFee: Balance = MARKETPLACE_MINT_FEE;
	pub const OffchainDataLimit: u32 = 150;
	pub const AccountSizeLimit: u32 = 100;
	pub const CollectionListSizeLimit: u32 = 100;

}

impl ternoa_nft::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ternoa_nft::weights::TernoaWeight<Test>;
	type Currency = Balances;
	type FeesCollector = ();
	type InitialMintFee = NFTInitialMintFee;
	type NFTOffchainDataLimit = NFTOffchainDataLimit;
	type CollectionOffchainDataLimit = CollectionOffchainDataLimit;
	type CollectionSizeLimit = CollectionSizeLimit;
	type InitialSecretMintFee = InitialSecretMintFee;
	type ShardsNumber = ShardsNumber;
	type TEEExt = TEE;
}

impl ternoa_marketplace::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type NFTExt = NFT;
	type WeightInfo = ();
	type FeesCollector = ();
	type InitialMintFee = MarketplaceInitialMintFee;
	type OffchainDataLimit = OffchainDataLimit;
	type AccountSizeLimit = AccountSizeLimit;
	type CollectionSizeLimit = CollectionListSizeLimit;
}

parameter_types! {
	pub const MinAuctionDuration: BlockNumber = MIN_AUCTION_DURATION;
	pub const MaxAuctionDuration: BlockNumber = MAX_AUCTION_DURATION;
	pub const MaxAuctionDelay: BlockNumber = MAX_AUCTION_DELAY;
	pub const AuctionGracePeriod: BlockNumber = AUCTION_GRACE_PERIOD;
	pub const AuctionEndingPeriod: BlockNumber = AUCTION_ENDING_PERIOD;
	pub const AuctionsPalletId: PalletId = PalletId(*b"tauction");
	pub const BidderListLengthLimit: u32 = 3;
	pub const ParallelAuctionLimit: u32 = PARALLEL_AUCTION_LIMIT;
	pub const ActionsInBlockLimit: u32 = 10;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
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
	type ActionsInBlockLimit = ActionsInBlockLimit;
}

pub struct ExtBuilder {
	balances: Vec<(u64, u128)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder { balances: Vec::new() }
	}
}

impl ExtBuilder {
	pub fn new(balances: Vec<(u64, u128)>) -> Self {
		ExtBuilder { balances }
	}

	pub fn new_build(balances: Option<Vec<(u64, u128)>>) -> sp_io::TestExternalities {
		Self::new(balances.unwrap_or_else(|| {
			vec![(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000), (DAVE, 1_000), (EVE, 1_000)]
		}))
		.build()
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	t.into()
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
