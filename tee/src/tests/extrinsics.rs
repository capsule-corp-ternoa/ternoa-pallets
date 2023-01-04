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
	EnclaveRegistrations, EnclaveUnregistrations, EnclaveUpdates, Error,
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

mod unregister_enclave {
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

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

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

mod update_enclave {
	use super::*;

	#[test]
	fn update_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"api_uri".as_bytes().to_vec().try_into().unwrap();
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_ok!(TEE::update_enclave(alice.clone(), BOB, new_api_uri.clone()));

				let updated_enclave = Enclave::new(BOB, new_api_uri.clone());

				assert_eq!(EnclaveUpdates::<Test>::get(ALICE).unwrap(), updated_enclave);
			})
	}

	#[test]
	fn update_invalid_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_noop!(
					TEE::update_enclave(alice.clone(), BOB, new_api_uri.clone()),
					Error::<Test>::EnclaveNotFound
				);
			})
	}
}

mod force_update_enclave {
	use super::*;

	#[test]
	fn force_update_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"api_uri".as_bytes().to_vec().try_into().unwrap();
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE, 0));
				assert_ok!(TEE::update_enclave(alice.clone(), BOB, new_api_uri.clone()));
				assert!(EnclaveUpdates::<Test>::get(ALICE).is_some());
				assert_ok!(TEE::force_update_enclave(root(), ALICE, BOB, new_api_uri.clone()));

				let updated_record = Enclave::new(BOB, new_api_uri);
				assert_eq!(EnclaveData::<Test>::get(ALICE).unwrap(), updated_record);
				assert!(EnclaveUpdates::<Test>::get(ALICE).is_none());
			})
	}

	#[test]
	fn enclave_not_found() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (EVE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"api_uri".as_bytes().to_vec().try_into().unwrap();
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), EVE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE, 0));

				assert_noop!(
					TEE::force_update_enclave(root(), EVE, BOB, new_api_uri.clone(),),
					Error::<Test>::EnclaveNotFound
				);
			})
	}
}
