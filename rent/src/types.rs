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

use frame_support::{
	traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebug, RuntimeDebugNoBound,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use primitives::nfts::NFTId;
use scale_info::TypeInfo;
use sp_arithmetic::traits::AtLeast32BitUnsigned;
use sp_std::fmt::Debug;

pub type AccountList<AccountId, AccountSizeLimit> = BoundedVec<AccountId, AccountSizeLimit>;

/// Enumeration of contract duration.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Duration<BlockNumber: Clone> {
	Fixed(BlockNumber),
	Subscription(BlockNumber, Option<BlockNumber>),
}

impl<Blocknumber: Clone> Duration<Blocknumber> {
	pub fn allows_rent_fee<Balance: Clone>(&self, rent_fee: &RentFee<Balance>) -> Option<()> {
		match self {
			Self::Subscription(_, _) => rent_fee.get_nft().is_none(),
			_ => true,
		}
		.then(|| ())
	}

	pub fn allows_revocation(&self, revocation: &RevocationType) -> Option<()> {
		match self {
			Self::Subscription(_, _) => true,
			_ => *revocation != RevocationType::OnSubscriptionChange,
		}
		.then(|| ())
	}

	pub fn allows_cancellation<Balance: Clone>(
		&self,
		cancellation: &CancellationFee<Balance>,
	) -> Option<()> {
		match self {
			Self::Fixed(_) => true,
			_ => !matches!(*cancellation, CancellationFee::FlexibleTokens { .. }),
		}
		.then(|| ())
	}

	pub fn get_sub_period(&self) -> Option<Blocknumber> {
		match self {
			Self::Subscription(x, _) => Some(x.clone()),
			_ => None,
		}
	}

	pub fn is_subscription(&self) -> bool {
		match self {
			Self::Subscription(_, _) => true,
			_ => false,
		}
	}

	pub fn as_subscription(&self) -> Option<(&Blocknumber, &Option<Blocknumber>)> {
		match self {
			Self::Subscription(x, y) => Some((x, y)),
			_ => None,
		}
	}
}

/// Enumeration of contract acceptance type.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AcceptanceType<AccountList> {
	AutoAcceptance(Option<AccountList>),
	ManualAcceptance(Option<AccountList>),
}

/// Enumeration of contract revocation type.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RevocationType {
	NoRevocation,
	OnSubscriptionChange,
	Anytime,
}

impl RevocationType {
	pub fn allows_cancellation<Balance: Clone>(
		&self,
		_cancellation: &CancellationFee<Balance>,
	) -> Option<()> {
		match self {
			Self::NoRevocation => false,
			_ => true,
		}
		.then(|| ())
	}
}

/// Enumeration of contract rent fees.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RentFee<Balance>
where
	Balance: Clone,
{
	Tokens(Balance),
	NFT(NFTId),
}

impl<Balance> RentFee<Balance>
where
	Balance: Clone,
{
	pub fn get_balance(&self) -> Option<Balance> {
		match self {
			Self::Tokens(x) => Some(x.clone()),
			_ => None,
		}
	}

	pub fn get_nft(&self) -> Option<NFTId> {
		match self {
			Self::NFT(x) => Some(*x),
			_ => None,
		}
	}
}

/// Enumeration of contract rent fees.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum CancellationFee<Balance>
where
	Balance: Clone,
{
	FixedTokens(Balance),
	FlexibleTokens(Balance),
	NFT(NFTId),
}

impl<Balance> CancellationFee<Balance>
where
	Balance: Clone,
{
	pub fn get_balance(&self) -> Option<Balance> {
		match self {
			Self::FixedTokens(x) | Self::FlexibleTokens(x) => Some(x.clone()),
			_ => None,
		}
	}

	pub fn get_nft(&self) -> Option<NFTId> {
		match self {
			Self::NFT(x) => Some(*x),
			_ => None,
		}
	}
}

#[derive(
	Encode, Decode, CloneNoBound, Eq, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(AccountSizeLimit))]
#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen, BlockNumber: MaxEncodedLen))]
pub struct RentContractData<AccountId, BlockNumber, Balance, AccountSizeLimit>
where
	AccountId: Clone + PartialEq + Debug,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
	AccountSizeLimit: Get<u32>,
{
	/// Flag indicating if the renting contract has starter.
	pub has_started: bool,
	/// Start block of the contract.
	pub start_block: Option<BlockNumber>,
	/// Renter of the NFT.
	pub renter: AccountId,
	/// Rentee of the NFT.
	pub rentee: Option<AccountId>,
	/// Duration of the renting contract.
	pub duration: Duration<BlockNumber>,
	/// Acceptance type of the renting contract.
	pub acceptance_type: AcceptanceType<AccountList<AccountId, AccountSizeLimit>>,
	/// Revocation type of the renting contract.
	pub revocation_type: RevocationType,
	/// Rent fee paid by rentee.
	pub rent_fee: RentFee<Balance>,
	/// Flag indicating if terms were changed.
	pub terms_changed: bool,
	/// Optional cancellation fee for renter.
	pub renter_cancellation_fee: Option<CancellationFee<Balance>>,
	/// Optional cancellation fee for rentee.
	pub rentee_cancellation_fee: Option<CancellationFee<Balance>>,
}

impl<AccountId, BlockNumber, Balance, AccountSizeLimit>
	RentContractData<AccountId, BlockNumber, Balance, AccountSizeLimit>
