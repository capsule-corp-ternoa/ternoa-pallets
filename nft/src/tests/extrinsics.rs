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
use pallet_balances::Error as BalanceError;
use primitives::nfts::NFTState;
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::NFTExt;

use crate::{tests::mock, Collection, CollectionId, Error, Event as NFTsEvent, NFTData, NFTId};

const ALICE_NFT_ID: NFTId = 0;
const BOB_NFT_ID: NFTId = 1;
const ALICE_COLLECTION_ID: CollectionId = 0;
const BOB_COLLECTION_ID: CollectionId = 1;
const INVALID_ID: NFTId = 1001;
const PERCENT_100: Permill = Permill::from_parts(1000000);
const PERCENT_80: Permill = Permill::from_parts(800000);
const PERCENT_0: Permill = Permill::from_parts(0);

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

fn prepare_tests() {
	let alice: mock::Origin = origin(ALICE);
	let bob: mock::Origin = origin(BOB);

	//Create alice NFT.
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_100, None, false).unwrap();

	// Create alice collection.
	NFT::create_collection(alice, BoundedVec::default(), None).unwrap();

	//Create bob NFT.
	NFT::create_nft(bob.clone(), BoundedVec::default(), PERCENT_100, None, false).unwrap();

	// Create bob collection.
	NFT::create_collection(bob, BoundedVec::default(), None).unwrap();

	assert_eq!(NFT::nfts(ALICE_NFT_ID).is_some(), true);
	assert_eq!(NFT::nfts(BOB_NFT_ID).is_some(), true);

	assert_eq!(NFT::collections(ALICE_COLLECTION_ID).is_some(), true);
	assert_eq!(NFT::collections(BOB_COLLECTION_ID).is_some(), true);
}

mod create_nft {
	use super::*;

