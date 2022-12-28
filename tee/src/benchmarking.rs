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
use crate::Pallet as TEE;
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite};
use frame_support::traits::Currency;
use frame_system::RawOrigin;

use sp_runtime::traits::Bounded;
use sp_std::prelude::*;

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

	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());
}

benchmarks! {
	register_enclave {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
		let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
		let enclave_id: EnclaveId = 0;
		let enclave = Enclave::new(uri.clone(), enclave_address.clone());

	}: _(alice.clone(), enclave_address, uri.clone())
	verify {
		assert!(EnclaveData::<T>::contains_key(enclave_id));
		assert_eq!(EnclaveData::<T>::get(enclave_id), Some(enclave));
		assert_eq!(EnclaveIdGenerator::<T>::get(), 1);
		assert_eq!(EnclaveRegistrationList::<T>::get().len(), 1);
		assert!(EnclaveRegistrationList::<T>::get().contains(&enclave_id));
	}

	assign_enclave {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");

		let enclave_id: EnclaveId = 0;
		let cluster_id: ClusterId = 0;
		let enclave_address: Vec<u8> = "192.168.1.1".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();

		TEE::<T>::register_cluster(RawOrigin::Root.into()).unwrap();
		TEE::<T>::register_enclave(alice.clone().into(), enclave_address.clone(), uri.clone()).unwrap();

	}: _(alice, cluster_id)
	verify {
		assert_eq!(ClusterData::<T>::get(cluster_id).unwrap().enclaves, vec![enclave_id]);
		assert_eq!(EnclaveClusterId::<T>::get(enclave_id), Some(cluster_id));
	}

	unassign_enclave {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");

		let enclave_id: EnclaveId = 0;
		let cluster_id: ClusterId = 0;
		let enclave_address: Vec<u8> = "192.168.1.1".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
		let empty: Vec<EnclaveId> = vec![];
		TEE::<T>::register_cluster(RawOrigin::Root.into()).unwrap();
		TEE::<T>::register_enclave(alice.clone().into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(alice.clone().into(), cluster_id).unwrap();
	}: _(alice)
	verify {
		assert_eq!(ClusterData::<T>::get(cluster_id).unwrap().enclaves, empty);
		assert_eq!(EnclaveClusterId::<T>::get(enclave_id), None);
	}

	update_enclave {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");

		let enclave_id: EnclaveId = 0;
		let cluster_id: ClusterId = 0;
		let enclave_address: Vec<u8> = "192.168.1.1".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
		let new_api_uri: Vec<u8> = "168.0.0.1".as_bytes().to_vec();
		let empty: Vec<EnclaveId> = vec![];

		TEE::<T>::register_enclave(alice.clone().into(), enclave_address.clone(), uri.clone()).unwrap();
	}: _(alice, new_api_uri.clone())
	verify {
		assert_eq!(EnclaveData::<T>::get(enclave_id).unwrap().api_uri, new_api_uri);
	}

	register_cluster {
		let cluster = Cluster::new(Default::default());
		let cluster_id: ClusterId = 0;
	}: _(RawOrigin::Root)
	verify {
		assert_eq!(EnclaveClusterId::<T>::iter().count(), 0);
		assert_eq!(ClusterData::<T>::get(cluster_id), Some(cluster));
		assert_eq!(ClusterData::<T>::iter().count(), 1);
		assert_eq!(ClusterIdGenerator::<T>::get(), 1);
	}

	unregister_cluster {
		prepare_benchmarks::<T>();
		let alice = origin::<T>("ALICE");
		let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
		let enclave_id: EnclaveId = 0;
		let cluster_id: ClusterId = 0;
		let enclave = Enclave::new(uri.clone(), enclave_address.clone());

		TEE::<T>::register_cluster(RawOrigin::Root.into()).unwrap();
		TEE::<T>::register_enclave(alice.clone().into(), enclave_address.clone(), uri.clone()).unwrap();

	}: _(RawOrigin::Root, cluster_id)
	verify {
		assert_eq!(ClusterData::<T>::get(cluster_id), None);
		assert_eq!(ClusterData::<T>::iter().count(), 0);
	}
}

impl_benchmark_test_suite!(TEE, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
