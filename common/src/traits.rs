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
	dispatch::{DispatchErrorWithPostInfo, DispatchResult},
	traits::Get,
};
use primitives::{
	nfts::{NFTData, NFTId, NFTState},
	U8BoundedVec,
};
use sp_arithmetic::per_things::Permill;
use sp_std::fmt::Debug;

pub trait NFTExt {
	type AccountId: Clone + PartialEq + Debug;
	type NFTOffchainDataLimit: Get<u32>;
	type CollectionSizeLimit: Get<u32>;
	type CollectionOffchainDataLimit: Get<u32>;

	/*
		create nft
		get nft
		set nft
		burn nft
		get state
		set state
		get delegated nft
		set delegated nft (since it needs updating an other storage)
		create collection
		get collection
		set collection
		burn collection
		add nft to collection (since it needs updating an other storage)

		benchmark_close_collection ?
		benchmark_limit_collection ?
	*/

	fn get_nft_state(id: NFTId) -> NFTState;

	fn set_nft_state(
		id: NFTId,
		is_capsule: bool,
		listed_for_sale: bool,
		is_secret: bool,
		is_delegated: bool,
		is_soulbound: bool,
	) -> DispatchResult;

	fn get_nft(id: NFTId) -> NFTData<Self::AccountId, Self::NFTOffchainDataLimit>;

	fn create_nft(
		owner: Self::AccountId,
		offchain_data: U8BoundedVec<Self::NFTOffchainDataLimit>,
		royalty: Permill,
		collection_id: Option<u32>,
		is_soulbound: bool,
	) -> Result<NFTId, DispatchErrorWithPostInfo>;

	fn set_nft(
		id: NFTId,
		owner: Self::AccountId,
		offchain_data: U8BoundedVec<Self::NFTOffchainDataLimit>,
		royalty: Permill,
		collection_id: Option<u32>,
	) -> DispatchResult;

	// /// Change the owner of an NFT.
	// fn set_owner(id: NFTId, owner: &Self::AccountId) -> DispatchResult;

	// /// Return the owner of an NFT.
	// fn owner(id: NFTId) -> Option<Self::AccountId>;

	// /// Is series completed(locked)
	// fn is_nft_in_completed_series(id: NFTId) -> Option<bool>;

	// /// Create NFT and return its NFTId
	// fn create_nft(
	// 	owner: Self::AccountId,
	// 	offchain_data: BoundedVec<u8, Self::OffchainDataLimit>,
	// 	collection_id: Option<CollectionId>,
	// ) -> Result<NFTId, DispatchErrorWithPostInfo>;

	// /// Get NFT data
	// fn get_nft(id: NFTId) -> Option<NFT<Self::AccountId, Self::OffchainDataLimit>>;

	// // /// Lock series WARNING: Only for benchmark purposes!
	// // fn benchmark_lock_series(series_id: NFTSeriesId);

	// /// TODO!
	// fn set_listed_for_sale(id: NFTId, value: bool) -> DispatchResult;

	// /// TODO!
	// fn is_listed_for_sale(id: NFTId) -> Option<bool>;

	// /// Set a collection to be either close or not-closed.
	// fn set_collection_completion(collection_id: &CollectionId, value: bool) -> DispatchResult;

	// /// Set the NFT viewer to a value.
	// fn set_viewer(id: NFTId, value: Option<Self::AccountId>) -> DispatchResult;
}

// /// Trait that implements basic functionalities related to Ternoa Marketplace
// /// TODO: Expand trait with more useful functions
// pub trait MarketplaceExt {
// 	type AccountId: Clone + PartialEq + Debug;
// 	type AccountCountLimit: Get<u32>;
// 	type NameLengthLimit: Get<u32>;
// 	type URILengthLimit: Get<u32>;
// 	type DescriptionLengthLimit: Get<u32>;

// 	/// Return if an account is permitted to list on given marketplace
// 	fn is_allowed_to_list(
// 		marketplace_id: MarketplaceId,
// 		account_id: Self::AccountId,
// 	) -> DispatchResult;

// 	/// Return marketplace
// 	fn get_marketplace(
// 		marketplace_id: MarketplaceId,
// 	) -> Option<
// 		MarketplaceData<
// 			Self::AccountId,
// 			Self::AccountCountLimit,
// 			Self::NameLengthLimit,
// 			Self::URILengthLimit,
// 			Self::DescriptionLengthLimit,
// 		>,
// 	>;

// 	/// create a new marketplace
// 	fn create(
// 		origin: Self::AccountId,
// 		kind: MarketplaceType,
// 		commission_fee: u8,
// 		name: BoundedVec<u8, Self::NameLengthLimit>,
// 		uri: BoundedVec<u8, Self::URILengthLimit>,
// 		logo_uri: BoundedVec<u8, Self::URILengthLimit>,
// 		description: BoundedVec<u8, Self::DescriptionLengthLimit>,
// 	) -> Result<MarketplaceId, DispatchErrorWithPostInfo>;
// }