	#[test]
	fn create_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);
			let data = NFTData::new_default(ALICE, BoundedVec::default(), PERCENT_100, None, false);

			// Create NFT without a collection.
			NFT::create_nft(
				alice,
				data.offchain_data.clone(),
				data.royalty,
				data.collection_id,
				data.state.is_soulbound,
			)
			.unwrap();
			let nft_id = NFT::get_next_nft_id() - 1;

			// Final state checks.
			let nft = NFT::nfts(nft_id);
			assert_eq!(nft, Some(data.clone()));
			assert_eq!(Balances::free_balance(ALICE), alice_balance - NFT::nft_mint_fee());

			// Events checks.
			let event = NFTsEvent::NFTCreated {
				nft_id,
				owner: data.owner,
				offchain_data: data.offchain_data,
				royalty: data.royalty,
				collection_id: data.collection_id,
				is_soulbound: data.state.is_soulbound,
				mint_fee: NFT::nft_mint_fee(),
			};
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn create_nft_with_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);
			let data = NFTData::new_default(
				ALICE,
				BoundedVec::default(),
				PERCENT_100,
				Some(ALICE_COLLECTION_ID),
				false,
			);

			// Create NFT with a collection.
			NFT::create_nft(
				alice,
				data.offchain_data.clone(),
				data.royalty,
				data.collection_id,
				data.state.is_soulbound,
			)
			.unwrap();
			let nft_id = NFT::get_next_nft_id() - 1;

			// Final state checks.
			let nft = NFT::nfts(nft_id);
			assert_eq!(nft, Some(data.clone()));
			assert_eq!(Balances::free_balance(ALICE), alice_balance - NFT::nft_mint_fee());
			assert_eq!(NFT::collections(ALICE_COLLECTION_ID).unwrap().nfts.contains(&nft_id), true);

			// Events checks.
			let event = NFTsEvent::NFTCreated {
				nft_id,
				owner: data.owner,
				offchain_data: data.offchain_data,
				royalty: data.royalty,
				collection_id: data.collection_id,
				is_soulbound: data.state.is_soulbound,
				mint_fee: NFT::nft_mint_fee(),
			};
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, 1)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			// Should fail and storage should remain empty.
			let err = NFT::create_nft(alice, BoundedVec::default(), PERCENT_0, None, false);
			assert_noop!(err, BalanceError::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Try to add Alice's NFT to Bob's collection.
			let err = NFT::create_nft(
				alice,
				BoundedVec::default(),
				PERCENT_0,
				Some(BOB_COLLECTION_ID),
				false,
			);

			// Should fail because Bob is not the collection owner.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}

	#[test]
	fn collection_is_closed() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Close alice's collection.
			NFT::close_collection(alice.clone(), ALICE_COLLECTION_ID).unwrap();

			// Add an NFT to this collection.
			let err = NFT::create_nft(
				alice,
				BoundedVec::default(),
				PERCENT_0,
				Some(ALICE_COLLECTION_ID),
				false,
			);

			// Should fail because collection is close.
			assert_noop!(err, Error::<Test>::CollectionIsClosed);
		})
	}

	#[test]
	fn collection_has_reached_max() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Add CollectionSizeLimit NFTs to Alice's collection.
			for _i in 0..CollectionSizeLimit::get() {
				NFT::create_nft(
					alice.clone(),
					BoundedVec::default(),
					PERCENT_0,
					Some(ALICE_COLLECTION_ID),
					false,
				)
				.unwrap();
			}

			// Add another nft to the collection.
			let err = NFT::create_nft(
				alice,
				BoundedVec::default(),
				PERCENT_0,
				Some(ALICE_COLLECTION_ID),
				false,
			);

			// Should fail because collection has reached maximum value.
			assert_noop!(err, Error::<Test>::CollectionHasReachedLimit);
		})
	}

	#[test]
	fn collection_has_reached_limit() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			// Create a collection with 1 as limit.
			NFT::create_collection(alice.clone(), BoundedVec::default(), Some(1)).unwrap();
			let collection_id = NFT::get_next_collection_id() - 1;

			// Add nft to the collection.
			NFT::create_nft(
				alice.clone(),
				BoundedVec::default(),
				PERCENT_0,
				Some(collection_id),
				false,
			)
			.unwrap();

			// Adding another nft to the collection.
			let err = NFT::create_nft(
				alice,
				BoundedVec::default(),
				PERCENT_0,
				Some(collection_id),
				false,
			);
			// Should fail because collection has reached limit.
			assert_noop!(err, Error::<Test>::CollectionHasReachedLimit);
		})
	}

	#[test]
	fn keep_alive() {
		ExtBuilder::new_build(vec![(ALICE, 2 * NFT_MINT_FEE), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);

			// Try to create an NFT.
			let err = NFT::create_nft(alice, BoundedVec::default(), PERCENT_0, None, false);

			// Should fail because Alice's account must stay alive.
			assert_noop!(err, BalanceError::<Test>::KeepAlive);
			// Alice's balance should not have been changed
			assert_eq!(Balances::free_balance(ALICE), alice_balance);
		})
	}
}

mod burn_nft {

	use super::*;

