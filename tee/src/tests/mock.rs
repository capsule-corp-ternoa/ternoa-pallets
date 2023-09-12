// Copyright 2023 Capsule Corp (France) SAS.
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

use frame_election_provider_support::{onchain, SequentialPhragmen};
use frame_support::{
	parameter_types,
	traits::{ConstU32, ConstU64, Contains, Get, Hooks},
	PalletId,
};
use sp_core::H256;
use sp_runtime::{
	curve::PiecewiseLinear,
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Zero},
	Perbill,
};
use sp_staking::{EraIndex, SessionIndex};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = u64;

pub const INIT_TIMESTAMP: u64 = 30_000;
pub const BLOCK_TIME: u64 = 1000;

use crate::{self as tee, Config};

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		TEE: tee,
		Timestamp: pallet_timestamp,
		Staking: pallet_staking,
		Session: pallet_session,
		Historical: pallet_session::historical,
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
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
}
pub type Balance = u64;

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1>;
	type WeightInfo = ();
}

impl pallet_session::historical::Config for Test {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Test>;
}

sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub foo: sp_runtime::testing::UintAuthorityId,
	}
}

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<AccountId> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[];

	fn on_genesis_session<Ks: sp_runtime::traits::OpaqueKeys>(_validators: &[(AccountId, Ks)]) {}

	fn on_new_session<Ks: sp_runtime::traits::OpaqueKeys>(
		_: bool,
		_: &[(AccountId, Ks)],
		_: &[(AccountId, Ks)],
	) {
	}

	fn on_disabled(_: u32) {}
}

parameter_types! {
	pub const Period: u64 = 1;
	pub const Offset: u64 = 0;
}

/// Custom `SessionHandler` since we use `TestSessionKeys` as `Keys`.
impl pallet_session::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = u64;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = pallet_session::PeriodicSessions<ConstU64<1>, ConstU64<0>>;
	type NextSessionRotation = pallet_session::PeriodicSessions<ConstU64<1>, ConstU64<0>>;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Test, Staking>;
	type SessionHandler = TestSessionHandler;
	type Keys = SessionKeys;
	type WeightInfo = ();
}

pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000u64,
		max_inflation: 0_100_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	pub const SessionsPerEra: SessionIndex = 3;
	pub const BondingDuration: EraIndex = 3;
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Test;
	type Solver = SequentialPhragmen<u64, Perbill>;
	type DataProvider = Staking;
	type WeightInfo = ();
}

impl pallet_staking::Config for Test {
	type MaxNominations = ConstU32<16>;
	type RewardRemainder = ();
	type CurrencyToVote = frame_support::traits::SaturatingCurrencyToVote;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
	type Slash = ();
	type Reward = ();
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = ();
	type SlashCancelOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type SessionInterface = Self;
	type UnixTime = pallet_timestamp::Pallet<Test>;
	type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
	type MaxNominatorRewardedPerValidator = ConstU32<64>;
	type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
	type NextNewSession = Session;
	type ElectionProvider = onchain::UnboundedExecution<OnChainSeqPhragmen>;
	type GenesisElectionProvider = Self::ElectionProvider;
	type VoterList = pallet_staking::UseNominatorsAndValidatorsMap<Self>;
	type TargetList = pallet_staking::UseValidatorsMap<Self>;
	type MaxUnlockingChunks = ConstU32<32>;
	type HistoryDepth = ConstU32<84>;
	type OnStakerSlash = ();
	type BenchmarkingConfig = pallet_staking::TestBenchmarkingConfig;
	type WeightInfo = ();
}
parameter_types! {
	pub const ClusterSize: u32 = 2;
	pub const MaxUriLen: u32 = 12;
	pub const ListSizeLimit: u32 = 10;
	pub const TeeBondingDuration: u32 = 10;
	pub const InitialStakingAmount: Balance = 20;
	pub const InitalDailyRewardPool: Balance = 2000;
	pub const TeePalletId: PalletId = PalletId(*b"tern/tee");
	pub const TeeHistoryDepth: u32 = 50;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type TeeWeightInfo = ();
	// type Currency = Balances;
	// type CurrencyBalance = <Self as pallet_balances::Config>::Balance;
	type ClusterSize = ClusterSize;
	type MaxUriLen = MaxUriLen;
	type ListSizeLimit = ListSizeLimit;
	type TeeBondingDuration = TeeBondingDuration;
	type InitialStakingAmount = InitialStakingAmount;
	type InitalDailyRewardPool = InitalDailyRewardPool;
	type PalletId = TeePalletId;
	type TeeHistoryDepth = TeeHistoryDepth;
}

// Do not use the `0` account id since this would be the default value
// for our account id. This would mess with some tests.
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const ALICE_ENCLAVE: u64 = 5;
pub const BOB_ENCLAVE: u64 = 6;
pub const CHARLIE_ENCLAVE: u64 = 7;
pub const EVE: u64 = 8;

pub struct ExtBuilder {
	endowed_accounts: Vec<(u64, u64)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder { endowed_accounts: Vec::new() }
	}
}

impl ExtBuilder {
	pub fn tokens(mut self, accounts: Vec<(u64, u64)>) -> Self {
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

/// Progress until the given era.
pub(crate) fn start_active_era(era_index: EraIndex) {
	start_session((era_index * <SessionsPerEra as Get<u32>>::get()).into());
	// assert_eq!(Staking::active_era().unwrap().index, era_index);
	// One way or another, current_era must have changed before the active era, so they must match
	// at this point.
	// assert_eq!(Staking::current_era().unwrap(), Staking::active_era().unwrap().index);
}

/// Progresses from the current block number (whatever that may be) to the `P * session_index + 1`.
pub(crate) fn start_session(session_index: SessionIndex) {
	let end: u64 = if Offset::get().is_zero() {
		(session_index as u64) * Period::get()
	} else {
		Offset::get() + (session_index.saturating_sub(1) as u64) * Period::get()
	};
	run_to_block(end);
	// session must have progressed properly.
	// assert_eq!(
	// 	Session::current_index(),
	// 	session_index,
	// 	"current session index = {}, expected = {}",
	// 	Session::current_index(),
	// 	session_index,
	// );
}

pub fn run_to_block(n: u64) {
	// while System::block_number() < n {
	// 	Staking::on_finalize(System::block_number());
	// 	System::on_finalize(System::block_number());
	// 	System::set_block_number(System::block_number() + 1);
	// 	System::on_initialize(System::block_number());
	// 	Staking::on_initialize(System::block_number());
	// 	Staking::on_finalize(System::block_number());
	// }
	Staking::on_finalize(System::block_number());
	for b in (System::block_number() + 1)..=n {
		System::set_block_number(b);
		Session::on_initialize(b);
		<Staking as Hooks<u64>>::on_initialize(b);
		Timestamp::set_timestamp(System::block_number() * BLOCK_TIME + INIT_TIMESTAMP);
		if b != n {
			Staking::on_finalize(System::block_number());
		}
	}
}
