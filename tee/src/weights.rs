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
	fn register_enclave() -> Weight;
	fn unregister_enclave() -> Weight;
	fn update_enclave() -> Weight;
	fn cancel_update() -> Weight;
	fn assign_enclave() -> Weight;
	fn remove_enclave() -> Weight;
	fn remove_registration() -> Weight;
	fn remove_update() -> Weight;
	fn force_update_enclave() -> Weight;
	fn create_cluster() -> Weight;
	fn remove_cluster() -> Weight;
}

impl WeightInfo for () {
	fn register_enclave() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn unregister_enclave() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn update_enclave() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn cancel_update() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn assign_enclave() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn remove_enclave() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn force_update_enclave() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn remove_registration() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn create_cluster() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn remove_cluster() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn remove_update() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
}
