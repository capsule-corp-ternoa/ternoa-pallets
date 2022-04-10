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

use crate::{TextFormat, U8BoundedVec};
use frame_support::{traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{fmt::Debug, vec::Vec};

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

#[derive(
	Decode, CloneNoBound, Eq, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen, Encode,
)]
#[scale_info(skip_type_params(
	AccountCountLimit,
	NameLengthLimit,
	URILengthLimit,
	DescriptionLengthLimit
))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct MarketplaceData<
	AccountId,
	AccountCountLimit,
	NameLengthLimit,
	URILengthLimit,
	DescriptionLengthLimit,
> where
	AccountId: Clone + PartialEq + Debug,
	AccountCountLimit: Get<u32>,
	NameLengthLimit: Get<u32>,
	URILengthLimit: Get<u32>,
	DescriptionLengthLimit: Get<u32>,
{
	pub kind: MarketplaceType,
	pub commission_fee: MarketplaceCommission,
	pub owner: AccountId,
	pub allow_list: BoundedVec<AccountId, AccountCountLimit>,
	pub disallow_list: BoundedVec<AccountId, AccountCountLimit>,
	pub name: U8BoundedVec<NameLengthLimit>,
	pub uri: U8BoundedVec<URILengthLimit>,
	pub logo_uri: U8BoundedVec<URILengthLimit>,
	pub description: U8BoundedVec<DescriptionLengthLimit>,
}

impl<AccountId, AccountCountLimit, NameLengthLimit, URILengthLimit, DescriptionLengthLimit>
	MarketplaceData<
		AccountId,
		AccountCountLimit,
		NameLengthLimit,
		URILengthLimit,
		DescriptionLengthLimit,
	> where
	AccountId: Clone + PartialEq + Debug,
	AccountCountLimit: Get<u32>,
	NameLengthLimit: Get<u32>,
	URILengthLimit: Get<u32>,
	DescriptionLengthLimit: Get<u32>,
{
	pub fn new(
		kind: MarketplaceType,
		commission_fee: MarketplaceCommission,
		owner: AccountId,
		allow_list: BoundedVec<AccountId, AccountCountLimit>,
		disallow_list: BoundedVec<AccountId, AccountCountLimit>,
		name: U8BoundedVec<NameLengthLimit>,
		uri: U8BoundedVec<URILengthLimit>,
		logo_uri: U8BoundedVec<URILengthLimit>,
		description: U8BoundedVec<DescriptionLengthLimit>,
	) -> MarketplaceData<
		AccountId,
		AccountCountLimit,
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
