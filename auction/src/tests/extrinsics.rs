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
use frame_support::{
	assert_noop, assert_ok, bounded_vec, pallet_prelude::DispatchResultWithPostInfo, BoundedVec,
};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use primitives::{
	marketplace::{MarketplaceId, MarketplaceType},
	nfts::NFTId,
	CompoundFee, ConfigOp,
};
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::{MarketplaceExt, NFTExt};

use crate::{
	tests::mock,
	types::{AuctionData, BidderList},
	Auctions, Claims, Config, Deadlines, Error, Event as AuctionEvent,
};

const PERCENT_0: Permill = Permill::from_parts(0);
const PERCENT_20: Permill = Permill::from_parts(200000);
const ALICE_COLLECTION_ID_0: NFTId = 0;
const ALICE_NFT_ID_0: NFTId = 0;
const ALICE_NFT_ID_1: NFTId = 1;
const ALICE_MARKETPLACE_ID: u32 = 0;
const BOB_NFT_ID: NFTId = 2;
const INVALID_NFT_ID: NFTId = 99;
const INVALID_MARKETPLACE_ID: MarketplaceId = 99;
const DEFAULT_STARTBLOCK: BlockNumber = 10;
const DEFAULT_ENDBLOCK: BlockNumber = 1_000;
const DEFAULT_PRICE: u128 = 100;

fn origin(account: u64) -> mock::RuntimeOrigin {
	RawOrigin::Signed(account).into()
}

pub fn prepare_tests() {
	let alice: mock::RuntimeOrigin = origin(ALICE);
	let bob: mock::RuntimeOrigin = origin(BOB);

	//Create Collection
	NFT::create_collection(alice.clone(), BoundedVec::default(), None).unwrap();

	//Create NFTs.
	NFT::create_nft(
		alice.clone(),
		BoundedVec::default(),
		PERCENT_0,
		Some(ALICE_COLLECTION_ID_0),
		false,
	)
	.unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(bob, BoundedVec::default(), PERCENT_0, None, false).unwrap();

	//Create marketplace.
	Marketplace::create_marketplace(alice.clone(), MarketplaceType::Public).unwrap();
	Marketplace::set_marketplace_configuration(
		alice.clone(),
		ALICE_MARKETPLACE_ID,
		ConfigOp::Set(CompoundFee::Percentage(PERCENT_20)),
		ConfigOp::Noop,
		ConfigOp::Noop,
		ConfigOp::Noop,
		ConfigOp::Noop,
	)
	.unwrap();

	//Create auction.
	Auction::create_auction(
		alice,
		ALICE_NFT_ID_1,
		ALICE_MARKETPLACE_ID,
		DEFAULT_STARTBLOCK,
		DEFAULT_ENDBLOCK,
		DEFAULT_PRICE,
		Some(DEFAULT_PRICE + 10),
	)
	.unwrap();

	//Check existence.
	assert!(NFT::nfts(ALICE_NFT_ID_0).is_some());
	assert!(NFT::nfts(ALICE_NFT_ID_1).is_some());
	assert!(NFT::nfts(BOB_NFT_ID).is_some());
	assert!(Marketplace::marketplaces(ALICE_MARKETPLACE_ID).is_some());
}

pub mod create_auction {
	use primitives::nfts::NFTState;

	pub use super::*;

	#[test]
	fn create_auction() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			// Before execution
			let auction_count = Auctions::<Test>::iter().count();
			let claim_count = Claims::<Test>::iter().count();
			let mut deadlines = Deadlines::<Test>::get();

			// Expected Data
			let start_block = 10;
			let auction = AuctionData {
				creator: ALICE,
				start_block,
				end_block: start_block + <Test as Config>::MinAuctionDuration::get(),
				start_price: 300,
				buy_it_price: Some(400),
				bidders: BidderList::new(),
				marketplace_id: ALICE_MARKETPLACE_ID,
				is_extended: false,
			};

			let _ = deadlines.insert(ALICE_NFT_ID_0, auction.end_block);
			let state = NFTState::new(false, true, false, false, false, false, false);

