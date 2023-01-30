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

use super::mock::{self, *};
use frame_support::{assert_noop, assert_ok, bounded_vec, error::BadOrigin, BoundedVec};
use frame_system::RawOrigin;

use crate::{
	self as ternoa_bridge,
	tests::mock::{Bridge, ExtBuilder, ProposalLifetime, System, RELAYER_A, RELAYER_B},
	types::{Proposal, ProposalStatus},
	ChainId, DepositNonce, Error, Event as BridgeEvent,
};

fn origin(account: u64) -> mock::RuntimeOrigin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::RuntimeOrigin {
	RawOrigin::Root.into()
}

const CHAIN_ID: ChainId = 0;
const THRESHOLD: u32 = 3;
const NONCE: DepositNonce = 0;
const AMOUNT: u64 = 100;

pub mod set_threshold {
	pub use super::*;

	#[test]
	fn set_threshold() {
		ExtBuilder::build().execute_with(|| {
			let new_threshold = Bridge::relayer_vote_threshold() + 1;

			let ok = Bridge::set_threshold(root(), new_threshold);
			assert_ok!(ok);

			assert_eq!(Bridge::relayer_vote_threshold(), new_threshold);

			let event = BridgeEvent::RelayerThresholdUpdated { threshold: new_threshold };
			let event = RuntimeEvent::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::build().execute_with(|| {
			assert_noop!(Bridge::set_threshold(origin(RELAYER_A), 3), BadOrigin);
		});
	}

	#[test]
	fn threshold_cannot_be_zero() {
		ExtBuilder::build().execute_with(|| {
			assert_noop!(Bridge::set_threshold(root(), 0), Error::<Test>::ThresholdCannotBeZero);
		});
	}
}

pub mod add_chain {
	pub use super::*;

	#[test]
	fn add_chain() {
		ExtBuilder::build().execute_with(|| {
			let ok = Bridge::add_chain(root(), CHAIN_ID);
			assert_ok!(ok);

			assert_eq!(Bridge::chain_nonces(CHAIN_ID), Some(0));

			let event = BridgeEvent::ChainAllowed { chain_id: CHAIN_ID };
			let event = RuntimeEvent::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::build().execute_with(|| {
			assert_noop!(Bridge::add_chain(origin(RELAYER_A), 0), BadOrigin);
		});
	}

	#[test]
	fn cannot_add_self_to_allowed_chain_list() {
		ExtBuilder::build().execute_with(|| {
			let chain_id = <Test as ternoa_bridge::Config>::ChainId::get();
			assert_noop!(
				Bridge::add_chain(root(), chain_id),
				Error::<Test>::CannotAddSelfToAllowedChainList
			);
		});
	}

	#[test]
	fn chain_already_whitelisted() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(
				Bridge::add_chain(root(), CHAIN_ID),
				Error::<Test>::ChainAlreadyWhitelisted
			);
		});
	}
}

pub mod set_relayers {
	pub use super::*;

	#[test]
	fn set_relayers() {
		ExtBuilder::build().execute_with(|| {
			let relayers: BoundedVec<u64, RelayerCountLimit> = bounded_vec![RELAYER_A, RELAYER_B];

			let ok = Bridge::set_relayers(root(), relayers.clone());
			assert_ok!(ok);

			assert_eq!(Bridge::relayers().clone(), relayers.clone());

			let event = BridgeEvent::RelayersUpdated { relayers };
			let event = RuntimeEvent::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::build().execute_with(|| {
			assert_noop!(Bridge::set_relayers(origin(RELAYER_A), bounded_vec![]), BadOrigin);
		});
	}
}

pub mod set_deposit_nonce {
	pub use super::*;

	#[test]
	fn set_deposit_nonce() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let new_nonce = 1;
			let ok = Bridge::set_deposit_nonce(root(), CHAIN_ID, new_nonce);
			assert_ok!(ok);

			// Check storage
			assert_eq!(Bridge::chain_nonces(CHAIN_ID), Some(new_nonce));

			// Check events
			let event = BridgeEvent::DepositNonceUpdated { chain_id: CHAIN_ID, nonce: new_nonce };
			let event = RuntimeEvent::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::build().execute_with(|| {
			assert_noop!(Bridge::set_deposit_nonce(origin(RELAYER_A), 0, 1), BadOrigin);
		});
	}

	#[test]
	fn chain_not_found() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(Bridge::set_deposit_nonce(root(), 1, 1), Error::<Test>::ChainNotFound);
		});
	}

	#[test]
	fn new_nonce_too_low() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(
				Bridge::set_deposit_nonce(root(), CHAIN_ID, 0),
				Error::<Test>::NewNonceTooLow
			);
		});
	}
}

