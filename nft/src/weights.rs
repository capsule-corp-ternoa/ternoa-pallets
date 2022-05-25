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

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn create_nft() -> Weight;
	fn burn_nft() -> Weight;
	fn transfer_nft() -> Weight;
	fn delegate_nft() -> Weight;
	fn set_royalty() -> Weight;
	fn set_nft_mint_fee() -> Weight;
	fn create_collection() -> Weight;
	fn burn_collection() -> Weight;
	fn close_collection() -> Weight;
	fn limit_collection() -> Weight;
	fn add_nft_to_collection() -> Weight;
}

/// Weight functions for `ternoa_nfts`.
pub struct TernoaWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for TernoaWeight<T> {
	// Storage: NFT NFTMintFee (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: NFT Collections (r:1 w:1)
	// Storage: NFT NextNFTId (r:1 w:1)
	// Storage: NFT NFTs (r:0 w:1)
	fn create_nft() -> Weight {
		(52_700_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: NFT NFTs (r:1 w:1)
	// Storage: NFT Collections (r:1 w:1)
	fn burn_nft() -> Weight {
		(22_500_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: NFT NFTs (r:1 w:1)
	fn transfer_nft() -> Weight {
		(17_600_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: NFT NFTs (r:1 w:1)
	// Storage: NFT DelegatedNFTs (r:0 w:1)
	fn delegate_nft() -> Weight {
		(17_700_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: NFT NFTs (r:1 w:1)
	fn set_royalty() -> Weight {
		(29_900_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: NFT NFTMintFee (r:0 w:1)
	fn set_nft_mint_fee() -> Weight {
		(20_000_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: NFT NextCollectionId (r:1 w:1)
	// Storage: NFT Collections (r:0 w:1)
	fn create_collection() -> Weight {
		(17_800_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: NFT Collections (r:1 w:1)
	fn burn_collection() -> Weight {
		(16_400_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: NFT Collections (r:1 w:1)
	fn close_collection() -> Weight {
		(16_100_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: NFT Collections (r:1 w:1)
	fn limit_collection() -> Weight {
		(17_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: NFT Collections (r:1 w:1)
	// Storage: NFT NFTs (r:1 w:1)
	fn add_nft_to_collection() -> Weight {
		(21_800_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