			// Execution
			let ok = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				auction.marketplace_id,
				auction.start_block,
				auction.end_block,
				auction.start_price,
				auction.buy_it_price,
			);
			assert_ok!(ok);

			// Storage.
			assert_eq!(NFT::get_nft(ALICE_NFT_ID_0).unwrap().state, state);
			assert_eq!(Auctions::<Test>::iter().count(), auction_count + 1);
			assert_eq!(Claims::<Test>::iter().count(), claim_count);

			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_0).unwrap(), auction);
			assert_eq!(Deadlines::<Test>::get(), deadlines);

			// Events.
			let event = AuctionEvent::AuctionCreated {
				nft_id: ALICE_NFT_ID_0,
				marketplace_id: auction.marketplace_id,
				creator: auction.creator,
				start_price: auction.start_price,
				buy_it_price: auction.buy_it_price,
				start_block: auction.start_block,
				end_block: auction.end_block,
			};
			let event = RuntimeEvent::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn auction_cannot_start_in_the_past() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = AuctionBuilder::new().start(System::block_number() - 1).execute();
			assert_noop!(ok, Error::<Test>::AuctionCannotStartInThePast);
		})
	}

	#[test]
	fn auction_cannot_end_before_it_has_started() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = AuctionBuilder::new().end(System::block_number() - 1).execute();
			assert_noop!(ok, Error::<Test>::AuctionCannotEndBeforeItHasStarted);
		})
	}

	#[test]
	fn auction_duration_is_too_long() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let end = System::block_number() + <Test as Config>::MaxAuctionDuration::get() + 1;
			let ok = AuctionBuilder::new().end(end).execute();
			assert_noop!(ok, Error::<Test>::AuctionDurationIsTooLong);
		})
	}

	#[test]
	fn auction_duration_is_too_short() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let end = System::block_number() + <Test as Config>::MinAuctionDuration::get() - 1;
			let ok = AuctionBuilder::new().end(end).execute();
			assert_noop!(ok, Error::<Test>::AuctionDurationIsTooShort);
		})
	}

	#[test]
	fn auction_start_is_too_far_away() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let start = System::block_number() + <Test as Config>::MaxAuctionDelay::get() + 1;
			let end = start + <Test as Config>::MinAuctionDuration::get();
			let ok = AuctionBuilder::new().start(start).end(end).execute();
			assert_noop!(ok, Error::<Test>::AuctionStartIsTooFarAway);
		})
	}

	#[test]
	fn buy_it_price_cannot_be_less_or_equal_than_start_price() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = AuctionBuilder::new().price(100).now_buy(Some(99)).execute();
			assert_noop!(ok, Error::<Test>::BuyItPriceCannotBeLessOrEqualThanStartPrice);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = AuctionBuilder::new().nft_id(INVALID_NFT_ID).execute();
			assert_noop!(ok, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn cannot_list_not_owned_nfts() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = AuctionBuilder::new().nft_id(BOB_NFT_ID).execute();
			assert_noop!(ok, Error::<Test>::CannotListNotOwnedNFTs);
		})
	}

	#[test]
	fn cannot_list_listed_nfts() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			// Set listed.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_listed = true;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let ok = AuctionBuilder::new().nft_id(ALICE_NFT_ID_0).execute();
			assert_noop!(ok, Error::<Test>::CannotListListedNFTs);
		})
	}

	#[test]
	fn cannot_list_capsules_nfts() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			// Set capsule.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_capsule = true;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let ok = AuctionBuilder::new().nft_id(ALICE_NFT_ID_0).execute();
			assert_noop!(ok, Error::<Test>::CannotListCapsulesNFTs);
		})
	}

	#[test]
	fn cannot_list_not_synced_secret_nfts() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			// Set secret.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_secret = true;
			nft.state.is_syncing = true;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let ok = AuctionBuilder::new().nft_id(ALICE_NFT_ID_0).execute();
			assert_noop!(ok, Error::<Test>::CannotListNotSyncedSecretNFTs);
		})
	}

	#[test]
	fn cannot_list_delegated_nfts() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			// Set delegated.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_delegated = true;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let ok = AuctionBuilder::new().nft_id(ALICE_NFT_ID_0).execute();
			assert_noop!(ok, Error::<Test>::CannotListDelegatedNFTs);
		})
	}

	#[test]
	fn cannot_list_soulbound_nfts() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			// Set soulbound.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_soulbound = true;
			nft.creator = BOB;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let ok = AuctionBuilder::new().nft_id(ALICE_NFT_ID_0).execute();
			assert_noop!(ok, Error::<Test>::CannotListNotCreatedSoulboundNFTs);
		})
	}

	#[test]
	fn cannot_list_rented_nfts() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			// Set rented.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_rented = true;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let ok = AuctionBuilder::new().nft_id(ALICE_NFT_ID_0).execute();
			assert_noop!(ok, Error::<Test>::CannotListRentedNFTs);
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = AuctionBuilder::new().mp_id(INVALID_MARKETPLACE_ID).execute();
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn not_allowed_to_list_public_account_blacklist() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Add Alice to disallow list.
			Marketplace::set_marketplace_configuration(
				alice.clone(),
				ALICE_MARKETPLACE_ID,
				ConfigOp::Noop,
				ConfigOp::Noop,
				ConfigOp::Set(BoundedVec::try_from(vec![ALICE]).unwrap()),
				ConfigOp::Noop,
				ConfigOp::Noop,
			)
			.unwrap();

			let ok = AuctionBuilder::new().execute();
			assert_noop!(ok, Error::<Test>::NotAllowedToList);
		})
	}

	#[test]
	fn not_allowed_to_list_public_collection_blacklist() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Set public marketplace collection list (ban list) with bob's collection.
			Marketplace::set_marketplace_configuration(
				alice.clone(),
				ALICE_MARKETPLACE_ID,
				ConfigOp::Noop,
				ConfigOp::Noop,
				ConfigOp::Noop,
				ConfigOp::Noop,
				ConfigOp::Set(BoundedVec::try_from(vec![ALICE_COLLECTION_ID_0]).unwrap()),
			)
			.unwrap();

			let err = AuctionBuilder::new().execute();
			assert_noop!(err, Error::<Test>::NotAllowedToList);
		})
	}

	#[test]
	fn not_allowed_to_list_public_account_and_collection_blacklist() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Set public marketplace collection list (ban list) with bob's collection.
			Marketplace::set_marketplace_configuration(
				alice.clone(),
				ALICE_MARKETPLACE_ID,
				ConfigOp::Noop,
				ConfigOp::Noop,
				ConfigOp::Set(BoundedVec::try_from(vec![ALICE]).unwrap()),
				ConfigOp::Noop,
				ConfigOp::Set(BoundedVec::try_from(vec![ALICE_COLLECTION_ID_0]).unwrap()),
			)
			.unwrap();

			let err = AuctionBuilder::new().execute();
			assert_noop!(err, Error::<Test>::NotAllowedToList);
		})
	}

	#[test]
	fn not_allowed_to_list_private_not_whitelist() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = origin(ALICE);

			// Set marketplace private (without alice's account in account list / allow list).
			Marketplace::set_marketplace_kind(
				alice.clone(),
				ALICE_MARKETPLACE_ID,
				MarketplaceType::Private,
			)
			.unwrap();

			let ok = AuctionBuilder::new().execute();
			assert_noop!(ok, Error::<Test>::NotAllowedToList);
		})
	}

	#[test]
	fn price_cannot_cover_marketplace_fee() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let price = 101;

			// Set flat commission fee.
			Marketplace::set_marketplace_configuration(
				origin(ALICE),
				ALICE_MARKETPLACE_ID,
				ConfigOp::Set(CompoundFee::Flat(price)),
				ConfigOp::Noop,
				ConfigOp::Noop,
				ConfigOp::Noop,
				ConfigOp::Noop,
			)
			.unwrap();

			let ok = AuctionBuilder::new().price(price - 1).execute();
			assert_noop!(ok, Error::<Test>::PriceCannotCoverMarketplaceFee);
		})
	}

	#[test]
	fn maximum_auctions_limit_reached() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let current_count = Auction::deadlines().len();
			let limit = <Test as Config>::ParallelAuctionLimit::get() as usize;

			(current_count..limit)
				.map(|_| {
					NFT::create_nft(origin(ALICE), BoundedVec::default(), PERCENT_0, None, false)
						.unwrap();
					NFT::next_nft_id() - 1
				})
				.for_each(|x| {
					AuctionBuilder::new().nft_id(x).execute().unwrap();
				});

			NFT::create_nft(origin(ALICE), BoundedVec::default(), PERCENT_0, None, false).unwrap();

			let ok = AuctionBuilder::new().nft_id(NFT::next_nft_id() - 1).execute();
			assert_noop!(ok, Error::<Test>::MaximumAuctionsLimitReached);
		})
	}
}

