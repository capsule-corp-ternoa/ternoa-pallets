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
	fn create_auction(s: u32) -> Weight;
	fn cancel_auction(s: u32) -> Weight;
	fn end_auction(s: u32) -> Weight;
	fn add_bid(s: u32) -> Weight;
	fn remove_bid(s: u32) -> Weight;
	fn buy_it_now(_s: u32) -> Weight;
	fn claim() -> Weight;
}

/// Weight functions for `ternoa_auctions`.
impl WeightInfo for () {
	fn create_auction(_s: u32) -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn cancel_auction(_s: u32) -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn end_auction(_s: u32) -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn add_bid(_s: u32) -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn remove_bid(_s: u32) -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn buy_it_now(_s: u32) -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn claim() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
}
