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
use ternoa_common::traits::{MarketplaceExt, NFTExt};

use crate::{tests::mock, DescriptionVec, Error, MarketplaceData, NameVec, SaleData, URIVec};

type MPT = MarketplaceType;

#[test]
fn list_happy() {
	ExtBuilder::default()
		.caps(vec![(ALICE, 1000), (BOB, 1000)])
		.build()
		.execute_with(|| {
			let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
			let bob: mock::Origin = RawOrigin::Signed(BOB).into();

			// Happy path Public marketplace
			let price = 50;
			let series_id = vec![50];
			let nft_id =
				NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone()), 0).unwrap();
			let sale_info = SaleData::new(ALICE, price.clone(), 0);

			help::finish_series(alice.clone(), series_id);
			assert_ok!(Marketplace::list_nft(alice.clone(), nft_id, price, Some(0)));
			assert_eq!(Marketplace::nft_for_sale(nft_id), Some(sale_info));
			assert_eq!(<NFT as NFTExt>::is_listed_for_sale(nft_id), Some(true));

			// Happy path Private marketplace
			let series_id = vec![51];
			let mkp_id =
				help::create_mkp(bob.clone(), MPT::Private, 0, bounded_vec![1], vec![ALICE]);
			let sale_info = SaleData::new(ALICE, price.clone(), mkp_id);
			let nft_id =
				NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone()), 0).unwrap();

			help::finish_series(alice.clone(), series_id);
			let ok = Marketplace::list_nft(alice.clone(), nft_id, price, Some(mkp_id));
			assert_ok!(ok);
			assert_eq!(Marketplace::nft_for_sale(nft_id), Some(sale_info));
			assert_eq!(NFT::is_listed_for_sale(nft_id), Some(true));
		})
}

#[test]
fn list_unhappy() {
	ExtBuilder::default()
		.caps(vec![(ALICE, 1000), (BOB, 1000)])
		.build()
		.execute_with(|| {
			let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
			let bob: mock::Origin = RawOrigin::Signed(BOB).into();
			let price = 50;

			// Unhappy unknown NFT
			let ok = Marketplace::list_nft(alice.clone(), 10001, price, Some(0));
			assert_noop!(ok, Error::<Test>::NFTNotFound);

			// Unhappy not the NFT owner
			let nft_id = NFT::create_nft(BOB, bounded_vec![50], None, 0).unwrap();
			let ok = Marketplace::list_nft(alice.clone(), nft_id, price, Some(0));
			assert_noop!(ok, Error::<Test>::NotTheNFTOwner);

			// Unhappy series not completed
			let series_id = vec![50];
			let nft_id =
				NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone()), 0).unwrap();
			let ok = Marketplace::list_nft(alice.clone(), nft_id, price, Some(0));
			assert_noop!(ok, Error::<Test>::CannotListNFTsInUncompletedSeries);

			// Unhappy nft is capsulized
			help::finish_series(alice.clone(), series_id);
			NFT::set_converted_to_capsule(nft_id, true).unwrap();
			let ok = Marketplace::list_nft(alice.clone(), nft_id, price, Some(0));
			assert_noop!(ok, Error::<Test>::CannotListCapsules);
			NFT::set_converted_to_capsule(nft_id, false).unwrap();

			// Unhappy unknown marketplace
			let ok = Marketplace::list_nft(alice.clone(), nft_id, price, Some(10001));
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);

			// Unhappy not on the private list
			let mkp_id = help::create_mkp(bob.clone(), MPT::Private, 0, bounded_vec![1], vec![]);
			let ok = Marketplace::list_nft(alice.clone(), nft_id, price, Some(mkp_id));
			assert_noop!(ok, Error::<Test>::AccountNotAllowedToList);

			// Unhappy on the disallow list
			let mkp_id =
				help::create_mkp(bob.clone(), MPT::Public, 0, bounded_vec![1], vec![ALICE]);
			let ok = Marketplace::list_nft(alice.clone(), nft_id, price, Some(mkp_id));
			assert_noop!(ok, Error::<Test>::AccountNotAllowedToList);

			// Unhappy already listed for sale
			NFT::set_listed_for_sale(nft_id, true).unwrap();
			let ok = Marketplace::list_nft(alice.clone(), nft_id, price, None);
			assert_noop!(ok, Error::<Test>::CannotListNFTsThatAreAlreadyListed);
		})
}

