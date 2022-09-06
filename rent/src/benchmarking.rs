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
use sp_runtime::traits::Bounded;
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

pub fn root<T: Config>() -> RawOrigin<T::AccountId> {
	RawOrigin::Root.into()
}

pub fn prepare_benchmarks<T: Config>() -> () {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	// Give them enough caps.
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value() / 5u32.into()); // to avoid overflow for renter
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value() / 5u32.into());

	// Create default NFTs.
	assert_ok!(T::NFTExt::create_nft(
		get_account::<T>("ALICE").into(),
		BoundedVec::default(),
		PERCENT_100,
		None,
		false,
	));
	assert_ok!(T::NFTExt::create_nft(
		get_account::<T>("ALICE").into(),
		BoundedVec::default(),
		PERCENT_100,
		None,
		false,
	));

	let cancellation_fee = BalanceOf::<T>::max_value() / 1000u32.into();
	let rent_fee = BalanceOf::<T>::max_value() / 10000u32.into();

	// Create default Contract.
	let ok = Rent::<T>::create_contract(
		origin::<T>("ALICE").into(),
		NFT_ID_0,
		Duration::Subscription(1000u32.into(), Some(10000u32.into())),
		AcceptanceType::AutoAcceptance(None),
		RevocationType::Anytime,
		RentFee::Tokens(rent_fee),
		Some(CancellationFee::FixedTokens(cancellation_fee)),
		Some(CancellationFee::FixedTokens(cancellation_fee)),
	);
	assert_eq!(T::Currency::free_balance(&alice.into()) > 100u32.into(), true);
	assert_ok!(ok);
}

