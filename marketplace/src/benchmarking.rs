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
use frame_support::{assert_ok, bounded_vec, traits::Currency};
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, StaticLookup};
use sp_std::prelude::*;
use ternoa_common::traits::NFTExt;

use crate::Pallet as Marketplace;

const SERIES_ID: u8 = 20;

pub fn prepare_benchmarks<T: Config>() -> (MarketplaceId, MarketplaceId, NFTId) {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	// Give them enough caps
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());

	// Create default NFT and series
	let series_id = vec![SERIES_ID];
	let nft_id =
		T::NFTExt::create_nft(alice.clone(), bounded_vec![1], Some(series_id.clone()), 0).unwrap();

	// Lock series
	T::NFTExt::benchmark_lock_series(series_id.clone());

	// Create Public Marketplace for Alice
	assert_ok!(Marketplace::<T>::create_marketplace(
		get_origin::<T>("ALICE").into(),
		MarketplaceType::Public,
		0,
		bounded_vec![50],
		bounded_vec![],
		bounded_vec![],
		bounded_vec![],
	));
	let public_id = Marketplace::<T>::marketplace_id_generator();

	// Create Private Marketplace for Alice
	assert_ok!(Marketplace::<T>::create_marketplace(
		get_origin::<T>("ALICE").into(),
		MarketplaceType::Private,
		0,
		bounded_vec![51],
		bounded_vec![],
		bounded_vec![],
		bounded_vec![],
	));
	let private_id = Marketplace::<T>::marketplace_id_generator();

	(public_id, private_id, nft_id)
}

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}

pub fn get_origin<T: Config>(name: &'static str) -> RawOrigin<T::AccountId> {
	RawOrigin::Signed(get_account::<T>(name))
}

