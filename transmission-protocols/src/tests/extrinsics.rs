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
use frame_support::{
	assert_noop, assert_ok, dispatch::DispatchResult, error::BadOrigin, BoundedVec,
};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use primitives::nfts::NFTId;
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::NFTExt;

use crate::{
	tests::mock, CancellationPeriod, Error, Event as TransmissionProtocolEvent,
	TransmissionProtocol, TransmissionProtocolKind,
};

pub const ALICE_NFT_ID: NFTId = 0;
pub const BOB_NFT_ID: NFTId = 1;
const INVALID_NFT_ID: NFTId = 1001;
const PERCENT_0: Permill = Permill::from_parts(0);

fn origin(account: u64) -> mock::RuntimeOrigin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::RuntimeOrigin {
	RawOrigin::Root.into()
}

pub fn prepare_tests() {
	let alice: mock::RuntimeOrigin = origin(ALICE);
	let bob: mock::RuntimeOrigin = origin(BOB);

	NFT::create_nft(alice, BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(bob, BoundedVec::default(), PERCENT_0, None, false).unwrap();

	assert_eq!(NFT::nfts(ALICE_NFT_ID).is_some(), true);
	assert_eq!(NFT::nfts(BOB_NFT_ID).is_some(), true);
}

mod set_transmission_protocol {
	use super::*;

	#[test]
	fn set_transmission_protocol_at_block() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);

			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;

			let ok = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_ok!(ok);

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			let transmission_data = TransmissionProtocols::transmissions(ALICE_NFT_ID);
			let mut queue = TransmissionProtocols::at_block_queue();
			assert!(nft.state.is_transmission);
			assert!(transmission_data.is_some());
			assert_eq!(queue.get(ALICE_NFT_ID), Some(10));
			assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance - TransmissionProtocols::at_block_fee()
			);

			// Events checks.
			let event = TransmissionProtocolEvent::ProtocolSet {
				nft_id: ALICE_NFT_ID,
				recipient: BOB,
				protocol: TransmissionProtocol::AtBlock(10),
				cancellation: CancellationPeriod::None,
			};
			let event = RuntimeEvent::TransmissionProtocols(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn set_transmission_protocol_at_block_with_reset() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);

			let protocol = TransmissionProtocol::AtBlockWithReset(10);
			let cancellation = CancellationPeriod::None;

			let ok = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_ok!(ok);

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			let transmission_data = TransmissionProtocols::transmissions(ALICE_NFT_ID);
			let mut queue = TransmissionProtocols::at_block_queue();
			assert!(nft.state.is_transmission);
			assert!(transmission_data.is_some());
			assert_eq!(queue.get(ALICE_NFT_ID), Some(10));
			assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance - TransmissionProtocols::at_block_with_reset_fee()
			);

			// Events checks.
			let event = TransmissionProtocolEvent::ProtocolSet {
				nft_id: ALICE_NFT_ID,
				recipient: BOB,
				protocol: TransmissionProtocol::AtBlockWithReset(10),
				cancellation: CancellationPeriod::None,
			};
			let event = RuntimeEvent::TransmissionProtocols(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn set_transmission_protocol_on_consent() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);

			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE, DAVE]).unwrap();

			let protocol = TransmissionProtocol::OnConsent {
				consent_list: consent_list.clone(),
				threshold: 2,
			};
			let cancellation = CancellationPeriod::None;

			let ok = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_ok!(ok);

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			let transmission_data = TransmissionProtocols::transmissions(ALICE_NFT_ID);
			assert!(nft.state.is_transmission);
			assert!(transmission_data.is_some());
			assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), None);
			assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance - TransmissionProtocols::on_consent_fee()
			);

			// Events checks.
			let event = TransmissionProtocolEvent::ProtocolSet {
				nft_id: ALICE_NFT_ID,
				recipient: BOB,
				protocol: TransmissionProtocol::OnConsent { consent_list, threshold: 2 },
				cancellation: CancellationPeriod::None,
			};
			let event = RuntimeEvent::TransmissionProtocols(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn set_transmission_protocol_on_consent_at_block() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);

			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE, DAVE]).unwrap();

			let protocol = TransmissionProtocol::OnConsentAtBlock {
				consent_list: consent_list.clone(),
				threshold: 2,
				block: 10,
			};
			let cancellation = CancellationPeriod::None;

			let ok = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_ok!(ok);

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			let transmission_data = TransmissionProtocols::transmissions(ALICE_NFT_ID);
			assert!(nft.state.is_transmission);
			assert!(transmission_data.is_some());
			assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), None);
			assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance - TransmissionProtocols::on_consent_at_block_fee()
			);

			// Events checks.
			let event = TransmissionProtocolEvent::ProtocolSet {
				nft_id: ALICE_NFT_ID,
				recipient: BOB,
				protocol: TransmissionProtocol::OnConsentAtBlock {
					consent_list,
					threshold: 2,
					block: 10,
				},
				cancellation: CancellationPeriod::None,
			};
			let event = RuntimeEvent::TransmissionProtocols(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				INVALID_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				BOB_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn invalid_recipient() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				ALICE,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::InvalidRecipient);
		})
	}

	#[test]
	fn cannot_set_transmission_for_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			NFT::mutate_nft(ALICE_NFT_ID, |x| -> DispatchResult {
				let nft = x.as_mut().unwrap();
				nft.state.is_listed = true;
				Ok(())
			})
			.unwrap();

			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::CannotSetTransmissionForListedNFTs);
		})
	}

	#[test]
	fn cannot_set_transmission_for_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			NFT::mutate_nft(ALICE_NFT_ID, |x| -> DispatchResult {
				let nft = x.as_mut().unwrap();
				nft.state.is_delegated = true;
				Ok(())
			})
			.unwrap();

			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::CannotSetTransmissionForDelegatedNFTs);
		})
	}

	#[test]
	fn cannot_set_transmission_for_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			NFT::mutate_nft(ALICE_NFT_ID, |x| -> DispatchResult {
				let nft = x.as_mut().unwrap();
				nft.state.is_rented = true;
				Ok(())
			})
			.unwrap();

			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::CannotSetTransmissionForRentedNFTs);
		})
	}

	#[test]
	fn cannot_set_transmission_for_not_created_soulbound_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			NFT::mutate_nft(BOB_NFT_ID, |x| -> DispatchResult {
				let nft = x.as_mut().unwrap();
				nft.owner = ALICE;
				nft.state.is_soulbound = true;
				Ok(())
			})
			.unwrap();

			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				BOB_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::CannotSetTransmissionForNotCreatedSoulboundNFTs);
		})
	}

	#[test]
	fn cannot_set_transmission_for_syncing_secret_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			NFT::mutate_nft(ALICE_NFT_ID, |x| -> DispatchResult {
				let nft = x.as_mut().unwrap();
				nft.state.is_syncing_secret = true;
				Ok(())
			})
			.unwrap();

			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::CannotSetTransmissionForSyncingSecretNFTs);
		})
	}

	#[test]
	fn cannot_set_transmission_for_syncing_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			NFT::mutate_nft(ALICE_NFT_ID, |x| -> DispatchResult {
				let nft = x.as_mut().unwrap();
				nft.state.is_syncing_capsule = true;
				Ok(())
			})
			.unwrap();

			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::CannotSetTransmissionForSyncingCapsules);
		})
	}

	#[test]
	fn cannot_set_transmission_for_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			NFT::mutate_nft(ALICE_NFT_ID, |x| -> DispatchResult {
				let nft = x.as_mut().unwrap();
				nft.state.is_transmission = true;
				Ok(())
			})
			.unwrap();

			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::CannotSetTransmissionForNFTsInTransmission);
		})
	}

	#[test]
	fn cannot_set_transmission_in_the_past() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			run_to_block(20);

			let protocol = TransmissionProtocol::AtBlockWithReset(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::CannotSetTransmissionInThePast);
		})
	}

	#[test]
	fn transmission_is_in_too_much_time() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			run_to_block(10);
			let max_block = MaxBlockDuration::get() + 1;
			let protocol = TransmissionProtocol::AtBlockWithReset((max_block + 10).into());
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::TransmissionIsInTooMuchTime);
		})
	}

	#[test]
	fn threshold_too_low() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let protocol = TransmissionProtocol::OnConsent {
				consent_list: BoundedVec::try_from(vec![ALICE]).unwrap(),
				threshold: 0,
			};
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::ThresholdTooLow);
		})
	}

	#[test]
	fn threshold_too_high() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let threshold = MaxConsentListSize::get() + 1;

			let protocol = TransmissionProtocol::OnConsent {
				consent_list: BoundedVec::try_from(vec![ALICE]).unwrap(),
				threshold: threshold.try_into().unwrap(),
			};
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::ThresholdTooHigh);
		})
	}

	#[test]
	fn invalid_consent_list() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let threshold = MaxConsentListSize::get();

			let protocol = TransmissionProtocol::OnConsent {
				consent_list: BoundedVec::try_from(vec![ALICE]).unwrap(),
				threshold: threshold.try_into().unwrap(),
			};
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::InvalidConsentList);
		})
	}

	#[test]
	fn simultaneous_transmission_limit_reached() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let max_queue_number = SimultaneousTransmissionLimit::get();
			TransmissionProtocols::fill_queue(max_queue_number, BOB_NFT_ID, 100).unwrap();
			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, Error::<Test>::SimultaneousTransmissionLimitReached);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			Balances::set_balance(root(), ALICE, 0, 0).unwrap();

			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;
			let err = TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			);
			assert_noop!(err, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

mod remove_transmission_protocol {
	use super::*;

	#[test]
	fn remove_transmission_protocol_at_block() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::Anytime;

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			TransmissionProtocols::remove_transmission_protocol(alice, ALICE_NFT_ID).unwrap();

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			assert!(!nft.state.is_transmission);
			assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_none());
			assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), None);
			assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);

			// Events checks.
			let event = TransmissionProtocolEvent::ProtocolRemoved { nft_id: ALICE_NFT_ID };
			let event = RuntimeEvent::TransmissionProtocols(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn remove_transmission_protocol_at_block_with_reset() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlockWithReset(10);
			let cancellation = CancellationPeriod::UntilBlock(8);

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			TransmissionProtocols::remove_transmission_protocol(alice, ALICE_NFT_ID).unwrap();

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			assert!(!nft.state.is_transmission);
			assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_none());
			assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), None);
			assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);

			// Events checks.
			let event = TransmissionProtocolEvent::ProtocolRemoved { nft_id: ALICE_NFT_ID };
			let event = RuntimeEvent::TransmissionProtocols(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn remove_transmission_protocol_on_consent() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);
			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE, DAVE]).unwrap();
			let protocol = TransmissionProtocol::OnConsent { consent_list, threshold: 2 };
			let cancellation = CancellationPeriod::UntilBlock(8);

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			TransmissionProtocols::add_consent(bob, ALICE_NFT_ID).unwrap();

			TransmissionProtocols::remove_transmission_protocol(alice, ALICE_NFT_ID).unwrap();

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			assert!(!nft.state.is_transmission);
			assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_none());
			assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), None);
			assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);

			// Events checks.
			let event = TransmissionProtocolEvent::ProtocolRemoved { nft_id: ALICE_NFT_ID };
			let event = RuntimeEvent::TransmissionProtocols(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn remove_transmission_protocol_on_consent_at_block() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);
			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE, DAVE]).unwrap();
			let protocol =
				TransmissionProtocol::OnConsentAtBlock { consent_list, threshold: 2, block: 10 };
			let cancellation = CancellationPeriod::UntilBlock(8);

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			TransmissionProtocols::add_consent(bob, ALICE_NFT_ID).unwrap();

			TransmissionProtocols::remove_transmission_protocol(alice, ALICE_NFT_ID).unwrap();

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			assert!(!nft.state.is_transmission);
			assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_none());
			assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), None);
			assert_eq!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID), None);

			// Events checks.
			let event = TransmissionProtocolEvent::ProtocolRemoved { nft_id: ALICE_NFT_ID };
			let event = RuntimeEvent::TransmissionProtocols(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let err = TransmissionProtocols::remove_transmission_protocol(alice, INVALID_NFT_ID);
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let err = TransmissionProtocols::remove_transmission_protocol(alice, BOB_NFT_ID);
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn nft_not_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let err = TransmissionProtocols::remove_transmission_protocol(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::NFTIsNotInTransmission);
		})
	}

	#[test]
	fn transmission_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			NFT::mutate_nft(ALICE_NFT_ID, |x| -> DispatchResult {
				let nft = x.as_mut().unwrap();
				nft.state.is_transmission = true;
				Ok(())
			})
			.unwrap();

			let err = TransmissionProtocols::remove_transmission_protocol(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::TransmissionNotFound);
		})
	}

	#[test]
	fn protocol_not_cancellable() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::None;

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			let err = TransmissionProtocols::remove_transmission_protocol(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::ProtocolIsNotCancellable);
		})
	}

	#[test]
	fn protocol_not_cancellable_until_block_passed() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::UntilBlock(8);
			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();
			run_to_block(9);
			let err = TransmissionProtocols::remove_transmission_protocol(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::ProtocolIsNotCancellable);
		})
	}
}

