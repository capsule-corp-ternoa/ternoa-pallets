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

use frame_support::{traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::fmt::Debug;

use crate::{nfts::CollectionId, CompoundFee, U8BoundedVec};

pub type MarketplaceId = u32;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum MarketplaceType {
	Public = 0,
	Private = 1,
}

#[derive(
	Encode, Decode, CloneNoBound, Eq, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(AccountSizeLimit, OffchainDataLimit, CollectionSizeLimit))]
#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen))]
pub struct MarketplaceData<
	AccountId,
	Balance,
	AccountSizeLimit,
	OffchainDataLimit,
	CollectionSizeLimit,
> where
	AccountId: Clone + PartialEq + Debug,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	AccountSizeLimit: Get<u32>,
	OffchainDataLimit: Get<u32>,
	CollectionSizeLimit: Get<u32>,
{
	pub owner: AccountId,
	pub kind: MarketplaceType,
	pub commission_fee: Option<CompoundFee<Balance>>,
	pub listing_fee: Option<CompoundFee<Balance>>,
	pub account_list: Option<BoundedVec<AccountId, AccountSizeLimit>>,
	pub offchain_data: Option<U8BoundedVec<OffchainDataLimit>>,
	pub collection_list: Option<BoundedVec<CollectionId, CollectionSizeLimit>>,
}

impl<AccountId, Balance, AccountSizeLimit, OffchainDataLimit, CollectionSizeLimit>
	MarketplaceData<AccountId, Balance, AccountSizeLimit, OffchainDataLimit, CollectionSizeLimit>
where
	AccountId: Clone + PartialEq + Debug,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	AccountSizeLimit: Get<u32>,
	OffchainDataLimit: Get<u32>,
	CollectionSizeLimit: Get<u32>,
{
	pub fn new(
		owner: AccountId,
		kind: MarketplaceType,
		commission_fee: Option<CompoundFee<Balance>>,
		listing_fee: Option<CompoundFee<Balance>>,
		account_list: Option<BoundedVec<AccountId, AccountSizeLimit>>,
		offchain_data: Option<U8BoundedVec<OffchainDataLimit>>,
		collection_list: Option<BoundedVec<CollectionId, CollectionSizeLimit>>,
	) -> MarketplaceData<AccountId, Balance, AccountSizeLimit, OffchainDataLimit, CollectionSizeLimit>
	{
		Self {
			owner,
			kind,
			commission_fee,
			listing_fee,
			account_list,
			offchain_data,
			collection_list,
		}
	}

	pub fn allowed_to_list(&self, who: &AccountId) -> Option<()> {
		let mut is_in_account_list = false;
		if let Some(account_list) = &self.account_list {
			is_in_account_list = account_list.contains(&who);
		}

		match self.kind {
			MarketplaceType::Public => !is_in_account_list,
			MarketplaceType::Private => is_in_account_list,
		}
		.then_some(())
	}

	pub fn collection_allowed(&self, collection_id: &CollectionId) -> Option<()> {
		let mut is_in_collection_list = false;
		if let Some(collection_list) = &self.collection_list {
			is_in_collection_list = collection_list.contains(collection_id);
		}

		match self.kind {
			MarketplaceType::Public => !is_in_collection_list,
			MarketplaceType::Private => is_in_collection_list,
		}
		.then_some(())
	}
}
