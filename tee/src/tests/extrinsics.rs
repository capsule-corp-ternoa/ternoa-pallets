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

use super::{mock, mock::*};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use sp_runtime::traits::BadOrigin;
use ternoa_common::traits::TEEExt;

use crate::{
	Cluster, ClusterId, ClusterIdGenerator, EnclaveClusterId, ClusterData, Enclave, EnclaveId,
	EnclaveIdGenerator, AccountEnclaveId, EnclaveData, Error,
};

#[test]
fn register_enclave() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 100), (BOB, 0), (DAVE, 10)])
		.build()
		.execute_with(|| {

			let short_uri = "http".as_bytes().to_vec();
			let valid_uri = "https://va".as_bytes().to_vec();
			let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();
			let long_uri = "https://this".as_bytes().to_vec();

			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
			let dave: mock::RuntimeOrigin = RawOrigin::Signed(DAVE).into();

			assert_eq!(AccountEnclaveId::<Test>::iter().count(), 0);
			assert_eq!(EnclaveData::<Test>::iter().count(), 0);
			assert_eq!(EnclaveIdGenerator::<Test>::get(), 0);

			// Alice should be able to create an enclave if she has enough tokens.
			assert_ok!(TEE::register_enclave(alice.clone(), enclave_address.clone(), valid_uri.clone()));
			assert_eq!(Balances::free_balance(ALICE), 95);

			let enclave = Enclave::new(valid_uri.clone(), enclave_address.clone());
			let enclave_id: EnclaveId = 0;
			assert!(EnclaveData::<Test>::contains_key(enclave_id));
			assert_eq!(EnclaveData::<Test>::get(enclave_id), Some(enclave));
			assert!(AccountEnclaveId::<Test>::contains_key(ALICE));
			assert_eq!(AccountEnclaveId::<Test>::get(ALICE).unwrap(), enclave_id);
			assert_eq!(EnclaveIdGenerator::<Test>::get(), 1);
			//
			// Alice should NOT be able to create an enclave if she already has one.
			// let ok = TEE::register_enclave(alice, enclave_address.clone(),  valid_uri.clone());
			// assert_noop!(ok, Error::<Test>::PublicKeyAlreadyTiedToACluster);

			// Bob should NOT be able to create an enclave if the doesn't have enough tokens.
			let ok = TEE::register_enclave(bob, enclave_address.clone(),  valid_uri.clone());
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);

			// Dave should NOT be able to create an enclave if the uri is too short.
			let ok = TEE::register_enclave(dave.clone(), enclave_address.clone(),  short_uri);
			assert_noop!(ok, Error::<Test>::UriTooShort);

			// Dave should NOT be able to create an enclave if the uri is too long.
			let ok = TEE::register_enclave(dave, enclave_address.clone(),  long_uri);
			assert_noop!(ok, Error::<Test>::UriTooLong);
		})
}

#[test]
fn assign_enclave() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10), (DAVE, 10)])
		.build()
		.execute_with(|| {
			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
			let _alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();


			let valid_uri = "https://va".as_bytes().to_vec();

			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
			let dave: mock::RuntimeOrigin = RawOrigin::Signed(DAVE).into();

			let cluster_id: ClusterId = 0;
			let enclave_id: EnclaveId = 0;
			assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.to_vec(), valid_uri.clone()));

			// Alice should be able to assign her enclave to a cluster.
			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
			let cluster = ClusterData::<Test>::get(cluster_id).unwrap();
			assert_eq!(cluster.enclaves, vec![enclave_id]);
			assert_eq!(EnclaveClusterId::<Test>::get(enclave_id), Some(cluster_id));

			// Alice should NOT be able to assign her enclave if it is already assigned.
			let ok = TEE::assign_enclave(alice, cluster_id);
			assert_noop!(ok, Error::<Test>::EnclaveAlreadyAssigned);

			// Bob should NOT be able to assign his enclave to an non existing cluster.
			assert_ok!(TEE::register_enclave(bob.clone(), att_rep.to_vec(), valid_uri.clone()));
			let ok = TEE::assign_enclave(bob.clone(), 1);
			assert_noop!(ok, Error::<Test>::UnknownClusterId);

			// Dave should NOT be able to register his enclave if the cluster is already full.
			assert_ok!(TEE::assign_enclave(bob, cluster_id));
			assert_ok!(TEE::register_enclave(dave.clone(), att_rep.to_vec(), valid_uri));
			let ok = TEE::assign_enclave(dave, 0);
			assert_noop!(ok, Error::<Test>::ClusterIsAlreadyFull);
		})
}