pub mod cancel_auction {
	pub use super::*;

	#[test]
	fn cancel_auction() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			// Before execution
			let mut deadlines = Deadlines::<Test>::get();
			let mut nft = NFT::get_nft(ALICE_NFT_ID_1).unwrap();
			let auction_count = Auctions::<Test>::iter().count();
			let claims_count = Claims::<Test>::iter().count();

			// Execution
			assert_ok!(Auction::cancel_auction(origin(ALICE), ALICE_NFT_ID_1));

			// Expected Data
			deadlines.remove(ALICE_NFT_ID_1);
			nft.state.is_listed = false;
			nft.owner = ALICE;

			// Storage.
			assert_eq!(NFT::get_nft(ALICE_NFT_ID_1).unwrap(), nft);
			assert_eq!(Claims::<Test>::iter().count(), claims_count);

			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_1), None);
			assert_eq!(Auctions::<Test>::iter().count(), auction_count - 1);
			assert_eq!(Deadlines::<Test>::get(), deadlines);

			// Check Events.
			let event = AuctionEvent::AuctionCancelled { nft_id: ALICE_NFT_ID_1 };
			let event = RuntimeEvent::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::cancel_auction(origin(ALICE), INVALID_NFT_ID);
			assert_noop!(ok, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			NFT::create_nft(origin(ALICE), BoundedVec::default(), PERCENT_0, None, false).unwrap();
			let nft_id = NFT::next_nft_id() - 1;

			let ok = Auction::cancel_auction(origin(ALICE), nft_id);
			assert_noop!(ok, Error::<Test>::AuctionDoesNotExist);
		})
	}

	#[test]
	fn not_the_auction_creator() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::cancel_auction(origin(BOB), ALICE_NFT_ID_1);
			assert_noop!(ok, Error::<Test>::NotTheAuctionCreator);
		})
	}

	#[test]
	fn cannot_cancel_auction_in_progress() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();
			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			run_to_block(auction.start_block);

			let ok = Auction::cancel_auction(alice, ALICE_NFT_ID_1);
			assert_noop!(ok, Error::<Test>::CannotCancelAuctionInProgress);
		})
	}
}

