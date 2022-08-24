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
use frame_support::{assert_noop, assert_ok, bounded_vec, error::BadOrigin, BoundedVec};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use primitives::{
	marketplace::{MarketplaceId, MarketplaceType},
	nfts::NFTId,
	CompoundFee, ConfigOp,
};
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::{MarketplaceExt, NFTExt};
use ternoa_marketplace::Error as MarketplaceError;

use crate::{
	tests::mock,
	types::{AuctionData, BidderList, DeadlineList},
	Auctions, Claims, Deadlines, Error, Event as AuctionEvent,
};

const PERCENT_0: Permill = Permill::from_parts(0);
const PERCENT_20: Permill = Permill::from_parts(200000);
const ALICE_NFT_ID_0: NFTId = 0;
const ALICE_NFT_ID_1: NFTId = 1;
const ALICE_MARKETPLACE_ID: u32 = 0;
const BOB_NFT_ID: NFTId = 2;
const INVALID_NFT_ID: NFTId = 99;
const INVALID_MARKETPLACE_ID: MarketplaceId = 99;
const DEFAULT_STARTBLOCK: BlockNumber = 10;
const DEFAULT_ENDBLOCK: BlockNumber = 1_000;
const DEFAULT_PRICE: u128 = 100;

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

pub fn prepare_tests() {
	let alice: mock::Origin = origin(ALICE);
	let bob: mock::Origin = origin(BOB);

	//Create NFTs.
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
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
	pub use super::*;

	#[test]
	fn create_auction() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let start_block = 10;
			let auction = AuctionData {
				creator: ALICE,
				start_block,
				end_block: start_block + MIN_AUCTION_DURATION,
				start_price: 300,
				buy_it_price: Some(400),
				bidders: BidderList::new(),
				marketplace_id: ALICE_MARKETPLACE_ID,
				is_extended: false,
			};

			let deadline = DeadlineList(bounded_vec![
				(ALICE_NFT_ID_0, auction.end_block),
				(ALICE_NFT_ID_1, DEFAULT_ENDBLOCK)
			]);

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
			assert_eq!(NFT::get_nft(ALICE_NFT_ID_0).unwrap().state.is_auctioned, true);
			assert_eq!(Auctions::<Test>::iter().count(), 2);
			assert_eq!(Claims::<Test>::iter().count(), 0);

			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_0).unwrap(), auction);
			assert_eq!(Deadlines::<Test>::get(), deadline);

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
			let event = Event::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn auction_cannot_start_in_the_past() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let current_block = System::block_number();
			let start_block = current_block - 1;
			assert!(start_block < current_block);

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				start_block,
				1_000,
				100,
				Some(200),
			);
			assert_noop!(err, Error::<Test>::AuctionCannotStartInThePast);
		})
	}

	#[test]
	fn auction_cannot_end_before_it_has_started() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let start_block = System::block_number();
			let end_block = start_block - 1;

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				start_block,
				end_block,
				100,
				Some(200),
			);
			assert_noop!(err, Error::<Test>::AuctionCannotEndBeforeItHasStarted);
		})
	}

	#[test]
	fn auction_duration_is_too_long() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let start_block = System::block_number();
			let end_block = start_block + MAX_AUCTION_DURATION + 1;

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				start_block,
				end_block,
				100,
				Some(200),
			);
			assert_noop!(err, Error::<Test>::AuctionDurationIsTooLong);
		})
	}

	#[test]
	fn auction_duration_is_too_short() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let start_block = System::block_number();
			let end_block = start_block + MIN_AUCTION_DURATION - 1;

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				start_block,
				end_block,
				100,
				Some(200),
			);
			assert_noop!(err, Error::<Test>::AuctionDurationIsTooShort);
		})
	}

	#[test]
	fn auction_start_is_too_far_away() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let start_block = System::block_number() + MAX_AUCTION_DELAY + 1;
			let end_block = start_block + MIN_AUCTION_DURATION;

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				start_block,
				end_block,
				100,
				Some(200),
			);
			assert_noop!(err, Error::<Test>::AuctionStartIsTooFarAway);
		})
	}

	#[test]
	fn buy_it_price_cannot_be_less_or_equal_than_start_price() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let start_price = 100;
			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				start_price,
				Some(start_price),
			);
			assert_noop!(err, Error::<Test>::BuyItPriceCannotBeLessOrEqualThanStartPrice);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::create_auction(
				origin(ALICE),
				INVALID_NFT_ID,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				INVALID_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn cannot_auction_not_owned_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::create_auction(
				origin(ALICE),
				BOB_NFT_ID,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, Error::<Test>::CannotAuctionNotOwnedNFTs);
		})
	}

	#[test]
	fn cannot_auction_listed_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			// Set listed.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_listed = true;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, Error::<Test>::CannotAuctionListedNFTs);
		})
	}

	#[test]
	fn cannot_auction_capsules_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			// Set capsule.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_capsule = true;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, Error::<Test>::CannotAuctionCapsulesNFTs);
		})
	}

	#[test]
	fn cannot_auction_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			// Set delegated.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_delegated = true;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, Error::<Test>::CannotAuctionDelegatedNFTs);
		})
	}

	#[test]
	fn cannot_auction_soulbound_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			// Set soulbound.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_soulbound = true;
			nft.creator = BOB;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, Error::<Test>::CannotAuctionSoulboundNFTs);
		})
	}

	#[test]
	fn cannot_auction_auctioned_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			// Set auctioned.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_auctioned = true;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, Error::<Test>::CannotAuctionAuctionedNFTs);
		})
	}

	#[test]
	fn cannot_auction_rented_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			// Set rented.
			let mut nft = NFT::get_nft(ALICE_NFT_ID_0).unwrap();
			nft.state.is_rented = true;
			NFT::set_nft(ALICE_NFT_ID_0, nft).unwrap();

			let err = Auction::create_auction(
				origin(ALICE),
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, Error::<Test>::CannotAuctionRentedNFTs);
		})
	}

	#[test]
	fn not_allowed_to_list() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Add Alice to disallow list.
			Marketplace::set_marketplace_configuration(
				alice.clone(),
				ALICE_MARKETPLACE_ID,
				ConfigOp::Noop,
				ConfigOp::Noop,
				ConfigOp::Set(BoundedVec::try_from(vec![ALICE]).unwrap()),
				ConfigOp::Noop,
			)
			.unwrap();

			let err = Auction::create_auction(
				alice,
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, MarketplaceError::<Test>::AccountNotAllowedToList);
		})
	}

	#[test]
	fn price_cannot_cover_marketplace_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = origin(ALICE);

			// Set flat commission fee.
			Marketplace::set_marketplace_configuration(
				alice.clone(),
				ALICE_MARKETPLACE_ID,
				ConfigOp::Set(CompoundFee::Flat(101)),
				ConfigOp::Noop,
				ConfigOp::Noop,
				ConfigOp::Noop,
			)
			.unwrap();

			let err = Auction::create_auction(
				alice,
				ALICE_NFT_ID_0,
				ALICE_MARKETPLACE_ID,
				System::block_number(),
				System::block_number() + MIN_AUCTION_DURATION,
				100,
				Some(101),
			);
			assert_noop!(err, Error::<Test>::PriceCannotCoverMarketplaceFee);
		})
	}
}

