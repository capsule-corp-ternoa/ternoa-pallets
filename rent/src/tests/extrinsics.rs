/* // Copyright 2022 Capsule Corp (France) SAS.
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
use frame_support::{assert_noop, error::BadOrigin, BoundedVec};
use frame_system::RawOrigin;
use primitives::nfts::{NFTId, NFTState};
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::NFTExt;

use crate::{
	tests::mock, AcceptanceType, CancellationFee, Duration, Error, Event as RentEvent,
	RentContractData, RentFee, RevocationType,
};

const ALICE_NFT_ID_0: NFTId = 0;
const ALICE_NFT_ID_1: NFTId = 1;
const ALICE_NFT_ID_2: NFTId = 2;
const ALICE_NFT_ID_3: NFTId = 3;
const ALICE_NFT_ID_4: NFTId = 4;
const ALICE_NFT_ID_5: NFTId = 5;
const ALICE_NFT_ID_6: NFTId = 6;
const ALICE_NFT_ID_7: NFTId = 7;
const BOB_NFT_ID_0: NFTId = 8;
const BOB_NFT_ID_1: NFTId = 9;
const BOB_NFT_ID_2: NFTId = 10;
const INVALID_NFT: NFTId = 99;
const PERCENT_0: Permill = Permill::from_parts(0);
const TOKENS: Balance = 100;
const LESS_TOKENS: Balance = 10;

// DURATION_ACCEPTANCE_REVOCATION_RENTFEE_RENTERCANCELLATION_RENTEECANCELLATION
const FIXED_AUTO_NOREV_NFT_NONE_NONE: NFTId = 0;
const SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK: NFTId = 1;
const INFINITE_AUTO_ANY_TOK_NFT_NFT: NFTId = 2;
const FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK: NFTId = 3;
const FIXED_MANU_ANY_NFT_NONE_NFT: NFTId = 4;

const BLOCK_DURATION: u64 = 100;
const BLOCK_MAX_DURATION: u64 = 1000;

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

fn prepare_tests() {
	let alice: mock::Origin = origin(ALICE);
	let bob: mock::Origin = origin(BOB);

	//Create NFTs.
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

	//Create contracts.
	Rent::create_contract(
		alice.clone(),
		FIXED_AUTO_NOREV_NFT_NONE_NONE,
		Duration::Fixed(BLOCK_DURATION),
		AcceptanceType::AutoAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		RevocationType::NoRevocation,
		RentFee::NFT(BOB_NFT_ID_0),
		None,
		None,
	)
	.unwrap();
	Rent::create_contract(
		alice.clone(),
		SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
		Duration::Subscription(BLOCK_DURATION, Some(BLOCK_MAX_DURATION)),
		AcceptanceType::ManualAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		RevocationType::OnSubscriptionChange,
		RentFee::Tokens(TOKENS),
		Some(CancellationFee::FixedTokens(LESS_TOKENS)),
		Some(CancellationFee::FixedTokens(LESS_TOKENS)),
	)
	.unwrap();
	Rent::create_contract(
		alice.clone(),
		INFINITE_AUTO_ANY_TOK_NFT_NFT,
		Duration::Infinite,
		AcceptanceType::AutoAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		RevocationType::Anytime,
		RentFee::Tokens(TOKENS),
		Some(CancellationFee::NFT(ALICE_NFT_ID_5)),
		Some(CancellationFee::NFT(BOB_NFT_ID_0)),
	)
	.unwrap();
	Rent::create_contract(
		alice.clone(),
		FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK,
		Duration::Fixed(BLOCK_DURATION),
		AcceptanceType::AutoAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		RevocationType::Anytime,
		RentFee::Tokens(TOKENS),
		Some(CancellationFee::FlexibleTokens(LESS_TOKENS)),
		Some(CancellationFee::FlexibleTokens(LESS_TOKENS)),
	)
	.unwrap();
	Rent::create_contract(
		alice,
		FIXED_MANU_ANY_NFT_NONE_NFT,
		Duration::Fixed(BLOCK_DURATION),
		AcceptanceType::ManualAcceptance(Some(BoundedVec::try_from(vec![BOB]).unwrap())),
		RevocationType::Anytime,
		RentFee::NFT(BOB_NFT_ID_1),
		None,
		Some(CancellationFee::NFT(BOB_NFT_ID_2)),
	)
	.unwrap();

	//Check existence
	assert!(NFT::nfts(ALICE_NFT_ID_0).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_1).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_2).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_3).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_4).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_5).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_6).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_7).is_some());
	assert!(NFT::nfts(BOB_NFT_ID_0).is_some());
	assert!(NFT::nfts(BOB_NFT_ID_1).is_some());
	assert!(NFT::nfts(BOB_NFT_ID_2).is_some());
	assert!(Rent::contracts(FIXED_AUTO_NOREV_NFT_NONE_NONE).is_some());
	assert!(Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).is_some());
	assert!(Rent::contracts(INFINITE_AUTO_ANY_TOK_NFT_NFT).is_some());
	assert!(Rent::contracts(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).is_some());
	assert!(Rent::contracts(FIXED_MANU_ANY_NFT_NONE_NFT).is_some());
}

mod create_contract {
	use super::*;

	#[test]
	fn create_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			let data: RentContractData<u64, u64, Balance, RentAccountSizeLimit> =
				RentContractData::new(
					false,
					None,
					ALICE,
					None,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					false,
					None,
					None,
				);

			// Create basic contract.
			Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				data.duration.clone(),
				data.acceptance_type.clone(),
				data.revocation_type.clone(),
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
				revocation_type: data.revocation_type,
				rent_fee: data.rent_fee,
				renter_cancellation_fee: data.renter_cancellation_fee,
				rentee_cancellation_fee: data.rentee_cancellation_fee,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn subscription_change_for_subscription_only() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with invalid Duration / RevocationType.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::OnSubscriptionChange,
				RentFee::Tokens(TOKENS),
				None,
				None,
			);

			assert_noop!(err, Error::<Test>::SubscriptionChangeForSubscriptionOnly);
		})
	}

	#[test]
	fn no_nft_rent_fee_with_subscription() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with invalid Duration / Rent fee.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				Duration::Subscription(BLOCK_DURATION, None),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::NFT(BOB_NFT_ID_0),
				None,
				None,
			);

			assert_noop!(err, Error::<Test>::NoNFTRentFeeWithSubscription);
		})
	}

	#[test]
	fn no_renter_cancellation_fee_with_no_revocation() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with invalid Cancellation fee / RevocationType.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::NoRevocation,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::FixedTokens(TOKENS)),
				None,
			);

			assert_noop!(err, Error::<Test>::NoRenterCancellationFeeWithNoRevocation);
		})
	}

	#[test]
	fn flexible_fee_only_for_fixed_duration_renter() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with invalid Cancellation fee / Duration.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				Duration::Infinite,
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::FlexibleTokens(TOKENS)),
				None,
			);

			assert_noop!(err, Error::<Test>::FlexibleFeeOnlyForFixedDuration);
		})
	}

	#[test]
	fn flexible_fee_only_for_fixed_duration_rentee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with invalid Cancellation fee / Duration.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				Duration::Subscription(BLOCK_DURATION, Some(BLOCK_MAX_DURATION)),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				None,
				Some(CancellationFee::FlexibleTokens(TOKENS)),
			);

			assert_noop!(err, Error::<Test>::FlexibleFeeOnlyForFixedDuration);
		})
	}

	#[test]
	fn invalid_fee_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with NFT used for contract, rentfee and cancellation fees.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::NFT(ALICE_NFT_ID_6),
				Some(CancellationFee::NFT(ALICE_NFT_ID_6)),
				Some(CancellationFee::NFT(ALICE_NFT_ID_6)),
			);

			assert_noop!(err, Error::<Test>::InvalidFeeNFT);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with invalid NFT.
			let err = Rent::create_contract(
				alice,
				INVALID_NFT,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				None,
				None,
			);

			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with unowned NFT.
			let err = Rent::create_contract(
				alice,
				BOB_NFT_ID_0,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				None,
				None,
			);

			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_use_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Set is_listed to true for Alice's NFT.
			let nft_state = NFTState::new(false, true, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				None,
				None,
			);
			assert_noop!(err, Error::<Test>::CannotUseListedNFTs);

			// Set is_capsule to true for Alice's NFT.
			let nft_state = NFTState::new(true, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				None,
				None,
			);
			assert_noop!(err, Error::<Test>::CannotUseCapsuleNFTs);

			// Set is_delegated to true for Alice's NFT.
			let nft_state = NFTState::new(false, false, false, true, false, false);
			NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				None,
				None,
			);
			assert_noop!(err, Error::<Test>::CannotUseDelegatedNFTs);

			// Set is_soulbound to true for Alice's NFT.
			let nft_state = NFTState::new(false, false, false, false, true, false);
			NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				None,
				None,
			);
			assert_noop!(err, Error::<Test>::CannotUseSoulboundNFTs);

			// Set is_rented to true for Alice's NFT.
			let nft_state = NFTState::new(false, false, false, false, false, true);
			NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				None,
				None,
			);
			assert_noop!(err, Error::<Test>::CannotUseRentedNFTs);
		})
	}

	#[test]
	fn not_enough_balance_for_fixed_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with unowned NFT.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::FixedTokens(1_000_001)),
				None,
			);

			assert_noop!(err, Error::<Test>::NotEnoughBalanceForCancellationFee);
		})
	}

	#[test]
	fn not_enough_balance_for_flexible_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with unowned NFT.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::FlexibleTokens(1_000_001)),
				None,
			);

			assert_noop!(err, Error::<Test>::NotEnoughBalanceForCancellationFee);
		})
	}

	#[test]
	fn nft_not_found_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with unowned NFT.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::NFT(INVALID_NFT)),
				None,
			);

			assert_noop!(err, Error::<Test>::NFTNotFoundForCancellationFee);
		})
	}

	#[test]
	fn not_the_nft_owner_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to create a contract with unowned NFT.
			let err = Rent::create_contract(
				alice,
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::NFT(BOB_NFT_ID_0)),
				None,
			);

			assert_noop!(err, Error::<Test>::NotTheNFTOwnerForCancellationFee);
		})
	}

	#[test]
	fn cannot_use_nft_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Set is_listed to true for Alice's cancellation NFT.
			let nft_state = NFTState::new(false, true, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID_7, nft_state).unwrap();
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::NFT(ALICE_NFT_ID_7)),
				None,
			);
			assert_noop!(err, Error::<Test>::CannotUseListedNFTs);

			// Set is_capsule to true for Alice's cancellation NFT.
			let nft_state = NFTState::new(true, false, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID_7, nft_state).unwrap();
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::NFT(ALICE_NFT_ID_7)),
				None,
			);
			assert_noop!(err, Error::<Test>::CannotUseCapsuleNFTs);

			// Set is_delegated to true for Alice's cancellation NFT.
			let nft_state = NFTState::new(false, false, false, true, false, false);
			NFT::set_nft_state(ALICE_NFT_ID_7, nft_state).unwrap();
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::NFT(ALICE_NFT_ID_7)),
				None,
			);
			assert_noop!(err, Error::<Test>::CannotUseDelegatedNFTs);

			// Set is_soulbound to true for Alice's cancellation NFT.
			let nft_state = NFTState::new(false, false, false, false, true, false);
			NFT::set_nft_state(ALICE_NFT_ID_7, nft_state).unwrap();
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::NFT(ALICE_NFT_ID_7)),
				None,
			);
			assert_noop!(err, Error::<Test>::CannotUseSoulboundNFTs);

			// Set is_rented to true for Alice's cancellation NFT.
			let nft_state = NFTState::new(false, false, false, false, false, true);
			NFT::set_nft_state(ALICE_NFT_ID_7, nft_state).unwrap();
			let err = Rent::create_contract(
				alice.clone(),
				ALICE_NFT_ID_6,
				Duration::Fixed(BLOCK_DURATION),
				AcceptanceType::AutoAcceptance(None),
				RevocationType::Anytime,
				RentFee::Tokens(TOKENS),
				Some(CancellationFee::NFT(ALICE_NFT_ID_7)),
				None,
			);
			assert_noop!(err, Error::<Test>::CannotUseRentedNFTs);
		})
	}
}

mod revoke_contract {
	use super::*;

	#[test]
	fn revoke_contract_before_start() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Revoke before start.
			Rent::revoke_contract(alice, FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();
			// State check.
			let nft = NFT::nfts(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();
			assert!(Rent::contracts(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).is_none());
			assert!(Rent::queues()
				.available_queue
				.get(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK)
				.is_none());
			assert!(!nft.state.is_rented);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK,
				revoked_by: ALICE,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_renter_fixed() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Rent::accept_rent_offer(alice.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);

			// Revoke.
			Rent::revoke_contract(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();

			// State check.
			let nft = NFT::nfts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			assert!(Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).is_none());
			assert!(Rent::queues().available_queue.get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(Balances::free_balance(ALICE), alice_balance);
			assert_eq!(Balances::free_balance(BOB), bob_balance + 2 * LESS_TOKENS);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				revoked_by: ALICE,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_rentee_fixed() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);

			// Revoke.
			Rent::revoke_contract(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();

			// State check.
			let nft = NFT::nfts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			assert!(Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).is_none());
			assert!(Rent::queues().available_queue.get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(Balances::free_balance(ALICE), alice_balance + 2 * LESS_TOKENS);
			assert_eq!(Balances::free_balance(BOB), bob_balance);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				revoked_by: BOB,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_renter_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();

			// Revoke.
			Rent::revoke_contract(alice, INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();

			// State check.
			let nft = NFT::nfts(INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();
			let alice_cancellation_nft = NFT::nfts(ALICE_NFT_ID_5).unwrap();
			let bob_cancellation_nft = NFT::nfts(BOB_NFT_ID_0).unwrap();
			assert!(Rent::contracts(INFINITE_AUTO_ANY_TOK_NFT_NFT).is_none());
			assert!(Rent::queues().available_queue.get(INFINITE_AUTO_ANY_TOK_NFT_NFT).is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(alice_cancellation_nft.owner, BOB);
			assert_eq!(bob_cancellation_nft.owner, BOB);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: INFINITE_AUTO_ANY_TOK_NFT_NFT,
				revoked_by: ALICE,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_rentee_nft() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob.clone(), INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();

			// Revoke.
			Rent::revoke_contract(bob, INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();

			// State check.
			let nft = NFT::nfts(INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();
			let alice_cancellation_nft = NFT::nfts(ALICE_NFT_ID_5).unwrap();
			let bob_cancellation_nft = NFT::nfts(BOB_NFT_ID_0).unwrap();
			assert!(Rent::contracts(INFINITE_AUTO_ANY_TOK_NFT_NFT).is_none());
			assert!(Rent::queues().available_queue.get(INFINITE_AUTO_ANY_TOK_NFT_NFT).is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(alice_cancellation_nft.owner, ALICE);
			assert_eq!(bob_cancellation_nft.owner, ALICE);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: INFINITE_AUTO_ANY_TOK_NFT_NFT,
				revoked_by: BOB,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_renter_flexible() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();

			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);

			// Change current block
			run_to_block(BLOCK_DURATION / 5);

			// Revoke.
			Rent::revoke_contract(alice, FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();

			// State check.
			let nft = NFT::nfts(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();
			assert!(Rent::contracts(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).is_none());
			assert!(Rent::queues()
				.available_queue
				.get(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK)
				.is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(Balances::free_balance(ALICE), alice_balance + (LESS_TOKENS / 5));
			assert_eq!(
				Balances::free_balance(BOB),
				bob_balance + LESS_TOKENS + (4 * LESS_TOKENS / 5)
			);
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK,
				revoked_by: ALICE,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn revoke_contract_by_rentee_flexible() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob.clone(), FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();

			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);

			// Change current block.
			run_to_block(BLOCK_DURATION / 5);

			// Revoke.
			Rent::revoke_contract(bob, FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();

			// State check.
			let nft = NFT::nfts(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();
			assert!(Rent::contracts(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).is_none());
			assert!(Rent::queues()
				.available_queue
				.get(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK)
				.is_none());
			assert!(!nft.state.is_rented);
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance + LESS_TOKENS + (4 * LESS_TOKENS / 5)
			);
			assert_eq!(Balances::free_balance(BOB), bob_balance + (LESS_TOKENS / 5));
			// Event check.
			let event = RentEvent::ContractRevoked {
				nft_id: FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK,
				revoked_by: BOB,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to revoke with invalid contract.
			let err = Rent::revoke_contract(alice, INVALID_NFT);

			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn not_the_renter_or_rentee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			// Try to revoke with unowned contract.
			let err = Rent::revoke_contract(bob, FIXED_AUTO_NOREV_NFT_NONE_NONE);

			assert_noop!(err, Error::<Test>::NotTheRenterOrRentee);
		})
	}

	#[test]
	fn cannot_revoke() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, FIXED_AUTO_NOREV_NFT_NONE_NONE).unwrap();

			// Try to revoke with no revocation as duration.
			let err = Rent::revoke_contract(alice, FIXED_AUTO_NOREV_NFT_NONE_NONE);
			assert_noop!(err, Error::<Test>::CannotRevoke);
		})
	}
}

mod rent {
	use super::*;

	#[test]
	fn rent_auto() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			// Rent contract.
			Rent::rent(bob, FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();

			// State check.
			let nft = NFT::nfts(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();
			let contract = Rent::contracts(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();
			assert_eq!(contract.rentee, Some(BOB));
			assert!(contract.has_started);
			assert!(contract.terms_accepted);
			assert!(Rent::queues()
				.available_queue
				.get(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK)
				.is_none());
			assert!(Rent::queues().fixed_queue.get(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).is_some());
			assert!(nft.state.is_rented);

			// Event check.
			let event = RentEvent::ContractStarted {
				nft_id: FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK,
				rentee: BOB,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn rent_manual() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			// Rent contract (make offer).
			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();

			// Accept rent offer.
			Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

			// State check.
			let nft = NFT::nfts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			let contract = Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			assert_eq!(contract.rentee, Some(BOB));
			assert!(contract.has_started);
			assert!(contract.terms_accepted);
			assert!(Rent::queues().available_queue.get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).is_none());
			assert!(Rent::queues()
				.subscription_queue
				.get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK)
				.is_some());
			assert!(nft.state.is_rented);

			// Event check.
			let event = RentEvent::ContractStarted {
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				rentee: BOB,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to rent invalid contract.
			let err = Rent::rent(alice, INVALID_NFT);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn cannot_rent_own_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to rent owned contract.
			let err = Rent::rent(alice, FIXED_AUTO_NOREV_NFT_NONE_NONE);
			assert_noop!(err, Error::<Test>::CannotRentOwnContract);
		})
	}

	#[test]
	fn not_enough_balance_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			Balances::set_balance(root(), BOB, LESS_TOKENS - 1, 0).unwrap();

			// Try to rent without enough balance for cancellation fee.
			let err = Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK);
			assert_noop!(err, Error::<Test>::NotEnoughBalanceForCancellationFee);
		})
	}

	#[test]
	fn not_enough_balance_for_rent_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			Balances::set_balance(root(), BOB, TOKENS - 1, 0).unwrap();

			// Try to rent without enough tokens for rent fee.
			let err = Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK);
			assert_noop!(err, Error::<Test>::NotEnoughBalanceForRentFee);
		})
	}

	#[test]
	fn not_enough_balance() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			Balances::set_balance(root(), BOB, LESS_TOKENS + TOKENS - 1, 0).unwrap();

			// Try to rent without enough tokens for rent fee + cancellation fee.
			let err = Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK);
			assert_noop!(err, Error::<Test>::NotEnoughBalance);
		})
	}

	#[test]
	fn not_the_nft_owner_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			// Change ownership of cancellation NFT.
			let mut nft = NFT::get_nft(BOB_NFT_ID_0).unwrap();
			nft.owner = ALICE;
			NFT::set_nft(BOB_NFT_ID_0, nft).unwrap();

			// Try to rent without nft for cancellation fee.
			let err = Rent::rent(bob, INFINITE_AUTO_ANY_TOK_NFT_NFT);
			assert_noop!(err, Error::<Test>::NotTheNFTOwnerForCancellationFee);
		})
	}

	#[test]
	fn not_the_nft_owner_for_rent_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			// Change ownership of rent fee NFT.
			let mut nft = NFT::get_nft(BOB_NFT_ID_0).unwrap();
			nft.owner = ALICE;
			NFT::set_nft(BOB_NFT_ID_0, nft).unwrap();

			// Try to rent without enough tokens for rent fee.
			let err = Rent::rent(bob, FIXED_AUTO_NOREV_NFT_NONE_NONE);
			assert_noop!(err, Error::<Test>::NotTheNFTOwnerForRentFee);
		})
	}

	#[test]
	fn not_authorized() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let charlie: mock::Origin = origin(CHARLIE);

			// Try to rent without being authorized manual acceptance.
			let err = Rent::rent(charlie.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK);
			assert_noop!(err, Error::<Test>::NotAuthorizedForRent);

			// Try to rent without being authorized auto acceptance.
			let err = Rent::rent(charlie, FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK);
			assert_noop!(err, Error::<Test>::NotAuthorizedForRent);
		})
	}
}

mod accept_rent_offer {
	use super::*;

	#[test]
	fn accept_rent_offer() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			// Make rent offer.
			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();

			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);

			// Accept rent offer
			Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

			// State check.
			let nft = NFT::nfts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			let contract = Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			assert_eq!(contract.rentee, Some(BOB));
			assert!(contract.has_started);
			assert!(contract.terms_accepted);
			assert!(Rent::queues().available_queue.get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).is_none());
			assert!(Rent::queues()
				.subscription_queue
				.get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK)
				.is_some());
			assert!(nft.state.is_rented);
			assert_eq!(Balances::free_balance(ALICE), alice_balance + TOKENS);
			assert_eq!(Balances::free_balance(BOB), bob_balance - TOKENS - LESS_TOKENS);
			// Event check.
			let event = RentEvent::ContractStarted {
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				rentee: BOB,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn not_the_renter() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			// Make rent offer.
			Rent::rent(bob.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();

			// Try to accept rent offer without being renter.
			let err = Rent::accept_rent_offer(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB);
			assert_noop!(err, Error::<Test>::NotTheRenter);
		})
	}

	#[test]
	fn cannot_accept_offer_for_auto_acceptance() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to accept rent offer for auto acceptance contract.
			let err = Rent::accept_rent_offer(alice, FIXED_AUTO_NOREV_NFT_NONE_NONE, BOB);
			assert_noop!(err, Error::<Test>::CannotAcceptOfferForAutoAcceptance);
		})
	}

	#[test]
	fn no_offers_for_this_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to accept rent offer without offers.
			let err = Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB);
			assert_noop!(err, Error::<Test>::NoOffersForThisContract);
		})
	}

	#[test]
	fn no_offer_from_rentee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			// Make rent offer.
			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();

			// Try to accept rent offer from account that did not make offer.
			let err = Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, CHARLIE);
			assert_noop!(err, Error::<Test>::NoOfferFromRentee);
		})
	}

	#[test]
	fn not_enough_balance_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Balances::set_balance(root(), BOB, 0, 0).unwrap();

			// Try to accept rent offer for rentee without cancellation funds anymore.
			let err = Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB);
			assert_noop!(err, Error::<Test>::NotEnoughBalanceForCancellationFee);
		})
	}

	#[test]
	fn not_enough_balance_for_rent_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Balances::set_balance(root(), BOB, LESS_TOKENS + 1, 0).unwrap();

			// Try to accept rent offer for rentee without rent funds anymore.
			let err = Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB);
			assert_noop!(err, Error::<Test>::NotEnoughBalanceForRentFee);
		})
	}

	#[test]
	fn not_enough_balance() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Balances::set_balance(root(), BOB, TOKENS + LESS_TOKENS - 1, 0).unwrap();

			// Try to accept rent offer for rentee without funds anymore.
			let err = Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB);
			assert_noop!(err, Error::<Test>::NotEnoughBalance);
		})
	}

	#[test]
	fn not_the_nft_owner_for_cancellation_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, FIXED_MANU_ANY_NFT_NONE_NFT).unwrap();
			let mut nft = NFT::get_nft(BOB_NFT_ID_2).unwrap();
			nft.owner = ALICE;
			NFT::set_nft(BOB_NFT_ID_2, nft).unwrap();

			// Try to accept rent offer for rentee without cancellation nft.
			let err = Rent::accept_rent_offer(alice, FIXED_MANU_ANY_NFT_NONE_NFT, BOB);
			assert_noop!(err, Error::<Test>::NotTheNFTOwnerForCancellationFee);
		})
	}

	#[test]
	fn not_the_nft_owner_for_rent_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, FIXED_MANU_ANY_NFT_NONE_NFT).unwrap();
			let mut nft = NFT::get_nft(BOB_NFT_ID_1).unwrap();
			nft.owner = ALICE;
			NFT::set_nft(BOB_NFT_ID_1, nft).unwrap();

			// Try to accept rent offer for rentee without rent nft.
			let err = Rent::accept_rent_offer(alice, FIXED_MANU_ANY_NFT_NONE_NFT, BOB);
			assert_noop!(err, Error::<Test>::NotTheNFTOwnerForRentFee);
		})
	}
}

mod retract_rent_offer {
	use super::*;

	#[test]
	fn retract_rent_offer() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob.clone(), FIXED_MANU_ANY_NFT_NONE_NFT).unwrap();
			let offers = Rent::offers(FIXED_MANU_ANY_NFT_NONE_NFT).unwrap();
			assert!(offers.contains(&BOB));

			// Retract offer.
			Rent::retract_rent_offer(bob, FIXED_MANU_ANY_NFT_NONE_NFT).unwrap();
			let offers = Rent::offers(FIXED_MANU_ANY_NFT_NONE_NFT).unwrap();
			assert!(!offers.contains(&BOB));

			// Event check.
			let event = RentEvent::ContractOfferRetracted {
				nft_id: FIXED_MANU_ANY_NFT_NONE_NFT,
				rentee: BOB,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			// Try to retract offer for invalid contract.
			let err = Rent::retract_rent_offer(bob, INVALID_NFT);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn cannot_retract_offer_for_auto_acceptance() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			// Try to retract offer for auto acceptance contract.
			let err = Rent::retract_rent_offer(bob, FIXED_AUTO_NOREV_NFT_NONE_NONE);
			assert_noop!(err, Error::<Test>::CannotRetractOfferForAutoAcceptance);
		})
	}

	#[test]
	fn contract_has_started() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob.clone(), FIXED_MANU_ANY_NFT_NONE_NFT).unwrap();
			Rent::accept_rent_offer(alice, FIXED_MANU_ANY_NFT_NONE_NFT, BOB).unwrap();

			// Try to retract offer after contract has started.
			let err = Rent::retract_rent_offer(bob, FIXED_MANU_ANY_NFT_NONE_NFT);
			assert_noop!(err, Error::<Test>::ContractHasStarted);
		})
	}
}

mod change_subscription_terms {
	use super::*;

	#[test]
	fn change_subscription_terms() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Rent::accept_rent_offer(alice.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

			// Change subscription terms.
			Rent::change_subscription_terms(
				alice,
				SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				Duration::Subscription(BLOCK_DURATION + 1, None),
				TOKENS + 1,
			)
			.unwrap();

			// State check.
			let contract = Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			assert!(contract.has_started);
			assert!(!contract.terms_accepted);
			assert_eq!(contract.duration, Duration::Subscription(BLOCK_DURATION + 1, None));
			assert_eq!(contract.rent_fee, RentFee::Tokens(TOKENS + 1));

			// Event check.
			let event = RentEvent::ContractSubscriptionTermsChanged {
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				duration: Duration::Subscription(BLOCK_DURATION + 1, None),
				rent_fee: RentFee::Tokens(TOKENS + 1),
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Change subscription terms for invalid contract.
			let err = Rent::change_subscription_terms(
				alice,
				INVALID_NFT,
				Duration::Subscription(BLOCK_DURATION + 1, None),
				TOKENS + 1,
			);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn not_the_renter() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			// Change subscription terms from invalid owner.
			let err = Rent::change_subscription_terms(
				bob,
				FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK,
				Duration::Subscription(BLOCK_DURATION + 1, None),
				TOKENS + 1,
			);
			assert_noop!(err, Error::<Test>::NotTheRenter);
		})
	}

	#[test]
	fn contract_has_not_started() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Change subscription terms for contract that did not start.
			let err = Rent::change_subscription_terms(
				alice,
				FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK,
				Duration::Subscription(BLOCK_DURATION + 1, None),
				TOKENS + 1,
			);
			assert_noop!(err, Error::<Test>::ContractHasNotStarted);
		})
	}

	#[test]
	fn can_change_term_for_subscription_only() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, FIXED_AUTO_NOREV_NFT_NONE_NONE).unwrap();

			// Change subscription terms for invalid contract
			let err = Rent::change_subscription_terms(
				alice,
				FIXED_AUTO_NOREV_NFT_NONE_NONE,
				Duration::Subscription(BLOCK_DURATION + 1, None),
				TOKENS + 1,
			);
			assert_noop!(err, Error::<Test>::CanChangeTermForSubscriptionOnly);
		})
	}

	#[test]
	fn can_set_term_for_subscription_only() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Rent::accept_rent_offer(alice.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

			// Change subscription terms for invalid contract
			let err = Rent::change_subscription_terms(
				alice,
				SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				Duration::Fixed(BLOCK_DURATION + 1),
				TOKENS + 1,
			);
			assert_noop!(err, Error::<Test>::CanSetTermsForSubscriptionOnly);
		})
	}
}

mod accept_subscription_terms {
	use super::*;

	#[test]
	fn accept_subscription_terms() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Rent::accept_rent_offer(alice.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

			// Change subscription terms.
			Rent::change_subscription_terms(
				alice,
				SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				Duration::Subscription(BLOCK_DURATION + 1, None),
				TOKENS + 1,
			)
			.unwrap();

			// Accept new subscription terms
			Rent::accept_subscription_terms(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();

			// State check.
			let contract = Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			assert!(contract.terms_accepted);

			// Event check.
			let event = RentEvent::ContractSubscriptionTermsAccepted {
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn not_the_rentee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			// Try to accept new subscription terms without being rentee
			let err = Rent::accept_subscription_terms(bob, FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK);
			assert_noop!(err, Error::<Test>::NotTheRentee);
		})
	}

	#[test]
	fn contract_terms_already_accepted() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob.clone(), FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();

			// Try to accept subscription terms while already accepted
			let err = Rent::accept_subscription_terms(bob, FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK);
			assert_noop!(err, Error::<Test>::ContractTermsAlreadyAccepted);
		})
	}
}

mod end_contract {
	use super::*;

	#[test]
	fn end_contract_fixed() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, FIXED_AUTO_NOREV_NFT_NONE_NONE).unwrap();

			run_to_block(BLOCK_DURATION + 1);

			// State check.
			let contract = Rent::contracts(FIXED_AUTO_NOREV_NFT_NONE_NONE);
			let rent_fee_nft = NFT::get_nft(BOB_NFT_ID_0).unwrap();
			assert!(contract.is_none());
			assert_eq!(rent_fee_nft.owner, ALICE);

			// Event check.
			let event = RentEvent::ContractEnded {
				nft_id: FIXED_AUTO_NOREV_NFT_NONE_NONE,
				revoked_by: None,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn end_contract_subscription() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);
			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);

			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

			run_to_block(BLOCK_MAX_DURATION + 1);

			// State check.
			let contract = Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK);
			assert!(contract.is_none());
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
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				revoked_by: None,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn end_contract_renter() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);
			let alice_balance = Balances::free_balance(ALICE);
			let bob_balance = Balances::free_balance(BOB);

			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Rent::accept_rent_offer(alice.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();
			Rent::change_subscription_terms(
				alice,
				SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				Duration::Subscription(BLOCK_DURATION / 2, Some(BLOCK_MAX_DURATION / 2)),
				TOKENS,
			)
			.unwrap();

			run_to_block(BLOCK_DURATION + 1);

			// State check.
			let contract = Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK);
			assert!(contract.is_none());
			assert_eq!(Balances::free_balance(ALICE), alice_balance + TOKENS);
			assert_eq!(Balances::free_balance(BOB), bob_balance - TOKENS + LESS_TOKENS);

			// Event check.
			let event = RentEvent::ContractEnded {
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				revoked_by: Some(ALICE),
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn end_contract_rentee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);
			let alice_balance = Balances::free_balance(ALICE);

			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();
			Balances::set_balance(root(), BOB, LESS_TOKENS, 0).unwrap();

			run_to_block(BLOCK_DURATION + 1);

			// State check.
			let contract = Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK);
			assert!(contract.is_none());
			assert_eq!(Balances::free_balance(ALICE), alice_balance + TOKENS + 2 * LESS_TOKENS);
			assert_eq!(Balances::free_balance(BOB), LESS_TOKENS);

			// Event check.
			let event = RentEvent::ContractEnded {
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
				revoked_by: Some(BOB),
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let err = Rent::end_contract(alice, INVALID_NFT, None);
			assert_noop!(err, BadOrigin);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let err = Rent::end_contract(root(), INVALID_NFT, None);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn contract_has_not_started() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let err = Rent::end_contract(root(), FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK, None);
			assert_noop!(err, Error::<Test>::ContractHasNotStarted);
		})
	}
}

mod renew_contract {
	use super::*;

	#[test]
	fn renew_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
			Rent::accept_rent_offer(alice, SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

			// Check subscription queue
			assert_eq!(
				Rent::queues().subscription_queue.get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK),
				Some(BLOCK_DURATION + 1)
			);

			run_to_block(BLOCK_DURATION + 1);

			// State check.
			let contract = Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK);
			assert!(contract.is_some());
			assert_eq!(
				Rent::queues().subscription_queue.get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK),
				Some(2 * BLOCK_DURATION + 1)
			);

			// Event check.
			let event = RentEvent::ContractSubscriptionPeriodStarted {
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
			};
			let event = Event::Rent(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let err = Rent::renew_contract(alice, INVALID_NFT);
			assert_noop!(err, BadOrigin);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let err = Rent::renew_contract(root(), INVALID_NFT);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}

	#[test]
	fn contract_has_not_started() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let err = Rent::renew_contract(root(), FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK);
			assert_noop!(err, Error::<Test>::ContractHasNotStarted);
		})
	}

	#[test]
	fn renewal_only_for_subscription() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = origin(BOB);

			Rent::rent(bob, FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).unwrap();

			let err = Rent::renew_contract(root(), FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK);
			assert_noop!(err, Error::<Test>::RenewalOnlyForSubscription);
		})
	}
}

mod remove_expired_contract {
	use super::*;

	#[test]
	fn remove_expired_contract() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			run_to_block((ContractExpirationDuration::get() + 1).into());

			// State check.
			assert!(Rent::contracts(FIXED_AUTO_NOREV_NFT_NONE_NONE).is_none());
			assert_eq!(Rent::queues().available_queue.get(FIXED_AUTO_NOREV_NFT_NONE_NONE), None);
			assert!(Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).is_none());
			assert_eq!(Rent::queues().available_queue.get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK), None);
			assert!(Rent::contracts(INFINITE_AUTO_ANY_TOK_NFT_NFT).is_none());
			assert_eq!(Rent::queues().available_queue.get(INFINITE_AUTO_ANY_TOK_NFT_NFT), None);
			assert!(Rent::contracts(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK).is_none());
			assert_eq!(
				Rent::queues().available_queue.get(FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK),
				None
			);
			assert!(Rent::contracts(FIXED_MANU_ANY_NFT_NONE_NFT).is_none());
			assert_eq!(Rent::queues().available_queue.get(FIXED_MANU_ANY_NFT_NONE_NFT), None);

			// Event check.
			let event_0 = Event::Rent(RentEvent::ContractAvailableExpired {
				nft_id: FIXED_AUTO_NOREV_NFT_NONE_NONE,
			});
			let event_1 = Event::Rent(RentEvent::ContractAvailableExpired {
				nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK,
			});
			let event_2 = Event::Rent(RentEvent::ContractAvailableExpired {
				nft_id: INFINITE_AUTO_ANY_TOK_NFT_NFT,
			});
			let event_3 = Event::Rent(RentEvent::ContractAvailableExpired {
				nft_id: FIXED_AUTO_ANY_TOK_FLEXTOK_FLEXTOK,
			});
			let event_4 = Event::Rent(RentEvent::ContractAvailableExpired {
				nft_id: FIXED_MANU_ANY_NFT_NONE_NFT,
			});
			System::assert_has_event(event_0);
			System::assert_has_event(event_1);
			System::assert_has_event(event_2);
			System::assert_has_event(event_3);
			System::assert_has_event(event_4);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let err = Rent::remove_expired_contract(alice, INVALID_NFT);
			assert_noop!(err, BadOrigin);
		})
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let err = Rent::remove_expired_contract(root(), INVALID_NFT);
			assert_noop!(err, Error::<Test>::ContractNotFound);
		})
	}
}
 */
