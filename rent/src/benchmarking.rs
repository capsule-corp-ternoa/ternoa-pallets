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
use crate::Pallet as Rent;
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite};
use frame_support::{assert_ok, traits::Currency, BoundedVec};
use frame_system::RawOrigin;
use sp_arithmetic::per_things::Permill;
use sp_std::prelude::*;
use ternoa_common::traits::NFTExt;

const PERCENT_100: Permill = Permill::from_parts(1000000);
const NFT_ID_0: NFTId = 0u32;
const NFT_ID_1: NFTId = 1u32;

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}

pub fn origin<T: Config>(name: &'static str) -> RawOrigin<T::AccountId> {
	RawOrigin::Signed(get_account::<T>(name))
}

pub fn prepare_benchmarks<T: Config>() -> () {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	// Give them enough caps.
	T::Currency::make_free_balance_be(&alice, 100_000u32.into());
	T::Currency::make_free_balance_be(&bob, 100_000u32.into());
	T::Currency::make_free_balance_be(&Rent::<T>::account_id(), 100_000u32.into());

	let account_list: BoundedVec<T::AccountId, T::AccountSizeLimit> =
		BoundedVec::try_from(vec![bob; T::AccountSizeLimit::get() as usize]).unwrap();

	// Create default NFT.
	assert_ok!(T::NFTExt::create_nft(
		get_account::<T>("ALICE").into(),
		BoundedVec::default(),
		PERCENT_100,
		None,
		false,
	));

	// Create default Contract.
	assert_ok!(Rent::<T>::create_contract(
		origin::<T>("ALICE").into(),
		NFT_ID_0,
		Duration::Subscription(1000u32.into(), Some(10000u32.into())),
		AcceptanceType::AutoAcceptance(Some(account_list)),
		RevocationType::Anytime,
		RentFee::Tokens(1000u32.into()),
		Some(CancellationFee::FixedTokens(100u32.into())),
		Some(CancellationFee::FixedTokens(100u32.into())),
	));
}

benchmarks! {
	create_contract {
		let s in 0 .. T::AccountSizeLimit::get() - 1;
		let benchmark_data = prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let account_list: BoundedVec<T::AccountId, T::AccountSizeLimit> = BoundedVec::try_from(vec![bob; s as usize]).unwrap();
		T::NFTExt::create_nft(
			alice.clone(),
			BoundedVec::default(),
			PERCENT_100,
			None,
			false,
		).unwrap();
	}: _(origin::<T>("ALICE"), NFT_ID_1, Duration::Subscription(1000u32.into(), Some(10000u32.into())), AcceptanceType::AutoAcceptance(Some(account_list)), RevocationType::Anytime, RentFee::Tokens(1000u32.into()), Some(CancellationFee::FixedTokens(100u32.into())),Some(CancellationFee::FixedTokens(100u32.into())))
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_1).unwrap();
		assert_eq!(contract.renter, alice);
	}

	revoke_contract {
		let benchmark_data = prepare_benchmarks::<T>();
		Rent::<T>::rent(origin::<T>("BOB").into(), NFT_ID_0).unwrap();
	}: _(origin::<T>("ALICE"), NFT_ID_0)
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_0);
		assert!(contract.is_none());
	}

	rent {
		let benchmark_data = prepare_benchmarks::<T>();
	}: _(origin::<T>("BOB"), NFT_ID_0)
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_0).unwrap();
		assert_eq!(contract.rentee, Some(get_account::<T>("BOB")))
	}

}

impl_benchmark_test_suite!(Rent, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
