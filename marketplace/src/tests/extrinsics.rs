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
use primitives::{
	nfts::{CollectionId, NFTId, NFTState},
	ConfigOp,
};
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::NFTExt;

use crate::{
	tests::mock, CompoundFee, Error, Event as MarketplaceEvent, MarketplaceData, MarketplaceId,
	MarketplaceType, Sale,
};

const ALICE_NFT_ID: NFTId = 0;
const ALICE_COLLECTION_ID: CollectionId = 0;
const ALICE_MARKETPLACE_ID: MarketplaceId = 0;
const BOB_NFT_ID: NFTId = 1;
const BOB_COLLECTION_ID: CollectionId = 1;
const BOB_MARKETPLACE_ID: MarketplaceId = 1;
const CHARLIE_MARKETPLACE_ID: MarketplaceId = 2;
const INVALID_NFT_ID: NFTId = 1001;
const INVALID_MARKETPLACE_ID: NFTId = 1001;
const PERCENT_100: Permill = Permill::from_parts(1000000);
const PERCENT_80: Permill = Permill::from_parts(800000);
const PERCENT_50: Permill = Permill::from_parts(500000);
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
	let charlie: mock::Origin = origin(CHARLIE);

	//Create alice NFT.
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();

	// Create alice collection.
	NFT::create_collection(alice.clone(), BoundedVec::default(), None).unwrap();

	// Create alice marketplace.
	Marketplace::create_marketplace(alice, MarketplaceType::Public).unwrap();

	//Create bob NFT.
	NFT::create_nft(bob.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();

	// Create bob collection.
	NFT::create_collection(bob.clone(), BoundedVec::default(), None).unwrap();

	// Create bob marketplace.
	Marketplace::create_marketplace(bob, MarketplaceType::Public).unwrap();

	// Create charlie marketplace.
	Marketplace::create_marketplace(charlie, MarketplaceType::Public).unwrap();

	assert_eq!(NFT::nfts(ALICE_NFT_ID).is_some(), true);
	assert_eq!(NFT::nfts(BOB_NFT_ID).is_some(), true);

	assert_eq!(NFT::collections(ALICE_COLLECTION_ID).is_some(), true);
	assert_eq!(NFT::collections(BOB_COLLECTION_ID).is_some(), true);

	assert_eq!(Marketplace::marketplaces(ALICE_MARKETPLACE_ID).is_some(), true);
	assert_eq!(Marketplace::marketplaces(BOB_MARKETPLACE_ID).is_some(), true);
	assert_eq!(Marketplace::marketplaces(CHARLIE_MARKETPLACE_ID).is_some(), true);
}

mod create_marketplace {
	use super::*;

	#[test]
	fn create_marketplace() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);
			let data = MarketplaceData::new(ALICE, MarketplaceType::Public, None, None, None, None);

			// Create a marketplace.
			Marketplace::create_marketplace(alice, data.kind).unwrap();
			let marketplace_id = Marketplace::get_next_marketplace_id() - 1;

			// Final state checks.
			let marketplace = Marketplace::marketplaces(marketplace_id);
			assert_eq!(marketplace, Some(data.clone()));
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance - Marketplace::marketplace_mint_fee()
			);

			// Events checks.
			let event = MarketplaceEvent::MarketplaceCreated {
				marketplace_id,
				owner: data.owner,
				kind: data.kind,
			};
			let event = Event::Marketplace(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, 1)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			// Should fail and storage should remain empty.
			let err = Marketplace::create_marketplace(alice, MarketplaceType::Public);
			assert_noop!(err, BalanceError::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn keep_alive() {
		ExtBuilder::new_build(vec![(ALICE, MARKETPLACE_MINT_FEE)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let alice_balance = Balances::free_balance(ALICE);
			// Should fail and storage should remain empty.
			let err = Marketplace::create_marketplace(alice, MarketplaceType::Public);
			assert_noop!(err, BalanceError::<Test>::KeepAlive);
			assert_eq!(Balances::free_balance(ALICE), alice_balance);
		})
	}
}

mod set_marketplace_owner {
	use super::*;

