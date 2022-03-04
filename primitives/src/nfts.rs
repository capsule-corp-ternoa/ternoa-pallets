#![cfg_attr(not(feature = "std"), no_std)]

use crate::TextFormat;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// How NFT IDs are encoded.
pub type NFTId = u32;

/// How NFT IDs are encoded. In the JSON Types this should be "Text" and not "Vec<8>".
pub type NFTSeriesId = Vec<u8>;

/// Data related to an NFT, such as who is its owner.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo)]
pub struct NFTData<AccountId>
where
	AccountId: Clone,
{
	// NFT owner
	pub owner: AccountId,
	// NFT creator
	pub creator: AccountId,
	// IPFS reference
	pub ipfs_reference: TextFormat,
	// Series ID
	pub series_id: NFTSeriesId,
	// Is listed for sale
	pub listed_for_sale: bool,
	// Is being transmitted
	pub in_transmission: bool,
	// Is NFT converted to capsule
	pub converted_to_capsule: bool,
	// NFT Viewer
	pub viewer: Option<AccountId>,
}

impl<AccountId> NFTData<AccountId>
where
	AccountId: Clone,
{
	pub fn new(
		owner: AccountId,
		creator: AccountId,
		ipfs_reference: TextFormat,
		series_id: NFTSeriesId,
		listed_for_sale: bool,
		in_transmission: bool,
		converted_to_capsule: bool,
		viewer: Option<AccountId>,
	) -> Self {
		Self {
			owner,
			creator,
			ipfs_reference,
			series_id,
			listed_for_sale,
			in_transmission,
			converted_to_capsule,
			viewer,
		}
	}

	pub fn new_default(
		owner: AccountId,
		ipfs_reference: TextFormat,
		series_id: NFTSeriesId,
	) -> Self {
		Self::new(owner.clone(), owner, ipfs_reference, series_id, false, false, false, None)
	}

	pub fn to_raw(&self, nft_id: NFTId) -> NFTsGenesis<AccountId> {
		(
			nft_id,
			self.owner.clone(),
			self.creator.clone(),
			self.ipfs_reference.clone(),
			self.series_id.clone(),
			self.listed_for_sale,
			self.in_transmission,
			self.converted_to_capsule,
			self.viewer.clone(),
		)
	}

	pub fn from_raw(raw: NFTsGenesis<AccountId>) -> Self {
		Self {
			owner: raw.1,
			creator: raw.2,
			ipfs_reference: raw.3,
			series_id: raw.4,
			listed_for_sale: raw.5,
			in_transmission: raw.6,
			converted_to_capsule: raw.7,
			viewer: raw.8,
		}
	}
}

// nft_id, owner, creator, ipfs, series, for sale, in transmission, is capsule, viewer
pub type NFTsGenesis<AccountId> =
	(NFTId, AccountId, AccountId, Vec<u8>, Vec<u8>, bool, bool, bool, Option<AccountId>);

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