	#[test]
	fn burn_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Burning the nft.
			let ok = NFT::burn_nft(alice, ALICE_NFT_ID);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID).is_some(), false);

			// Events checks.
			let event = NFTsEvent::NFTBurned { nft_id: ALICE_NFT_ID };
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn burn_nft_in_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let expected_collection = NFT::collections(ALICE_COLLECTION_ID).unwrap();
			// Add alice's NFT to her collection.
			NFT::add_nft_to_collection(alice.clone(), ALICE_NFT_ID, ALICE_COLLECTION_ID).unwrap();
			// Burning the nft.
			let ok = NFT::burn_nft(alice, ALICE_NFT_ID);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID).is_some(), false);
			assert_eq!(
				NFT::collections(ALICE_COLLECTION_ID).unwrap().nfts,
				expected_collection.nfts
			);

			// Events checks.
			let event = NFTsEvent::NFTBurned { nft_id: ALICE_NFT_ID };
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			// Burning an nft.
			let err = NFT::burn_nft(origin(ALICE), ALICE_NFT_ID);
			// Should fail because NFT was not created.
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Burning an nft.
			let err = NFT::burn_nft(origin(BOB), ALICE_NFT_ID);
			// Should fail because BOB is not the owner of alice's NFT.
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_burn_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set listed to true for Alice's NFT.
			let nft_state = NFTState::new(false, true, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Burning an nft.
			let err = NFT::burn_nft(origin(ALICE), ALICE_NFT_ID);
			// Should fail because NFT is listed for sale.
			assert_noop!(err, Error::<Test>::CannotBurnListedNFTs);
		})
	}

	#[test]
	fn cannot_burn_capsule_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set capsule to true for Alice's NFT.
			let nft_state = NFTState::new(true, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Burning an nft.
			let err = NFT::burn_nft(origin(ALICE), ALICE_NFT_ID);
			// Should fail because NFT is capsule.
			assert_noop!(err, Error::<Test>::CannotBurnCapsuleNFTs);
		})
	}

	#[test]
	fn cannot_burn_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set delegated to true for Alice's NFT.
			NFT::delegate_nft(origin(ALICE), ALICE_NFT_ID, Some(BOB)).unwrap();
			// Burning an nft.
			let err = NFT::burn_nft(origin(ALICE), ALICE_NFT_ID);
			// Should fail because NFT is delegated.
			assert_noop!(err, Error::<Test>::CannotBurnDelegatedNFTs);
		})
	}
}

mod transfer_nft {
	use super::*;

	#[test]
	fn transfer_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Transfer nft ownership from ALICE to BOB.
			let ok = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			assert_ok!(ok);

			// Final state checks.
			let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
			assert_eq!(nft.owner, BOB);
			assert_eq!(nft.creator, ALICE);

			// Events checks.
			let event =
				NFTsEvent::NFTTransferred { nft_id: ALICE_NFT_ID, sender: ALICE, recipient: BOB };
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Try to transfer with an unknown NFT id.
			let err = NFT::transfer_nft(alice, INVALID_ID, BOB);
			// Should fail because NFT does not exist.
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Try to transfer an unowned NFT.
			let err = NFT::transfer_nft(alice, BOB_NFT_ID, BOB);
			// Should fail because Alice is not the NFT owner.
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_transfer_nfts_to_yourself() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Try to transfer to current owner.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, ALICE);
			// Should fail because alice is owner and recipient.
			assert_noop!(err, Error::<Test>::CannotTransferNFTsToYourself);
		})
	}

	#[test]
	fn cannot_transfer_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Set NFT to listed.
			let nft_state = NFTState::new(false, true, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Try to transfer.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			// Should fail because NFT is listed.
			assert_noop!(err, Error::<Test>::CannotTransferListedNFTs);
		})
	}

	#[test]
	fn cannot_transfer_capsule_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Set NFT to capsule.
			let nft_state = NFTState::new(true, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Try to transfer.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			// Should fail because NFT is capsule.
			assert_noop!(err, Error::<Test>::CannotTransferCapsuleNFTs);
		})
	}

	#[test]
	fn cannot_transfer_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Set NFT to delegated.
			NFT::delegate_nft(origin(ALICE), ALICE_NFT_ID, Some(BOB)).unwrap();
			// Try to transfer.
			let err = NFT::transfer_nft(alice, ALICE_NFT_ID, BOB);
			// Should fail because NFT is delegated.
			assert_noop!(err, Error::<Test>::CannotTransferDelegatedNFTs);
		})
	}

	#[test]
	fn cannot_transfer_not_created_soulbound_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Create soulbound NFTs.
			let ok = NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, true);
			assert_ok!(ok);
			let nft_id = NFT::get_next_nft_id() - 1;
			let mut nft = NFT::get_nft(nft_id).unwrap();
			nft.creator = BOB;
			NFT::set_nft(nft_id, nft).unwrap();
			
			// Try to transfer.
			let err = NFT::transfer_nft(alice, nft_id, BOB);
			// Should fail because NFT is soulbound.
			assert_noop!(err, Error::<Test>::CannotTransferNotCreatedSoulboundNFTs);
		})
	}
}

