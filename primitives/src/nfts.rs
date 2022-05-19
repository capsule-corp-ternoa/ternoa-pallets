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

use frame_support::{traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, RuntimeDebug};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::fmt::Debug;
use sp_arithmetic::per_things::Permill;

use crate::U8BoundedVec;

/// How NFT IDs are encoded.
pub type NFTId = u32;

/// How collection IDs are encoded.
pub type CollectionId = u32;

/// Data related to an NFT state, such as if it is listed for sale.
#[derive(
	Encode, Decode, Eq, Default, TypeInfo, Clone, PartialEq, RuntimeDebug,
)]
pub struct NFTState
{
	/// Is NFT converted to capsule
	pub is_capsule: bool,
	/// Is NFT listed for sale
	pub listed_for_sale: bool,
	/// Is NFT contains secret
	pub is_secret: bool,
	/// Is NFT delegated
	pub is_delegated: bool,
}

impl NFTState
{
	pub fn new(
		is_capsule: bool,
		listed_for_sale: bool,
		is_secret: bool,
		is_delegated: bool,
	) -> Self {
		Self {
			is_capsule,
			listed_for_sale,
			is_secret,
			is_delegated,
		}
	}

	pub fn new_default() -> Self {
		Self::new(
			false,
			false,
			false,
			false,
		)
	}
}

/// Data related to an NFT, such as who is its owner.
#[derive(
	Encode, Decode, Eq, Default, TypeInfo, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
)]
#[scale_info(skip_type_params(OffchainDataLimit))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct NFT<
	AccountId, 
	OffchainDataLimit,
> where
	AccountId: Clone + PartialEq + Debug,
	OffchainDataLimit: Get<u32>,
{
	/// NFT owner
	pub owner: AccountId,
	/// NFT creator
	pub creator: AccountId,
	/// NFT offchain_data
	pub offchain_data: U8BoundedVec<OffchainDataLimit>,
	/// Collection ID
	pub collection_id: Option<CollectionId>,
	/// Royalty
	pub royalty: Permill,
	/// NFT state
	pub state: NFTState,
}

impl<
	AccountId, 
	OffchainDataLimit,
> NFT<
	AccountId, 
	OffchainDataLimit,
> where
	AccountId: Clone + PartialEq + Debug,
	OffchainDataLimit: Get<u32>,
{
	pub fn new(
		owner: AccountId,
		creator: AccountId,
		offchain_data: U8BoundedVec<OffchainDataLimit>,
		royalty: Permill,
		state: NFTState,
		collection_id: Option<CollectionId>,
	) -> Self {
		Self {
			owner,
			creator,
			offchain_data,
			royalty,
			state,
			collection_id,
		}
	}

	pub fn new_default(
		owner: AccountId,
		offchain_data: U8BoundedVec<OffchainDataLimit>,
		royalty: Permill,
		collection_id: Option<CollectionId>,
	) -> Self {
		Self::new(
			owner.clone(),
			owner,
			offchain_data,
			royalty,
			NFTState::new_default(),
			collection_id,
		)
	}
}

/// Data related to collections
#[derive(
	Encode, Decode, Eq, Default, TypeInfo, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
)]
#[scale_info(skip_type_params(
	CollectionNameLimit,
	CollectionDescriptionLimit,
	CollectionSizeLimit,
))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct Collection<
	AccountId,
	CollectionNameLimit,
	CollectionDescriptionLimit,
	CollectionSizeLimit,
> where
	AccountId: Clone + PartialEq + Debug,
	CollectionNameLimit: Get<u32>,
	CollectionDescriptionLimit: Get<u32>,
	CollectionSizeLimit: Get<u32>,
{
	/// Collection owner
	pub owner: AccountId,
	/// Collection name
	pub name: U8BoundedVec<CollectionNameLimit>,
	/// Collection description
	pub description: U8BoundedVec<CollectionDescriptionLimit>,
	/// NFTs in that collection
	pub nfts: BoundedVec<NFTId, CollectionSizeLimit>,
	/// Maximum length of the collection
	pub limit: Option<u32>,
	/// Is collection closed for adding new NFTs
	pub is_closed: bool,
  }


impl<
	AccountId, 
	CollectionNameLimit, 
	CollectionDescriptionLimit, 
	CollectionSizeLimit,
> Collection<
	AccountId, 
	CollectionNameLimit,
	CollectionDescriptionLimit,
	CollectionSizeLimit,
> where
	AccountId: Clone + PartialEq + Debug,
	CollectionNameLimit: Get<u32>,
	CollectionDescriptionLimit: Get<u32>,
	CollectionSizeLimit: Get<u32>,
{
	pub fn new(
		owner: AccountId,
		name: U8BoundedVec<CollectionNameLimit>, 
		description: U8BoundedVec<CollectionDescriptionLimit>, 
		limit: Option<u32>,
	) -> Self {
		Self { 
			owner,
			name, 
			description,
			nfts: BoundedVec::default(),
			limit,
			is_closed: false
		}
	}
}