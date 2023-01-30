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
use frame_support::{assert_noop, BoundedVec};
use frame_system::RawOrigin;
use primitives::nfts::{NFTId, NFTState};
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::NFTExt;

use crate::{
	tests::mock, AcceptanceType, CancellationFee, Duration, DurationInput, Error,
	Event as RentEvent, RentContractData, RentFee, SubscriptionInput,
};

pub const BLOCK_DURATION: u64 = 10;
pub const BLOCK_MAX_DURATION: u64 = 100;
pub const TOKENS: Balance = 100;
pub const LESS_TOKENS: Balance = 10;

pub const ALICE_NFT_ID_0: NFTId = 0;
pub const ALICE_NFT_ID_1: NFTId = 1;
pub const ALICE_NFT_ID_2: NFTId = 2;
pub const ALICE_NFT_ID_3: NFTId = 3;
pub const ALICE_NFT_ID_4: NFTId = 4;
pub const ALICE_NFT_ID_5: NFTId = 5;
pub const ALICE_NFT_ID_6: NFTId = 6;
pub const ALICE_NFT_ID_7: NFTId = 7;
pub const ALICE_NFT_ID_8: NFTId = 8;
pub const ALICE_NFT_ID_9: NFTId = 9;
pub const BOB_NFT_ID_0: NFTId = 10;
pub const BOB_NFT_ID_1: NFTId = 11;
pub const BOB_NFT_ID_2: NFTId = 12;
pub const INVALID_NFT: NFTId = 99;
pub const PERCENT_0: Permill = Permill::from_parts(0);

pub const FIXED_AUTO_REV_NFT_TOKENS_TOKENS: NFTId = 0;
pub const FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK: NFTId = 1;
pub const FIXED_MANU_REV_NFT_NFT_NFT: NFTId = 2;
pub const SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK: NFTId = 3;
pub const SUBSC_AUTO_NOREV_NOT_CHANGEABLE_TOK_NONE_FIXTOK: NFTId = 4;
pub const FIXED_AUTO_REV_NFT_NFT_NFT: NFTId = 8;

fn origin(account: u64) -> mock::RuntimeOrigin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::RuntimeOrigin {
	RawOrigin::Root.into()
}

pub fn prepare_tests() {
	let alice: mock::RuntimeOrigin = origin(ALICE);
	let bob: mock::RuntimeOrigin = origin(BOB);

	//Create NFTs.
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(bob.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(bob.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(bob, BoundedVec::default(), PERCENT_0, None, false).unwrap();

	//Check existence
	assert!(NFT::nfts(ALICE_NFT_ID_0).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_1).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_2).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_3).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_4).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_5).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_6).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_7).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_8).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_9).is_some());
	assert!(NFT::nfts(BOB_NFT_ID_0).is_some());
	assert!(NFT::nfts(BOB_NFT_ID_1).is_some());
	assert!(NFT::nfts(BOB_NFT_ID_2).is_some());

	//Create contracts.
	Rent::create_contract(
		alice.clone(),
		FIXED_AUTO_REV_NFT_TOKENS_TOKENS,
		DurationInput::Fixed(BLOCK_DURATION),
		AcceptanceType::AutoAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		true,
		RentFee::NFT(BOB_NFT_ID_0),
		CancellationFee::FixedTokens(LESS_TOKENS),
		CancellationFee::FixedTokens(LESS_TOKENS),
	)
	.unwrap();
	Rent::create_contract(
		alice.clone(),
		FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK,
		DurationInput::Fixed(BLOCK_DURATION),
		AcceptanceType::AutoAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		true,
		RentFee::Tokens(TOKENS),
		CancellationFee::FlexibleTokens(LESS_TOKENS),
		CancellationFee::FlexibleTokens(LESS_TOKENS),
	)
	.unwrap();
	Rent::create_contract(
		alice.clone(),
		FIXED_MANU_REV_NFT_NFT_NFT,
		DurationInput::Fixed(BLOCK_DURATION),
		AcceptanceType::ManualAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		true,
		RentFee::NFT(BOB_NFT_ID_1),
		CancellationFee::NFT(ALICE_NFT_ID_5),
		CancellationFee::NFT(BOB_NFT_ID_0),
	)
	.unwrap();
	Rent::create_contract(
		alice.clone(),
		SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
		DurationInput::Subscription(SubscriptionInput {
			period_length: BLOCK_DURATION,
			max_duration: Some(BLOCK_MAX_DURATION),
			is_changeable: true,
		}),
		AcceptanceType::ManualAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		true,
		RentFee::Tokens(TOKENS),
		CancellationFee::FixedTokens(LESS_TOKENS),
		CancellationFee::FixedTokens(LESS_TOKENS),
	)
	.unwrap();
	Rent::create_contract(
		alice.clone(),
		SUBSC_AUTO_NOREV_NOT_CHANGEABLE_TOK_NONE_FIXTOK,
		DurationInput::Subscription(SubscriptionInput {
			period_length: BLOCK_DURATION,
			max_duration: None,
			is_changeable: false,
		}),
		AcceptanceType::AutoAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		false,
		RentFee::Tokens(TOKENS),
		CancellationFee::None,
		CancellationFee::FixedTokens(LESS_TOKENS),
	)
	.unwrap();
	Rent::create_contract(
		alice.clone(),
		FIXED_AUTO_REV_NFT_NFT_NFT,
		DurationInput::Fixed(BLOCK_DURATION),
		AcceptanceType::AutoAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		true,
		RentFee::NFT(BOB_NFT_ID_1),
		CancellationFee::NFT(ALICE_NFT_ID_9),
		CancellationFee::NFT(BOB_NFT_ID_0),
	)
	.unwrap();

	//Check existence
	assert!(Rent::contracts(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).is_some());
	assert!(Rent::contracts(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).is_some());
	assert!(Rent::contracts(FIXED_MANU_REV_NFT_NFT_NFT).is_some());
	assert!(Rent::contracts(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).is_some());
	assert!(Rent::contracts(SUBSC_AUTO_NOREV_NOT_CHANGEABLE_TOK_NONE_FIXTOK).is_some());
	assert!(Rent::contracts(FIXED_AUTO_REV_NFT_NFT_NFT).is_some());
}

