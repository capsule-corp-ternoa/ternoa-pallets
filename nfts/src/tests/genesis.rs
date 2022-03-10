use super::mock::*;
use crate::{GenesisConfig, NFTData};
use frame_support::traits::GenesisBuild;
use primitives::nfts::NFTsGenesis;

#[test]
fn register_nfts() {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let nft_id = 100;
	let mint_fee = 10;
	let data_original = NFTData::new_default(ALICE, vec![1], vec![48], 0);
	let data = data_original.clone();

	let genesis_nfts: NFTsGenesis<u64> = (
		nft_id,
		data.owner,
		data.creator,
		data.ipfs_reference,
		data.series_id,
		data.listed_for_sale,
		data.is_in_transmission,
		data.is_capsule,
		data.is_secret,
		data.is_delegated,
		data.royaltie_fee,
	);

	GenesisConfig::<Test> { nfts: vec![genesis_nfts], series: vec![], nft_mint_fee: mint_fee }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		assert_eq!(NFTs::nft_id_generator(), nft_id + 1);
		assert_eq!(NFTs::series_id_generator(), 0);
		assert_eq!(NFTs::data(nft_id), Some(data_original));
		assert_eq!(NFTs::nft_mint_fee(), mint_fee);
	});
}
