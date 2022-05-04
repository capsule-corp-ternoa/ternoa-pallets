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
	fn create() -> Weight;
	fn transfer() -> Weight;
	fn burn() -> Weight;
	fn finish_series() -> Weight;
	fn set_nft_mint_fee() -> Weight;
	fn delegate() -> Weight;
	fn set_nft_royaltie_fee() -> Weight;
}

/// Weight functions for `ternoa_nfts`.
pub struct TernoaWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for TernoaWeight<T> {
	// Storage: Nfts NFTMintFee (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: Nfts NFTIdGenerator (r:1 w:1)
	// Storage: Nfts SeriesIdGenerator (r:1 w:1)
	// Storage: Nfts Series (r:1 w:1)
	// Storage: Nfts Data (r:0 w:1)
	fn create() -> Weight {
		(46_461_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: Nfts Data (r:1 w:1)
	// Storage: Nfts Series (r:1 w:0)
	fn transfer() -> Weight {
		(17_960_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Nfts Data (r:1 w:1)
	fn burn() -> Weight {
		(14_920_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Nfts Series (r:1 w:1)
	fn finish_series() -> Weight {
		(14_170_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Nfts NFTMintFee (r:0 w:1)
	fn set_nft_mint_fee() -> Weight {
		(10_100_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Nfts Data (r:1 w:1)
	fn delegate() -> Weight {
		(14_870_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Nfts Data (r:0 w:1)
	fn set_nft_royaltie_fee() -> Weight {
		(10_100_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