mod create_contract {

	use super::*;

	#[test]
	fn create_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			assert!(true);

			let data: RentContractData<u64, u64, Balance, RentAccountSizeLimit> =
				RentContractData::new(
					None,
					ALICE,
					None,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					true,
					RentFee::Tokens(TOKENS),
					CancellationFee::None,
					CancellationFee::None,
				);

			// Create basic contract.
			Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				data.acceptance_type.clone(),
				data.renter_can_revoke,
				data.rent_fee.clone(),
				data.renter_cancellation_fee.clone(),
				data.rentee_cancellation_fee.clone(),
			)
			.unwrap();

			// State check.
			let contract = Rent::contracts(ALICE_NFT_ID_6).unwrap();
			let nft = NFT::nfts(ALICE_NFT_ID_6).unwrap();
			assert_eq!(contract, data.clone());
			assert!(nft.state.is_rented);

			// Event check.
			let event = RentEvent::ContractCreated {
				nft_id: ALICE_NFT_ID_6,
				renter: ALICE,
				duration: data.duration,
				acceptance_type: data.acceptance_type,
				renter_can_revoke: true,
				rent_fee: data.rent_fee,
				renter_cancellation_fee: data.renter_cancellation_fee,
				rentee_cancellation_fee: data.rentee_cancellation_fee,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn max_simultaneous_contract_reached() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Create contracts until Limit.
			let max_contract = SimultaneousContractLimit::get();
			let current_size = Rent::queues().size();
			let nb_contract_to_create = max_contract - current_size;
			for i in 13..13 + nb_contract_to_create {
				NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false)
					.unwrap();
				Rent::create_contract(
					alice.clone(),
					i,
					DurationInput::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					false,
					RentFee::Tokens(TOKENS),
					CancellationFee::None,
					CancellationFee::None,
				)
				.unwrap();
			}

			// Try to add an other contract.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::MaxSimultaneousContractReached);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract with invalid NFT.
			let err = Rent::create_contract(
				alice,
				INVALID_NFT,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract with unowned NFT.
			let err = Rent::create_contract(
				alice,
				BOB_NFT_ID_0,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn contract_nft_not_in_a_valid_state() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Set to capsule.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, false, true, false);
			NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
			// Try to create a contract with an NFT in invalid state.
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);
			assert_noop!(err, Error::<Test>::ContractNFTNotInAValidState);

			// Set to listed.
			let nft_state =
				NFTState::new(false, true, false, false, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
			// Try to create a contract with an NFT in invalid state.
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);
			assert_noop!(err, Error::<Test>::ContractNFTNotInAValidState);

			// Set to delegated.
			let nft_state =
				NFTState::new(false, false, false, true, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
			// Try to create a contract with an NFT in invalid state.
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);
			assert_noop!(err, Error::<Test>::ContractNFTNotInAValidState);

			// Set to soulbound.
			let nft_state =
				NFTState::new(false, false, false, false, true, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
			// Try to create a contract with an NFT in invalid state.
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);
			assert_noop!(err, Error::<Test>::ContractNFTNotInAValidState);

			// Set to rented.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, true, false, false);
			NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
			// Try to create a contract with an NFT in invalid state.
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);
			assert_noop!(err, Error::<Test>::ContractNFTNotInAValidState);
		})
	}

	#[test]
	fn duration_exceeds_maximum_limit() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract duration above limit.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_MAX_DURATION + 1),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::DurationExceedsMaximumLimit);
		})
	}

	#[test]
	fn duration_invalid() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a fixed contract with 0 duration.
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				DurationInput::Fixed(0),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);
			assert_noop!(err, Error::<Test>::DurationInvalid);

			// Try to create a subscription contract with 0 duration.
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				DurationInput::Subscription(SubscriptionInput {
					period_length: 0,
					max_duration: None,
					is_changeable: false,
				}),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);
			assert_noop!(err, Error::<Test>::DurationInvalid);

			// Try to create a subscription contract with 0 duration.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Subscription(SubscriptionInput {
					period_length: BLOCK_DURATION,
					max_duration: Some(0),
					is_changeable: false,
				}),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::None,
			);
			assert_noop!(err, Error::<Test>::DurationInvalid);
		})
	}

	#[test]
	fn duration_and_rent_fee_mismatch() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract with incompatible duration and rent fee type.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Subscription(SubscriptionInput {
					period_length: BLOCK_DURATION,
					max_duration: None,
					is_changeable: false,
				}),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::NFT(BOB_NFT_ID_0),
				CancellationFee::None,
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::DurationAndRentFeeMismatch);
		})
	}

	#[test]
	fn duration_and_cancellation_fee_mismatch_renter() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract with incompatible duration and cancellation fee type.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Subscription(SubscriptionInput {
					period_length: BLOCK_DURATION,
					max_duration: None,
					is_changeable: false,
				}),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::FlexibleTokens(LESS_TOKENS),
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::DurationAndCancellationFeeMismatch);
		})
	}

	#[test]
	fn duration_and_cancellation_fee_mismatch_rentee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract with incompatible duration and cancellation fee type.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Subscription(SubscriptionInput {
					period_length: BLOCK_DURATION,
					max_duration: None,
					is_changeable: false,
				}),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::FlexibleTokens(LESS_TOKENS),
			);

			assert_noop!(err, Error::<Test>::DurationAndCancellationFeeMismatch);
		})
	}

	#[test]
	fn rent_nft_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract with invalid NFT.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::NFT(INVALID_NFT),
				CancellationFee::None,
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::RentNFTNotFound);
		})
	}

	#[test]
	fn cancellation_nft_not_found_rentee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract with invalid cancellation fee nft.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::NFT(INVALID_NFT),
			);

			assert_noop!(err, Error::<Test>::CancellationNFTNotFound);
		})
	}

	#[test]
	fn amount_too_low_renter_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract without enough funds to cover for the cancellation fee.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::FixedTokens(1),
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::AmountTooLow);
		})
	}

	#[test]
	fn amount_too_low_rentee_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract without enough funds to cover for the cancellation fee.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::None,
				CancellationFee::FixedTokens(1),
			);

			assert_noop!(err, Error::<Test>::AmountTooLow);
		})
	}

	#[test]
	fn not_ennough_funds_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract without enough funds to cover for the cancellation fee.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::FixedTokens(1_000_000),
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::NotEnoughFundsForCancellationFee);
		})
	}

	#[test]
	fn cancellation_nft_not_found_renter() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract with invalid cancellation fee NFT.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::NFT(INVALID_NFT),
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::CancellationNFTNotFound);
		})
	}

	#[test]
	fn caller_does_not_own_cancellation_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a contract with unowned cancellation fee NFT.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::NFT(BOB_NFT_ID_0),
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::CallerDoesNotOwnCancellationNFT);
		})
	}

	#[test]
	fn cancellation_nft_not_in_valid_state() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Set cancellation fee NFT to capsule.
			let nft_state =
				NFTState::new(false, false, false, false, false, false, false, false, true);
			NFT::set_nft_state(ALICE_NFT_ID_7, nft_state).unwrap();

			// Try to create a contract with invalid state cancellation fee NFT.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				false,
				RentFee::Tokens(TOKENS),
				CancellationFee::NFT(ALICE_NFT_ID_7),
				CancellationFee::None,
			);

			assert_noop!(err, Error::<Test>::CancellationNFTNotInValidState);
		})
	}
}

