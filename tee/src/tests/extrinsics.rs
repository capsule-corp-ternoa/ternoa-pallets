// Copyright 2022 Capsule Corp (France) SAS.
// This file is part of Ternoa.
//
// Ternoa is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Ternoa is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Ternoa.  If not, see <http://www.gnu.org/licenses/>.

use super::{mock, mock::*};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use frame_system::RawOrigin;

use crate::{
	Cluster, ClusterData, Enclave, EnclaveAccountOperator, EnclaveClusterId, EnclaveData,
	EnclaveRegistrations, EnclaveUnregistrations, Error,
};

fn origin(account: u64) -> mock::RuntimeOrigin {
	RawOrigin::Signed(account).into()
}
fn root() -> mock::RuntimeOrigin {
	RawOrigin::Root.into()
}

mod register_enclave {
	use super::*;

	#[test]
	fn register_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));

				let expected = Enclave::new(EVE, api_uri.clone());
				assert_eq!(EnclaveRegistrations::<Test>::get(ALICE), Some(expected));
				assert!(EnclaveData::<Test>::get(ALICE).is_none());
				assert!(EnclaveAccountOperator::<Test>::get(ALICE).is_none());
			})
	}

	#[test]
	fn register_enclave_enclave_address_exists() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));

				assert_noop!(
					TEE::register_enclave(alice.clone(), EVE, api_uri),
					Error::<Test>::RegistrationAlreadyExists
				);
			})
	}
}

mod remove_enclave_registration {
	use super::*;

	#[test]
	fn remove_registration() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();
				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert!(EnclaveRegistrations::<Test>::get(ALICE).is_some());
				assert_ok!(TEE::remove_registration(root(), ALICE));
				assert!(EnclaveRegistrations::<Test>::get(ALICE).is_none());
			})
	}
}

mod unresgiter_enclave {
	use super::*;
	use frame_support::traits::Len;

	#[test]
	fn unregister_unassigned_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert_ok!(TEE::unregister_enclave(alice.clone()));

				assert!(EnclaveRegistrations::<Test>::get(ALICE).is_none());
			})
	}

	#[test]
	fn unregister_assigned_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_ok!(TEE::unregister_enclave(alice.clone()));
				assert_eq!(EnclaveUnregistrations::<Test>::get().len(), 1);
			})
	}

	#[test]
	fn unregistration_called_morethan_once() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_ok!(TEE::unregister_enclave(alice.clone()));
				assert_noop!(
					TEE::unregister_enclave(alice.clone()),
					Error::<Test>::UnregistrationAlreadyExists
				);
			})
	}
}

// TODO: This is failing, not all scenarios been covered
mod remove_enclave {
	use super::*;

	#[test]
	fn remove_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();
				// origin, enclave_address, api_uri
				// origin is operator which is alice
				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				// origin is root
				// operator address is alice
				// cluster -> 0
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				// origin -> root
				// operator address -> alice
				assert_ok!(TEE::remove_enclave(root(), ALICE));
			})
	}
}

mod create_cluster {
	use super::*;

	#[test]
	fn create_cluster() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TEE::create_cluster(root()));
			let cluster = Cluster::new(Default::default());
			assert_eq!(ClusterData::<Test>::get(0), Some(cluster));
		})
	}
}

mod remove_cluster {
	use super::*;

	#[test]
	fn remove_cluster() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TEE::create_cluster(root()));
			assert_ok!(TEE::remove_cluster(root(), 0u32));
			assert!(ClusterData::<Test>::get(0).is_none());
		})
	}

	#[test]
	fn remove_invalid_cluster_id() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(TEE::remove_cluster(root(), 0u32), Error::<Test>::ClusterNotFound);
		})
	}

	#[test]
	fn cluster_is_not_empty() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE, 0));

				assert_noop!(TEE::remove_cluster(root(), 0u32), Error::<Test>::ClusterIsNotEmpty);
			})
	}
}

mod assign_enclave {
	use super::*;

	#[test]
	fn assign_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				let enclave_data = Enclave::new(EVE.clone(), api_uri);
				assert_eq!(EnclaveData::<Test>::get(ALICE), Some(enclave_data));

