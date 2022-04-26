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

use super::mock::{self, *};
use frame_support::{assert_noop, assert_ok, bounded_vec, error::BadOrigin, BoundedVec};
use frame_system::RawOrigin;

use crate::{
	self as ternoa_bridge,
	tests::mock::{Bridge, ExtBuilder, ProposalLifetime, System, RELAYER_A, RELAYER_B},
	types::{Proposal, ProposalStatus},
	ChainId, Error, Event as BridgeEvent,
};

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

const CHAIN_ID: ChainId = 0;
const THRESHOLD: u32 = 3;

pub mod try_to_complete {
	pub use super::*;

	#[test]
	fn try_to_complete_approved() {
		ExtBuilder::default().build().execute_with(|| {
			let mut prop: Proposal<_, _, RelayerCountLimit> = Proposal {
				votes: bounded_vec![(RELAYER_A, true)],
				status: ProposalStatus::Initiated,
				expiry: ProposalLifetime::get(),
			};

			prop.try_to_complete(1);
			assert_eq!(prop.status, ProposalStatus::Approved);
		});
	}

	#[test]
	fn try_to_complete_rejected() {
		ExtBuilder::default().build().execute_with(|| {
			let mut prop: Proposal<_, _, RelayerCountLimit> = Proposal {
				votes: bounded_vec![(RELAYER_A, false)],
				status: ProposalStatus::Initiated,
				expiry: ProposalLifetime::get(),
			};

			prop.try_to_complete(1);
			assert_eq!(prop.status, ProposalStatus::Rejected);
		});
	}

	#[test]
	fn try_to_complete_bad_threshold() {
		ExtBuilder::default().build().execute_with(|| {
			let mut prop: Proposal<_, _, RelayerCountLimit> = Proposal {
				votes: bounded_vec![(RELAYER_A, true), (RELAYER_B, true)],
				status: ProposalStatus::Initiated,
				expiry: ProposalLifetime::get(),
			};

			prop.try_to_complete(3);
			assert_eq!(prop.status, ProposalStatus::Initiated);
		});
	}
}

pub mod add_chain {
	pub use super::*;

	#[test]
	fn add_chain() {
		ExtBuilder::default().build().execute_with(|| {
			assert!(!Bridge::chain_allowed(0));

			let ok = Bridge::add_chain(root(), 0);
			assert_ok!(ok);

			assert!(Bridge::chain_nonces(0).is_some());

			let event = BridgeEvent::NewChainAllowed { chain_id: 0 };
			let event = Event::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::default().build().execute_with(|| {
			assert!(!Bridge::chain_allowed(0));

			assert_noop!(Bridge::add_chain(origin(RELAYER_A), 0), BadOrigin);
		});
	}

	#[test]
	fn chain_not_found() {
		ExtBuilder::default().build().execute_with(|| {
			assert!(!Bridge::chain_allowed(0));

			assert_noop!(
				Bridge::add_chain(root(), MockChainId::get()),
				Error::<Test>::ChainNotFound
			);
		});
	}

	#[test]
	fn chain_already_whitelisted() {
		ExtBuilder::default().build().execute_with(|| {
			assert!(!Bridge::chain_allowed(0));

			let ok = Bridge::add_chain(root(), 0);
			assert_ok!(ok);

			assert_noop!(Bridge::add_chain(root(), 0), Error::<Test>::ChainAlreadyWhitelisted);
		});
	}
}

pub mod set_threshold {
	pub use super::*;

	#[test]
	fn set_threshold() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(Bridge::relayer_vote_threshold(), DEFAULT_RELAYER_VOTE_THRESHOLD);

			let ok = Bridge::set_threshold(root(), 3);
			assert_ok!(ok);

			assert_eq!(Bridge::relayer_vote_threshold(), 3);

			let event = BridgeEvent::RelayerThresholdUpdated { threshold: 3 };
			let event = Event::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(Bridge::relayer_vote_threshold(), DEFAULT_RELAYER_VOTE_THRESHOLD);

			assert_noop!(Bridge::set_threshold(origin(RELAYER_A), 3), BadOrigin);
		});
	}

	#[test]
	fn threshold_cannot_be_zero() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(Bridge::relayer_vote_threshold(), DEFAULT_RELAYER_VOTE_THRESHOLD);

			assert_noop!(Bridge::set_threshold(root(), 0), Error::<Test>::ThresholdCannotBeZero);
		});
	}
}

pub mod vote_for_proposal {
	pub use super::*;

	#[test]
	fn vote_for_proposal_not_existing() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let recipient = RELAYER_C;
			let amount = 100;
			let deposit_nonce = Bridge::chain_nonces(CHAIN_ID).unwrap();

			let proposal = Bridge::get_votes(CHAIN_ID, (deposit_nonce, recipient, amount));
			assert!(proposal.is_none());

			let ok = Bridge::vote_for_proposal(
				origin(RELAYER_A),
				CHAIN_ID,
				deposit_nonce,
				recipient,
				amount,
				true,
			);
			assert_ok!(ok);

