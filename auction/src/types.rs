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

use frame_support::{traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use primitives::{marketplace::MarketplaceId, nfts::NFTId};
use scale_info::TypeInfo;
use sp_std::{fmt::Debug, vec::Vec};

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound(AccountId: MaxEncodedLen, BlockNumber: MaxEncodedLen, Balance: MaxEncodedLen))]
#[scale_info(skip_type_params(BidderListLengthLimit))]
/// Structure to store Auction data
pub struct AuctionData<AccountId, BlockNumber, Balance, BidderListLengthLimit>
where
	AccountId: Clone + PartialEq + Debug + sp_std::cmp::Ord,
	BlockNumber:
		Copy + PartialEq + Debug + sp_std::cmp::PartialOrd + sp_runtime::traits::Saturating,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	BidderListLengthLimit: Get<u32>,
{
	/// The owner of the nft that has listed the item on auction
	pub creator: AccountId,
	/// `BlockNumber` at which the auction will accept bids
	pub start_block: BlockNumber,
	/// `BlockNumber` at which the auction will no longer accept bids
	pub end_block: BlockNumber,
	/// Floor `Balance` for creating a bid
	pub start_price: Balance,
	/// Optional price at which the auction is stopped and item can be bought
	pub buy_it_price: Option<Balance>,
	/// List of bidders
	pub bidders: BidderList<AccountId, Balance, BidderListLengthLimit>,
	/// The marketplace where the auction has been listed
	pub marketplace_id: MarketplaceId,
	/// Is the auction going beyond the original end_block
	pub is_extended: bool,
}

impl<AccountId, BlockNumber, Balance, BidderListLengthLimit>
	AuctionData<AccountId, BlockNumber, Balance, BidderListLengthLimit>
where
	AccountId: Clone + PartialEq + Debug + sp_std::cmp::Ord,
	BlockNumber:
		Copy + PartialEq + Debug + sp_std::cmp::PartialOrd + sp_runtime::traits::Saturating,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	BidderListLengthLimit: Get<u32>,
{
	pub fn pop_highest_bid(&mut self) -> Option<(AccountId, Balance)> {
		self.bidders.remove_highest_bid()
	}

	pub fn get_bidders(&self) -> &BoundedVec<(AccountId, Balance), BidderListLengthLimit> {
		&self.bidders.list
	}

	pub fn get_highest_bid(&self) -> Option<&(AccountId, Balance)> {
		self.bidders.get_highest_bid()
	}

	pub fn has_started(&self, now: BlockNumber) -> bool {
		now >= self.start_block
	}

	pub fn is_creator(&self, account_id: &AccountId) -> bool {
		self.creator == *account_id
	}

	pub fn for_each_bidder(&self, f: &dyn Fn(&(AccountId, Balance))) {
		self.bidders.list.iter().for_each(f);
	}

	/// Remove a specific bid from `account_id` from list if it exists
	pub fn remove_bid(&mut self, account_id: &AccountId) -> Option<(AccountId, Balance)> {
		self.bidders.remove_bid(account_id)
	}

	/// Return the bid of `account_id` if it exists
	pub fn find_bid(&self, account_id: &AccountId) -> Option<&(AccountId, Balance)> {
		self.bidders.find_bid(account_id)
	}

	pub fn insert_new_bid(
		&mut self,
		account_id: AccountId,
		value: Balance,
	) -> Option<(AccountId, Balance)> {
		self.bidders.insert_new_bid(account_id, value)
	}

	pub fn extend_if_necessary(
		&mut self,
		now: BlockNumber,
		grace_period: BlockNumber,
	) -> Option<BlockNumber> {
		let end_block = self.end_block;
		let remaining_blocks = end_block.saturating_sub(now);

		if remaining_blocks < grace_period {
			let blocks_to_add = grace_period.saturating_sub(remaining_blocks);

			self.end_block = end_block.saturating_add(blocks_to_add);
			self.is_extended = true;

			return Some(self.end_block)
		}

		None
	}
}

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen))]
#[scale_info(skip_type_params(BidderListLengthLimit))]
/// wrapper type to store sorted list of all bids
/// The wrapper exists to ensure a queue implementation of sorted bids
pub struct BidderList<AccountId, Balance, BidderListLengthLimit>
where
	AccountId: Clone + PartialEq + Debug + sp_std::cmp::Ord,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	BidderListLengthLimit: Get<u32>,
{
	pub list: BoundedVec<(AccountId, Balance), BidderListLengthLimit>,
}

impl<AccountId, Balance, BidderListLengthLimit>
	BidderList<AccountId, Balance, BidderListLengthLimit>