mod cancel_contract {
	use super::*;

	#[test]
	fn cancel_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Cancel contract.
			Rent::cancel_contract(alice, FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).unwrap();
			// State check.
			let nft = NFT::nfts(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).unwrap();
			assert!(Rent::contracts(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).is_none());
			assert!(Rent::queues()
				.available_queue
				.get(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK)
				.is_none());
			assert!(!nft.state.is_rented);
			// Event check.
			let event = RentEvent::ContractCanceled { nft_id: FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK };
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			// Try to cancel contract.
			let err = Rent::cancel_contract(alice, INVALID_NFT);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn not_the_contract_owner() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);
			// Try to cancel contract.
			let err = Rent::cancel_contract(bob, FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK);
			assert_noop!(err, Error::<Test>::NotTheContractOwner);
		})
	}

	#[test]
	fn cannot_cancel_running_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			// Start the contract.
			Rent::rent(bob, FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).unwrap();

			// Try to cancel contract.
			let err = Rent::cancel_contract(alice, FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK);
			assert_noop!(err, Error::<Test>::CannotCancelRunningContract);
		})
	}
}

mod revoke_contract {
	use super::*;

	#[test]
	fn revoke_contract_by_renter_fixed() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::rent(bob, FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();

			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);

			// Revoke.
			Rent::revoke_contract(alice, FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();

			// State check.
			let nft = NFT::nfts(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();
			assert!(Rent::contracts(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).is_none());
			assert!(Rent::queues().available_queue.get(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(Balances::free_balance(ALICE), alice_balance);
			assert_eq!(Balances::free_balance(BOB), bob_balance + 2 * LESS_TOKENS);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: FIXED_AUTO_REV_NFT_TOKENS_TOKENS,
				revoked_by: ALICE,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_rentee_fixed() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::rent(bob.clone(), FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();

			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);

			// Revoke.
			Rent::revoke_contract(bob, FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();

			// State check.
			let nft = NFT::nfts(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();
			assert!(Rent::contracts(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).is_none());
			assert!(Rent::queues().available_queue.get(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(Balances::free_balance(ALICE), alice_balance + 2 * LESS_TOKENS);
			assert_eq!(Balances::free_balance(BOB), bob_balance);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: FIXED_AUTO_REV_NFT_TOKENS_TOKENS,
				revoked_by: BOB,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_renter_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT).unwrap();
			Rent::accept_rent_offer(alice.clone(), FIXED_MANU_REV_NFT_NFT_NFT, BOB).unwrap();

			// Revoke.
			Rent::revoke_contract(alice, FIXED_MANU_REV_NFT_NFT_NFT).unwrap();

			// State check.
			let nft = NFT::nfts(FIXED_MANU_REV_NFT_NFT_NFT).unwrap();
			let alice_cancellation_nft = NFT::nfts(ALICE_NFT_ID_5).unwrap();
			let bob_cancellation_nft = NFT::nfts(BOB_NFT_ID_0).unwrap();
			assert!(Rent::contracts(FIXED_MANU_REV_NFT_NFT_NFT).is_none());
			assert!(Rent::queues().available_queue.get(FIXED_MANU_REV_NFT_NFT_NFT).is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(alice_cancellation_nft.owner, BOB);
			assert_eq!(bob_cancellation_nft.owner, BOB);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: FIXED_MANU_REV_NFT_NFT_NFT,
				revoked_by: ALICE,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_rentee_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob.clone(), FIXED_MANU_REV_NFT_NFT_NFT).unwrap();
			Rent::accept_rent_offer(alice, FIXED_MANU_REV_NFT_NFT_NFT, BOB).unwrap();

			// Revoke.
			Rent::revoke_contract(bob, FIXED_MANU_REV_NFT_NFT_NFT).unwrap();

			// State check.
			let nft = NFT::nfts(FIXED_MANU_REV_NFT_NFT_NFT).unwrap();
			let alice_cancellation_nft = NFT::nfts(ALICE_NFT_ID_5).unwrap();
			let bob_cancellation_nft = NFT::nfts(BOB_NFT_ID_0).unwrap();
			assert!(Rent::contracts(FIXED_MANU_REV_NFT_NFT_NFT).is_none());
			assert!(Rent::queues().available_queue.get(FIXED_MANU_REV_NFT_NFT_NFT).is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(alice_cancellation_nft.owner, ALICE);
			assert_eq!(bob_cancellation_nft.owner, ALICE);
			// Event check.
			let event =
				RentEvent::ContractRevoked { nft_id: FIXED_MANU_REV_NFT_NFT_NFT, revoked_by: BOB };
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_renter_flexible() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::rent(bob, FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).unwrap();

			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);
			let now = System::block_number();

			// Change current block.
			run_to_block(now + 2);

			// Revoke.
			Rent::revoke_contract(alice, FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).unwrap();

			// State check.
			let nft = NFT::nfts(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).unwrap();
			assert!(Rent::contracts(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).is_none());
			assert!(Rent::queues()
				.available_queue
				.get(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK)
				.is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(Balances::free_balance(ALICE), alice_balance + 2);
			assert_eq!(Balances::free_balance(BOB), bob_balance + LESS_TOKENS + 8);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK,
				revoked_by: ALICE,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_rentee_flexible() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::rent(bob.clone(), FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).unwrap();

			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);
			let now = System::block_number();

			// Change current block.
			run_to_block(now + 2);

			// Revoke.
			Rent::revoke_contract(bob, FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).unwrap();

			// State check.
			let nft = NFT::nfts(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).unwrap();
			assert!(Rent::contracts(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK).is_none());
			assert!(Rent::queues()
				.available_queue
				.get(FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK)
				.is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(Balances::free_balance(ALICE), alice_balance + LESS_TOKENS + 8);
			assert_eq!(Balances::free_balance(BOB), bob_balance + 2);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK,
				revoked_by: BOB,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to revoke with invalid contract.
			let err = Rent::revoke_contract(alice, INVALID_NFT);

			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn cannot_revoke_non_runing_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			// Try to revoke with unowned contract.
			let err = Rent::revoke_contract(bob, FIXED_AUTO_REV_NFT_TOKENS_TOKENS);

			assert_noop!(err, Error::<Test>::CannotRevokeNonRunningContract);
		})
	}

	#[test]
	fn not_a_contract_participant() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);
			let charlie: mock::RuntimeOrigin = origin(CHARLIE);

			Rent::rent(bob, FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();

			// Try to revoke with unowned contract.
			let err = Rent::revoke_contract(charlie, FIXED_AUTO_REV_NFT_TOKENS_TOKENS);

			assert_noop!(err, Error::<Test>::NotAContractParticipant);
		})
	}

	#[test]
	fn contract_cannot_be_canceled_by_renter() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::rent(bob.clone(), SUBSC_AUTO_NOREV_NOT_CHANGEABLE_TOK_NONE_FIXTOK).unwrap();

			// Try to revoke with unowned contract.
			let err = Rent::revoke_contract(alice, SUBSC_AUTO_NOREV_NOT_CHANGEABLE_TOK_NONE_FIXTOK);

			assert_noop!(err, Error::<Test>::ContractCannotBeCanceledByRenter);
		})
	}
}

