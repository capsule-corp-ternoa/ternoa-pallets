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

use frame_support::weights::Weight;
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn set_threshold() -> Weight;
	fn add_chain() -> Weight;
	fn set_relayers() -> Weight;
	fn vote_for_proposal() -> Weight;
	fn deposit() -> Weight;
	fn set_bridge_fee() -> Weight;
	fn set_deposit_nonce() -> Weight;
}

pub struct TernoaWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for TernoaWeight<T> {
	fn set_threshold() -> Weight {
		195_000_000 as Weight
	}

	fn add_chain() -> Weight {
		195_000_000 as Weight
	}

	fn set_relayers() -> Weight {
		195_000_000 as Weight
	}

	fn vote_for_proposal() -> Weight {
		195_000_000 as Weight
	}

	fn deposit() -> Weight {
		195_000_000 as Weight
	}

	fn set_bridge_fee() -> Weight {
		195_000_000 as Weight
	}

	fn set_deposit_nonce() -> Weight {
		195_000_000 as Weight
	}
}
