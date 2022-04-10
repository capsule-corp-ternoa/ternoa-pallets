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
use crate::{GenesisConfig, NFTData};
use frame_support::{bounded_vec, traits::GenesisBuild};
use primitives::nfts::NFTsGenesis;

#[test]
fn register_nfts() {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let nft_id = 100;
	let mint_fee = 10;
	let data_original = NFTData::new_default(ALICE, bounded_vec![1], vec![48]);
	let data = data_original.clone();

	let genesis_nfts: NFTsGenesis<u64> = (
		nft_id,
		data.owner,
		data.creator,
		data.ipfs_reference.to_vec(),
		data.series_id,
		data.listed_for_sale,
		data.is_in_transmission,
		data.is_capsule,
		data.is_secret,
		data.is_delegated,
		data.royalties,
	);

	GenesisConfig::<Test> { nfts: vec![genesis_nfts], series: vec![], nft_mint_fee: mint_fee }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		assert_eq!(NFT::nft_id_generator(), nft_id + 1);
		assert_eq!(NFT::series_id_generator(), 0);
		assert_eq!(NFT::data(nft_id), Some(data_original));
		assert_eq!(NFT::nft_mint_fee(), mint_fee);
	});
}
