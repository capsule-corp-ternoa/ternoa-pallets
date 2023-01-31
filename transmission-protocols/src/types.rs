// Copyright 2023 Capsule Corp (France) SAS.
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
use sp_std::{fmt::Debug, vec};

pub type ConsentList<AccountId, MaxConsentListSize> = BoundedVec<AccountId, MaxConsentListSize>;

/// Enumeration of Transmission protocols kind.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TransmissionProtocolKind {
	AtBlock,
	AtBlockWithReset,
	OnConsent,
	OnConsentAtBlock,
}

/// Enumeration of Transmission protocols.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TransmissionProtocol<BlockNumber: Clone, ConsentList> {
	AtBlock(BlockNumber),
	AtBlockWithReset(BlockNumber),
	OnConsent { consent_list: ConsentList, threshold: u8 },
	OnConsentAtBlock { consent_list: ConsentList, threshold: u8, block: BlockNumber },
}

impl<BlockNumber: Clone, ConsentList> TransmissionProtocol<BlockNumber, ConsentList> {
	pub fn to_kind(&self) -> TransmissionProtocolKind {
		match self {
			TransmissionProtocol::AtBlock(_) => TransmissionProtocolKind::AtBlock,
			TransmissionProtocol::AtBlockWithReset(_) => TransmissionProtocolKind::AtBlockWithReset,
			TransmissionProtocol::OnConsent { consent_list: _, threshold: _ } =>
				TransmissionProtocolKind::OnConsent,
			TransmissionProtocol::OnConsentAtBlock { consent_list: _, threshold: _, block: _ } =>
				TransmissionProtocolKind::OnConsentAtBlock,
		}
	}

	pub fn get_end_block(&self) -> Option<BlockNumber> {
		match self {
			TransmissionProtocol::AtBlock(x) => Some(x.clone()),
			TransmissionProtocol::AtBlockWithReset(x) => Some(x.clone()),
			TransmissionProtocol::OnConsent { consent_list: _, threshold: _ } => None,
			TransmissionProtocol::OnConsentAtBlock { consent_list: _, threshold: _, block } =>
				Some(block.clone()),
		}
	}

	pub fn get_block_to_queue(&self) -> Option<BlockNumber> {
		match self {
			TransmissionProtocol::AtBlock(x) => Some(x.clone()),
			TransmissionProtocol::AtBlockWithReset(x) => Some(x.clone()),
			TransmissionProtocol::OnConsent { consent_list: _, threshold: _ } => None,
			TransmissionProtocol::OnConsentAtBlock { consent_list: _, threshold: _, block: _ } =>
				None,
		}
	}

	pub fn can_reset_timer(&self) -> bool {
		match self {
			TransmissionProtocol::AtBlock(_) => false,
			TransmissionProtocol::AtBlockWithReset(_) => true,
			TransmissionProtocol::OnConsent { consent_list: _, threshold: _ } => false,
			TransmissionProtocol::OnConsentAtBlock { consent_list: _, threshold: _, block: _ } =>
				false,
		}
	}

	pub fn can_add_consent(&self) -> bool {
		match self {
			TransmissionProtocol::AtBlock(_) => false,
			TransmissionProtocol::AtBlockWithReset(_) => false,
			TransmissionProtocol::OnConsent { consent_list: _, threshold: _ } => true,
			TransmissionProtocol::OnConsentAtBlock { consent_list: _, threshold: _, block: _ } =>
				true,
		}
	}

	pub fn get_consent_list(&self) -> Option<&ConsentList> {
		match self {
			TransmissionProtocol::AtBlock(_) => None,
			TransmissionProtocol::AtBlockWithReset(_) => None,
			TransmissionProtocol::OnConsent { consent_list, threshold: _ } => Some(consent_list),
			TransmissionProtocol::OnConsentAtBlock { consent_list, threshold: _, block: _ } =>
				Some(consent_list),
		}
	}

	pub fn get_consent_data(&self) -> Option<(&ConsentList, u8)> {
		match self {
			TransmissionProtocol::AtBlock(_) => None,
			TransmissionProtocol::AtBlockWithReset(_) => None,
			TransmissionProtocol::OnConsent { consent_list, threshold } =>
				Some((consent_list, *threshold)),
			TransmissionProtocol::OnConsentAtBlock { consent_list, threshold, block: _ } =>
				Some((consent_list, *threshold)),
		}
	}