				let cluster_id = 0;
				assert_eq!(EnclaveClusterId::<Test>::get(ALICE), Some(cluster_id));
			})
	}

	#[test]
	fn enclave_registration_not_found() {
		ExtBuilder::default().tokens(vec![(ALICE, 1000)]).build().execute_with(|| {
			assert_noop!(
				TEE::assign_enclave(root(), ALICE, 0),
				Error::<Test>::RegistrationNotFound
			);
		})
	}

	#[test]
	fn cluster_not_found() {
		ExtBuilder::default().tokens(vec![(ALICE, 1000)]).build().execute_with(|| {
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let api_uri: BoundedVec<u8, MaxUriLen> =
				"enclave_api".as_bytes().to_vec().try_into().unwrap();

			assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
			assert_noop!(TEE::assign_enclave(root(), ALICE, 0), Error::<Test>::ClusterNotFound);
		})
	}

	#[test]
	fn cluster_is_full() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (BOB, 1000)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let bob: mock::RuntimeOrigin = origin(BOB);
				let eve: mock::RuntimeOrigin = origin(EVE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				let api_uri_1: BoundedVec<u8, MaxUriLen> =
					"enclave_api1".as_bytes().to_vec().try_into().unwrap();

				let api_uri_2: BoundedVec<u8, MaxUriLen> =
					"enclave_api2".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), ALICE, api_uri.clone()));
				assert_ok!(TEE::register_enclave(bob.clone(), BOB, api_uri_1.clone()));
				assert_ok!(TEE::register_enclave(eve.clone(), EVE, api_uri_2.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE, 0));
				assert_ok!(TEE::assign_enclave(root(), BOB, 0));
				assert_noop!(TEE::assign_enclave(root(), EVE, 0), Error::<Test>::ClusterIsFull);
			})
	}
}

