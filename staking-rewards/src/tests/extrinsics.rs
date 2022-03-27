use super::mock::*;
use crate::{tests::mock, Event as StakingRewardsEvent};
use frame_support::{assert_ok, error::BadOrigin};
use frame_system::RawOrigin;

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

mod set_session_extra_reward_payout {
	use super::*;
	use frame_support::assert_noop;

	use crate::StakingRewardsData;

	#[test]
	fn set_session_extra_reward_payout() {
		ExtBuilder::new_build().execute_with(|| {
			let value: Balance = 100u64.into();

			// Create NFT with new serie id while there is no series already registered
			let ok = StakingRewards::set_session_extra_reward_payout(root(), value);
			assert_ok!(ok);

			// Final state checks
			let expected =
				StakingRewardsData { session_era_payout: 0, session_extra_reward_payout: value };
			assert_eq!(StakingRewards::session_era_payout(), expected);

			// Events checks
			let event = StakingRewardsEvent::SessionExtraRewardPayoutChanged { value };
			let event = Event::StakingRewards(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build().execute_with(|| {
			// Create NFT with new serie id while there is no series already registered
			let ok = StakingRewards::set_session_extra_reward_payout(origin(ALICE), 100u64);
			assert_noop!(ok, BadOrigin);
		})
	}
}