#[test]
fn unassign_enclave() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			let valid_uri = "https://va".as_bytes().to_vec();
			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();

			let cluster_id: ClusterId = 0;
			let enclave_id: EnclaveId = 0;


			assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri));
			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
			let cluster = ClusterData::<Test>::get(cluster_id).unwrap();
			assert_eq!(cluster.enclaves, vec![enclave_id]);
			assert_eq!(EnclaveClusterId::<Test>::get(enclave_id), Some(cluster_id));

			// Alice should be able to unassign her enclave from a cluster.
			assert_ok!(TEE::unassign_enclave(alice.clone()));
			let cluster = ClusterData::<Test>::get(cluster_id).unwrap();
			let empty: Vec<EnclaveId> = Default::default();
			assert_eq!(cluster.enclaves, empty);
			assert_eq!(EnclaveClusterId::<Test>::get(enclave_id), None);

			// Alice should NOT be able to unassign her enclave if the enclave is already
			// unassigned.
			let ok = TEE::unassign_enclave(alice.clone());
			assert_noop!(ok, Error::<Test>::EnclaveNotAssigned);

			// Bob should NOT be able to unassign his enclave if he does not have one
			let ok = TEE::unassign_enclave(bob.clone());
			assert_noop!(ok, Error::<Test>::NotEnclaveOwner);
		})
}

#[test]
fn update_enclave() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			let mut valid_uri = "https://va".as_bytes().to_vec();
			let long_uri = "https://this".as_bytes().to_vec();
			let short_uri = "http".as_bytes().to_vec();
			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
			let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();

			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
			let enclave_id: EnclaveId = 0;

			// Alice should be able to update her enclave.
			valid_uri = "https://zza".as_bytes().to_vec();
			let enclave = Enclave::new(valid_uri.clone(), enclave_address);
			assert_ok!(TEE::update_enclave(alice.clone(), valid_uri.clone()));
			assert_eq!(EnclaveData::<Test>::get(enclave_id), Some(enclave));

			// Dave should NOT be able to update an enclave if the uri is too short.
			let ok = TEE::update_enclave(alice.clone(), short_uri.clone());
			assert_noop!(ok, Error::<Test>::UriTooShort);

			// Dave should NOT be able to update an enclave if the uri is too long.
			let ok = TEE::update_enclave(alice.clone(), long_uri);
			assert_noop!(ok, Error::<Test>::UriTooLong);

			// Bob should NOT be able to update his enclave if he doesn't have one.
			let ok = TEE::update_enclave(bob.clone(), valid_uri.clone());
			assert_noop!(ok, Error::<Test>::NotEnclaveOwner);
		})
}

// #[test]
// fn change_enclave_owner() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let valid_uri = "https://va".as_bytes().to_vec();
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
// 			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
// 			let enclave_id: EnclaveId = 0;
//
// 			// Alice should be able to change owner of his enclave.
// 			assert_ok!(TEE::change_enclave_owner(alice.clone(), BOB));
// 			assert_eq!(EnclaveIndex::<Test>::get(BOB), Some(enclave_id));
//
// 			// Alice should NOT be able to change the owner if she doesn't own an enclave.
// 			let ok = TEE::change_enclave_owner(alice.clone(), BOB);
// 			assert_noop!(ok, Error::<Test>::NotEnclaveOwner);
//
// 			// Alice should NOT be able to change the owner if the new owner already has an enclave.
// 			assert_ok!(TEE::register_enclave(alice.clone(), att_rep, valid_uri));
// 			// let ok = TEE::change_enclave_owner(alice.clone(), BOB);
// 			// assert_noop!(ok, Error::<Test>::PublicKeyAlreadyTiedToACluster);
// 		})
// }

