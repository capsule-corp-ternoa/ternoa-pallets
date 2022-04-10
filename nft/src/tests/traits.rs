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
use crate::{tests::mock, Error};
use frame_support::{assert_noop, assert_ok, bounded_vec};
use frame_system::RawOrigin;
use ternoa_common::traits::NFTTrait;

#[test]
fn set_owner_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 100)]).build().execute_with(|| {
		// Happy path
		let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, bounded_vec![1], None).unwrap();
		assert_ok!(NFTs::set_owner(nft_id, &BOB));
		assert_eq!(NFTs::data(nft_id).unwrap().owner, BOB);
	})
}

#[test]
fn set_owner_unhappy() {
	ExtBuilder::default().caps(vec![(ALICE, 100)]).build().execute_with(|| {
		// Unhappy Unknown NFT
		assert_noop!(NFTs::set_owner(1000, &BOB), Error::<Test>::NFTNotFound);
	})
}

#[test]
fn owner_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 100)]).build().execute_with(|| {
		// Happy path
		let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, bounded_vec![1], None).unwrap();
		assert_eq!(NFTs::owner(nft_id), Some(ALICE));
	})
}

#[test]
fn owner_unhappy() {
	ExtBuilder::default().build().execute_with(|| {
		// Unhappy invalid NFT Id
		assert_eq!(NFTs::owner(1000), None);
	})
}

#[test]
fn is_series_completed_happy() {
	ExtBuilder::default().caps(vec![(ALICE, 100)]).build().execute_with(|| {
		let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

		// Happy path
		let series_id = vec![50];
		let nft_id =
			<NFTs as NFTTrait>::create_nft(ALICE, bounded_vec![1], Some(series_id.clone()))
				.unwrap();
		assert_eq!(NFTs::is_nft_in_completed_series(nft_id), Some(false));
		assert_ok!(NFTs::finish_series(alice, series_id));
		assert_eq!(NFTs::is_nft_in_completed_series(nft_id), Some(true));
	})
}

#[test]
fn is_series_completed_unhappy() {
	ExtBuilder::default().build().execute_with(|| {
		// Unhappy invalid NFT Id
		assert_eq!(NFTs::is_nft_in_completed_series(1001), None);
	})
}