pub mod cancel_auction {
	pub use super::*;

	#[test]
	fn cancel_auction() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let mut deadlines = Deadlines::<Test>::get();

			assert_ok!(Auction::cancel_auction(origin(ALICE), ALICE_NFT_ID_1));

			// NFT.
			let nft = NFT::get_nft(ALICE_NFT_ID_1).unwrap();

			// Storage.
			deadlines.remove(ALICE_NFT_ID_1);

			assert_eq!(nft.state.is_auctioned, false);
			assert_eq!(nft.owner, ALICE);
			assert_eq!(Claims::<Test>::iter().count(), 0);

			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_1), None);
			assert_eq!(Deadlines::<Test>::get(), deadlines);

			// Check Events.
			let event = AuctionEvent::AuctionCancelled { nft_id: ALICE_NFT_ID_1 };
			let event = Event::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::cancel_auction(origin(ALICE), INVALID_NFT_ID);
			assert_noop!(err, Error::<Test>::AuctionDoesNotExist);
		})
	}

	#[test]
	fn not_the_auction_creator() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::cancel_auction(origin(BOB), ALICE_NFT_ID_1);
			assert_noop!(err, Error::<Test>::NotTheAuctionCreator);
		})
	}

	#[test]
	fn cannot_cancel_auction_in_progress() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			run_to_block(auction.start_block);

			let err = Auction::cancel_auction(alice, ALICE_NFT_ID_1);
			assert_noop!(err, Error::<Test>::CannotCancelAuctionInProgress);
		})
	}
}

