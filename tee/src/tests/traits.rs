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

use super::mock::*;
use frame_system::RawOrigin;
use primitives::tee::ClusterId;

use crate::tests::mock;
use frame_support::assert_ok;
use ternoa_common::traits::TEEExt;
#[test]
fn ensure_enclave() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
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
