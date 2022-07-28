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
use frame_support::{assert_noop, assert_ok, error::BadOrigin, BoundedVec};
use frame_system::RawOrigin;
use primitives::nfts::{NFTData, NFTId, NFTState};
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::NFTExt;

use crate::{
	tests::mock, AcceptanceType, CancellationFee, Duration, Error, Event as RentEvent, RentContractData, RentFee,
	RevocationType,
};

const ALICE_NFT_ID_0: NFTId = 0;
const ALICE_NFT_ID_1: NFTId = 1;
const ALICE_NFT_ID_2: NFTId = 2;
const ALICE_NFT_ID_3: NFTId = 3;
const ALICE_NFT_ID_4: NFTId = 4;
const ALICE_NFT_ID_5: NFTId = 5;
const ALICE_NFT_ID_6: NFTId = 6;
const BOB_NFT_ID_0: NFTId = 7;
const BOB_NFT_ID_1: NFTId = 8;
const INVALID_NFT: NFTId = 99;
const PERCENT_0: Permill = Permill::from_parts(0);
const TOKENS: Balance = 100;
const LESS_TOKENS: Balance = 10;

// DURATION_ACCEPTANCE_REVOCATION_RENTFEE_RENTERCANCELLATION_RENTEECANCELLATION
const FIXED_AUTO_NOREV_NFT_NONE_NONE: NFTId = 0;
const SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK: NFTId = 1;
const INFINITE_AUTO_ANY_TOK_NFT_NFT: NFTId = 2;
const SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK: NFTId = 3;

const BLOCK_DURATION: u64 = 100;
const BLOCK_MAX_DURATION: u64 = 1000;

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

fn escrow_account() -> mock::Origin {
	RawOrigin::Signed(Rent::account_id()).into()
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
	NFT::create_nft(bob.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(bob.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();

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
		AcceptanceType::AutoAcceptance(None),
		RevocationType::Anytime,
		RentFee::Tokens(TOKENS),
		Some(CancellationFee::NFT(ALICE_NFT_ID_4)),
		Some(CancellationFee::NFT(BOB_NFT_ID_0)),
	)
	.unwrap();
	Rent::create_contract(
		alice.clone(),
		SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK,
		Duration::Subscription(BLOCK_DURATION, Some(BLOCK_MAX_DURATION)),
		AcceptanceType::AutoAcceptance(None),
		RevocationType::OnSubscriptionChange,
		RentFee::Tokens(TOKENS),
		Some(CancellationFee::FlexibleTokens(LESS_TOKENS)),
		Some(CancellationFee::FlexibleTokens(LESS_TOKENS)),
	)
	.unwrap();

	//Check existence
	assert_eq!(NFT::nfts(ALICE_NFT_ID_0).is_some(), true);
	assert_eq!(NFT::nfts(ALICE_NFT_ID_1).is_some(), true);
	assert_eq!(NFT::nfts(ALICE_NFT_ID_2).is_some(), true);
	assert_eq!(NFT::nfts(ALICE_NFT_ID_3).is_some(), true);
	assert_eq!(NFT::nfts(ALICE_NFT_ID_4).is_some(), true);
	assert_eq!(NFT::nfts(ALICE_NFT_ID_5).is_some(), true);
	assert_eq!(NFT::nfts(ALICE_NFT_ID_6).is_some(), true);
	assert_eq!(NFT::nfts(BOB_NFT_ID_0).is_some(), true);
	assert_eq!(NFT::nfts(BOB_NFT_ID_1).is_some(), true);
	assert_eq!(Rent::contracts(FIXED_AUTO_NOREV_NFT_NONE_NONE).is_some(), true);
	assert_eq!(Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).is_some(), true);
	assert_eq!(Rent::contracts(INFINITE_AUTO_ANY_TOK_NFT_NFT).is_some(), true);
	assert_eq!(Rent::contracts(SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK).is_some(), true);
}

mod create_contract {
	use super::*;

