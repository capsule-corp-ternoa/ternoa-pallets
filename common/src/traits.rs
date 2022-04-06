#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, DispatchResult},
	traits::Get,
	BoundedVec,
};
use primitives::{
	marketplace::{MarketplaceData, MarketplaceId, MarketplaceType},
	nfts::{NFTData, NFTId, NFTSeriesId},
};
use sp_std::fmt::Debug;

pub trait NFTTrait {
	type AccountId: Clone + PartialEq + Debug;
	type IPFSLengthLimit: Get<u32>;

	/// Change the owner of an NFT.
	fn set_owner(id: NFTId, owner: &Self::AccountId) -> DispatchResult;

	/// Return the owner of an NFT.
	fn owner(id: NFTId) -> Option<Self::AccountId>;

	/// Is series completed(locked)
	fn is_nft_in_completed_series(id: NFTId) -> Option<bool>;

	/// Create NFT and return its NFTId
	fn create_nft(
		owner: Self::AccountId,
		ipfs_reference: BoundedVec<u8, Self::IPFSLengthLimit>,
		series_id: Option<NFTSeriesId>,
	) -> Result<NFTId, DispatchErrorWithPostInfo>;

	/// Get NFT data
	fn get_nft(id: NFTId) -> Option<NFTData<Self::AccountId, Self::IPFSLengthLimit>>;

	/// Lock series WARNING: Only for benchmark purposes!
	fn benchmark_lock_series(series_id: NFTSeriesId);

	/// TODO!
	fn set_listed_for_sale(id: NFTId, value: bool) -> DispatchResult;

	/// TODO!
	fn is_listed_for_sale(id: NFTId) -> Option<bool>;

	/// TODO!
	fn set_in_transmission(id: NFTId, value: bool) -> DispatchResult;

	/// TODO!
	fn is_in_transmission(id: NFTId) -> Option<bool>;

	/// TODO!
	fn set_converted_to_capsule(id: NFTId, value: bool) -> DispatchResult;

	/// TODO!
	fn is_converted_to_capsule(id: NFTId) -> Option<bool>;

	/// Set a series to be either completed or not-completed.
	fn set_series_completion(series_id: &NFTSeriesId, value: bool) -> DispatchResult;

	/// Set the NFT viewer to a value.
	fn set_viewer(id: NFTId, value: Option<Self::AccountId>) -> DispatchResult;
}

/// Trait that implements basic functionalities related to Ternoa Marketplace
/// TODO: Expand trait with more useful functions
pub trait MarketplaceTrait {
	type AccountId: Clone;
	type AccountListLength: Get<u32>;
	type NameLengthLimit: Get<u32>;
	type URILengthLimit: Get<u32>;
	type DescriptionLengthLimit: Get<u32>;

	/// Return if an account is permitted to list on given marketplace
	fn is_allowed_to_list(
		marketplace_id: MarketplaceId,
		account_id: Self::AccountId,
	) -> DispatchResult;

	/// Return marketplace
	fn get_marketplace(
		marketplace_id: MarketplaceId,
	) -> Option<
		MarketplaceData<
			Self::AccountId,
			Self::AccountListLength,
			Self::NameLengthLimit,
			Self::URILengthLimit,
			Self::DescriptionLengthLimit,
		>,
	>;

	/// create a new marketplace
	fn create(
		origin: Self::AccountId,
		kind: MarketplaceType,
		commission_fee: u8,
		name: BoundedVec<u8, Self::NameLengthLimit>,
		uri: BoundedVec<u8, Self::URILengthLimit>,
		logo_uri: BoundedVec<u8, Self::URILengthLimit>,
		description: BoundedVec<u8, Self::DescriptionLengthLimit>,
	) -> Result<MarketplaceId, DispatchErrorWithPostInfo>;
}