	#[test]
	fn set_marketplace_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// transfer a marketplace.
				Marketplace::set_marketplace_owner(alice, ALICE_MARKETPLACE_ID, BOB).unwrap();

				// Final state checks.
				let marketplace = Marketplace::marketplaces(ALICE_MARKETPLACE_ID).unwrap();
				assert_eq!(marketplace.owner, BOB);

				// Events checks.
				let event = MarketplaceEvent::MarketplaceOwnerSet {
					marketplace_id: ALICE_MARKETPLACE_ID,
					owner: BOB,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// transfer a marketplace with invalid marketplace id.
				let err = Marketplace::set_marketplace_owner(alice, INVALID_MARKETPLACE_ID, BOB);
				assert_noop!(err, Error::<Test>::MarketplaceNotFound);
			},
		)
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// transfer a marketplace with not owned marketplace id.
				let err = Marketplace::set_marketplace_owner(alice, BOB_MARKETPLACE_ID, BOB);
				assert_noop!(err, Error::<Test>::NotTheMarketplaceOwner);
			},
		)
	}

	#[test]
	fn cannot_transfer_marketplace_to_yourself() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// transfer a marketplace with already owned marketplace id.
				let err = Marketplace::set_marketplace_owner(alice, ALICE_MARKETPLACE_ID, ALICE);
				assert_noop!(err, Error::<Test>::CannotTransferMarketplaceToYourself);
			},
		)
	}
}

mod set_marketplace_kind {
	use super::*;

	#[test]
	fn set_marketplace_kind() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// set marketplace kind.
				Marketplace::set_marketplace_kind(
					alice,
					ALICE_MARKETPLACE_ID,
					MarketplaceType::Private,
				)
				.unwrap();

				// Final state checks.
				let marketplace = Marketplace::marketplaces(ALICE_MARKETPLACE_ID).unwrap();
				assert_eq!(marketplace.kind, MarketplaceType::Private);

				// Events checks.
				let event = MarketplaceEvent::MarketplaceKindSet {
					marketplace_id: ALICE_MARKETPLACE_ID,
					kind: MarketplaceType::Private,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// set marketplace kind for invalid marketplace.
				let err = Marketplace::set_marketplace_kind(
					alice,
					INVALID_MARKETPLACE_ID,
					MarketplaceType::Private,
				);

				assert_noop!(err, Error::<Test>::MarketplaceNotFound);
			},
		)
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// set marketplace kind for not owned marketplace.
				let err = Marketplace::set_marketplace_kind(
					alice,
					BOB_MARKETPLACE_ID,
					MarketplaceType::Private,
				);

				assert_noop!(err, Error::<Test>::NotTheMarketplaceOwner);
			},
		)
	}
}

mod set_marketplace_configuration {
	use super::*;

	#[test]
	fn set_marketplace_configuration() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let data = MarketplaceData::new(
					ALICE,
					MarketplaceType::Public,
					Some(CompoundFee::Percentage(PERCENT_100)),
					Some(CompoundFee::Percentage(PERCENT_100)),
					Some(BoundedVec::try_from(vec![ALICE, BOB]).unwrap()),
					Some(BoundedVec::try_from(vec![1]).unwrap()),
				);
				let data_none =
					MarketplaceData::new(ALICE, MarketplaceType::Public, None, None, None, None);

				// set marketplace configuration, all set.
				Marketplace::set_marketplace_configuration(
					alice.clone(),
					ALICE_MARKETPLACE_ID,
					ConfigOp::Set(data.commission_fee.unwrap()),
					ConfigOp::Set(data.listing_fee.unwrap()),
					ConfigOp::Set(data.account_list.clone().unwrap()),
					ConfigOp::Set(data.offchain_data.clone().unwrap()),
				)
				.unwrap();

				// State checks.
				let marketplace = Marketplace::marketplaces(ALICE_MARKETPLACE_ID).unwrap();
				assert_eq!(marketplace, data.clone());