mod reset_timer {
	use super::*;

	#[test]
	fn reset_timer() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlockWithReset(10);
			let cancellation = CancellationPeriod::Anytime;

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			TransmissionProtocols::reset_timer(alice, ALICE_NFT_ID, 20).unwrap();

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			assert!(nft.state.is_transmission);
			assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_some());
			assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), Some(20));

			// Events checks.
			let event = TransmissionProtocolEvent::TimerReset {
				nft_id: ALICE_NFT_ID,
				new_block_number: 20,
			};
			let event = RuntimeEvent::TransmissionProtocols(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let err = TransmissionProtocols::reset_timer(alice, INVALID_NFT_ID, 20);
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let err = TransmissionProtocols::reset_timer(alice, BOB_NFT_ID, 20);
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn nfts_not_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let err = TransmissionProtocols::reset_timer(alice, ALICE_NFT_ID, 20);
			assert_noop!(err, Error::<Test>::NFTIsNotInTransmission);
		})
	}

	#[test]
	fn transmission_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			NFT::mutate_nft(ALICE_NFT_ID, |x| -> DispatchResult {
				let nft = x.as_mut().unwrap();
				nft.state.is_transmission = true;
				Ok(())
			})
			.unwrap();

			let err = TransmissionProtocols::reset_timer(alice, ALICE_NFT_ID, 20);
			assert_noop!(err, Error::<Test>::TransmissionNotFound);
		})
	}

	#[test]
	fn protocol_timer_cannot_be_reset() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::Anytime;
			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();
			let err = TransmissionProtocols::reset_timer(alice, ALICE_NFT_ID, 20);
			assert_noop!(err, Error::<Test>::ProtocolTimerCannotBeReset);
		})
	}

	#[test]
	fn cannot_set_transmission_in_the_past() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlockWithReset(10);
			let cancellation = CancellationPeriod::Anytime;
			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();
			run_to_block(5);
			let err = TransmissionProtocols::reset_timer(alice, ALICE_NFT_ID, 5);
			assert_noop!(err, Error::<Test>::CannotSetTransmissionInThePast);
		})
	}

	#[test]
	fn transmission_is_in_too_much_time() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			run_to_block(10);
			let max_block = MaxBlockDuration::get() + 1;
			let protocol = TransmissionProtocol::AtBlockWithReset(20);
			let cancellation = CancellationPeriod::Anytime;
			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();
			run_to_block(5);
			let err =
				TransmissionProtocols::reset_timer(alice, ALICE_NFT_ID, (20 + max_block).into());
			assert_noop!(err, Error::<Test>::TransmissionIsInTooMuchTime);
		})
	}
}

