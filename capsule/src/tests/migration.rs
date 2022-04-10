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
use frame_support::traits::OnRuntimeUpgrade;

mod version_1 {
	use super::*;

	frame_support::generate_storage_alias!(
		TernoaCapsules, CapsuleMintFee => Value<()>
	);

	#[test]
	fn set_to_version_1() {
		ExtBuilder::default().build().execute_with(|| {
			CapsuleMintFee::kill();

			let weight = <TernoaCapsules as OnRuntimeUpgrade>::on_runtime_upgrade();
			let mint_fee = TernoaCapsules::capsule_mint_fee();

			// Check NFT mint fee
			assert_eq!(weight, 1);
			assert_eq!(mint_fee, 1000000000000000000000);
		})
	}
}

#[test]
fn upgrade_from_latest_to_latest() {
	ExtBuilder::default().build().execute_with(|| {
		let weight = <TernoaCapsules as OnRuntimeUpgrade>::on_runtime_upgrade();
		assert_eq!(weight, 0);
	})
}
