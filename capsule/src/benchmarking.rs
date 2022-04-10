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
use crate::Pallet as Capsule;
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite};
use frame_support::{assert_ok, bounded_vec};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;
use sp_std::prelude::*;
use ternoa_common::traits::NFTExt;

const SERIES_ID: u8 = 20;

pub fn prepare_benchmarks<T: Config>() -> (NFTId, NFTId) {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	// Give them enough caps
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());

	// Create default Capsule
	assert_ok!(Capsule::<T>::create(
		RawOrigin::Signed(alice.clone()).into(),
		bounded_vec![1],
		bounded_vec![2],
		None,
	));

	// Create default NFT and series
	let series_id = vec![SERIES_ID];
	let nft_id =
		T::NFTExt::create_nft(alice.clone(), bounded_vec![1], Some(series_id.clone())).unwrap();

	// Lock series
	T::NFTExt::benchmark_lock_series(series_id.clone());

	(nft_id - 1, nft_id)
}

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}

pub fn get_origin<T: Config>(name: &'static str) -> RawOrigin<T::AccountId> {
	RawOrigin::Signed(get_account::<T>(name))
}

benchmarks! {
	create {
		let (_, nft_id) = prepare_benchmarks::<T>();

		let alice: T::AccountId = get_account::<T>("ALICE");
		let nft_reference = bounded_vec![50];
		let capsule_reference: CapsuleIPFSReference<T> = bounded_vec![51];
		let nft_id = nft_id + 1;
		let capsule = CapsuleData::new(alice.clone(), capsule_reference.clone());

	}: _(RawOrigin::Signed(alice.clone()), nft_reference, capsule_reference, None)
	verify {
		assert_eq!(Capsule::<T>::capsules(nft_id), Some(capsule));
	}

	create_from_nft {
		let (_, nft_id) = prepare_benchmarks::<T>();

		let alice: T::AccountId = get_account::<T>("ALICE");
		let capsule_reference: CapsuleIPFSReference<T>  = bounded_vec![51];
		let capsule = CapsuleData::new(alice.clone(), capsule_reference.clone());

	}: _(RawOrigin::Signed(alice.clone()), nft_id, capsule_reference.clone())
	verify {
		assert_eq!(Capsule::<T>::capsules(nft_id), Some(capsule));
	}

	remove {
		let (nft_id, ..) = prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");

	}: _(RawOrigin::Signed(alice.clone()), nft_id)
	verify {
		assert!(Capsule::<T>::capsules(nft_id).is_none());
		assert!(Capsule::<T>::ledgers(&alice).is_none());
	}

	add_funds {
		let (nft_id, ..) = prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");

		let fee = Capsule::<T>::capsule_mint_fee();
		let amount = 200u32;
	}: _(RawOrigin::Signed(alice.clone()), nft_id, amount.into())
	verify {
		assert_eq!(Capsule::<T>::ledgers(&alice).unwrap()[0].1, fee + amount.into());
	}

	set_ipfs_reference {
		let (nft_id, ..) = prepare_benchmarks::<T>();
		let new_reference: CapsuleIPFSReference<T>  = bounded_vec![101];

	}: _(get_origin::<T>("ALICE"), nft_id, new_reference.clone())
	verify {
		let reference = Capsule::<T>::capsules(nft_id).unwrap().ipfs_reference.clone();
		assert_eq!(reference, new_reference);
	}

	set_capsule_mint_fee {
		let old_mint_fee = Capsule::<T>::capsule_mint_fee();
		let new_mint_fee = 1234u32;
		assert_ne!(old_mint_fee, new_mint_fee.clone().into());

	}: _(RawOrigin::Root, new_mint_fee.clone().into())
	verify {
		assert_eq!(Capsule::<T>::capsule_mint_fee(), new_mint_fee.into());
	}
}

impl_benchmark_test_suite!(Capsule, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
