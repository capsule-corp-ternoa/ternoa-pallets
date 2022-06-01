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
		<NFT as NFTExt>::set_nft_state(nft_id, true, true, true, true, true).unwrap();
		assert_eq!(NFT::nfts(nft_id).unwrap().state.is_capsule, true);
		assert_eq!(NFT::nfts(nft_id).unwrap().state.listed_for_sale, true);
		assert_eq!(NFT::nfts(nft_id).unwrap().state.is_secret, true);
		assert_eq!(NFT::nfts(nft_id).unwrap().state.is_delegated, true);
		assert_eq!(NFT::nfts(nft_id).unwrap().state.is_soulbound, true);
	})
}
