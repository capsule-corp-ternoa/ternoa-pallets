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
use frame_support::{assert_noop, assert_ok, bounded_vec, error::BadOrigin};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use primitives::marketplace::MarketplaceType;
use ternoa_common::traits::NFTExt;

use crate::{
	tests::mock, DescriptionVec, Error, Event as MarketplaceEvent, MarketplaceData, NameVec,
	SaleData, URIVec,
};

type MPT = MarketplaceType;

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

pub mod list_nft {
	pub use super::*;

	#[test]
	fn list_nft_on_public_marketplace() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let series_id = vec![51];
			let price = 50;
			let mkp_id =
				help::create_mkp(origin(BOB), MPT::Private, 0, bounded_vec![1], vec![ALICE]);
			let sale_info = SaleData::new(ALICE, price, mkp_id);
			let nft_id = NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone())).unwrap();
			help::finish_series(alice.clone(), series_id);

			let ok = Marketplace::list_nft(alice, nft_id, price, Some(mkp_id));
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::nft_for_sale(nft_id), Some(sale_info));
			assert_eq!(NFT::is_listed_for_sale(nft_id), Some(true));

			// Event
			let event = MarketplaceEvent::NFTListed { nft_id, price, marketplace_id: mkp_id };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn list_nft_on_private_marketplace() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let series_id = vec![51];
			let price = 50;
			let mkp_id =
				help::create_mkp(origin(BOB), MPT::Private, 0, bounded_vec![1], vec![ALICE]);
			let sale_info = SaleData::new(ALICE, price, mkp_id);
			let nft_id = NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone())).unwrap();
			help::finish_series(alice.clone(), series_id);

			let ok = Marketplace::list_nft(alice, nft_id, price, Some(mkp_id));
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::nft_for_sale(nft_id), Some(sale_info));
			assert_eq!(NFT::is_listed_for_sale(nft_id), Some(true));

			// Event
			let event = MarketplaceEvent::NFTListed { nft_id, price, marketplace_id: mkp_id };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let ok = Marketplace::list_nft(origin(ALICE), 10001, 50, Some(0));
			assert_noop!(ok, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let nft_id = NFT::create_nft(BOB, bounded_vec![50], None).unwrap();

			let ok = Marketplace::list_nft(origin(ALICE), nft_id, 50, Some(0));
			assert_noop!(ok, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_list_nfts_in_uncompleted_series() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let nft_id = NFT::create_nft(ALICE, bounded_vec![50], Some(vec![50])).unwrap();

			let ok = Marketplace::list_nft(origin(ALICE), nft_id, 50, Some(0));
			assert_noop!(ok, Error::<Test>::CannotListNFTsInUncompletedSeries);
		})
	}

	#[test]
	fn cannot_list_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id = NFT::create_nft(ALICE, bounded_vec![50], Some(vec![50])).unwrap();
			NFT::set_converted_to_capsule(nft_id, true).unwrap();

			let ok = Marketplace::list_nft(alice, nft_id, 50, Some(0));
			assert_noop!(ok, Error::<Test>::CannotListCapsules);

			NFT::set_converted_to_capsule(nft_id, false).unwrap();
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let series_id = vec![50];
			let nft_id = NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone())).unwrap();
			help::finish_series(alice.clone(), series_id);

			let ok = Marketplace::list_nft(alice.clone(), nft_id, 50, Some(10001));
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn account_not_allowed_to_list() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let mkp_id = help::create_mkp(origin(BOB), MPT::Private, 0, bounded_vec![1], vec![]);
			let series_id = vec![50];
			let nft_id = NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone())).unwrap();
			help::finish_series(origin(ALICE), series_id);

			let ok = Marketplace::list_nft(origin(ALICE), nft_id, 50, Some(mkp_id));
			assert_noop!(ok, Error::<Test>::AccountNotAllowedToList);
		})
	}

	#[test]
	fn account_not_allowed_to_list_because_on_disallow_list() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let mkp_id =
				help::create_mkp(origin(BOB), MPT::Public, 0, bounded_vec![1], vec![ALICE]);
			let series_id = vec![50];
			let nft_id = NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone())).unwrap();
			help::finish_series(origin(ALICE), series_id);

			let ok = Marketplace::list_nft(origin(ALICE), nft_id, 50, Some(mkp_id));
			assert_noop!(ok, Error::<Test>::AccountNotAllowedToList);
		})
	}

	#[test]
	fn cannot_list_nfts_that_are_already_listed() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let series_id = vec![50];
			let price = 50;
			let nft_id = NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone())).unwrap();
			help::finish_series(origin(ALICE), series_id);

			let ok = Marketplace::list_nft(alice.clone(), nft_id, price, Some(0));
			assert_ok!(ok);
			NFT::set_listed_for_sale(nft_id, true).unwrap();

			let ok = Marketplace::list_nft(alice.clone(), nft_id, price, None);
			assert_noop!(ok, Error::<Test>::CannotListNFTsThatAreAlreadyListed);
		})
	}
}

