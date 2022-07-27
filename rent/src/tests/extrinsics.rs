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
use frame_support::{assert_noop, assert_ok, error::BadOrigin, BoundedVec};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use primitives::{
	nfts::{NFTId, NFTState, NFTData},
};
use sp_arithmetic::per_things::Permill;
use ternoa_common::traits::NFTExt;

use crate::{
	tests::mock, Error, Event as RentEvent, 
};

const ALICE_NFT_ID_0: NFTId = 0;
const ALICE_NFT_ID_1: NFTId = 1;
const BOB_NFT_ID_0: NFTId = 2;
const BOB_NFT_ID_1: NFTId = 3;
const CHARLIE_NFT_ID_0: NFTId = 4;
const CHARLIE_NFT_ID_1: NFTId = 5;
const PERCENT_0: Permill = Permill::from_parts(0);

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

fn prepare_tests() {
	let alice: mock::Origin = origin(ALICE);
	let bob: mock::Origin = origin(BOB);
	let charlie: mock::Origin = origin(CHARLIE);

	//Create alice NFT.
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(alice.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(bob.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(bob.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(charlie.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();
	NFT::create_nft(charlie.clone(), BoundedVec::default(), PERCENT_0, None, false).unwrap();


	assert_eq!(NFT::nfts(ALICE_NFT_ID_0).is_some(), true);
	assert_eq!(NFT::nfts(ALICE_NFT_ID_1).is_some(), true);
	assert_eq!(NFT::nfts(BOB_NFT_ID_0).is_some(), true);
	assert_eq!(NFT::nfts(BOB_NFT_ID_1).is_some(), true);
	assert_eq!(NFT::nfts(CHARLIE_NFT_ID_0).is_some(), true);
	assert_eq!(NFT::nfts(CHARLIE_NFT_ID_1).is_some(), true);
}

mod create_marketplace {
	use super::*;

	#[test]
	fn create_contract() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			assert_eq!(true, true);
		})
	}
}