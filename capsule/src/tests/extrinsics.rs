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
			let nft_id = 0;
			let capsule_mint_fee = Capsule::capsule_mint_fee();
			let nft_fee = NFT::nft_mint_fee();
			let alice_balance_before = Balances::free_balance(ALICE);

			let ok = Capsule::create(origin(ALICE), bounded_vec![50], bounded_vec![60], None);
			assert_ok!(ok);

			// Storage
			assert_eq!(Capsule::capsules(&nft_id), Some(CapsuleData::new(ALICE, bounded_vec![60])));
			assert_eq!(
				Capsule::ledgers(&ALICE),
				Some(bounded_vec![(nft_id, Capsule::capsule_mint_fee())])
			);
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
	fn create_transactional() {
		ExtBuilder::new_build(vec![(ALICE, 10002), (BOB, 10002)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let balance = Balances::free_balance(ALICE);
			let capsule_fee = Capsule::capsule_mint_fee();
			let nft_fee = NFT::nft_mint_fee();
			let pallet_id = Capsule::account_id();

			// Lets make sure that Alice has enough funds
			assert!(balance > (capsule_fee + nft_fee));

			let series_id = Some("AAA".into());
			let ok = NFT::create_nft(BOB, bounded_vec![], series_id.clone());
			assert_ok!(ok);

			// Trigger an error
			let ok = Capsule::create(alice.clone(), bounded_vec![], bounded_vec![], series_id);
			assert_noop!(ok, ternoa_nft::Error::<Test>::NotTheSeriesOwner);

			// She should not have lost any caps
			assert_eq!(Balances::free_balance(ALICE), balance);
			assert_eq!(Balances::free_balance(pallet_id), 0);
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
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = help::create_nft_fast(alice.clone());
			let ipfs_reference: BoundedVec<u8, IPFSLengthLimit> = bounded_vec![60];
			assert_eq!(Capsule::capsules(&nft_id), None);
			assert_eq!(Capsule::ledgers(&ALICE), None);

			// Happy path
			let data = CapsuleData::new(ALICE, ipfs_reference.clone());
			let ledger = bounded_vec![(nft_id, Capsule::capsule_mint_fee())];

			let ok = Capsule::create_from_nft(alice.clone(), nft_id, ipfs_reference);
			assert_ok!(ok);
			assert_eq!(Capsule::capsules(&nft_id), Some(data));
			assert_eq!(Capsule::ledgers(&ALICE), Some(ledger));
		})
	}

	#[test]
	fn create_from_nft_caps_transfer() {
		ExtBuilder::new_build(vec![(ALICE, 10001)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let capsule_fee = Capsule::capsule_mint_fee();
			let pallet_id = Capsule::account_id();
			assert_ne!(capsule_fee, 0);
			assert_eq!(Balances::free_balance(pallet_id), 0);

			// Funds are transferred
			let nft_id = help::create_nft_fast(alice.clone());
			let balance = Balances::free_balance(ALICE);
			let ok = Capsule::create_from_nft(alice.clone(), nft_id, bounded_vec![50]);
			assert_ok!(ok);
			assert_eq!(Balances::free_balance(ALICE), balance - capsule_fee);
			assert_eq!(Balances::free_balance(pallet_id), capsule_fee);
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
			let nft_id = help::create_nft_fast(alice.clone());
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
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id_1 = help::create_capsule_fast(alice.clone());
			let nft_id_2 = help::create_capsule_fast(alice.clone());
			let ledger = bounded_vec![(nft_id_2, Capsule::capsule_mint_fee())];

			// Happy path delete one nft id associated with that owner
			let ok = Capsule::remove(alice.clone(), nft_id_1);
			assert_ok!(ok);
			assert_eq!(Capsule::capsules(&nft_id_1), None);
			assert_eq!(Capsule::ledgers(&ALICE), Some(ledger));

			// Happy path delete last nft id associated with that owner
			let ok = Capsule::remove(alice.clone(), nft_id_2);
			assert_ok!(ok);
			assert_eq!(Capsule::capsules(&nft_id_2), None);
			assert_eq!(Capsule::ledgers(&ALICE), None);
		})
	}

	#[test]
	fn remove_caps_transfer() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);

			let nft_id = help::create_capsule_fast(alice.clone());
			let fee = Capsule::ledgers(ALICE).unwrap()[0].1;
			let pallet_id = Capsule::account_id();

			let pallet_balance = Balances::free_balance(pallet_id);
			let alice_balance = Balances::free_balance(ALICE);

			// Funds are transferred
			let ok = Capsule::remove(alice.clone(), nft_id);
			assert_ok!(ok);
			assert_eq!(Balances::free_balance(ALICE), alice_balance + fee);
			assert_eq!(Balances::free_balance(pallet_id), pallet_balance - fee);
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
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = help::create_capsule_fast(alice.clone());
			let fee = Capsule::capsule_mint_fee();
			let ledger = bounded_vec![(nft_id, fee)];
			assert_eq!(Capsule::ledgers(&ALICE), Some(ledger));

			// Happy path
			let add = 55;
			let ledger = bounded_vec![(nft_id, fee + add)];
			let ok = Capsule::add_funds(alice.clone(), nft_id, add);
			assert_ok!(ok);
			assert_eq!(Capsule::ledgers(&ALICE), Some(ledger));
		})
	}

	#[test]
	fn add_funds_caps_transfer() {
		ExtBuilder::new_build(vec![(ALICE, 10001)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);

			let nft_id = help::create_capsule_fast(alice.clone());
			let pallet_id = Capsule::account_id();

			let alice_balance = Balances::free_balance(ALICE);
			let pallet_balance = Balances::free_balance(pallet_id);

			// Funds are transferred
			let add = 1010;
			let ok = Capsule::add_funds(alice.clone(), nft_id, add);
			assert_ok!(ok);
			assert_eq!(Balances::free_balance(ALICE), alice_balance - add);
			assert_eq!(Balances::free_balance(pallet_id), pallet_balance + add);
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
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = help::create_capsule_fast(alice.clone());
			let data = Capsule::capsules(nft_id).unwrap();
			let old_reference = data.ipfs_reference.clone();
			let new_reference: CapsuleIPFSReference<Test> = bounded_vec![67];
			assert_ne!(old_reference, new_reference);

			// Happy path
			let ok = Capsule::set_ipfs_reference(alice.clone(), nft_id, new_reference.clone());
			assert_ok!(ok);
			assert_eq!(Capsule::capsules(nft_id).unwrap().ipfs_reference, new_reference);
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
			// Happy path
			let old_mint_fee = Capsule::capsule_mint_fee();
			let new_mint_fee = 654u128;
			assert_eq!(Capsule::capsule_mint_fee(), old_mint_fee);

			let ok = Capsule::set_capsule_mint_fee(root(), new_mint_fee);
			assert_ok!(ok);
			assert_eq!(Capsule::capsule_mint_fee(), new_mint_fee);
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