				// set marketplace configuration, all noop.
				Marketplace::set_marketplace_configuration(
					alice.clone(),
					ALICE_MARKETPLACE_ID,
					ConfigOp::Noop,
					ConfigOp::Noop,
					ConfigOp::Noop,
					ConfigOp::Noop,
				)
				.unwrap();

				// State checks.
				let marketplace = Marketplace::marketplaces(ALICE_MARKETPLACE_ID).unwrap();
				assert_eq!(marketplace, data.clone());

				// set marketplace configuration, all remove.
				Marketplace::set_marketplace_configuration(
					alice.clone(),
					ALICE_MARKETPLACE_ID,
					ConfigOp::Remove,
					ConfigOp::Remove,
					ConfigOp::Remove,
					ConfigOp::Remove,
				)
				.unwrap();

				// State checks.
				let marketplace = Marketplace::marketplaces(ALICE_MARKETPLACE_ID).unwrap();
				assert_eq!(marketplace, data_none.clone());

				// Events checks.
				let event = MarketplaceEvent::MarketplaceConfigSet {
					marketplace_id: ALICE_MARKETPLACE_ID,
					commission_fee: ConfigOp::Remove,
					listing_fee: ConfigOp::Remove,
					account_list: ConfigOp::Remove,
					offchain_data: ConfigOp::Remove,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// set marketplace configuration, all remove.
				let err = Marketplace::set_marketplace_configuration(
					alice.clone(),
					INVALID_MARKETPLACE_ID,
					ConfigOp::Remove,
					ConfigOp::Remove,
					ConfigOp::Remove,
					ConfigOp::Remove,
				);

				assert_noop!(err, Error::<Test>::MarketplaceNotFound);
			},
		)
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// set marketplace configuration, all remove.
				let err = Marketplace::set_marketplace_configuration(
					alice.clone(),
					BOB_MARKETPLACE_ID,
					ConfigOp::Remove,
					ConfigOp::Remove,
					ConfigOp::Remove,
					ConfigOp::Remove,
				);

				assert_noop!(err, Error::<Test>::NotTheMarketplaceOwner);
			},
		)
	}
}

mod set_marketplace_mint_fee {
	use super::*;

	#[test]
	fn set_marketplace_mint_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let old_mint_fee = Marketplace::marketplace_mint_fee();
			let new_mint_fee = 123u64;
			assert_eq!(Marketplace::marketplace_mint_fee(), old_mint_fee);

			let ok = Marketplace::set_marketplace_mint_fee(root(), new_mint_fee);
			assert_ok!(ok);
			assert_eq!(Marketplace::marketplace_mint_fee(), new_mint_fee);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let new_mint_fee = 123u64;
			let err = Marketplace::set_marketplace_mint_fee(alice, new_mint_fee);
			assert_noop!(err, BadOrigin);
		})
	}
}

mod list_nft {
	use super::*;

	#[test]
	fn list_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let marketplace = Marketplace::marketplaces(ALICE_MARKETPLACE_ID).unwrap();
				let data = Sale::new(ALICE, ALICE_MARKETPLACE_ID, 10, marketplace.commission_fee);

				// List NFT.
				Marketplace::list_nft(alice, ALICE_NFT_ID, data.marketplace_id, data.price)
					.unwrap();