pub mod end_auction {
	pub use super::*;

	#[test]
	fn end_auction() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let eve: mock::RuntimeOrigin = RawOrigin::Signed(EVE).into();

			// Bob creates the NFT
			NFT::create_nft(origin(BOB), BoundedVec::default(), PERCENT_20, None, false).unwrap();
			let nft_id = NFT::next_nft_id() - 1;

			// Bob sends the NFT to EVE
			let mut nft = NFT::get_nft(nft_id).unwrap();
			nft.owner = EVE;
			NFT::set_nft(nft_id, nft.clone()).unwrap();

			// Creating the auction
			let start = System::block_number() + <Test as Config>::MaxAuctionDelay::get();
			let end = start + <Test as Config>::MaxAuctionDuration::get();
			let ab = AuctionBuilder::new().origin(eve.clone()).nft_id(nft_id).start(start).end(end);
			ab.execute().unwrap();

			run_to_block(start);

			let mp_owner_balance = Balances::free_balance(ALICE);
			let nft_creator_balance = Balances::free_balance(BOB);
			let old_nft_owner_balance = Balances::free_balance(EVE);
			let new_nft_owner_balance = Balances::free_balance(DAVE);
			let failed_bidder_balance = Balances::free_balance(CHARLIE);
			let auction = Auctions::<Test>::get(nft_id).unwrap();
			let marketplace = Marketplace::marketplaces(ALICE_MARKETPLACE_ID).unwrap();
			let commission_fee = marketplace.commission_fee.unwrap();
			let charlie_bid = auction.start_price + 10;
			let dave_bid = charlie_bid + 10;

			assert_eq!(commission_fee, CompoundFee::Percentage(PERCENT_20));
			assert_ok!(Auction::add_bid(origin(CHARLIE), nft_id, charlie_bid));

			run_to_block(end - 1);
			assert_ok!(Auction::add_bid(origin(DAVE), nft_id, dave_bid));
			assert_eq!(Balances::free_balance(Auction::account_id()), charlie_bid + dave_bid);

			// Deadline storage before execution
			let mut deadlines = Deadlines::<Test>::get();

			// Execute end Auction
			run_to_block(end + 1);
			assert_ok!(Auction::end_auction(eve, nft_id));

			// Balance.
			let mp_owner_new_balance = Balances::free_balance(ALICE);
			let nft_creator_new_balance = Balances::free_balance(BOB);
			let old_nft_owner_new_balance = Balances::free_balance(EVE);
			let new_nft_owner_new_balance = Balances::free_balance(DAVE);
			let failed_bidder_new_balance = Balances::free_balance(CHARLIE);
			let pallet_new_balance = Balances::free_balance(Auction::account_id());

			let bidder_paid = dave_bid;
			let marketplace_cut: u128 = match commission_fee {
				CompoundFee::Flat(x) => x,
				CompoundFee::Percentage(x) => x * bidder_paid,
			};
			let royalty_cut: u128 = PERCENT_20 * bidder_paid.saturating_sub(marketplace_cut.into());
			let auctioneer_cut: u128 =
				bidder_paid.saturating_sub(marketplace_cut.into()).saturating_sub(royalty_cut);

			// Let's see if owners got their money
			assert_eq!(mp_owner_new_balance, mp_owner_balance + marketplace_cut);
			assert_eq!(nft_creator_new_balance, nft_creator_balance + royalty_cut);
			assert_eq!(old_nft_owner_new_balance, old_nft_owner_balance + auctioneer_cut);
			assert_eq!(new_nft_owner_new_balance, new_nft_owner_balance - bidder_paid);
			assert_eq!(failed_bidder_new_balance, failed_bidder_balance - charlie_bid);
			assert_eq!(pallet_new_balance, charlie_bid);

			// Expected NFT state
			nft.state.is_listed = false;
			nft.owner = DAVE;
			assert_eq!(NFT::get_nft(nft_id).unwrap(), nft);

			// Expected Auction state
			deadlines.remove(nft_id);
			assert_eq!(Claims::<Test>::iter().count(), 1);
			assert_eq!(Claims::<Test>::get(CHARLIE), Some(charlie_bid));
			assert_eq!(Auctions::<Test>::get(nft_id), None);
			assert_eq!(Deadlines::<Test>::get(), deadlines);