#[test]
fn cumulated_fees_to_high() {
	ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
		let mkp_id = help::create_mkp(alice.clone(), MPT::Public, 70, bounded_vec![50], vec![]);
		let nft_id = help::create_nft(alice.clone(), bounded_vec![1], Some(vec![2]), 50);
		help::finish_series(alice.clone(), vec![2]);

		// Should fail and storage should remains empty
		let response = Marketplace::list_nft(alice.clone(), nft_id, 0, Some(mkp_id));
		assert_noop!(response, Error::<Test>::CumulatedFeesToHigh);
	})
}

#[test]
fn unlist_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

		let price = 50;
		let series_id = vec![50];
		let nft_id = NFT::create_nft(ALICE, bounded_vec![50], Some(series_id.clone()), 0).unwrap();

		// Happy path
		help::finish_series(alice.clone(), series_id);
		assert_ok!(Marketplace::list_nft(alice.clone(), nft_id, price, Some(0)));
		assert_ok!(Marketplace::unlist_nft(alice.clone(), nft_id));
		assert_eq!(Marketplace::nft_for_sale(nft_id), None);
		assert_eq!(NFT::is_listed_for_sale(nft_id), Some(false));
	})
}

#[test]
fn unlist_unhappy() {
	ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

		// Unhappy not the NFT owner
		let ok = Marketplace::unlist_nft(alice.clone(), 10001);
		assert_noop!(ok, Error::<Test>::NotTheNFTOwner);

		// Unhappy not listed NFT
		let nft_id = NFT::create_nft(ALICE, bounded_vec![50], None, 0).unwrap();
		let ok = Marketplace::unlist_nft(alice.clone(), nft_id);
		assert_noop!(ok, Error::<Test>::NFTNotForSale);
	})
}

#[test]
fn buy_happy() {
	ExtBuilder::default()
		.caps(vec![(ALICE, 1000), (BOB, 1000), (DAVE, 1000)])
		.build()
		.execute_with(|| {
			let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
			let bob: mock::Origin = RawOrigin::Signed(BOB).into();
			let dave: mock::Origin = RawOrigin::Signed(DAVE).into();

			let nft_id_1 =
				help::create_nft_and_lock_series(alice.clone(), bounded_vec![50], vec![50], 0);
			let nft_id_2 =
				help::create_nft_and_lock_series(alice.clone(), bounded_vec![50], vec![51], 0);
			let mkt_id =
				help::create_mkp(dave.clone(), MPT::Private, 10, bounded_vec![0], vec![ALICE]);

			let price = 50;
			assert_ok!(Marketplace::list_nft(alice.clone(), nft_id_1, price, None));

			let ok = Marketplace::list_nft(alice.clone(), nft_id_2, price, Some(mkt_id));
			assert_ok!(ok);

			// Happy path CAPS
			let bob_before = Balances::free_balance(BOB);
			let alice_before = Balances::free_balance(ALICE);

			assert_ok!(Marketplace::buy_nft(bob.clone(), nft_id_1));
			assert_eq!(NFT::is_listed_for_sale(nft_id_1), Some(false));
			assert_eq!(NFT::owner(nft_id_1), Some(BOB));
			assert_eq!(Marketplace::nft_for_sale(nft_id_1), None);

			assert_eq!(Balances::free_balance(BOB), bob_before - 50);
			assert_eq!(Balances::free_balance(ALICE), alice_before + 50);

			// Happy path PRIVATE (with commission fee)
			let bob_before = Balances::free_balance(BOB);
			let alice_before = Balances::free_balance(ALICE);
			let dave_before = Balances::free_balance(DAVE);

			assert_ok!(Marketplace::buy_nft(bob.clone(), nft_id_2));
			assert_eq!(NFT::is_listed_for_sale(nft_id_2), Some(false));
			assert_eq!(NFT::owner(nft_id_2), Some(BOB));
			assert_eq!(Marketplace::nft_for_sale(nft_id_2), None);

			assert_eq!(Balances::free_balance(BOB), bob_before - 50);
			assert_eq!(Balances::free_balance(ALICE), alice_before + 45);
			assert_eq!(Balances::free_balance(DAVE), dave_before + 5);
		})
}

