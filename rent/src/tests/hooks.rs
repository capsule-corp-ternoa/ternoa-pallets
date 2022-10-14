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
use ternoa_common::traits::NFTExt;

use crate::{
	tests::{extrinsics::*, mock},
	Event as RentEvent,
};

fn origin(account: u64) -> mock::RuntimeOrigin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::RuntimeOrigin {
	RawOrigin::Root.into()
}

#[test]
fn end_contract_fixed() {
	ExtBuilder::new_build(None).execute_with(|| {
		prepare_tests();
		let bob: mock::RuntimeOrigin = origin(BOB);

		Rent::rent(bob, FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();

		run_to_block(BLOCK_DURATION + 1);

		// State check.
		let contract = Rent::contracts(FIXED_AUTO_REV_NFT_TOKENS_TOKENS);
		let rent_fee_nft = NFT::get_nft(BOB_NFT_ID_0).unwrap();
		assert!(contract.is_none());
		assert_eq!(rent_fee_nft.owner, ALICE);

		// Event check.
		let event =
			RentEvent::ContractEnded { nft_id: FIXED_AUTO_REV_NFT_TOKENS_TOKENS, revoked_by: None };
		let event = RuntimeEvent::Rent(event);
		System::assert_last_event(event);
	})
}

#[test]
fn end_contract_subscription() {
	ExtBuilder::new_build(None).execute_with(|| {
		prepare_tests();
		let alice: mock::RuntimeOrigin = origin(ALICE);
		let bob: mock::RuntimeOrigin = origin(BOB);
		let alice_balance = Balances::free_balance(ALICE);
		let bob_balance = Balances::free_balance(BOB);

		Rent::make_rent_offer(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();
		Rent::accept_rent_offer(alice, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK, BOB).unwrap();

		run_to_block(BLOCK_MAX_DURATION + 1);

		// State check.
		let contract = Rent::contracts(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK);
		assert_eq!(contract, None);
		assert_eq!(
			Balances::free_balance(ALICE),
			alice_balance + (BLOCK_MAX_DURATION * TOKENS / BLOCK_DURATION) + LESS_TOKENS
		);
		assert_eq!(
			Balances::free_balance(BOB),
			bob_balance - (BLOCK_MAX_DURATION * TOKENS / BLOCK_DURATION)
		);

		// Event check.
		let event = RentEvent::ContractEnded {
			nft_id: SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
			revoked_by: None,
		};
		let event = RuntimeEvent::Rent(event);
		System::assert_last_event(event);
	})
}

#[test]
fn end_contract_renter() {
	ExtBuilder::new_build(None).execute_with(|| {
		prepare_tests();
		let alice: mock::RuntimeOrigin = origin(ALICE);
		let bob: mock::RuntimeOrigin = origin(BOB);
		let alice_balance = Balances::free_balance(ALICE);
		let bob_balance = Balances::free_balance(BOB);

		Rent::make_rent_offer(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();
		Rent::accept_rent_offer(alice.clone(), SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK, BOB)
			.unwrap();
		Rent::change_subscription_terms(
			alice,
			SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
			TOKENS,
			BLOCK_DURATION / 2,
			Some(BLOCK_MAX_DURATION / 2),
			true,
		)
		.unwrap();

		run_to_block(BLOCK_DURATION + 1);

		// State check.
		let contract = Rent::contracts(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK);
		assert!(contract.is_none());
		assert_eq!(Balances::free_balance(ALICE), alice_balance + TOKENS + LESS_TOKENS);
		assert_eq!(Balances::free_balance(BOB), bob_balance - TOKENS);

		// Event check.
		let event = RentEvent::ContractEnded {
			nft_id: SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
			revoked_by: None,
		};
		let event = RuntimeEvent::Rent(event);
		System::assert_last_event(event);
	})
}

#[test]
fn end_contract_rentee() {
	ExtBuilder::new_build(None).execute_with(|| {
		prepare_tests();
		let alice: mock::RuntimeOrigin = origin(ALICE);
		let bob: mock::RuntimeOrigin = origin(BOB);
		let alice_balance = Balances::free_balance(ALICE);

		Rent::make_rent_offer(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();
		Rent::accept_rent_offer(alice, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK, BOB).unwrap();
		Balances::set_balance(root(), BOB, 0, 0).unwrap();

		run_to_block(BLOCK_DURATION + 1);

		// State check.
		let contract = Rent::contracts(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK);
		assert!(contract.is_none());
		assert_eq!(Balances::free_balance(ALICE), alice_balance + TOKENS + LESS_TOKENS);
		assert_eq!(Balances::free_balance(BOB), LESS_TOKENS);

		// Event check.
		let event = RentEvent::ContractEnded {
			nft_id: SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
			revoked_by: None,
		};
		let event = RuntimeEvent::Rent(event);
		System::assert_last_event(event);
	})
}

#[test]
fn renew_contract() {
	ExtBuilder::new_build(None).execute_with(|| {
		prepare_tests();
		let alice: mock::RuntimeOrigin = origin(ALICE);
		let bob: mock::RuntimeOrigin = origin(BOB);

		Rent::make_rent_offer(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();
		Rent::accept_rent_offer(alice, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK, BOB).unwrap();

		// Check subscription queue
		assert_eq!(
			Rent::queues()
				.subscription_queue
				.get(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK),
			Some(BLOCK_DURATION + 1)
		);

		run_to_block(BLOCK_DURATION + 1);

		// State check.
		let contract = Rent::contracts(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK);
		assert!(contract.is_some());
		assert_eq!(
			Rent::queues()
				.subscription_queue
				.get(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK),
			Some(2 * BLOCK_DURATION + 1)
		);

		// Event check.
		let event = RentEvent::ContractSubscriptionPeriodStarted {
			nft_id: SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
		};
		let event = RuntimeEvent::Rent(event);
		System::assert_last_event(event);
	})
}

#[test]
fn remove_expired_contract() {
	ExtBuilder::new_build(None).execute_with(|| {
		prepare_tests();
		run_to_block((MaximumContractAvailabilityLimit::get() + 1).into());

		// State check.
		assert!(Rent::contracts(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).is_none());
		assert_eq!(Rent::queues().available_queue.get(FIXED_AUTO_REV_NFT_TOKENS_TOKENS), None);
		assert!(Rent::contracts(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).is_none());
		assert_eq!(Rent::queues().available_queue.get(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK), None);
		assert!(Rent::contracts(FIXED_MANU_REV_NFT_NFT_NFT).is_none());
		assert_eq!(Rent::queues().available_queue.get(FIXED_MANU_REV_NFT_NFT_NFT), None);
		assert!(Rent::contracts(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).is_none());
		assert_eq!(
			Rent::queues().available_queue.get(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK),
			None
		);
		assert!(Rent::contracts(SUBSC_AUTO_NOREV_NOT_CHANGEABLE_TOK_NONE_FIXTOK).is_none());
		assert_eq!(
			Rent::queues()
				.available_queue
				.get(SUBSC_AUTO_NOREV_NOT_CHANGEABLE_TOK_NONE_FIXTOK),
			None
		);
		assert!(Rent::contracts(FIXED_AUTO_REV_NFT_NFT_NFT).is_none());
		assert_eq!(Rent::queues().available_queue.get(FIXED_AUTO_REV_NFT_NFT_NFT), None);

		// Event check.
		let event_0 =
			RuntimeEvent::Rent(RentEvent::ContractExpired { nft_id: FIXED_AUTO_REV_NFT_TOKENS_TOKENS });
		let event_1 =
			RuntimeEvent::Rent(RentEvent::ContractExpired { nft_id: FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK });
		let event_2 =
			RuntimeEvent::Rent(RentEvent::ContractExpired { nft_id: FIXED_MANU_REV_NFT_NFT_NFT });
		let event_3 = RuntimeEvent::Rent(RentEvent::ContractExpired {
			nft_id: SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
		});
		let event_4 = RuntimeEvent::Rent(RentEvent::ContractExpired {
			nft_id: SUBSC_AUTO_NOREV_NOT_CHANGEABLE_TOK_NONE_FIXTOK,
		});
		let event_5 =
			RuntimeEvent::Rent(RentEvent::ContractExpired { nft_id: FIXED_AUTO_REV_NFT_NFT_NFT });
		System::assert_has_event(event_0);
		System::assert_has_event(event_1);
		System::assert_has_event(event_2);
		System::assert_has_event(event_3);
		System::assert_has_event(event_4);
		System::assert_has_event(event_5);
	})
}
