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

use frame_support::weights::{RefTimeWeight, Weight};

pub trait WeightInfo {
	fn register_enclave() -> Weight;
	fn assign_enclave() -> Weight;
	fn unassign_enclave() -> Weight;
	fn update_enclave() -> Weight;
	fn change_enclave_owner() -> Weight;
	fn create_cluster() -> Weight;
	fn remove_cluster() -> Weight;
}

impl WeightInfo for () {
	// Storage: Sgx EnclaveIndex (r:1 w:1)
	// Storage: Sgx EnclaveIdGenerator (r:1 w:1)
	// Storage: Sgx EnclaveRegistry (r:0 w:1)
	fn register_enclave() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: Sgx EnclaveIndex (r:1 w:0)
	// Storage: Sgx ClusterIndex (r:1 w:1)
	// Storage: Sgx ClusterRegistry (r:1 w:1)
	fn assign_enclave() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: Sgx EnclaveIndex (r:1 w:0)
	// Storage: Sgx ClusterIndex (r:1 w:1)
	// Storage: Sgx ClusterRegistry (r:1 w:1)
	fn unassign_enclave() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: Sgx EnclaveIndex (r:1 w:0)
	// Storage: Sgx EnclaveRegistry (r:1 w:1)
	fn update_enclave() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: Sgx EnclaveIndex (r:2 w:2)
	// Storage: Sgx EnclaveRegistry (r:1 w:0)
	fn change_enclave_owner() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: Sgx EnclaveIdGenerator (r:1 w:0)
	// Storage: Sgx ClusterIdGenerator (r:0 w:1)
	// Storage: Sgx ClusterRegistry (r:0 w:1)
	fn create_cluster() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
	// Storage: Sgx ClusterRegistry (r:1 w:1)
	fn remove_cluster() -> Weight {
		Weight::from_ref_time(10_000_000 as RefTimeWeight)
	}
}