#[test]
fn register_cluster() {
	ExtBuilder::default().build().execute_with(|| {
		let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();

		assert_eq!(EnclaveClusterId::<Test>::iter().count(), 0);
		assert_eq!(ClusterData::<Test>::iter().count(), 0);
		assert_eq!(ClusterIdGenerator::<Test>::get(), 0);
		assert_eq!(EnclaveIdGenerator::<Test>::get(), 0);
		let cluster_id: ClusterId = 0;
		let cluster = Cluster::new(Default::default());

		// Sudo should be able to create clusters.
		assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
		assert_eq!(EnclaveClusterId::<Test>::iter().count(), 0);
		assert_eq!(ClusterData::<Test>::get(cluster_id), Some(cluster));
		assert_eq!(ClusterIdGenerator::<Test>::get(), 1);
		assert_eq!(EnclaveIdGenerator::<Test>::get(), 0);

		// Alice should NOT be able to create a cluster.
		let ok = TEE::register_cluster(alice.clone());
		assert_noop!(ok, BadOrigin);
	})
}

#[test]
fn unregister_cluster() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
			let valid_uri = "https://va".as_bytes().to_vec();
			let cluster_id: ClusterId = 0;
			let cluster = Cluster::new(vec![0, 1]);
			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();

			assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
			assert_ok!(TEE::register_enclave(bob.clone(), att_rep.clone(), valid_uri.clone()));
			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
			assert_ok!(TEE::assign_enclave(bob.clone(), cluster_id));

			assert_eq!(EnclaveClusterId::<Test>::iter().count(), 2);
			assert_eq!(EnclaveClusterId::<Test>::get(0), Some(0));
			assert_eq!(EnclaveClusterId::<Test>::get(1), Some(0));
			assert_eq!(ClusterData::<Test>::iter().count(), 1);
			assert_eq!(ClusterData::<Test>::get(0), Some(cluster));
			assert_eq!(ClusterIdGenerator::<Test>::get(), 1);

			// Sudo should be remove an existing cluster
			assert_ok!(TEE::unregister_cluster(RawOrigin::Root.into(), cluster_id));
			assert_eq!(EnclaveClusterId::<Test>::iter().count(), 0);
			assert_eq!(ClusterData::<Test>::iter().count(), 0);

			// Sudo should NOT be able to remove an non-existing cluster
			let ok = TEE::unregister_cluster(RawOrigin::Root.into(), 10);
			assert_noop!(ok, Error::<Test>::UnknownClusterId);

			// Alice should NOT be able to remove a cluster.
			let ok = TEE::unregister_cluster(alice.clone(), 1);
			assert_noop!(ok, BadOrigin);
		})
}



#[test]
fn ensure_enclave() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			let _amd_provider = "AMD".as_bytes().to_vec();
			let _intel_provider = "INTEL".as_bytes().to_vec();

			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();

			let valid_uri = "https://va".as_bytes().to_vec();
			let cluster_id: ClusterId = 0;

			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();



			assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
			assert_ok!(TEE::register_enclave(bob.clone(), att_rep.clone(), valid_uri.clone()));
			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
			assert_ok!(TEE::assign_enclave(bob.clone(), cluster_id));


			let res = TEE::ensure_enclave(BOB);
			// Returns the registered `clusterId` and `enclaveId` for the given Enclave Operator
			// AccountId
			assert_eq!(res, Some((0, 1)));
		})
}