			// Check Events.
			let event = AuctionEvent::AuctionCompleted {
				nft_id,
				new_owner: Some(DAVE),
				paid_amount: Some(bidder_paid),
				marketplace_cut: Some(marketplace_cut),
				royalty_cut: Some(royalty_cut),
				auctioneer_cut: Some(auctioneer_cut),
			};
			let event = RuntimeEvent::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::end_auction(origin(ALICE), INVALID_NFT_ID);
			assert_noop!(ok, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			NFT::create_nft(origin(ALICE), BoundedVec::default(), PERCENT_0, None, false).unwrap();
			let nft_id = NFT::next_nft_id() - 1;

			let ok = Auction::end_auction(origin(ALICE), nft_id);
			assert_noop!(ok, Error::<Test>::AuctionDoesNotExist);
		})
	}

	#[test]
	fn not_the_auction_creator() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::end_auction(origin(BOB), ALICE_NFT_ID_1);
			assert_noop!(ok, Error::<Test>::NotTheAuctionCreator);
		})
	}

	#[test]
	fn cannot_end_auction_that_was_not_extended() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let alice: mock::RuntimeOrigin = RawOrigin::Signed(ALICE).into();

			let ok = Auction::end_auction(alice, ALICE_NFT_ID_1);
			assert_noop!(ok, Error::<Test>::CannotEndAuctionThatWasNotExtended);
		})
	}
}

pub mod add_bid {
	pub use super::*;

	#[test]
	fn add_bid() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bidder_balance = Balances::free_balance(BOB);

			let mut auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			run_to_block(auction.start_block);

			let bid = auction.start_price + 10;
			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid));

			// Balance.
			let bidder_new_balance = Balances::free_balance(BOB);
			let pallet_new_balance = Balances::free_balance(Auction::account_id());
			assert_eq!(bidder_new_balance, bidder_balance - bid);
			assert_eq!(pallet_new_balance, bid);

			// Storage.
			auction.bidders.list = bounded_vec![(BOB, bid)];

			assert_eq!(Claims::<Test>::iter().count(), 0);
			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_1), Some(auction));

			// Check Events.
			let event = AuctionEvent::BidAdded { nft_id: ALICE_NFT_ID_1, bidder: BOB, amount: bid };
			let event = RuntimeEvent::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn add_bid_above_max_bidder_history_size() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			// Before execution #1
			let claims_count = Claims::<Test>::iter().count();

			let final_bidder_balance = Balances::free_balance(EVE);
			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			run_to_block(auction.start_block);

			let bidder1_bid = auction.start_price + 1;
			let bidder2_bid = bidder1_bid + 1;
			let bidder3_bid = bidder2_bid + 1;

			let accounts = vec![(BOB, bidder1_bid), (CHARLIE, bidder2_bid), (DAVE, bidder3_bid)];
			for bidder in accounts.iter() {
				assert_ok!(Auction::add_bid(origin(bidder.0), ALICE_NFT_ID_1, bidder.1));
			}
			assert_eq!(
				Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap().bidders.len(),
				<Test as Config>::BidderListLengthLimit::get() as usize
			);

			// Before execution #2
			let mut auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			// Execution
			let final_bid = bidder3_bid + 1;
			assert_ok!(Auction::add_bid(origin(EVE), ALICE_NFT_ID_1, final_bid));

			// Balance.
			let final_bidder_new_balance = Balances::free_balance(EVE);
			let pallet_new_balance = Balances::free_balance(Auction::account_id());
			assert_eq!(final_bidder_new_balance, final_bidder_balance - final_bid);
			assert_eq!(pallet_new_balance, bidder1_bid + bidder2_bid + bidder3_bid + final_bid);

			// Expected Storage
			auction.bidders.remove_lowest_bid();
			auction.bidders.insert_new_bid(EVE, final_bid);

			// Storage.
			assert_eq!(Claims::<Test>::iter().count(), claims_count + 1);
			assert_eq!(Claims::<Test>::get(BOB), Some(bidder1_bid));
			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_1), Some(auction));

			// Check Events.
			let event = AuctionEvent::BidDropped {
				nft_id: ALICE_NFT_ID_1,
				bidder: BOB,
				amount: bidder1_bid,
			};
			let event = RuntimeEvent::Auction(event);
			System::assert_has_event(event);

			let event =
				AuctionEvent::BidAdded { nft_id: ALICE_NFT_ID_1, bidder: EVE, amount: final_bid };
			let event = RuntimeEvent::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn add_bid_increase_auction_duration() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob_balance = Balances::free_balance(BOB);
			let mut auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();
			let mut deadlines = Deadlines::<Test>::get();

			let grace_period = <Test as Config>::AuctionGracePeriod::get();
			let remaining_blocks = 3;
			let target_block = auction.end_block - remaining_blocks;
			let new_end_block = auction.end_block + (grace_period - remaining_blocks);

			run_to_block(target_block);

			let bid = auction.start_price + 10;
			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid));

			// Balance.
			let bob_new_balance = Balances::free_balance(BOB);
			assert_eq!(bob_new_balance, bob_balance - bid);

			// Storage.
			auction.bidders.insert_new_bid(BOB, bid);
			auction.end_block = new_end_block;
			auction.is_extended = true;
			deadlines.update(ALICE_NFT_ID_1, new_end_block);

			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_1), Some(auction));
			assert_eq!(Deadlines::<Test>::get(), deadlines);

			// Check Events.
			let event = AuctionEvent::BidAdded { nft_id: ALICE_NFT_ID_1, bidder: BOB, amount: bid };
			let event = RuntimeEvent::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn add_bid_and_replace_current() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob_balance = Balances::free_balance(BOB);
			let mut auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			run_to_block(auction.start_block);

			let old_bid = auction.start_price + 10;
			let new_bid = old_bid + 10;
			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, old_bid));
			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, new_bid));

			// Balance.
			let bob_new_balance = Balances::free_balance(BOB);
			assert_eq!(bob_new_balance, bob_balance - new_bid);

			// Storage.
			auction.bidders.list = bounded_vec![(BOB, new_bid)];

			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_1), Some(auction));

			// Check Events.
			let event =
				AuctionEvent::BidAdded { nft_id: ALICE_NFT_ID_1, bidder: BOB, amount: new_bid };
			let event = RuntimeEvent::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::add_bid(origin(ALICE), INVALID_NFT_ID, 1);
			assert_noop!(ok, Error::<Test>::AuctionDoesNotExist);
		})
	}

	#[test]
	fn cannot_add_bid_to_your_own_auctions() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::add_bid(origin(ALICE), ALICE_NFT_ID_1, 1);
			assert_noop!(ok, Error::<Test>::CannotAddBidToYourOwnAuctions);
		})
	}

	#[test]
	fn auction_not_started() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, 1);
			assert_noop!(ok, Error::<Test>::AuctionNotStarted);
		})
	}

	#[test]
	fn cannot_bid_less_than_the_highest_bid() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			let bob_bid = auction.start_price + 1;
			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bob_bid));

			let ok = Auction::add_bid(origin(DAVE), ALICE_NFT_ID_1, bob_bid);
			assert_noop!(ok, Error::<Test>::CannotBidLessThanTheHighestBid);
		})
	}

	#[test]
	fn cannot_bid_less_than_the_starting_price() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			let ok = Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, auction.start_price - 1);
			assert_noop!(ok, Error::<Test>::CannotBidLessThanTheStartingPrice);
		})
	}

	#[test]
	fn not_enough_funds() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			let balance = Balances::free_balance(BOB);
			let bid = balance + 1;
			assert!(bid > auction.start_price);

			let ok = Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn not_enough_funds_to_replace() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			let bid = Balances::free_balance(BOB);
			assert!(bid > auction.start_price);

			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid));

			let ok = Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid + 10);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

