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
	Cluster, ClusterIdGenerator, ClusterIndex, ClusterRegistry, Enclave,
	EnclaveIdGenerator, EnclaveIndex, EnclaveProviderRegistry, EnclaveRegistry, Error, ProviderId,
	ProviderKeys,
};

use primitives::{
	tee::{ClusterId, EnclaveId},
};

#[test]
fn register_enclave() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 100), (BOB, 0), (DAVE, 10)])
		.build()
		.execute_with(|| {

			let short_uri = "http".as_bytes().to_vec();
			let valid_uri = "https://va".as_bytes().to_vec();
			let att_rep: Vec<u8> = include_bytes!("./mock_attestation.json").to_vec();
			let long_uri = "https://this".as_bytes().to_vec();

			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
			let dave: mock::RuntimeOrigin = RawOrigin::Signed(DAVE).into();

			assert_ok!(TEE::register_enclave_operator(alice.clone(), ALICE));
			assert_ok!(TEE::register_enclave_operator(alice.clone(), BOB));
			assert_ok!(TEE::register_enclave_operator(alice.clone(), DAVE));

			assert_eq!(EnclaveIndex::<Test>::iter().count(), 0);
			assert_eq!(EnclaveRegistry::<Test>::iter().count(), 0);
			assert_eq!(EnclaveIdGenerator::<Test>::get(), 0);

			// Alice should be able to create an enclave if she has enough tokens.
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
			assert_eq!(Balances::free_balance(ALICE), 95);

			let enclave = Enclave::new(valid_uri.clone());
			let enclave_id: EnclaveId = 0;
			assert!(EnclaveRegistry::<Test>::contains_key(enclave_id));
			assert_eq!(EnclaveRegistry::<Test>::get(enclave_id), Some(enclave));
			assert!(EnclaveIndex::<Test>::contains_key(ALICE));
			assert_eq!(EnclaveIndex::<Test>::get(ALICE).unwrap(), enclave_id);
			assert_eq!(EnclaveIdGenerator::<Test>::get(), 1);
			//
			// Alice should NOT be able to create an enclave if she already has one.
			let ok = TEE::register_enclave(alice, att_rep.clone(), valid_uri.clone());
			assert_noop!(ok, Error::<Test>::PublicKeyAlreadyTiedToACluster);

			// Bob should NOT be able to create an enclave if the doesn't have enough tokens.
			let ok = TEE::register_enclave(bob, att_rep.clone(), valid_uri.clone());
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);

			// Dave should NOT be able to create an enclave if the uri is too short.
			let ok = TEE::register_enclave(dave.clone(), att_rep.clone(), short_uri);
			assert_noop!(ok, Error::<Test>::UriTooShort);

			// Dave should NOT be able to create an enclave if the uri is too long.
			let ok = TEE::register_enclave(dave, att_rep, long_uri);
			assert_noop!(ok, Error::<Test>::UriTooLong);
		})
}

