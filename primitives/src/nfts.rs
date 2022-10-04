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

use frame_support::{
	traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebug, RuntimeDebugNoBound,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::per_things::Permill;
use sp_std::fmt::Debug;

use crate::U8BoundedVec;

/// How NFT IDs are encoded.
pub type NFTId = u32;

/// How collection IDs are encoded.
pub type CollectionId = u32;

#[derive(Encode, Decode, Eq, TypeInfo, Clone, Copy, PartialEq, RuntimeDebug, MaxEncodedLen)]
pub enum NFTStateModifiers {
	Capsule = 0x01,
	IsListed = 0x02,
	Secret = 0x03,
	Delegated = 0x04,
	Soulbound = 0x05,
	//SecretSyncing = 0x06,
	Rented = 0x07,
}

/// Data related to an NFT state, such as if it is listed for sale.
#[derive(Encode, Decode, Eq, Default, TypeInfo, Clone, PartialEq, RuntimeDebug, MaxEncodedLen)]
pub struct NFTState {
	/// Is NFT converted to capsule.
	pub is_capsule: bool,
	/// Is NFT listed for sale.
	pub is_listed: bool,
	/// Is NFT contains secret.
	pub is_secret: bool,
	/// Is NFT delegated.
	pub is_delegated: bool,
	/// Is NFT soulbound.
	pub is_soulbound: bool,
	/// Is NFT Secret synced
	pub is_secret_synced: bool,
	/// Is NFT Rented or available for rent.
	pub is_rented: bool,
}

impl NFTState {
	pub fn new(
		is_capsule: bool,
		is_listed: bool,
		is_secret: bool,
		is_delegated: bool,
		is_soulbound: bool,
		is_secret_synced: bool,
		is_rented: bool,
	) -> Self {
		Self {
			is_capsule,
			is_listed,
			is_secret,
			is_delegated,
			is_soulbound,
			is_secret_synced,
			is_rented,
		}
	}

	pub fn new_default(is_soulbound: bool) -> Self {
		Self::new(false, false, false, false, is_soulbound, false, false)
	}
}

/// Data related to an NFT, such as who is its owner.
#[derive(
	Encode,
	Decode,
	Eq,
	Default,
	TypeInfo,
	CloneNoBound,
	PartialEqNoBound,
	RuntimeDebugNoBound,
	MaxEncodedLen,
)]
#[scale_info(skip_type_params(NFTOffchainDataLimit))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct NFTData<AccountId, NFTOffchainDataLimit>
where
	AccountId: Clone + PartialEq + Debug,
	NFTOffchainDataLimit: Get<u32>,
{
	/// NFT owner
	pub owner: AccountId,
	/// NFT creator
	pub creator: AccountId,
	/// NFT offchain_data
	pub offchain_data: U8BoundedVec<NFTOffchainDataLimit>,
	/// Collection ID
	pub collection_id: Option<CollectionId>,
	/// Royalty
	pub royalty: Permill,
	/// NFT state
	pub state: NFTState,
}

impl<AccountId, NFTOffchainDataLimit> NFTData<AccountId, NFTOffchainDataLimit>
where
	AccountId: Clone + PartialEq + Debug,
	NFTOffchainDataLimit: Get<u32>,
{
	pub fn new(
		owner: AccountId,
		creator: AccountId,
		offchain_data: U8BoundedVec<NFTOffchainDataLimit>,
		royalty: Permill,
		state: NFTState,
		collection_id: Option<CollectionId>,
	) -> Self {
		Self { owner, creator, offchain_data, royalty, state, collection_id }
	}

	pub fn new_default(
		owner: AccountId,
		offchain_data: U8BoundedVec<NFTOffchainDataLimit>,
		royalty: Permill,
		collection_id: Option<CollectionId>,
		is_soulbound: bool,
	) -> Self {
		Self::new(
			owner.clone(),
			owner,
			offchain_data,
			royalty,
			NFTState::new_default(is_soulbound),
			collection_id,
		)
	}

	pub fn not_in_state(&self, list: &Vec<NFTStateModifiers>) -> Result<(), NFTStateModifiers> {
		for modifier in list {
			let in_state = match modifier {
				NFTStateModifiers::Capsule => self.state.is_capsule == true,
				NFTStateModifiers::IsListed => self.state.is_listed == true,
				NFTStateModifiers::Secret => self.state.is_secret == true,
				NFTStateModifiers::Delegated => self.state.is_delegated == true,
				NFTStateModifiers::Soulbound => self.state.is_soulbound == true,
				NFTStateModifiers::Rented => self.state.is_rented == true,
			};
			if in_state {
				return Err(*modifier)
			}
		}

		Ok(())
	}

	pub fn set_state(&mut self, state: NFTStateModifiers, active: bool) -> Option<()> {
		let is_already_in_state = match state {
			NFTStateModifiers::Capsule => self.state.is_capsule == active,
			NFTStateModifiers::IsListed => self.state.is_listed == active,
			NFTStateModifiers::Secret => self.state.is_secret == active,
			NFTStateModifiers::Delegated => self.state.is_delegated == active,
			NFTStateModifiers::Soulbound => self.state.is_soulbound == active,
			NFTStateModifiers::Rented => self.state.is_rented == active,
		};
		if is_already_in_state {
			return None
		}

		match state {
			NFTStateModifiers::Capsule => self.state.is_capsule = active,
			NFTStateModifiers::IsListed => self.state.is_listed = active,
			NFTStateModifiers::Secret => self.state.is_secret = active,
			NFTStateModifiers::Delegated => self.state.is_delegated = active,
			NFTStateModifiers::Soulbound => self.state.is_soulbound = active,
			NFTStateModifiers::Rented => self.state.is_rented = active,
		};

		Some(())
	}

	pub fn is_owner(&self, who: &AccountId) -> Option<()> {
		(self.owner == *who).then(|| {})
	}
}

/// Data related to collections
#[derive(
	Encode,
	Decode,
	Eq,
	Default,
	TypeInfo,
	CloneNoBound,
	PartialEqNoBound,
	RuntimeDebugNoBound,
	MaxEncodedLen,
)]
#[scale_info(skip_type_params(CollectionOffChainDataLimit, CollectionSizeLimit,))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct Collection<AccountId, CollectionOffChainDataLimit, CollectionSizeLimit>
where
	AccountId: Clone + PartialEq + Debug,
	CollectionOffChainDataLimit: Get<u32>,
	CollectionSizeLimit: Get<u32>,
{
	/// Collection owner
	pub owner: AccountId,
	/// Collection offchain_data
	pub offchain_data: U8BoundedVec<CollectionOffChainDataLimit>,
	/// NFTs in that collection
	pub nfts: BoundedVec<NFTId, CollectionSizeLimit>,
	/// Maximum length of the collection
	pub limit: Option<u32>,
	/// Is collection closed for adding new NFTs
	pub is_closed: bool,
}

impl<AccountId, CollectionOffChainDataLimit, CollectionSizeLimit>
	Collection<AccountId, CollectionOffChainDataLimit, CollectionSizeLimit>
where
	AccountId: Clone + PartialEq + Debug,
	CollectionOffChainDataLimit: Get<u32>,
	CollectionSizeLimit: Get<u32>,
{
	pub fn new(
		owner: AccountId,
		offchain_data: U8BoundedVec<CollectionOffChainDataLimit>,
		limit: Option<u32>,
	) -> Self {
		Self { owner, offchain_data, nfts: BoundedVec::default(), limit, is_closed: false }
	}
}