mod delegate_nft {
	use super::*;

	#[test]
	fn delegate_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Expected data.
			let mut expected_data = NFT::nfts(ALICE_NFT_ID).unwrap();
			expected_data.state.is_delegated = true;
			// Delegating NFT to another account.
			let ok = NFT::delegate_nft(alice, ALICE_NFT_ID, Some(BOB));
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID), Some(expected_data));
			assert_eq!(NFT::delegated_nfts(ALICE_NFT_ID), Some(BOB));

			// Events checks.
			let event = NFTsEvent::NFTDelegated { nft_id: ALICE_NFT_ID, recipient: Some(BOB) };
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn delegate_nft_to_none() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Expected data.
			let mut expected_data = NFT::nfts(ALICE_NFT_ID).unwrap();
			expected_data.state.is_delegated = false;
			// Delegating NFT to another account.
			NFT::delegate_nft(alice.clone(), ALICE_NFT_ID, Some(BOB)).unwrap();
			// Delegate NFT to none.
			let ok = NFT::delegate_nft(alice, ALICE_NFT_ID, None);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID), Some(expected_data));
			assert_eq!(NFT::delegated_nfts(ALICE_NFT_ID), None);

			// Events checks.
			let event = NFTsEvent::NFTDelegated { nft_id: ALICE_NFT_ID, recipient: None };
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Delegating unexisting NFT.
			let err = NFT::delegate_nft(alice, INVALID_ID, None);
			// Should fail because NFT does not exist.
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Delegating unowned NFT.
			let err = NFT::delegate_nft(alice, BOB_NFT_ID, None);
			// Should fail because NFT is not owned by Alice.
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_delegate_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Set alice's NFT to listed.
			let nft_state = NFTState::new(false, true, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Delegate listed NFT.
			let err = NFT::delegate_nft(alice, ALICE_NFT_ID, None);
			// Should fail because NFT is listed.
			assert_noop!(err, Error::<Test>::CannotDelegateListedNFTs);
		})
	}

	#[test]
	fn cannot_delegate_capsule_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Set alice's NFT to capsule.
			let nft_state = NFTState::new(true, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Delegate capsule NFT.
			let err = NFT::delegate_nft(alice, ALICE_NFT_ID, None);
			// Should fail because NFT is capsule.
			assert_noop!(err, Error::<Test>::CannotDelegateCapsuleNFTs);
		})
	}
}

mod set_royalty {
	use super::*;

	#[test]
	fn set_royalty() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Expected data.
			let mut expected_data = NFT::nfts(ALICE_NFT_ID).unwrap();
			expected_data.royalty = PERCENT_80;
			// Set royalty.
			let ok = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID), Some(expected_data));

			// Events checks.
			let event = NFTsEvent::NFTRoyaltySet { nft_id: ALICE_NFT_ID, royalty: PERCENT_80 };
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Set royalty.
			let err = NFT::set_royalty(alice, INVALID_ID, PERCENT_80);
			// Should failt because NFT does not exist.
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Set royalty.
			let err = NFT::set_royalty(alice, BOB_NFT_ID, PERCENT_80);
			// Should failt because Alice is not the owner of Bob's NFT.
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn not_the_creator() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);
			// Transfer Bob's NFT to Alice.
			NFT::transfer_nft(bob, BOB_NFT_ID, ALICE).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, BOB_NFT_ID, PERCENT_80);
			// Should failt because Alice is not the creator of Bob's NFT.
			assert_noop!(err, Error::<Test>::NotTheNFTCreator);
		})
	}

	#[test]
	fn cannot_set_royalty_for_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Set Alice's NFT to listed.
			let nft_state = NFTState::new(false, true, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			// Should fail because you cannot set royalty for listed NFTs.
			assert_noop!(err, Error::<Test>::CannotSetRoyaltyForListedNFTs);
		})
	}

	#[test]
	fn cannot_set_royalty_for_capsule_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Set Alice's NFT to capsule.
			let nft_state = NFTState::new(true, false, false, false, false);
			NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			// Should fail because you cannot set royalty for capsule NFTs.
			assert_noop!(err, Error::<Test>::CannotSetRoyaltyForCapsuleNFTs);
		})
	}

	#[test]
	fn cannot_set_royalty_for_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Set Alice's NFT to delegated.
			NFT::delegate_nft(origin(ALICE), ALICE_NFT_ID, Some(BOB)).unwrap();
			// Set royalty.
			let err = NFT::set_royalty(alice, ALICE_NFT_ID, PERCENT_80);
			// Should fail because you cannot set royalty for delegated NFTs.
			assert_noop!(err, Error::<Test>::CannotSetRoyaltyForDelegatedNFTs);
		})
	}
}

