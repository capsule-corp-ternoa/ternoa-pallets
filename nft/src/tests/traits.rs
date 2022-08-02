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
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use primitives::nfts::NFTState;
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::NFTExt;

use crate::tests::mock;

const PERCENT_0: Permill = Permill::from_parts(0);

#[test]
fn set_nft_state() {
	ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
		let alice: Origin = RawOrigin::Signed(ALICE).into();
		NFT::create_nft(alice, BoundedVec::default(), PERCENT_0, None, false).unwrap();
		let nft_id = mock::NFT::get_next_nft_id() - 1;
		let nft_state = NFTState::new(true, true, true, true, true, false, false);
		<NFT as NFTExt>::set_nft_state(nft_id, nft_state.clone()).unwrap();
		let nft = NFT::nfts(nft_id).unwrap();
		assert_eq!(nft.state, nft_state);
	})
}

#[test]
fn create_filled_collection() {
	ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
		<NFT as NFTExt>::create_filled_collection(ALICE, 0, 0, CollectionSizeLimit::get()).unwrap();
		let collection = NFT::collections(0).unwrap();
		assert_eq!(collection.owner, ALICE);
		assert_eq!(collection.nfts.len(), CollectionSizeLimit::get() as usize);
	})
}

#[test]
fn get_nft() {
	ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
		let alice: Origin = RawOrigin::Signed(ALICE).into();
		NFT::create_nft(alice, BoundedVec::default(), PERCENT_0, None, false).unwrap();
		let nft_id = mock::NFT::get_next_nft_id() - 1;
		let nft = NFT::get_nft(nft_id).unwrap();
		assert_eq!(nft.owner, ALICE);
		let invalid_id = 999;
		let no_nft = NFT::get_nft(invalid_id);
		assert_eq!(no_nft, None);
	})
}

#[test]
fn set_nft() {
	ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
		let alice: Origin = RawOrigin::Signed(ALICE).into();
		NFT::create_nft(alice, BoundedVec::default(), PERCENT_0, None, false).unwrap();
		let nft_id = mock::NFT::get_next_nft_id() - 1;
		let nft = NFT::get_nft(nft_id).unwrap();
		let mut nft_data = nft.clone();
		nft_data.owner = BOB;
		NFT::set_nft(nft_id, nft_data).unwrap();
		let nft = NFT::get_nft(nft_id).unwrap();
		assert_eq!(nft.owner, BOB);
	})
}

#[test]
fn create_nft() {
	ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
		let nft_id = <NFT as NFTExt>::create_nft(ALICE, BoundedVec::default(), PERCENT_0, None, false).unwrap();
		let nft = NFT::get_nft(nft_id).unwrap();
		assert_eq!(nft.owner, ALICE);
	})
}
