use frame_support::{traits::Get, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use primitives::{marketplace::MarketplaceId, nfts::NFTId};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Structure to store Auction data
pub struct AuctionData<AccountId, BlockNumber, Balance, ListLengthLimit>
where
	AccountId: sp_std::cmp::Ord + Clone,
	BlockNumber: Clone,
	Balance: sp_std::cmp::PartialOrd + Clone + Default,
	ListLengthLimit: Get<u32>,
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
	pub bidders: BidderList<AccountId, Balance, ListLengthLimit>,
	/// The marketplace where the auction has been listed
	pub marketplace_id: MarketplaceId,
	/// Is the auction going beyond the original end_block
	pub is_extended: bool,
}

impl<AccountId, BlockNumber, Balance, ListLengthLimit>
	AuctionData<AccountId, BlockNumber, Balance, ListLengthLimit>
where
	AccountId: sp_std::cmp::Ord + Clone,
	BlockNumber: Clone,
	Balance: sp_std::cmp::PartialOrd + Clone + Default,
	ListLengthLimit: Get<u32>,
{
	pub fn to_raw(&self, nft_id: NFTId) -> AuctionsGenesis<AccountId, BlockNumber, Balance> {
		(
			nft_id,
			self.creator.clone(),
			self.start_block.clone(),
			self.end_block.clone(),
			self.start_price.clone(),
			self.buy_it_price.clone(),
			self.bidders.to_raw(),
			self.marketplace_id.clone(),
			self.is_extended.clone(),
		)
	}

	pub fn from_raw(raw: AuctionsGenesis<AccountId, BlockNumber, Balance>) -> Self {
		Self {
			creator: raw.1,
			start_block: raw.2,
			end_block: raw.3,
			start_price: raw.4,
			buy_it_price: raw.5,
			bidders: BidderList::from_raw(raw.6),
			marketplace_id: raw.7,
			is_extended: raw.8,
		}
	}
}

// nft id, creator, start_block, end_block, start_price, buy it
pub type AuctionsGenesis<AccountId, BlockNumber, Balance> = (
	NFTId,
	AccountId,
	BlockNumber,
	BlockNumber,
	Balance,
	Option<Balance>,
	Vec<(AccountId, Balance)>,
	MarketplaceId,
	bool,
);

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// wrapper type to store sorted list of all bids
/// The wrapper exists to ensure a queue implementation of sorted bids
pub struct BidderList<AccountId, Balance, ListLengthLimit>
where
	AccountId: sp_std::cmp::Ord + Clone,
	Balance: sp_std::cmp::PartialOrd + Clone,
	ListLengthLimit: Get<u32>,
{
	pub list: BoundedVec<(AccountId, Balance), ListLengthLimit>,
}

impl<AccountId, Balance, ListLengthLimit> BidderList<AccountId, Balance, ListLengthLimit>
where
	AccountId: sp_std::cmp::Ord + Clone,
	Balance: sp_std::cmp::PartialOrd + Clone,
	ListLengthLimit: Get<u32>,
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
	pub fn remove_bid(&mut self, account_id: AccountId) -> Option<(AccountId, Balance)> {
		match self.list.iter().position(|x| x.0 == account_id) {
			Some(index) => Some(self.list.remove(index)),
			None => None,
		}
	}

	/// Return the bid of `account_id` if it exists
	pub fn find_bid(&self, account_id: AccountId) -> Option<&(AccountId, Balance)> {
		// this is not optimal since we traverse the entire link, but we cannot use binary search
		// here since the list is not sorted by accountId but rather by bid value, this should not
		// drastically affect performance as long as max_size remains small.
		self.list.iter().find(|&x| x.0 == account_id)
	}

	pub fn to_raw(&self) -> Vec<(AccountId, Balance)> {
		self.list.to_vec()
	}

	pub fn from_raw(raw: Vec<(AccountId, Balance)>) -> Self {
		let list = BoundedVec::try_from(raw).expect("It will never happen.");
		Self { list }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen)]
/// wrapper type to store sorted list of all bids
/// The wrapper exists to ensure a queue implementation of sorted bids
pub struct DeadlineList<BlockNumber, ParallelAuctionLimit: Get<u32>>(
	pub BoundedVec<(NFTId, BlockNumber), ParallelAuctionLimit>,
);

impl<BlockNumber, ParallelAuctionLimit> DeadlineList<BlockNumber, ParallelAuctionLimit>
where
	BlockNumber: sp_std::cmp::PartialOrd,
	ParallelAuctionLimit: Get<u32>,
{
	pub fn insert(&mut self, nft_id: NFTId, block_number: BlockNumber) -> Result<(), ()> {
		let index = self.0.iter().position(|x| x.1 > block_number);
		let index = index.unwrap_or_else(|| self.0.len());

		self.0.try_insert(index, (nft_id, block_number))
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
}