			let proposal = Bridge::get_votes(CHAIN_ID, (deposit_nonce, recipient, amount));
			assert!(proposal.is_some());
			let count = proposal.unwrap().votes.iter().filter(|x| x.1 == true).count() as u32;
			assert_eq!(count, 1);

			let event = BridgeEvent::RelayerVoted {
				chain_id: CHAIN_ID,
				nonce: deposit_nonce,
				account: RELAYER_A,
				in_favour: true,
			};
			let event = Event::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn vote_for_proposal_existing() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let recipient = RELAYER_C;
			let amount = 100;
			let deposit_nonce = Bridge::chain_nonces(CHAIN_ID).unwrap();

			let proposal = Bridge::get_votes(CHAIN_ID, (deposit_nonce, recipient, amount));
			assert!(proposal.is_none());

			let ok = Bridge::vote_for_proposal(
				origin(RELAYER_A),
				CHAIN_ID,
				deposit_nonce,
				recipient,
				amount,
				true,
			);
			assert_ok!(ok);

			let ok = Bridge::vote_for_proposal(
				origin(RELAYER_B),
				CHAIN_ID,
				deposit_nonce,
				recipient,
				amount,
				true,
			);
			assert_ok!(ok);

			let proposal = Bridge::get_votes(CHAIN_ID, (deposit_nonce, recipient, amount));
			assert!(proposal.is_some());
			let count = proposal.unwrap().votes.iter().filter(|x| x.1 == true).count() as u32;
			assert_eq!(count, 2);

			let event = BridgeEvent::RelayerVoted {
				chain_id: CHAIN_ID,
				nonce: deposit_nonce,
				account: RELAYER_B,
				in_favour: true,
			};
			let event = Event::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn vote_for_proposal_existing_and_reach_threshold() {
		let threshold = 2;

		ExtBuilder::default().build_with(CHAIN_ID, threshold).execute_with(|| {
			let recipient = RELAYER_C;
			let amount = 100;
			let deposit_nonce = Bridge::chain_nonces(CHAIN_ID).unwrap();
			let relayer_c_before = Balances::free_balance(RELAYER_C);
			let total_issuance = Balances::total_issuance();

			let proposal = Bridge::get_votes(CHAIN_ID, (deposit_nonce, recipient, amount));
			assert!(proposal.is_none());

			let ok = Bridge::vote_for_proposal(
				origin(RELAYER_A),
				CHAIN_ID,
				deposit_nonce,
				recipient,
				amount,
				true,
			);
			assert_ok!(ok);

			let ok = Bridge::vote_for_proposal(
				origin(RELAYER_B),
				CHAIN_ID,
				deposit_nonce,
				recipient,
				amount,
				true,
			);
			assert_ok!(ok);

			let proposal = Bridge::get_votes(CHAIN_ID, (deposit_nonce, recipient, amount));
			assert!(proposal.is_some());
			assert_eq!(proposal.unwrap().votes.iter().filter(|x| x.1 == true).count() as u32, 2);
			assert_eq!(relayer_c_before + amount, Balances::free_balance(RELAYER_C));
			assert_eq!(Balances::total_issuance(), total_issuance + amount);

			let event = BridgeEvent::ProposalApproved { chain_id: CHAIN_ID, nonce: deposit_nonce };
			let event = Event::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn vote_for_proposal_must_be_relayer() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				Bridge::vote_for_proposal(origin(5), 0, 0, RELAYER_C, 100, true),
				Error::<Test>::MustBeRelayer
			);
		});
	}

	#[test]
	fn vote_for_proposal_chain_not_whitelisted() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(
				Bridge::vote_for_proposal(origin(RELAYER_A), 1, 0, RELAYER_C, 100, true),
				Error::<Test>::ChainNotWhitelisted
			);
		});
	}

	#[test]
	fn vote_for_proposal_proposal_already_complete() {
		ExtBuilder::default().build_with(CHAIN_ID, 1).execute_with(|| {
			let ok =
				Bridge::vote_for_proposal(origin(RELAYER_A), CHAIN_ID, 0, RELAYER_C, 100, true);
			assert_ok!(ok);

			assert_noop!(
				Bridge::vote_for_proposal(origin(RELAYER_B), CHAIN_ID, 0, RELAYER_C, 100, true),
				Error::<Test>::ProposalAlreadyComplete
			);
		});
	}

	#[test]
	fn vote_for_proposal_proposal_expired() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let ok =
				Bridge::vote_for_proposal(origin(RELAYER_A), CHAIN_ID, 0, RELAYER_C, 100, true);
			assert_ok!(ok);

			System::set_block_number(
				frame_system::Pallet::<Test>::block_number() + ProposalLifetime::get() + 1,
			);

			assert_noop!(
				Bridge::vote_for_proposal(origin(RELAYER_B), CHAIN_ID, 0, RELAYER_C, 100, true),
				Error::<Test>::ProposalExpired
			);
		});
	}

	#[test]
	fn vote_for_proposal_relayer_already_voted() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let ok =
				Bridge::vote_for_proposal(origin(RELAYER_A), CHAIN_ID, 0, RELAYER_C, 100, true);
			assert_ok!(ok);

			assert_noop!(
				Bridge::vote_for_proposal(origin(RELAYER_A), CHAIN_ID, 0, RELAYER_C, 100, true),
				Error::<Test>::RelayerAlreadyVoted
			);
		});
	}
}