benchmarks! {
	create_contract {
		let s in 0 .. T::SimultaneousContractLimit::get() - 2;
		prepare_benchmarks::<T>();
		Rent::<T>::fill_available_queue(s, 99u32.into(), 10u32.into()).unwrap();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
	}: _(origin::<T>("ALICE"), NFT_ID_1, Duration::Subscription(1000u32.into(), Some(10000u32.into())), AcceptanceType::AutoAcceptance(None), RevocationType::Anytime, RentFee::Tokens(1000u32.into()), Some(CancellationFee::FixedTokens(100u32.into())),Some(CancellationFee::FixedTokens(100u32.into())))
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_1).unwrap();
		assert_eq!(contract.renter, alice);
	}

	revoke_contract {
		let s in 0 .. T::SimultaneousContractLimit::get() - 1;
		prepare_benchmarks::<T>();
		Rent::<T>::fill_subscription_queue(s, 99u32.into(), 10u32.into()).unwrap();
		Rent::<T>::rent(origin::<T>("BOB").into(), NFT_ID_0).unwrap();
	}: _(origin::<T>("ALICE"), NFT_ID_0)
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_0);
		assert!(contract.is_none());
	}

	rent {
		let s in 0 .. T::SimultaneousContractLimit::get() - 2;
		let t in 0 .. T::SimultaneousContractLimit::get() - 1;
		prepare_benchmarks::<T>();
		Rent::<T>::fill_available_queue(s, 99u32.into(), 10u32.into()).unwrap();
		Rent::<T>::fill_subscription_queue(t, 99u32.into(), 10u32.into()).unwrap();
	}: _(origin::<T>("BOB"), NFT_ID_0)
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_0).unwrap();
		assert_eq!(contract.rentee, Some(get_account::<T>("BOB")))
	}

	accept_rent_offer {
		let s in 0 .. T::SimultaneousContractLimit::get() - 2;
		let t in 0 .. T::SimultaneousContractLimit::get() - 1;
		let u in 0 .. T::AccountSizeLimit::get() - 1;
		prepare_benchmarks::<T>();
		Rent::<T>::fill_available_queue(s/2, 99u32.into(), 10u32.into()).unwrap();
		Rent::<T>::fill_subscription_queue(t/2, 99u32.into(), 10u32.into()).unwrap();
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		Rent::<T>::create_contract(
			origin::<T>("ALICE").into(),
			NFT_ID_1,
			Duration::Subscription(1000u32.into(), Some(10000u32.into())),
			AcceptanceType::ManualAcceptance(None),
			RevocationType::Anytime,
			RentFee::Tokens(1000u32.into()),
			Some(CancellationFee::FixedTokens(100u32.into())),
			Some(CancellationFee::FixedTokens(100u32.into()))
		).unwrap();
		Rent::<T>::fill_offers_vector(u, NFT_ID_1, charlie)?;
		Rent::<T>::rent(origin::<T>("BOB").into(), NFT_ID_1).unwrap();
	}: _(origin::<T>("ALICE"), NFT_ID_1, bob)
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_1).unwrap();
		assert_eq!(contract.rentee, Some(get_account::<T>("BOB")));
		assert!(contract.has_started);
	}

	retract_rent_offer {
		let s in 0 .. T::AccountSizeLimit::get() - 1;
		prepare_benchmarks::<T>();
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		Rent::<T>::create_contract(
			origin::<T>("ALICE").into(),
			NFT_ID_1,
			Duration::Subscription(1000u32.into(), Some(10000u32.into())),
			AcceptanceType::ManualAcceptance(None),
			RevocationType::Anytime,
			RentFee::Tokens(1000u32.into()),
			Some(CancellationFee::FixedTokens(100u32.into())),
			Some(CancellationFee::FixedTokens(100u32.into()))
		).unwrap();
		Rent::<T>::fill_offers_vector(s, NFT_ID_1, charlie)?;
		Rent::<T>::rent(origin::<T>("BOB").into(), NFT_ID_1).unwrap();
	}: _(origin::<T>("BOB"), NFT_ID_1)
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_1).unwrap();
		assert!(contract.rentee.is_none());
		assert!(!contract.has_started);
		// Check that offer has been removed
		let offers = Rent::<T>::offers(NFT_ID_1).unwrap();
		assert!(!offers.contains(&bob))
	}

	change_subscription_terms {
		prepare_benchmarks::<T>();
		let bob: T::AccountId = get_account::<T>("BOB");
		Rent::<T>::create_contract(
			origin::<T>("ALICE").into(),
			NFT_ID_1,
			Duration::Subscription(1000u32.into(), Some(10000u32.into())),
			AcceptanceType::AutoAcceptance(None),
			RevocationType::OnSubscriptionChange,
			RentFee::Tokens(1000u32.into()),
			Some(CancellationFee::FixedTokens(100u32.into())),
			Some(CancellationFee::FixedTokens(100u32.into()))
		).unwrap();
		Rent::<T>::rent(origin::<T>("BOB").into(), NFT_ID_1).unwrap();
	}: _(origin::<T>("ALICE"), NFT_ID_1, Duration::Subscription(500u32.into(), Some(5000u32.into())), 150u32.into())
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_1).unwrap();
		assert!(!contract.terms_accepted);
		assert_eq!(contract.duration, Duration::Subscription(500u32.into(), Some(5000u32.into())));
		assert_eq!(contract.rent_fee, RentFee::Tokens(150u32.into()));
	}

	accept_subscription_terms {
		prepare_benchmarks::<T>();
		let bob: T::AccountId = get_account::<T>("BOB");
		Rent::<T>::create_contract(
			origin::<T>("ALICE").into(),
			NFT_ID_1,
			Duration::Subscription(1000u32.into(), Some(10000u32.into())),
			AcceptanceType::AutoAcceptance(None),
			RevocationType::OnSubscriptionChange,
			RentFee::Tokens(1000u32.into()),
			Some(CancellationFee::FixedTokens(100u32.into())),
			Some(CancellationFee::FixedTokens(100u32.into()))
		).unwrap();
		Rent::<T>::rent(origin::<T>("BOB").into(), NFT_ID_1).unwrap();
		Rent::<T>::change_subscription_terms(
			origin::<T>("ALICE").into(),
			NFT_ID_1,
			Duration::Subscription(500u32.into(), Some(5000u32.into())),
			150u32.into()
		).unwrap();
	}: _(origin::<T>("BOB"), NFT_ID_1)
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_1).unwrap();
		assert!(contract.terms_accepted);
		assert_eq!(contract.duration, Duration::Subscription(500u32.into(), Some(5000u32.into())));
		assert_eq!(contract.rent_fee, RentFee::Tokens(150u32.into()));
	}

	end_contract {
		let s in 0 .. T::SimultaneousContractLimit::get() - 1;
		prepare_benchmarks::<T>();
		Rent::<T>::fill_subscription_queue(s, 99u32.into(), 10u32.into()).unwrap();
		Rent::<T>::rent(origin::<T>("BOB").into(), NFT_ID_0).unwrap();
	}: _(root::<T>(), NFT_ID_0, None)
	verify {
		assert!(Rent::<T>::contracts(NFT_ID_0).is_none());
	}

	renew_contract {
		let s in 0 .. T::SimultaneousContractLimit::get() - 1;
		prepare_benchmarks::<T>();
		Rent::<T>::fill_subscription_queue(s, 99u32.into(), 10u32.into()).unwrap();
		Rent::<T>::rent(origin::<T>("BOB").into(), NFT_ID_0).unwrap();
	}: _(root::<T>(), NFT_ID_0)
	verify {
		assert!(Rent::<T>::contracts(NFT_ID_0).is_some());
		let next_renew_block = Rent::<T>::renting_queues().subscription_queue.get(NFT_ID_0).unwrap();
		assert_eq!(next_renew_block, 1000u32.into());
	}

	remove_expired_contract {
		let s in 0 .. T::SimultaneousContractLimit::get() - 1;
		prepare_benchmarks::<T>();
		Rent::<T>::fill_available_queue(s, 99u32.into(), 10u32.into()).unwrap();
	}: _(root::<T>(), NFT_ID_0)
	verify {
		assert!(Rent::<T>::contracts(NFT_ID_0).is_none());
	}
}

impl_benchmark_test_suite!(Rent, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