#[test]
fn assign_enclave() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10), (DAVE, 10)])
		.build()
		.execute_with(|| {
			let att_rep: &[u8] = include_bytes!("./mock_attestation.json");
			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();

			assert_ok!(TEE::register_enclave_operator(alice.clone(), ALICE));
			assert_ok!(TEE::register_enclave_operator(alice.clone(), BOB));
			assert_ok!(TEE::register_enclave_operator(alice.clone(), DAVE));
			let valid_uri = "https://va".as_bytes().to_vec();

			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
			let dave: mock::RuntimeOrigin = RawOrigin::Signed(DAVE).into();

			let cluster_id: ClusterId = 0;
			let enclave_id: EnclaveId = 0;
			assert_ok!(TEE::create_cluster(RawOrigin::Root.into()));
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.to_vec(), valid_uri.clone()));

			// Alice should be able to assign her enclave to a cluster.
			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
			let cluster = ClusterRegistry::<Test>::get(cluster_id).unwrap();
			assert_eq!(cluster.enclaves, vec![enclave_id]);
			assert_eq!(ClusterIndex::<Test>::get(enclave_id), Some(cluster_id));

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
			let att_rep: Vec<u8> = include_bytes!("./mock_attestation.json").to_vec();

			let cluster_id: ClusterId = 0;
			let enclave_id: EnclaveId = 0;

			assert_ok!(TEE::register_enclave_operator(alice.clone(), ALICE));

			assert_ok!(TEE::create_cluster(RawOrigin::Root.into()));
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri));
			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
			let cluster = ClusterRegistry::<Test>::get(cluster_id).unwrap();
			assert_eq!(cluster.enclaves, vec![enclave_id]);
			assert_eq!(ClusterIndex::<Test>::get(enclave_id), Some(cluster_id));

			// Alice should be able to unassign her enclave from a cluster.
			assert_ok!(TEE::unassign_enclave(alice.clone()));
			let cluster = ClusterRegistry::<Test>::get(cluster_id).unwrap();
			let empty: Vec<EnclaveId> = Default::default();
			assert_eq!(cluster.enclaves, empty);
			assert_eq!(ClusterIndex::<Test>::get(enclave_id), None);

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
			let att_rep: Vec<u8> = include_bytes!("./mock_attestation.json").to_vec();

			assert_ok!(TEE::register_enclave_operator(alice.clone(), ALICE));

			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
			let enclave_id: EnclaveId = 0;

			// Alice should be able to update her enclave.
			valid_uri = "https://zza".as_bytes().to_vec();
			let enclave = Enclave::new(valid_uri.clone());
			assert_ok!(TEE::update_enclave(alice.clone(), valid_uri.clone()));
			assert_eq!(EnclaveRegistry::<Test>::get(enclave_id), Some(enclave));

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

#[test]
fn change_enclave_owner() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			let valid_uri = "https://va".as_bytes().to_vec();
			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let att_rep: Vec<u8> = include_bytes!("./mock_attestation.json").to_vec();

			assert_ok!(TEE::register_enclave_operator(alice.clone(), ALICE));

			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
			let enclave_id: EnclaveId = 0;

			// Alice should be able to change owner of his enclave.
			assert_ok!(TEE::change_enclave_owner(alice.clone(), BOB));
			assert_eq!(EnclaveIndex::<Test>::get(BOB), Some(enclave_id));

			// Alice should NOT be able to change the owner if she doesn't own an enclave.
			let ok = TEE::change_enclave_owner(alice.clone(), BOB);
			assert_noop!(ok, Error::<Test>::NotEnclaveOwner);

			// Alice should NOT be able to change the owner if the new owner already has an enclave.
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep, valid_uri));
			let ok = TEE::change_enclave_owner(alice.clone(), BOB);
			assert_noop!(ok, Error::<Test>::PublicKeyAlreadyTiedToACluster);
		})
}

#[test]
fn create_cluster() {
	ExtBuilder::default().build().execute_with(|| {
		let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();

		assert_eq!(ClusterIndex::<Test>::iter().count(), 0);
		assert_eq!(ClusterRegistry::<Test>::iter().count(), 0);
		assert_eq!(ClusterIdGenerator::<Test>::get(), 0);
		assert_eq!(EnclaveIdGenerator::<Test>::get(), 0);
		let cluster_id: ClusterId = 0;
		let cluster = Cluster::new(Default::default());

		// Sudo should be able to create clusters.
		assert_ok!(TEE::create_cluster(RawOrigin::Root.into()));
		assert_eq!(ClusterIndex::<Test>::iter().count(), 0);
		assert_eq!(ClusterRegistry::<Test>::get(cluster_id), Some(cluster));
		assert_eq!(ClusterIdGenerator::<Test>::get(), 1);
		assert_eq!(EnclaveIdGenerator::<Test>::get(), 0);

		// Alice should NOT be able to create a cluster.
		let ok = TEE::create_cluster(alice.clone());
		assert_noop!(ok, BadOrigin);
	})
}