mod add_consent {
	use super::*;

	#[test]
	fn add_consent() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);
			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE, DAVE]).unwrap();
			let protocol = TransmissionProtocol::OnConsent { consent_list, threshold: 2 };
			let cancellation = CancellationPeriod::Anytime;

			TransmissionProtocols::set_transmission_protocol(
				alice,
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			TransmissionProtocols::add_consent(bob, ALICE_NFT_ID).unwrap();

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			let consent_data = TransmissionProtocols::on_consent_data(ALICE_NFT_ID).unwrap();
			assert!(nft.state.is_transmission);
			assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_some());
			assert_eq!(consent_data.len(), 1);
			assert!(consent_data.contains(&BOB));

			// Events checks.
			let event = TransmissionProtocolEvent::ConsentAdded { nft_id: ALICE_NFT_ID, from: BOB };
			let event = RuntimeEvent::TransmissionProtocols(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn add_consent_reached_threshold_transmitted() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);
			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE, DAVE]).unwrap();
			let protocol = TransmissionProtocol::OnConsent { consent_list, threshold: 2 };
			let cancellation = CancellationPeriod::Anytime;

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

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			assert!(nft.owner == BOB);
			assert!(!nft.state.is_transmission);
			assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_none());
			assert!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID).is_none());

			// Events checks.
			let event1 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::ConsentAdded {
					nft_id: ALICE_NFT_ID,
					from: ALICE,
				});
			let event2 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::ConsentAdded {
					nft_id: ALICE_NFT_ID,
					from: BOB,
				});
			let event3 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::ThresholdReached {
					nft_id: ALICE_NFT_ID,
				});
			let event4 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::Transmitted {
					nft_id: ALICE_NFT_ID,
				});

			System::assert_has_event(event1);
			System::assert_has_event(event2);
			System::assert_has_event(event3);
			System::assert_last_event(event4);
		})
	}

	#[test]
	fn add_consent_reached_threshold_put_in_queue() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);
			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE, DAVE]).unwrap();
			let protocol =
				TransmissionProtocol::OnConsentAtBlock { consent_list, threshold: 2, block: 10 };
			let cancellation = CancellationPeriod::Anytime;

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

			// Final state checks.
			let nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
			assert!(nft.owner == ALICE);
			assert!(nft.state.is_transmission);
			assert!(TransmissionProtocols::transmissions(ALICE_NFT_ID).is_some());
			assert!(TransmissionProtocols::on_consent_data(ALICE_NFT_ID).is_none());
			assert_eq!(TransmissionProtocols::at_block_queue().get(ALICE_NFT_ID), Some(10));

			// Events checks.
			let event1 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::ConsentAdded {
					nft_id: ALICE_NFT_ID,
					from: ALICE,
				});
			let event2 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::ConsentAdded {
					nft_id: ALICE_NFT_ID,
					from: BOB,
				});
			let event3 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::ThresholdReached {
					nft_id: ALICE_NFT_ID,
				});
			System::assert_has_event(event1);
			System::assert_has_event(event2);
			System::assert_last_event(event3);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let err = TransmissionProtocols::add_consent(alice, INVALID_NFT_ID);
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn nft_is_not_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let err = TransmissionProtocols::add_consent(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::NFTIsNotInTransmission);
		})
	}

	#[test]
	fn transmission_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			NFT::mutate_nft(ALICE_NFT_ID, |x| -> DispatchResult {
				let nft = x.as_mut().unwrap();
				nft.state.is_transmission = true;
				Ok(())
			})
			.unwrap();

			let err = TransmissionProtocols::add_consent(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::TransmissionNotFound);
		})
	}

	#[test]
	fn protocol_does_not_accept_consent() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let protocol = TransmissionProtocol::AtBlock(10);
			let cancellation = CancellationPeriod::Anytime;

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			let err = TransmissionProtocols::add_consent(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::ProtocolDoesNotAcceptConsent);
		})
	}

	#[test]
	fn consent_already_reached_threshold() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);
			let charlie: mock::RuntimeOrigin = origin(CHARLIE);
			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE, DAVE]).unwrap();
			let protocol =
				TransmissionProtocol::OnConsentAtBlock { consent_list, threshold: 2, block: 10 };
			let cancellation = CancellationPeriod::Anytime;

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

			let err = TransmissionProtocols::add_consent(charlie, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::ConsentAlreadyReachedThreshold);
		})
	}

	#[test]
	fn consent_not_allowed() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let dave: mock::RuntimeOrigin = origin(DAVE);
			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE]).unwrap();
			let protocol =
				TransmissionProtocol::OnConsentAtBlock { consent_list, threshold: 2, block: 10 };
			let cancellation = CancellationPeriod::Anytime;

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			let err = TransmissionProtocols::add_consent(dave, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::ConsentNotAllowed);
		})
	}

	#[test]
	fn already_added_consent() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE]).unwrap();
			let protocol =
				TransmissionProtocol::OnConsentAtBlock { consent_list, threshold: 2, block: 10 };
			let cancellation = CancellationPeriod::Anytime;

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			TransmissionProtocols::add_consent(alice.clone(), ALICE_NFT_ID).unwrap();

			let err = TransmissionProtocols::add_consent(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::AlreadyAddedConsent);
		})
	}

	#[test]
	fn consent_list_full() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE]).unwrap();
			let protocol =
				TransmissionProtocol::OnConsentAtBlock { consent_list, threshold: 2, block: 10 };
			let cancellation = CancellationPeriod::Anytime;
			let max_account_in_list = MaxConsentListSize::get();

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			TransmissionProtocols::fill_consent_list(max_account_in_list, ALICE_NFT_ID, BOB)
				.unwrap();

			let err = TransmissionProtocols::add_consent(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::ConsentListFull);
		})
	}

	#[test]
	fn simultaneous_transmission_limit_reached() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);
			let consent_list = BoundedVec::try_from(vec![ALICE, BOB, CHARLIE]).unwrap();
			let protocol =
				TransmissionProtocol::OnConsentAtBlock { consent_list, threshold: 2, block: 10 };
			let cancellation = CancellationPeriod::Anytime;
			let max_queue_number = SimultaneousTransmissionLimit::get();

			TransmissionProtocols::set_transmission_protocol(
				alice.clone(),
				ALICE_NFT_ID,
				BOB,
				protocol,
				cancellation,
			)
			.unwrap();

			TransmissionProtocols::fill_queue(max_queue_number, BOB_NFT_ID, 100).unwrap();

			TransmissionProtocols::add_consent(bob, ALICE_NFT_ID).unwrap();

			let err = TransmissionProtocols::add_consent(alice, ALICE_NFT_ID);
			assert_noop!(err, Error::<Test>::SimultaneousTransmissionLimitReached);
		})
	}
}