where
	AccountId: Clone + PartialEq + Debug + sp_std::cmp::Ord,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	BidderListLengthLimit: Get<u32>,
{
	/// Create a new empty bidders list
	pub fn new() -> Self {
		Self { list: BoundedVec::default() }
	}

	/// Insert a new bid to the list
	pub fn insert_new_bid(
		&mut self,
		account_id: AccountId,
		value: Balance,
	) -> Option<(AccountId, Balance)> {
		// If list is at max capacity, remove lowest bid
		if self.list.is_full() {
			let removed_bid = self.list.remove(0);
			self.list.try_push((account_id, value)).expect("Cannot happen.");
			// return removed bid
			Some(removed_bid)
		} else {
			self.list.try_push((account_id, value)).expect("Cannot happen.");
			None
		}
	}

	/// Get length of bidders list
	pub fn len(&self) -> usize {
		self.list.len()
	}

	/// Get current highest bid in list
	pub fn get_highest_bid(&self) -> Option<&(AccountId, Balance)> {
		self.list.last()
	}

	/// Get current lowest bid in list
	pub fn get_lowest_bid(&self) -> Option<&(AccountId, Balance)> {
		self.list.first()
	}

	/// Remove the lowest bid in list
	pub fn remove_lowest_bid(&mut self) -> (AccountId, Balance) {
		self.list.remove(0)
	}

	/// Remove the highest bid in list
	pub fn remove_highest_bid(&mut self) -> Option<(AccountId, Balance)> {
		match self.list.len() {
			0 => None,
			n => Some(self.list.remove(n - 1)),
		}
	}

	/// Remove a specific bid from `account_id` from list if it exists
	pub fn remove_bid(&mut self, account_id: &AccountId) -> Option<(AccountId, Balance)> {
		match self.list.iter().position(|x| x.0 == *account_id) {
			Some(index) => Some(self.list.remove(index)),
			None => None,
		}
	}

	/// Return the bid of `account_id` if it exists
	pub fn find_bid(&self, account_id: &AccountId) -> Option<&(AccountId, Balance)> {
		// this is not optimal since we traverse the entire link, but we cannot use binary search
		// here since the list is not sorted by accountId but rather by bid value, this should not
		// drastically affect performance as long as max_size remains small.
		self.list.iter().find(|&x| x.0 == *account_id)
	}

	pub fn to_raw(&self) -> Vec<(AccountId, Balance)> {
		self.list.to_vec()
	}

	pub fn from_raw(raw: Vec<(AccountId, Balance)>) -> Self {
		let list = BoundedVec::try_from(raw).expect("It will never happen.");
		Self { list }
	}
}

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound(BlockNumber: MaxEncodedLen))]
#[scale_info(skip_type_params(ParallelAuctionLimit))]
/// wrapper type to store sorted list of all bids
/// The wrapper exists to ensure a queue implementation of sorted bids
pub struct DeadlineList<
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	ParallelAuctionLimit: Get<u32>,
>(pub BoundedVec<(NFTId, BlockNumber), ParallelAuctionLimit>);

impl<BlockNumber, ParallelAuctionLimit> DeadlineList<BlockNumber, ParallelAuctionLimit>
where
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	ParallelAuctionLimit: Get<u32>,
{
	pub fn insert(&mut self, nft_id: NFTId, block_number: BlockNumber) -> Result<(), ()> {
		let index = self.0.iter().position(|x| x.1 > block_number);
		let index = index.unwrap_or_else(|| self.0.len());

		self.0.try_insert(index, (nft_id, block_number))
	}

	pub fn bulk_inser(&mut self, nft_id: NFTId, block_number: BlockNumber, number: u32) -> Result<(), ()> {
		let data = vec![(nft_id, block_number); number as usize];
		self.0.try_extend(BoundedVec::try_from(data))
	}

	pub fn remove(&mut self, nft_id: NFTId) -> bool {
		let index = self.0.iter().position(|x| x.0 == nft_id);
		if let Some(index) = index {
			self.0.remove(index);
			true
		} else {
			false
		}
	}

	pub fn update(&mut self, nft_id: NFTId, block_number: BlockNumber) -> bool {
		let removed = self.remove(nft_id);
		if removed {
			self.insert(nft_id, block_number).expect("Cannot happen.");
			true
		} else {
			false
		}
	}

	pub fn next(&self, block_number: BlockNumber) -> Option<NFTId> {
		let front = self.0.get(0)?;
		if front.1 <= block_number {
			Some(front.0)
		} else {
			None
		}
	}

	pub fn pop_next(&mut self, block_number: BlockNumber) -> Option<NFTId> {
		let front = self.0.get(0)?;
		if front.1 <= block_number {
			let nft_id = front.0;
			self.remove(nft_id);
			Some(nft_id)
		} else {
			None
		}
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}
}

impl<BlockNumber, ParallelAuctionLimit> Default for DeadlineList<BlockNumber, ParallelAuctionLimit>
where
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	ParallelAuctionLimit: Get<u32>,
{
	fn default() -> Self {
		Self(BoundedVec::default())
	}
}
