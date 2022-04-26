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
use sp_runtime::traits::Bounded;
use sp_std::prelude::*;

use crate::Pallet as ChainBridge;

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
}

benchmarks! {
	set_threshold {
		prepare_benchmarks::<T>();

		let root = RawOrigin::Root;
		let threshold = 3;

	}: _(root, threshold)
	verify {
		assert_eq!(RelayerVoteThreshold::<T>::get(), threshold);
	}

	whitelist_chain {
		prepare_benchmarks::<T>();

		let root = RawOrigin::Root;
		let chain_id = 0;

	}: _(root, chain_id)
	verify {
		assert!(ChainNonces::<T>::get(chain_id).is_some());
	}

	set_relayers {
		prepare_benchmarks::<T>();

		let root = RawOrigin::Root;
		let relayer_a = get_account::<T>("RELAYER_A");
		let relayer_b = get_account::<T>("RELAYER_B");
		let relayer_c = get_account::<T>("RELAYER_C");
		let relayers: BoundedVec<T::AccountId, T::RelayerCountLimit> = bounded_vec![relayer_a, relayer_b, relayer_c];

	}: _(root, relayers.clone())
	verify {
		assert_eq!(Relayers::<T>::get(), relayers);
	}

	vote_for_proposal {
		prepare_benchmarks::<T>();

		let root = RawOrigin::Root;
		let relayer_a: T::AccountId = get_account::<T>("RELAYER_A");
		assert_ok!(ChainBridge::<T>::set_relayers(root.clone().into(), bounded_vec![relayer_a.clone()]));

		let recipient: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(get_account::<T>("RELAYER_C"));
		let chain_id = 0;

		assert_ok!(ChainBridge::<T>::whitelist_chain(root.clone().into(), chain_id));

		let amount = 100u32;
		let deposit_nonce = ChainNonces::<T>::get(chain_id).unwrap();
		// let proposal = ChainBridge::get_votes(chain_id, (deposit_nonce, recipient, amount));

	}: _(RawOrigin::Signed(relayer_a), chain_id, deposit_nonce, recipient, amount.into(), true)
	verify {
		// let proposal = ChainBridge::get_votes(chain_id, (deposit_nonce, recipient, amount));
		// assert!(proposal.is_some());
		// let count = proposal.unwrap().votes.iter().filter(|x| x.1 == true).count() as u32;
		// assert_eq!(count, 1);
	}

	deposit {
		prepare_benchmarks::<T>();

		let root = RawOrigin::Root;
		let relayer_a: T::AccountId = get_account::<T>("RELAYER_A");
		let collector: T::AccountId = get_account::<T>("COLLECTOR");
		let amount: BalanceOf<T> = 10u32.into();
		let chain_id = 1;
		let bridge_fee: BalanceOf<T> = 100u32.into();
		let recipient = vec![0];
		let deposit_nonce = ChainBridge::<T>::chain_nonces(chain_id);
		// let total_issuance: BalanceOf<T> = BalanceOf::total_issuance();
		// let collector_before = Balances::free_balance(&collector);
		// let relayer_a_balance_before: BalanceOf<T> = BalanceOf::<T>::free_balance(&relayer_a);

		assert_ok!(ChainBridge::<T>::whitelist_chain(root.clone().into(), chain_id));
		assert_ok!(ChainBridge::<T>::set_bridge_fee(root.into(), bridge_fee));

	}: _(RawOrigin::Signed(relayer_a), amount.clone().into(), recipient, chain_id)
	verify {
		// assert_eq!(BalanceOf::total_issuance(), total_issuance.into() - amount.into());
		// assert_eq!(BalanceOf::free_balance(&relayer_a), relayer_a_balance_before - amount - bridge_fee);
		// assert_eq!(BalanceOf::free_balance(&collector), collector_before + bridge_fee.into());
		// assert_eq!(ChainBridge::chain_nonces(chain_id).unwrap(), deposit_nonce.unwrap() + 1);
	}

	set_bridge_fee {
		prepare_benchmarks::<T>();

		let root = RawOrigin::Root;
		let bridge_fee = 100u32;

	}: _(root, bridge_fee.clone().into())
	verify {
		assert_eq!(BridgeFee::<T>::get(), bridge_fee.into());
	}
}

impl_benchmark_test_suite!(
	ChainBridge,
	crate::tests::mock::new_test_ext(),
	crate::tests::mock::MockRuntime
);