pub mod remove_bid {
	pub use super::*;

	#[test]
	fn remove_bid() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let bob: mock::RuntimeOrigin = RawOrigin::Signed(BOB).into();
			let bob_balance = Balances::free_balance(BOB);
			let mut auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();
			run_to_block(DEFAULT_STARTBLOCK);

			let bid = auction.start_price + 10;
			assert_ok!(Auction::add_bid(bob.clone(), ALICE_NFT_ID_1, bid));
			assert_ok!(Auction::remove_bid(bob, ALICE_NFT_ID_1));

			// Balance.
			let bob_new_balance = Balances::free_balance(BOB);
			let pallet_new_balance = Balances::free_balance(Auction::account_id());
			assert_eq!(bob_new_balance, bob_balance);
			assert_eq!(pallet_new_balance, 0);

			// Storage.
			auction.bidders.list = bounded_vec![];

			assert_eq!(Claims::<Test>::iter().count(), 0);
			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_1), Some(auction));

			// Check Events.
			let event =
				AuctionEvent::BidRemoved { nft_id: ALICE_NFT_ID_1, bidder: BOB, amount: bid };
			let event = RuntimeEvent::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::remove_bid(origin(ALICE), INVALID_NFT_ID);
			assert_noop!(ok, Error::<Test>::AuctionDoesNotExist);
		})
	}

	#[test]
	fn cannot_remove_bid_at_the_end_of_auction() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();
			let auction_end_period = <Test as Config>::AuctionEndingPeriod::get();
			let target_block = auction.end_block - auction_end_period;

			run_to_block(auction.start_block);

			let bid = auction.start_price + 1;
			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid));

			run_to_block(target_block);

			let ok = Auction::remove_bid(origin(BOB), ALICE_NFT_ID_1);
			assert_noop!(ok, Error::<Test>::CannotRemoveBidAtTheEndOfAuction);
		})
	}

	#[test]
	fn bid_does_not_exist() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::remove_bid(origin(BOB), ALICE_NFT_ID_1);
			assert_noop!(ok, Error::<Test>::BidDoesNotExist);
		})
	}
}

