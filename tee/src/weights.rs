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
	fn force_remove_enclave() -> Weight;
	fn remove_registration() -> Weight;
	fn reject_update() -> Weight;
	fn force_update_enclave() -> Weight;
	fn create_cluster() -> Weight;
	fn update_cluster() -> Weight;
	fn remove_cluster() -> Weight;
	fn withdraw_unbonded() -> Weight;
	fn register_metrics_server() -> Weight;
	fn unregister_metrics_server() -> Weight;
	fn force_update_metrics_server_type() -> Weight;
	fn submit_metrics_server_report() -> Weight;
	fn set_report_params_weightage() -> Weight;
	fn set_staking_amount() -> Weight;
	fn set_daily_reward_pool() -> Weight;
	fn claim_rewards() -> Weight;
	fn update_operator_assigned_era() -> Weight;
	fn bond_extra() -> Weight;
	fn refund_excess() -> Weight;
}

impl WeightInfo for () {
	fn register_enclave() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn unregister_enclave() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn update_enclave() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn cancel_update() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn assign_enclave() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn force_remove_enclave() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn force_update_enclave() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn remove_registration() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn create_cluster() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn update_cluster() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn remove_cluster() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn reject_update() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn withdraw_unbonded() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn register_metrics_server() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn unregister_metrics_server() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn force_update_metrics_server_type() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn submit_metrics_server_report() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn set_report_params_weightage() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn set_staking_amount() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn set_daily_reward_pool() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn claim_rewards() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn update_operator_assigned_era() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn bond_extra() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
	fn refund_excess() -> Weight {
		Weight::from_parts(10_000_000, 0)
	}
}
