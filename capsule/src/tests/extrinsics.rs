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

use super::mock::*;
use frame_support::{assert_noop, assert_ok, bounded_vec, error::BadOrigin, BoundedVec};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use ternoa_common::traits::NFTExt;

use crate::{tests::mock, CapsuleData, CapsuleIPFSReference, Error, Event as CapsuleEvent};

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

pub mod create {
	pub use super::*;

	#[test]
	fn create() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let alice_balance_before = Balances::free_balance(ALICE);
			let capsule_mint_fee = Capsule::capsule_mint_fee();
			let nft_fee = NFT::nft_mint_fee();
			let nft_id = 0;
			let ipfs_reference: CapsuleIPFSReference<Test> = bounded_vec![60];

			let ok = Capsule::create(alice, bounded_vec![50], ipfs_reference.clone(), None);
			assert_ok!(ok);

			// Storage
			assert_eq!(Capsule::capsules(&nft_id), Some(CapsuleData::new(ALICE, ipfs_reference)));
			assert_eq!(Capsule::ledgers(&ALICE), Some(bounded_vec![(nft_id, capsule_mint_fee)]));
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance_before - capsule_mint_fee - nft_fee
			);
			assert_eq!(Balances::free_balance(Capsule::account_id()), capsule_mint_fee);

			// Event
			let event = CapsuleEvent::CapsuleCreated {
				owner: ALICE,
				nft_id,
				frozen_balance: capsule_mint_fee,
			};
			let event = Event::Capsule(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(BOB, 100)]).execute_with(|| {
			let ok = Capsule::create(origin(BOB), bounded_vec![], bounded_vec![1], None);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

pub mod create_from_nft {
	pub use super::*;

	#[test]
	fn create_from_nft() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let alice_balance_before = Balances::free_balance(ALICE);
			let capsule_mint_fee = Capsule::capsule_mint_fee();
			let nft_fee = NFT::nft_mint_fee();
			let nft_id = help::create_nft_fast(alice.clone());
			let pallet_id = Capsule::account_id();
			let ipfs_reference: BoundedVec<u8, IPFSLengthLimit> = bounded_vec![60];

			let ok = Capsule::create_from_nft(alice, nft_id, ipfs_reference.clone());
			assert_ok!(ok);

			// Storage
			assert_eq!(Capsule::capsules(&nft_id), Some(CapsuleData::new(ALICE, ipfs_reference)));
			assert_eq!(Capsule::ledgers(&ALICE), Some(bounded_vec![(nft_id, capsule_mint_fee)]));
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance_before - capsule_mint_fee - nft_fee
			);
			assert_eq!(Balances::free_balance(pallet_id), capsule_mint_fee);

			// Event
			let event = CapsuleEvent::CapsuleCreated {
				owner: ALICE,
				nft_id,
				frozen_balance: capsule_mint_fee,
			};
			let event = Event::Capsule(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 10000), (BOB, 10000)]).execute_with(|| {
			let nft_id = help::create_nft_fast(origin(BOB));

			let ok = Capsule::create_from_nft(origin(ALICE), nft_id, bounded_vec![25]);
			assert_noop!(ok, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_create_capsules_from_nft_listed_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id = help::create_nft_fast(origin(ALICE));
			let ok = <NFT as NFTExt>::set_listed_for_sale(nft_id, true);
			assert_ok!(ok);

			let ok = Capsule::create_from_nft(alice, nft_id, bounded_vec![25]);
			assert_noop!(ok, Error::<Test>::CannotCreateCapsulesFromNFTsListedForSale);
		})
	}

	#[test]
	fn cannot_create_capsules_from_nft_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id = help::create_nft_fast(alice.clone());
			let ok = <NFT as NFTExt>::set_in_transmission(nft_id, true);
			assert_ok!(ok);

			let ok = Capsule::create_from_nft(alice, nft_id, bounded_vec![25]);
			assert_noop!(ok, Error::<Test>::CannotCreateCapsulesFromNFTsInTransmission);
		})
	}

	#[test]
	fn cannot_create_capsules_from_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id = help::create_nft_fast(alice.clone());
			let ok = Capsule::create_from_nft(alice.clone(), nft_id, bounded_vec![25]);
			assert_ok!(ok);

			let ok = Capsule::create_from_nft(alice, nft_id, bounded_vec![30]);
			assert_noop!(ok, Error::<Test>::CannotCreateCapsulesFromCapsules);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(BOB, 100)]).execute_with(|| {
			let bob: mock::Origin = origin(BOB);
			let nft_id = help::create_nft_fast(bob.clone());

			let ok = Capsule::create_from_nft(bob, nft_id, bounded_vec![30]);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

pub mod remove {
	pub use super::*;

	#[test]
	fn remove() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id = help::create_capsule_fast(alice.clone());
			let capsule_mint_fee = Capsule::capsule_mint_fee();
			let pallet_id = Capsule::account_id();
			let pallet_balance_before = Balances::free_balance(pallet_id);
			let alice_balance_before = Balances::free_balance(ALICE);

			let ok = Capsule::remove(alice, nft_id);
			assert_ok!(ok);

			// Storage
			assert_eq!(Capsule::capsules(&nft_id), None);
			assert_eq!(Capsule::ledgers(&ALICE), None);
			assert_eq!(Balances::free_balance(ALICE), alice_balance_before + capsule_mint_fee);
			assert_eq!(Balances::free_balance(pallet_id), pallet_balance_before - capsule_mint_fee);

			// Event
			let event = CapsuleEvent::CapsuleRemoved { nft_id, unfrozen_balance: capsule_mint_fee };
			let event = Event::Capsule(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 10000), (BOB, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let bob_nft_id = help::create_capsule_fast(origin(BOB));

			let ok = Capsule::remove(alice, bob_nft_id);
			assert_noop!(ok, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let alice_nft_id = help::create_capsule_fast(alice.clone());
			let ok = Balances::set_balance(root(), Capsule::account_id(), 0, 0);
			assert_ok!(ok);

			let ok = Capsule::remove(alice, alice_nft_id);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

pub mod add_funds {
	pub use super::*;

	#[test]
	fn add_funds() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id = help::create_capsule_fast(alice.clone());
			let capsule_mint_fee = Capsule::capsule_mint_fee();
			let add = 55;
			let pallet_id = Capsule::account_id();
			let alice_balance_before = Balances::free_balance(ALICE);
			let pallet_balance_before = Balances::free_balance(pallet_id);

			let ok = Capsule::add_funds(alice.clone(), nft_id, add);
			assert_ok!(ok);

			// Storage
			assert_eq!(
				Capsule::ledgers(&ALICE),
				Some(bounded_vec![(nft_id, capsule_mint_fee + add)])
			);
			assert_eq!(Balances::free_balance(ALICE), alice_balance_before - add);
			assert_eq!(Balances::free_balance(pallet_id), pallet_balance_before + add);

			// Event
			let event = CapsuleEvent::CapsuleFundsAdded { nft_id, balance: add };
			let event = Event::Capsule(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 10000), (BOB, 10000)]).execute_with(|| {
			let bob_nft_id = help::create_capsule_fast(origin(BOB));

			let ok = Capsule::add_funds(origin(ALICE), bob_nft_id, 10000000);
			assert_noop!(ok, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let alice_nft_id = help::create_capsule_fast(alice.clone());

			let ok = Capsule::add_funds(alice, alice_nft_id, 10000000);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

pub mod set_ipfs_reference {
	pub use super::*;

	#[test]
	fn set_ipfs_reference() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id = help::create_capsule_fast(alice.clone());
			let data = Capsule::capsules(nft_id).unwrap();
			let old_reference = data.ipfs_reference.clone();
			let new_reference: CapsuleIPFSReference<Test> = bounded_vec![67];
			assert_ne!(old_reference, new_reference);

			let ok = Capsule::set_ipfs_reference(alice, nft_id, new_reference.clone());
			assert_ok!(ok);

			// Storage
			assert_eq!(Capsule::capsules(nft_id).unwrap().ipfs_reference, new_reference);

			// Event
			let event =
				CapsuleEvent::CapsuleIpfsReferenceUpdated { nft_id, ipfs_reference: new_reference };
			let event = Event::Capsule(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 10000), (BOB, 10000)]).execute_with(|| {
			let bob_nft_id = help::create_capsule_fast(origin(BOB));

			let ok = Capsule::set_ipfs_reference(origin(ALICE), bob_nft_id, bounded_vec![1]);
			assert_noop!(ok, Error::<Test>::NotTheNFTOwner);
		})
	}
}

pub mod set_capsule_mint_fee {
	pub use super::*;

	#[test]
	fn set_capsule_mint_fee() {
		ExtBuilder::new_build(vec![]).execute_with(|| {
			let new_mint_fee = 654u128;
			assert_ne!(Capsule::capsule_mint_fee(), new_mint_fee);

			let ok = Capsule::set_capsule_mint_fee(root(), new_mint_fee);
			assert_ok!(ok);

			// Storage
			assert_eq!(Capsule::capsule_mint_fee(), new_mint_fee);

			// Event
			let event = CapsuleEvent::CapsuleMintFeeUpdated { fee: new_mint_fee };
			let event = Event::Capsule(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let ok = Capsule::set_capsule_mint_fee(origin(ALICE), 654);
			assert_noop!(ok, BadOrigin);
		})
	}
}
