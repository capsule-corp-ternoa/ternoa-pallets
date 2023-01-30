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

use super::*;
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite};
use frame_support::{assert_ok, traits::Currency, BoundedVec};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;
use sp_std::prelude::*;

use crate::{ChainId, Pallet as Bridge};

const CHAIN_ID: ChainId = 15;

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}

pub fn prepare_benchmarks<T: Config>() {
	let relayer_a: T::AccountId = get_account::<T>("RELAYER_A");
	let relayer_b: T::AccountId = get_account::<T>("RELAYER_B");
	let relayer_c: T::AccountId = get_account::<T>("RELAYER_C");

	T::Currency::make_free_balance_be(&relayer_a, BalanceOf::<T>::max_value() / 5u32.into());
	T::Currency::make_free_balance_be(&relayer_b, BalanceOf::<T>::max_value() / 5u32.into());
	T::Currency::make_free_balance_be(&relayer_c, BalanceOf::<T>::max_value() / 5u32.into());

	assert_ok!(Bridge::<T>::add_chain(RawOrigin::Root.into(), CHAIN_ID));
}

benchmarks! {
	set_threshold {
		prepare_benchmarks::<T>();
		let threshold = 3;

	}: _(RawOrigin::Root, threshold)
	verify {
		assert_eq!(RelayerVoteThreshold::<T>::get(), threshold);
	}

	add_chain {
		prepare_benchmarks::<T>();
		let chain_id = 14;

	}: _(RawOrigin::Root, chain_id)
	verify {
		assert!(ChainNonces::<T>::get(chain_id).is_some());
	}

	set_relayers {
		prepare_benchmarks::<T>();
		let relayer_a = get_account::<T>("RELAYER_A");
		let relayer_b = get_account::<T>("RELAYER_B");
		let relayer_c = get_account::<T>("RELAYER_C");
		let relayers: BoundedVec<T::AccountId, T::RelayerCountLimit> = BoundedVec::try_from(vec![relayer_a, relayer_b, relayer_c]).expect("It will never happen.");

	}: _(RawOrigin::Root, relayers.clone())
	verify {
		assert_eq!(Relayers::<T>::get(), relayers);
	}

	vote_for_proposal {
		prepare_benchmarks::<T>();
		let relayer_a: T::AccountId = get_account::<T>("RELAYER_A");
		let relayers: BoundedVec<T::AccountId, T::RelayerCountLimit> = BoundedVec::try_from(vec![relayer_a.clone()]).expect("It will never happen.");
		assert_ok!(Bridge::<T>::set_relayers(RawOrigin::Root.into(), relayers));

		let recipient: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(get_account::<T>("RELAYER_C"));
		let amount: BalanceOf<T> = 100u32.into();
		let deposit_nonce = ChainNonces::<T>::get(CHAIN_ID).unwrap();
		let relayer_c: T::AccountId = get_account::<T>("RELAYER_C");
	}: _(RawOrigin::Signed(relayer_a.clone()), CHAIN_ID, deposit_nonce, recipient.clone(), amount.clone())
	verify {
		let proposal = Bridge::<T>::get_votes(CHAIN_ID, (deposit_nonce, relayer_c, amount)).unwrap();
		assert_eq!(proposal.votes.len(), 1);
	}

	deposit {
		prepare_benchmarks::<T>();
		let relayer_a: T::AccountId = get_account::<T>("RELAYER_A");
		let collector: T::AccountId = get_account::<T>("COLLECTOR");
		let relayer_a_old_balance = T::Currency::free_balance(&relayer_a);

		let amount: BalanceOf<T> = 10u32.into();
		let bridge_fee: BalanceOf<T> = 100u32.into();
		let recipient = vec![0];
		let deposit_nonce = Bridge::<T>::chain_nonces(CHAIN_ID);
		assert_ok!(Bridge::<T>::set_bridge_fee(RawOrigin::Root.into(), bridge_fee));

	}: _(RawOrigin::Signed(relayer_a.clone()), amount.clone().into(), recipient, CHAIN_ID)
	verify {
		assert_eq!(T::Currency::free_balance(&relayer_a), relayer_a_old_balance - amount - bridge_fee);
	}

	set_bridge_fee {
		prepare_benchmarks::<T>();
		let bridge_fee = 100u32;

	}: _(RawOrigin::Root, bridge_fee.clone().into())
	verify {
		assert_eq!(BridgeFee::<T>::get(), bridge_fee.into());
	}

	set_deposit_nonce {
		prepare_benchmarks::<T>();
		let nonce = 1;

	}: _(RawOrigin::Root, CHAIN_ID, nonce)
	verify {
		assert_eq!(ChainNonces::<T>::get(CHAIN_ID), Some(nonce));
	}
}

impl_benchmark_test_suite!(Bridge, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