pub mod end_auction {
	pub use super::*;

	#[test]
	fn end_auction() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000), (DAVE, 1_000)])
			.execute_with(|| {
				prepare_tests();
				let bob: mock::Origin = RawOrigin::Signed(BOB).into();

				Auction::create_auction(
					bob.clone(),
					BOB_NFT_ID,
					ALICE_MARKETPLACE_ID,
					DEFAULT_STARTBLOCK,
					DEFAULT_ENDBLOCK,
					DEFAULT_PRICE,
					Some(DEFAULT_PRICE + 1),
				)
				.unwrap();

				run_to_block(DEFAULT_STARTBLOCK);

				let alice_balance = Balances::free_balance(ALICE);

				let bob_balance = Balances::free_balance(BOB);
				let charlie_balance = Balances::free_balance(CHARLIE);
				let dave_balance = Balances::free_balance(CHARLIE);
				let auction = Auctions::<Test>::get(BOB_NFT_ID).unwrap();
				let marketplace = Marketplace::marketplaces(ALICE_MARKETPLACE_ID).unwrap();
				let commission_fee = marketplace.commission_fee.unwrap();
				let charlie_bid = auction.start_price + 10;
				let dave_bid = charlie_bid + 10;

				assert_eq!(commission_fee, CompoundFee::Percentage(PERCENT_20));
				assert_ok!(Auction::add_bid(origin(CHARLIE), BOB_NFT_ID, charlie_bid));

				run_to_block(DEFAULT_ENDBLOCK - 1);
				assert_ok!(Auction::add_bid(origin(DAVE), BOB_NFT_ID, dave_bid));
				assert_eq!(Balances::free_balance(Auction::account_id()), charlie_bid + dave_bid);

				run_to_block(DEFAULT_ENDBLOCK + 1);
				assert_ok!(Auction::end_auction(bob, BOB_NFT_ID));

				// Balance.
				let alice_new_balance = Balances::free_balance(ALICE);
				let bob_new_balance = Balances::free_balance(BOB);
				let charlie_new_balance = Balances::free_balance(CHARLIE);
				let dave_new_balance = Balances::free_balance(DAVE);
				let pallet_new_balance = Balances::free_balance(Auction::account_id());
				let marketplace_cut: u128 = match commission_fee {
					CompoundFee::Flat(x) => x,
					CompoundFee::Percentage(x) => x * dave_bid,
				};
				let artist_cut: u128 = dave_bid.saturating_sub(marketplace_cut.into());

				assert_eq!(alice_new_balance, alice_balance + marketplace_cut);
				assert_eq!(
					bob_new_balance,
					bob_balance + artist_cut + (dave_bid - artist_cut - marketplace_cut)
				);
				assert_eq!(charlie_new_balance, charlie_balance - charlie_bid);
				assert_eq!(dave_new_balance, dave_balance - dave_bid);
				assert_eq!(pallet_new_balance, charlie_bid);

				// NFT.
				let nft = NFT::get_nft(BOB_NFT_ID).unwrap();
				assert_eq!(nft.state.is_auctioned, false);
				assert_eq!(nft.owner, DAVE);

				assert_eq!(Claims::<Test>::iter().count(), 1);
				assert_eq!(Auctions::<Test>::get(BOB_NFT_ID), None);
				assert_eq!(Claims::<Test>::get(CHARLIE), Some(charlie_bid));

				// Check Events.
				let event = AuctionEvent::AuctionCompleted {
					nft_id: BOB_NFT_ID,
					new_owner: Some(DAVE),
					amount: Some(dave_bid),
					marketplace_cut: Some(marketplace_cut),
					royalty_cut: Some(PERCENT_0 * dave_bid),
				};
				let event = Event::Auction(event);
				System::assert_last_event(event);
			})
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::end_auction(origin(ALICE), INVALID_NFT_ID);
			assert_noop!(err, Error::<Test>::AuctionDoesNotExist);
		})
	}

	#[test]
	fn not_the_auction_creator() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::end_auction(origin(BOB), ALICE_NFT_ID_1);
			assert_noop!(err, Error::<Test>::NotTheAuctionCreator);
		})
	}

	#[test]
	fn cannot_end_auction_that_was_not_extended() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

			let err = Auction::end_auction(alice, ALICE_NFT_ID_1);
			assert_noop!(err, Error::<Test>::CannotEndAuctionThatWasNotExtended);
		})
	}
}