	#[test]
	fn create_contract() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				let data: RentContractData<u64, u64, Balance, RentAccountSizeLimit> = RentContractData::new(
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
					alice.clone(),
					ALICE_NFT_ID_5,
					data.duration.clone(),
					data.acceptance_type.clone(),
					data.revocation_type.clone(),
					data.rent_fee.clone(),
					data.renter_cancellation_fee.clone(),
					data.rentee_cancellation_fee.clone(),
				)
				.unwrap();

				// State check.
				let contract = Rent::contracts(ALICE_NFT_ID_5).unwrap();
				let nft = NFT::nfts(ALICE_NFT_ID_5).unwrap();
				assert_eq!(contract, data.clone());
				assert_eq!(nft.state.is_rented, true);

				// Event check.
				let event = RentEvent::ContractCreated {
					nft_id: ALICE_NFT_ID_5,
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
			},
		)
	}

	#[test]
	fn subscription_change_for_subscription_only() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with invalid Duration / RevocationType.
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::OnSubscriptionChange,
					RentFee::Tokens(TOKENS),
					None,
					None,
				);

				assert_noop!(err, Error::<Test>::SubscriptionChangeForSubscriptionOnly);
			},
		)
	}

	#[test]
	fn no_nft_rent_fee_with_subscription() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with invalid Duration / Rent fee.
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Subscription(BLOCK_DURATION, None),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::NFT(BOB_NFT_ID_0),
					None,
					None,
				);

				assert_noop!(err, Error::<Test>::NoNFTRentFeeWithSubscription);
			},
		)
	}

	#[test]
	fn no_renter_cancellation_fee_with_no_revocation() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with invalid Cancellation fee / RevocationType.
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::NoRevocation,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::FixedTokens(TOKENS)),
					None,
				);

				assert_noop!(err, Error::<Test>::NoRenterCancellationFeeWithNoRevocation);
			},
		)
	}

	#[test]
	fn no_infinite_with_flexible_fee_renter() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with invalid Cancellation fee / Duration.
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Infinite,
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::FlexibleTokens(TOKENS)),
					None,
				);

				assert_noop!(err, Error::<Test>::NoInfiniteWithFlexibleFee);
			},
		)
	}

	#[test]
	fn no_infinite_with_flexible_fee_rentee() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with invalid Cancellation fee / Duration.
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Infinite,
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					None,
					Some(CancellationFee::FlexibleTokens(TOKENS)),
				);

				assert_noop!(err, Error::<Test>::NoInfiniteWithFlexibleFee);
			},
		)
	}

	#[test]
	fn invalid_fee_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with NFT used for contract, rentfee and cancellation fees.
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::NFT(ALICE_NFT_ID_5),
					Some(CancellationFee::NFT(ALICE_NFT_ID_5)),
					Some(CancellationFee::NFT(ALICE_NFT_ID_5)),
				);

				assert_noop!(err, Error::<Test>::InvalidFeeNFT);
			},
		)
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with invalid NFT.
				let err = Rent::create_contract(
					alice.clone(),
					INVALID_NFT,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					None,
					None,
				);

				assert_noop!(err, Error::<Test>::NFTNotFound);
			},
		)
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with unowned NFT.
				let err = Rent::create_contract(
					alice.clone(),
					BOB_NFT_ID_0,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					None,
					None,
				);

				assert_noop!(err, Error::<Test>::NotTheNFTOwner);
			},
		)
	}

	#[test]
	fn cannot_use_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Set is_listed to true for Alice's NFT.
				let nft_state = NFTState::new(false, true, false, false, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID_5, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					None,
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseListedNFTs);

				// Set is_capsule to true for Alice's NFT.
				let nft_state = NFTState::new(true, false, false, false, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID_5, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					None,
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseCapsuleNFTs);

				// Set is_delegated to true for Alice's NFT.
				let nft_state = NFTState::new(false, false, false, true, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID_5, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					None,
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseDelegatedNFTs);

				// Set is_soulbound to true for Alice's NFT.
				let nft_state = NFTState::new(false, false, false, false, true, false, false);
				NFT::set_nft_state(ALICE_NFT_ID_5, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					None,
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseSoulboundNFTs);

				// Set is_rented to true for Alice's NFT.
				let nft_state = NFTState::new(false, false, false, false, false, true, false);
				NFT::set_nft_state(ALICE_NFT_ID_5, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					None,
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseRentedNFTs);

				// Set is_auctioned to true for Alice's NFT.
				let nft_state = NFTState::new(false, false, false, false, false, false, true);
				NFT::set_nft_state(ALICE_NFT_ID_5, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					None,
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseAuctionedNFTs);
			},
		)
	}

	#[test]
	fn not_enough_balance_for_fixed_cancellation_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with unowned NFT.
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::FixedTokens(1_000_001)),
					None,
				);

				assert_noop!(err, Error::<Test>::NotEnoughBalanceForCancellationFee);
			},
		)
	}

	#[test]
	fn not_enough_balance_for_flexible_cancellation_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with unowned NFT.
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::FlexibleTokens(1_000_001)),
					None,
				);

				assert_noop!(err, Error::<Test>::NotEnoughBalanceForCancellationFee);
			},
		)
	}

	#[test]
	fn nft_not_found_for_cancellation_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with unowned NFT.
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::NFT(INVALID_NFT)),
					None,
				);

				assert_noop!(err, Error::<Test>::NFTNotFoundForCancellationFee);
			},
		)
	}

	#[test]
	fn not_the_nft_owner_for_cancellation_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to create a contract with unowned NFT.
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::NFT(BOB_NFT_ID_0)),
					None,
				);

				assert_noop!(err, Error::<Test>::NotTheNFTOwnerForCancellationFee);
			},
		)
	}

	#[test]
	fn cannot_use_nft_for_cancellation_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Set is_listed to true for Alice's cancellation NFT.
				let nft_state = NFTState::new(false, true, false, false, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::NFT(ALICE_NFT_ID_6)),
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseListedNFTs);

				// Set is_capsule to true for Alice's cancellation NFT.
				let nft_state = NFTState::new(true, false, false, false, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::NFT(ALICE_NFT_ID_6)),
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseCapsuleNFTs);

				// Set is_delegated to true for Alice's cancellation NFT.
				let nft_state = NFTState::new(false, false, false, true, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::NFT(ALICE_NFT_ID_6)),
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseDelegatedNFTs);

				// Set is_soulbound to true for Alice's cancellation NFT.
				let nft_state = NFTState::new(false, false, false, false, true, false, false);
				NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::NFT(ALICE_NFT_ID_6)),
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseSoulboundNFTs);

				// Set is_rented to true for Alice's cancellation NFT.
				let nft_state = NFTState::new(false, false, false, false, false, true, false);
				NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::NFT(ALICE_NFT_ID_6)),
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseRentedNFTs);

				// Set is_auctioned to true for Alice's cancellation NFT.
				let nft_state = NFTState::new(false, false, false, false, false, false, true);
				NFT::set_nft_state(ALICE_NFT_ID_6, nft_state).unwrap();
				let err = Rent::create_contract(
					alice.clone(),
					ALICE_NFT_ID_5,
					Duration::Fixed(BLOCK_DURATION),
					AcceptanceType::AutoAcceptance(None),
					RevocationType::Anytime,
					RentFee::Tokens(TOKENS),
					Some(CancellationFee::NFT(ALICE_NFT_ID_6)),
					None,
				);
				assert_noop!(err, Error::<Test>::CannotUseAuctionedNFTs);
			},
		)
	}
}