mod set_nft_mint_fee {
	use super::*;

	#[test]
	fn set_nft_mint_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			// Set new mint fee.
			let ok = NFT::set_nft_mint_fee(root(), 20);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nft_mint_fee(), 20);

			// Events checks.
			let event = NFTsEvent::NFTMintFeeSet { fee: 20 };
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			// Try to change nft mint fee as not root.
			let err = NFT::set_nft_mint_fee(origin(ALICE), 20);
			// Should fail because Alice is not the root.
			assert_noop!(err, BadOrigin);
		})
	}
}

mod create_collection {
	use super::*;

	#[test]
	fn create_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let data = Collection::new(ALICE, BoundedVec::default(), Some(5));
			// Create collection.
			let ok = NFT::create_collection(alice, data.offchain_data.clone(), data.limit);
			assert_ok!(ok);
			let collection_id = NFT::get_next_collection_id() - 1;

			// Final state checks.
			let collection = NFT::collections(collection_id);
			assert_eq!(collection, Some(data.clone()));

			// Events checks.
			let event = NFTsEvent::CollectionCreated {
				collection_id,
				owner: data.owner,
				offchain_data: data.offchain_data,
				limit: data.limit,
			};
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_limit_is_too_high() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let collection_limit = CollectionSizeLimit::get() + 1;
			// Create NFT without a collection.
			let err = NFT::create_collection(alice, BoundedVec::default(), Some(collection_limit));
			// Should fail because max + 1 is not a valid limit.
			assert_noop!(err, Error::<Test>::CollectionLimitExceededMaximumAllowed);
		})
	}
}

mod burn_collection {
	use super::*;

	#[test]
	fn burn_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Burn collection.
			let ok = NFT::burn_collection(alice, ALICE_COLLECTION_ID);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::collections(ALICE_COLLECTION_ID).is_some(), false);

			// Events checks.
			let event = NFTsEvent::CollectionBurned { collection_id: ALICE_COLLECTION_ID };
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Burn invalid collection.
			let err = NFT::burn_collection(alice, INVALID_ID);
			// Should fail because collection does not exist.
			assert_noop!(err, Error::<Test>::CollectionNotFound);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Burn Bob's collection from Alice's account.
			let err = NFT::burn_collection(alice, BOB_COLLECTION_ID);
			// Should fail because Alice is not the collection owner.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}

	#[test]
	fn collection_is_not_empty() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Add Alice's NFT to her collection.
			NFT::add_nft_to_collection(alice.clone(), ALICE_NFT_ID, ALICE_COLLECTION_ID).unwrap();
			// Burn non empty collection.
			let err = NFT::burn_collection(alice, ALICE_COLLECTION_ID);
			// Should fail because collection is not empty.
			assert_noop!(err, Error::<Test>::CollectionIsNotEmpty);
		})
	}
}