pub mod add_bid {
	pub use super::*;

	#[test]
	fn add_bid() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			let bob_balance = Balances::free_balance(BOB);

			let mut auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			run_to_block(auction.start_block);

			let bid = auction.start_price + 10;
			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid));

			// Balance.
			let bob_new_balance = Balances::free_balance(BOB);
			let pallet_new_balance = Balances::free_balance(Auction::account_id());
			assert_eq!(bob_new_balance, bob_balance - bid);
			assert_eq!(pallet_new_balance, bid);

			// Storage.
			auction.bidders.list = bounded_vec![(BOB, bid)];

			assert_eq!(Claims::<Test>::iter().count(), 0);
			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_1), Some(auction));

			// Check Events.
			let event = AuctionEvent::BidAdded { nft_id: ALICE_NFT_ID_1, bidder: BOB, amount: bid };
			let event = Event::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn add_bid_above_max_bidder_history_size() {
		ExtBuilder::new_build(vec![
			(ALICE, 1_000),
			(BOB, 1_000),
			(CHARLIE, 1_000),
			(DAVE, 1_000),
			(EVE, 1_000),
		])
		.execute_with(|| {
			prepare_tests();

			let eve_balance = Balances::free_balance(EVE);
			let mut auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			run_to_block(auction.start_block);

			let bob_bid = auction.start_price + 1;
			let charlie_bid = bob_bid + 1;
			let dave_bid = charlie_bid + 1;
			let eve_bid = dave_bid + 1;
			let mut accounts =
				vec![(BOB, bob_bid), (CHARLIE, charlie_bid), (DAVE, dave_bid), (EVE, eve_bid)];
			assert_eq!(accounts.len(), (BidderListLengthLimit::get() + 1) as usize);

			for bidder in accounts.iter() {
				assert_ok!(Auction::add_bid(origin(bidder.0), ALICE_NFT_ID_1, bidder.1));
			}

			// Balance.
			let eve_new_balance = Balances::free_balance(EVE);
			let pallet_new_balance = Balances::free_balance(Auction::account_id());
			assert_eq!(eve_new_balance, eve_balance - eve_bid);
			assert_eq!(pallet_new_balance, bob_bid + charlie_bid + dave_bid + eve_bid);

			// Storage.
			accounts.remove(0);
			let accounts: BoundedVec<(AccountId, u128), BidderListLengthLimit> =
				BoundedVec::try_from(accounts).unwrap();
			auction.bidders.list = accounts;

			assert_eq!(Claims::<Test>::iter().count(), 1);
			assert_eq!(Claims::<Test>::get(BOB), Some(bob_bid));
			assert_eq!(Auctions::<Test>::get(ALICE_NFT_ID_1), Some(auction));

			// Check Events.
			let event =
				AuctionEvent::BidAdded { nft_id: ALICE_NFT_ID_1, bidder: EVE, amount: eve_bid };
			let event = Event::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn add_bid_increase_auction_duration() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			let bob_balance = Balances::free_balance(BOB);
			let mut auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();
			let mut deadlines = Deadlines::<Test>::get();

			let grace_period = AUCTION_GRACE_PERIOD;
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
			let event = Event::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn add_bid_and_replace_current() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
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
			let event = Event::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let err = Auction::add_bid(origin(ALICE), INVALID_NFT_ID, 1);
			assert_noop!(err, Error::<Test>::AuctionDoesNotExist);
		})
	}

	#[test]
	fn cannot_add_bid_to_your_own_auctions() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let err = Auction::add_bid(origin(ALICE), ALICE_NFT_ID_1, 1);
			assert_noop!(err, Error::<Test>::CannotAddBidToYourOwnAuctions);
		})
	}

	#[test]
	fn auction_not_started() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, 1);
			assert_noop!(err, Error::<Test>::AuctionNotStarted);
		})
	}

	#[test]
	fn cannot_bid_less_than_the_highest_bid() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			let bob_bid = auction.start_price + 1;
			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bob_bid));

			let err = Auction::add_bid(origin(DAVE), ALICE_NFT_ID_1, bob_bid);
			assert_noop!(err, Error::<Test>::CannotBidLessThanTheHighestBid);
		})
	}

	#[test]
	fn cannot_bid_less_than_the_starting_price() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			let err = Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, auction.start_price - 1);
			assert_noop!(err, Error::<Test>::CannotBidLessThanTheStartingPrice);
		})
	}

	#[test]
	fn not_enough_funds() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			let balance = Balances::free_balance(BOB);
			let bid = balance + 1;
			assert!(bid > auction.start_price);

			let err = Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid);
			assert_noop!(err, BalanceError::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn not_enough_funds_to_replace() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();

			let bid = Balances::free_balance(BOB);
			assert!(bid > auction.start_price);

			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid));

			let err = Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid + 10);
			assert_noop!(err, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

pub mod remove_bid {
	pub use super::*;

	#[test]
	fn remove_bid() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			let bob: mock::Origin = RawOrigin::Signed(BOB).into();
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
			let event = Event::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::remove_bid(origin(ALICE), INVALID_NFT_ID);
			assert_noop!(err, Error::<Test>::AuctionDoesNotExist);
		})
	}

	#[test]
	fn cannot_remove_bid_at_the_end_of_auction() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			let auction = Auctions::<Test>::get(ALICE_NFT_ID_1).unwrap();
			let auction_end_period = AUCTION_ENDING_PERIOD;
			let target_block = auction.end_block - auction_end_period;

			run_to_block(DEFAULT_STARTBLOCK);

			let bid = auction.start_price + 1;
			assert_ok!(Auction::add_bid(origin(BOB), ALICE_NFT_ID_1, bid));

			run_to_block(target_block);

			let err = Auction::remove_bid(origin(BOB), ALICE_NFT_ID_1);
			assert_noop!(err, Error::<Test>::CannotRemoveBidAtTheEndOfAuction);
		})
	}

	#[test]
	fn bid_does_not_exist() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let err = Auction::remove_bid(origin(BOB), ALICE_NFT_ID_1);
			assert_noop!(err, Error::<Test>::BidDoesNotExist);
		})
	}
}

