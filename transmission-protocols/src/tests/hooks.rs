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
use crate::{
	tests::{extrinsics::*, mock},
	CancellationPeriod, Event as TransmissionProtocolEvent, TransmissionProtocol,
};
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use ternoa_common::traits::NFTExt;

fn origin(account: u64) -> mock::RuntimeOrigin {
	RawOrigin::Signed(account).into()
}

#[test]
fn transmit_at_block() {
	ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
		prepare_tests();
		let alice: mock::RuntimeOrigin = origin(ALICE);
		let protocol = TransmissionProtocol::AtBlock(10);
		let cancellation = CancellationPeriod::None;

		TransmissionProtocols::set_transmission_protocol(
			alice,
			ALICE_NFT_ID,
			BOB,
			protocol,
			cancellation,
		)
		.unwrap();

		run_to_block(10);

		// State check
		let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
		assert_eq!(nft.owner, BOB);
		assert!(!nft.state.is_transmission);
		assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_none());
		assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), None);
		assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);

		let event = TransmissionProtocolEvent::Transmitted { nft_id: ALICE_NFT_ID };
		let event = RuntimeEvent::TransmissionProtocols(event);
		System::assert_last_event(event);
	})
}

#[test]
fn transmit_at_block_with_reset() {
	ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
		prepare_tests();
		let alice: mock::RuntimeOrigin = origin(ALICE);
		let protocol = TransmissionProtocol::AtBlockWithReset(10);
		let cancellation = CancellationPeriod::None;

		TransmissionProtocols::set_transmission_protocol(
			alice.clone(),
			ALICE_NFT_ID,
			BOB,
			protocol,
			cancellation,
		)
		.unwrap();

		run_to_block(8);

		TransmissionProtocols::reset_timer(alice, ALICE_NFT_ID, 20).unwrap();

		run_to_block(20);

		// State check
		let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
		assert_eq!(nft.owner, BOB);
		assert!(!nft.state.is_transmission);
		assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_none());
		assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), None);
		assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);

		let event = TransmissionProtocolEvent::Transmitted { nft_id: ALICE_NFT_ID };
		let event = RuntimeEvent::TransmissionProtocols(event);
		System::assert_last_event(event);
	})
}

#[test]
fn transmit_on_consent_at_block() {
	ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
		prepare_tests();
		let alice: mock::RuntimeOrigin = origin(ALICE);
		let bob: mock::RuntimeOrigin = origin(BOB);
		let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE, DAVE]).unwrap();
		let protocol =
			TransmissionProtocol::OnConsentAtBlock { consent_list, threshold: 2, block: 10 };
		let cancellation = CancellationPeriod::None;

		TransmissionProtocols::set_transmission_protocol(
			alice.clone(),
			ALICE_NFT_ID,
			BOB,
			protocol,
			cancellation,
		)
		.unwrap();

		TransmissionProtocols::add_consent(alice, ALICE_NFT_ID).unwrap();

		TransmissionProtocols::add_consent(bob, ALICE_NFT_ID).unwrap();

		run_to_block(10);

		// State check
		let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
		assert_eq!(nft.owner, BOB);
		assert!(!nft.state.is_transmission);
		assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_none());
		assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), None);
		assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);

		let event = TransmissionProtocolEvent::Transmitted { nft_id: ALICE_NFT_ID };
		let event = RuntimeEvent::TransmissionProtocols(event);
		System::assert_last_event(event);
	})
}

#[test]
fn multiple_transmit_at_block() {
	ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
		prepare_tests();
		let alice: mock::RuntimeOrigin = origin(ALICE);
		let bob: mock::RuntimeOrigin = origin(BOB);
		let protocol = TransmissionProtocol::AtBlock(10);
		let cancellation = CancellationPeriod::None;

		TransmissionProtocols::set_transmission_protocol(
			alice,
			ALICE_NFT_ID,
			BOB,
			protocol.clone(),
			cancellation.clone(),
		)
		.unwrap();

		TransmissionProtocols::set_transmission_protocol(
			bob,
			BOB_NFT_ID,
			ALICE,
			protocol,
			cancellation,
		)
		.unwrap();

		run_to_block(10);

		// State check
		let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
		assert_eq!(nft.owner, BOB);
		assert!(!nft.state.is_transmission);
		assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_none());
		assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), None);
		assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);

		let nft = NFT::get_nft(BOB_NFT_ID).unwrap();
		assert_eq!(nft.owner, ALICE);
		assert!(!nft.state.is_transmission);
		assert!(TransmissionProtocols::transmissions(BOB_NFT_ID).is_none());
		assert_eq!(TransmissionProtocols::at_block_queue().get(BOB_NFT_ID), None);
		assert_eq!(TransmissionProtocols::on_consent_data(BOB_NFT_ID), None);

		let event1 = RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::Transmitted {
			nft_id: ALICE_NFT_ID,
		});
		let event2 = RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::Transmitted {
			nft_id: BOB_NFT_ID,
		});
		System::assert_has_event(event1);
		System::assert_last_event(event2);
	})
}
