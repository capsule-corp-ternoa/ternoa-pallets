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
use sp_runtime::{Permill, SaturatedConversion};
use sp_std::fmt::Debug;

pub type AccountList<AccountId, AccountSizeLimit> = BoundedVec<AccountId, AccountSizeLimit>;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct SubscriptionInput<BlockNumber: Clone> {
	pub period_length: BlockNumber,
	pub max_duration: Option<BlockNumber>,
	pub is_changeable: bool,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum DurationInput<BlockNumber: Clone> {
	Fixed(BlockNumber),
	Subscription(SubscriptionInput<BlockNumber>),
}

impl<BlockNumber: Clone> DurationInput<BlockNumber> {
	pub fn to_duration(self, max_duration: BlockNumber) -> Duration<BlockNumber> {
		match self {
			DurationInput::Fixed(x) => Duration::Fixed(x),
			DurationInput::Subscription(x) => Duration::Subscription(Subscription {
				period_length: x.period_length,
				max_duration: x.max_duration.unwrap_or_else(|| max_duration),
				is_changeable: x.is_changeable,
				new_terms: false,
			}),
		}
	}
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Subscription<BlockNumber: Clone> {
	pub period_length: BlockNumber,
	pub max_duration: BlockNumber,
	pub is_changeable: bool,
	pub new_terms: bool,
}

/// Enumeration of contract duration.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Duration<BlockNumber: Clone> {
	Fixed(BlockNumber),
	Subscription(Subscription<BlockNumber>),
}

impl<Blocknumber: Clone> Duration<Blocknumber> {
	pub fn allows_rent_fee<Balance: Clone>(&self, rent_fee: &RentFee<Balance>) -> Option<()> {
		match self {
			Self::Subscription(_) => rent_fee.get_nft().is_none(),
			_ => true,
		}
		.then(|| ())
	}

	pub fn allows_cancellation<Balance: Clone>(
		&self,
		cancellation: &CancellationFee<Balance>,
	) -> Option<()> {
		match self {
			Self::Fixed(_) => true,
			_ => cancellation.as_flexible().is_none(),
		}
		.then(|| ())
	}

	pub fn as_subscription(&self) -> Option<&Subscription<Blocknumber>> {
		match self {
			Self::Subscription(x) => Some(&x),
			_ => None,
		}
	}

	pub fn get_full_duration(&self) -> &Blocknumber {
		match self {
			Duration::Fixed(x) => &x,
			Duration::Subscription(x) => &x.max_duration,
		}
	}

	pub fn get_duration_or_period(&self) -> &Blocknumber {
		match self {
			Duration::Fixed(x) => &x,
			Duration::Subscription(x) => &x.period_length,
		}
	}

	pub fn queue_kind(&self) -> QueueKind {
		match self {
			Duration::Fixed(_) => QueueKind::Fixed,
			Duration::Subscription(_) => QueueKind::Subscription,
		}
	}

	pub fn terms_changed(&self) -> bool {
		match self {
			Duration::Subscription(x) => x.new_terms,
			_ => false,
		}
	}

	pub fn set_terms_changed(&mut self, value: bool) {
		match self {
			Duration::Subscription(x) => x.new_terms = value,
			_ => (),
		}
	}
}

/// Enumeration of contract acceptance type.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AcceptanceType<AccountList> {
	AutoAcceptance(Option<AccountList>),
	ManualAcceptance(Option<AccountList>),
}

impl<AccountList> AcceptanceType<AccountList> {
	pub fn get_allow_list(&self) -> &Option<AccountList> {
		match self {
			AcceptanceType::AutoAcceptance(x) => x,
			AcceptanceType::ManualAcceptance(x) => x,
		}
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
	None,
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

	pub fn as_flexible(&self) -> Option<Balance> {
		match self {
			CancellationFee::FlexibleTokens(x) => Some(x.clone()),
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
	/// Renter can cancel.
	pub renter_can_revoke: bool,
	/// Rent fee paid by rentee.
	pub rent_fee: RentFee<Balance>,
	/// Optional cancellation fee for renter.
	pub renter_cancellation_fee: CancellationFee<Balance>,
	/// Optional cancellation fee for rentee.
	pub rentee_cancellation_fee: CancellationFee<Balance>,
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
		start_block: Option<BlockNumber>,
		renter: AccountId,
		rentee: Option<AccountId>,
		duration: Duration<BlockNumber>,
		acceptance_type: AcceptanceType<AccountList<AccountId, AccountSizeLimit>>,
		renter_can_revoke: bool,
		rent_fee: RentFee<Balance>,
		renter_cancellation_fee: CancellationFee<Balance>,
		rentee_cancellation_fee: CancellationFee<Balance>,
	) -> RentContractData<AccountId, BlockNumber, Balance, AccountSizeLimit> {
		Self {
			start_block,
			renter,
			rentee,
			duration,
			acceptance_type,
			renter_can_revoke,
			rent_fee,
			renter_cancellation_fee,
			rentee_cancellation_fee,
		}
	}

	pub fn has_ended(&self, now: &BlockNumber) -> bool {
		let start = match self.start_block {
			Some(x) => x,
			None => return false,
		};

		let end = self.duration.get_full_duration();

		if start > *now {
			return false
		}

		(*now - start) >= *end
	}

	pub fn can_adjust_subscription(&self) -> bool {
		let is_changeable = self.duration.as_subscription().and_then(|x| Some(x.is_changeable));
		is_changeable.unwrap_or(false)
	}

	pub fn is_manual_acceptance(&self) -> bool {
		match self.acceptance_type {
			AcceptanceType::ManualAcceptance(_) => true,
			_ => false,
		}
	}

	pub fn percentage_of_completion(&self, now: &BlockNumber) -> Permill {
		let now: u32 = (*now).saturated_into();
		let start: u32 = self.start_block.expect("qed").saturated_into();
		let full_duration: u32 = self.duration.get_full_duration().clone().saturated_into();
		let ellapsed_duration: u32 = now - start;
		let percent = (ellapsed_duration as u32)
			.saturating_mul(100)
			.saturating_div(full_duration as u32);
		Permill::from_percent(percent)
	}

	pub fn is_renter(&self, account: &AccountId) -> Option<()> {
		(self.renter == *account).then(|| {})
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

pub enum QueueKind {
	Fixed,
	Subscription,
	Available,
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
	pub fn size(&self) -> u32 {
		(self.fixed_queue.0.len() + self.subscription_queue.0.len() + self.available_queue.0.len())
			as u32
	}

	/// Returns the addition of queues length.
	pub fn can_be_increased(&self, len: u32) -> Option<()> {
		(self.size() + len <= self.limit()).then(|| {})
	}

	pub fn insert(
		&mut self,
		nft_id: NFTId,
		block_number: BlockNumber,
		kind: QueueKind,
	) -> Result<(), ()> {
		match kind {
			QueueKind::Fixed => self.fixed_queue.insert(nft_id, block_number),
			QueueKind::Subscription => self.subscription_queue.insert(nft_id, block_number),
			QueueKind::Available => self.available_queue.insert(nft_id, block_number),
		}
	}

	pub fn remove(&mut self, nft_id: NFTId, kind: QueueKind) {
		match kind {
			QueueKind::Fixed => self.fixed_queue.remove(nft_id),
			QueueKind::Subscription => self.subscription_queue.remove(nft_id),
			QueueKind::Available => self.available_queue.remove(nft_id),
		};
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