#[test]
fn buy_happy_with_royalties() {
	ExtBuilder::default()
		.caps(vec![(ALICE, 1000), (BOB, 1000), (DAVE, 1000), (JACK, 1000)])
		.build()
		.execute_with(|| {
			let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
			let bob: mock::Origin = RawOrigin::Signed(BOB).into();
			let dave: mock::Origin = RawOrigin::Signed(DAVE).into();
			let jack: mock::Origin = RawOrigin::Signed(JACK).into();
			let mkp_id = help::create_mkp(alice.clone(), MPT::Public, 0, bounded_vec![0], vec![]);
			let commission_fee = Marketplace::get_marketplace(mkp_id).unwrap().commission_fee;
			let nft_id =
				help::create_nft_and_lock_series(bob.clone(), bounded_vec![50], vec![50], 20);
			let royaltie_fee = NFT::get_nft(nft_id).unwrap().royaltie_fee;

			let price = 150;
			assert_ok!(Marketplace::list_nft(bob.clone(), nft_id, price, Some(mkp_id)));

			let bob_before = Balances::free_balance(BOB);
			let alice_before = Balances::free_balance(ALICE);
			let dave_before = Balances::free_balance(DAVE);
			assert_ok!(Marketplace::buy_nft(dave.clone(), nft_id));

			// mkp owner
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_before + (price.saturating_mul(commission_fee.into()) / 100)
			);
			// nft creator and owner
			assert_eq!(
				Balances::free_balance(BOB),
				bob_before + price - (price.saturating_mul(commission_fee.into()) / 100)
			);
			// nft buyer
			assert_eq!(Balances::free_balance(DAVE), dave_before - price);

			let price = 260;
			assert_ok!(Marketplace::list_nft(dave, nft_id, price, Some(mkp_id)));

			let bob_before = Balances::free_balance(BOB);
			let alice_before = Balances::free_balance(ALICE);
			let dave_before = Balances::free_balance(DAVE);
			let jack_before = Balances::free_balance(JACK);

			assert_ok!(Marketplace::buy_nft(jack, nft_id));

			// mkp owner
			assert_eq!(
				Balances::free_balance(ALICE),
				alice_before + (price.saturating_mul(commission_fee.into()) / 100)
			);
			// nft creator
			assert_eq!(
				Balances::free_balance(BOB),
				bob_before + price.saturating_mul(royaltie_fee.into()) / 100
			);
			// nft owner
			assert_eq!(
				Balances::free_balance(DAVE),
				dave_before + price -
					(price.saturating_mul(commission_fee.into()) / 100) -
					(price.saturating_mul(royaltie_fee.into()) / 100)
			);
			// nft buyer
			assert_eq!(Balances::free_balance(JACK), jack_before - price);
		})
}

#[test]
fn buy_unhappy() {
	ExtBuilder::default()
		.caps(vec![(ALICE, 100), (BOB, 100)])
		.build()
		.execute_with(|| {
			let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
			let bob: mock::Origin = RawOrigin::Signed(BOB).into();

			let price = 5000;
			let nft_id =
				help::create_nft_and_lock_series(alice.clone(), bounded_vec![50], vec![50], 0);
			assert_ok!(Marketplace::list_nft(alice.clone(), nft_id, price, None));

			// Unhappy nft not on sale
			let ok = Marketplace::buy_nft(bob.clone(), 1001);
			assert_noop!(ok, Error::<Test>::NFTNotForSale);

			// Unhappy not enough caps
			let ok = Marketplace::buy_nft(bob.clone(), nft_id);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
}

#[test]
fn create_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 10000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

		assert_eq!(Marketplace::marketplace_id_generator(), 0);
		assert_eq!(Marketplace::marketplaces(1), None);
		let balance = Balances::free_balance(ALICE);
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

		// Happy path
		assert_ok!(Marketplace::create_marketplace(
			alice.clone(),
			kind,
			fee,
			name,
			uri,
			logo_uri,
			description,
		));
		assert_eq!(Marketplace::marketplace_id_generator(), 1);
		assert_eq!(Marketplace::marketplaces(1), Some(info));
		assert_eq!(Balances::free_balance(ALICE), balance - Marketplace::marketplace_mint_fee());
	})
}

#[test]
fn create_unhappy() {
	ExtBuilder::default().caps(vec![(ALICE, 5)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
		let normal_uri: URIVec<Test> = bounded_vec![66];

		// Unhappy invalid commission fee
		let ok = Marketplace::create_marketplace(
			alice.clone(),
			MPT::Public,
			101,
			bounded_vec![50],
			normal_uri.clone(),
			normal_uri.clone(),
			bounded_vec![],
		);
		assert_noop!(ok, Error::<Test>::InvalidCommissionFeeValue);

		// Unhappy not enough funds
		let ok = Marketplace::create_marketplace(
			alice.clone(),
			MPT::Public,
			5,
			bounded_vec![50],
			normal_uri.clone(),
			normal_uri.clone(),
			bounded_vec![],
		);
		assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
	})
}

