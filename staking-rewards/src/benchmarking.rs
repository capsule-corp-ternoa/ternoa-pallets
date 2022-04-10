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

use crate::{Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_runtime::traits::Saturating;
use sp_std::prelude::*;

benchmarks! {
	set_session_extra_reward_payout {
		let mut expected_value = Pallet::<T>::session_era_payout();
		expected_value.session_extra_reward_payout = expected_value.session_extra_reward_payout.saturating_add(10000u32.into());
	}: _(RawOrigin::Root, expected_value.session_extra_reward_payout.clone())
	verify {
		assert_eq!(Pallet::<T>::session_era_payout(), expected_value.clone());
	}
}

impl_benchmark_test_suite!(Pallet, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
