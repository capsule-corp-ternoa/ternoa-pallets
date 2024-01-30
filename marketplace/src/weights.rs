// Copyright 2023 Capsule Corp (France) SAS.
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

use frame_support::weights::Weight;

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
	fn create_marketplace() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn set_marketplace_owner() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn set_marketplace_kind() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn set_marketplace_configuration() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn set_marketplace_mint_fee() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn list_nft() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn unlist_nft() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn buy_nft() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
}
