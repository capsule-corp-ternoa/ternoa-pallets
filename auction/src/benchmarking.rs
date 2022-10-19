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

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::{Auctions as AuctionsStorage, Claims, Pallet as TernoaAuctions};
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	assert_ok,
	traits::{Currency, OnFinalize, OnInitialize},
};
use frame_system::{pallet_prelude::OriginFor, Pallet as System, RawOrigin};
use primitives::{
	marketplace::{MarketplaceData, MarketplaceId, MarketplaceType},
	nfts::NFTId,
};
use sp_arithmetic::per_things::Permill;
use sp_runtime::traits::Bounded;
use sp_std::prelude::*;
use ternoa_common::traits::{MarketplaceExt, NFTExt};

use crate::Pallet as Auction;

pub enum AuctionState {
	Before,
	InProgress,
	Extended,
}

pub struct BenchmarkData {
	pub alice_nft_id: NFTId,
	pub alice_marketplace_id: MarketplaceId,
	pub bob_nft_id: NFTId,
}

const PERCENT_0: Permill = Permill::from_parts(0);
const PERCENT_20: Permill = Permill::from_parts(200000);

pub fn prepare_benchmarks<T: Config>(state: Option<AuctionState>) -> BenchmarkData {
	// Get accounts
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");
	let charlie: T::AccountId = get_account::<T>("CHARLIE");
	let eve: T::AccountId = get_account::<T>("EVE");

	// Give them enough caps
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value() / 5u32.into()); // to avoid overflow for auction owner
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value() / 5u32.into());
	T::Currency::make_free_balance_be(&charlie, BalanceOf::<T>::max_value() / 5u32.into());
	T::Currency::make_free_balance_be(&eve, BalanceOf::<T>::max_value() / 5u32.into());

	// Create Alice's marketplace
	let marketplace_id = 0u32;
	let marketplace_data = MarketplaceData::new(
		alice.clone(),
		MarketplaceType::Public,
		Some(CompoundFee::Percentage(PERCENT_20)),
		None,
		None,
		None,
		None,
	);
	T::MarketplaceExt::set_marketplace(marketplace_id, marketplace_data).unwrap();

	// Create NFTs
	let alice_nft_id =
		T::NFTExt::create_nft(alice, BoundedVec::default(), PERCENT_0, None, false).unwrap();
	let bob_nft_id =
		T::NFTExt::create_nft(bob, BoundedVec::default(), PERCENT_0, None, false).unwrap();

	// Create auctions
	if let Some(state) = state {
		let (start_block, is_extended) = match state {
			AuctionState::Before =>
				(System::<T>::block_number() + T::MaxAuctionDelay::get(), false),
			AuctionState::InProgress => (System::<T>::block_number(), false),
			AuctionState::Extended => (System::<T>::block_number(), true),
		};

		let end_block = start_block + T::MinAuctionDuration::get() + 10u32.into();
		let start_price = BalanceOf::<T>::max_value() / 1000u32.into();
		let buy_it_price = Some(start_price.saturating_mul(2u16.into()));

		assert_ok!(TernoaAuctions::<T>::create_auction(
			origin::<T>("BOB"),
			bob_nft_id,
			marketplace_id,
			start_block,
			end_block,
			start_price,
			buy_it_price,
		));

		AuctionsStorage::<T>::mutate(bob_nft_id, |x| {
			let mut x = x.as_mut().unwrap();
			x.is_extended = is_extended;
		});
	}

	BenchmarkData { alice_nft_id, alice_marketplace_id: marketplace_id, bob_nft_id }
}

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}

pub fn origin<T: Config>(name: &'static str) -> OriginFor<T> {
	RawOrigin::Signed(get_account::<T>(name)).into()
}

#[allow(dead_code)]
pub fn run_to_block<T: Config>(n: T::BlockNumber) {
	while System::<T>::block_number() < n {
		<TernoaAuctions<T> as OnFinalize<T::BlockNumber>>::on_finalize(System::<T>::block_number());
		<System<T> as OnFinalize<T::BlockNumber>>::on_finalize(System::<T>::block_number());
		System::<T>::set_block_number(System::<T>::block_number() + 1u16.into());
		<System<T> as OnInitialize<T::BlockNumber>>::on_initialize(System::<T>::block_number());
		<TernoaAuctions<T> as OnInitialize<T::BlockNumber>>::on_initialize(
			System::<T>::block_number(),
		);
	}
}

