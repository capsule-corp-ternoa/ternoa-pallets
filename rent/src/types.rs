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

use frame_support::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, BoundedVec, RuntimeDebug, traits::Get};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use primitives::nfts::NFTId;
use scale_info::TypeInfo;
use sp_std::fmt::Debug;

pub type AccountList<AccountId, AccountSizeLimit> = BoundedVec<AccountId, AccountSizeLimit>;

/// Enumeration of contract duration.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Duration<BlockNumber> {
	Fixed(BlockNumber),
	Subscription(BlockNumber, Option<BlockNumber>),
	Infinite,
}

/// Enumeration of contract acceptance type.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AcceptanceType<AccountList> {
	AutoAcceptance(Option<AccountList>),
	ManualAcceptance(Option<AccountList>),
}

/// Enumeration of contract revocation type.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RevocationType<Balance, BlockNumber> {
	NoRevocation,
	OnSubscriptionChange(Duration<BlockNumber>, Balance),
	Anytime,
}

/// Enumeration of contract rent fees.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RentFee<Balance> {
	Tokens(Balance),
	NFT(NFTId),
}

/// Enumeration of contract rent fees.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum CancellationFee<Balance> {
	FixedTokens(Balance),
  	FlexibleTokens(Balance),
  	NFT(NFTId),
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
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	AccountSizeLimit: Get<u32>,
{
	/// Flag indicating if the renting contract has starter.
	pub has_started:               bool,
	/// Renter of the NFT.
  	pub renter:                    AccountId,
	/// Rentee of the NFT.
	pub rentee:                    Option<AccountId>,
	/// Duration of the renting contract.
	pub duration:                  Duration<BlockNumber>,
	/// Acceptance type of the renting contract.
	pub acceptance_type:           AcceptanceType<AccountList<AccountId, AccountSizeLimit>>,
	/// Revocation type of the renting contract.
	pub revocation_type:           RevocationType<Balance, BlockNumber>,
	/// Rent fee paid by rentee.
	pub rent_fee:                  RentFee<Balance>,
	/// Flag indicating if terms were accepted in case of change.
	pub terms_accepted:            bool,
	/// Optional cancellation fee for renter.
	pub renter_cancellation_fee:   Option<CancellationFee<Balance>>,
	/// Optional cancellation fee for rentee.
	pub rentee_cancellation_fee:   Option<CancellationFee<Balance>>,
}

impl<AccountId, BlockNumber, Balance, AccountSizeLimit> RentContractData<AccountId, BlockNumber, Balance, AccountSizeLimit>
where
	AccountId: Clone + PartialEq + Debug,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	AccountSizeLimit: Get<u32>,
{
	pub fn new(
		has_started: bool,
		renter: AccountId,
		rentee: Option<AccountId>,
		duration: Duration<BlockNumber>,
		acceptance_type: AcceptanceType<AccountList<AccountId, AccountSizeLimit>>,
		revocation_type: RevocationType<Balance, BlockNumber>,
		rent_fee: RentFee<Balance>,
		terms_accepted: bool,
		renter_cancellation_fee: Option<CancellationFee<Balance>>,
		rentee_cancellation_fee: Option<CancellationFee<Balance>>,
	) -> RentContractData<AccountId, BlockNumber, Balance, AccountSizeLimit> {
		Self { 
			has_started,
			renter,
			rentee,
			duration,
			acceptance_type,
			revocation_type,
			rent_fee,
			terms_accepted,
			renter_cancellation_fee,
			rentee_cancellation_fee,
		}
	}
}

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound(BlockNumber: MaxEncodedLen))]
#[scale_info(skip_type_params(Limit))]
/// wrapper type to store queues of either fixed duration contracts, subscription contract or available contract.
/// The wrapper exists to ensure a queue implementation.
pub struct Queue<
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	Limit: Get<u32>,
>(pub BoundedVec<(NFTId, BlockNumber), Limit>);

impl<BlockNumber, Limit> Queue<BlockNumber, Limit>
where
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	Limit: Get<u32>,
{
	pub fn get(&mut self, nft_id: NFTId) -> Option<BlockNumber> {
		let index = self.0.iter().position(|x| x.0 == nft_id);
		if let Some(index) = index{
			Some(self.0[index].1.clone())
		} else {
			None
		}
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
}

impl<BlockNumber, Limit> Default for Queue<BlockNumber, Limit>
where
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	Limit: Get<u32>,
{
	fn default() -> Self {
		Self(BoundedVec::default())
	}
}