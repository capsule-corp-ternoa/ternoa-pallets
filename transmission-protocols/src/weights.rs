// Copyright 2023 Capsule Corp (France) SAS.
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

use frame_support::weights::Weight;

pub trait WeightInfo {
	fn set_transmission_protocol() -> Weight;
	fn remove_transmission_protocol() -> Weight;
	fn reset_timer() -> Weight;
	fn add_consent() -> Weight;
	fn set_protocol_fee() -> Weight;
}

impl WeightInfo for () {
	fn set_transmission_protocol() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn remove_transmission_protocol() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn reset_timer() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn add_consent() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
	fn set_protocol_fee() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
	}
}
