use crate::{
	self as ternoa_erc20_bridge, Config as PalletERC20BridgeConfig,
	NegativeImbalanceOf,
};

use frame_support::{
	parameter_types,
	traits::{ConstU32, Currency, Everything, SortedMembers},
	weights::Weight,
	PalletId,
};

use frame_system::EnsureRoot;
use sp_core::{hashing::blake2_128, H256};

use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

use chainbridge::{
	constants::DEFAULT_RELAYER_VOTE_THRESHOLD,
	types::{ChainId, ResourceId},
};

// ----------------------------------------------------------------------------
// Types and constants declaration
// ----------------------------------------------------------------------------

type Balance = u64;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) const RELAYER_A: u64 = 0x2;
pub(crate) const RELAYER_B: u64 = 0x3;
pub(crate) const RELAYER_C: u64 = 0x4;
pub(crate) const ENDOWED_BALANCE: u64 = 100_000_000;
pub(crate) const TEST_RELAYER_VOTE_THRESHOLD: u32 = 2;

pub const COLLECTOR: u64 = 99;

// ----------------------------------------------------------------------------
// Mock runtime configuration
// ----------------------------------------------------------------------------

// Build mock runtime
frame_support::construct_runtime!(

	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
		ChainBridge: chainbridge::{Pallet, Call, Storage, Event<T>},
		ERC20Bridge: ternoa_erc20_bridge::{Pallet, Call, Event<T>}
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
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
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

// Parameterize chainbridge pallet
parameter_types! {
	pub const MockChainId: ChainId = 5;
	pub const ChainBridgePalletId: PalletId = PalletId(*b"cb/bridg");
	pub const ProposalLifetime: u64 = 10;
	pub const RelayerVoteThreshold: u32 = DEFAULT_RELAYER_VOTE_THRESHOLD;
}

// Implement chainbridge pallet configuration trait for the mock runtime
impl chainbridge::Config for Test {
	type Event = Event;
	type Proposal = Call;
	type ChainId = MockChainId;
	type PalletId = ChainBridgePalletId;
	type AdminOrigin = EnsureRoot<Self::AccountId>;
	type ProposalLifetime = ProposalLifetime;
	type RelayerVoteThreshold = RelayerVoteThreshold;
	type WeightInfo = ();
}

// Parameterize ERC721 and ERC20Bridge pallets
parameter_types! {
	pub NativeTokenId: ResourceId = chainbridge::derive_resource_id(1, &blake2_128(b"DAV"));
}

// Implement ERC20Bridge pallet configuration trait for the mock runtime
impl PalletERC20BridgeConfig for Test {
	type Event = Event;
	type BridgeOrigin = chainbridge::EnsureBridge<Test>;
	type Currency = Balances;
	type NativeTokenId = NativeTokenId;
	type WeightInfo = ();
	type FeesCollector = MockFeeCollector;
}

pub struct MockFeeCollector;

impl frame_support::traits::OnUnbalanced<NegativeImbalanceOf<Test>> for MockFeeCollector {
	fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<Test>) {
		Balances::resolve_creating(&COLLECTOR, amount);
	}
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

		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		// pre-fill balances
		pallet_balances::GenesisConfig::<Test> {
			balances: vec![(bridge_id, ENDOWED_BALANCE), (RELAYER_A, ENDOWED_BALANCE)],
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		let mut externalities = TestExternalities::new(storage);
		externalities.execute_with(|| System::set_block_number(1));
		externalities
	}
}

// ----------------------------------------------------------------------------
// Helper functions
// ----------------------------------------------------------------------------

pub(crate) mod helpers {

	use super::{Call, Event, Test};

	fn last_event() -> Event {
		frame_system::Pallet::<Test>::events()
			.pop()
			.map(|e| e.event)
			.expect("Event expected")
	}

	pub fn expect_event<E: Into<Event>>(e: E) {
		assert_eq!(last_event(), e.into());
	}

	// Checks events against the latest. A contiguous set of events must be provided. They must
	// include the most recent event, but do not have to include every past event.
	pub fn assert_events(mut expected: Vec<Event>) {
		let mut actual: Vec<Event> =
			frame_system::Pallet::<Test>::events().iter().map(|e| e.event.clone()).collect();

		expected.reverse();

		for evt in expected {
			let next = actual.pop().expect("event expected");
			assert_eq!(next, evt.into(), "Events don't match");
		}
	}

	pub(crate) fn make_transfer_proposal(to: u64, amount: u64) -> Call {
		Call::ERC20Bridge(crate::Call::transfer { to, amount: amount.into() })
	}
} // end of 'helpers' module
