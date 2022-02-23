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

/// The type of marketplace Id
pub type MarketplaceId = u32;

/// Type of marketplace commission
pub type MarketplaceCommission = u8;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum MarketplaceType {
	Public,
	Private,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MarketplaceInformation<AccountId> {
	pub kind: MarketplaceType,
	pub commission_fee: MarketplaceCommission,
	pub owner: AccountId,
	pub allow_list: Vec<AccountId>,
	pub disallow_list: Vec<AccountId>,
	pub name: TextFormat,
	pub uri: Option<TextFormat>,
	pub logo_uri: Option<TextFormat>,
	pub description: Option<TextFormat>,
}

impl<AccountId> MarketplaceInformation<AccountId> {
	pub fn new(
		kind: MarketplaceType,
		commission_fee: MarketplaceCommission,
		owner: AccountId,
		allow_list: Vec<AccountId>,
		disallow_list: Vec<AccountId>,
		name: TextFormat,
		uri: Option<TextFormat>,
		logo_uri: Option<TextFormat>,
		description: Option<TextFormat>,
	) -> MarketplaceInformation<AccountId> {
		Self {
			kind,
			commission_fee,
			owner,
			allow_list,
			disallow_list,
			name,
			uri,
			logo_uri,
			description,
		}
	}
}