where
	AccountId: Clone + PartialEq + Debug,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
	AccountSizeLimit: Get<u32>,
{
	pub fn new(
		has_started: bool,
		start_block: Option<BlockNumber>,
		renter: AccountId,
		rentee: Option<AccountId>,
		duration: Duration<BlockNumber>,
		acceptance_type: AcceptanceType<AccountList<AccountId, AccountSizeLimit>>,
		revocation_type: RevocationType,
		rent_fee: RentFee<Balance>,
		terms_changed: bool,
		renter_cancellation_fee: Option<CancellationFee<Balance>>,
		rentee_cancellation_fee: Option<CancellationFee<Balance>>,
	) -> RentContractData<AccountId, BlockNumber, Balance, AccountSizeLimit> {
		Self {
			has_started,
			start_block,
			renter,
			rentee,
			duration,
			acceptance_type,
			revocation_type,
			rent_fee,
			terms_changed,
			renter_cancellation_fee,
			rentee_cancellation_fee,
		}
	}

	pub fn has_ended(&self, now: &BlockNumber) -> bool {
		let start = match self.start_block {
			Some(x) => x,
			None => return false,
		};

		let end = match self.duration {
			Duration::Fixed(x) => Some(x),
			Duration::Subscription(_, x) => x,
		};

		let end = match end {
			Some(x) => x,
			None => return false,
		};

		if start > *now {
			return false
		}

		(*now - start) > end
	}

	pub fn can_adjust_subscription(&self) -> Option<()> {
		if matches!(self.revocation_type, RevocationType::OnSubscriptionChange { .. }) {
			return Some(())
		}
		if self.rentee.is_none() && self.duration.is_subscription() {
			return Some(())
		}

		None
	}
}

/// wrapper type to store queues of either fixed duration contracts, subscription contract or
/// available contract. The wrapper exists to ensure a queue implementation.
#[derive(
	Encode, Decode, CloneNoBound, Eq, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(Limit))]
#[codec(mel_bound(BlockNumber: MaxEncodedLen))]
pub struct Queue<BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd, Limit: Get<u32>>(
	pub BoundedVec<(NFTId, BlockNumber), Limit>,
);
impl<BlockNumber, Limit> Queue<BlockNumber, Limit>
where
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	Limit: Get<u32>,
{
	pub fn get(&mut self, nft_id: NFTId) -> Option<BlockNumber> {
		let index = self.0.iter().position(|x| x.0 == nft_id);
		if let Some(index) = index {
			Some(self.0[index].1.clone())
		} else {
			None
		}
	}

	pub fn size(&mut self) -> u32 {
		self.0.len() as u32
	}

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
}

impl<BlockNumber, Limit> Queue<BlockNumber, Limit>
where
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	Limit: Get<u32>,
{
	fn default() -> Self {
		Self(BoundedVec::default())
	}
}

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound(BlockNumber: MaxEncodedLen))]
#[scale_info(skip_type_params(Limit))]
pub struct RentingQueues<
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	Limit: Get<u32>,
> {
	pub fixed_queue: Queue<BlockNumber, Limit>,
	pub subscription_queue: Queue<BlockNumber, Limit>,
	pub available_queue: Queue<BlockNumber, Limit>,
}
impl<BlockNumber, Limit> RentingQueues<BlockNumber, Limit>
where
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	Limit: Get<u32>,
{
	/// Returns the queue limit.
	pub fn limit(&self) -> u32 {
		Limit::get()
	}

	/// Returns the addition of queues length.
	pub fn total_size(&mut self) -> u32 {
		(self.fixed_queue.0.len() + self.subscription_queue.0.len() + self.available_queue.0.len())
			as u32
	}

	/// Put the contract in available queue.
	pub fn insert_in_available_queue(
		&mut self,
		nft_id: NFTId,
		expiration_block: BlockNumber,
	) -> Result<(), ()> {
		self.available_queue.insert(nft_id, expiration_block)
	}

	/// Remove contract from available for rent queue.
	pub fn remove_from_available_queue(&mut self, nft_id: NFTId) -> bool {
		self.available_queue.remove(nft_id)
	}

	/// Put contract deadlines in fixed / subscription queue.
	pub fn insert_in_queue(
		&mut self,
		nft_id: NFTId,
		duration: &Duration<BlockNumber>,
		expiration_block: BlockNumber,
	) -> Result<(), ()> {
		match duration {
			Duration::Fixed(_) => self.fixed_queue.insert(nft_id, expiration_block).map_err(|_| ()),
			Duration::Subscription(_, _) =>
				self.subscription_queue.insert(nft_id, expiration_block).map_err(|_| ()),
		}
	}

	/// Remove a contract from all queues and remove offers if some exist.
	pub fn remove_from_queue(
		&mut self,
		nft_id: NFTId,
		has_started: bool,
		duration: &Duration<BlockNumber>,
	) -> bool {
		let mut removed = false;
		if !has_started {
			// Remove from available queue
			removed = self.available_queue.remove(nft_id);
		} else {
			// Remove from fixed queue
			if let Duration::Fixed(_) = duration {
				removed = self.fixed_queue.remove(nft_id);
			}

			// Remove from subscription queue
			if let Duration::Subscription(_, _) = duration {
				removed = self.subscription_queue.remove(nft_id);
			}
		};
		removed
	}
}
impl<BlockNumber, Limit> Default for RentingQueues<BlockNumber, Limit>
where
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	Limit: Get<u32>,
{
	fn default() -> Self {
		Self {
			fixed_queue: Queue::default(),
			subscription_queue: Queue::default(),
			available_queue: Queue::default(),
		}
	}
}
