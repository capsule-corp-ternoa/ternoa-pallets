use super::mock::*;
use crate::{CapsuleData, GenesisConfig};
use frame_support::{bounded_vec, traits::GenesisBuild, BoundedVec};

#[test]
fn register_capsules() {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mint_fee = 1000;
	let nft_id = 1;
	let owner = ALICE;
	let reference: BoundedVec<u8, IPFSLengthLimit> = bounded_vec![20];

	let data = CapsuleData::new(owner, reference.clone());
	let ledger = vec![(nft_id, mint_fee)];

	GenesisConfig::<Test> {
		capsule_mint_fee: mint_fee,
		capsules: vec![(nft_id, owner, reference.to_vec())],
		ledgers: vec![(owner, ledger.clone())],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let ledger = bounded_vec![(nft_id, mint_fee)];
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		assert_eq!(TernoaCapsules::ledgers(owner), Some(ledger));
		assert_eq!(TernoaCapsules::capsules(nft_id), Some(data));
		assert_eq!(TernoaCapsules::capsule_mint_fee(), mint_fee);
	});
}