pub mod unlist_nft {
	pub use super::*;

	#[test]
	fn unlist_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let series_id = vec![50];
			let nft_id = NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone())).unwrap();
			help::finish_series(alice.clone(), series_id);
			let ok = Marketplace::list_nft(alice.clone(), nft_id, 50, Some(0));
			assert_ok!(ok);

			let ok = Marketplace::unlist_nft(alice.clone(), nft_id);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::nft_for_sale(nft_id), None);
			assert_eq!(NFT::is_listed_for_sale(nft_id), Some(false));

			// Events
			let event = MarketplaceEvent::NFTUnlisted { nft_id };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let ok = Marketplace::unlist_nft(origin(ALICE), 10001);
			assert_noop!(ok, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn nft_not_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let nft_id = NFT::create_nft(ALICE, bounded_vec![50], None).unwrap();
			let ok = Marketplace::unlist_nft(origin(ALICE), nft_id);
			assert_noop!(ok, Error::<Test>::NFTNotForSale);
		})
	}
}

pub mod buy_nft {
	pub use super::*;

	#[test]
	fn buy_nft() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id =
				help::create_nft_and_lock_series(alice.clone(), bounded_vec![50], vec![50]);
			let ok = Marketplace::list_nft(alice, nft_id, 50, None);
			assert_ok!(ok);
			let bob_balance_before = Balances::free_balance(BOB);
			let alice_balance_before = Balances::free_balance(ALICE);

			let ok = Marketplace::buy_nft(origin(BOB), nft_id);
			assert_ok!(ok);

			// Storage
			assert_eq!(NFT::is_listed_for_sale(nft_id), Some(false));
			assert_eq!(NFT::owner(nft_id), Some(BOB));
			assert_eq!(Marketplace::nft_for_sale(nft_id), None);
			assert_eq!(Balances::free_balance(BOB), bob_balance_before - 50);
			assert_eq!(Balances::free_balance(ALICE), alice_balance_before + 50);

			// Event
			let event = MarketplaceEvent::NFTSold { nft_id, owner: BOB };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn buy_nft_on_private_marketplace() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id =
				help::create_nft_and_lock_series(alice.clone(), bounded_vec![50], vec![51]);
			let mkt_id =
				help::create_mkp(origin(DAVE), MPT::Private, 10, bounded_vec![0], vec![ALICE]);
			let ok = Marketplace::list_nft(alice, nft_id, 50, Some(mkt_id));
			assert_ok!(ok);
			let bob_before = Balances::free_balance(BOB);
			let alice_before = Balances::free_balance(ALICE);
			let dave_before = Balances::free_balance(DAVE);

			let ok = Marketplace::buy_nft(origin(BOB), nft_id);
			assert_ok!(ok);

			// Storage
			assert_eq!(NFT::is_listed_for_sale(nft_id), Some(false));
			assert_eq!(NFT::owner(nft_id), Some(BOB));
			assert_eq!(Marketplace::nft_for_sale(nft_id), None);
			assert_eq!(Balances::free_balance(BOB), bob_before - 50);
			assert_eq!(Balances::free_balance(ALICE), alice_before + 45);
			assert_eq!(Balances::free_balance(DAVE), dave_before + 5);

