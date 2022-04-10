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
use crate::{CapsuleData, CapsuleIPFSReference, GenesisConfig};
use frame_support::{bounded_vec, traits::GenesisBuild};

#[test]
fn register_capsules() {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mint_fee = 1000;
	let nft_id = 1;
	let owner = ALICE;
	let reference: CapsuleIPFSReference<Test> = bounded_vec![20];

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