mod rent {
	use super::*;

	#[test]
	fn rent() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			// Rent contract.
			Rent::rent(bob, FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();

			// State check.
			let nft = NFT::nfts(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();
			let contract = Rent::contracts(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).unwrap();
			assert_eq!(contract.rentee, Some(BOB));
			assert!(contract.start_block.is_some());
			assert!(Rent::queues().available_queue.get(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).is_none());
			assert!(Rent::queues().fixed_queue.get(FIXED_AUTO_REV_NFT_TOKENS_TOKENS).is_some());
			assert!(nft.state.is_rented);

			// Event check.
			let event = RentEvent::ContractStarted {
				nft_id: FIXED_AUTO_REV_NFT_TOKENS_TOKENS,
				rentee: BOB,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to rent invalid contract.
			let err = Rent::rent(alice, INVALID_NFT);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn cannot_rent_own_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to rent owned contract.
			let err = Rent::rent(alice, FIXED_AUTO_REV_NFT_TOKENS_TOKENS);
			assert_noop!(err, Error::<Test>::CannotRentOwnContract);
		})
	}

	#[test]
	fn contract_does_not_support_automatic_rent() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			// Try to rent owned contract.
			let err = Rent::rent(bob, FIXED_MANU_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::ContractDoesNotSupportAutomaticRent);
		})
	}

	#[test]
	fn not_whitelisted() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let charlie: mock::RuntimeOrigin = origin(CHARLIE);

			// Try to rent without being authorized auto acceptance.
			let err = Rent::rent(charlie, SUBSC_AUTO_NOREV_NOT_CHANGEABLE_TOK_NONE_FIXTOK);
			assert_noop!(err, Error::<Test>::NotWhitelisted);
		})
	}

	#[test]
	fn not_enough_funds_for_rent_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			Balances::set_balance(root(), BOB, TOKENS - 1, 0).unwrap();

			// Try to rent without enough tokens for rent fee.
			let err = Rent::rent(bob, SUBSC_AUTO_NOREV_NOT_CHANGEABLE_TOK_NONE_FIXTOK);
			assert_noop!(err, Error::<Test>::NotEnoughFundsForRentFee);
		})
	}

	#[test]
	fn not_enough_funds_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			Balances::set_balance(root(), BOB, TOKENS + LESS_TOKENS - 1, 0).unwrap();

			// Try to rent without enough balance for cancellation fee.
			let err = Rent::rent(bob, FIXED_AUTO_REV_TOK_FLEXTOK_FLEXTOK);
			assert_noop!(err, Error::<Test>::NotEnoughFundsForCancellationFee);
		})
	}

	#[test]
	fn rentee_does_not_own_the_rent_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			// Change ownership of rent fee NFT.
			let mut nft = NFT::get_nft(BOB_NFT_ID_0).unwrap();
			nft.owner = ALICE;
			NFT::set_nft(BOB_NFT_ID_0, nft).unwrap();

			// Try to rent without enough tokens for rent fee.
			let err = Rent::rent(bob, FIXED_AUTO_REV_NFT_TOKENS_TOKENS);
			assert_noop!(err, Error::<Test>::RenteeDoesNotOwnTheRentNFT);
		})
	}

	#[test]
	fn rentee_does_not_own_the_cancellation_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			// Change ownership of cancellation NFT.
			let mut nft = NFT::get_nft(BOB_NFT_ID_0).unwrap();
			nft.owner = ALICE;
			NFT::set_nft(BOB_NFT_ID_0, nft).unwrap();

			// Try to rent without nft for cancellation fee.
			let err = Rent::rent(bob, FIXED_AUTO_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::RenteeDoesNotOwnTheCancellationNFT);
		})
	}
}

