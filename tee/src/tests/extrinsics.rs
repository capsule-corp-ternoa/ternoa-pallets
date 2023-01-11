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
use crate::{
	Cluster, ClusterData, Enclave, EnclaveAccountOperator, EnclaveClusterId, EnclaveData,
	EnclaveRegistrations, EnclaveUnregistrations, EnclaveUpdates, Error,
};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use frame_system::RawOrigin;
use ternoa_common::traits::TEEExt;

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
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));

				let expected = Enclave::new(CHARLIE, api_uri.clone());
				assert_eq!(EnclaveRegistrations::<Test>::get(ALICE), Some(expected));
				assert!(EnclaveData::<Test>::get(ALICE).is_none());
				assert!(EnclaveAccountOperator::<Test>::get(ALICE).is_none());
			})
	}

	#[test]
	fn operator_and_enclave_are_the_same() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_noop!(
					TEE::register_enclave(alice.clone(), ALICE, api_uri),
					Error::<Test>::OperatorAndEnclaveAreSame
				);
			})
	}

	#[test]
	fn registration_already_exists() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));

				assert_noop!(
					TEE::register_enclave(alice.clone(), CHARLIE, api_uri),
					Error::<Test>::RegistrationAlreadyExists
				);
			})
	}

	#[test]
	fn operator_already_exists() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_noop!(
					TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()),
					Error::<Test>::OperatorAlreadyExists
				);
			})
	}

	// Failing
	#[test]
	fn enclave_address_already_exists() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_noop!(
					TEE::register_enclave(alice.clone(), BOB, api_uri.clone()),
					Error::<Test>::EnclaveAddressAlreadyExists
				);
			})
	}
}

mod unregister_enclave {
	use super::*;

	#[test]
	fn unregister_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::unregister_enclave(alice.clone()));

				assert!(EnclaveRegistrations::<Test>::get(ALICE).is_none());
			})
	}

	#[test]
	fn unregistration_limit_reached() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let charlie: mock::RuntimeOrigin = origin(CHARLIE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				TEE::fill_unregistration_list(CHARLIE, 10).unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_noop!(
					TEE::unregister_enclave(charlie.clone()),
					Error::<Test>::UnregistrationLimitReached
				);
			})
	}

	#[test]
	fn unregister_enclave_assigned() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_ok!(TEE::unregister_enclave(alice.clone()));
				assert_eq!(EnclaveUnregistrations::<Test>::get().len(), 1);
			})
	}

	#[test]
	fn unregister_enclave_unassigned() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);

				let err = TEE::unregister_enclave(alice);

				assert_noop!(err, Error::<Test>::RegistrationNotFound);
			})
	}

	// failing
	#[test]
	fn unregistration_already_exists() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
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

mod update_enclave {
	use super::*;

	#[test]
	fn update_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_ok!(TEE::update_enclave(alice.clone(), BOB, new_api_uri.clone()));

				let updated_enclave = Enclave::new(BOB, new_api_uri.clone());

				assert_eq!(EnclaveUpdates::<Test>::get(ALICE).unwrap(), updated_enclave);
			})
	}

	#[test]
	fn operator_and_enclave_are_same() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_noop!(
					TEE::update_enclave(alice.clone(), ALICE, new_api_uri.clone()),
					Error::<Test>::OperatorAndEnclaveAreSame
				);
			})
	}

	#[test]
	fn update_request_already_exists() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, BoundedVec::default()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));
				assert_ok!(TEE::update_enclave(alice.clone(), BOB, BoundedVec::default()));

				let err = TEE::update_enclave(alice.clone(), BOB, BoundedVec::default());
				assert_noop!(err, Error::<Test>::UpdateRequestAlreadyExists);
			})
	}

	#[test]
	fn enclave_not_found() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
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

	#[test]
	fn enclave_address_already_exists() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let eve: mock::RuntimeOrigin = origin(EVE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_ok!(TEE::create_cluster(root()));

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::register_enclave(eve.clone(), DAVE, api_uri.clone()));

				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));
				assert_ok!(TEE::assign_enclave(root(), EVE.clone(), 0));

				assert_noop!(
					TEE::update_enclave(alice.clone(), DAVE, api_uri.clone()),
					Error::<Test>::EnclaveAddressAlreadyExists
				);
			})
	}
}

mod cancel_update {
	use super::*;

	#[test]
	fn cancel_update() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);

				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, BoundedVec::default()));
				assert_ok!(TEE::assign_enclave(root(), ALICE, 0));
				assert_ok!(TEE::update_enclave(alice.clone(), CHARLIE, BoundedVec::default()));
				assert_ok!(TEE::cancel_update(alice.clone()));

				assert!(EnclaveUpdates::<Test>::get(ALICE).is_none());
			})
	}

	#[test]
	fn update_request_not_found() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let err = TEE::cancel_update(alice.clone());
				assert_noop!(err, Error::<Test>::UpdateRequestNotFound);
			})
	}
}

mod assign_enclave {
	use super::*;

	#[test]
	fn assign_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				let enclave_data = Enclave::new(CHARLIE.clone(), api_uri);
				assert_eq!(EnclaveData::<Test>::get(ALICE), Some(enclave_data));

