// Copyright 2023 Capsule Corp (France) SAS.
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

pub fn prepare_benchmarks<T: Config>() -> () {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	// Give them enough caps.
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value() / 2u32.into()); // to avoid overflow for renter
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value() / 2u32.into());

	// Create default NFTs.
	for _i in 0..2 {
		let data = BoundedVec::default();
		let ok = T::NFTExt::create_nft(alice.clone().into(), data, PERCENT_100, None, false);
		assert_ok!(ok);
	}

	let cancellation_fee = BalanceOf::<T>::max_value() / 100000u32.into();
	let rent_fee = BalanceOf::<T>::max_value() / 10000u32.into();

	// Create default Contract.
	let ok = Rent::<T>::create_contract(
		origin::<T>("ALICE").into(),
		NFT_ID_0,
		DurationInput::Subscription(SubscriptionInput {
			period_length: 2u32.into(),
			max_duration: Some(10u32.into()),
			is_changeable: true,
		}),
		AcceptanceType::AutoAcceptance(None),
		true,
		RentFee::Tokens(rent_fee),
		CancellationFee::FixedTokens(cancellation_fee),
		CancellationFee::FixedTokens(cancellation_fee),
	);
	assert_ok!(ok);
}

benchmarks! {
	create_contract {
		let s in 0 .. T::SimultaneousContractLimit::get();
		prepare_benchmarks::<T>();

		let mut new_contracts_amount = s;
		if new_contracts_amount > 2 {
			new_contracts_amount -= 2;
		};

		let alice: T::AccountId = get_account::<T>("ALICE");
		let org = origin::<T>("ALICE");
		Rent::<T>::benchmark_fill_available_queue(new_contracts_amount, 100u32.into()).unwrap();

		let sub = SubscriptionInput {
			period_length: 2u32.into(),
			max_duration: Some(10u32.into()),
			is_changeable: false,
		} ;

	}: _(org, NFT_ID_1, DurationInput::Subscription(sub), AcceptanceType::AutoAcceptance(None), true, RentFee::Tokens(1000u32.into()), CancellationFee::FixedTokens(100u32.into()),CancellationFee::FixedTokens(100u32.into()))
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_1).unwrap();
		assert_eq!(contract.renter, alice);
	}

	 revoke_contract {
		let s in 0 .. T::SimultaneousContractLimit::get();
		prepare_benchmarks::<T>();

		let mut new_contracts_amount = s;
		if new_contracts_amount > 2 {
			new_contracts_amount -= 2;
		};

		let alice: T::AccountId = get_account::<T>("ALICE");
		let org = origin::<T>("ALICE");
		Rent::<T>::benchmark_fill_available_queue(new_contracts_amount, 100u32.into()).unwrap();

		Rent::<T>::rent(origin::<T>("BOB").into(), NFT_ID_0).unwrap();
	}: _(org, NFT_ID_0)
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_0);
		assert!(contract.is_none());
	}

	cancel_contract {
		let s in 0 .. T::SimultaneousContractLimit::get();
		prepare_benchmarks::<T>();

		let mut new_contracts_amount = s;
		if new_contracts_amount > 2 {
			new_contracts_amount -= 2;
		};

		let alice: T::AccountId = get_account::<T>("ALICE");
		let org = origin::<T>("ALICE");

		Rent::<T>::benchmark_fill_available_queue(new_contracts_amount, 100u32.into()).unwrap();
	}: _(org, NFT_ID_0)
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_0);
		assert!(contract.is_none());
	}

	rent {
		let s in 0 .. T::SimultaneousContractLimit::get();
		prepare_benchmarks::<T>();

		let mut new_contracts_amount = s;
		if new_contracts_amount > 2 {
			new_contracts_amount -= 2;
		};

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let org = origin::<T>("ALICE");

		Rent::<T>::benchmark_fill_available_queue(new_contracts_amount, 100u32.into()).unwrap();
	}: _(origin::<T>("BOB"), NFT_ID_0)
	verify {
		// Get The contract.
		let contract = Rent::<T>::contracts(NFT_ID_0).unwrap();
		assert_eq!(contract.rentee, Some(bob))
	}


	make_rent_offer {
		let s in 0 .. T::SimultaneousContractLimit::get();
		prepare_benchmarks::<T>();

		let mut new_contracts_amount = s;
		if new_contracts_amount > 3 {
			new_contracts_amount -= 3;
		};

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let org = origin::<T>("ALICE");

		let ok = Rent::<T>::create_contract(
			origin::<T>("ALICE").into(),
			NFT_ID_1,
			DurationInput::Fixed(10u32.into()),
			AcceptanceType::ManualAcceptance(None),
			true,
			RentFee::Tokens(100u32.into()),
			CancellationFee::None,
			CancellationFee::None,
		);
		assert_ok!(ok);

		Rent::<T>::benchmark_fill_available_queue(new_contracts_amount, 100u32.into()).unwrap();
	}: _(origin::<T>("BOB"), NFT_ID_1)
	verify {
		// Get The offer.
		let offers = Rent::<T>::offers(NFT_ID_1).unwrap();
		assert!(offers.contains(&bob))
	}

	accept_rent_offer {
		let s in 0 .. T::SimultaneousContractLimit::get();
		prepare_benchmarks::<T>();

		let mut new_contracts_amount = s;
		if new_contracts_amount > 3 {
			new_contracts_amount -= 3;
		};

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let org = origin::<T>("ALICE");

		let ok = Rent::<T>::create_contract(
			origin::<T>("ALICE").into(),
			NFT_ID_1,
			DurationInput::Fixed(10u32.into()),
			AcceptanceType::ManualAcceptance(None),
			true,
			RentFee::Tokens(100u32.into()),
			CancellationFee::None,
			CancellationFee::None,
		);
		assert_ok!(ok);

		Rent::<T>::benchmark_fill_available_queue(new_contracts_amount, 100u32.into()).unwrap();
		Rent::<T>::make_rent_offer(origin::<T>("BOB").into(), NFT_ID_1).unwrap();
	}: _(origin::<T>("ALICE"), NFT_ID_1, bob.clone())
	verify {
		let contract = Rent::<T>::contracts(NFT_ID_1).unwrap();
		assert_eq!(contract.rentee, Some(bob));
	}

	retract_rent_offer {
		let s in 0 .. T::AccountSizeLimit::get();
		prepare_benchmarks::<T>();

		let mut new_offer_amount = s;
		if new_offer_amount > 1 {
			new_offer_amount -= 1;
		};

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let org = origin::<T>("ALICE");

		let ok = Rent::<T>::create_contract(
			origin::<T>("ALICE").into(),
			NFT_ID_1,
			DurationInput::Fixed(10u32.into()),
			AcceptanceType::ManualAcceptance(None),
			true,
			RentFee::Tokens(100u32.into()),
			CancellationFee::None,
			CancellationFee::None,
		);
		assert_ok!(ok);

		Rent::<T>::benchmark_fill_offers(new_offer_amount, NFT_ID_1, alice.clone()).unwrap();
		Rent::<T>::make_rent_offer(origin::<T>("BOB").into(), NFT_ID_1).unwrap();

	}: _(origin::<T>("BOB"), NFT_ID_1)
	verify {
		// Check that offer has been removed
		let offers = Rent::<T>::offers(NFT_ID_1).unwrap();
		assert!(!offers.contains(&bob))
	}

	change_subscription_terms {
		let s in 0 .. T::SimultaneousContractLimit::get();
		prepare_benchmarks::<T>();

		let mut new_contracts_amount = s;
		if new_contracts_amount > 2 {
			new_contracts_amount -= 2;
		};

		let alice: T::AccountId = get_account::<T>("ALICE");
		let org = origin::<T>("ALICE");

		Rent::<T>::benchmark_fill_available_queue(new_contracts_amount, 100u32.into()).unwrap();
	}: _(origin::<T>("ALICE"), NFT_ID_0, 500u32.into(), 2u32.into(), Some(10u32.into()), false)
	verify {
		let contract = Rent::<T>::contracts(NFT_ID_0).unwrap();
		let sub = contract.duration.as_subscription().unwrap();
		assert_eq!(sub.new_terms, true);
	}

	 accept_subscription_terms {
		let s in 0 .. T::SimultaneousContractLimit::get();
		prepare_benchmarks::<T>();

		let mut new_contracts_amount = s;
		if new_contracts_amount > 2 {
			new_contracts_amount -= 2;
		};

		let alice: T::AccountId = get_account::<T>("ALICE");
		let org = origin::<T>("ALICE");

		Rent::<T>::benchmark_fill_available_queue(new_contracts_amount, 100u32.into()).unwrap();
		Rent::<T>::rent(origin::<T>("BOB").into(), NFT_ID_0).unwrap();
		Rent::<T>::change_subscription_terms(origin::<T>("ALICE").into(), NFT_ID_0, 2u32.into(), 10u32.into(), None, true).unwrap();
	}: _(origin::<T>("BOB"), NFT_ID_0)
	verify {
		let contract = Rent::<T>::contracts(NFT_ID_0).unwrap();
		let sub = contract.duration.as_subscription().unwrap();
		assert_eq!(sub.new_terms, false);
	}
}

impl_benchmark_test_suite!(Rent, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