pub mod vote_for_proposal {
	pub use super::*;

	#[test]
	fn vote_for_proposal_not_existing() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let recipient = RELAYER_C;
			let deposit_nonce = Bridge::chain_nonces(CHAIN_ID).unwrap();
			let account = RELAYER_A;

			let ok = Bridge::vote_for_proposal(
				origin(account),
				CHAIN_ID,
				deposit_nonce,
				recipient,
				AMOUNT,
			);
			assert_ok!(ok);

			let initial_votes = bounded_vec![account];
			let now = System::block_number();
			let block_expiry = now + <Test as ternoa_bridge::Config>::ProposalLifetime::get();
			let expected_proposal = Proposal::new(initial_votes, block_expiry);

			let actual_proposal =
				Bridge::get_votes(CHAIN_ID, (deposit_nonce, recipient, AMOUNT)).unwrap();
			assert_eq!(actual_proposal, expected_proposal);

			let event =
				BridgeEvent::RelayerVoted { chain_id: CHAIN_ID, nonce: deposit_nonce, account };
			let event = RuntimeEvent::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn vote_for_proposal_existing() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let recipient = RELAYER_C;
			let deposit_nonce = Bridge::chain_nonces(CHAIN_ID).unwrap();

			let account = RELAYER_B;
			let ok = Bridge::vote_for_proposal(
				origin(RELAYER_A),
				CHAIN_ID,
				deposit_nonce,
				recipient,
				AMOUNT,
			);
			assert_ok!(ok);

			let ok = Bridge::vote_for_proposal(
				origin(account),
				CHAIN_ID,
				deposit_nonce,
				recipient,
				AMOUNT,
			);
			assert_ok!(ok);

			let initial_votes = bounded_vec![RELAYER_A, account];
			let now = System::block_number();
			let block_expiry = now + <Test as ternoa_bridge::Config>::ProposalLifetime::get();
			let expected_proposal = Proposal::new(initial_votes, block_expiry);

			let actual_proposal =
				Bridge::get_votes(CHAIN_ID, (deposit_nonce, recipient, AMOUNT)).unwrap();
			assert_eq!(actual_proposal, expected_proposal);

			let event =
				BridgeEvent::RelayerVoted { chain_id: CHAIN_ID, nonce: deposit_nonce, account };
			let event = RuntimeEvent::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn vote_for_proposal_existing_and_reach_threshold() {
		ExtBuilder::build_with(CHAIN_ID, 1).execute_with(|| {
			let recipient = RELAYER_C;
			let deposit_nonce = Bridge::chain_nonces(CHAIN_ID).unwrap();
			let relayer_c_before = Balances::free_balance(RELAYER_C);
			let total_issuance = Balances::total_issuance();

			let ok = Bridge::vote_for_proposal(
				origin(RELAYER_A),
				CHAIN_ID,
				deposit_nonce,
				recipient,
				AMOUNT,
			);
			assert_ok!(ok);

			let initial_votes = bounded_vec![RELAYER_A];
			let now = System::block_number();
			let block_expiry = now + <Test as ternoa_bridge::Config>::ProposalLifetime::get();
			let mut expected_proposal = Proposal::new(initial_votes, block_expiry);
			expected_proposal.status = ProposalStatus::Approved;

			let actual_proposal =
				Bridge::get_votes(CHAIN_ID, (deposit_nonce, recipient, AMOUNT)).unwrap();
			assert_eq!(actual_proposal, expected_proposal);

			// Fund checks
			assert_eq!(relayer_c_before + AMOUNT, Balances::free_balance(RELAYER_C));
			assert_eq!(Balances::total_issuance(), total_issuance + AMOUNT);

			let event = BridgeEvent::ProposalApproved { chain_id: CHAIN_ID, nonce: deposit_nonce };
			let event = RuntimeEvent::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn must_be_relayer() {
		ExtBuilder::build().execute_with(|| {
			assert_noop!(
				Bridge::vote_for_proposal(origin(5), 0, 0, RELAYER_C, AMOUNT),
				Error::<Test>::MustBeRelayer
			);
		});
	}

	#[test]
	fn chain_not_allowed() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(
				Bridge::vote_for_proposal(origin(RELAYER_A), 1, 0, RELAYER_C, AMOUNT),
				Error::<Test>::ChainNotAllowed
			);
		});
	}