benchmarks! {
	create_auction {
		let s in 0 .. T::ParallelAuctionLimit::get() - 2;
		let bench_data = prepare_benchmarks::<T>(None);
		Auction::<T>::fill_deadline_queue(s, 99u32.into(), 10u32.into()).unwrap();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let nft_id = bench_data.alice_nft_id;
		let marketplace_id = bench_data.alice_marketplace_id;
		let start_block = System::<T>::block_number() + T::MaxAuctionDelay::get();
		let end_block = start_block + T::MinAuctionDuration::get();
		let start_price = BalanceOf::<T>::max_value() / 100u32.into();
		let buy_now_price = start_price.saturating_mul(2u16.into());

	}: _(RawOrigin::Signed(alice), nft_id, marketplace_id, start_block, end_block, start_price, Some(buy_now_price))
	verify {
		assert_eq!(T::NFTExt::get_nft(nft_id).unwrap().state.is_listed, true);
	}

	 cancel_auction {
		let s in 0 .. T::ParallelAuctionLimit::get() - 1;
		let bench_data = prepare_benchmarks::<T>(Some(AuctionState::Before));
		Auction::<T>::fill_deadline_queue(s, 99u32.into(), 10u32.into()).unwrap();
		let bob: T::AccountId = get_account::<T>("BOB");
		let nft_id = bench_data.bob_nft_id;

	}: _(RawOrigin::Signed(bob), nft_id)
	verify {
		assert_eq!(T::NFTExt::get_nft(nft_id).unwrap().state.is_listed, false);
	}

	end_auction {
		let s in 0 .. T::ParallelAuctionLimit::get() - 1;
		let bench_data = prepare_benchmarks::<T>(Some(AuctionState::Extended));
		Auction::<T>::fill_deadline_queue(s, 99u32.into(), 10u32.into()).unwrap();
		let bob: T::AccountId = get_account::<T>("BOB");
		let nft_id = bench_data.bob_nft_id;
		let auction = AuctionsStorage::<T>::get(nft_id).unwrap();
		let charlie_bid = auction.buy_it_price.unwrap();
		let eve_bid = charlie_bid.saturating_mul(2u16.into());

		assert_ok!(TernoaAuctions::<T>::add_bid(origin::<T>("CHARLIE"), nft_id, charlie_bid));
		assert_ok!(TernoaAuctions::<T>::add_bid(origin::<T>("EVE"), nft_id, eve_bid));
	}: _(RawOrigin::Signed(bob), nft_id)
	verify {
		let eve: T::AccountId = get_account::<T>("EVE");
		let nft = T::NFTExt::get_nft(nft_id).unwrap();
		assert_eq!(nft.state.is_listed, false);
		assert_eq!(nft.owner, eve);
	}

	add_bid {
		let s in 0 .. T::BidderListLengthLimit::get() - 1;
		let bench_data = prepare_benchmarks::<T>(Some(AuctionState::InProgress));
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let eve: T::AccountId = get_account::<T>("EVE");
		let nft_id = bench_data.bob_nft_id;
		Auction::<T>::fill_bidders_list(s, nft_id, eve, 10u32.into()).unwrap();

		let auction = AuctionsStorage::<T>::get(nft_id).unwrap();
		let charlie_bid =  auction.buy_it_price.unwrap();

	}: _(RawOrigin::Signed(charlie.clone()), nft_id, charlie_bid)
	verify {
		let auction = AuctionsStorage::<T>::get(nft_id).unwrap();
		assert!(auction.bidders.list.contains(&(charlie, charlie_bid)))
	}

	remove_bid {
		let s in 0 .. T::BidderListLengthLimit::get() - 1;
		let bench_data = prepare_benchmarks::<T>(Some(AuctionState::InProgress));
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let eve: T::AccountId = get_account::<T>("EVE");
		let nft_id = bench_data.bob_nft_id;
		Auction::<T>::fill_bidders_list(s, nft_id, eve, 10u32.into()).unwrap();

		let auction = AuctionsStorage::<T>::get(nft_id).unwrap();
		let charlie_bid =  auction.buy_it_price.unwrap();
		assert_ok!(TernoaAuctions::<T>::add_bid(origin::<T>("CHARLIE"), nft_id, charlie_bid));
	}: _(RawOrigin::Signed(charlie.clone()), nft_id)
	verify {
		assert!(!auction.bidders.list.contains(&(charlie, charlie_bid)))
	}

	buy_it_now {
		let s in 0 .. T::ParallelAuctionLimit::get() - 1;
		let bench_data = prepare_benchmarks::<T>(Some(AuctionState::InProgress));
		Auction::<T>::fill_deadline_queue(s, 99u32.into(), 10u32.into()).unwrap();
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let nft_id = bench_data.bob_nft_id;

	}: _(RawOrigin::Signed(charlie.clone()), nft_id)
	verify {
		let nft = T::NFTExt::get_nft(nft_id).unwrap();
		assert_eq!(nft.state.is_listed, false);
		assert_eq!(nft.owner, charlie);
	}

	claim {
		let bench_data = prepare_benchmarks::<T>(Some(AuctionState::InProgress));
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let nft_id = bench_data.bob_nft_id;

		let auction = AuctionsStorage::<T>::get(nft_id).unwrap();
		let charlie_bid = auction.buy_it_price.unwrap();
		let eve_bid = charlie_bid.saturating_mul(2u16.into());

		assert_ok!(TernoaAuctions::<T>::add_bid(origin::<T>("CHARLIE"), nft_id, charlie_bid));
		assert_ok!(TernoaAuctions::<T>::add_bid(origin::<T>("EVE"), nft_id, eve_bid));

		run_to_block::<T>(auction.end_block + 1u32.into());
	}: _(RawOrigin::Signed(charlie.clone()))
	verify {
		assert_eq!(Claims::<T>::get(charlie), None);
	}
}

impl_benchmark_test_suite!(
	TernoaAuctions,
	crate::tests::mock::new_test_ext(),
	crate::tests::mock::Test
);
