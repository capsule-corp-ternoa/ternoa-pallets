#![cfg_attr(not(feature = "std"), no_std)]

use crate::StringData;
use codec::{Decode, Encode};
use frame_support::traits::Get;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// How NFT IDs are encoded.
pub type NFTId = u32;

/// Data related to an NFT, such as who is its owner.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo)]
pub struct NFTData<AccountId, IPFSReference, SeriesId>
where
	AccountId: Clone,
	IPFSReference: Clone,
	SeriesId: Clone,
{
	// NFT owner
	pub owner: AccountId,
	// NFT creator
	pub creator: AccountId,
	// IPFS reference
	pub ipfs_reference: IPFSReference,
	// Series ID
	pub series_id: SeriesId,
	// Is listed for sale
	pub listed_for_sale: bool,
	// Is being transmitted
	pub in_transmission: bool,
	// Is NFT converted to capsule
	pub converted_to_capsule: bool,
	// NFT Viewer
	pub viewer: Option<AccountId>,
}

impl<AccountId, IPFSReference, SeriesId> NFTData<AccountId, IPFSReference, SeriesId>
where
	AccountId: Clone,
	IPFSReference: Clone,
	SeriesId: Clone,
{
	pub fn new(
		owner: AccountId,
		creator: AccountId,
		ipfs_reference: IPFSReference,
		series_id: SeriesId,
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
		ipfs_reference: IPFSReference,
		series_id: SeriesId,
	) -> Self {
		Self::new(owner.clone(), owner, ipfs_reference, series_id, false, false, false, None)
	}
}

// nft_id, owner, creator, ipfs, series, for sale, in transmission, is capsule, viewer
pub type NFTsGenesis<AccountId> =
	(NFTId, AccountId, AccountId, Vec<u8>, Vec<u8>, bool, bool, bool, Option<AccountId>);

/// Data related to an NFT Series.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo)]
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

// series id, owner, draft
pub type SeriesGenesis<AccountId> = (Vec<u8>, AccountId, bool);
