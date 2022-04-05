use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

pub trait WeightInfo {
	fn transfer_hash() -> Weight;
	fn transfer_native() -> Weight;
	fn transfer_erc721() -> Weight;
	fn transfer() -> Weight;
	fn remark() -> Weight;
	fn mint_erc721() -> Weight;
	fn set_bridge_fee() -> Weight;
}

impl WeightInfo for () {
	fn transfer_hash() -> Weight {
		195_000_000 as Weight
	}

	fn transfer_native() -> Weight {
		195_000_000 as Weight
	}

	fn transfer_erc721() -> Weight {
		195_000_000 as Weight
	}

	fn transfer() -> Weight {
		195_000_000 as Weight
	}

	fn remark() -> Weight {
		195_000_000 as Weight
	}

	fn mint_erc721() -> Weight {
		195_000_000 as Weight
	}

	fn set_bridge_fee() -> Weight {
		(10_100_000 as Weight).saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
