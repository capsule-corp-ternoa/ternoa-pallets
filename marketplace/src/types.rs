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

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use primitives::{marketplace::MarketplaceId, nfts::NFTId};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct SaleData<AccountId, Balance>
where
	Balance: Clone + Default,
{
	pub account_id: AccountId,
	pub price: Balance,
	pub marketplace_id: MarketplaceId,
}

impl<AccountId, Balance> Default for SaleData<AccountId, Balance>
where
	AccountId: Clone + Default,
	Balance: Clone + Default,
{
	fn default() -> Self {
		Self {
			account_id: Default::default(),
			price: Default::default(),
			marketplace_id: Default::default(),
		}
	}
}

impl<AccountId, Balance> SaleData<AccountId, Balance>
where
	Balance: Clone + Default,
{
	pub fn new(
		account_id: AccountId,
		price: Balance,
		marketplace_id: MarketplaceId,
	) -> SaleData<AccountId, Balance> {
		Self { account_id, price, marketplace_id }
	}
}

// nft_id, account id, price, market id
pub type NFTsGenesis<AccountId, Balance> = (NFTId, AccountId, Balance, MarketplaceId);
