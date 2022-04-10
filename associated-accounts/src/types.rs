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

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use parity_scale_codec::{Decode, Encode};
use primitives::TextFormat;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SupportedAccount {
	pub key: TextFormat,
	pub min_length: u16,
	pub max_length: u16,
	pub initial_set_fee: bool,
}

impl SupportedAccount {
	pub fn new(key: TextFormat, min_length: u16, max_length: u16, initial_set_fee: bool) -> Self {
		Self { key, min_length, max_length, initial_set_fee }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Account {
	pub key: TextFormat,
	pub value: TextFormat,
}

impl Account {
	pub fn new(key: TextFormat, value: TextFormat) -> Self {
		Self { key, value }
	}
}
