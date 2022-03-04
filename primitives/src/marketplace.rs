#![cfg_attr(not(feature = "std"), no_std)]

use crate::TextFormat;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// The type of marketplace Id
pub type MarketplaceId = u32;

/// Type of marketplace commission
pub type MarketplaceCommission = u8;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[repr(u8)]
pub enum MarketplaceType {
	Public = 0,
	Private = 1,
}

impl MarketplaceType {
	pub fn from(num: u8) -> Result<MarketplaceType, ()> {
		match num {
			0 => Ok(MarketplaceType::Public),
			1 => Ok(MarketplaceType::Private),
			_ => Err(()),
		}
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct MarketplaceData<AccountId> {
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

impl<AccountId> MarketplaceData<AccountId> {
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
	) -> MarketplaceData<AccountId> {
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

// marketplace id, marketplace type, commission fee, owner, allow list, disallow list, name, uri,
// logo_uri, description
pub type MarketplacesGenesis<AccountId> = (
	MarketplaceId,
	u8,
	u8,
	AccountId,
	Vec<AccountId>,
	Vec<AccountId>,
	TextFormat,
	Option<TextFormat>,
	Option<TextFormat>,
	Option<TextFormat>,
);