pub mod buy_it_now {
	pub use super::*;

	#[test]
	fn buy_it_now() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000)]).execute_with(
			|| {
				prepare_tests();
				let bob: mock::Origin = RawOrigin::Signed(BOB).into();

				Auction::create_auction(
					bob,
					BOB_NFT_ID,
					ALICE_MARKETPLACE_ID,
					DEFAULT_STARTBLOCK,
					DEFAULT_ENDBLOCK,
					DEFAULT_PRICE,
					Some(DEFAULT_PRICE + 1),
				)
				.unwrap();

				run_to_block(DEFAULT_STARTBLOCK);

				let alice_balance = Balances::free_balance(ALICE);
				let bob_balance = Balances::free_balance(BOB);
				let charlie_balance = Balances::free_balance(CHARLIE);
				let nft = NFT::get_nft(BOB_NFT_ID).unwrap();
				let auction = Auctions::<Test>::get(BOB_NFT_ID).unwrap();
				let price = auction.buy_it_price.unwrap();
				let marketplace = Marketplace::get_marketplace(ALICE_MARKETPLACE_ID).unwrap();
				let marketplace_cut = match marketplace.commission_fee.unwrap() {
					CompoundFee::Flat(x) => x,
					CompoundFee::Percentage(x) => x * price,
				};
				let artist_cut: u128 = nft.royalty * price.saturating_sub(marketplace_cut);
				let auctioneer_cut: u128 =
					price.saturating_sub(marketplace_cut).saturating_sub(artist_cut);

				assert_ok!(Auction::buy_it_now(origin(CHARLIE), BOB_NFT_ID));

				// Balance.
				let alice_new_balance = Balances::free_balance(ALICE);
				let bob_new_balance = Balances::free_balance(BOB);
				let charlie_new_balance = Balances::free_balance(CHARLIE);
				let pallet_new_balance = Balances::free_balance(Auction::account_id());

				assert_eq!(alice_new_balance, alice_balance + marketplace_cut);
				assert_eq!(bob_new_balance, bob_balance + artist_cut + auctioneer_cut);
				assert_eq!(charlie_new_balance, charlie_balance - price);
				assert_eq!(pallet_new_balance, 0);

				// NFT.
				let nft = NFT::get_nft(BOB_NFT_ID).unwrap();
				assert_eq!(nft.state.is_auctioned, false);
				assert_eq!(nft.owner, CHARLIE);

				// Storage.
				assert_eq!(Claims::<Test>::iter().count(), 0);
				assert_eq!(Auctions::<Test>::get(BOB_NFT_ID), None);

				// Check Events.
				let event = AuctionEvent::AuctionCompleted {
					nft_id: BOB_NFT_ID,
					new_owner: Some(CHARLIE),
					amount: Some(price),
					marketplace_cut: Some(marketplace_cut),
					royalty_cut: Some(artist_cut),
				};
				let event = Event::Auction(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn buy_it_now_with_existing_bids() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000)]).execute_with(
			|| {
				prepare_tests();
				run_to_block(DEFAULT_STARTBLOCK);

				let bob_balance = Balances::free_balance(BOB);
				let charlie_balance = Balances::free_balance(CHARLIE);
				let nft_id = ALICE_NFT_ID_1;
				let auction = Auctions::<Test>::get(nft_id).unwrap();

				let bob_bid = auction.start_price + 1;
				assert_ok!(Auction::add_bid(origin(BOB), nft_id, bob_bid));

				let price = auction.buy_it_price.unwrap();
				assert_ok!(Auction::buy_it_now(origin(CHARLIE), nft_id));

				// Balance.
				let bob_new_balance = Balances::free_balance(BOB);
				let charlie_new_balance = Balances::free_balance(CHARLIE);
				let pallet_new_balance = Balances::free_balance(Auction::account_id());

				assert_eq!(bob_new_balance, bob_balance - bob_bid);
				assert_eq!(charlie_new_balance, charlie_balance - price);
				assert_eq!(pallet_new_balance, bob_bid);

				// NFT.
				let nft = NFT::get_nft(nft_id).unwrap();
				assert_eq!(nft.state.is_auctioned, false);
				assert_eq!(nft.owner, CHARLIE);

				// Storage.
				assert_eq!(Claims::<Test>::iter().count(), 1);
				assert_eq!(Claims::<Test>::get(BOB), Some(bob_bid));
				assert_eq!(Auctions::<Test>::get(nft_id), None);

				// Check Events.
				let event = AuctionEvent::AuctionCompleted {
					nft_id,
					new_owner: Some(CHARLIE),
					amount: Some(price),
					marketplace_cut: Some(PERCENT_20 * price),
					royalty_cut: Some(PERCENT_0 * price),
				};
				let event = Event::Auction(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			let err = Auction::buy_it_now(origin(BOB), INVALID_NFT_ID);
			assert_noop!(err, Error::<Test>::AuctionDoesNotExist);
		})
	}

	#[test]
	fn auction_does_not_support_buy_it_now() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let nft_id = ALICE_NFT_ID_1;
			Auctions::<Test>::mutate(nft_id, |x| {
				let x = x.as_mut().unwrap();
				x.buy_it_price = None;
			});

			let err = Auction::buy_it_now(origin(BOB), nft_id);
			assert_noop!(err, Error::<Test>::AuctionDoesNotSupportBuyItNow);
		})
	}

	#[test]
	fn cannot_buy_it_now_to_your_own_auctions() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();
			run_to_block(DEFAULT_STARTBLOCK);

			let err = Auction::buy_it_now(origin(ALICE), ALICE_NFT_ID_1);
			assert_noop!(err, Error::<Test>::CannotBuyItNowToYourOwnAuctions);
		})
	}

	#[test]
	fn auction_not_started() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::buy_it_now(origin(BOB), ALICE_NFT_ID_1);
			assert_noop!(err, Error::<Test>::AuctionNotStarted);
		})
	}

	#[test]
	fn cannot_buy_it_when_a_bid_is_higher_than_buy_it_price() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000)]).execute_with(
			|| {
				prepare_tests();
				run_to_block(DEFAULT_STARTBLOCK);

				let nft_id = ALICE_NFT_ID_1;
				let auction = Auctions::<Test>::get(nft_id).unwrap();

				let price = auction.buy_it_price.unwrap();
				assert_ok!(Auction::add_bid(origin(CHARLIE), nft_id, price));

				let err = Auction::buy_it_now(origin(BOB), nft_id);
				assert_noop!(err, Error::<Test>::CannotBuyItWhenABidIsHigherThanBuyItPrice);
			},
		)
	}
}

