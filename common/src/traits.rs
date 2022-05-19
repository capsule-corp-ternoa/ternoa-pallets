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

// use frame_support::{
// 	dispatch::{DispatchErrorWithPostInfo, DispatchResult},
// 	traits::Get,
// 	BoundedVec,
// };
// use primitives::{
// 	marketplace::{MarketplaceData, MarketplaceId, MarketplaceType},
// 	nfts::{NFT, NFTId, CollectionId},
// };
use sp_std::fmt::Debug;

pub trait NFTExt {
	type AccountId: Clone + PartialEq + Debug;
	// type OffchainDataLimit: Get<u32>;
	// type CollectionSizeLimit: Get<u32>;
	// type CollectionNameLimit: Get<u32>;
	// type CollectionDescriptionLimit: Get<u32>;
	// type InitialMintFee: Get<u128>;


	/*
		create nft
		get nft
		delegate nft (since it needs updating an other storage)
		get state
		set state
		create collection
		get collection
		add nft to collection
		limit collection
		close collection
		set collection limit

		benchmark_close_collection ?
		benchmark_limit_collection ?
	*/

//OR ?

	/*
		create nft
		get nft
		set nft
		get state
		set state
		get delegated nft
		set delegated nft
		create collection
		get collection
		set collection

		benchmark_close_collection ?
		benchmark_limit_collection ?
	*/


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