mod make_rent_offer {
	use super::*;

	#[test]
	fn make_rent_offer() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();

			// State check.
			assert!(Rent::offers(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK)
				.unwrap()
				.contains(&BOB));
			assert!(Rent::queues()
				.available_queue
				.get(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK)
				.is_some());

			// Event check.
			let event = RentEvent::ContractOfferCreated {
				nft_id: SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				rentee: BOB,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			let err = Rent::make_rent_offer(bob, INVALID_NFT);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn cannot_rent_own_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			let err = Rent::make_rent_offer(alice, FIXED_MANU_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::CannotRentOwnContract);
		})
	}

	#[test]
	fn contract_does_not_support_offers() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			let err = Rent::make_rent_offer(bob, FIXED_AUTO_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::ContractDoesNotSupportOffers);
		})
	}
	// Balances::set_balance(root(), BOB, 0, 0).unwrap();
	#[test]
	fn not_whitelisted() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let charlie: mock::RuntimeOrigin = origin(CHARLIE);

			let err = Rent::make_rent_offer(charlie, FIXED_MANU_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::NotWhitelisted);
		})
	}

	#[test]
	fn not_enough_funds_for_rent_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
			let nft_id = NFT::next_nft_id() - 1;

			Rent::create_contract(
				alice,
				nft_id,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::ManualAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
				true,
				RentFee::Tokens(TOKENS),
				CancellationFee::FixedTokens(LESS_TOKENS),
				CancellationFee::FixedTokens(LESS_TOKENS),
			)
			.unwrap();

			Balances::set_balance(root(), BOB, TOKENS - 1, 0).unwrap();

			let err = Rent::make_rent_offer(bob, nft_id);
			assert_noop!(err, Error::<Test>::NotEnoughFundsForRentFee);
		})
	}

	#[test]
	fn not_enough_funds_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
			let nft_id = NFT::next_nft_id() - 1;

			Rent::create_contract(
				alice,
				nft_id,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::ManualAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
				true,
				RentFee::Tokens(LESS_TOKENS),
				CancellationFee::FixedTokens(TOKENS),
				CancellationFee::FixedTokens(TOKENS),
			)
			.unwrap();

			Balances::set_balance(root(), BOB, LESS_TOKENS, 0).unwrap();

			let err = Rent::make_rent_offer(bob, nft_id);
			assert_noop!(err, Error::<Test>::NotEnoughFundsForCancellationFee);
		})
	}

	#[test]
	fn not_enough_funds_for_fees() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
			let nft_id = NFT::next_nft_id() - 1;

			Rent::create_contract(
				alice,
				nft_id,
				DurationInput::Fixed(BLOCK_DURATION),
				AcceptanceType::ManualAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
				true,
				RentFee::Tokens(TOKENS),
				CancellationFee::FixedTokens(TOKENS),
				CancellationFee::FixedTokens(TOKENS),
			)
			.unwrap();

			Balances::set_balance(root(), BOB, TOKENS, 0).unwrap();

			let err = Rent::make_rent_offer(bob, nft_id);
			assert_noop!(err, Error::<Test>::NotEnoughFundsForFees);
		})
	}

	#[test]
	fn caller_does_not_own_rent_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			let mut nft = NFT::get_nft(BOB_NFT_ID_1).unwrap();
			nft.owner = ALICE;
			NFT::set_nft(BOB_NFT_ID_1, nft).unwrap();

			let err = Rent::make_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::CallerDoesNotOwnRentNFT);
		})
	}

	#[test]
	fn rent_nft_not_in_valid_state() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			let mut nft = NFT::get_nft(BOB_NFT_ID_1).unwrap();
			nft.state.is_listed = true;
			NFT::set_nft(BOB_NFT_ID_1, nft).unwrap();

			let err = Rent::make_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::RentNFTNotInValidState);
		})
	}

	#[test]
	fn caller_does_not_own_cancellation_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			let mut nft = NFT::get_nft(BOB_NFT_ID_0).unwrap();
			nft.owner = ALICE;
			NFT::set_nft(BOB_NFT_ID_0, nft).unwrap();

			let err = Rent::make_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::CallerDoesNotOwnCancellationNFT);
		})
	}

	#[test]
	fn cancellation_nft_not_in_valid_state() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			let mut nft = NFT::get_nft(BOB_NFT_ID_0).unwrap();
			nft.state.is_listed = true;
			NFT::set_nft(BOB_NFT_ID_0, nft).unwrap();

			let err = Rent::make_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::CancellationNFTNotInValidState);
		})
	}
}