#[test]
fn remove_cluster() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
			let valid_uri = "https://va".as_bytes().to_vec();
			let cluster_id: ClusterId = 0;
			let cluster = Cluster::new(vec![0, 1]);
			let att_rep: Vec<u8> = include_bytes!("./mock_attestation.json").to_vec();

			assert_ok!(TEE::register_enclave_operator(alice.clone(), ALICE));
			assert_ok!(TEE::register_enclave_operator(alice.clone(), BOB));

			assert_ok!(TEE::create_cluster(RawOrigin::Root.into()));
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
			assert_ok!(TEE::register_enclave(bob.clone(), att_rep.clone(), valid_uri.clone()));
			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
			assert_ok!(TEE::assign_enclave(bob.clone(), cluster_id));

			assert_eq!(ClusterIndex::<Test>::iter().count(), 2);
			assert_eq!(ClusterIndex::<Test>::get(0), Some(0));
			assert_eq!(ClusterIndex::<Test>::get(1), Some(0));
			assert_eq!(ClusterRegistry::<Test>::iter().count(), 1);
			assert_eq!(ClusterRegistry::<Test>::get(0), Some(cluster));
			assert_eq!(ClusterIdGenerator::<Test>::get(), 1);

			// Sudo should be remove an existing cluster
			assert_ok!(TEE::remove_cluster(RawOrigin::Root.into(), cluster_id));
			assert_eq!(ClusterIndex::<Test>::iter().count(), 0);
			assert_eq!(ClusterRegistry::<Test>::iter().count(), 0);

			// Sudo should NOT be able to remove an non-existing cluster
			let ok = TEE::remove_cluster(RawOrigin::Root.into(), 10);
			assert_noop!(ok, Error::<Test>::UnknownClusterId);

			// Alice should NOT be able to remove a cluster.
			let ok = TEE::remove_cluster(alice.clone(), 1);
			assert_noop!(ok, BadOrigin);
		})
}

#[test]
fn register_enclave_provider() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			let amd_provider = "AMD".as_bytes().to_vec();
			let intel_provider = "INTEL".as_bytes().to_vec();

			assert_ok!(TEE::register_enclave_provider(
				RawOrigin::Root.into(),
				amd_provider.clone()
			));
			let amd = EnclaveProviderRegistry::<Test>::get(0).unwrap();

			assert_eq!(amd.enclave_provider_name, amd_provider.clone());
			assert_ok!(TEE::register_enclave_provider(
				RawOrigin::Root.into(),
				intel_provider.clone()
			));

			let intel = EnclaveProviderRegistry::<Test>::get(1).unwrap();
			assert_eq!(intel.enclave_provider_name, intel_provider.clone());

			// Error if provider already exists
			assert_noop!(
				TEE::register_enclave_provider(RawOrigin::Root.into(), intel_provider.clone()),
				Error::<Test>::EnclaveProviderAlreadyRegistered
			);
		})
}

#[test]
fn enclave_operator() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			// Provider should be present
			let amd_provider = "AMD".as_bytes().to_vec();
			let intel_provider = "INTEL".as_bytes().to_vec();

			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();

			// Enclave URI
			let valid_uri = "https://va".as_bytes().to_vec();

			// This is a mocked entry only, does not reflect the actual report
			let att_rep: Vec<u8> = include_bytes!("./mock_attestation.json").to_vec();

			// Register enclave provider -> Registering AMD as the provider
			assert_ok!(TEE::register_enclave_provider(
				RawOrigin::Root.into(),
				amd_provider.clone()
			));
			// Register enclave provider -> Registering intel as the provider
			assert_ok!(TEE::register_enclave_provider(
				RawOrigin::Root.into(),
				intel_provider.clone()
			));

			// Registering BOB as an enclave operator
			assert_ok!(TEE::register_enclave_operator(alice.clone(), BOB));
			// Try to register BOB again
			// Registering BOB again should throw an error
			assert_noop!(
				TEE::register_enclave_operator(alice.clone(), BOB),
				Error::<Test>::EnclaveOperatorExists
			);

			// Register enclave by BOB the operator should not throw an error
			assert_ok!(TEE::register_enclave(bob.clone(), att_rep.clone(), valid_uri.clone()));

			// Alice havent been registered as an enclave operator, should throw an error
			assert_noop!(
				TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()),
				Error::<Test>::UnknownEnclaveOperatorAccount
			);

			// Registering ALICE as an enclave operator
			assert_ok!(TEE::register_enclave_operator(alice.clone(), ALICE));

			// Should not throw an error
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep, valid_uri.clone()));
		})
}

