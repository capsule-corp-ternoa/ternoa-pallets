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
	fn create_marketplace() -> Weight;
	fn set_marketplace_owner() -> Weight;
	fn set_marketplace_kind() -> Weight;
	fn set_marketplace_configuration() -> Weight;
	fn set_marketplace_mint_fee() -> Weight;
	fn list_nft() -> Weight;
	fn unlist_nft() -> Weight;
	fn buy_nft() -> Weight;
}

impl WeightInfo for () {
	// Storage: Marketplace MarketplaceMintFee (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Marketplace NextMarketplaceId (r:1 w:1)
	// Storage: Marketplace Marketplaces (r:0 w:1)
	fn create_marketplace() -> Weight {
		(47_400_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn set_marketplace_owner() -> Weight {
		(16_500_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn set_marketplace_kind() -> Weight {
		(16_600_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace Marketplaces (r:1 w:1)
	fn set_marketplace_configuration() -> Weight {
		(41_800_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: Marketplace MarketplaceMintFee (r:0 w:1)
	fn set_marketplace_mint_fee() -> Weight {
		(11_000_000 as Weight)
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: Marketplace Marketplaces (r:1 w:0)
	// Storage: Marketplace ListedNfts (r:0 w:1)
	fn list_nft() -> Weight {
		(26_300_000 as Weight)
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: Marketplace ListedNfts (r:1 w:1)
	fn unlist_nft() -> Weight {
		(22_000_000 as Weight)
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: Marketplace ListedNfts (r:1 w:1)
	// Storage: Marketplace Marketplaces (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	fn buy_nft() -> Weight {
		(73_500_000 as Weight)
			.saturating_add(DbWeight::get().reads(5 as Weight))
			.saturating_add(DbWeight::get().writes(4 as Weight))
	}
}
