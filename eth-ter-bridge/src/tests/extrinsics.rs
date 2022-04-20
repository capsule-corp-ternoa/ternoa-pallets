// Copyright 2021 Centrifuge Foundation (centrifuge.io).
// This file is part of Centrifuge chain project.

// Centrifuge is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version (see http://www.gnu.org/licenses).

// Centrifuge is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

//! Unit test cases for the Substrate/Ethereum chains bridging pallet.

// ----------------------------------------------------------------------------
// Module imports and re-exports
// ----------------------------------------------------------------------------

use super::mock::{self, *};
use frame_support::{assert_noop, assert_ok, bounded_vec, error::BadOrigin};
use frame_system::RawOrigin;

use crate::{
	types::{Proposal, ProposalStatus},
	Error, Event as ChainBridgeEvent,
};

use crate::tests::mock::{
	ChainBridge, ProposalLifetime, System, TestExternalitiesBuilder, RELAYER_A, RELAYER_B,
};

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

pub mod try_to_complete {
	pub use super::*;

	#[test]
	fn try_to_complete_approved() {
		TestExternalitiesBuilder::default().build().execute_with(|| {
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
		TestExternalitiesBuilder::default().build().execute_with(|| {
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
		TestExternalitiesBuilder::default().build().execute_with(|| {
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

pub mod whitelist_chain {
	pub use super::*;

	#[test]
	fn whitelist_chain() {
		TestExternalitiesBuilder::default().build().execute_with(|| {
			assert!(!ChainBridge::chain_whitelisted(0));

			assert_ok!(ChainBridge::whitelist_chain(root(), 0));

			let event = ChainBridgeEvent::ChainWhitelisted { chain_id: 0 };
			let event = Event::ChainBridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn whitelist_chain_bad_origin() {
		TestExternalitiesBuilder::default().build().execute_with(|| {
			assert!(!ChainBridge::chain_whitelisted(0));

			assert_noop!(ChainBridge::whitelist_chain(origin(RELAYER_A), 0), BadOrigin);
		});
	}

	#[test]
	fn whitelist_chain_invalid_chain_id() {
		TestExternalitiesBuilder::default().build().execute_with(|| {
			assert!(!ChainBridge::chain_whitelisted(0));

			assert_noop!(
				ChainBridge::whitelist_chain(root(), MockChainId::get()),
				Error::<MockRuntime>::InvalidChainId
			);
		});
	}

	#[test]
	fn whitelist_chain_chain_already_whitelisted() {
		TestExternalitiesBuilder::default().build().execute_with(|| {
			assert!(!ChainBridge::chain_whitelisted(0));

			assert_ok!(ChainBridge::whitelist_chain(root(), 0));
			assert_noop!(
				ChainBridge::whitelist_chain(root(), 0),
				Error::<MockRuntime>::ChainAlreadyWhitelisted
			);
		});
	}
}

pub mod set_threshold {
	pub use super::*;

	// #[test]
	// fn set_get_threshold() {
	// 	TestExternalitiesBuilder::default().build().execute_with(|| {
	// 		assert_eq!(ChainBridge::get_threshold(), DEFAULT_RELAYER_VOTE_THRESHOLD);

	// 		assert_ok!(ChainBridge::set_threshold(root(), TEST_RELAYER_VOTE_THRESHOLD));
	// 		assert_eq!(ChainBridge::get_threshold(), TEST_RELAYER_VOTE_THRESHOLD);

	// 		assert_ok!(ChainBridge::set_threshold(root(), 5));
	// 		assert_eq!(ChainBridge::get_threshold(), 5);

	// 		assert_events(vec![
	// 			Event::ChainBridge(pallet::Event::<MockRuntime>::RelayerThresholdChanged(
	// 				TEST_RELAYER_VOTE_THRESHOLD,
	// 			)),
	// 			Event::ChainBridge(pallet::Event::<MockRuntime>::RelayerThresholdChanged(5)),
	// 		]);
	// 	})
	// }
}

// #[test]
// fn asset_transfer_success() {
// 	TestExternalitiesBuilder::default().build().execute_with(|| {
// 		let dest_id = 2;
// 		let to = vec![2];
// 		let resource_id = [1; 32];
// 		let amount = 100;

// 		assert_ok!(ChainBridge::set_threshold(root(), TEST_RELAYER_VOTE_THRESHOLD));

// 		assert_ok!(ChainBridge::whitelist_chain(root(), dest_id.clone()));
// 		assert_ok!(ChainBridge::transfer_fungible(
// 			dest_id.clone(),
// 			resource_id.clone(),
// 			to.clone(),
// 			amount.into()
// 		));

// 		assert_events(vec![
// 			Event::ChainBridge(pallet::Event::<MockRuntime>::ChainWhitelisted(dest_id.clone())),
// 			Event::ChainBridge(pallet::Event::<MockRuntime>::FungibleTransfer(
// 				dest_id.clone(),
// 				1,
// 				resource_id.clone(),
// 				amount.into(),
// 				to.clone(),
// 			)),
// 		]);
// 	})
// }

// #[test]
// fn asset_transfer_invalid_chain() {
// 	TestExternalitiesBuilder::default().build().execute_with(|| {
// 		let chain_id = 2;
// 		let bad_dest_id = 3;
// 		let resource_id = [4; 32];

// 		assert_ok!(ChainBridge::whitelist_chain(root(), chain_id.clone()));
// 		assert_events(vec![Event::ChainBridge(pallet::Event::<MockRuntime>::ChainWhitelisted(
// 			chain_id.clone(),
// 		))]);

// 		assert_noop!(
// 			ChainBridge::transfer_fungible(bad_dest_id, resource_id.clone(), vec![], U256::zero()),
// 			Error::<MockRuntime>::ChainNotWhitelisted
// 		);
// 	})
// }

// #[test]
// fn add_remove_relayer() {
// 	TestExternalitiesBuilder::default().build().execute_with(|| {
// 		assert_ok!(ChainBridge::set_threshold(root(), TEST_RELAYER_VOTE_THRESHOLD,));
// 		assert_eq!(ChainBridge::get_relayer_count(), 0);

// 		assert_ok!(ChainBridge::add_relayer(root(), RELAYER_A));
// 		assert_ok!(ChainBridge::add_relayer(root(), RELAYER_B));
// 		assert_ok!(ChainBridge::add_relayer(root(), RELAYER_C));
// 		assert_eq!(ChainBridge::get_relayer_count(), 3);

// 		// Already exists
// 		assert_noop!(
// 			ChainBridge::add_relayer(root(), RELAYER_A),
// 			Error::<MockRuntime>::RelayerAlreadyExists
// 		);

// 		// Confirm removal
// 		assert_ok!(ChainBridge::remove_relayer(root(), RELAYER_B));
// 		assert_eq!(ChainBridge::get_relayer_count(), 2);
// 		assert_noop!(
// 			ChainBridge::remove_relayer(root(), RELAYER_B),
// 			Error::<MockRuntime>::RelayerInvalid
// 		);
// 		assert_eq!(ChainBridge::get_relayer_count(), 2);
// 		assert_events(vec![
// 			Event::ChainBridge(pallet::Event::<MockRuntime>::RelayerAdded(RELAYER_A)),
// 			Event::ChainBridge(pallet::Event::<MockRuntime>::RelayerAdded(RELAYER_B)),
// 			Event::ChainBridge(pallet::Event::<MockRuntime>::RelayerAdded(RELAYER_C)),
// 			Event::ChainBridge(pallet::Event::<MockRuntime>::RelayerRemoved(RELAYER_B)),
// 		]);
// 	})
// }

// #[test]
// fn create_successful_remark_proposal() {
// 	let src_id: ChainId = 1;
// 	let r_id = derive_resource_id(src_id, b"remark");

// 	TestExternalitiesBuilder::default()
// 		.build_with(src_id, r_id, b"System.remark".to_vec())
// 		.execute_with(|| {
// 			let prop_id = 1;

// 			// Create a dummy system remark proposal
// 			let proposal = Call::System(SystemCall::remark { remark: vec![10] });

// 			// Create proposal (& vote)
// 			assert_ok!(ChainBridge::acknowledge_proposal(
// 				Origin::signed(RELAYER_A),
// 				prop_id,
// 				src_id,
// 				r_id,
// 				Box::new(proposal.clone())
// 			));

// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();

// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A],
// 				votes_against: vec![],
// 				status: ProposalStatus::Initiated,
// 				expiry: ProposalLifetime::get() + 1,
// 			};
// 			assert_eq!(prop, expected);

// 			// Second relayer votes against
// 			assert_ok!(ChainBridge::reject_proposal(
// 				Origin::signed(RELAYER_B),
// 				prop_id,
// 				src_id,
// 				r_id,
// 				Box::new(proposal.clone())
// 			));

// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();

// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A],
// 				votes_against: vec![RELAYER_B],
// 				status: ProposalStatus::Initiated,
// 				expiry: ProposalLifetime::get() + 1,
// 			};
// 			assert_eq!(prop, expected);

// 			// Third relayer votes in favour
// 			assert_ok!(ChainBridge::acknowledge_proposal(
// 				Origin::signed(RELAYER_C),
// 				prop_id,
// 				src_id,
// 				r_id,
// 				Box::new(proposal.clone())
// 			));

// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A, RELAYER_C],
// 				votes_against: vec![RELAYER_B],
// 				status: ProposalStatus::Approved,
// 				expiry: ProposalLifetime::get() + 1,
// 			};

// 			assert_eq!(prop, expected);

// 			assert_events(vec![
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::VoteFor(
// 					src_id, prop_id, RELAYER_A,
// 				)),
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::VoteAgainst(
// 					src_id, prop_id, RELAYER_B,
// 				)),
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::VoteFor(
// 					src_id, prop_id, RELAYER_C,
// 				)),
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::ProposalApproved(src_id, prop_id)),
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::ProposalSucceeded(
// 					src_id, prop_id,
// 				)),
// 			]);
// 		})
// }

// #[test]
// fn create_unsuccessful_transfer_proposal() {
// 	let src_id = 1;
// 	let r_id = derive_resource_id(src_id, b"transfer");

// 	TestExternalitiesBuilder::default()
// 		.build_with(src_id, r_id, b"System.remark".to_vec())
// 		.execute_with(|| {
// 			let prop_id = 1;

// 			// Create a dummy system remark proposal
// 			let proposal = Call::System(SystemCall::remark { remark: vec![11] });

// 			// Create proposal (& vote)
// 			assert_ok!(ChainBridge::acknowledge_proposal(
// 				Origin::signed(RELAYER_A),
// 				prop_id,
// 				src_id,
// 				r_id,
// 				Box::new(proposal.clone())
// 			));
// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A],
// 				votes_against: vec![],
// 				status: ProposalStatus::Initiated,
// 				expiry: ProposalLifetime::get() + 1,
// 			};
// 			assert_eq!(prop, expected);

// 			// Second relayer votes against
// 			assert_ok!(ChainBridge::reject_proposal(
// 				Origin::signed(RELAYER_B),
// 				prop_id,
// 				src_id,
// 				r_id,
// 				Box::new(proposal.clone())
// 			));
// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A],
// 				votes_against: vec![RELAYER_B],
// 				status: ProposalStatus::Initiated,
// 				expiry: ProposalLifetime::get() + 1,
// 			};
// 			assert_eq!(prop, expected);

// 			// Third relayer votes against
// 			assert_ok!(ChainBridge::reject_proposal(
// 				Origin::signed(RELAYER_C),
// 				prop_id,
// 				src_id,
// 				r_id,
// 				Box::new(proposal.clone())
// 			));
// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A],
// 				votes_against: vec![RELAYER_B, RELAYER_C],
// 				status: ProposalStatus::Rejected,
// 				expiry: ProposalLifetime::get() + 1,
// 			};
// 			assert_eq!(prop, expected);

// 			assert_eq!(Balances::free_balance(RELAYER_B), 0);
// 			assert_eq!(Balances::free_balance(ChainBridge::account_id()), ENDOWED_BALANCE);

// 			assert_events(vec![
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::VoteFor(
// 					src_id, prop_id, RELAYER_A,
// 				)),
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::VoteAgainst(
// 					src_id, prop_id, RELAYER_B,
// 				)),
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::VoteAgainst(
// 					src_id, prop_id, RELAYER_C,
// 				)),
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::ProposalRejected(src_id, prop_id)),
// 			]);
// 		})
// }

// #[test]
// fn execute_after_threshold_change() {
// 	let src_id = 1;
// 	let r_id = derive_resource_id(src_id, b"transfer");

// 	TestExternalitiesBuilder::default()
// 		.build_with(src_id, r_id, b"System.remark".to_vec())
// 		.execute_with(|| {
// 			let prop_id = 1;

// 			// Create a dummy system remark proposal
// 			let proposal = Call::System(SystemCall::remark { remark: vec![11] });

// 			// Create proposal (& vote)
// 			assert_ok!(ChainBridge::acknowledge_proposal(
// 				Origin::signed(RELAYER_A),
// 				prop_id,
// 				src_id,
// 				r_id,
// 				Box::new(proposal.clone())
// 			));
// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A],
// 				votes_against: vec![],
// 				status: ProposalStatus::Initiated,
// 				expiry: ProposalLifetime::get() + 1,
// 			};
// 			assert_eq!(prop, expected);

// 			// Change threshold
// 			assert_ok!(ChainBridge::set_threshold(root(), 1));

// 			// Attempt to execute
// 			assert_ok!(ChainBridge::eval_vote_state(
// 				Origin::signed(RELAYER_A),
// 				prop_id,
// 				src_id,
// 				Box::new(proposal.clone())
// 			));

// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A],
// 				votes_against: vec![],
// 				status: ProposalStatus::Approved,
// 				expiry: ProposalLifetime::get() + 1,
// 			};
// 			assert_eq!(prop, expected);

// 			assert_eq!(Balances::free_balance(RELAYER_B), 0);
// 			assert_eq!(Balances::free_balance(ChainBridge::account_id()), ENDOWED_BALANCE);

// 			assert_events(vec![
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::VoteFor(
// 					src_id, prop_id, RELAYER_A,
// 				)),
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::RelayerThresholdChanged(1)),
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::ProposalApproved(src_id, prop_id)),
// 				Event::ChainBridge(pallet::Event::<MockRuntime>::ProposalSucceeded(
// 					src_id, prop_id,
// 				)),
// 			]);
// 		})
// }

// #[test]
// fn proposal_expires() {
// 	let src_id = 1;
// 	let r_id = derive_resource_id(src_id, b"remark");

// 	TestExternalitiesBuilder::default()
// 		.build_with(src_id, r_id, b"System.remark".to_vec())
// 		.execute_with(|| {
// 			let prop_id = 1;

// 			// Create a dummy system remark proposal
// 			let proposal = Call::System(SystemCall::remark { remark: vec![10] });

// 			// Create proposal (& vote)
// 			assert_ok!(ChainBridge::acknowledge_proposal(
// 				Origin::signed(RELAYER_A),
// 				prop_id,
// 				src_id,
// 				r_id,
// 				Box::new(proposal.clone())
// 			));
// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A],
// 				votes_against: vec![],
// 				status: ProposalStatus::Initiated,
// 				expiry: ProposalLifetime::get() + 1,
// 			};
// 			assert_eq!(prop, expected);

// 			// Increment enough blocks such that now == expiry
// 			System::set_block_number(ProposalLifetime::get() + 1);

// 			// Attempt to submit a vote should fail
// 			assert_noop!(
// 				ChainBridge::reject_proposal(
// 					Origin::signed(RELAYER_B),
// 					prop_id,
// 					src_id,
// 					r_id,
// 					Box::new(proposal.clone())
// 				),
// 				Error::<MockRuntime>::ProposalExpired
// 			);

// 			// Proposal state should remain unchanged
// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A],
// 				votes_against: vec![],
// 				status: ProposalStatus::Initiated,
// 				expiry: ProposalLifetime::get() + 1,
// 			};
// 			assert_eq!(prop, expected);

// 			// eval_vote_state should have no effect
// 			assert_noop!(
// 				ChainBridge::eval_vote_state(
// 					Origin::signed(RELAYER_C),
// 					prop_id,
// 					src_id,
// 					Box::new(proposal.clone())
// 				),
// 				Error::<MockRuntime>::ProposalExpired
// 			);
// 			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
// 			let expected = Proposal {
// 				votes_for: vec![RELAYER_A],
// 				votes_against: vec![],
// 				status: ProposalStatus::Initiated,
// 				expiry: ProposalLifetime::get() + 1,
// 			};
// 			assert_eq!(prop, expected);

// 			assert_events(vec![mock::Event::ChainBridge(pallet::Event::<MockRuntime>::VoteFor(
// 				src_id, prop_id, RELAYER_A,
// 			))]);
// 		})
// }
