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

use frame_support::{bounded_vec, parameter_types};

use crate::{BidderList, DeadlineList};

parameter_types! {
	pub const BidderListLengthLimit: u32 = 3;
	pub const ListLengthLimit10: u32 = 10;
	pub const ParallelAuctionLimit: u32 = 10;
}

mod bidder_list {
	use super::*;

	#[test]
	fn test_sorted_bid_works() {
		type MockBalance = u32;
		type MockAccount = u32;

		let mut bidders_list: BidderList<MockAccount, MockBalance, ListLengthLimit10> =
			BidderList::new();

		// insert to list works.
		bidders_list.insert_new_bid(1u32, 2u32);
		assert_eq!(bidders_list.list, vec![(1u32, 2u32)]);

		bidders_list.insert_new_bid(2u32, 3u32);
		assert_eq!(bidders_list.list, vec![(1u32, 2u32), (2u32, 3u32)]);

		// get highest bid works.
		assert_eq!(bidders_list.get_highest_bid(), Some(&(2u32, 3u32)));

		// get lowest bid works.
		assert_eq!(bidders_list.get_lowest_bid(), Some(&(1u32, 2u32)));

		// insert max bids.
		for n in 4..12 {
			bidders_list.insert_new_bid(n, n + 1);
		}

		// ensure the insertion has worked correctly.
		assert_eq!(
			bidders_list.list,
			vec![
				(1, 2),
				(2, 3),
				(4, 5),
				(5, 6),
				(6, 7),
				(7, 8),
				(8, 9),
				(9, 10),
				(10, 11),
				(11, 12)
			]
		);

		// inserting the new bid should replace the lowest bid.
		let lowest_bid = bidders_list.insert_new_bid(1u32, 102u32);
		assert_eq!(lowest_bid, Some((1, 2)));

		// ensure the insertion has worked correctly.
		assert_eq!(
			bidders_list.list,
			vec![
				(2, 3),
				(4, 5),
				(5, 6),
				(6, 7),
				(7, 8),
				(8, 9),
				(9, 10),
				(10, 11),
				(11, 12),
				(1, 102)
			]
		);

		// ensure find_bid works.
		assert_eq!(bidders_list.find_bid(&5), Some(&(5, 6)));
		assert_eq!(bidders_list.find_bid(&11), Some(&(11, 12)));
		assert_eq!(bidders_list.find_bid(&7), Some(&(7, 8)));
		assert_eq!(bidders_list.find_bid(&2021), None);

		// ensure remove_bid works.
		assert_eq!(bidders_list.remove_bid(&5), Some((5, 6)));
		assert_eq!(
			bidders_list.list,
			vec![(2, 3), (4, 5), (6, 7), (7, 8), (8, 9), (9, 10), (10, 11), (11, 12), (1, 102)]
		);

		// ensure remove_bid works.
		assert_eq!(bidders_list.remove_bid(&11), Some((11, 12)));
		assert_eq!(
			bidders_list.list,
			vec![(2, 3), (4, 5), (6, 7), (7, 8), (8, 9), (9, 10), (10, 11), (1, 102)]
		);
		assert_eq!(bidders_list.remove_bid(&2022), None);
		// ensure remove_bid works till empty.
		assert_eq!(bidders_list.remove_bid(&2), Some((2, 3)));
		assert_eq!(bidders_list.remove_bid(&4), Some((4, 5)));
		assert_eq!(bidders_list.remove_bid(&6), Some((6, 7)));
		assert_eq!(bidders_list.remove_bid(&7), Some((7, 8)));
		assert_eq!(bidders_list.remove_bid(&8), Some((8, 9)));
		assert_eq!(bidders_list.remove_bid(&9), Some((9, 10)));
		assert_eq!(bidders_list.remove_bid(&10), Some((10, 11)));
		assert_eq!(bidders_list.remove_bid(&1), Some((1, 102)));
		assert_eq!(bidders_list.list, vec![]);

		// insert max bids.
		for n in 4..12 {
			bidders_list.insert_new_bid(n, n + 1);
		}

		assert_eq!(bidders_list.remove_highest_bid(), Some((11, 12)));
		assert_eq!(bidders_list.remove_highest_bid(), Some((10, 11)));
	}
}

mod deadline_list {
	use frame_support::assert_ok;

	use super::*;

	#[test]
	fn insert_random_values() {
		let mut deadlines = DeadlineList::<u32, ParallelAuctionLimit>(bounded_vec![]);

		// Insert 5 different values and after every insert check if the order is correct.

		let entires = vec![
			(0, 100, vec![(0, 100)]),
			(1, 50, vec![(1, 50), (0, 100)]),
			(2, 150, vec![(1, 50), (0, 100), (2, 150)]),
			(3, 75, vec![(1, 50), (3, 75), (0, 100), (2, 150)]),
			(4, 25, vec![(4, 25), (1, 50), (3, 75), (0, 100), (2, 150)]),
		];

		for entry in entires {
			assert_ok!(deadlines.insert(entry.0, entry.1));
			assert_eq!(deadlines.0, entry.2);
		}
	}

	#[test]
	fn remove_random_values() {
		let mut deadlines = DeadlineList::<u32, ParallelAuctionLimit>(bounded_vec![]);

		// Insert 5 different values and after every insert check if the order is correct.

		let entires = vec![
			(0, 100, vec![]),
			(1, 50, vec![(0, 100)]),
			(2, 150, vec![(1, 50), (0, 100)]),
			(3, 75, vec![(1, 50), (0, 100), (2, 150)]),
			(4, 25, vec![(1, 50), (3, 75), (0, 100), (2, 150)]),
		];

		for entry in entires.iter() {
			assert_ok!(deadlines.insert(entry.0, entry.1));
		}

		for entry in entires.iter().rev() {
			let result = deadlines.remove(entry.0);
			assert_eq!(result, true);
			assert_eq!(deadlines.0, entry.2);
		}
	}

	#[test]
	fn update_values() {
		let mut deadlines = DeadlineList::<u32, ParallelAuctionLimit>(bounded_vec![]);

		// Insert 5 different values and after every insert check if the order is correct.

		let entires = vec![(0, 100), (1, 50), (2, 150)];
		let new_entires = vec![
			(0, 200, vec![(1, 50), (2, 150), (0, 200)]),
			(1, 175, vec![(2, 150), (1, 175), (0, 200)]),
			(1, 25, vec![(1, 25), (2, 150), (0, 200)]),
		];

		for entry in entires {
			assert_ok!(deadlines.insert(entry.0, entry.1));
		}

		for entry in new_entires {
			let index = deadlines.update(entry.0, entry.1);
			assert_eq!(index, true);
			assert_eq!(deadlines.0, entry.2);
		}
	}

	#[test]
	fn get_next_ready_blocks() {
		let mut deadlines = DeadlineList::<u32, ParallelAuctionLimit>(bounded_vec![]);

		// Insert 5 different values and after every insert check if the order is correct.

		let entries = vec![(0, 100), (1, 50), (2, 150)];
		for entry in entries.iter() {
			assert_ok!(deadlines.insert(entry.0, entry.1));
		}

		assert_eq!(deadlines.next(49), None);
		assert_eq!(deadlines.next(50), Some(1));

		let mut nfts = vec![];
		loop {
			if let Some(nft_id) = deadlines.next(500) {
				nfts.push(nft_id);
				deadlines.remove(nft_id);
			} else {
				break
			}
		}
		assert_eq!(nfts, vec![1, 0, 2]);
	}
}