mod accept_rent_offer {
	use super::*;

	#[test]
	fn accept_rent_offer() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			// Make rent offer.
			Rent::make_rent_offer(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();

			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);

			// Accept rent offer
			Rent::accept_rent_offer(alice, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK, BOB)
				.unwrap();

			// State check.
			let nft = NFT::nfts(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();
			let contract = Rent::contracts(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();
			assert_eq!(contract.rentee, Some(BOB));
			assert!(contract.start_block.is_some());
			assert!(Rent::queues()
				.available_queue
				.get(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK)
				.is_none());
			assert!(Rent::queues()
				.subscription_queue
				.get(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK)
				.is_some());
			assert!(nft.state.is_rented);
			assert_eq!(Balances::free_balance(ALICE), alice_balance + TOKENS);
			assert_eq!(Balances::free_balance(BOB), bob_balance - TOKENS - LESS_TOKENS);
			// Event check.
			let event = RentEvent::ContractStarted {
				nft_id: SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				rentee: BOB,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to accept rent offer for non existing contract.
			let err = Rent::accept_rent_offer(alice, INVALID_NFT, BOB);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn no_offers_for_this_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to accept rent offer for non existing contract.
			let err = Rent::accept_rent_offer(alice, FIXED_MANU_REV_NFT_NFT_NFT, BOB);
			assert_noop!(err, Error::<Test>::NoOffersForThisContract);
		})
	}

