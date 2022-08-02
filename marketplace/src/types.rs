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

use frame_support::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use primitives::{marketplace::MarketplaceId, CompoundFee};
use scale_info::TypeInfo;
use sp_std::fmt::Debug;

#[derive(Encode, Decode, CloneNoBound, Eq, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen))]
pub struct Sale<AccountId, Balance>
where
	AccountId: Clone + PartialEq + Debug,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
{
	pub account_id: AccountId,
	pub marketplace_id: MarketplaceId,
	pub price: Balance,
	pub commission_fee: Option<CompoundFee<Balance>>,
}

impl<AccountId, Balance> Sale<AccountId, Balance>
where
	AccountId: Clone + PartialEq + Debug,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
{
	pub fn new(
		account_id: AccountId,
		marketplace_id: MarketplaceId,
		price: Balance,
		commission_fee: Option<CompoundFee<Balance>>,
	) -> Sale<AccountId, Balance> {
		Self { account_id, marketplace_id, price, commission_fee }
	}
}
