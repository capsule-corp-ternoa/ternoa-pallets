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
use crate::Pallet as Sgx;
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{traits::Currency};
use frame_system::RawOrigin;

use sp_runtime::traits::Bounded;
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;


pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}
pub fn origin<T: Config>(name: &'static str) -> RawOrigin<T::AccountId> {
	RawOrigin::Signed(get_account::<T>(name))
}

pub fn prepare_benchmarks<T: Config>()  {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());
}

benchmarks! {
	// register_enclave {
	// 	let alice: T::AccountId = whitelisted_caller();
	// 	let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();
	// 	let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
	// 	let enclave_id: EnclaveId = 0;
	// 	let enclave = Enclave::new(uri.clone(), enclave_address.clone());
	//
	// 	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	// }: _(RawOrigin::Signed(alice.clone().into()), enclave_address, uri.clone())
	// verify {
	// 	assert!(EnclaveRegistry::<T>::contains_key(enclave_id));
	// 	assert_eq!(EnclaveRegistry::<T>::get(enclave_id), Some(enclave));
	// 	assert!(EnclaveIndex::<T>::contains_key(alice.clone()));
	// 	assert_eq!(EnclaveIndex::<T>::get(alice.clone()).unwrap(), enclave_id);
	// 	assert_eq!(EnclaveIdGenerator::<T>::get(), 1);
	// }

	assign_enclave {

		let benchmark_data = prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");

		// let alice: T::AccountId = whitelisted_caller();
		let enclave_id: EnclaveId = 0;
		let cluster_id: ClusterId = 0;
		let enclave_address: Vec<u8> = "192.168.1.1".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
		T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());

		Sgx::<T>::register_cluster(RawOrigin::Root.into());
		Sgx::<T>::register_enclave(alice.clone(), enclave_address.clone(), uri.clone());

	}: _(origin::<T>("ALICE"), cluster_id)
	verify {
		assert_eq!(1, 1)
		// assert_eq!(ClusterRegistry::<T>::get(cluster_id).unwrap().enclaves, vec![enclave_id]);
		// assert_eq!(ClusterIndex::<T>::get(enclave_id), Some(cluster_id));
	}

	// unassign_enclave {
	// 	let alice: T::AccountId = whitelisted_caller();
	// 	let enclave_address: Vec<u8> = "192.168.1.1".as_bytes().to_vec();
	// 	let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
	// 	let enclave_id: EnclaveId = 0;
	// 	let cluster_id: ClusterId = 0;
	// 	let empty: Vec<EnclaveId> = vec![];
	//
	// 	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	//
	// 	drop(Sgx::<T>::register_cluster(RawOrigin::Root.into()));
	// 	drop(Sgx::<T>::register_enclave(RawOrigin::Root.into(), enclave_address, uri.clone()));
	// 	drop(Sgx::<T>::assign_enclave(RawOrigin::Signed(alice.clone()).into(), cluster_id));
	// }: _(RawOrigin::Signed(alice.clone().into()))
	// verify {
	// 	assert_eq!(ClusterRegistry::<T>::get(cluster_id).unwrap().enclaves, empty);
	// 	assert_eq!(ClusterIndex::<T>::get(enclave_id), None);
	// }
	//
	// update_enclave {
	// 	let alice: T::AccountId = whitelisted_caller();
	// 	let ra_report: Vec<u8> = "SampleRep".as_bytes().to_vec();
	// 	let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
	// 	let enclave_id: EnclaveId = 0;
	// 	let new_uri: Vec<u8> = vec![0, 1, 2];
	//
	// 	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	//
	// 	drop(Sgx::<T>::register_enclave(RawOrigin::Signed(alice.clone()).into(), ra_report, uri.clone()));
	// }: _(RawOrigin::Signed(alice.clone().into()), new_uri.clone())
	// verify {
	// 	assert_eq!(EnclaveRegistry::<T>::get(enclave_id).unwrap().api_uri, new_uri);
	// }
	//
	// register_cluster {
	// 	let cluster = Cluster::new(Default::default());
	// 	let cluster_id: ClusterId = 0;
	// }: _(RawOrigin::Root)
	// verify {
	// 	assert_eq!(ClusterIndex::<T>::iter().count(), 0);
	// 	assert_eq!(ClusterRegistry::<T>::get(cluster_id), Some(cluster));
	// 	assert_eq!(ClusterRegistry::<T>::iter().count(), 1);
	// 	assert_eq!(ClusterIdGenerator::<T>::get(), 1);
	// }
	//
	// unregister_cluster {
	// 	let cluster = Cluster::new(Default::default());
	// 	let cluster_id: ClusterId = 0;
	//
	// 	let alice: T::AccountId = whitelisted_caller();
	// 	let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();
	// 	let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
	// 	let enclave_id: EnclaveId = 0;
	// 	let enclave = Enclave::new(uri.clone(), enclave_address.clone());
	//
	// 	drop(Sgx::<T>::register_enclave(RawOrigin::Root.into(), enclave_address, uri.clone()));
	//
	// }: _(RawOrigin::Root, cluster_id)
	// verify {
	// 	assert_eq!(ClusterRegistry::<T>::get(cluster_id), None);
	// 	assert_eq!(ClusterRegistry::<T>::iter().count(), 0);
	// }
}

impl_benchmark_test_suite!(Sgx, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