pub mod complete_auction {
	pub use super::*;

	#[test]
	fn complete_auction_without_bid() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let nft_id = ALICE_NFT_ID_1;
			let auction = Auctions::<Test>::get(nft_id).unwrap();
			let mut deadlines = Deadlines::<Test>::get();

			assert_ok!(Auction::complete_auction(root(), nft_id));

			// NFT.
			let nft = NFT::get_nft(nft_id).unwrap();
			assert_eq!(nft.state.is_auctioned, false);
			assert_eq!(nft.owner, auction.creator);

			// Storage.
			deadlines.remove(nft_id);

			assert_eq!(Claims::<Test>::iter().count(), 0);
			assert_eq!(Auctions::<Test>::get(nft_id), None);
			assert_eq!(Deadlines::<Test>::get(), deadlines);

			// Event.
			let event = AuctionEvent::AuctionCompleted {
				nft_id,
				new_owner: None,
				amount: None,
				marketplace_cut: None,
				royalty_cut: None,
			};
			let event = Event::Auction(event);
			System::assert_last_event(event);
		})
	}

	#[test]
	fn complete_auction_with_one_bid() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000)]).execute_with(
			|| {
				prepare_tests();
				let bob: mock::Origin = RawOrigin::Signed(BOB).into();

				Auction::create_auction(
					bob,
					BOB_NFT_ID,
					ALICE_MARKETPLACE_ID,
					DEFAULT_STARTBLOCK,
					DEFAULT_ENDBLOCK,
					DEFAULT_PRICE,
					Some(DEFAULT_PRICE + 1),
				)
				.unwrap();

				run_to_block(DEFAULT_STARTBLOCK);

				let nft_id = BOB_NFT_ID;
				let alice_balance = Balances::free_balance(ALICE);
				let bob_balance = Balances::free_balance(BOB);
				let charlie_balance = Balances::free_balance(CHARLIE);
				let nft = NFT::get_nft(BOB_NFT_ID).unwrap();
				let auction = Auctions::<Test>::get(nft_id).unwrap();
				let bid = auction.start_price + 1;
				let marketplace = Marketplace::get_marketplace(ALICE_MARKETPLACE_ID).unwrap();
				let marketplace_fee = match marketplace.commission_fee.unwrap() {
					CompoundFee::Flat(x) => x,
					CompoundFee::Percentage(x) => x * bid,
				};
				let royalty_fee = nft.royalty * bid.saturating_sub(marketplace_fee);

				let mut deadlines = Deadlines::<Test>::get();
				assert_ok!(Auction::add_bid(origin(CHARLIE), nft_id, bid));
				assert_ok!(Auction::complete_auction(root(), nft_id));

				// Balance.
				let alice_new_balance = Balances::free_balance(ALICE);
				let bob_new_balance = Balances::free_balance(BOB);
				let charlie_new_balance = Balances::free_balance(CHARLIE);
				let pallet_new_balance = Balances::free_balance(Auction::account_id());

				assert_eq!(alice_new_balance, alice_balance + marketplace_fee);
				assert_eq!(
					bob_new_balance,
					bob_balance + royalty_fee + (bid - royalty_fee - marketplace_fee)
				);
				assert_eq!(charlie_new_balance, charlie_balance - bid);
				assert_eq!(pallet_new_balance, 0);

				// NFT.
				let nft = NFT::get_nft(nft_id).unwrap();
				assert_eq!(nft.state.is_auctioned, false);
				assert_eq!(nft.owner, CHARLIE);

				// Storage.
				deadlines.remove(nft_id);

				assert_eq!(Claims::<Test>::iter().count(), 0);
				assert_eq!(Auctions::<Test>::get(nft_id), None);
				assert_eq!(Deadlines::<Test>::get(), deadlines);

				// Event.
				let event = AuctionEvent::AuctionCompleted {
					nft_id,
					new_owner: Some(CHARLIE),
					amount: Some(bid),
					marketplace_cut: Some(marketplace_fee),
					royalty_cut: Some(royalty_fee),
				};
				let event = Event::Auction(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn complete_auction_with_two_bids() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000)]).execute_with(
			|| {
				prepare_tests();

				run_to_block(DEFAULT_STARTBLOCK);

				let nft_id = ALICE_NFT_ID_1;
				let alice_balance = Balances::free_balance(ALICE);
				let bob_balance = Balances::free_balance(BOB);
				let charlie_balance = Balances::free_balance(CHARLIE);
				let auction = Auctions::<Test>::get(nft_id).unwrap();
				let mut deadlines = Deadlines::<Test>::get();

				let bob_bid = auction.start_price + 1;
				let charlie_bid = bob_bid + 1;
				assert_ok!(Auction::add_bid(origin(BOB), nft_id, bob_bid));
				assert_ok!(Auction::add_bid(origin(CHARLIE), nft_id, charlie_bid));
				assert_ok!(Auction::complete_auction(root(), nft_id));

				// Balance.
				let alice_new_balance = Balances::free_balance(ALICE);
				let bob_new_balance = Balances::free_balance(BOB);
				let charlie_new_balance = Balances::free_balance(CHARLIE);
				let pallet_new_balance = Balances::free_balance(Auction::account_id());

				assert_eq!(alice_new_balance, alice_balance + charlie_bid);
				assert_eq!(bob_new_balance, bob_balance - bob_bid);
				assert_eq!(charlie_new_balance, charlie_balance - charlie_bid);
				assert_eq!(pallet_new_balance, bob_bid);

				// NFT.
				let nft = NFT::get_nft(nft_id).unwrap();
				assert_eq!(nft.state.is_auctioned, false);
				assert_eq!(nft.owner, CHARLIE);

				// Storage.
				deadlines.remove(nft_id);

				assert_eq!(Claims::<Test>::iter().count(), 1);
				assert_eq!(Claims::<Test>::get(BOB), Some(bob_bid));
				assert_eq!(Auctions::<Test>::get(nft_id), None);
				assert_eq!(Deadlines::<Test>::get(), deadlines);

				// Event.
				let event = AuctionEvent::AuctionCompleted {
					nft_id,
					new_owner: Some(CHARLIE),
					amount: Some(charlie_bid),
					marketplace_cut: Some(PERCENT_20 * charlie_bid),
					royalty_cut: Some(PERCENT_0 * charlie_bid),
				};
				let event = Event::Auction(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::complete_auction(origin(ALICE), ALICE_NFT_ID_1);
			assert_noop!(err, BadOrigin);
		})
	}

	#[test]
	fn auction_does_not_exist() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			let err = Auction::complete_auction(root(), INVALID_NFT_ID);
			assert_noop!(err, Error::<Test>::AuctionDoesNotExist);
		})
	}
}

