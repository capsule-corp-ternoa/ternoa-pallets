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
use frame_support::{assert_ok, bounded_vec};
use frame_system::RawOrigin;

use crate::{
	tests::mock,
	types::{AuctionData, BidderList, DeadlineList},
	Auctions as AuctionsStorage, Deadlines,
};

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

#[test]
fn on_initialize() {
	ExtBuilder::new_build(vec![], None).execute_with(|| {
		let (alice_nft_id, market_id) = (ALICE_NFT_ID, ALICE_MARKET_ID);
		let bob_nft_id = BOB_NFT_ID;

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
				marketplace_id: market_id,
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
				marketplace_id: market_id,
				is_extended: false,
			};

		let ok = Auction::create_auction(
			origin(ALICE),
			alice_nft_id,
			alice_auction.marketplace_id,
			alice_auction.start_block,
			alice_auction.end_block,
			alice_auction.start_price,
			alice_auction.buy_it_price.clone(),
		);
		assert_ok!(ok);

		let ok = Auction::create_auction(
			origin(BOB),
			bob_nft_id,
			bob_auction.marketplace_id,
			bob_auction.start_block,
			bob_auction.end_block,
			bob_auction.start_price,
			bob_auction.buy_it_price.clone(),
		);
		assert_ok!(ok);

		// At block one we should have two auctions and two entries in deadlines
		let deadlines = DeadlineList(bounded_vec![
			(ALICE_NFT_ID, alice_end_block),
			(BOB_NFT_ID, bob_end_block)
		]);

		assert_eq!(Deadlines::<Test>::get(), deadlines);
		assert_eq!(AuctionsStorage::<Test>::iter().count(), 2);
		assert!(AuctionsStorage::<Test>::contains_key(alice_nft_id));
		assert!(AuctionsStorage::<Test>::contains_key(bob_nft_id));

		// At block alice_auction.end_block we should have 1 auction and 1 entry in deadlines
		run_to_block(alice_auction.end_block);

		let deadlines = DeadlineList(bounded_vec![(BOB_NFT_ID, bob_end_block)]);

		assert_eq!(Deadlines::<Test>::get(), deadlines);
		assert_eq!(AuctionsStorage::<Test>::iter().count(), 1);
		assert!(AuctionsStorage::<Test>::contains_key(bob_nft_id));

		// At block bob_auction.end_block we should have 0 auctions and 0 entries in deadlines
		run_to_block(bob_auction.end_block);

		let deadlines = DeadlineList(bounded_vec![]);

		assert_eq!(Deadlines::<Test>::get(), deadlines);
		assert_eq!(AuctionsStorage::<Test>::iter().count(), 0);
	})
}