#[test]
fn add_account_to_allow_list_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

		// Happy path
		let list = vec![];
		let mkp_1 =
			help::create_mkp(alice.clone(), MPT::Private, 0, bounded_vec![50], list.clone());
		assert_eq!(Marketplace::marketplaces(mkp_1).unwrap().allow_list, list);

		let ok = Marketplace::add_account_to_allow_list(alice.clone(), mkp_1, BOB);
		assert_ok!(ok);
		let list = vec![BOB];
		assert_eq!(Marketplace::marketplaces(mkp_1).unwrap().allow_list, list);
	})
}

#[test]
fn add_account_to_allow_list_unhappy() {
	ExtBuilder::default()
		.caps(vec![(BOB, 1000), (DAVE, 1000)])
		.build()
		.execute_with(|| {
			let bob: mock::Origin = RawOrigin::Signed(BOB).into();

			// Unhappy unknown marketplace
			let ok = Marketplace::add_account_to_allow_list(bob.clone(), 1001, DAVE);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);

			// Unhappy not marketplace owner
			let ok = Marketplace::add_account_to_allow_list(bob.clone(), 0, DAVE);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);

			// Unhappy unsupported marketplace type
			let mkp_id = help::create_mkp(bob.clone(), MPT::Public, 0, bounded_vec![50], vec![]);
			let ok = Marketplace::add_account_to_allow_list(bob.clone(), mkp_id, DAVE);
			assert_noop!(ok, Error::<Test>::UnsupportedMarketplace);
		})
}

#[test]
fn remove_account_from_allow_list_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

		// Happy path
		let list = vec![BOB];
		let mkp_id =
			help::create_mkp(alice.clone(), MPT::Private, 0, bounded_vec![50], list.clone());
		assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().allow_list, list);

		let ok = Marketplace::remove_account_from_allow_list(alice.clone(), mkp_id, BOB);
		assert_ok!(ok);
		let list: Vec<u64> = vec![];
		assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().allow_list, list);
	})
}

#[test]
fn remove_account_from_allow_list_unhappy() {
	ExtBuilder::default()
		.caps(vec![(BOB, 1000), (DAVE, 1000)])
		.build()
		.execute_with(|| {
			let bob: mock::Origin = RawOrigin::Signed(BOB).into();

			// Unhappy unknown marketplace
			let ok = Marketplace::remove_account_from_allow_list(bob.clone(), 1001, DAVE);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);

			// Unhappy not marketplace owner
			let ok = Marketplace::remove_account_from_allow_list(bob.clone(), 0, DAVE);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);

			// Unhappy unsupported marketplace type
			let mkp_id = help::create_mkp(bob.clone(), MPT::Public, 0, bounded_vec![50], vec![]);
			let ok = Marketplace::remove_account_from_allow_list(bob.clone(), mkp_id, DAVE);
			assert_noop!(ok, Error::<Test>::UnsupportedMarketplace);
		})
}

#[test]
fn set_owner_happy() {
	ExtBuilder::default()
		.caps(vec![(ALICE, 1000), (BOB, 1000)])
		.build()
		.execute_with(|| {
			let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

			// Happy path
			let mkp_id = help::create_mkp(alice.clone(), MPT::Private, 0, bounded_vec![50], vec![]);
			assert_ok!(Marketplace::set_marketplace_owner(alice.clone(), mkp_id, BOB));
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().owner, BOB);
		})
}

#[test]
fn set_owner_unhappy() {
	ExtBuilder::default().caps(vec![(BOB, 1000)]).build().execute_with(|| {
		let bob: mock::Origin = RawOrigin::Signed(BOB).into();

		// Unhappy unknown marketplace
		let ok = Marketplace::set_marketplace_owner(bob.clone(), 1001, DAVE);
		assert_noop!(ok, Error::<Test>::MarketplaceNotFound);

		// Unhappy not marketplace owner
		let ok = Marketplace::set_marketplace_owner(bob.clone(), 0, DAVE);
		assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
	})
}

#[test]
fn set_market_type_happy() {
	ExtBuilder::default()
		.caps(vec![(ALICE, 1000), (BOB, 1000)])
		.build()
		.execute_with(|| {
			let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

			let kind = MPT::Public;
			let mkp_id = help::create_mkp(alice.clone(), MPT::Public, 0, bounded_vec![50], vec![]);
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().kind, kind);

			// Happy path Public to Private
			let kind = MPT::Private;
			assert_ok!(Marketplace::set_marketplace_type(alice.clone(), mkp_id, kind));
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().kind, kind);

			// Happy path Private to Public
			let kind = MPT::Public;
			assert_ok!(Marketplace::set_marketplace_type(alice.clone(), mkp_id, kind));
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().kind, kind);
		})
}

