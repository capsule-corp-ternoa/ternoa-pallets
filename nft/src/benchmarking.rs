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

use crate::Pallet as NFT;

const SERIES_ID: u8 = 20;
const NFT_ID: u32 = 0;

pub fn prepare_benchmarks<T: Config>() {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	// Give them enough caps
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());

	// Create default NFT and series
	let series_id = vec![SERIES_ID];
	assert_ok!(NFT::<T>::create(
		RawOrigin::Signed(alice.clone()).into(),
		bounded_vec![1],
		Some(series_id.clone()),
		10
	));
}

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}

pub fn origin<T: Config>(name: &'static str) -> RawOrigin<T::AccountId> {
	RawOrigin::Signed(get_account::<T>(name))
}

benchmarks! {
	create {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let nft_id = NFT::<T>::nft_id_generator();

	}: _(RawOrigin::Signed(alice.clone()), bounded_vec![55], None, 0)
	verify {
		assert_eq!(NFT::<T>::data(nft_id).unwrap().owner, alice);
	}

	transfer {
		prepare_benchmarks::<T>();

		let alice = origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());

		assert_ok!(NFT::<T>::finish_series(alice.clone().into(), vec![SERIES_ID]));
	}: _(alice.clone(), NFT_ID, bob_lookup)
	verify {
		assert_eq!(NFT::<T>::data(NFT_ID).unwrap().owner, bob);
	}

	burn {
		prepare_benchmarks::<T>();

	}: _(origin::<T>("ALICE"), NFT_ID)
	verify {
		assert_eq!(NFT::<T>::data(NFT_ID), None);
	}

	finish_series {
		prepare_benchmarks::<T>();

		let series_id: Vec<u8> = vec![SERIES_ID];

	}: _(origin::<T>("ALICE"), series_id.clone())
	verify {
		assert_eq!(NFT::<T>::series(&series_id).unwrap().draft, false);
	}

	set_nft_mint_fee {
		prepare_benchmarks::<T>();

		let old_mint_fee = NFT::<T>::nft_mint_fee();
		let new_mint_fee = 1000u32;

	}: _(RawOrigin::Root, new_mint_fee.clone().into())
	verify {
		assert_ne!(old_mint_fee, new_mint_fee.clone().into());
		assert_eq!(NFT::<T>::nft_mint_fee(), new_mint_fee.into());
	}

	delegate {
		prepare_benchmarks::<T>();

		let bob: T::AccountId = get_account::<T>("BOB");

	}: _(origin::<T>("ALICE"), NFT_ID, Some(bob.clone()))
	verify {
		assert_eq!(NFT::<T>::data(NFT_ID).unwrap().is_delegated, true);
		assert_eq!(NFT::<T>::delegated_nfts(NFT_ID), Some(bob));
	}

	set_nft_royaltie_fee {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");

		let old_royaltie_fee = NFT::<T>::data(NFT_ID).unwrap().royaltie_fee;
		let new_royaltie_fee = 15;

	}: _(RawOrigin::Signed(alice.clone()), NFT_ID, new_royaltie_fee.clone())
	verify {
		assert_ne!(old_royaltie_fee, new_royaltie_fee.clone());
		assert_eq!(NFT::<T>::data(NFT_ID).unwrap().royaltie_fee, new_royaltie_fee);
	}
}

impl_benchmark_test_suite!(NFT, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