			// Event
			let event = MarketplaceEvent::NFTSold { nft_id, owner: BOB };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn nft_not_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100), (BOB, 100)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id =
				help::create_nft_and_lock_series(alice.clone(), bounded_vec![50], vec![50]);
			let ok = Marketplace::list_nft(alice, nft_id, 5000, None);
			assert_ok!(ok);

			let ok = Marketplace::buy_nft(origin(BOB), 1001);
			assert_noop!(ok, Error::<Test>::NFTNotForSale);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, 100), (BOB, 100)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let nft_id =
				help::create_nft_and_lock_series(alice.clone(), bounded_vec![50], vec![50]);
			let ok = Marketplace::list_nft(alice, nft_id, 5000, None);
			assert_ok!(ok);

			let ok = Marketplace::buy_nft(origin(BOB), nft_id);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

pub mod create_marketplace {
	pub use super::*;

	#[test]
	fn create_marketplace() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice_balance_before = Balances::free_balance(ALICE);
			let fee = 25;
			let name: NameVec<Test> = bounded_vec![50];
			let kind = MPT::Public;
			let uri: URIVec<Test> = bounded_vec![65];
			let logo_uri: URIVec<Test> = bounded_vec![66];
			let description: DescriptionVec<Test> = bounded_vec![];
			let info = MarketplaceData::new(
				kind,
				fee,
				ALICE,
				bounded_vec![],
				bounded_vec![],
				name.clone(),
				uri.clone(),
				logo_uri.clone(),
				description.clone(),
			);

			let ok = Marketplace::create_marketplace(
				origin(ALICE),
				kind,
				fee,
				name,
				uri,
				logo_uri,
				description,
			);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplace_id_generator(), 1);
			assert_eq!(Marketplace::marketplaces(1), Some(info));
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_balance_before - Marketplace::marketplace_mint_fee()
			);

			// Event
			let event = MarketplaceEvent::MarketplaceCreated { marketplace_id: 1, owner: ALICE };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn invalid_commission_fee_value() {
		ExtBuilder::new_build(vec![(ALICE, 5)]).execute_with(|| {
			let normal_uri: URIVec<Test> = bounded_vec![66];

			let ok = Marketplace::create_marketplace(
				origin(ALICE),
				MPT::Public,
				101,
				bounded_vec![50],
				normal_uri.clone(),
				normal_uri,
				bounded_vec![],
			);
			assert_noop!(ok, Error::<Test>::InvalidCommissionFeeValue);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, 5)]).execute_with(|| {
			let normal_uri: URIVec<Test> = bounded_vec![66];

			let ok = Marketplace::create_marketplace(
				origin(ALICE),
				MPT::Public,
				5,
				bounded_vec![50],
				normal_uri.clone(),
				normal_uri,
				bounded_vec![],
			);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}
}

pub mod add_account_to_allow_list {
	pub use super::*;

	#[test]
	fn add_account_to_allow_list() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let list = vec![];
			let mkp_id =
				help::create_mkp(alice.clone(), MPT::Private, 0, bounded_vec![50], list.clone());
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().allow_list, list);
			let ok = Marketplace::add_account_to_allow_list(alice, mkp_id, BOB);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().allow_list, vec![BOB]);

			// Event
			let event =
				MarketplaceEvent::AccountAddedToAllowList { marketplace_id: mkp_id, owner: BOB };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let ok = Marketplace::add_account_to_allow_list(origin(BOB), 1001, DAVE);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let ok = Marketplace::add_account_to_allow_list(origin(BOB), 0, DAVE);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
		})
	}

	#[test]
	fn unsupported_marketplace() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let bob: mock::Origin = origin(BOB);
			let mkp_id = help::create_mkp(bob.clone(), MPT::Public, 0, bounded_vec![50], vec![]);

			let ok = Marketplace::add_account_to_allow_list(bob, mkp_id, DAVE);
			assert_noop!(ok, Error::<Test>::UnsupportedMarketplace);
		})
	}
}

pub mod remove_account_from_allow_list {
	pub use super::*;

