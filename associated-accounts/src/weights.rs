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
	fn set_account() -> Weight;
	fn add_new_supported_account() -> Weight;
	fn remove_supported_account() -> Weight;
}

/// Weight functions for `ternoa_associated_accounts`.
pub struct TernoaWeights<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for TernoaWeights<T> {
	// Storage: AssociatedAccounts SupportedAccounts (r:1 w:0)
	// Storage: AssociatedAccounts Users (r:1 w:1)
	fn set_account() -> Weight {
		(13_180_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: AssociatedAccounts SupportedAccounts (r:1 w:1)
	fn add_new_supported_account() -> Weight {
		(10_750_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: AssociatedAccounts SupportedAccounts (r:1 w:1)
	fn remove_supported_account() -> Weight {
		(10_690_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
