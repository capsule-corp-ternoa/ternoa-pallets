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
use crate::Pallet as NFT;
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite};
use frame_support::{assert_ok, traits::Currency, BoundedVec};
use frame_system::RawOrigin;
use sp_arithmetic::per_things::Permill;
use sp_runtime::traits::{Bounded, StaticLookup};
use sp_std::prelude::*;
use ternoa_common::traits::NFTExt;

const NFT_ID: u32 = 0;
const COLLECTION_ID: u32 = 0;
const PERCENT_100: Permill = Permill::from_parts(1000000);

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
	let alice_origin = origin::<T>("ALICE");

	// Give them enough caps.
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());

	let nft_offchain_data = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize])
		.expect("It will never happen.");
	let collection_offchain_data =
		BoundedVec::try_from(vec![1; T::CollectionOffchainDataLimit::get() as usize])
			.expect("It will never happen.");

	// Create default NFT and collection.
	assert_ok!(NFT::<T>::create_nft(
		alice_origin.clone().into(),
		nft_offchain_data,
		PERCENT_100,
		None,
		false,
	));
	assert_ok!(NFT::<T>::create_collection(alice_origin.into(), collection_offchain_data, None,));
}

benchmarks! {
	create_nft {
		let s in 0 .. T::CollectionSizeLimit::get() - 1;
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let alice_origin = origin::<T>("ALICE");
		let nft_offchain_data: BoundedVec<u8, T::NFTOffchainDataLimit> = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).expect("It will never happen.");
		// Fill the collection.
		let collection_id = NFT::<T>::get_next_collection_id();
		NFT::<T>::create_filled_collection(alice.clone(), collection_id, 0, s).unwrap();
	}: _(alice_origin, nft_offchain_data, PERCENT_100, Some(collection_id), false)
	verify {
		// Get The NFT id.
		let nft_id = NFT::<T>::get_next_nft_id() - 1;
		// Get The NFT.
		let nft = NFT::<T>::nfts(nft_id).unwrap();
		assert_eq!(nft.owner, alice);
		assert_eq!(NFT::<T>::collections(collection_id).unwrap().nfts.contains(&nft_id), true);
		assert_eq!(nft.collection_id, Some(collection_id));
	}

	burn_nft {
		let s in 0 .. T::CollectionSizeLimit::get() - 1;
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
		let nft_offchain_data: BoundedVec<u8, T::NFTOffchainDataLimit> = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).expect("It will never happen.");
		// Fill the collection.
		let collection_id = NFT::<T>::get_next_collection_id();
		NFT::<T>::create_filled_collection(get_account::<T>("ALICE"), collection_id, NFT_ID + 1, s).unwrap();
		// Add NFT to collection.
		NFT::<T>::add_nft_to_collection(alice.clone().into(), NFT_ID, collection_id).unwrap();
	}: _(alice, NFT_ID)
	verify {
		assert_eq!(NFT::<T>::nfts(NFT_ID), None);
		assert_eq!(NFT::<T>::collections(collection_id).unwrap().nfts.contains(&NFT_ID), false);
	}

	transfer_nft {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());
	}: _(alice, NFT_ID, bob_lookup)
	verify {
		assert_eq!(NFT::<T>::nfts(NFT_ID).unwrap().owner, bob);
	}

	delegate_nft {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());
	}: _(alice, NFT_ID, Some(bob_lookup))
	verify {
		assert_eq!(NFT::<T>::nfts(NFT_ID).unwrap().state.is_delegated, true);
		assert_eq!(NFT::<T>::delegated_nfts(NFT_ID), Some(bob));
	}

	set_royalty {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
	}: _(alice, NFT_ID, PERCENT_100)
	verify {
		assert_eq!(NFT::<T>::nfts(NFT_ID).unwrap().royalty, PERCENT_100);
	}

	set_nft_mint_fee {
		let old_mint_fee = NFT::<T>::nft_mint_fee();
		let new_mint_fee = 20u32;
	}: _(RawOrigin::Root, new_mint_fee.clone().into())
	verify {
		assert_ne!(old_mint_fee, new_mint_fee.clone().into());
		assert_eq!(NFT::<T>::nft_mint_fee(), new_mint_fee.into());
	}

	create_collection {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let alice_origin = origin::<T>("ALICE");
		let collection_id = 1;
		let collection_offchain_data = BoundedVec::try_from(vec![1; T::CollectionOffchainDataLimit::get() as usize]).expect("It will never happen.");
	}: _(alice_origin, collection_offchain_data, Some(10))
	verify {
		assert_eq!(NFT::<T>::collections(collection_id).unwrap().owner, alice);
	}

	burn_collection {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
	}: _(alice, COLLECTION_ID)
	verify {
		assert_eq!(NFT::<T>::collections(COLLECTION_ID), None);
	}

	close_collection {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
	}: _(alice, COLLECTION_ID)
	verify {
		assert_eq!(NFT::<T>::collections(COLLECTION_ID).unwrap().is_closed, true);
	}

	limit_collection {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
	}: _(alice, COLLECTION_ID, 1)
	verify {
		assert_eq!(NFT::<T>::collections(COLLECTION_ID).unwrap().limit, Some(1));
	}

	add_nft_to_collection {
		let s in 0 .. T::CollectionSizeLimit::get() - 1;
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
		let nft_offchain_data: BoundedVec<u8, T::NFTOffchainDataLimit> = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).expect("It will never happen.");
		// Fill the collection.
		let collection_id = NFT::<T>::get_next_collection_id();
		NFT::<T>::create_filled_collection(get_account::<T>("ALICE"), collection_id, NFT_ID + 1, s).unwrap();
	}: _(alice, NFT_ID, collection_id)
	verify {
		assert_eq!(NFT::<T>::nfts(NFT_ID).unwrap().collection_id, Some(collection_id));
		assert_eq!(NFT::<T>::collections(collection_id).unwrap().nfts.contains(&NFT_ID), true);
	}
}

impl_benchmark_test_suite!(NFT, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
