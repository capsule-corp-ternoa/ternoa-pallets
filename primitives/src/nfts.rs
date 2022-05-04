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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{fmt::Debug, vec::Vec};

use crate::U8BoundedVec;

/// How NFT IDs are encoded.
pub type NFTId = u32;

/// How NFT IDs are encoded. In the JSON Types this should be "Text" and not "Vec<8>".
pub type NFTSeriesId = Vec<u8>;

/// IPFS Reference Type
pub type IPFSReference<S> = U8BoundedVec<S>;

/// Data related to an NFT, such as who is its owner.
#[derive(
	Encode, Decode, Eq, Default, TypeInfo, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
)]
#[scale_info(skip_type_params(IPFSLengthLimit))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct NFTData<AccountId, IPFSLengthLimit>
where
	AccountId: Clone + PartialEq + Debug,
	IPFSLengthLimit: Get<u32>,
{
	// NFT owner
	pub owner: AccountId,
	// NFT creator
	pub creator: AccountId,
	// IPFS reference
	pub ipfs_reference: IPFSReference<IPFSLengthLimit>,
	// Series ID
	pub series_id: NFTSeriesId,
	// Is listed for sale
	pub listed_for_sale: bool,
	// Is being transmitted
	pub is_in_transmission: bool,
	// Is NFT converted to capsule
	pub is_capsule: bool,
	// Is secret
	pub is_secret: bool,
	// Delegated
	pub is_delegated: bool,
	// Royalties fee
	pub royaltie_fee: u8,
}

impl<AccountId, IPFSLengthLimit> NFTData<AccountId, IPFSLengthLimit>
where
	AccountId: Clone + PartialEq + Debug,
	IPFSLengthLimit: Get<u32>,
{
	pub fn new(
		owner: AccountId,
		creator: AccountId,
		ipfs_reference: IPFSReference<IPFSLengthLimit>,
		series_id: NFTSeriesId,
		listed_for_sale: bool,
		is_in_transmission: bool,
		is_capsule: bool,
		is_secret: bool,
		is_delegated: bool,
		royaltie_fee: u8,
	) -> Self {
		Self {
			owner,
			creator,
			ipfs_reference,
			series_id,
			listed_for_sale,
			is_in_transmission,
			is_capsule,
			is_secret,
			is_delegated,
			royaltie_fee,
		}
	}

	pub fn new_default(
		owner: AccountId,
		ipfs_reference: IPFSReference<IPFSLengthLimit>,
		series_id: NFTSeriesId,
		royaltie_fee: u8,
	) -> Self {
		Self::new(
			owner.clone(),
			owner,
			ipfs_reference,
			series_id,
			false,
			false,
			false,
			false,
			false,
			royaltie_fee,
		)
	}

	pub fn to_raw(&self, nft_id: NFTId) -> NFTsGenesis<AccountId> {
		(
			nft_id,
			self.owner.clone(),
			self.creator.clone(),
			self.ipfs_reference.to_vec(),
			self.series_id.clone(),
			self.listed_for_sale,
			self.is_in_transmission,
			self.is_capsule,
			self.is_secret,
			self.is_delegated,
			self.royaltie_fee,
		)
	}

	pub fn from_raw(raw: NFTsGenesis<AccountId>) -> Self {
		let ipfs_reference = BoundedVec::try_from(raw.3).expect("It will never happen.");
		Self {
			owner: raw.1,
			creator: raw.2,
			ipfs_reference,
			series_id: raw.4,
			listed_for_sale: raw.5,
			is_in_transmission: raw.6,
			is_capsule: raw.7,
			is_secret: raw.8,
			is_delegated: raw.9,
			royaltie_fee: raw.10,
		}
	}
}

// nft_id, owner, creator, ipfs, series, for sale, in transmission, is capsule, viewer
pub type NFTsGenesis<AccountId> =
	(NFTId, AccountId, AccountId, Vec<u8>, Vec<u8>, bool, bool, bool, bool, bool, u8);

/// Data related to an NFT Series.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo)]
pub struct NFTSeriesDetails<AccountId>
where
	AccountId: Clone,
{
	pub owner: AccountId, // Series Owner
	pub draft: bool,      /* If Yes, the owner can add new nfts to that series but cannot list
	                       * that nft for sale */
}

impl<AccountId> NFTSeriesDetails<AccountId>
where
	AccountId: Clone,
{
	pub fn new(owner: AccountId, draft: bool) -> Self {
		Self { owner, draft }
	}

	pub fn to_raw(&self, series_id: NFTSeriesId) -> SeriesGenesis<AccountId> {
		(series_id, self.owner.clone(), self.draft)
	}

	pub fn from_raw(raw: SeriesGenesis<AccountId>) -> Self {
		Self { owner: raw.1, draft: raw.2 }
	}
}

/// Data related to an NFT Series.
// series id, owner, draft
pub type SeriesGenesis<AccountId> = (Vec<u8>, AccountId, bool);