	#[test]
	fn remove_account_from_allow_list() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice = origin(ALICE);
			let list = vec![BOB];
			let mkp_id =
				help::create_mkp(alice.clone(), MPT::Private, 0, bounded_vec![50], list.clone());
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().allow_list, list);

			let ok = Marketplace::remove_account_from_allow_list(alice, mkp_id, BOB);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().allow_list, vec![]);

			// Event
			let event = MarketplaceEvent::AccountRemovedFromAllowList {
				marketplace_id: mkp_id,
				owner: BOB,
			};
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let ok = Marketplace::remove_account_from_allow_list(origin(BOB), 1001, DAVE);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let ok = Marketplace::remove_account_from_allow_list(origin(BOB), 0, DAVE);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
		})
	}

	#[test]
	fn unsupported_marketplace() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let bob: mock::Origin = origin(BOB);
			let mkp_id = help::create_mkp(bob.clone(), MPT::Public, 0, bounded_vec![50], vec![]);

			let ok = Marketplace::remove_account_from_allow_list(bob, mkp_id, DAVE);
			assert_noop!(ok, Error::<Test>::UnsupportedMarketplace);
		})
	}
}

pub mod set_marketplace_owner {
	pub use super::*;

	#[test]
	fn set_marketplace_owner() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let mkp_id = help::create_mkp(alice.clone(), MPT::Private, 0, bounded_vec![50], vec![]);
			let ok = Marketplace::set_marketplace_owner(alice, mkp_id, BOB);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().owner, BOB);

			// Event
			let event =
				MarketplaceEvent::MarketplaceOwnerChanged { marketplace_id: mkp_id, owner: BOB };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(BOB, 1000)]).execute_with(|| {
			let ok = Marketplace::set_marketplace_owner(origin(BOB), 1001, DAVE);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(BOB, 1000)]).execute_with(|| {
			let ok = Marketplace::set_marketplace_owner(origin(BOB), 0, DAVE);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
		})
	}
}

pub mod set_marketplace_type {
	pub use super::*;

	#[test]
	fn set_marketplace_type_public_to_private() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let mkp_id = help::create_mkp(alice.clone(), MPT::Public, 0, bounded_vec![50], vec![]);
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().kind, MPT::Public);

			let ok = Marketplace::set_marketplace_type(alice.clone(), mkp_id, MPT::Private);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().kind, MPT::Private);

			// Event
			let event = MarketplaceEvent::MarketplaceTypeChanged {
				marketplace_id: mkp_id,
				kind: MPT::Private,
			};
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn set_marketplace_type_private_to_public() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let mkp_id = help::create_mkp(alice.clone(), MPT::Private, 0, bounded_vec![50], vec![]);
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().kind, MPT::Private);

			let ok = Marketplace::set_marketplace_type(alice, mkp_id, MPT::Public);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().kind, MPT::Public);

			// Event
			let event = MarketplaceEvent::MarketplaceTypeChanged {
				marketplace_id: mkp_id,
				kind: MPT::Public,
			};
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(BOB, 1000)]).execute_with(|| {
			let ok = Marketplace::set_marketplace_type(origin(BOB), 1001, MPT::Public);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(BOB, 1000)]).execute_with(|| {
			let ok = Marketplace::set_marketplace_type(origin(BOB), 0, MPT::Public);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
		})
	}
}

pub mod set_marketplace_name {
	pub use super::*;

	#[test]
	fn set_marketplace_name() {
		ExtBuilder::new_build(vec![(ALICE, 1000), (BOB, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let name: NameVec<Test> = bounded_vec![50];
			let mkp_id = help::create_mkp(alice.clone(), MPT::Private, 0, name.clone(), vec![]);
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().name, name);
			let new_name: NameVec<Test> = bounded_vec![51];

			let ok = Marketplace::set_marketplace_name(alice, mkp_id, new_name.clone());
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().name, new_name);

			// Event
			let event =
				MarketplaceEvent::MarketplaceNameUpdated { marketplace_id: mkp_id, name: new_name };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(BOB, 1000)]).execute_with(|| {
			let ok = Marketplace::set_marketplace_name(origin(BOB), 1001, bounded_vec![51]);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(BOB, 1000)]).execute_with(|| {
			let ok = Marketplace::set_marketplace_name(origin(BOB), 0, bounded_vec![51]);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
		})
	}
}

