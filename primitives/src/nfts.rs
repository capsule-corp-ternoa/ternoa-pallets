#![cfg_attr(not(feature = "std"), no_std)]

use crate::TextFormat;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiSignature, OpaqueExtrinsic, RuntimeDebug,
};
use sp_std::vec::Vec;

/// How NFT IDs are encoded.
pub type NFTId = u32;

/// How NFT IDs are encoded. In the JSON Types this should be "Text" and not "Vec<8>".
pub type NFTSeriesId = Vec<u8>;

/// Data related to an NFT, such as who is its owner.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
}

/// Data related to an NFT Series.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NFTSeriesDetails<AccountId> {
	pub owner: AccountId, // Series Owner
	pub draft: bool,      /* If Yes, the owner can add new nfts to that series but cannot list
	                       * that nft for sale */
}

impl<AccountId> NFTSeriesDetails<AccountId> {
	pub fn new(owner: AccountId, draft: bool) -> Self {
		Self { owner, draft }
	}
}