benchmarks! {
	list_nft {
		let (mkp_id, _, nft_id) = prepare_benchmarks::<T>();

		let alice: T::AccountId = get_account::<T>("ALICE");
		let price: BalanceOf<T> = 100u32.into();

	}: _(RawOrigin::Signed(alice.clone()), nft_id, price, Some(mkp_id))
	verify {
		assert_eq!(T::NFTExt::owner(nft_id), Some(alice));
		assert_eq!(NFTsForSale::<T>::contains_key(nft_id), true);
	}

	unlist_nft {
		let (mkp_id, _, nft_id) = prepare_benchmarks::<T>();

		let alice = get_origin::<T>("ALICE");
		let price: BalanceOf<T> = 100u32.into();
		drop(Marketplace::<T>::list_nft(alice.clone().into(), nft_id, price, Some(mkp_id)));

	}: _(alice.clone(), nft_id)
	verify {
		assert_eq!(NFTsForSale::<T>::contains_key(nft_id), false);
	}

	buy_nft {
		let (mkp_id, _, nft_id) = prepare_benchmarks::<T>();

		let bob: T::AccountId = get_account::<T>("BOB");
		let price: BalanceOf<T> = 0u32.into();

		drop(Marketplace::<T>::list_nft(get_origin::<T>("ALICE").into(), nft_id, price, Some(mkp_id)));
	}: _(RawOrigin::Signed(bob.clone().into()), nft_id)
	verify {
		assert_eq!(T::NFTExt::owner(nft_id), Some(bob));
		assert_eq!(NFTsForSale::<T>::contains_key(nft_id), false);
	}

	create_marketplace {
		prepare_benchmarks::<T>();

		let alice: T::AccountId = get_account::<T>("ALICE");
		let mkp_id = Marketplace::<T>::marketplace_id_generator() + 1;
	}: _(RawOrigin::Signed(alice.clone().into()), MarketplaceType::Public, 0, bounded_vec![20, 30, 40], bounded_vec![], bounded_vec![], bounded_vec![])
	verify {
		assert_eq!(Marketplaces::<T>::contains_key(mkp_id), true);
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().owner, alice);
		assert_eq!(MarketplaceIdGenerator::<T>::get(), mkp_id);
	}

	add_account_to_allow_list {
		let (_, mkp_id, _) = prepare_benchmarks::<T>();

		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());


	}: _(get_origin::<T>("ALICE"), mkp_id, bob_lookup.into())
	verify {
		let allow_list: AccountVec<T> = bounded_vec![bob];
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().allow_list, allow_list);
	}

	remove_account_from_allow_list {
		let (_, mkp_id, _) = prepare_benchmarks::<T>();

		let alice = get_origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());
		drop(Marketplace::<T>::add_account_to_allow_list(alice.clone().into(), mkp_id, bob_lookup.clone()));

	}: _(alice.clone(), mkp_id, bob_lookup)
	verify {
		let allow_list: AccountVec<T> = bounded_vec![];
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().allow_list, allow_list);
	}

	set_marketplace_owner {
		let (mkp_id, ..) = prepare_benchmarks::<T>();

		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());

	}: _(get_origin::<T>("ALICE"), mkp_id, bob_lookup)
	verify {
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().owner, bob);
	}

	set_marketplace_type {
		let (mkp_id, ..) = prepare_benchmarks::<T>();

	}: _(get_origin::<T>("ALICE"), mkp_id, MarketplaceType::Private)
	verify {
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().kind, MarketplaceType::Private);
	}

	set_marketplace_name {
		let (mkp_id, ..) = prepare_benchmarks::<T>();

		let new_name: NameVec<T> = bounded_vec![40, 30, 20];
	}: _(get_origin::<T>("ALICE"), mkp_id, new_name.clone())
	verify {
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().name, new_name);
	}

	set_marketplace_mint_fee {
		prepare_benchmarks::<T>();

		let old_mint_fee = Marketplace::<T>::marketplace_mint_fee();
		let new_mint_fee = 1000u32;

	}: _(RawOrigin::Root, new_mint_fee.clone().into())
	verify {
		assert_ne!(old_mint_fee, new_mint_fee.clone().into());
		assert_eq!(Marketplace::<T>::marketplace_mint_fee(), new_mint_fee.into());
	}

	set_marketplace_commission_fee {
		let (mkp_id, ..) = prepare_benchmarks::<T>();

		let commission_fee = 67;
	}: _(get_origin::<T>("ALICE"), mkp_id, commission_fee)
	verify {
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().commission_fee, commission_fee);
	}

	set_marketplace_uri {
		let (mkp_id, ..) = prepare_benchmarks::<T>();

		let uri = BoundedVec::try_from("test".as_bytes().to_vec()).unwrap();
	}: _(get_origin::<T>("ALICE"), mkp_id, uri.clone())
	verify {
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().uri, uri);
	}

	set_marketplace_logo_uri {
		let (mkp_id, ..) = prepare_benchmarks::<T>();

		let uri = BoundedVec::try_from("test".as_bytes().to_vec()).unwrap();
	}: _(get_origin::<T>("ALICE"), mkp_id, uri.clone())
	verify {
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().logo_uri, uri);
	}

	add_account_to_disallow_list {
		let (mkp_id, ..) = prepare_benchmarks::<T>();

		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());

	}: _(get_origin::<T>("ALICE"), mkp_id, bob_lookup.into())
	verify {
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().disallow_list, vec![bob]);
	}

	remove_account_from_disallow_list {
		let (mkp_id, ..) = prepare_benchmarks::<T>();

		let alice = get_origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());

		drop(Marketplace::<T>::add_account_to_disallow_list(alice.clone().into(), mkp_id, bob_lookup.clone()));

	}: _(alice.clone(), 1, bob_lookup.into())
	verify {
		assert_eq!(Marketplaces::<T>::get(mkp_id).unwrap().disallow_list, vec![]);
	}
}

impl_benchmark_test_suite!(
	Marketplace,
	crate::tests::mock::new_test_ext(),
	crate::tests::mock::Test
);