mod set_protocol_fee {
	use super::*;

	#[test]
	fn set_protocol_fee() {
		ExtBuilder::new_build(vec![]).execute_with(|| {
			let new_fee: u64 = 20;
			// Set new protocol fees.
			TransmissionProtocols::set_protocol_fee(
				root(),
				TransmissionProtocolKind::AtBlock,
				new_fee,
			)
			.unwrap();
			TransmissionProtocols::set_protocol_fee(
				root(),
				TransmissionProtocolKind::AtBlockWithReset,
				2 * new_fee,
			)
			.unwrap();
			TransmissionProtocols::set_protocol_fee(
				root(),
				TransmissionProtocolKind::OnConsent,
				3 * new_fee,
			)
			.unwrap();
			TransmissionProtocols::set_protocol_fee(
				root(),
				TransmissionProtocolKind::OnConsentAtBlock,
				4 * new_fee,
			)
			.unwrap();

			// Final state checks.
			assert_eq!(TransmissionProtocols::at_block_fee(), new_fee);
			assert_eq!(TransmissionProtocols::at_block_with_reset_fee(), 2 * new_fee);
			assert_eq!(TransmissionProtocols::on_consent_fee(), 3 * new_fee);
			assert_eq!(TransmissionProtocols::on_consent_at_block_fee(), 4 * new_fee);

			// Events checks.
			let event1 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::ProtocolFeeSet {
					protocol: TransmissionProtocolKind::AtBlock,
					fee: new_fee,
				});
			let event2 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::ProtocolFeeSet {
					protocol: TransmissionProtocolKind::AtBlockWithReset,
					fee: 2 * new_fee,
				});
			let event3 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::ProtocolFeeSet {
					protocol: TransmissionProtocolKind::OnConsent,
					fee: 3 * new_fee,
				});
			let event4 =
				RuntimeEvent::TransmissionProtocols(TransmissionProtocolEvent::ProtocolFeeSet {
					protocol: TransmissionProtocolKind::OnConsentAtBlock,
					fee: 4 * new_fee,
				});
			System::assert_has_event(event1);
			System::assert_has_event(event2);
			System::assert_has_event(event3);
			System::assert_has_event(event4);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice = origin(ALICE);
			// Try to change protocol fee as not root.
			let err = TransmissionProtocols::set_protocol_fee(
				alice,
				TransmissionProtocolKind::OnConsentAtBlock,
				20,
			);
			// Should fail because Alice is not the root.
			assert_noop!(err, BadOrigin);
		})
	}
}
