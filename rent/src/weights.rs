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
	fn create_contract(s: u32) -> Weight;
	fn revoke_contract(s: u32) -> Weight;
	fn cancel_contract(s: u32) -> Weight;
	fn rent(_s: u32) -> Weight;
	fn make_rent_offer(_s: u32) -> Weight;
	fn accept_rent_offer(s: u32) -> Weight;
	fn retract_rent_offer(_s: u32) -> Weight;
	fn change_subscription_terms(_s: u32) -> Weight;
	fn accept_subscription_terms(_s: u32) -> Weight;
}

/// Weight functions for `ternoa_rent`.
pub struct TernoaWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for TernoaWeight<T> {
	// Storage: Rent NumberOfCurrentContracts (r:1 w:1)
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Rent AvailableQueue (r:1 w:1)
	// Storage: Rent Contracts (r:0 w:1)
	/// The range of component `s` is `[0, 8]`.
	fn create_contract(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	// Storage: Rent Contracts (r:1 w:1)
	// Storage: NFT Nfts (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Rent SubscriptionQueue (r:1 w:1)
	// Storage: Rent NumberOfCurrentContracts (r:1 w:1)
	/// The range of component `s` is `[0, 9]`.
	fn revoke_contract(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn cancel_contract(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	// Storage: Rent Contracts (r:1 w:1)
	// Storage: System Account (r:3 w:3)
	// Storage: Rent SubscriptionQueue (r:1 w:1)
	// Storage: Rent AvailableQueue (r:1 w:1)
	// Storage: Rent Offers (r:0 w:1)
	/// The range of component `s` is `[0, 8]`.
	/// The range of component `t` is `[0, 9]`.
	fn rent(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn make_rent_offer(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	// Storage: Rent Contracts (r:1 w:1)
	// Storage: Rent Offers (r:1 w:1)
	// Storage: System Account (r:3 w:3)
	// Storage: Rent SubscriptionQueue (r:1 w:1)
	// Storage: Rent AvailableQueue (r:1 w:1)
	/// The range of component `s` is `[0, 8]`.
	/// The range of component `t` is `[0, 9]`.
	/// The range of component `u` is `[0, 2]`.
	fn accept_rent_offer(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	// Storage: Rent Contracts (r:1 w:0)
	// Storage: Rent Offers (r:1 w:1)
	/// The range of component `s` is `[0, 2]`.
	fn retract_rent_offer(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	// Storage: Rent Contracts (r:1 w:1)
	fn change_subscription_terms(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	// Storage: Rent Contracts (r:1 w:1)
	fn accept_subscription_terms(_s: u32) -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
}
