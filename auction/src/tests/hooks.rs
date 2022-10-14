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
use frame_support::{assert_ok, bounded_vec, BoundedVec};
use frame_system::RawOrigin;
use primitives::{marketplace::MarketplaceType, nfts::NFTId};
use sp_runtime::Permill;

use crate::{
	tests::{extrinsics::AuctionBuilder, mock},
	types::{AuctionData, BidderList, DeadlineList},
	Auctions as AuctionsStorage, Config, Deadlines,
};

const PERCENT_0: Permill = Permill::from_parts(0);
const ALICE_NFT_ID_0: NFTId = 0;
const ALICE_MARKETPLACE_ID: u32 = 0;
const BOB_NFT_ID: NFTId = 1;

fn origin(account: u64) -> mock::RuntimeOrigin {
	RawOrigin::Signed(account).into()
}

#[test]
fn on_initialize() {
	ExtBuilder::new_build(None).execute_with(|| {
		let alice: mock::RuntimeOrigin = origin(ALICE);
		let bob: mock::RuntimeOrigin = origin(BOB);

		NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
		NFT::create_nft(bob.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
		Marketplace::create_marketplace(alice.clone(), MarketplaceType::Public).unwrap();

		let alice_start_block = 10;
		let alice_end_block = alice_start_block + MIN_AUCTION_DURATION;
		let alice_auction: AuctionData<AccountId, BlockNumber, u128, BidderListLengthLimit> =
			AuctionData {
				creator: ALICE,
				start_block: alice_start_block,
				end_block: alice_end_block,
				start_price: 300,
				buy_it_price: Some(400),
				bidders: BidderList::new(),
				marketplace_id: ALICE_MARKETPLACE_ID,
				is_extended: false,
			};

		let bob_start_block = 10 + 5;
		let bob_end_block = bob_start_block + MIN_AUCTION_DURATION;
		let bob_auction: AuctionData<AccountId, BlockNumber, u128, BidderListLengthLimit> =
			AuctionData {
				creator: BOB,
				start_block: bob_start_block,
				end_block: bob_end_block,
				start_price: 300,
				buy_it_price: Some(400),
				bidders: BidderList::new(),
				marketplace_id: ALICE_MARKETPLACE_ID,
				is_extended: false,
			};

		let ok = Auction::create_auction(
			alice,
			ALICE_NFT_ID_0,
			alice_auction.marketplace_id,
			alice_auction.start_block,
			alice_auction.end_block,
			alice_auction.start_price,
			alice_auction.buy_it_price,
		);
		assert_ok!(ok);

		let ok = Auction::create_auction(
			bob,
			BOB_NFT_ID,
			bob_auction.marketplace_id,
			bob_auction.start_block,
			bob_auction.end_block,
			bob_auction.start_price,
			bob_auction.buy_it_price,
		);
		assert_ok!(ok);

		// At block one we should have two auctions and two entries in deadlines.
		let deadlines = DeadlineList(bounded_vec![
			(ALICE_NFT_ID_0, alice_end_block),
			(BOB_NFT_ID, bob_end_block),
		]);

		assert_eq!(Deadlines::<Test>::get(), deadlines);
		assert_eq!(AuctionsStorage::<Test>::iter().count(), 2);
		assert!(AuctionsStorage::<Test>::contains_key(ALICE_NFT_ID_0));
		assert!(AuctionsStorage::<Test>::contains_key(BOB_NFT_ID));

		// At block alice_auction.end_block we should have 1 auction and 1 entry in deadlines.
		run_to_block(alice_auction.end_block);

		let deadlines = DeadlineList(bounded_vec![(BOB_NFT_ID, bob_end_block)]);

		assert_eq!(Deadlines::<Test>::get(), deadlines);
		assert_eq!(AuctionsStorage::<Test>::iter().count(), 1);
		assert!(AuctionsStorage::<Test>::contains_key(BOB_NFT_ID));

		// At block bob_auction.end_block we should have 0 auctions and 0 entries in deadlines.
		run_to_block(bob_auction.end_block);

		let deadlines = DeadlineList(bounded_vec![]);

		assert_eq!(Deadlines::<Test>::get(), deadlines);
		assert_eq!(AuctionsStorage::<Test>::iter().count(), 0);
	})
}

#[test]
fn auctions_in_block() {
	ExtBuilder::new_build(None).execute_with(|| {
		let alice: mock::RuntimeOrigin = origin(ALICE);

		let auctions_in_block = <Test as Config>::ActionsInBlockLimit::get();
		let offset = 10;

		// Create Marketplace
		Marketplace::create_marketplace(alice.clone(), MarketplaceType::Public).unwrap();

		// Create NFTs and auction them
		for _i in 0..auctions_in_block + offset {
			// Create NFTs
			NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
			let id = NFT::next_nft_id() - 1;

			AuctionBuilder::new().nft_id(id).execute().unwrap();
		}

		let default_auction = AuctionBuilder::new();

		run_to_block(default_auction.end - 1);

		let expected_len = auctions_in_block + offset;
		assert_eq!(Deadlines::<Test>::get().len(), expected_len as usize);
		assert_eq!(AuctionsStorage::<Test>::iter().count(), expected_len as usize);

		run_to_block(default_auction.end);
		assert_eq!(Deadlines::<Test>::get().len(), offset as usize);
		assert_eq!(AuctionsStorage::<Test>::iter().count(), offset as usize);

		run_to_block(default_auction.end + 1);
		assert_eq!(Deadlines::<Test>::get().len(), 0);
		assert_eq!(AuctionsStorage::<Test>::iter().count(), 0);
	})
}
