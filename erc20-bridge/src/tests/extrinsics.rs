use super::mock::*;

use crate::{
	tests::mock,
	tests::mock::helpers::{expect_event, assert_events, make_transfer_proposal},
	Error, Event as ERC20BridgeEvent
};
use frame_support::assert_ok;
use frame_system::RawOrigin;

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

mod transfer_native {
    use super::*;

	#[test]
	fn transfer_native() {
		TestExternalitiesBuilder::default().build().execute_with(|| {
			let origin = origin(RELAYER_A);
			let dest_chain = 0;
			let resource_id = NativeTokenId::get();
			let amount: u64 = 100;
			let recipient = vec![99];
            let bridge_fee = 3;

			assert_ok!(ChainBridge::whitelist_chain(Origin::root(), dest_chain.clone()));

            assert_ok!(ERC20Bridge::set_bridge_fee(root(), bridge_fee));

			let origin_balance_before = Balances::free_balance(RELAYER_A);
			let total_issuance_before = Balances::total_issuance();
            let treasury_before = Balances::free_balance(COLLECTOR);

			assert_ok!(ERC20Bridge::transfer_native(
				origin.clone(),
				amount.clone(),
				recipient.clone(),
				dest_chain,
			));

			assert_eq!(Balances::free_balance(RELAYER_A), origin_balance_before - amount - bridge_fee);
			assert_eq!(Balances::total_issuance(), total_issuance_before - amount);
            assert_eq!(Balances::free_balance(COLLECTOR), treasury_before + bridge_fee);

			expect_event(chainbridge::Event::FungibleTransfer(
				dest_chain,
				1,
				resource_id,
				amount.into(),
				recipient,
			));
		})
	}
}

mod transfer {
	use super::*;
	#[test]
	fn transfer() {
		TestExternalitiesBuilder::default().build().execute_with(|| {
			let amount = 10;
			let relayer_a_balance_before = Balances::free_balance(RELAYER_A);
			let total_issuance_before = Balances::total_issuance();

			assert_ok!(ERC20Bridge::transfer(
				Origin::signed(ChainBridge::account_id()),
				RELAYER_A,
				amount,
			));

			assert_eq!(Balances::free_balance(RELAYER_A), relayer_a_balance_before + 10);
			assert_eq!(Balances::total_issuance(), total_issuance_before + amount);

			assert_events(vec![mock::Event::Balances(pallet_balances::Event::Deposit {
				who: RELAYER_A,
				amount,
			})]);
		})
	}

	#[test]
	fn create_sucessful_transfer_proposal() {
		TestExternalitiesBuilder::default().build().execute_with(|| {
			let prop_id = 1;
			let src_id = 1;
			let r_id = chainbridge::derive_resource_id(src_id, b"transfer");
			let resource = b"ERC20Bridge.transfer".to_vec();
			let proposal = make_transfer_proposal(RELAYER_A, 10);

			assert_ok!(ChainBridge::set_threshold(Origin::root(), TEST_RELAYER_VOTE_THRESHOLD));
			assert_ok!(ChainBridge::add_relayer(Origin::root(), RELAYER_A));
			assert_ok!(ChainBridge::add_relayer(Origin::root(), RELAYER_B));
			assert_ok!(ChainBridge::add_relayer(Origin::root(), RELAYER_C));
			assert_ok!(ChainBridge::whitelist_chain(Origin::root(), src_id));
			assert_ok!(ChainBridge::set_resource(Origin::root(), r_id, resource));

			// Create proposal (& vote)
			assert_ok!(ChainBridge::acknowledge_proposal(
				Origin::signed(RELAYER_A),
				prop_id,
				src_id,
				r_id,
				Box::new(proposal.clone())
			));
			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
			let expected = chainbridge::types::ProposalVotes {
				votes_for: vec![RELAYER_A],
				votes_against: vec![],
				status: chainbridge::types::ProposalStatus::Initiated,
				expiry: ProposalLifetime::get() + 1,
			};
			assert_eq!(prop, expected);

			// Second relayer votes against
			assert_ok!(ChainBridge::reject_proposal(
				Origin::signed(RELAYER_B),
				prop_id,
				src_id,
				r_id,
				Box::new(proposal.clone())
			));
			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
			let expected = chainbridge::types::ProposalVotes {
				votes_for: vec![RELAYER_A],
				votes_against: vec![RELAYER_B],
				status: chainbridge::types::ProposalStatus::Initiated,
				expiry: ProposalLifetime::get() + 1,
			};
			assert_eq!(prop, expected);

			let total_issuance_before = Balances::total_issuance();

			// Third relayer votes in favour
			assert_ok!(ChainBridge::acknowledge_proposal(
				Origin::signed(RELAYER_C),
				prop_id,
				src_id,
				r_id,
				Box::new(proposal.clone())
			));
			let prop = ChainBridge::get_votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
			let expected = chainbridge::types::ProposalVotes {
				votes_for: vec![RELAYER_A, RELAYER_C],
				votes_against: vec![RELAYER_B],
				status: chainbridge::types::ProposalStatus::Approved,
				expiry: ProposalLifetime::get() + 1,
			};
			assert_eq!(prop, expected);

			assert_eq!(Balances::free_balance(RELAYER_A), ENDOWED_BALANCE + 10);
			assert_eq!(Balances::total_issuance(), total_issuance_before + 10);

			assert_events(vec![
				mock::Event::ChainBridge(chainbridge::Event::VoteFor(src_id, prop_id, RELAYER_A)),
				mock::Event::ChainBridge(chainbridge::Event::VoteAgainst(
					src_id, prop_id, RELAYER_B,
				)),
				mock::Event::ChainBridge(chainbridge::Event::VoteFor(src_id, prop_id, RELAYER_C)),
				mock::Event::ChainBridge(chainbridge::Event::ProposalApproved(src_id, prop_id)),
				mock::Event::Balances(pallet_balances::Event::Deposit {
					who: RELAYER_A,
					amount: 10,
				}),
				mock::Event::ChainBridge(chainbridge::Event::ProposalSucceeded(src_id, prop_id)),
			]);
		})
	}
}

mod set_bridge_fee {
	use super::*;

	#[test]
	fn set_bridge_fee() {
		TestExternalitiesBuilder::default().build().execute_with(|| {
			let old_bridge_fee = ERC20Bridge::bridge_fee();
			let new_bridge_fee = 3u64;
			assert!(old_bridge_fee != new_bridge_fee);
	
			// Change the bridge fee
			let ok = ERC20Bridge::set_bridge_fee(root(), new_bridge_fee);
			assert_ok!(ok);
	
			// Final state checks
			assert_eq!(ERC20Bridge::bridge_fee(), new_bridge_fee);
	
			// Events checks
			let event = ERC20BridgeEvent::BridgeFeeUpdated { fee: new_bridge_fee };
			let event = Event::ERC20Bridge(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}
}