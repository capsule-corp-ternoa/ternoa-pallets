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

use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub trait WeightInfo {
	fn list_nft() -> Weight;
	fn unlist_nft() -> Weight;
	fn buy_nft() -> Weight;
	fn create_marketplace() -> Weight;
	fn set_marketplace_owner() -> Weight;
	fn set_marketplace_type() -> Weight;
	fn set_marketplace_name() -> Weight;
	fn set_marketplace_mint_fee() -> Weight;
	fn set_marketplace_commission_fee() -> Weight;
	fn set_marketplace_uri() -> Weight;
	fn set_marketplace_logo_uri() -> Weight;
	fn set_marketplace_description() -> Weight;
	fn add_account_to_allow_list() -> Weight;
	fn remove_account_from_allow_list() -> Weight;
	fn add_account_to_disallow_list() -> Weight;
	fn remove_account_from_disallow_list() -> Weight;
}

impl WeightInfo for () {
	// Storage: Nfts Data (r:1 w:1)
	// Storage: Nfts Series (r:1 w:0)
	// Storage: Capsules Capsules (r:1 w:0)
	// Storage: Marketplace Marketplaces (r:1 w:0)
	// Storage: Marketplace NFTsForSale (r:0 w:1)
	fn list_nft() -> Weight {
		(51_580_000 as Weight)
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	// Storage: Nfts Data (r:1 w:1)
	// Storage: Marketplace NFTsForSale (r:1 w:1)
	fn unlist_nft() -> Weight {
		(34_760_000 as Weight)
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	// Storage: Marketplace NFTsForSale (r:1 w:1)
	// Storage: Marketplace Marketplaces (r:1 w:0)
	// Storage: Nfts Data (r:1 w:1)
	fn buy_nft() -> Weight {
		(43_881_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	// Storage: Marketplace MarketplaceMintFee (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Marketplace MarketplaceIdGenerator (r:1 w:1)
	// Storage: Marketplace Marketplaces (r:0 w:1)
	fn create_marketplace() -> Weight {
		(66_831_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn add_account_to_allow_list() -> Weight {
		(27_150_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn remove_account_from_allow_list() -> Weight {
		(25_770_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn set_marketplace_owner() -> Weight {
		(26_170_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn set_marketplace_type() -> Weight {
		(25_990_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn set_marketplace_name() -> Weight {
		(26_481_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace MarketplaceMintFee (r:0 w:1)
	fn set_marketplace_mint_fee() -> Weight {
		(19_041_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn set_marketplace_commission_fee() -> Weight {
		(25_770_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn set_marketplace_uri() -> Weight {
		(26_270_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn set_marketplace_logo_uri() -> Weight {
		(26_501_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn set_marketplace_description() -> Weight {
		(26_501_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn add_account_to_disallow_list() -> Weight {
		(26_810_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn remove_account_from_disallow_list() -> Weight {
		(25_470_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