				let cluster_id = 0;
				assert_eq!(EnclaveClusterId::<Test>::get(ALICE), Some(cluster_id));
			})
	}

	#[test]
	fn enclave_address_already_exists() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let bob: mock::RuntimeOrigin = origin(BOB);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();

				// alice -> operator acc , charlie enclave-address acc
				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::register_enclave(bob.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));
				assert_noop!(
					TEE::assign_enclave(root(), BOB.clone(), 0),
					Error::<Test>::EnclaveAddressAlreadyExists
				);
			})
	}

	// This fails
	#[test]
	fn operator_already_exists() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let bob: mock::RuntimeOrigin = origin(BOB);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();
				let api_uri_2: BoundedVec<u8, MaxUriLen> =
					"enclave_2".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::create_cluster(root()));

				// alice -> operator acc , charlie enclave-address acc
				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				// bob -> operator, charlie enc address
				assert_ok!(TEE::register_enclave(bob.clone(), CHARLIE, api_uri_2.clone()));

				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));
				assert_noop!(
					TEE::assign_enclave(root(), ALICE.clone(), 0),
					Error::<Test>::OperatorAlreadyExists
				);
			})
	}

	#[test]
	fn registration_not_found() {
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

			assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
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
				let charlie: mock::RuntimeOrigin = origin(CHARLIE);

				assert_ok!(TEE::register_enclave(
					alice.clone(),
					ALICE_ENCLAVE,
					BoundedVec::default()
				));
				assert_ok!(TEE::register_enclave(bob.clone(), BOB_ENCLAVE, BoundedVec::default()));
				assert_ok!(TEE::register_enclave(
					charlie.clone(),
					CHARLIE_ENCLAVE,
					BoundedVec::default()
				));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE, 0));
				assert_ok!(TEE::assign_enclave(root(), BOB, 0));
				assert_noop!(TEE::assign_enclave(root(), CHARLIE, 0), Error::<Test>::ClusterIsFull);
			})
	}
}

mod remove_registration {
	use super::*;

	#[test]
	fn remove_registration() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> =
					"enclave_api".as_bytes().to_vec().try_into().unwrap();
				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert!(EnclaveRegistrations::<Test>::get(ALICE).is_some());
				assert_ok!(TEE::remove_registration(root(), ALICE));
				assert!(EnclaveRegistrations::<Test>::get(ALICE).is_none());
			})
	}
}

mod remove_update {
	use super::*;
	use crate::tests;

	#[test]
	fn remove_update() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_ok!(TEE::update_enclave(alice.clone(), BOB, new_api_uri.clone()));

				assert_ok!(TEE::remove_update(root(), ALICE));
				assert!(EnclaveUpdates::<Test>::get(ALICE).is_none());

				let event = tests::mock::RuntimeEvent::TEE(crate::Event::UpdateRequestRemoved {
					operator_address: ALICE,
				});
				assert!(System::events().iter().any(|record| record.event == event));
			})
	}

	#[test]
	fn remove_update_with_invalid_operator() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_ok!(TEE::update_enclave(alice.clone(), BOB, new_api_uri.clone()));

				assert_ok!(TEE::remove_update(root(), BOB));
				assert!(EnclaveUpdates::<Test>::get(ALICE).is_some());

				let event_update_request_removed =
					tests::mock::RuntimeEvent::TEE(crate::Event::UpdateRequestRemoved {
						operator_address: ALICE,
					});
				assert!(System::events()
					.iter()
					.any(|record| record.event != event_update_request_removed));
			})
	}
}

mod remove_enclave {
	use super::*;

	#[test]
	fn remove_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_ok!(TEE::remove_enclave(root(), ALICE));
			})
	}

	#[test]
	fn enclave_not_found() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				assert_ok!(TEE::remove_enclave(root(), ALICE));
				assert_noop!(TEE::remove_enclave(root(), ALICE), Error::<Test>::EnclaveNotFound);
			})
	}

	#[test]
	fn cluster_id_not_found() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE.clone(), 0));

				// TODO: Check if this is correct?
				EnclaveClusterId::<Test>::remove(ALICE);

				assert_noop!(TEE::remove_enclave(root(), ALICE), Error::<Test>::ClusterIdNotFound);
			})
	}
}

mod force_update_enclave {
	use super::*;

	#[test]
	fn force_update_enclave() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
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
	fn enclave_address_already_exists() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE, 0));
				assert_ok!(TEE::update_enclave(alice.clone(), BOB, new_api_uri.clone()));
				assert!(EnclaveUpdates::<Test>::get(ALICE).is_some());
				assert_noop!(
					TEE::force_update_enclave(root(), ALICE, BOB, api_uri.clone()),
					Error::<Test>::EnclaveAddressAlreadyExists
				);
			})
	}

	#[test]
	fn operator_and_enclave_are_same() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE, 0));

				assert_noop!(
					TEE::force_update_enclave(root(), ALICE, ALICE, new_api_uri.clone(),),
					Error::<Test>::OperatorAndEnclaveAreSame
				);
			})
	}

	#[test]
	fn enclave_not_found() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();
				let new_api_uri: BoundedVec<u8, MaxUriLen> =
					"new_api_uri".as_bytes().to_vec().try_into().unwrap();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE, 0));

				assert_noop!(
					TEE::force_update_enclave(root(), CHARLIE, BOB, new_api_uri.clone(),),
					Error::<Test>::EnclaveNotFound
				);
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
	fn cluster_not_found() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(TEE::remove_cluster(root(), 0u32), Error::<Test>::ClusterNotFound);
		})
	}

	#[test]
	fn cluster_is_not_empty() {
		ExtBuilder::default()
			.tokens(vec![(ALICE, 1000), (CHARLIE, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::RuntimeOrigin = origin(ALICE);
				let api_uri: BoundedVec<u8, MaxUriLen> = BoundedVec::default();

				assert_ok!(TEE::register_enclave(alice.clone(), CHARLIE, api_uri.clone()));
				assert_ok!(TEE::create_cluster(root()));
				assert_ok!(TEE::assign_enclave(root(), ALICE, 0));

				assert_noop!(TEE::remove_cluster(root(), 0u32), Error::<Test>::ClusterIsNotEmpty);
			})
	}
}