#[test]
fn set_market_type_unhappy() {
	ExtBuilder::default().caps(vec![(BOB, 1000)]).build().execute_with(|| {
		let bob: mock::Origin = RawOrigin::Signed(BOB).into();

		let kind = MPT::Public;

		// Unhappy unknown marketplace
		let ok = Marketplace::set_marketplace_type(bob.clone(), 1001, kind);
		assert_noop!(ok, Error::<Test>::MarketplaceNotFound);

		// Unhappy not marketplace owner
		let ok = Marketplace::set_marketplace_type(bob.clone(), 0, kind);
		assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
	})
}

#[test]
fn set_name_happy() {
	ExtBuilder::default()
		.caps(vec![(ALICE, 1000), (BOB, 1000)])
		.build()
		.execute_with(|| {
			let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

			// Happy path
			let name: NameVec<Test> = bounded_vec![50];
			let mkp_id = help::create_mkp(alice.clone(), MPT::Private, 0, name.clone(), vec![]);
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().name, name);

			let name: NameVec<Test> = bounded_vec![51];
			assert_ok!(Marketplace::set_marketplace_name(alice.clone(), mkp_id, name.clone()));
			assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().name, name);
		})
}

#[test]
fn set_name_unhappy() {
	ExtBuilder::default().caps(vec![(BOB, 1000)]).build().execute_with(|| {
		let bob: mock::Origin = RawOrigin::Signed(BOB).into();

		// Unhappy unknown marketplace
		let ok = Marketplace::set_marketplace_name(bob.clone(), 1001, bounded_vec![51]);
		assert_noop!(ok, Error::<Test>::MarketplaceNotFound);

		// Unhappy not marketplace owner
		let ok = Marketplace::set_marketplace_name(bob.clone(), 0, bounded_vec![51]);
		assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
	})
}

#[test]
fn set_marketplace_mint_fee_happy() {
	ExtBuilder::default().build().execute_with(|| {
		// Happy path
		let old_mint_fee = Marketplace::marketplace_mint_fee();
		let new_mint_fee = 654u128;
		assert_eq!(Marketplace::marketplace_mint_fee(), old_mint_fee);

		let ok = Marketplace::set_marketplace_mint_fee(mock::Origin::root(), new_mint_fee);
		assert_ok!(ok);
		assert_eq!(Marketplace::marketplace_mint_fee(), new_mint_fee);
	})
}

#[test]
fn set_marketplace_mint_fee_unhappy() {
	ExtBuilder::default().caps(vec![(ALICE, 10000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

		// Unhappy non root user tries to modify the mint fee
		let ok = Marketplace::set_marketplace_mint_fee(alice.clone(), 654);
		assert_noop!(ok, BadOrigin);
	})
}

#[test]
fn set_commission_fee_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

		let fee = 10;
		let id = help::create_mkp(alice.clone(), MPT::Public, fee, bounded_vec![50], vec![]);
		assert_eq!(Marketplace::marketplaces(id).unwrap().commission_fee, fee);

		// Happy path
		let fee = 15;
		assert_ok!(Marketplace::set_marketplace_commission_fee(alice.clone(), id, fee));
		assert_eq!(Marketplace::marketplaces(id).unwrap().commission_fee, fee);
	})
}

#[test]
fn set_commission_fee_unhappy() {
	ExtBuilder::default().caps(vec![(BOB, 1000)]).build().execute_with(|| {
		let bob: mock::Origin = RawOrigin::Signed(BOB).into();

		// Unhappy commission fee is more than 100
		let ok = Marketplace::set_marketplace_commission_fee(bob.clone(), 0, 101);
		assert_noop!(ok, Error::<Test>::InvalidCommissionFeeValue);

		// Unhappy unknown marketplace
		let ok = Marketplace::set_marketplace_commission_fee(bob.clone(), 1001, 15);
		assert_noop!(ok, Error::<Test>::MarketplaceNotFound);

		// Unhappy not marketplace owner
		let ok = Marketplace::set_marketplace_commission_fee(bob.clone(), 0, 15);
		assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
	})
}

