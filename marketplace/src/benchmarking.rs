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
use crate::Pallet as Marketplace;
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite};
use frame_support::{assert_ok, traits::Currency};
use frame_system::RawOrigin;
use sp_arithmetic::per_things::Permill;
use sp_runtime::traits::{Bounded, StaticLookup};
use sp_std::prelude::*;
use ternoa_common::traits::NFTExt;

const PERCENT_50: Permill = Permill::from_parts(500000);

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

	// Give them enough caps
	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());

	let marketplace_offchain_data =
		BoundedVec::try_from(vec![1; T::OffchainDataLimit::get() as usize])
			.expect("It will never happen.");

	// Create default marketplace.
	assert_ok!(Marketplace::<T>::create_marketplace(
		alice_origin.into(),
		MarketplaceType::Public,
		Some(MarketplaceFee::Percentage(PERCENT_50)),
		Some(MarketplaceFee::Percentage(PERCENT_50)),
		Some(marketplace_offchain_data),
	));
}

benchmarks! {
	create_marketplace {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let alice_origin = origin::<T>("ALICE");
		let marketplace_offchain_data =
			BoundedVec::try_from(vec![1; T::OffchainDataLimit::get() as usize])
				.expect("It will never happen.");
	}: _(alice_origin, MarketplaceType::Public, Some(MarketplaceFee::Percentage(PERCENT_50)), Some(MarketplaceFee::Percentage(PERCENT_50)), Some(marketplace_offchain_data))
	verify {
		let marketplace_id = Marketplace::<T>::get_next_marketplace_id() - 1;
		assert_eq!(Marketplaces::<T>::contains_key(marketplace_id), true);
		assert_eq!(Marketplaces::<T>::get(marketplace_id).unwrap().owner, alice);
	}

	set_marketplace_owner {
		prepare_benchmarks::<T>();
		let alice_origin = origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());
		let marketplace_id = Marketplace::<T>::get_next_marketplace_id() - 1;
	}: _(alice_origin, marketplace_id, bob_lookup)
	verify {
		assert_eq!(Marketplaces::<T>::get(marketplace_id).unwrap().owner, bob);
	}

	set_marketplace_kind {
		prepare_benchmarks::<T>();
		let alice_origin = origin::<T>("ALICE");
		let marketplace_id = Marketplace::<T>::get_next_marketplace_id() - 1;
	}: _(alice_origin, marketplace_id, MarketplaceType::Private)
	verify {
		assert_eq!(Marketplaces::<T>::get(marketplace_id).unwrap().kind, MarketplaceType::Private);
	}

	set_marketplace_configuration{
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let alice_origin = origin::<T>("ALICE");
		let marketplace_id = Marketplace::<T>::get_next_marketplace_id() - 1;
		let marketplace_offchain_data =
			BoundedVec::try_from(vec![1; T::OffchainDataLimit::get() as usize])
				.expect("It will never happen.");
		let marketplace_account_list: BoundedVec<T::AccountId, T::AccountSizeLimit> =
			BoundedVec::try_from(vec![alice.clone(); (T::AccountSizeLimit::get() / 100) as usize]).unwrap();
	}: _(alice_origin, marketplace_id, ConfigOp::Set(MarketplaceFee::Percentage(PERCENT_50)), ConfigOp::Set(MarketplaceFee::Percentage(PERCENT_50)), ConfigOp::Set(marketplace_account_list.clone()), ConfigOp::Set(marketplace_offchain_data.clone()))
	verify {
		let marketplace = Marketplaces::<T>::get(marketplace_id).unwrap();
		assert_eq!(marketplace.commission_fee, Some(MarketplaceFee::Percentage(PERCENT_50)));
		assert_eq!(marketplace.listing_fee, Some(MarketplaceFee::Percentage(PERCENT_50)));
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
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let alice_origin = origin::<T>("ALICE");
		let marketplace_id = Marketplace::<T>::get_next_marketplace_id() - 1;
		Marketplace::<T>::set_marketplace_configuration(
			alice_origin.clone().into(),
			marketplace_id,
			ConfigOp::Set(MarketplaceFee::Percentage(PERCENT_50)),
			ConfigOp::Noop,
			ConfigOp::Noop,
			ConfigOp::Noop,
		).unwrap();
		let nft_id = T::NFTExt::create_nft(alice, BoundedVec::default(), PERCENT_50, None, false).unwrap();
	}: _(alice_origin, nft_id, 10u32.into(), marketplace_id)
	verify {
		assert_eq!(T::NFTExt::get_nft(nft_id).unwrap().state.listed_for_sale, true);
		assert!(Marketplace::<T>::nfts_for_sale(nft_id).is_some());
	}

	unlist_nft {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let alice_origin = origin::<T>("ALICE");
		let marketplace_id = Marketplace::<T>::get_next_marketplace_id() - 1;
		let nft_id = T::NFTExt::create_nft(alice, BoundedVec::default(), PERCENT_50, None, false).unwrap();
		Marketplace::<T>::list_nft(alice_origin.clone().into(), nft_id, 10u32.into(), marketplace_id).unwrap();
	}: _(alice_origin, nft_id)
	verify {
		assert_eq!(T::NFTExt::get_nft(nft_id).unwrap().state.listed_for_sale, false);
		assert!(Marketplace::<T>::nfts_for_sale(nft_id).is_none());
	}

	buy_nft {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let alice_origin = origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let bob_origin = origin::<T>("BOB");
		let marketplace_id = Marketplace::<T>::get_next_marketplace_id() - 1;
		Marketplace::<T>::set_marketplace_configuration(
			alice_origin.clone().into(),
			marketplace_id,
			ConfigOp::Set(MarketplaceFee::Percentage(PERCENT_50)),
			ConfigOp::Noop,
			ConfigOp::Noop,
			ConfigOp::Noop,
		).unwrap();
		let nft_id = T::NFTExt::create_nft(alice, BoundedVec::default(), PERCENT_50, None, false).unwrap();
		Marketplace::<T>::list_nft(alice_origin.into(), nft_id, 10u32.into(), marketplace_id).unwrap();
	}: _(bob_origin, nft_id)
	verify {
		assert!(Marketplace::<T>::nfts_for_sale(nft_id).is_none());
		assert_eq!(T::NFTExt::get_nft(nft_id).unwrap().owner, bob);
	}
}

impl_benchmark_test_suite!(
	Marketplace,
	crate::tests::mock::new_test_ext(),
	crate::tests::mock::Test
);