				// Final state checks.
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID).unwrap();
				assert_eq!(sale, data);

				// Events checks.
				let event = MarketplaceEvent::NFTListed {
					marketplace_id: data.marketplace_id,
					nft_id: ALICE_NFT_ID,
					commission_fee: data.commission_fee,
					price: data.price,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn list_nft_listing_fee_flat() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let bob: mock::Origin = origin(BOB);
				let alice_balance = Balances::free_balance(ALICE);
				let new_listing_fee = 10;

				// Set listing fee percentage.
				Marketplace::set_marketplace_configuration(
					bob.clone(),
					BOB_MARKETPLACE_ID,
					ConfigOp::Noop,
					ConfigOp::Set(CompoundFee::Flat(new_listing_fee)),
					ConfigOp::Noop,
					ConfigOp::Noop,
				)
				.unwrap();

				let marketplace = Marketplace::marketplaces(BOB_MARKETPLACE_ID).unwrap();
				let data = Sale::new(ALICE, BOB_MARKETPLACE_ID, 10, marketplace.commission_fee);

				// List nft.
				Marketplace::list_nft(alice, ALICE_NFT_ID, data.marketplace_id, data.price)
					.unwrap();

				// Final state checks.
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID).unwrap();
				assert_eq!(sale, data);
				assert_eq!(Balances::free_balance(ALICE), alice_balance - new_listing_fee);

				// Events checks.
				let event = MarketplaceEvent::NFTListed {
					marketplace_id: data.marketplace_id,
					nft_id: ALICE_NFT_ID,
					commission_fee: data.commission_fee,
					price: data.price,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn list_nft_listing_fee_percentage() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let bob: mock::Origin = origin(BOB);
				let alice_balance = Balances::free_balance(ALICE);
				let new_listing_fee = PERCENT_80;

				// Set listing fee percentage.
				Marketplace::set_marketplace_configuration(
					bob.clone(),
					BOB_MARKETPLACE_ID,
					ConfigOp::Noop,
					ConfigOp::Set(CompoundFee::Percentage(new_listing_fee)),
					ConfigOp::Noop,
					ConfigOp::Noop,
				)
				.unwrap();

				let marketplace = Marketplace::marketplaces(BOB_MARKETPLACE_ID).unwrap();
				let data = Sale::new(ALICE, BOB_MARKETPLACE_ID, 10, marketplace.commission_fee);

				// List nft.
				Marketplace::list_nft(alice, ALICE_NFT_ID, data.marketplace_id, data.price)
					.unwrap();

				// Final state checks.
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID).unwrap();
				assert_eq!(sale, data);
				assert_eq!(
					Balances::free_balance(ALICE),
					alice_balance - (new_listing_fee * data.price)
				);

				// Events checks.
				let event = MarketplaceEvent::NFTListed {
					marketplace_id: data.marketplace_id,
					nft_id: ALICE_NFT_ID,
					commission_fee: data.commission_fee,
					price: data.price,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn keep_alive() {
		let new_listing_fee = 10;
		ExtBuilder::new_build(vec![
			(ALICE, MARKETPLACE_MINT_FEE + NFT_MINT_FEE + new_listing_fee),
			(BOB, 1000),
			(CHARLIE, 1000),
		])
		.execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);
			let bob: mock::Origin = origin(BOB);
			let alice_balance = Balances::free_balance(ALICE);

			// Set listing fee.
			Marketplace::set_marketplace_configuration(
				bob.clone(),
				BOB_MARKETPLACE_ID,
				ConfigOp::Noop,
				ConfigOp::Set(CompoundFee::Flat(new_listing_fee)),
				ConfigOp::Noop,
				ConfigOp::Noop,
			)
			.unwrap();

			let err = Marketplace::list_nft(alice, ALICE_NFT_ID, BOB_MARKETPLACE_ID, 10);
			assert_noop!(err, BalanceError::<Test>::KeepAlive);
			assert_eq!(Balances::free_balance(ALICE), alice_balance);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// List invalid nft.
				let err = Marketplace::list_nft(alice, INVALID_NFT_ID, ALICE_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::NFTNotFound);
			},
		)
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Try to list unowned nft.
				let err = Marketplace::list_nft(alice, BOB_NFT_ID, ALICE_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::NotTheNFTOwner);
			},
		)
	}

	#[test]
	fn cannot_list_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// List twice the same nft.
				Marketplace::list_nft(alice.clone(), ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10)
					.unwrap();
				let err = Marketplace::list_nft(alice, ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::CannotListListedNFTs);
			},
		)
	}

	#[test]
	fn cannot_list_capsule_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Set capsule to true for Alice's NFT.
				let nft_state = NFTState::new(true, false, false, false, false, false);
				NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

				let err = Marketplace::list_nft(alice, ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::CannotListCapsuleNFTs);
			},
		)
	}

	#[test]
	fn cannot_list_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Set delegated to true for Alice's NFT.
				let nft_state = NFTState::new(false, false, false, true, false, false);
				NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

				let err = Marketplace::list_nft(alice, ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::CannotListDelegatedNFTs);
			},
		)
	}

	#[test]
	fn cannot_list_soulbound_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Set soulbound to true for Alice's NFT.
				let mut nft = NFT::get_nft(ALICE_NFT_ID).unwrap();
				nft.state.is_soulbound = true;
				nft.creator = BOB;
				NFT::set_nft(ALICE_NFT_ID, nft).unwrap();

				let err = Marketplace::list_nft(alice, ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::CannotListNotCreatedSoulboundNFTs);
			},
		)
	}

	#[test]
	fn cannot_list_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Set capsule to true for Alice's NFT.
				let nft_state = NFTState::new(false, false, false, false, false, true);
				NFT::set_nft_state(ALICE_NFT_ID, nft_state).unwrap();

				let err = Marketplace::list_nft(alice, ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::CannotListRentedNFTs);
			},
		)
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// List on invalid marketplace.
				let err = Marketplace::list_nft(alice, ALICE_NFT_ID, INVALID_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::MarketplaceNotFound);
			},
		)
	}

	#[test]
	fn account_not_allowed_to_list_banned() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Set public marketplace account list (ban list) with alice's account.
				Marketplace::set_marketplace_configuration(
					alice.clone(),
					ALICE_MARKETPLACE_ID,
					ConfigOp::Noop,
					ConfigOp::Noop,
					ConfigOp::Set(BoundedVec::try_from(vec![ALICE]).unwrap()),
					ConfigOp::Noop,
				)
				.unwrap();

				let err = Marketplace::list_nft(alice, ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::AccountNotAllowedToList);
			},
		)
	}

	#[test]
	fn account_not_allowed_to_list_not_authorized() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Set marketplace private (without alice's account in account list / allow list).
				Marketplace::set_marketplace_kind(
					alice.clone(),
					ALICE_MARKETPLACE_ID,
					MarketplaceType::Private,
				)
				.unwrap();

				let err = Marketplace::list_nft(alice, ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::AccountNotAllowedToList);
			},
		)
	}

	#[test]
	fn price_too_low_for_commission_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Set high commission fee.
				Marketplace::set_marketplace_configuration(
					alice.clone(),
					ALICE_MARKETPLACE_ID,
					ConfigOp::Set(CompoundFee::Flat(10_000)),
					ConfigOp::Noop,
					ConfigOp::Noop,
					ConfigOp::Noop,
				)
				.unwrap();

				let err = Marketplace::list_nft(alice, ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10);
				assert_noop!(err, Error::<Test>::PriceCannotCoverMarketplaceFee);
			},
		)
	}
}

