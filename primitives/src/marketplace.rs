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
use sp_arithmetic::per_things::Permill;
use sp_runtime::RuntimeDebug;
use sp_std::fmt::Debug;

use crate::U8BoundedVec;

pub type MarketplaceId = u32;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum MarketplaceType {
	Public = 0,
	Private = 1,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MarketplaceFee<Balance> {
	Flat(Balance),
	Percentage(Permill),
}

// impl <Balance> MarketplaceFee<Balance>
// where Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd {
// 	fn default() -> Self {
// 		MarketplaceFee::Percentage(Permill::from_parts(0))
// 	}
// }

#[derive(
	Encode, Decode, CloneNoBound, Eq, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(AccountSizeLimit, OffchainDataLimit,))]
#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen))]
pub struct MarketplaceData<AccountId, Balance, AccountSizeLimit, OffchainDataLimit>
where
	AccountId: Clone + PartialEq + Debug,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	AccountSizeLimit: Get<u32>,
	OffchainDataLimit: Get<u32>,
{
	pub owner: AccountId,
	pub kind: MarketplaceType,
	pub commission_fee: Option<MarketplaceFee<Balance>>,
	pub listing_fee: Option<MarketplaceFee<Balance>>,
	pub account_list: Option<BoundedVec<AccountId, AccountSizeLimit>>,
	pub offchain_data: Option<U8BoundedVec<OffchainDataLimit>>,
}

impl<AccountId, Balance, AccountSizeLimit, OffchainDataLimit>
	MarketplaceData<AccountId, Balance, AccountSizeLimit, OffchainDataLimit>
where
	AccountId: Clone + PartialEq + Debug,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
	AccountSizeLimit: Get<u32>,
	OffchainDataLimit: Get<u32>,
{
	pub fn new(
		owner: AccountId,
		kind: MarketplaceType,
		commission_fee: Option<MarketplaceFee<Balance>>,
		listing_fee: Option<MarketplaceFee<Balance>>,
		account_list: Option<BoundedVec<AccountId, AccountSizeLimit>>,
		offchain_data: Option<U8BoundedVec<OffchainDataLimit>>,
	) -> MarketplaceData<AccountId, Balance, AccountSizeLimit, OffchainDataLimit> {
		Self { owner, kind, commission_fee, listing_fee, account_list, offchain_data }
	}
}
