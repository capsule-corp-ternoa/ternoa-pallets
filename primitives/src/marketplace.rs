#![cfg_attr(not(feature = "std"), no_std)]

use crate::TextFormat;
use frame_support::{traits::Get, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// The type of marketplace Id
pub type MarketplaceId = u32;

/// Type of marketplace commission
pub type MarketplaceCommission = u8;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum MarketplaceType {
	Public = 0,
	Private = 1,
}

impl MarketplaceType {
	pub fn from_raw(num: u8) -> Result<MarketplaceType, ()> {
		match num {
			0 => Ok(MarketplaceType::Public),
			1 => Ok(MarketplaceType::Private),
			_ => Err(()),
		}
	}

	pub fn to_raw(&self) -> u8 {
		match self {
			MarketplaceType::Public => 0,
			MarketplaceType::Private => 1,
		}
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MarketplaceData<
	AccountId,
	AccountListLength,
	NameLengthLimit,
	URILengthLimit,
	DescriptionLengthLimit,
> where
	AccountId: Clone,
	AccountListLength: Get<u32>,
	NameLengthLimit: Get<u32>,
	URILengthLimit: Get<u32>,
	DescriptionLengthLimit: Get<u32>,
{
	pub kind: MarketplaceType,
	pub commission_fee: MarketplaceCommission,
	pub owner: AccountId,
	pub allow_list: BoundedVec<AccountId, AccountListLength>,
	pub disallow_list: BoundedVec<AccountId, AccountListLength>,
	pub name: BoundedVec<u8, NameLengthLimit>,
	pub uri: BoundedVec<u8, URILengthLimit>,
	pub logo_uri: BoundedVec<u8, URILengthLimit>,
	pub description: BoundedVec<u8, DescriptionLengthLimit>,
}

impl<AccountId, AccountListLength, NameLengthLimit, URILengthLimit, DescriptionLengthLimit>
	MarketplaceData<
		AccountId,
		AccountListLength,
		NameLengthLimit,
		URILengthLimit,
		DescriptionLengthLimit,
	> where
	AccountId: Clone,
	AccountListLength: Get<u32>,
	NameLengthLimit: Get<u32>,
	URILengthLimit: Get<u32>,
	DescriptionLengthLimit: Get<u32>,
{
	pub fn new(
		kind: MarketplaceType,
		commission_fee: MarketplaceCommission,
		owner: AccountId,
		allow_list: BoundedVec<AccountId, AccountListLength>,
		disallow_list: BoundedVec<AccountId, AccountListLength>,
		name: BoundedVec<u8, NameLengthLimit>,
		uri: BoundedVec<u8, URILengthLimit>,
		logo_uri: BoundedVec<u8, URILengthLimit>,
		description: BoundedVec<u8, DescriptionLengthLimit>,
	) -> MarketplaceData<
		AccountId,
		AccountListLength,
		NameLengthLimit,
		URILengthLimit,
		DescriptionLengthLimit,
	> {
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

	pub fn to_raw(&self, market_id: MarketplaceId) -> MarketplacesGenesis<AccountId> {
		(
			market_id,
			self.kind.to_raw(),
			self.commission_fee,
			self.owner.clone(),
			self.allow_list.to_vec(),
			self.disallow_list.to_vec(),
			self.name.to_vec(),
			self.uri.to_vec(),
			self.logo_uri.to_vec(),
			self.description.to_vec(),
		)
	}

	pub fn from_raw(raw: MarketplacesGenesis<AccountId>) -> Self {
		let kind = MarketplaceType::from_raw(raw.1).expect("Cannot fail.");
		let allow_list = BoundedVec::try_from(raw.4).expect("It will never happen.");
		let disallow_list = BoundedVec::try_from(raw.5).expect("It will never happen.");
		let name = BoundedVec::try_from(raw.6).expect("It will never happen.");
		let uri = BoundedVec::try_from(raw.7).expect("It will never happen.");
		let logo_uri = BoundedVec::try_from(raw.8).expect("It will never happen.");
		let description = BoundedVec::try_from(raw.9).expect("It will never happen.");
		Self {
			kind,
			commission_fee: raw.2,
			owner: raw.3,
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
	TextFormat,
	TextFormat,
	TextFormat,
);
