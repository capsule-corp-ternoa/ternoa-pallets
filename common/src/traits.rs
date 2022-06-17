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
use primitives::nfts::{CollectionId, NFTData, NFTId};
use sp_runtime::Permill;
use sp_std::fmt::Debug;

pub trait NFTExt {
	type AccountId: Clone + PartialEq + Debug;
	type NFTOffchainDataLimit: Get<u32>;
	type CollectionSizeLimit: Get<u32>;
	type CollectionOffchainDataLimit: Get<u32>;

	fn set_nft_state(
		id: NFTId,
		is_capsule: bool,
		listed_for_sale: bool,
		is_secret: bool,
		is_delegated: bool,
		is_soulbound: bool,
	) -> DispatchResult;

	fn create_filled_collection(
		owner: Self::AccountId,
		collection_id: CollectionId,
		start_nft_id: NFTId,
		amount_in_collection: u32,
	) -> DispatchResult;

	fn get_nft(id: NFTId) -> Option<NFTData<Self::AccountId, Self::NFTOffchainDataLimit>>;

	fn set_owner(id: NFTId, owner: &Self::AccountId) -> DispatchResult;

	fn create_nft(
		owner: Self::AccountId,
		offchain_data: BoundedVec<u8, Self::NFTOffchainDataLimit>,
		royalty: Permill,
		collection_id: Option<CollectionId>,
		is_soulbound: bool,
	) -> Result<NFTId, DispatchResult>;
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