	#[test]
	fn not_enough_funds_for_rent_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();

			Balances::set_balance(root(), BOB, 0, 0).unwrap();

			// Try to accept rent offer for non existing contract.
			let err =
				Rent::accept_rent_offer(alice, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK, BOB);
			assert_noop!(err, Error::<Test>::NotEnoughFundsForRentFee);
		})
	}

	#[test]
	fn not_enough_funds_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();

			Balances::set_balance(root(), BOB, TOKENS + 1, 0).unwrap();

			// Try to accept rent offer for non existing contract.
			let err =
				Rent::accept_rent_offer(alice, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK, BOB);
			assert_noop!(err, Error::<Test>::NotEnoughFundsForCancellationFee);
		})
	}

	#[test]
	fn rentee_does_not_own_the_rent_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT).unwrap();

			let mut nft = NFT::get_nft(BOB_NFT_ID_1).unwrap();
			nft.owner = ALICE;
			NFT::set_nft(BOB_NFT_ID_1, nft).unwrap();

			// Try to accept rent offer for non existing contract.
			let err = Rent::accept_rent_offer(alice, FIXED_MANU_REV_NFT_NFT_NFT, BOB);
			assert_noop!(err, Error::<Test>::RenteeDoesNotOwnTheRentNFT);
		})
	}

	#[test]
	fn rent_nft_not_in_valid_state() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT).unwrap();

			let mut nft = NFT::get_nft(BOB_NFT_ID_1).unwrap();
			nft.state.is_listed = true;
			NFT::set_nft(BOB_NFT_ID_1, nft).unwrap();

			// Try to accept rent offer for non existing contract.
			let err = Rent::accept_rent_offer(alice, FIXED_MANU_REV_NFT_NFT_NFT, BOB);
			assert_noop!(err, Error::<Test>::RentNFTNotInValidState);
		})
	}

	#[test]
	fn rentee_does_not_own_the_cancellation_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT).unwrap();

			let mut nft = NFT::get_nft(BOB_NFT_ID_0).unwrap();
			nft.owner = ALICE;
			NFT::set_nft(BOB_NFT_ID_0, nft).unwrap();

			// Try to accept rent offer for non existing contract.
			let err = Rent::accept_rent_offer(alice, FIXED_MANU_REV_NFT_NFT_NFT, BOB);
			assert_noop!(err, Error::<Test>::RenteeDoesNotOwnTheCancellationNFT);
		})
	}

	#[test]
	fn cancellation_nft_not_in_valid_state() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT).unwrap();

			let mut nft = NFT::get_nft(BOB_NFT_ID_0).unwrap();
			nft.state.is_listed = true;
			NFT::set_nft(BOB_NFT_ID_0, nft).unwrap();

			// Try to accept rent offer for non existing contract.
			let err = Rent::accept_rent_offer(alice, FIXED_MANU_REV_NFT_NFT_NFT, BOB);
			assert_noop!(err, Error::<Test>::CancellationNFTNotInValidState);
		})
	}
}

mod retract_rent_offer {
	use super::*;

	#[test]
	fn retract_rent_offer() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob.clone(), FIXED_MANU_REV_NFT_NFT_NFT).unwrap();
			let offers = Rent::offers(FIXED_MANU_REV_NFT_NFT_NFT).unwrap();
			assert!(offers.contains(&BOB));

			// Retract offer.
			Rent::retract_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT).unwrap();
			let offers = Rent::offers(FIXED_MANU_REV_NFT_NFT_NFT).unwrap();
			assert!(!offers.contains(&BOB));

			// Event check.
			let event = RentEvent::ContractOfferRetracted {
				nft_id: FIXED_MANU_REV_NFT_NFT_NFT,
				rentee: BOB,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn no_offers_for_this_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			// Retract offer.
			let err = Rent::retract_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::NoOffersForThisContract);
		})
	}

	#[test]
	fn no_offers_from_this_address() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);
			let charlie: mock::RuntimeOrigin = origin(CHARLIE);

			Rent::make_rent_offer(bob, FIXED_MANU_REV_NFT_NFT_NFT).unwrap();
			let offers = Rent::offers(FIXED_MANU_REV_NFT_NFT_NFT).unwrap();
			assert!(offers.contains(&BOB));

			// Retract offer.
			let err = Rent::retract_rent_offer(charlie, FIXED_MANU_REV_NFT_NFT_NFT);
			assert_noop!(err, Error::<Test>::NoOfferFromThisAddress);
		})
	}
}

mod change_subscription_terms {
	use super::*;

	#[test]
	fn change_subscription_terms() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();
			Rent::accept_rent_offer(
				alice.clone(),
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				BOB,
			)
			.unwrap();