pub mod marketplace_mint_fee {
	pub use super::*;

	#[test]
	fn marketplace_mint_fee() {
		ExtBuilder::new_build(vec![]).execute_with(|| {
			let new_mint_fee = 654u128;

			let ok = Marketplace::set_marketplace_mint_fee(root(), new_mint_fee);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplace_mint_fee(), new_mint_fee);

			// Event
			let event = MarketplaceEvent::MarketplaceMintFeeUpdated { fee: new_mint_fee };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let ok = Marketplace::set_marketplace_mint_fee(origin(ALICE), 654);
			assert_noop!(ok, BadOrigin);
		})
	}
}

pub mod set_marketplace_commission_fee {
	pub use super::*;

	#[test]
	fn set_marketplace_commission_fee() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let fee = 10;
			let mkp_id =
				help::create_mkp(alice.clone(), MPT::Public, fee, bounded_vec![50], vec![]);
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().commission_fee, fee);
			let new_fee = 15;

			let ok = Marketplace::set_marketplace_commission_fee(alice, mkp_id, new_fee);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().commission_fee, new_fee);

			// Event
			let event = MarketplaceEvent::MarketplaceCommissionFeeUpdated {
				marketplace_id: mkp_id,
				fee: new_fee,
			};
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn invalid_commission_fee_value() {
		ExtBuilder::new_build(vec![(BOB, 1000)]).execute_with(|| {
			let ok = Marketplace::set_marketplace_commission_fee(origin(BOB), 0, 101);
			assert_noop!(ok, Error::<Test>::InvalidCommissionFeeValue);
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(BOB, 1000)]).execute_with(|| {
			let ok = Marketplace::set_marketplace_commission_fee(origin(BOB), 1001, 15);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(BOB, 1000)]).execute_with(|| {
			let ok = Marketplace::set_marketplace_commission_fee(origin(BOB), 0, 15);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
		})
	}
}

pub mod set_marketplace_uri {
	pub use super::*;

	#[test]
	fn set_marketplace_uri() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let fee = 25;
			let name: NameVec<Test> = bounded_vec![50];
			let kind = MPT::Public;
			let uri: URIVec<Test> = bounded_vec![66];
			let updated_uri: URIVec<Test> = bounded_vec![67];
			let updated_info = MarketplaceData::new(
				kind,
				fee,
				ALICE,
				bounded_vec![],
				bounded_vec![],
				name.clone(),
				updated_uri.clone(),
				uri.clone(),
				bounded_vec![],
			);

			let ok = Marketplace::create_marketplace(
				alice.clone(),
				kind.clone(),
				fee,
				name.clone(),
				uri.clone(),
				uri.clone(),
				bounded_vec![],
			);
			assert_ok!(ok);
			assert_ne!(Marketplace::marketplaces(1).unwrap().uri, updated_uri);

			let ok = Marketplace::set_marketplace_uri(alice, 1, updated_uri.clone());
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(1), Some(updated_info));

			// Event
			let event =
				MarketplaceEvent::MarketplaceUriUpdated { marketplace_id: 1, uri: updated_uri };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}
}

pub mod set_marketplace_logo_uri {
	pub use super::*;

	#[test]
	fn set_marketplace_logo_uri() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let fee = 25;
			let name: NameVec<Test> = bounded_vec![50];
			let kind = MPT::Public;
			let uri: URIVec<Test> = bounded_vec![66];
			let updated_uri: URIVec<Test> = bounded_vec![67];
			let updated_info = MarketplaceData::new(
				kind,
				fee,
				ALICE,
				bounded_vec![],
				bounded_vec![],
				name.clone(),
				uri.clone(),
				updated_uri.clone(),
				bounded_vec![],
			);
			let ok = Marketplace::create_marketplace(
				alice.clone(),
				kind.clone(),
				fee,
				name.clone(),
				uri.clone(),
				uri.clone(),
				bounded_vec![],
			);
			assert_ok!(ok);
			assert_ne!(Marketplace::marketplaces(1).unwrap().uri, updated_uri.clone());