mod close_collection {
	use super::*;

	#[test]
	fn close_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Close collection.
			let ok = NFT::close_collection(alice, ALICE_COLLECTION_ID);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::collections(ALICE_COLLECTION_ID).unwrap().is_closed, true);

			// Events checks.
			let event = NFTsEvent::CollectionClosed { collection_id: ALICE_COLLECTION_ID };
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Close invalid collection.
			let err = NFT::close_collection(alice, INVALID_ID);
			// Should fail because collection does not exist.
			assert_noop!(err, Error::<Test>::CollectionNotFound);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Close invalid collection.
			let err = NFT::close_collection(alice, BOB_COLLECTION_ID);
			// Should fail because Alice is not the owner of the collection.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}
}

mod limit_collection {
	use super::*;

	#[test]
	fn limit_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Limit collection.
			let ok = NFT::limit_collection(alice, ALICE_COLLECTION_ID, 1);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::collections(ALICE_COLLECTION_ID).unwrap().limit, Some(1));

			// Events checks.
			let event =
				NFTsEvent::CollectionLimited { collection_id: ALICE_COLLECTION_ID, limit: 1 };
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Limit invalid collection.
			let err = NFT::limit_collection(alice, INVALID_ID, 1);
			// Should fail because the collection does not exist.
			assert_noop!(err, Error::<Test>::CollectionNotFound);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Limit unowned collection.
			let err = NFT::limit_collection(alice, BOB_COLLECTION_ID, 1);
			// Should fail because Alice is not the collection owner.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}

	#[test]
	fn collection_limit_already_set() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Limit once.
			let ok = NFT::limit_collection(alice.clone(), ALICE_COLLECTION_ID, 1);
			assert_ok!(ok);
			// Limit again.
			let err = NFT::limit_collection(alice, ALICE_COLLECTION_ID, 2);
			// Should fail because the collection limit is already set.
			assert_noop!(err, Error::<Test>::CollectionLimitAlreadySet);
		})
	}

	#[test]
	fn collection_is_closed() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Close collection.
			let ok = NFT::close_collection(alice.clone(), ALICE_COLLECTION_ID);
			assert_ok!(ok);
			// Limit.
			let err = NFT::limit_collection(alice, ALICE_COLLECTION_ID, 1);
			// Should fail because the collection is closed.
			assert_noop!(err, Error::<Test>::CollectionIsClosed);
		})
	}

	#[test]
	fn collection_nfts_number_greater_than_limit() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Add Alice's NFT to her collection.
			let ok = NFT::add_nft_to_collection(alice.clone(), ALICE_NFT_ID, ALICE_COLLECTION_ID);
			assert_ok!(ok);
			// Create a second nft for alice.
			let ok = NFT::create_nft(
				alice.clone(),
				BoundedVec::default(),
				PERCENT_100,
				Some(ALICE_COLLECTION_ID),
				false,
			);
			assert_ok!(ok);
			// Limit collection with value 1.
			let err = NFT::limit_collection(alice, ALICE_COLLECTION_ID, 1);
			// Should fail because the selected limit is lower than the number of NFTs currently in
			// the collection.
			assert_noop!(err, Error::<Test>::CollectionHasTooManyNFTs);
		})
	}

	#[test]
	fn collection_limit_is_too_high() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let limit = CollectionSizeLimit::get() + 1;
			// Limit again.
			let err = NFT::limit_collection(alice, ALICE_COLLECTION_ID, limit);
			// Should fail because the selected limit is greater than the size limit from config.
			assert_noop!(err, Error::<Test>::CollectionLimitExceededMaximumAllowed);
		})
	}
}

mod add_nft_to_collection {
	use super::*;

