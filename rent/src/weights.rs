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
	fn create_contract(s: u32, t: u32) -> Weight;
	fn revoke_contract(s: u32) -> Weight;
	fn rent(s: u32, t: u32) -> Weight;
	fn accept_rent_offer(s: u32, t: u32) -> Weight;
	fn retract_rent_offer() -> Weight;
	fn change_subscription_terms() -> Weight;
	fn accept_subscription_terms() -> Weight;
	fn renew_contract(s: u32) -> Weight;
	fn remove_expired_contract(s: u32) -> Weight;
	fn end_contract(s: u32) -> Weight;
}

/// Weight functions for `ternoa_rent`.
pub struct TernoaWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for TernoaWeight<T> {
	// Storage: Rent ContractNb (r:1 w:1)
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Rent AvailableQueue (r:1 w:1)
	// Storage: Rent Contracts (r:0 w:1)
	fn create_contract(s: u32, t: u32, ) -> Weight {
		(71_239_000 as Weight)
			// Standard Error: 2_755_000
			.saturating_add((103_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: Rent Contracts (r:1 w:1)
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Rent SubscriptionQueue (r:1 w:1)
	// Storage: Rent ContractNb (r:1 w:1)
	fn revoke_contract(s: u32, ) -> Weight {
		(83_409_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: Rent Contracts (r:1 w:1)
	// Storage: System Account (r:3 w:3)
	// Storage: Rent SubscriptionQueue (r:1 w:1)
	// Storage: Rent AvailableQueue (r:1 w:1)
	// Storage: Rent Offers (r:0 w:1)
	fn rent(s: u32, t: u32, ) -> Weight {
		(103_418_000 as Weight)
			// Standard Error: 832_000
			.saturating_add((273_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	// Storage: Rent Contracts (r:1 w:1)
	// Storage: Rent Offers (r:1 w:1)
	// Storage: System Account (r:3 w:3)
	// Storage: Rent SubscriptionQueue (r:1 w:1)
	// Storage: Rent AvailableQueue (r:1 w:1)
	fn accept_rent_offer(s: u32, t: u32, ) -> Weight {
		(49_652_000 as Weight)
			// Standard Error: 767_000
			.saturating_add((2_853_000 as Weight).saturating_mul(s as Weight))
			// Standard Error: 669_000
			.saturating_add((3_352_000 as Weight).saturating_mul(t as Weight))
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	// Storage: Rent Contracts (r:1 w:0)
	// Storage: Rent Offers (r:1 w:1)
	fn retract_rent_offer() -> Weight {
		(21_800_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Rent Contracts (r:1 w:1)
	fn change_subscription_terms() -> Weight {
		(18_400_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Rent Contracts (r:1 w:1)
	fn accept_subscription_terms() -> Weight {
		(18_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Rent Contracts (r:1 w:1)
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: System Account (r:3 w:3)
	// Storage: Rent SubscriptionQueue (r:1 w:1)
	// Storage: Rent ContractNb (r:1 w:1)
	fn end_contract(s: u32, ) -> Weight {
		(87_663_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
	// Storage: Rent Contracts (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	// Storage: Rent SubscriptionQueue (r:1 w:1)
	fn renew_contract(s: u32, ) -> Weight {
		(46_392_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: Rent Contracts (r:1 w:1)
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Rent AvailableQueue (r:1 w:1)
	// Storage: Rent ContractNb (r:1 w:1)
	// Storage: Rent Offers (r:0 w:1)
	fn remove_expired_contract(s: u32, ) -> Weight {
		(70_018_000 as Weight)
			// Standard Error: 786_000
			.saturating_add((2_975_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(7 as Weight))
	}
}