mod revoke_contract {
	use super::*;

	#[test]
	fn revoke_contract_before_start() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Revoke before start.
				Rent::revoke_contract(alice.clone(), SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK).unwrap();
				// State check.
				let nft = NFT::nfts(SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK).unwrap();
				assert_eq!(Rent::contracts(SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK), None);
				assert_eq!(Rent::available_queue().get(SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK), None);
				assert_eq!(nft.state.is_rented, false);
				// Event check.
				let event =
					RentEvent::ContractRevoked { nft_id: SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK, revoked_by: ALICE };
				let event = Event::Rent(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn revoke_contract_by_renter_fixed() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let bob: mock::Origin = origin(BOB);

				Rent::rent(bob.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
				Rent::accept_rent_offer(alice.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

				let alice_balance = Balances::free_balance(ALICE);
				let bob_balance = Balances::free_balance(BOB);

				// Revoke.
				Rent::revoke_contract(alice.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();

				// State check.
				let nft = NFT::nfts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
				assert_eq!(Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK), None);
				assert_eq!(Rent::available_queue().get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK), None);
				assert_eq!(nft.state.is_rented, false);
				assert_eq!(Balances::free_balance(ALICE), alice_balance);
				assert_eq!(Balances::free_balance(BOB), bob_balance + 2 *LESS_TOKENS);
				// Event check.
				let event = RentEvent::ContractRevoked { nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, revoked_by: ALICE };
				let event = Event::Rent(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn revoke_contract_by_rentee_fixed() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let bob: mock::Origin = origin(BOB);

				Rent::rent(bob.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
				Rent::accept_rent_offer(alice.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, BOB).unwrap();

				let alice_balance = Balances::free_balance(ALICE);
				let bob_balance = Balances::free_balance(BOB);

				// Revoke.
				Rent::revoke_contract(bob.clone(), SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();

				// State check.
				let nft = NFT::nfts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK).unwrap();
				assert_eq!(Rent::contracts(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK), None);
				assert_eq!(Rent::available_queue().get(SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK), None);
				assert_eq!(nft.state.is_rented, false);
				assert_eq!(Balances::free_balance(ALICE), alice_balance + 2 * LESS_TOKENS);
				assert_eq!(Balances::free_balance(BOB), bob_balance);
				// Event check.
				let event = RentEvent::ContractRevoked { nft_id: SUBSC_MANU_OSC_TOK_FIXTOK_FIXTOK, revoked_by: BOB };
				let event = Event::Rent(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn revoke_contract_by_renter_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let bob: mock::Origin = origin(BOB);

				Rent::rent(bob.clone(), INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();

				// Revoke.
				Rent::revoke_contract(alice.clone(), INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();

				// State check.
				let nft = NFT::nfts(INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();
				let alice_cancellation_nft = NFT::nfts(ALICE_NFT_ID_4).unwrap();
				let bob_cancellation_nft = NFT::nfts(BOB_NFT_ID_0).unwrap();
				assert_eq!(Rent::contracts(INFINITE_AUTO_ANY_TOK_NFT_NFT), None);
				assert_eq!(Rent::available_queue().get(INFINITE_AUTO_ANY_TOK_NFT_NFT), None);
				assert_eq!(nft.state.is_rented, false);
				assert_eq!(alice_cancellation_nft.owner, BOB);
				assert_eq!(bob_cancellation_nft.owner, BOB);
				// Event check.
				let event = RentEvent::ContractRevoked { nft_id: INFINITE_AUTO_ANY_TOK_NFT_NFT, revoked_by: ALICE };
				let event = Event::Rent(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn revoke_contract_by_rentee_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let bob: mock::Origin = origin(BOB);

				Rent::rent(bob.clone(), INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();

				// Revoke.
				Rent::revoke_contract(bob.clone(), INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();

				// State check.
				let nft = NFT::nfts(INFINITE_AUTO_ANY_TOK_NFT_NFT).unwrap();
				let alice_cancellation_nft = NFT::nfts(ALICE_NFT_ID_4).unwrap();
				let bob_cancellation_nft = NFT::nfts(BOB_NFT_ID_0).unwrap();
				assert_eq!(Rent::contracts(INFINITE_AUTO_ANY_TOK_NFT_NFT), None);
				assert_eq!(Rent::available_queue().get(INFINITE_AUTO_ANY_TOK_NFT_NFT), None);
				assert_eq!(nft.state.is_rented, false);
				assert_eq!(alice_cancellation_nft.owner, ALICE);
				assert_eq!(bob_cancellation_nft.owner, ALICE);
				// Event check.
				let event = RentEvent::ContractRevoked { nft_id: INFINITE_AUTO_ANY_TOK_NFT_NFT, revoked_by: BOB };
				let event = Event::Rent(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn revoke_contract_by_renter_flexible() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let bob: mock::Origin = origin(BOB);

				Rent::rent(bob.clone(), SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK).unwrap();

				let alice_balance = Balances::free_balance(ALICE);
				let bob_balance = Balances::free_balance(BOB);

				// Change current block
				run_to_block(BLOCK_DURATION / 5);

				// Revoke.
				Rent::revoke_contract(alice.clone(), SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK).unwrap();

				// State check.
				let nft = NFT::nfts(SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK).unwrap();
				assert_eq!(Rent::contracts(SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK), None);
				assert_eq!(Rent::available_queue().get(SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK), None);
				assert_eq!(nft.state.is_rented, false);
				assert_eq!(Balances::free_balance(ALICE), alice_balance + (LESS_TOKENS / 5));
				assert_eq!(Balances::free_balance(BOB), bob_balance + LESS_TOKENS + (4 * LESS_TOKENS / 5));
				// Event check.
				let event =
					RentEvent::ContractRevoked { nft_id: SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK, revoked_by: ALICE };
				let event = Event::Rent(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn revoke_contract_by_rentee_flexible() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let bob: mock::Origin = origin(BOB);

				Rent::rent(bob.clone(), SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK).unwrap();

				let alice_balance = Balances::free_balance(ALICE);
				let bob_balance = Balances::free_balance(BOB);

				// Change current block
				run_to_block(BLOCK_DURATION / 5);

				// Revoke.
				Rent::revoke_contract(bob.clone(), SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK).unwrap();

				// State check.
				let nft = NFT::nfts(SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK).unwrap();
				assert_eq!(Rent::contracts(SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK), None);
				assert_eq!(Rent::available_queue().get(SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK), None);
				assert_eq!(nft.state.is_rented, false);
				assert_eq!(Balances::free_balance(ALICE), alice_balance + LESS_TOKENS + (4 * LESS_TOKENS / 5));
				assert_eq!(Balances::free_balance(BOB), bob_balance + (LESS_TOKENS / 5));
				// Event check.
				let event =
					RentEvent::ContractRevoked { nft_id: SUBSC_AUTO_OSC_TOK_FLEXTOK_FLEXTOK, revoked_by: BOB };
				let event = Event::Rent(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn contract_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to revoke with invalid contract.
				let err = Rent::revoke_contract(
					alice.clone(),
					INVALID_NFT,
				);

				assert_noop!(err, Error::<Test>::ContractNotFound);
			},
		)
	}

	#[test]
	fn not_the_renter_or_rentee() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let bob: mock::Origin = origin(BOB);

				// Try to revoke with unowned contract.
				let err = Rent::revoke_contract(
					bob.clone(),
					FIXED_AUTO_NOREV_NFT_NONE_NONE,
				);

				assert_noop!(err, Error::<Test>::NotTheRenterOrRentee);
			},
		)
	}

	#[test]
	fn cannot_revoke() {
		ExtBuilder::new_build(vec![(ALICE, 1_000_000), (BOB, 1_000_000), (Rent::account_id(), 1_000_000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let bob: mock::Origin = origin(BOB);

				Rent::rent(bob, FIXED_AUTO_NOREV_NFT_NONE_NONE).unwrap();

				// Try to revoke with no revocation as duration.
				let err = Rent::revoke_contract(
					alice.clone(),
					FIXED_AUTO_NOREV_NFT_NONE_NONE,
				);

				assert_noop!(err, Error::<Test>::CannotRevoke);
			},
		)
	}
}

mod rent {
	use super::*;

	#[test]
	fn rent_auto() {

	}
	
	#[test]
	fn rent_manual() {

	}

	
	#[test]
	fn rent_authorized() {

	}
}