#[test]
fn update_uri_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 10000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

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

		assert_ok!(Marketplace::create_marketplace(
			alice.clone(),
			kind.clone(),
			fee,
			name.clone(),
			uri.clone(),
			uri.clone(),
			bounded_vec![],
		));
		assert_ne!(Marketplace::marketplaces(1).unwrap().uri, updated_uri);
		assert_ok!(Marketplace::set_marketplace_uri(alice.clone(), 1, updated_uri));
		assert_eq!(Marketplace::marketplaces(1), Some(updated_info));
	})
}

#[test]
fn update_logo_uri_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 10000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

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

		assert_ok!(Marketplace::create_marketplace(
			alice.clone(),
			kind.clone(),
			fee,
			name.clone(),
			uri.clone(),
			uri.clone(),
			bounded_vec![],
		));
		assert_ne!(Marketplace::marketplaces(1).unwrap().uri, updated_uri.clone());

		assert_ok!(Marketplace::set_marketplace_logo_uri(alice.clone(), 1, updated_uri));
		assert_eq!(Marketplace::marketplaces(1), Some(updated_info));
	})
}

#[test]
fn add_account_to_disallow_list_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

		// Happy path
		let list = vec![];
		let mkp_1 = help::create_mkp(alice.clone(), MPT::Public, 0, bounded_vec![50], list.clone());
		assert_eq!(Marketplace::marketplaces(mkp_1).unwrap().disallow_list, list);

		let ok = Marketplace::add_account_to_disallow_list(alice.clone(), mkp_1, BOB);
		assert_ok!(ok);
		let list = vec![BOB];
		assert_eq!(Marketplace::marketplaces(mkp_1).unwrap().disallow_list, list);
	})
}

#[test]
fn add_account_to_disallow_list_unhappy() {
	ExtBuilder::default()
		.caps(vec![(BOB, 1000), (DAVE, 1000)])
		.build()
		.execute_with(|| {
			let bob: mock::Origin = RawOrigin::Signed(BOB).into();

			// Unhappy unknown marketplace
			let ok = Marketplace::add_account_to_disallow_list(bob.clone(), 1001, DAVE);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);

			// Unhappy not marketplace owner
			let ok = Marketplace::add_account_to_disallow_list(bob.clone(), 0, DAVE);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);

			// Unhappy unsupported marketplace type
			let mkp_id = help::create_mkp(bob.clone(), MPT::Private, 0, bounded_vec![50], vec![]);
			let ok = Marketplace::add_account_to_disallow_list(bob.clone(), mkp_id, DAVE);
			assert_noop!(ok, Error::<Test>::UnsupportedMarketplace);
		})
}

#[test]
fn remove_account_from_disallow_list_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

		// Happy path
		let list = vec![BOB];
		let mkp_id =
			help::create_mkp(alice.clone(), MPT::Public, 0, bounded_vec![50], list.clone());
		assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().disallow_list, list);

		let ok = Marketplace::remove_account_from_disallow_list(alice.clone(), mkp_id, BOB);
		assert_ok!(ok);
		let list: Vec<u64> = vec![];
		assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().disallow_list, list);
	})
}

#[test]
fn remove_account_from_disallow_list_unhappy() {
	ExtBuilder::default()
		.caps(vec![(BOB, 1000), (DAVE, 1000)])
		.build()
		.execute_with(|| {
			let bob: mock::Origin = RawOrigin::Signed(BOB).into();

			// Unhappy unknown marketplace
			let ok = Marketplace::remove_account_from_disallow_list(bob.clone(), 1001, DAVE);
			assert_noop!(ok, Error::<Test>::MarketplaceNotFound);

			// Unhappy not marketplace owner
			let ok = Marketplace::remove_account_from_disallow_list(bob.clone(), 0, DAVE);
			assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);

			// Unhappy unsupported marketplace type
			let mkp_id = help::create_mkp(bob.clone(), MPT::Private, 0, bounded_vec![50], vec![]);
			let ok = Marketplace::remove_account_from_disallow_list(bob.clone(), mkp_id, DAVE);
			assert_noop!(ok, Error::<Test>::UnsupportedMarketplace);
		})
}

#[test]
fn set_description_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 10000)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

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

		assert_ok!(Marketplace::create_marketplace(
			alice.clone(),
			kind.clone(),
			fee,
			name.clone(),
			uri.clone(),
			uri.clone(),
			description.clone(),
		));

		assert_ok!(Marketplace::set_marketplace_description(alice.clone(), 1, updated_description));
		assert_eq!(Marketplace::marketplaces(1), Some(updated_info));
	})
}