	#[test]
	fn proposal_already_completed() {
		ExtBuilder::build_with(CHAIN_ID, 1).execute_with(|| {
			let ok =
				Bridge::vote_for_proposal(origin(RELAYER_A), CHAIN_ID, NONCE, RELAYER_C, AMOUNT);
			assert_ok!(ok);

			assert_noop!(
				Bridge::vote_for_proposal(origin(RELAYER_B), CHAIN_ID, NONCE, RELAYER_C, AMOUNT),
				Error::<Test>::ProposalAlreadyCompleted
			);
		});
	}

	#[test]
	fn proposal_expired() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let ok =
				Bridge::vote_for_proposal(origin(RELAYER_A), CHAIN_ID, NONCE, RELAYER_C, AMOUNT);
			assert_ok!(ok);

			System::set_block_number(
				frame_system::Pallet::<Test>::block_number() + ProposalLifetime::get() + 1,
			);

			assert_noop!(
				Bridge::vote_for_proposal(origin(RELAYER_B), CHAIN_ID, NONCE, RELAYER_C, AMOUNT),
				Error::<Test>::ProposalExpired
			);
		});
	}

	#[test]
	fn relayer_already_voted() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let ok =
				Bridge::vote_for_proposal(origin(RELAYER_A), CHAIN_ID, NONCE, RELAYER_C, AMOUNT);
			assert_ok!(ok);

			assert_noop!(
				Bridge::vote_for_proposal(origin(RELAYER_A), CHAIN_ID, NONCE, RELAYER_C, AMOUNT),
				Error::<Test>::RelayerAlreadyVoted
			);
		});
	}
}

pub mod deposit {
	pub use super::*;

	#[test]
	fn deposit() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let recipient = vec![0];
			let relayer_a_balance_before = Balances::free_balance(RELAYER_A);
			let deposit_nonce = Bridge::chain_nonces(CHAIN_ID).unwrap();
			let total_issuance = Balances::total_issuance();
			let collector_before = Balances::free_balance(COLLECTOR);

			let bridge_fee = Bridge::bridge_fee();
			assert_ne!(bridge_fee, 0);

			let ok = Bridge::deposit(origin(RELAYER_A), AMOUNT, recipient.clone(), CHAIN_ID);
			assert_ok!(ok);

			let new_deposit_nonce = Bridge::chain_nonces(CHAIN_ID).unwrap();

			assert_eq!(
				Balances::free_balance(RELAYER_A),
				relayer_a_balance_before - AMOUNT - bridge_fee
			);
			assert_eq!(Balances::total_issuance(), total_issuance - AMOUNT);
			assert_eq!(Balances::free_balance(COLLECTOR), collector_before + bridge_fee);
			assert_eq!(Bridge::chain_nonces(CHAIN_ID).unwrap(), deposit_nonce + 1);

			let event = BridgeEvent::DepositMade {
				chain_id: CHAIN_ID,
				nonce: new_deposit_nonce,
				amount: AMOUNT.into(),
				recipient,
			};
			let event = RuntimeEvent::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn chain_not_allowed() {
		ExtBuilder::build().execute_with(|| {
			assert_noop!(
				Bridge::deposit(origin(RELAYER_A), 10, vec![0], 0),
				Error::<Test>::ChainNotAllowed
			);
		});
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(
				Bridge::deposit(origin(RELAYER_A), 200000000, vec![0], CHAIN_ID),
				Error::<Test>::InsufficientBalance
			);
		});
	}
}

pub mod set_bridge_fee {
	pub use super::*;

	#[test]
	fn set_bridge_fee() {
		ExtBuilder::build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let old_fee = Bridge::bridge_fee();
			assert_ne!(old_fee, AMOUNT);

			let ok = Bridge::set_bridge_fee(root(), AMOUNT);
			assert_ok!(ok);

			assert_eq!(Bridge::bridge_fee(), AMOUNT);

			let event = BridgeEvent::BridgeFeeUpdated { fee: AMOUNT };
			let event = RuntimeEvent::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn set_bridge_fee_bad_origin() {
		ExtBuilder::build().execute_with(|| {
			assert_noop!(Bridge::set_bridge_fee(origin(RELAYER_A), AMOUNT), BadOrigin);
		});
	}
}