pub mod buy_it_now {
	pub use super::*;

	#[test]
	fn buy_it_now() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let claims_count = Claims::<Test>::iter().count();
			let eve: mock::RuntimeOrigin = origin(EVE);
			let dave: mock::RuntimeOrigin = origin(DAVE);

			// Bob creates the NFT.
			NFT::create_nft(origin(BOB), BoundedVec::default(), PERCENT_20, None, false).unwrap();
			let nft_id = NFT::next_nft_id() - 1;

			// Bob gives the NFT to EVE
			let mut nft = NFT::get_nft(nft_id).unwrap();
			nft.owner = EVE;
			NFT::set_nft(nft_id, nft.clone()).unwrap();

			// Eve creates the auction
			AuctionBuilder::new()
				.origin(eve)
				.nft_id(nft_id)
				.now_buy(Some(DEFAULT_PRICE + 100))
				.execute()
				.unwrap();
			let auction = Auctions::<Test>::get(nft_id).unwrap();

			run_to_block(auction.start_block);

			// Check balances before execution buy_it_now
			let mp_owner_balance = Balances::free_balance(ALICE);
			let nft_creator_balance = Balances::free_balance(BOB);
			let old_nft_owner_balance = Balances::free_balance(EVE);
			let new_nft_owner_balance = Balances::free_balance(CHARLIE);
			let loser_bidder_balance = Balances::free_balance(DAVE);
			let pallet_balance = Balances::free_balance(Auction::account_id());

			// Add one bid
			let loser_bid = auction.start_price + 10;
			Auction::add_bid(dave, nft_id, loser_bid).unwrap();

			// Execute buy it now
			assert_ok!(Auction::buy_it_now(origin(CHARLIE), nft_id));

			// Balances after transfer
			let mp_owner_new_balance = Balances::free_balance(ALICE);
			let nft_creator_new_balance = Balances::free_balance(BOB);
			let old_nft_owner_new_balance = Balances::free_balance(EVE);
			let new_nft_owner_new_balance = Balances::free_balance(CHARLIE);
			let loser_bidder_new_balance = Balances::free_balance(DAVE);
			let pallet_new_balance = Balances::free_balance(Auction::account_id());

			// Expected balance change
			let mp = Marketplace::get_marketplace(ALICE_MARKETPLACE_ID).unwrap();
			let paid_amount = auction.buy_it_price.unwrap();
			let marketplace_cut = match mp.commission_fee.unwrap() {
				CompoundFee::Flat(x) => x,
				CompoundFee::Percentage(x) => x * paid_amount,
			};
			let royalty_cut: u128 = PERCENT_20 * paid_amount.saturating_sub(marketplace_cut.into());
			let auctioneer_cut: u128 =
				paid_amount.saturating_sub(marketplace_cut.into()).saturating_sub(royalty_cut);

			assert_eq!(mp_owner_new_balance, mp_owner_balance + marketplace_cut);
			assert_eq!(nft_creator_new_balance, nft_creator_balance + royalty_cut);
			assert_eq!(old_nft_owner_new_balance, old_nft_owner_balance + auctioneer_cut);
			assert_eq!(new_nft_owner_new_balance, new_nft_owner_balance - paid_amount);
			assert_eq!(loser_bidder_new_balance, loser_bidder_balance - loser_bid);
			assert_eq!(pallet_new_balance, pallet_balance + loser_bid);

			// Expected NFT state
			nft.state.is_listed = false;
			nft.owner = CHARLIE;
			assert_eq!(NFT::get_nft(nft_id).unwrap(), nft);

			// Expected Auction state
			assert_eq!(Claims::<Test>::iter().count(), claims_count + 1);
			assert_eq!(Claims::<Test>::get(DAVE).unwrap(), loser_bid);
			assert_eq!(Auctions::<Test>::get(nft_id), None);

