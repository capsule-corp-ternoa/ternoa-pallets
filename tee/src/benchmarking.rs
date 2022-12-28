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
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;

use sp_runtime::traits::Bounded;
use sp_std::prelude::*;

benchmarks! {
	register_enclave {
		let alice: T::AccountId = whitelisted_caller();
		let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
		let enclave_id: EnclaveId = 0;
		let enclave = Enclave::new(uri.clone(), enclave_address.clone());

		T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	}: _(RawOrigin::Signed(alice.clone().into()), enclave_address, uri.clone())
	verify {
		assert!(EnclaveRegistry::<T>::contains_key(enclave_id));
		assert_eq!(EnclaveRegistry::<T>::get(enclave_id), Some(enclave));
		assert!(EnclaveIndex::<T>::contains_key(alice.clone()));
		assert_eq!(EnclaveIndex::<T>::get(alice.clone()).unwrap(), enclave_id);
		assert_eq!(EnclaveIdGenerator::<T>::get(), 1);
	}

	assign_enclave {
		let alice: T::AccountId = whitelisted_caller();
		let enclave_id: EnclaveId = 0;
		let cluster_id: ClusterId = 0;
		let ra_report: Vec<u8> = "SampleRep".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
		T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());

		drop(Sgx::<T>::create_cluster(RawOrigin::Root.into()));
		drop(Sgx::<T>::register_enclave(RawOrigin::Signed(alice.clone()).into(), ra_report, uri.clone()));
	}: _(RawOrigin::Signed(alice.clone().into()), cluster_id)
	verify {
		assert_eq!(ClusterRegistry::<T>::get(cluster_id).unwrap().enclaves, vec![enclave_id]);
		assert_eq!(ClusterIndex::<T>::get(enclave_id), Some(cluster_id));
	}

	unassign_enclave {
		let alice: T::AccountId = whitelisted_caller();
		let ra_report: Vec<u8> = "SampleRep".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
		let enclave_id: EnclaveId = 0;
		let cluster_id: ClusterId = 0;
		let empty: Vec<EnclaveId> = vec![];

		T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());

		drop(Sgx::<T>::create_cluster(RawOrigin::Root.into()));
		drop(Sgx::<T>::register_enclave(RawOrigin::Signed(alice.clone()).into(), ra_report, uri.clone()));
		drop(Sgx::<T>::assign_enclave(RawOrigin::Signed(alice.clone()).into(), cluster_id));
	}: _(RawOrigin::Signed(alice.clone().into()))
	verify {
		assert_eq!(ClusterRegistry::<T>::get(cluster_id).unwrap().enclaves, empty);
		assert_eq!(ClusterIndex::<T>::get(enclave_id), None);
	}

	update_enclave {
		let alice: T::AccountId = whitelisted_caller();
		let ra_report: Vec<u8> = "SampleRep".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
		let enclave_id: EnclaveId = 0;
		let new_uri: Vec<u8> = vec![0, 1, 2];

		T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());

		drop(Sgx::<T>::register_enclave(RawOrigin::Signed(alice.clone()).into(), ra_report, uri.clone()));
	}: _(RawOrigin::Signed(alice.clone().into()), new_uri.clone())
	verify {
		assert_eq!(EnclaveRegistry::<T>::get(enclave_id).unwrap().api_uri, new_uri);
	}

	change_enclave_owner {
		let alice: T::AccountId = whitelisted_caller();
		let bob: T::AccountId = account("bob", 0, 0);
		let bob_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(bob.clone());
		let ra_report: Vec<u8> = "SampleRep".as_bytes().to_vec();
		let uri: Vec<u8> = "127.0.0.1".as_bytes().to_vec();
		T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());

		drop(Sgx::<T>::register_enclave(RawOrigin::Signed(alice.clone()).into(), ra_report, uri.clone()));
	}: _(RawOrigin::Signed(alice.clone().into()), bob_lookup)
	verify {
		assert!(EnclaveIndex::<T>::contains_key(bob.clone()));
		assert!(!EnclaveIndex::<T>::contains_key(alice.clone()));
	}

	create_cluster {
		let cluster = Cluster::new(Default::default());
		let cluster_id: ClusterId = 0;
	}: _(RawOrigin::Root)
	verify {
		assert_eq!(ClusterIndex::<T>::iter().count(), 0);
		assert_eq!(ClusterRegistry::<T>::get(cluster_id), Some(cluster));
		assert_eq!(ClusterRegistry::<T>::iter().count(), 1);
		assert_eq!(ClusterIdGenerator::<T>::get(), 1);
	}

	remove_cluster {
		let cluster = Cluster::new(Default::default());
		let cluster_id: ClusterId = 0;

		drop(Sgx::<T>::create_cluster(RawOrigin::Root.into()));
	}: _(RawOrigin::Root, cluster_id)
	verify {
		assert_eq!(ClusterRegistry::<T>::get(cluster_id), None);
		assert_eq!(ClusterRegistry::<T>::iter().count(), 0);
	}
}

impl_benchmark_test_suite!(Sgx, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