			// Change subscription terms.
			Rent::change_subscription_terms(
				alice,
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				TOKENS + 1,
				2 * BLOCK_DURATION,
				Some(4 * BLOCK_DURATION),
				true,
			)
			.unwrap();

			// State check.
			let contract = Rent::contracts(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();
			assert!(contract.start_block.is_some());
			assert!(contract.duration.terms_changed());

			// Event check.
			let event = RentEvent::ContractSubscriptionTermsChanged {
				nft_id: SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				period: 2 * BLOCK_DURATION,
				max_duration: 4 * BLOCK_DURATION,
				is_changeable: true,
				rent_fee: TOKENS + 1,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Change subscription terms for invalid contract.
			let err = Rent::change_subscription_terms(
				alice,
				INVALID_NFT,
				TOKENS + 1,
				2 * BLOCK_DURATION,
				Some(20 * BLOCK_DURATION),
				true,
			);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn not_the_contract_owner() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			// Change subscription terms for without being contract owner.
			let err = Rent::change_subscription_terms(
				bob,
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				TOKENS + 1,
				2 * BLOCK_DURATION,
				Some(20 * BLOCK_DURATION),
				true,
			);
			assert_noop!(err, Error::<Test>::NotTheContractOwner);
		})
	}

	#[test]
	fn cannot_adjust_subscription_terms() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Change subscription terms for fixed contract.
			let err = Rent::change_subscription_terms(
				alice,
				FIXED_AUTO_REV_NFT_NFT_NFT,
				TOKENS + 1,
				2 * BLOCK_DURATION,
				Some(20 * BLOCK_DURATION),
				true,
			);
			assert_noop!(err, Error::<Test>::CannotAdjustSubscriptionTerms);
		})
	}

	#[test]
	fn duration_exceeds_maximum_limit() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Change subscription terms with invalid max duration.
			let err = Rent::change_subscription_terms(
				alice,
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				TOKENS + 1,
				1000 * BLOCK_DURATION,
				Some(20 * BLOCK_DURATION),
				true,
			);
			assert_noop!(err, Error::<Test>::DurationExceedsMaximumLimit);
		})
	}

	#[test]
	fn duration_invalid() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Try to create a subscription contract with 0 duration.
			let err = Rent::change_subscription_terms(
				alice.clone(),
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				TOKENS,
				0,
				None,
				true,
			);
			assert_noop!(err, Error::<Test>::DurationInvalid);

			// Try to create a subscription contract with 0 duration.
			let err = Rent::change_subscription_terms(
				alice.clone(),
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				TOKENS,
				BLOCK_DURATION,
				Some(0),
				true,
			);
			assert_noop!(err, Error::<Test>::DurationInvalid);
		})
	}
}

mod accept_subscription_terms {
	use super::*;

	#[test]
	fn accept_subscription_terms() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob.clone(), SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK)
				.unwrap();
			Rent::accept_rent_offer(
				alice.clone(),
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				BOB,
			)
			.unwrap();

			// Change subscription terms.
			Rent::change_subscription_terms(
				alice,
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				TOKENS + 1,
				2 * BLOCK_DURATION,
				Some(4 * BLOCK_DURATION),
				true,
			)
			.unwrap();

			// Accept new subscription terms
			Rent::accept_subscription_terms(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK)
				.unwrap();

			// State check.
			let contract = Rent::contracts(SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK).unwrap();
			assert!(!contract.duration.terms_changed());

			// Event check.
			let event = RentEvent::ContractSubscriptionTermsAccepted {
				nft_id: SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
			};
			let event = RuntimeEvent::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = origin(BOB);

			// Try to accept new subscription terms for invalid contract.
			let err = Rent::accept_subscription_terms(bob, INVALID_NFT);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn not_the_contract_rentee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);
			let charlie: mock::RuntimeOrigin = origin(CHARLIE);

			Rent::make_rent_offer(bob.clone(), SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK)
				.unwrap();
			Rent::accept_rent_offer(
				alice.clone(),
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				BOB,
			)
			.unwrap();

			// Change subscription terms.
			Rent::change_subscription_terms(
				alice,
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
				TOKENS + 1,
				2 * BLOCK_DURATION,
				Some(4 * BLOCK_DURATION),
				true,
			)
			.unwrap();

			// Try to accept new subscription terms without being rentee
			let err = Rent::accept_subscription_terms(
				charlie,
				SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK,
			);
			assert_noop!(err, Error::<Test>::NotTheContractRentee);
		})
	}

	#[test]
	fn contract_terms_already_accepted() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);
			let bob: mock::RuntimeOrigin = origin(BOB);

			Rent::make_rent_offer(bob.clone(), SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK)
				.unwrap();
			Rent::accept_rent_offer(alice, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK, BOB)
				.unwrap();

			// Try to accept subscription terms already accepted
			let err =
				Rent::accept_subscription_terms(bob, SUBSC_MANU_REV_CHANGEABLE_TOK_FIXTOK_FIXTOK);
			assert_noop!(err, Error::<Test>::ContractTermsAlreadyAccepted);
		})
	}
}