	#[test]
	fn add_nft_to_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let mut expected_collection = NFT::collections(ALICE_COLLECTION_ID).unwrap();
			expected_collection.nfts.try_push(ALICE_COLLECTION_ID).unwrap();
			// Add Alice's NFT to her collection.
			let ok = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, ALICE_COLLECTION_ID);
			assert_ok!(ok);

			// Final state checks.
			assert_eq!(NFT::nfts(ALICE_NFT_ID).unwrap().collection_id, Some(ALICE_COLLECTION_ID));
			assert_eq!(NFT::collections(ALICE_COLLECTION_ID).unwrap(), expected_collection);

			// Events checks.
			let event = NFTsEvent::NFTAddedToCollection {
				nft_id: ALICE_NFT_ID,
				collection_id: ALICE_COLLECTION_ID,
			};
			let event = Event::NFT(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn collection_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Add Alice's NFT to invalid collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, INVALID_ID);
			// Should fail because collection does not exist.
			assert_noop!(err, Error::<Test>::CollectionNotFound);
		})
	}

	#[test]
	fn not_the_collection_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Add Alice's NFT to Bob's collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, BOB_COLLECTION_ID);
			// Should fail because collection belong to Bob.
			assert_noop!(err, Error::<Test>::NotTheCollectionOwner);
		})
	}

	#[test]
	fn collection_is_closed() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Close Alice's collection.
			NFT::close_collection(alice.clone(), ALICE_COLLECTION_ID).unwrap();
			// Add Alice's NFT to Bob's collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, ALICE_COLLECTION_ID);
			// Should fail because collection belong to Bob.
			assert_noop!(err, Error::<Test>::CollectionIsClosed);
		})
	}

	#[test]
	fn collection_has_reached_max() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Add CollectionSizeLimit NFTs to Alice's collection.
			for _i in 0..CollectionSizeLimit::get() {
				NFT::create_nft(
					alice.clone(),
					BoundedVec::default(),
					PERCENT_0,
					Some(ALICE_COLLECTION_ID),
					false,
				)
				.unwrap();
			}
			// Add another nft to the collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, ALICE_COLLECTION_ID);
			// Should fail because collection has reached maximum value.
			assert_noop!(err, Error::<Test>::CollectionHasReachedLimit);
		})
	}

	#[test]
	fn collection_has_reached_limit() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let limit = 5;
			// Set limit to Alice's NFT.
			NFT::limit_collection(alice.clone(), ALICE_COLLECTION_ID, limit).unwrap();
			// Add CollectionSizeLimit NFTs to Alice's collection.
			for _i in 0..limit {
				NFT::create_nft(
					alice.clone(),
					BoundedVec::default(),
					PERCENT_0,
					Some(ALICE_COLLECTION_ID),
					false,
				)
				.unwrap();
			}
			// Add another nft to the collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, ALICE_COLLECTION_ID);
			// Should fail because collection has reached limit value.
			assert_noop!(err, Error::<Test>::CollectionHasReachedLimit);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Add invalid NFT to the collection.
			let err = NFT::add_nft_to_collection(alice, INVALID_ID, ALICE_COLLECTION_ID);
			// Should fail because NFT does not exist.
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Add unowned NFT in collection.
			let err = NFT::add_nft_to_collection(alice, BOB_NFT_ID, ALICE_COLLECTION_ID);
			// Should fail because the NFT does not belong to Alice.
			assert_noop!(err, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn nft_belong_to_a_collection() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			// Add NFT in collection.
			let ok = NFT::add_nft_to_collection(alice.clone(), ALICE_NFT_ID, ALICE_COLLECTION_ID);
			assert_ok!(ok);
			// Create new collection.
			let ok = NFT::create_collection(alice.clone(), BoundedVec::default(), None);
			assert_ok!(ok);
			let collection_id = NFT::get_next_collection_id() - 1;
			// Add NFT to the new collection.
			let err = NFT::add_nft_to_collection(alice, ALICE_NFT_ID, collection_id);
			// Should fail because the NFT already belong to an other collection.
			assert_noop!(err, Error::<Test>::NFTBelongToACollection);
		})
	}
}