pub mod claim {
	pub use super::*;

	#[test]
	fn claim() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000)]).execute_with(
			|| {
				prepare_tests();

				run_to_block(DEFAULT_STARTBLOCK);

				let nft_id = ALICE_NFT_ID_1;
				let bob_balance = Balances::free_balance(BOB);
				let pallet_balance = Balances::free_balance(Auction::account_id());
				let auction = Auctions::<Test>::get(nft_id).unwrap();

				let bob_bid = auction.start_price + 1;
				let charlie_bid = bob_bid + 1;
				assert_ok!(Auction::add_bid(origin(BOB), nft_id, bob_bid));
				assert_ok!(Auction::add_bid(origin(CHARLIE), nft_id, charlie_bid));
				assert_ok!(Auction::complete_auction(root(), nft_id));

				let claim = Claims::<Test>::get(BOB).unwrap();
				assert_ok!(Auction::claim(origin(BOB)));

				// Balance.
				let bob_new_balance = Balances::free_balance(BOB);

				assert_eq!(bob_new_balance, bob_balance);
				assert_eq!(pallet_balance, 0);
				assert_eq!(claim, bob_bid);

				// Storage.
				assert_eq!(Claims::<Test>::iter().count(), 0);
				assert_eq!(Claims::<Test>::get(BOB), None);
				// Event.
				let event = AuctionEvent::BalanceClaimed { account: BOB, amount: claim };
				let event = Event::Auction(event);
				System::assert_last_event(event);
			},
		)
	}

	#[test]
	fn claim_does_not_exist() {
		ExtBuilder::new_build(vec![(ALICE, 1_000), (BOB, 1_000)]).execute_with(|| {
			prepare_tests();

			let err = Auction::claim(origin(BOB));
			assert_noop!(err, Error::<Test>::ClaimDoesNotExist);
		})
	}
}
