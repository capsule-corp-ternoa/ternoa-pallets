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
use frame_support::{assert_ok, traits::Currency, BoundedVec};
use frame_system::RawOrigin;
use sp_arithmetic::per_things::Permill;
use sp_runtime::traits::{Bounded, StaticLookup};
use sp_std::prelude::*;

use crate::Pallet as NFT;

const NFT_ID: u32 = 0;
const COLLECTION_ID: u32 = 0;

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}

pub fn origin<T: Config>(name: &'static str) -> RawOrigin<T::AccountId> {
	RawOrigin::Signed(get_account::<T>(name))
}

pub fn prepare_benchmarks<T: Config>() {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	// Give them enough caps
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());

	// Create default NFT and collection
	assert_ok!(NFT::<T>::create_nft(
		RawOrigin::Signed(alice.clone()).into(),
		BoundedVec::try_from(vec![1]).unwrap(),
		Permill::from_parts(100000),
		None,
		false,
	));
	assert_ok!(NFT::<T>::create_collection(
		RawOrigin::Signed(alice).into(),
		BoundedVec::try_from(vec![1]).unwrap(),
		None,
	));
}

benchmarks! {
	create_nft {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let alice_origin = origin::<T>("ALICE");
		
		// Fill the collection
		let limit = T::CollectionSizeLimit::get() - 1;
		for _i in 0..limit {
			NFT::<T>::create_nft(
				alice_origin.clone(),
				BoundedVec::default(),
				Permill::from_parts(0),
				Some(COLLECTION_ID),
				false,
			)
			.unwrap();
		}

		// Get The NFT id
		let nft_id = NFT::<T>::get_next_nft_id() - 1;

	}: _(alice_origin, BoundedVec::try_from(vec![1]).unwrap(), Permill::from_parts(100000), Some(COLLECTION_ID), false)
	verify {
		assert_eq!(NFT::<T>::nfts(nft_id).unwrap().owner, alice);
	}

	// burn_nft {
	// 	prepare_benchmarks::<T>();
	// 	let alice = origin::<T>("ALICE");
	// }: _(alice, NFT_ID)
	// verify {
	// 	assert_eq!(NFT::<T>::nfts(NFT_ID), None);
	// }

	// transfer_nft {
	// 	prepare_benchmarks::<T>();
	// 	let alice = origin::<T>("ALICE");
	// 	let bob: T::AccountId = get_account::<T>("BOB");
	// 	let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());
	// }: _(alice, NFT_ID, bob_lookup)
	// verify {
	// 	assert_eq!(NFT::<T>::nfts(NFT_ID).unwrap().owner, bob);
	// }

	// delegate_nft {
	// 	prepare_benchmarks::<T>();
	// 	let alice = origin::<T>("ALICE");
	// 	let bob: T::AccountId = get_account::<T>("BOB");
	// 	let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());
	// }: _(alice, NFT_ID, Some(bob_lookup))
	// verify {
	// 	assert_eq!(NFT::<T>::nfts(NFT_ID).unwrap().state.is_delegated, true);
	// 	assert_eq!(NFT::<T>::delegated_nfts(NFT_ID), Some(bob));
	// }

	// set_royalty {
	// 	prepare_benchmarks::<T>();
	// 	let alice = origin::<T>("ALICE");
	// }: _(alice, NFT_ID, Permill::from_parts(1000000))
	// verify {
	// 	assert_eq!(NFT::<T>::nfts(NFT_ID).unwrap().royalty, Permill::from_parts(1000000));
	// }

	// set_nft_mint_fee {
	// 	let old_mint_fee = NFT::<T>::nft_mint_fee();
	// 	let new_mint_fee = 20u32;
	// }: _(RawOrigin::Root, new_mint_fee.clone().into())
	// verify {
	// 	assert_ne!(old_mint_fee, new_mint_fee.clone().into());
	// 	assert_eq!(NFT::<T>::nft_mint_fee(), new_mint_fee.into());
	// }

	// create_collection {
	// 	prepare_benchmarks::<T>();
	// 	let alice: T::AccountId = get_account::<T>("ALICE");
	// 	let collection_id = 1;
	// }: _(alice.clone(), BoundedVec::try_from(vec![1]).unwrap(), Some(10))
	// verify {
	// 	assert_eq!(NFT::<T>::collections(collection_id).unwrap().owner, alice);
	// }

	// burn_collection {
	// 	prepare_benchmarks::<T>();
	// 	let alice = origin::<T>("ALICE");
	// }: _(alice, COLLECTION_ID)
	// verify {
	// 	assert_eq!(NFT::<T>::collections(COLLECTION_ID), None);
	// }

	// close_collection {
	// 	prepare_benchmarks::<T>();
	// 	let alice = origin::<T>("ALICE");
	// }: _(alice, COLLECTION_ID)
	// verify {
	// 	assert_eq!(NFT::<T>::collections(COLLECTION_ID).unwrap().is_closed, true);
	// }

	// limit_collection {
	// 	prepare_benchmarks::<T>();
	// 	let alice: T::AccountId = get_account::<T>("ALICE");
	// 	let alice_origin = origin::<T>("ALICE");
	// 	let limit = T::CollectionSizeLimit::get();
	// 	for _i in 0..limit {
	// 		NFT::<T>::create_nft(
	// 			alice_origin.into(),
	// 			BoundedVec::default(),
	// 			Permill::from_parts(0),
	// 			Some(COLLECTION_ID),
	// 			false,
	// 		)
	// 		.unwrap();
	// 	}
	// }: _(alice, COLLECTION_ID, limit)
	// verify {
	// 	assert_eq!(NFT::<T>::collections(COLLECTION_ID).unwrap().limit, Some(limit));
	// }

	// add_nft_to_collection {
	// 	prepare_benchmarks::<T>();
	// 	let alice: T::AccountId = get_account::<T>("ALICE");
	// 	let alice_origin = origin::<T>("ALICE");
	// 	let limit = T::CollectionSizeLimit::get() - 1;
	// 	for _i in 0..limit {
	// 		NFT::<T>::create_nft(
	// 			alice_origin.into(),
	// 			BoundedVec::default(),
	// 			Permill::from_parts(0),
	// 			Some(COLLECTION_ID),
	// 			false,
	// 		)
	// 		.unwrap();
	// 	}
	// }: _(alice, NFT_ID, COLLECTION_ID)
	// verify {
	// 	assert_eq!(NFT::<T>::nfts(NFT_ID).unwrap().collection_id, Some(COLLECTION_ID));
	// }
}

impl_benchmark_test_suite!(NFT, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);