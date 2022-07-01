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
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite};
use frame_support::{assert_ok, traits::Currency};
use frame_system::RawOrigin;
use sp_arithmetic::per_things::Permill;
use sp_runtime::traits::{Bounded, StaticLookup};
use sp_std::prelude::*;
use ternoa_common::traits::NFTExt;

use crate::Pallet as Marketplace;

pub struct BenchmarkData {
	marketplace_id: MarketplaceId,
	nft_id: NFTId,
}

const PERCENT_50: Permill = Permill::from_parts(500000);

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}

pub fn origin<T: Config>(name: &'static str) -> RawOrigin<T::AccountId> {
	RawOrigin::Signed(get_account::<T>(name))
}

pub fn prepare_benchmarks<T: Config>() -> BenchmarkData {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	// Give them enough caps
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());

	// Create default NFT.
	let nft_id =
		T::NFTExt::create_nft(alice, BoundedVec::default(), PERCENT_50, None, false).unwrap();

	// Create default marketplace.
	assert_ok!(Marketplace::<T>::create_marketplace(
		origin::<T>("ALICE").into(),
		MarketplaceType::Public,
	));

	BenchmarkData { nft_id, marketplace_id: Marketplace::<T>::next_marketplace_id() - 1 }
}

benchmarks! {
	create_marketplace {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
	}: _(origin::<T>("ALICE"), MarketplaceType::Public)
	verify {
		let marketplace_id = Marketplace::<T>::next_marketplace_id() - 1;
		assert_eq!(Marketplaces::<T>::get(marketplace_id).unwrap().owner, alice);
	}

	set_marketplace_owner {
		let benchmark_data = prepare_benchmarks::<T>();
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());
	}: _(origin::<T>("ALICE"), benchmark_data.marketplace_id, bob_lookup)
	verify {
		assert_eq!(Marketplaces::<T>::get(benchmark_data.marketplace_id).unwrap().owner, bob);
	}

	set_marketplace_kind {
		let benchmark_data = prepare_benchmarks::<T>();
	}: _(origin::<T>("ALICE"), benchmark_data.marketplace_id, MarketplaceType::Private)
	verify {
		assert_eq!(Marketplaces::<T>::get(benchmark_data.marketplace_id).unwrap().kind, MarketplaceType::Private);
	}

	set_marketplace_configuration{
		let benchmark_data = prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let marketplace_offchain_data =
			BoundedVec::try_from(vec![1; T::OffchainDataLimit::get() as usize]).unwrap();
		let marketplace_account_list: BoundedVec<T::AccountId, T::AccountSizeLimit> =
			BoundedVec::try_from(vec![alice.clone(); (T::AccountSizeLimit::get() / 100) as usize]).unwrap();
	}: _(origin::<T>("ALICE"), benchmark_data.marketplace_id, ConfigOp::Set(CompoundFee::Percentage(PERCENT_50)), ConfigOp::Set(CompoundFee::Percentage(PERCENT_50)), ConfigOp::Set(marketplace_account_list.clone()), ConfigOp::Set(marketplace_offchain_data.clone()))
	verify {
		let marketplace = Marketplaces::<T>::get(benchmark_data.marketplace_id).unwrap();
		assert_eq!(marketplace.commission_fee, Some(CompoundFee::Percentage(PERCENT_50)));
		assert_eq!(marketplace.listing_fee, Some(CompoundFee::Percentage(PERCENT_50)));
		assert_eq!(marketplace.account_list, Some(marketplace_account_list));
		assert_eq!(marketplace.offchain_data, Some(marketplace_offchain_data));
	}

	set_marketplace_mint_fee {
		let old_mint_fee = Marketplace::<T>::marketplace_mint_fee();
		let new_mint_fee = 20u32;
	}: _(RawOrigin::Root, new_mint_fee.clone().into())
	verify {
		assert_ne!(old_mint_fee, new_mint_fee.clone().into());
		assert_eq!(Marketplace::<T>::marketplace_mint_fee(), new_mint_fee.into());
	}

	list_nft {
		let benchmark_data = prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		Marketplace::<T>::set_marketplace_configuration(
			origin::<T>("ALICE").into(),
			benchmark_data.marketplace_id,
			ConfigOp::Set(CompoundFee::Percentage(PERCENT_50)),
			ConfigOp::Noop,
			ConfigOp::Noop,
			ConfigOp::Noop,
		).unwrap();
	}: _(origin::<T>("ALICE"), benchmark_data.nft_id, benchmark_data.marketplace_id, 10u32.into())
	verify {
		assert_eq!(T::NFTExt::get_nft(benchmark_data.nft_id).unwrap().state.listed_for_sale, true);
		assert!(Marketplace::<T>::listed_nfts(benchmark_data.nft_id).is_some());
	}

	unlist_nft {
		let benchmark_data = prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		Marketplace::<T>::list_nft(origin::<T>("ALICE").into(), benchmark_data.marketplace_id, benchmark_data.nft_id, 10u32.into()).unwrap();
	}: _(origin::<T>("ALICE"), benchmark_data.nft_id)
	verify {
		assert_eq!(T::NFTExt::get_nft(benchmark_data.nft_id).unwrap().state.listed_for_sale, false);
		assert!(Marketplace::<T>::listed_nfts(benchmark_data.nft_id).is_none());
	}

	buy_nft {
		let benchmark_data = prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_origin = origin::<T>("BOB");
		Marketplace::<T>::set_marketplace_configuration(
			origin::<T>("ALICE").into(),
			benchmark_data.marketplace_id,
			ConfigOp::Set(CompoundFee::Percentage(PERCENT_50)),
			ConfigOp::Noop,
			ConfigOp::Noop,
			ConfigOp::Noop,
		).unwrap();
		let nft_id = T::NFTExt::create_nft(alice, BoundedVec::default(), PERCENT_50, None, false).unwrap();
		Marketplace::<T>::list_nft(origin::<T>("ALICE").into(), benchmark_data.nft_id, benchmark_data.marketplace_id, 10u32.into()).unwrap();
	}: _(bob_origin, benchmark_data.nft_id)
	verify {
		assert!(Marketplace::<T>::listed_nfts(benchmark_data.nft_id).is_none());
		assert_eq!(T::NFTExt::get_nft(benchmark_data.nft_id).unwrap().owner, bob);
	}
}

impl_benchmark_test_suite!(
	Marketplace,
	crate::tests::mock::new_test_ext(),
	crate::tests::mock::Test
);