mod unlist_nft {
	use super::*;

	#[test]
	fn unlist_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let marketplace = Marketplace::marketplaces(ALICE_MARKETPLACE_ID).unwrap();
				let data = Sale::new(ALICE, ALICE_MARKETPLACE_ID, 10, marketplace.commission_fee);

				// List NFT.
				Marketplace::list_nft(alice.clone(), ALICE_NFT_ID, data.marketplace_id, data.price)
					.unwrap();

				// Unlist NFT.
				Marketplace::unlist_nft(alice, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID);
				assert_eq!(sale, None);

				// Events checks.
				let event = MarketplaceEvent::NFTUnlisted { nft_id: ALICE_NFT_ID };
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Unlist invalid NFT.
				let err = Marketplace::unlist_nft(alice, INVALID_NFT_ID);
				assert_noop!(err, Error::<Test>::NFTNotFound);
			},
		)
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let bob: mock::Origin = origin(BOB);

				// List bob's nft.
				Marketplace::list_nft(bob.clone(), BOB_NFT_ID, ALICE_MARKETPLACE_ID, 0).unwrap();

				let err = Marketplace::unlist_nft(alice, BOB_NFT_ID);
				assert_noop!(err, Error::<Test>::NotTheNFTOwner);
			},
		)
	}

	#[test]
	fn nft_not_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Unlist an NFT not for sale.
				let err = Marketplace::unlist_nft(alice, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::NFTNotForSale);
			},
		)
	}
}

mod buy_nft {
	use super::*;

