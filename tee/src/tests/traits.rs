// Copyright 2023 Capsule Corp (France) SAS.
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
use frame_support::{assert_ok, BoundedVec};
use ternoa_common::traits::TEEExt;

fn root() -> mock::RuntimeOrigin {
	RawOrigin::Root.into()
}

#[test]
fn ensure_enclave() {
	ExtBuilder::default()
		.tokens(vec![(ALICE, 10), (BOB, 10)])
		.build()
		.execute_with(|| {
			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
			let cluster_id: ClusterId = 0;
			let api_uri: BoundedVec<u8, MaxUriLen>= b"test".to_vec().try_into().unwrap();

      assert_ok!(TEE::create_cluster(root(), crate::ClusterType::Public));
			assert_ok!(TEE::register_enclave(alice.clone(), ALICE_ENCLAVE, BoundedVec::default()));
			assert_ok!(TEE::register_enclave(bob.clone(), BOB_ENCLAVE, BoundedVec::default()));
			assert_ok!(TEE::assign_enclave(root(), ALICE, cluster_id, 0));
			assert_ok!(TEE::assign_enclave(root(), BOB, cluster_id, 1));

			let res = TEE::ensure_enclave(BOB_ENCLAVE);
			assert!(res.is_some());
		})
}