			// Check Events.
			let event = AuctionEvent::AuctionCompleted {
				nft_id,
				new_owner: Some(CHARLIE),
				paid_amount: Some(paid_amount),
				marketplace_cut: Some(marketplace_cut),
				royalty_cut: Some(royalty_cut),
				auctioneer_cut: Some(auctioneer_cut),
			};
			let event = RuntimeEvent::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(None).execute_with(|| {
			let ok = Auction::buy_it_now(origin(BOB), INVALID_NFT_ID);
			assert_noop!(ok, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(None).execute_with(|| {
			NFT::create_nft(origin(ALICE), BoundedVec::default(), PERCENT_0, None, false).unwrap();
			let nft_id = NFT::next_nft_id() - 1;

			let ok = Auction::buy_it_now(origin(BOB), nft_id);
			assert_noop!(ok, Error::<Test>::AuctionDoesNotExist);
		})
	}

	#[test]
	fn auction_does_not_support_buy_it_now() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let nft_id = ALICE_NFT_ID_1;
			Auctions::<Test>::mutate(nft_id, |x| {
				let x = x.as_mut().unwrap();
				x.buy_it_price = None;
			});

			let ok = Auction::buy_it_now(origin(BOB), nft_id);
			assert_noop!(ok, Error::<Test>::AuctionDoesNotSupportBuyItNow);
		})
	}

	#[test]
	fn cannot_buy_it_now_to_your_own_auctions() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let ok = Auction::buy_it_now(origin(ALICE), ALICE_NFT_ID_1);
			assert_noop!(ok, Error::<Test>::CannotBuyItNowToYourOwnAuctions);
		})
	}

	#[test]
	fn auction_not_started() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::buy_it_now(origin(BOB), ALICE_NFT_ID_1);
			assert_noop!(ok, Error::<Test>::AuctionNotStarted);
		})
	}

	#[test]
	fn cannot_buy_it_when_a_bid_is_higher_than_buy_it_price() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let nft_id = ALICE_NFT_ID_1;
			let auction = Auctions::<Test>::get(nft_id).unwrap();

			let price = auction.buy_it_price.unwrap();
			assert_ok!(Auction::add_bid(origin(CHARLIE), nft_id, price));

			let ok = Auction::buy_it_now(origin(BOB), nft_id);
			assert_noop!(ok, Error::<Test>::CannotBuyItWhenABidIsHigherThanBuyItPrice);
		})
	}
}

pub mod claim {
	pub use super::*;

	#[test]
	fn claim() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let nft_id = ALICE_NFT_ID_1;
			let auction = Auctions::<Test>::get(nft_id).unwrap();

			run_to_block(auction.start_block);

			let lost_bidder_balance = Balances::free_balance(BOB);
			let pallet_balance = Balances::free_balance(Auction::account_id());

			let loser_bid = auction.start_price + 1;
			let winner_bid = loser_bid + 1;
			assert_ok!(Auction::add_bid(origin(BOB), nft_id, loser_bid));
			assert_ok!(Auction::add_bid(origin(CHARLIE), nft_id, winner_bid));

			// Let auction finish
			run_to_block(auction.end_block + 1);

			// Execute claim
			let claim = Claims::<Test>::get(BOB).unwrap();
			assert_ok!(Auction::claim(origin(BOB)));

			// Balance check.
			let lost_bidder_new_balance = Balances::free_balance(BOB);
			let pallet_new_balance = Balances::free_balance(Auction::account_id());

			assert_eq!(lost_bidder_new_balance, lost_bidder_balance);
			assert_eq!(pallet_new_balance, pallet_balance);
			assert_eq!(claim, loser_bid);

			// Auction storage check.
			assert_eq!(Claims::<Test>::iter().count(), 0);
			assert_eq!(Claims::<Test>::get(BOB), None);

			// Event.
			let event = AuctionEvent::BalanceClaimed { account: BOB, amount: claim };
			let event = RuntimeEvent::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn claim_does_not_exist() {
		ExtBuilder::new_build(None).execute_with(|| {
			prepare_tests();

			let ok = Auction::claim(origin(BOB));
			assert_noop!(ok, Error::<Test>::ClaimDoesNotExist);
		})
	}
}

pub struct AuctionBuilder {
	pub origin: mock::RuntimeOrigin,
	pub nft_id: NFTId,
	pub mp_id: MarketplaceId,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub price: u128,
	pub now_buy: Option<u128>,
}

impl AuctionBuilder {
	pub fn new() -> AuctionBuilder {
		Self {
			origin: origin(ALICE),
			nft_id: ALICE_NFT_ID_0,
			mp_id: ALICE_MARKETPLACE_ID,
			start: System::block_number(),
			end: System::block_number() + <Test as Config>::MaxAuctionDuration::get() - 1,
			price: DEFAULT_PRICE,
			now_buy: None,
		}
	}

	pub fn origin(mut self, o: mock::RuntimeOrigin) -> Self {
		self.origin = o;
		self
	}

	pub fn nft_id(mut self, n: NFTId) -> Self {
		self.nft_id = n;
		self
	}

	pub fn mp_id(mut self, m: MarketplaceId) -> Self {
		self.mp_id = m;
		self
	}

	pub fn start(mut self, b: BlockNumber) -> Self {
		self.start = b;
		self
	}

	pub fn end(mut self, b: BlockNumber) -> Self {
		self.end = b;
		self
	}

	pub fn price(mut self, p: u128) -> Self {
		self.price = p;
		self
	}

	pub fn now_buy(mut self, n: Option<u128>) -> Self {
		self.now_buy = n;
		self
	}

	pub fn execute(self) -> DispatchResultWithPostInfo {
		Auction::create_auction(
			self.origin,
			self.nft_id,
			self.mp_id,
			self.start,
			self.end,
			self.price,
			self.now_buy,
		)
	}
}