// #[test]
// fn register_enclave_success() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 100), (BOB, 0), (DAVE, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let valid_uri = "https://va".as_bytes().to_vec();
// 			let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
//
// 			assert_eq!(AccountEnclaveId::<Test>::iter().count(), 0);
// 			assert_eq!(EnclaveData::<Test>::iter().count(), 0);
// 			assert_eq!(EnclaveIdGenerator::<Test>::get(), 0);
//
// 			// Alice should be able to create an enclave if she has enough tokens.
// 			assert_ok!(TEE::register_enclave(
// 				alice.clone(),
// 				enclave_address.clone(),
// 				valid_uri.clone()
// 			));
// 			assert_eq!(Balances::free_balance(ALICE), 95);
//
// 			let enclave = Enclave::new(valid_uri.clone(), enclave_address.clone());
// 			let enclave_id: EnclaveId = 0;
//
// 			assert!(EnclaveData::<Test>::contains_key(enclave_id));
// 			assert_eq!(EnclaveData::<Test>::get(enclave_id), Some(enclave));
// 			assert!(AccountEnclaveId::<Test>::contains_key(ALICE));
// 			assert_eq!(AccountEnclaveId::<Test>::get(ALICE).unwrap(), enclave_id);
// 			assert_eq!(EnclaveIdGenerator::<Test>::get(), 1);
// 		})
// }
//
// #[test]
// fn register_enclave_insufficient_balance() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 100), (BOB, 0), (DAVE, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let valid_uri = "https://va".as_bytes().to_vec();
// 			let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
//
// 			let ok = TEE::register_enclave(bob, enclave_address.clone(), valid_uri.clone());
// 			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
// 		})
// }
//
// #[test]
// fn register_enclave_erroneous_url() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 100), (BOB, 0), (DAVE, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let short_uri = "http".as_bytes().to_vec();
// 			let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();
// 			let long_uri = "https://this".as_bytes().to_vec();
//
// 			let dave: mock::RuntimeOrigin = RawOrigin::Signed(DAVE).into();
//
// 			// Dave should NOT be able to create an enclave if the uri is too short.
// 			let ok = TEE::register_enclave(dave.clone(), enclave_address.clone(), short_uri);
// 			assert_noop!(ok, Error::<Test>::UriTooShort);
//
// 			// Dave should NOT be able to create an enclave if the uri is too long.
// 			let ok = TEE::register_enclave(dave, enclave_address.clone(), long_uri);
// 			assert_noop!(ok, Error::<Test>::UriTooLong);
// 		})
// }
//
// #[test]
// fn assign_enclave_success() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10), (DAVE, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			let valid_uri = "https://va".as_bytes().to_vec();
//
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
// 			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
// 			let dave: mock::RuntimeOrigin = RawOrigin::Signed(DAVE).into();
//
// 			let cluster_id: ClusterId = 0;
// 			let enclave_id: EnclaveId = 0;
//
// 			assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
// 			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.to_vec(), valid_uri.clone()));
//
// 			// Alice should be able to assign her enclave to a cluster.
// 			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
// 			let cluster = ClusterData::<Test>::get(cluster_id).unwrap();
// 			assert_eq!(cluster.enclaves, vec![enclave_id]);
// 			assert_eq!(EnclaveClusterId::<Test>::get(enclave_id), Some(cluster_id));
//
// 			// Bob should NOT be able to assign his enclave to an non existing cluster.
// 			assert_ok!(TEE::register_enclave(bob.clone(), att_rep.to_vec(), valid_uri.clone()));
// 			let ok = TEE::assign_enclave(bob.clone(), 1);
// 			assert_noop!(ok, Error::<Test>::UnknownClusterId);
//
// 			// Dave should NOT be able to register his enclave if the cluster is already full.
// 			assert_ok!(TEE::assign_enclave(bob, cluster_id));
// 			assert_ok!(TEE::register_enclave(dave.clone(), att_rep.to_vec(), valid_uri));
//
// 			let ok = TEE::assign_enclave(dave, 0);
// 			assert_noop!(ok, Error::<Test>::ClusterIsFull);
// 		})
// }
//
// #[test]
// fn assignig_when_cluster_is_full() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10), (DAVE, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			let valid_uri = "https://va".as_bytes().to_vec();
//
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
// 			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
// 			let dave: mock::RuntimeOrigin = RawOrigin::Signed(DAVE).into();
//
// 			let cluster_id: ClusterId = 0;
//
// 			assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
// 			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.to_vec(), valid_uri.clone()));
// 			assert_ok!(TEE::register_enclave(dave.clone(), att_rep.to_vec(), valid_uri.clone()));
// 			assert_ok!(TEE::register_enclave(bob.clone(), att_rep.to_vec(), valid_uri));
// 			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
// 			assert_ok!(TEE::assign_enclave(bob, cluster_id));
//
// 			assert_noop!(TEE::assign_enclave(dave, cluster_id), Error::<Test>::ClusterIsFull);
// 		})
// }
//
// #[test]
// fn re_assigning_enclave_to_same_cluster() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10), (DAVE, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			let valid_uri = "https://va".as_bytes().to_vec();
//
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
//
// 			let cluster_id: ClusterId = 0;
//
// 			assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
// 			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.to_vec(), valid_uri.clone()));
//
// 			// Alice should be able to assign her enclave to a cluster.
// 			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
//
// 			// Alice should NOT be able to assign her enclave if it is already assigned.
// 			let ok = TEE::assign_enclave(alice, cluster_id);
// 			assert_noop!(ok, Error::<Test>::EnclaveAlreadyAssigned);
// 		})
// }
//
// #[test]
// fn assigning_enclave_to_unknown_cluster() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10), (DAVE, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			let valid_uri = "https://va".as_bytes().to_vec();
//
// 			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
//
// 			// Bob should NOT be able to assign his enclave to an non existing cluster.
// 			assert_ok!(TEE::register_enclave(bob.clone(), att_rep.to_vec(), valid_uri.clone()));
// 			let ok = TEE::assign_enclave(bob.clone(), 1);
// 			assert_noop!(ok, Error::<Test>::UnknownClusterId);
// 		})
// }
//
// #[test]
// fn unassign_enclave_success() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let valid_uri = "https://va".as_bytes().to_vec();
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
// 			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			let cluster_id: ClusterId = 0;
// 			let enclave_id: EnclaveId = 0;
//
// 			assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
// 			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri));
// 			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
// 			let cluster = ClusterData::<Test>::get(cluster_id).unwrap();
// 			assert_eq!(cluster.enclaves, vec![enclave_id]);
// 			assert_eq!(EnclaveClusterId::<Test>::get(enclave_id), Some(cluster_id));
//
// 			// Alice should be able to unassigned her enclave from a cluster.
// 			assert_ok!(TEE::unassign_enclave(alice.clone()));
// 			let cluster = ClusterData::<Test>::get(cluster_id).unwrap();
// 			let empty: Vec<EnclaveId> = Default::default();
// 			assert_eq!(cluster.enclaves, empty);
// 			assert_eq!(EnclaveClusterId::<Test>::get(enclave_id), None);
// 		})
// }
//
// #[test]
// fn unassign_an_uassigned_enclave() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let valid_uri = "https://va".as_bytes().to_vec();
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
// 			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			let cluster_id: ClusterId = 0;
//
// 			assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
// 			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri));
// 			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
//
// 			// Alice should be able to unassigned her enclave from a cluster.
// 			assert_ok!(TEE::unassign_enclave(alice.clone()));
//
// 			// Alice should NOT be able to unassigned her enclave if the enclave is already
// 			// unassigned.
// 			assert_noop!(TEE::unassign_enclave(alice.clone()), Error::<Test>::EnclaveNotAssigned);
// 		})
// }
//
// #[test]
// fn unassign_by_non_enclave_owner() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
// 			let ok = TEE::unassign_enclave(bob.clone());
// 			assert_noop!(ok, Error::<Test>::EnclaveNotFound);
// 		})
// }
//
// #[test]
// fn update_enclave() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let mut valid_uri = "https://va".as_bytes().to_vec();
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
// 			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
// 			let enclave_address: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
// 			let enclave_id: EnclaveId = 0;
//
// 			// Alice should be able to update her enclave.
// 			valid_uri = "https://zza".as_bytes().to_vec();
// 			let enclave = Enclave::new(valid_uri.clone(), enclave_address);
// 			assert_ok!(TEE::update_enclave(alice.clone(), valid_uri.clone()));
// 			assert_eq!(EnclaveData::<Test>::get(enclave_id), Some(enclave));
// 		})
// }
//
// #[test]
// fn update_enclave_by_nop_operator_account() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let mut valid_uri = "https://va".as_bytes().to_vec();
//
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
// 			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
// 			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
// 			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
//
// 			// Alice should be able to update her enclave.
// 			valid_uri = "https://tna".as_bytes().to_vec();
//
// 			let ok = TEE::update_enclave(bob.clone(), valid_uri.clone());
// 			assert_noop!(ok, Error::<Test>::EnclaveNotFound);
// 		})
// }
//
// #[test]
// fn register_cluster_success() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		assert_eq!(EnclaveClusterId::<Test>::iter().count(), 0);
// 		assert_eq!(ClusterData::<Test>::iter().count(), 0);
// 		assert_eq!(ClusterIdGenerator::<Test>::get(), 0);
// 		assert_eq!(EnclaveIdGenerator::<Test>::get(), 0);
// 		let cluster_id: ClusterId = 0;
// 		let cluster = Cluster::new(Default::default());
//
// 		// Sudo should be able to create clusters.
// 		assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
// 		assert_eq!(EnclaveClusterId::<Test>::iter().count(), 0);
// 		assert_eq!(ClusterData::<Test>::get(cluster_id), Some(cluster));
// 		assert_eq!(ClusterIdGenerator::<Test>::get(), 1);
// 		assert_eq!(EnclaveIdGenerator::<Test>::get(), 0);
// 	})
// }
//
// #[test]
// fn register_cluster_bad_origin() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
// 		assert_noop!(TEE::register_cluster(alice.clone()), BadOrigin);
// 	})
// }
//
// #[test]
// fn unregister_cluster() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
// 			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
// 			let valid_uri = "https://va".as_bytes().to_vec();
// 			let cluster_id: ClusterId = 0;
// 			let att_rep: Vec<u8> = "samplere".as_bytes().to_vec();
//
// 			assert_ok!(TEE::register_cluster(RawOrigin::Root.into()));
// 			assert_ok!(TEE::register_enclave(alice.clone(), att_rep.clone(), valid_uri.clone()));
// 			assert_ok!(TEE::register_enclave(bob.clone(), att_rep.clone(), valid_uri.clone()));
// 			assert_ok!(TEE::assign_enclave(alice.clone(), cluster_id));
// 			assert_ok!(TEE::assign_enclave(bob.clone(), cluster_id));
//
// 			// Sudo should be remove an existing cluster
// 			assert_ok!(TEE::unregister_cluster(RawOrigin::Root.into(), cluster_id));
// 			assert_eq!(EnclaveClusterId::<Test>::iter().count(), 0);
// 			assert_eq!(ClusterData::<Test>::iter().count(), 0);
// 		})
// }
//
// #[test]
// fn unregister_unknown_cluster() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let ok = TEE::unregister_cluster(RawOrigin::Root.into(), 10);
// 			assert_noop!(ok, Error::<Test>::UnknownClusterId);
// 		})
// }
//
// #[test]
// fn unregister_cluster_by_unknown_account() {
// 	ExtBuilder::default()
// 		.tokens(vec![(ALICE, 10), (BOB, 10)])
// 		.build()
// 		.execute_with(|| {
// 			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
//
// 			assert_noop!(TEE::unregister_cluster(alice.clone(), 1), BadOrigin);
// 		})
// }
