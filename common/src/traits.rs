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

use frame_support::{dispatch::DispatchResult, traits::Get, BoundedVec};
use primitives::{
	marketplace::{MarketplaceData, MarketplaceId},
	nfts::{CollectionId, NFTData, NFTId, NFTState},
};
use sp_runtime::Permill;
use sp_std::fmt::Debug;

pub trait NFTExt {
	type AccountId: Clone + PartialEq + Debug;
	type NFTOffchainDataLimit: Get<u32>;
	type CollectionSizeLimit: Get<u32>;
	type CollectionOffchainDataLimit: Get<u32>;
	type ShardsNumber: Get<u32>;

	/// Change the state data of an NFT.
	fn set_nft_state(id: NFTId, nft_state: NFTState) -> DispatchResult;

	/// Create a collection filled with amount_in_collection NFTs.
	fn create_filled_collection(
		owner: Self::AccountId,
		collection_id: CollectionId,
		start_nft_id: NFTId,
		amount_in_collection: u32,
	) -> DispatchResult;

	/// Returns an NFT corresponding to its id.
	fn get_nft(id: NFTId) -> Option<NFTData<Self::AccountId, Self::NFTOffchainDataLimit>>;

	/// Set the NFT data
	fn set_nft(
		id: NFTId,
		nft_data: NFTData<Self::AccountId, Self::NFTOffchainDataLimit>,
	) -> DispatchResult;

	/// Create an NFT
	fn create_nft(
		owner: Self::AccountId,
		offchain_data: BoundedVec<u8, Self::NFTOffchainDataLimit>,
		royalty: Permill,
		collection_id: Option<CollectionId>,
		is_soulbound: bool,
	) -> Result<NFTId, DispatchResult>;

	fn mutate_nft<
		R,
		E,
		F: FnOnce(&mut Option<NFTData<Self::AccountId, Self::NFTOffchainDataLimit>>) -> Result<R, E>,
	>(
		id: NFTId,
		f: F,
	) -> Result<R, E>;

	fn exists(id: NFTId) -> bool;
}

pub trait MarketplaceExt {
	type AccountId: Clone + PartialEq + Debug;
	type Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd;
	type OffchainDataLimit: Get<u32>;
	type AccountSizeLimit: Get<u32>;
	type CollectionSizeLimit: Get<u32>;

	/// Returns a marketplace corresponding to its id.
	fn get_marketplace(
		id: MarketplaceId,
	) -> Option<
		MarketplaceData<
			Self::AccountId,
			Self::Balance,
			Self::AccountSizeLimit,
			Self::OffchainDataLimit,
			Self::CollectionSizeLimit,
		>,
	>;

	/// Set marketplace data for specified id.
	fn set_marketplace(
		id: MarketplaceId,
		value: MarketplaceData<
			Self::AccountId,
			Self::Balance,
			Self::AccountSizeLimit,
			Self::OffchainDataLimit,
			Self::CollectionSizeLimit,
		>,
	) -> DispatchResult;
}