pub mod set_relayers {
	pub use super::*;

	#[test]
	fn set_relayers() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let raw_relayers: BoundedVec<u64, RelayerCountLimit> =
				bounded_vec![RELAYER_A, RELAYER_B];

			let ok = Bridge::set_relayers(root(), raw_relayers.clone());
			assert_ok!(ok);

			let relayers = Bridge::relayers();
			assert_eq!(relayers.clone(), raw_relayers);

			let event = BridgeEvent::RelayersUpdated { relayers };
			let event = Event::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(
				Bridge::set_relayers(origin(RELAYER_A), bounded_vec![RELAYER_A, RELAYER_B]),
				BadOrigin
			);
		});
	}
}

pub mod set_deposit_nonce {
	pub use super::*;

	#[test]
	fn set_deposit_nonce() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let new_nonce = 1;
			let ok = Bridge::set_deposit_nonce(root(), CHAIN_ID, new_nonce);
			assert_ok!(ok);

			// Check storage
			assert_eq!(Bridge::chain_nonces(CHAIN_ID), Some(new_nonce));

			// Check events
			let event = BridgeEvent::DepositNonceUpdated { chain_id: CHAIN_ID, nonce: new_nonce };
			let event = Event::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(Bridge::set_deposit_nonce(origin(RELAYER_A), 0, 1), BadOrigin);
		});
	}

	#[test]
	fn chain_not_found() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(Bridge::set_deposit_nonce(root(), 1, 1), Error::<Test>::ChainNotFound);
		});
	}

	#[test]
	fn new_nonce_too_low() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(
				Bridge::set_deposit_nonce(root(), CHAIN_ID, 0),
				Error::<Test>::NewNonceTooLow
			);
		});
	}
}

pub mod deposit {
	pub use super::*;

	#[test]
	fn deposit() {
		let chain_id = 0;
		let threshold = 3;

		ExtBuilder::default().build_with(chain_id, threshold).execute_with(|| {
			let amount = 10;
			let recipient = vec![0];
			let relayer_a_balance_before = Balances::free_balance(RELAYER_A);
			let bridge_fee = 100;
			let deposit_nonce = Bridge::chain_nonces(chain_id);
			let total_issuance = Balances::total_issuance();
			let collector_before = Balances::free_balance(COLLECTOR);

			let ok = Bridge::set_bridge_fee(root(), bridge_fee);
			assert_ok!(ok);

			let ok = Bridge::deposit(origin(RELAYER_A), amount, recipient.clone(), chain_id);
			assert_ok!(ok);

			assert_eq!(
				Balances::free_balance(RELAYER_A),
				relayer_a_balance_before - amount - bridge_fee
			);
			assert_eq!(Balances::total_issuance(), total_issuance - amount);
			assert_eq!(Balances::free_balance(COLLECTOR), collector_before + bridge_fee);
			let new_deposit_nonce = Bridge::chain_nonces(chain_id);
			assert_eq!(new_deposit_nonce.unwrap(), deposit_nonce.unwrap() + 1);

			let event = BridgeEvent::DepositMade {
				chain_id,
				nonce: new_deposit_nonce.unwrap(),
				amount: amount.into(),
				recipient,
			};
			let event = Event::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn deposit_chain_not_whitelisted() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				Bridge::deposit(origin(RELAYER_A), 10, vec![0], 0),
				Error::<Test>::ChainNotWhitelisted
			);
		});
	}

	#[test]
	fn deposit_removal_impossible() {
		let chain_id = 0;
		let threshold = 3;

		ExtBuilder::default().build_with(chain_id, threshold).execute_with(|| {
			assert_noop!(
				Bridge::deposit(origin(RELAYER_A), 200000000, vec![0], chain_id),
				Error::<Test>::InsufficientBalance
			);
		});
	}
}

pub mod set_bridge_fee {
	pub use super::*;

	#[test]
	fn set_bridge_fee() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			let ok = Bridge::set_bridge_fee(root(), 100);
			assert_ok!(ok);

			let fee = Bridge::bridge_fee();
			assert_eq!(fee.clone(), 100);

			let event = BridgeEvent::BridgeFeeUpdated { fee };
			let event = Event::Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		});
	}

	#[test]
	fn set_bridge_fee_bad_origin() {
		ExtBuilder::default().build_with(CHAIN_ID, THRESHOLD).execute_with(|| {
			assert_noop!(Bridge::set_bridge_fee(origin(RELAYER_A), 100), BadOrigin);
		});
	}
}