	pub fn get_threshold(&self) -> Option<u8> {
		match self {
			TransmissionProtocol::AtBlock(_) => None,
			TransmissionProtocol::AtBlockWithReset(_) => None,
			TransmissionProtocol::OnConsent { consent_list: _, threshold } => Some(*threshold),
			TransmissionProtocol::OnConsentAtBlock { consent_list: _, threshold, block: _ } =>
				Some(*threshold),
		}
	}
}

/// Enumeration of Cancellation period.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum CancellationPeriod<BlockNumber: Clone> {
	None,
	UntilBlock(BlockNumber),
	Anytime,
}

impl<BlockNumber: Clone> CancellationPeriod<BlockNumber>
where
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
{
	pub fn is_cancellable(&self, now: BlockNumber) -> bool {
		match self {
			CancellationPeriod::None => false,
			CancellationPeriod::UntilBlock(block) => *block >= now,
			CancellationPeriod::Anytime => true,
		}
	}
}

/// Transmission data structure.
#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, Eq, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(MaxConsentListSize))]
#[codec(mel_bound(AccountId: MaxEncodedLen, BlockNumber: MaxEncodedLen))]
pub struct TransmissionData<AccountId, BlockNumber, MaxConsentListSize>
where
	AccountId: Clone + PartialEq + Debug,
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
	MaxConsentListSize: Get<u32>,
{
	pub recipient: AccountId,
	pub protocol: TransmissionProtocol<BlockNumber, ConsentList<AccountId, MaxConsentListSize>>,
	pub cancellation: CancellationPeriod<BlockNumber>,
}
impl<AccountId, BlockNumber, MaxConsentListSize>
	TransmissionData<AccountId, BlockNumber, MaxConsentListSize>
where
	AccountId: Clone + PartialEq + Debug,
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
	MaxConsentListSize: Get<u32>,
{
	pub fn new(
		recipient: AccountId,
		protocol: TransmissionProtocol<BlockNumber, ConsentList<AccountId, MaxConsentListSize>>,
		cancellation: CancellationPeriod<BlockNumber>,
	) -> Self {
		Self { recipient, protocol, cancellation }
	}
}

/// Queue containing the nft id that must be transferred at the specified block.
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
	/// Returns an empty queue.
	fn default() -> Self {
		Self(BoundedVec::default())
	}

	/// Get the block number for the spcified NFT id if it exist.
	pub fn get(&mut self, nft_id: NFTId) -> Option<BlockNumber> {
		let index = self.0.iter().position(|x| x.0 == nft_id);
		if let Some(index) = index {
			Some(self.0[index].1.clone())
		} else {
			None
		}
	}

	/// Returns the current size of the queue.
	pub fn size(&self) -> u32 {
		self.0.len() as u32
	}

	/// Inserts a value in the queue in the correct position depending on the block number.
	pub fn insert(&mut self, nft_id: NFTId, block_number: BlockNumber) -> Result<(), ()> {
		let index = self.0.iter().position(|x| x.1 > block_number);
		let index = index.unwrap_or_else(|| self.0.len());

		self.0.try_insert(index, (nft_id, block_number))
	}

	/// Remove a value in the queue if it exists.
	pub fn remove(&mut self, nft_id: NFTId) -> bool {
		let index = self.0.iter().position(|x| x.0 == nft_id);
		if let Some(index) = index {
			self.0.remove(index);
			true
		} else {
			false
		}
	}

	/// Update a value in the list.
	pub fn update(&mut self, nft_id: NFTId, block_number: BlockNumber) -> bool {
		let removed = self.remove(nft_id);
		if removed {
			self.insert(nft_id, block_number).expect("Cannot happen.");
			true
		} else {
			false
		}
	}

	/// Return the first value of the queue.
	pub fn next(&self, block_number: BlockNumber) -> Option<NFTId> {
		let front = self.0.get(0)?;
		if front.1 <= block_number {
			Some(front.0)
		} else {
			None
		}
	}

	/// Pop and return the first value of the queue.
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

	/// Returns the queue limit.
	pub fn limit(&self) -> u32 {
		Limit::get()
	}

	/// Returns the addition of queues length.
	pub fn can_be_increased(&self, len: u32) -> Option<()> {
		(self.size() + len <= self.limit()).then(|| {})
	}

	// Benchmark / tests only
	pub fn bulk_insert(
		&mut self,
		nft_id: NFTId,
		block_number: BlockNumber,
		number: u32,
	) -> Result<(), ()> {
		self.0.try_extend(vec![(nft_id, block_number); number as usize].into_iter())
	}
}
impl<BlockNumber, Limit> Default for Queue<BlockNumber, Limit>
where
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	Limit: Get<u32>,
{
	fn default() -> Self {
		Self::default()
	}
}