#[test]
fn ensure_enclave() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			let amd_provider = "AMD".as_bytes().to_vec();
			let intel_provider = "INTEL".as_bytes().to_vec();

			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();

			let valid_uri = "https://va".as_bytes().to_vec();
			let cluster_id: ClusterId = 0;

			let att_rep: Vec<u8> = include_bytes!("./mock_attestation.json").to_vec();

			assert_ok!(TEE::register_enclave_operator(alice.clone(), ALICE));
			assert_ok!(TEE::register_enclave_operator(alice.clone(), BOB));

			assert_ok!(TEE::create_cluster(RawOrigin::Root.into()));
			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
			assert_ok!(TEE::register_enclave(bob.clone(), att_rep.clone(), valid_uri.clone()));
			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
			assert_ok!(TEE::assign_enclave(bob.clone(), cluster_id));
			assert_ok!(TEE::register_enclave_provider(
				RawOrigin::Root.into(),
				amd_provider.clone()
			));
			assert_ok!(TEE::register_enclave_provider(
				RawOrigin::Root.into(),
				intel_provider.clone()
			));

			let res = TEE::ensure_enclave(BOB);
			// Returns the registered `clusterId` and `enclaveId` for the given Enclave Operator
			// AccountId
			assert_eq!(res, Some((0, 1)));
		})
}

#[test]
fn register_provider_keys() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let public_key = "MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgHgI3ZgcuSPUzt9bIs857s9198lM"
				.as_bytes()
				.to_vec();
			let enclave_class = Some("X86_64".as_bytes().to_vec());
			let provider_id: ProviderId = 0;
			// Registers INTEL as a provider
			let intel_provider = "INTEL".as_bytes().to_vec();
			assert_ok!(TEE::register_enclave_provider(
				RawOrigin::Root.into(),
				intel_provider.clone()
			));

			assert_ok!(TEE::register_provider_keys(
				alice.clone(),
				provider_id,
				enclave_class.clone(),
				public_key.clone(),
			));

			let intel = ProviderKeys::<Test>::get(0).unwrap();
			assert_eq!(intel.public_key, public_key);

			// Registering public key to already registered provider Id
			assert_noop!(
				TEE::register_provider_keys(
					alice.clone(),
					provider_id,
					enclave_class.clone(),
					public_key.clone(),
				),
				Error::<Test>::ProviderAlreadyRegistered
			);

			// Trying top register a key for not registered enclave provider
			assert_noop!(
				TEE::register_provider_keys(
					alice.clone(),
					1,
					enclave_class.clone(),
					public_key.clone(),
				),
				Error::<Test>::UnregisteredEnclaveProvider
			);

			// Register AMD as the provider but with intel public key
			let amd_provider = "AMD".as_bytes().to_vec();
			assert_ok!(TEE::register_enclave_provider(
				RawOrigin::Root.into(),
				amd_provider.clone()
			));

			assert_noop!(
				TEE::register_provider_keys(alice.clone(), 1, enclave_class, public_key.clone(),),
				Error::<Test>::PublicKeyRegisteredForDifferentEnclaveProvider
			);
		})
}