	#[test]
	fn buy_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let alice_balance = Balances::free_balance(ALICE);
				let bob: mock::Origin = origin(BOB);
				let bob_balance = Balances::free_balance(BOB);

				// List NFT.
				Marketplace::list_nft(alice, ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10).unwrap();

				// Buy NFT.
				Marketplace::buy_nft(bob, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID);
				assert_eq!(sale, None);
				assert_eq!(nft.owner, BOB);
				assert_eq!(Balances::free_balance(BOB), bob_balance - 10);
				assert_eq!(Balances::free_balance(ALICE), alice_balance + 10);

				// Events checks.
				let event = MarketplaceEvent::NFTSold {
					nft_id: ALICE_NFT_ID,
					marketplace_id: ALICE_MARKETPLACE_ID,
					buyer: BOB,
					listed_price: 10,
					marketplace_cut: 0,
					royalty_cut: 0,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn buy_nft_flat_commission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let alice_balance = Balances::free_balance(ALICE);
				let bob: mock::Origin = origin(BOB);
				let bob_balance = Balances::free_balance(BOB);
				let charlie: mock::Origin = origin(CHARLIE);
				let charlie_balance = Balances::free_balance(CHARLIE);

				// Set marketplace commission fee.
				Marketplace::set_marketplace_configuration(
					charlie,
					CHARLIE_MARKETPLACE_ID,
					ConfigOp::Set(CompoundFee::Flat(5)),
					ConfigOp::Noop,
					ConfigOp::Noop,
					ConfigOp::Noop,
				)
				.unwrap();

				// List NFT.
				Marketplace::list_nft(alice, ALICE_NFT_ID, CHARLIE_MARKETPLACE_ID, 10).unwrap();

				// Buy NFT.
				Marketplace::buy_nft(bob, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID);
				assert_eq!(sale, None);
				assert_eq!(nft.owner, BOB);
				// Buyer check.
				assert_eq!(Balances::free_balance(BOB), bob_balance - 10);
				// Seller check.
				assert_eq!(Balances::free_balance(ALICE), alice_balance + 5);
				// Marketplace owner check.
				assert_eq!(Balances::free_balance(CHARLIE), charlie_balance + 5);

				// Events checks.
				let event = MarketplaceEvent::NFTSold {
					nft_id: ALICE_NFT_ID,
					marketplace_id: CHARLIE_MARKETPLACE_ID,
					buyer: BOB,
					listed_price: 10,
					marketplace_cut: 5,
					royalty_cut: 0,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn buy_nft_percentage_commission() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let alice_balance = Balances::free_balance(ALICE);
				let bob: mock::Origin = origin(BOB);
				let bob_balance = Balances::free_balance(BOB);
				let charlie: mock::Origin = origin(CHARLIE);
				let charlie_balance = Balances::free_balance(CHARLIE);

				// Set marketplace commission fee.
				Marketplace::set_marketplace_configuration(
					charlie,
					CHARLIE_MARKETPLACE_ID,
					ConfigOp::Set(CompoundFee::Percentage(PERCENT_80)),
					ConfigOp::Noop,
					ConfigOp::Noop,
					ConfigOp::Noop,
				)
				.unwrap();

				// List NFT.
				Marketplace::list_nft(alice, ALICE_NFT_ID, CHARLIE_MARKETPLACE_ID, 10).unwrap();

				// Buy NFT.
				Marketplace::buy_nft(bob, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID);
				assert_eq!(sale, None);
				assert_eq!(nft.owner, BOB);
				// Buyer check.
				assert_eq!(Balances::free_balance(BOB), bob_balance - 10);
				// Seller check.
				assert_eq!(Balances::free_balance(ALICE), alice_balance + 2);
				// Marketplace owner check.
				assert_eq!(Balances::free_balance(CHARLIE), charlie_balance + 8);

				// Events checks.
				let event = MarketplaceEvent::NFTSold {
					nft_id: ALICE_NFT_ID,
					marketplace_id: CHARLIE_MARKETPLACE_ID,
					buyer: BOB,
					listed_price: 10,
					marketplace_cut: 8,
					royalty_cut: 0,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn buy_nft_royalty() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let alice_balance = Balances::free_balance(ALICE);
				let bob: mock::Origin = origin(BOB);
				let bob_balance = Balances::free_balance(BOB);
				let charlie: mock::Origin = origin(CHARLIE);
				let charlie_balance = Balances::free_balance(CHARLIE);

				// Set the royalty of alice's NFT.
				NFT::set_royalty(alice.clone(), ALICE_NFT_ID, PERCENT_80).unwrap();

				// Transfer the NFT to charlie.
				NFT::transfer_nft(alice, ALICE_NFT_ID, CHARLIE).unwrap();

				// List NFT.
				Marketplace::list_nft(charlie, ALICE_NFT_ID, BOB_MARKETPLACE_ID, 10).unwrap();

				// Buy NFT.
				Marketplace::buy_nft(bob, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID);
				assert_eq!(sale, None);
				assert_eq!(nft.owner, BOB);
				// Buyer check.
				assert_eq!(Balances::free_balance(BOB), bob_balance - 10);
				// Creator check.
				assert_eq!(Balances::free_balance(ALICE), alice_balance + 8);
				// Seller check.
				assert_eq!(Balances::free_balance(CHARLIE), charlie_balance + 2);

				// Events checks.
				let event = MarketplaceEvent::NFTSold {
					nft_id: ALICE_NFT_ID,
					marketplace_id: BOB_MARKETPLACE_ID,
					buyer: BOB,
					listed_price: 10,
					marketplace_cut: 0,
					royalty_cut: 8,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn buy_nft_flat_commission_and_royalty() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let alice_balance = Balances::free_balance(ALICE);
				let bob: mock::Origin = origin(BOB);
				let bob_balance = Balances::free_balance(BOB);
				let charlie: mock::Origin = origin(CHARLIE);
				let charlie_balance = Balances::free_balance(CHARLIE);
				let dave: mock::Origin = origin(DAVE);
				let dave_balance = Balances::free_balance(DAVE);

				// Set marketplace commission fee.
				Marketplace::set_marketplace_configuration(
					charlie,
					CHARLIE_MARKETPLACE_ID,
					ConfigOp::Set(CompoundFee::Flat(40)),
					ConfigOp::Noop,
					ConfigOp::Noop,
					ConfigOp::Noop,
				)
				.unwrap();

				// Set the royalty of alice's NFT.
				NFT::set_royalty(alice.clone(), ALICE_NFT_ID, PERCENT_80).unwrap();

				// Transfer the NFT to dave.
				NFT::transfer_nft(alice, ALICE_NFT_ID, DAVE).unwrap();

				// List NFT.
				Marketplace::list_nft(dave, ALICE_NFT_ID, CHARLIE_MARKETPLACE_ID, 100).unwrap();

				// Buy NFT
				Marketplace::buy_nft(bob, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID);
				assert_eq!(sale, None);
				assert_eq!(nft.owner, BOB);
				// Buyer check
				assert_eq!(Balances::free_balance(BOB), bob_balance - 100);
				// Marketplace owner check.
				assert_eq!(Balances::free_balance(CHARLIE), charlie_balance + 40);
				// Royalty check.
				assert_eq!(Balances::free_balance(ALICE), alice_balance + 48);
				// Seller check.
				assert_eq!(Balances::free_balance(DAVE), dave_balance + 12);

				// Events checks.
				let event = MarketplaceEvent::NFTSold {
					nft_id: ALICE_NFT_ID,
					marketplace_id: CHARLIE_MARKETPLACE_ID,
					buyer: BOB,
					listed_price: 100,
					marketplace_cut: 40,
					royalty_cut: 48,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn buy_nft_percentage_commission_and_royalty() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let alice_balance = Balances::free_balance(ALICE);
				let bob: mock::Origin = origin(BOB);
				let bob_balance = Balances::free_balance(BOB);
				let charlie: mock::Origin = origin(CHARLIE);
				let charlie_balance = Balances::free_balance(CHARLIE);
				let dave: mock::Origin = origin(DAVE);
				let dave_balance = Balances::free_balance(DAVE);

				// Set marketplace commission fee.
				Marketplace::set_marketplace_configuration(
					charlie,
					CHARLIE_MARKETPLACE_ID,
					ConfigOp::Set(CompoundFee::Percentage(PERCENT_50)),
					ConfigOp::Noop,
					ConfigOp::Noop,
					ConfigOp::Noop,
				)
				.unwrap();

				// Set the royalty of alice's NFT.
				NFT::set_royalty(alice.clone(), ALICE_NFT_ID, PERCENT_80).unwrap();

				// Transfer the NFT to dave.
				NFT::transfer_nft(alice, ALICE_NFT_ID, DAVE).unwrap();

				// List NFT.
				Marketplace::list_nft(dave, ALICE_NFT_ID, CHARLIE_MARKETPLACE_ID, 100).unwrap();

				// Buy NFT.
				Marketplace::buy_nft(bob, ALICE_NFT_ID).unwrap();

				// Final state checks.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID);
				assert_eq!(sale, None);
				assert_eq!(nft.owner, BOB);
				// Buyer check.
				assert_eq!(Balances::free_balance(BOB), bob_balance - 100);
				// Marketplace owner check.
				assert_eq!(Balances::free_balance(CHARLIE), charlie_balance + 50);
				// Royalty check.
				assert_eq!(Balances::free_balance(ALICE), alice_balance + 40);
				// Seller check.
				assert_eq!(Balances::free_balance(DAVE), dave_balance + 10);

				// Events checks.
				let event = MarketplaceEvent::NFTSold {
					nft_id: ALICE_NFT_ID,
					marketplace_id: CHARLIE_MARKETPLACE_ID,
					buyer: BOB,
					listed_price: 100,
					marketplace_cut: 50,
					royalty_cut: 40,
				};
				let event = Event::Marketplace(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn keep_alive() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let bob: mock::Origin = origin(BOB);
				let bob_balance = Balances::free_balance(BOB);

				// List NFT.
				Marketplace::list_nft(alice, ALICE_NFT_ID, CHARLIE_MARKETPLACE_ID, bob_balance)
					.unwrap();

				// Buy NFT.
				let err = Marketplace::buy_nft(bob, ALICE_NFT_ID);

				// Nothing should have changed.
				let nft = NFT::nfts(ALICE_NFT_ID).unwrap();
				let sale = Marketplace::listed_nfts(ALICE_NFT_ID);
				assert!(sale.is_some());
				assert_eq!(nft.owner, ALICE);
				assert_eq!(Balances::free_balance(BOB), bob_balance);
				assert_noop!(err, BalanceError::<Test>::KeepAlive);
			},
		)
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Buy invalid NFT.
				let err = Marketplace::buy_nft(alice, INVALID_NFT_ID);
				assert_noop!(err, Error::<Test>::NFTNotFound);
			},
		)
	}

	#[test]
	fn nft_not_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// Buy non listed NFT.
				let err = Marketplace::buy_nft(alice, BOB_NFT_ID);
				assert_noop!(err, Error::<Test>::NFTNotForSale);
			},
		)
	}

	#[test]
	fn cannot_buy_owned_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);

				// List NFT.
				Marketplace::list_nft(alice.clone(), ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10)
					.unwrap();

				// Buy owned NFT.
				let err = Marketplace::buy_nft(alice, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::CannotBuyOwnedNFT);
			},
		)
	}

	#[test]
	fn not_enough_balance_to_buy() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (CHARLIE, 1000)]).execute_with(
			|| {
				prepare_tests();
				let alice: mock::Origin = origin(ALICE);
				let bob: mock::Origin = origin(BOB);

				// List NFT.
				Marketplace::list_nft(alice, ALICE_NFT_ID, ALICE_MARKETPLACE_ID, 10_000).unwrap();

				// Buy owned NFT.
				let err = Marketplace::buy_nft(bob, ALICE_NFT_ID);
				assert_noop!(err, Error::<Test>::NotEnoughBalanceToBuy);
			},
		)
	}
}
