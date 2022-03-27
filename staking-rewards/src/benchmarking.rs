use crate::{BalanceOf, Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, Saturating, StaticLookup};
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
