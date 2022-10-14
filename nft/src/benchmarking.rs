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
use ternoa_common::traits::NFTExt;

use crate::Pallet as NFT;

pub struct BenchmarkData {
	nft_id: NFTId,
	collection_id: CollectionId,
}

const PERCENT_100: Permill = Permill::from_parts(1000000);

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

	// Give them enough caps.
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());

	let nft_offchain_data =
		BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).unwrap();
	let collection_offchain_data =
		BoundedVec::try_from(vec![1; T::CollectionOffchainDataLimit::get() as usize]).unwrap();

	// Create default NFT and collection.
	assert_ok!(NFT::<T>::create_nft(
		origin::<T>("ALICE").into(),
		nft_offchain_data,
		PERCENT_100,
		None,
		false,
	));
	assert_ok!(NFT::<T>::create_collection(
		origin::<T>("ALICE").into(),
		collection_offchain_data,
		None,
	));
	BenchmarkData {
		nft_id: NFT::<T>::next_nft_id() - 1,
		collection_id: NFT::<T>::next_collection_id() - 1,
	}
}

benchmarks! {
	create_nft {
		let s in 0 .. T::CollectionSizeLimit::get() - 1;
		let benchmark_data = prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let nft_offchain_data: BoundedVec<u8, T::NFTOffchainDataLimit> = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).unwrap();
		// Fill the collection.
		NFT::<T>::create_filled_collection(alice.clone(), benchmark_data.collection_id, 0, s).unwrap();
	}: _(origin::<T>("ALICE"), nft_offchain_data, PERCENT_100, Some(benchmark_data.collection_id), false)
	verify {
		// Get The NFT id.
		let nft_id = NFT::<T>::next_nft_id() - 1;
		// Get The NFT.
		let nft = NFT::<T>::nfts(nft_id).unwrap();
		assert_eq!(nft.owner, alice);
		assert_eq!(NFT::<T>::collections(benchmark_data.collection_id).unwrap().nfts.contains(&nft_id), true);
		assert_eq!(nft.collection_id, Some(benchmark_data.collection_id));
	}

	burn_nft {
		let s in 0 .. T::CollectionSizeLimit::get() - 1;
		let benchmark_data = prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
		let nft_offchain_data: BoundedVec<u8, T::NFTOffchainDataLimit> = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).unwrap();
		// Fill the collection.
		NFT::<T>::create_filled_collection(get_account::<T>("ALICE"), benchmark_data.collection_id, benchmark_data.nft_id + 1, s).unwrap();
		// Add NFT to collection.
		NFT::<T>::add_nft_to_collection(alice.clone().into(), benchmark_data.nft_id, benchmark_data.collection_id).unwrap();
	}: _(alice, benchmark_data.nft_id)
	verify {
		assert_eq!(NFT::<T>::nfts(benchmark_data.nft_id), None);
		assert_eq!(NFT::<T>::collections(benchmark_data.collection_id).unwrap().nfts.contains(&benchmark_data.nft_id), false);
	}

	transfer_nft {
		let benchmark_data = prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());
	}: _(alice, benchmark_data.nft_id, bob_lookup)
	verify {
		assert_eq!(NFT::<T>::nfts(benchmark_data.nft_id).unwrap().owner, bob);
	}

	delegate_nft {
		let benchmark_data = prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());
	}: _(alice, benchmark_data.nft_id, Some(bob_lookup))
	verify {
		assert_eq!(NFT::<T>::nfts(benchmark_data.nft_id).unwrap().state.is_delegated, true);
		assert_eq!(NFT::<T>::delegated_nfts(benchmark_data.nft_id), Some(bob));
	}

	set_royalty {
		let benchmark_data = prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
	}: _(alice, benchmark_data.nft_id, PERCENT_100)
	verify {
		assert_eq!(NFT::<T>::nfts(benchmark_data.nft_id).unwrap().royalty, PERCENT_100);
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
		let collection_offchain_data = BoundedVec::try_from(vec![1; T::CollectionOffchainDataLimit::get() as usize]).unwrap();
	}: _(origin::<T>("ALICE"), collection_offchain_data, Some(10))
	verify {
		let collection_id = NFT::<T>::next_nft_id() - 1;
		assert_eq!(NFT::<T>::collections(collection_id).unwrap().owner, alice);
	}

	burn_collection {
		let benchmark_data = prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
	}: _(alice, benchmark_data.collection_id)
	verify {
		assert_eq!(NFT::<T>::collections(benchmark_data.collection_id), None);
	}

	close_collection {
		let benchmark_data = prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
	}: _(alice, benchmark_data.collection_id)
	verify {
		assert_eq!(NFT::<T>::collections(benchmark_data.collection_id).unwrap().is_closed, true);
	}

	limit_collection {
		let benchmark_data = prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
	}: _(alice, benchmark_data.collection_id, 1)
	verify {
		assert_eq!(NFT::<T>::collections(benchmark_data.collection_id).unwrap().limit, Some(1));
	}

	add_nft_to_collection {
		let s in 0 .. T::CollectionSizeLimit::get() - 1;
		let benchmark_data = prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
		let nft_offchain_data: BoundedVec<u8, T::NFTOffchainDataLimit> = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).unwrap();
		// Fill the collection.
		NFT::<T>::create_filled_collection(get_account::<T>("ALICE"), benchmark_data.collection_id, benchmark_data.nft_id + 1, s).unwrap();
	}: _(alice, benchmark_data.nft_id, benchmark_data.collection_id)
	verify {
		assert_eq!(NFT::<T>::nfts(benchmark_data.nft_id).unwrap().collection_id, Some(benchmark_data.collection_id));
		assert_eq!(NFT::<T>::collections(benchmark_data.collection_id).unwrap().nfts.contains(&benchmark_data.nft_id), true);
	}

	// create_secret_nft {
	// 	let s in 0 .. T::CollectionSizeLimit::get() - 1;
	// 	let benchmark_data = prepare_benchmarks::<T>();
	// 	let alice: T::AccountId = get_account::<T>("ALICE");
	// 	let nft_offchain_data: BoundedVec<u8, T::NFTOffchainDataLimit> = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).unwrap();
	// 	let secret_offchain_data: BoundedVec<u8, T::NFTOffchainDataLimit> = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).unwrap();
	// 	// Fill the collection.
	// 	NFT::<T>::create_filled_collection(alice.clone(), benchmark_data.collection_id, 0, s).unwrap();
	// }: _(origin::<T>("ALICE"), nft_offchain_data, secret_offchain_data, PERCENT_100, Some(benchmark_data.collection_id), false)
	// verify {
	// 	// Get The NFT id.
	// 	let nft_id = NFT::<T>::next_nft_id() - 1;
	// 	// Get The NFT.
	// 	let nft = NFT::<T>::nfts(nft_id).unwrap();
	// 	// Get the secret offchain_data
	// 	let secret_offchain_data = NFT::<T>::secret_nfts_offchain_data(nft_id);
	// 	assert_eq!(nft.owner, alice);
	// 	assert_eq!(NFT::<T>::collections(benchmark_data.collection_id).unwrap().nfts.contains(&nft_id), true);
	// 	assert_eq!(nft.collection_id, Some(benchmark_data.collection_id));
	// 	assert_eq!(nft.state.is_secret, true);
	// 	assert_eq!(nft.state.is_syncing, true);
	// 	assert!(secret_offchain_data.is_some());
	// }

	// add_secret {
	// 	let benchmark_data = prepare_benchmarks::<T>();
	// 	let alice: T::AccountId = get_account::<T>("ALICE");
	// 	let secret_offchain_data: BoundedVec<u8, T::NFTOffchainDataLimit> = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).unwrap();
	// }: _(origin::<T>("ALICE"), benchmark_data.nft_id, secret_offchain_data)
	// verify {
	// 	// Get The NFT.
	// 	let nft = NFT::<T>::nfts(benchmark_data.nft_id).unwrap();
	// 	let secret_offchain_data = NFT::<T>::secret_nfts_offchain_data(benchmark_data.nft_id);
	// 	assert_eq!(nft.state.is_secret, true);
	// 	assert_eq!(nft.state.is_syncing, true);
	// 	assert!(secret_offchain_data.is_some());
	// }

	// //TODO change when sgx
	// add_secret_shard {
	// 	let benchmark_data = prepare_benchmarks::<T>();
	// 	let alice = origin::<T>("ALICE");
	// 	let secret_offchain_data: BoundedVec<u8, T::NFTOffchainDataLimit> = BoundedVec::try_from(vec![1; T::NFTOffchainDataLimit::get() as usize]).unwrap();
	// 	NFT::<T>::add_secret(alice.clone().into(), benchmark_data.nft_id, secret_offchain_data).unwrap();
	// }: _(origin::<T>("ALICE"), benchmark_data.nft_id)
	// verify {
	// 	// Get The NFT.
	// 	let nft = NFT::<T>::nfts(benchmark_data.nft_id).unwrap();
	// 	let shards = NFT::<T>::secret_nfts_shards_count(benchmark_data.nft_id).unwrap();
	// 	let alice: T::AccountId = get_account::<T>("ALICE");
	// 	assert_eq!(nft.state.is_secret, true);
	// 	assert_eq!(nft.state.is_syncing, true);
	// 	assert!(shards.contains(&alice));

	// }

	// set_secret_nft_mint_fee {
	// 	let old_mint_fee = NFT::<T>::secret_nft_mint_fee();
	// 	let new_mint_fee = 150u32;
	// }: _(RawOrigin::Root, new_mint_fee.clone().into())
	// verify {
	// 	assert_ne!(old_mint_fee, new_mint_fee.clone().into());
	// 	assert_eq!(NFT::<T>::secret_nft_mint_fee(), new_mint_fee.into());
	// }
}

impl_benchmark_test_suite!(NFT, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