			let ok = Marketplace::set_marketplace_logo_uri(alice.clone(), 1, updated_uri.clone());
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(1), Some(updated_info));

			// Event
			let event =
				MarketplaceEvent::MarketplaceLogoUriUpdated { marketplace_id: 1, uri: updated_uri };
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}
}

pub mod add_account_to_disallow_list {
	pub use super::*;

	#[test]
	fn add_account_to_disallow_list() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let list = vec![];
			let mkp_id =
				help::create_mkp(alice.clone(), MPT::Public, 0, bounded_vec![50], list.clone());
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().disallow_list, list);

			let ok = Marketplace::add_account_to_disallow_list(alice.clone(), mkp_id, BOB);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().disallow_list, vec![BOB]);

			// Event
			let event = MarketplaceEvent::AccountAddedToDisallowList {
				marketplace_id: mkp_id,
				account_id: BOB,
			};
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let ok = Marketplace::add_account_to_disallow_list(origin(BOB), 1001, DAVE);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let ok = Marketplace::add_account_to_disallow_list(origin(BOB), 0, DAVE);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
		})
	}

	#[test]
	fn unsupported_marketplace() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let bob: mock::Origin = origin(BOB);
			let mkp_id = help::create_mkp(bob.clone(), MPT::Private, 0, bounded_vec![50], vec![]);

			let ok = Marketplace::add_account_to_disallow_list(bob, mkp_id, DAVE);
			assert_noop!(ok, Error::<Test>::UnsupportedMarketplace);
		})
	}
}

pub mod remove_account_from_disallow_list {
	pub use super::*;

	#[test]
	fn remove_account_from_disallow_list() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let list = vec![BOB];
			let mkp_id =
				help::create_mkp(alice.clone(), MPT::Public, 0, bounded_vec![50], list.clone());
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().disallow_list, list);

			let ok = Marketplace::remove_account_from_disallow_list(alice, mkp_id, BOB);
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().disallow_list, vec![]);

			// Event
			let event = MarketplaceEvent::AccountRemovedFromDisallowList {
				marketplace_id: mkp_id,
				account_id: BOB,
			};
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn marketplace_not_found() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let ok = Marketplace::remove_account_from_disallow_list(origin(BOB), 1001, DAVE);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);
		})
	}

	#[test]
	fn not_the_marketplace_owner() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let ok = Marketplace::remove_account_from_disallow_list(origin(BOB), 0, DAVE);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
		})
	}

	#[test]
	fn unsupported_marketplace() {
		ExtBuilder::new_build(vec![(BOB, 1000), (DAVE, 1000)]).execute_with(|| {
			let bob: mock::Origin = origin(BOB);
			let mkp_id = help::create_mkp(bob.clone(), MPT::Private, 0, bounded_vec![50], vec![]);

			let ok = Marketplace::remove_account_from_disallow_list(bob, mkp_id, DAVE);
			assert_noop!(ok, Error::<Test>::UnsupportedMarketplace);
		})
	}
}

pub mod set_marketplace_description {
	pub use super::*;

	#[test]
	fn set_marketplace_description() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let fee = 25;
			let name: NameVec<Test> = bounded_vec![50];
			let kind = MPT::Public;
			let uri: URIVec<Test> = bounded_vec![66];
			let description: DescriptionVec<Test> = bounded_vec![66];
			let updated_description: DescriptionVec<Test> = bounded_vec![67];
			let updated_info = MarketplaceData::new(
				kind,
				fee,
				ALICE,
				bounded_vec![],
				bounded_vec![],
				name.clone(),
				uri.clone(),
				uri.clone(),
				updated_description.clone(),
			);

			let ok = Marketplace::create_marketplace(
				alice.clone(),
				kind.clone(),
				fee,
				name.clone(),
				uri.clone(),
				uri.clone(),
				description.clone(),
			);
			assert_ok!(ok);

			let ok =
				Marketplace::set_marketplace_description(alice, 1, updated_description.clone());
			assert_ok!(ok);

			// Storage
			assert_eq!(Marketplace::marketplaces(1), Some(updated_info));

			// Event
			let event = MarketplaceEvent::MarketplaceDescriptionUpdated {
				marketplace_id: 1,
				description: updated_description,
			};
			let event = Event::Marketplace(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}
}
