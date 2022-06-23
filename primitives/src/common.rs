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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::Codec, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::per_things::Permill;
use sp_runtime::RuntimeDebug;

pub type U8BoundedVec<S> = BoundedVec<u8, S>;

/// Possible operations on the configuration values of this pallet.
#[derive(TypeInfo, Debug, Clone, Encode, Decode, PartialEq)]
pub enum ConfigOp<T: Codec> {
	/// Don't change.
	Noop,
	/// Set the given value.
	Set(T),
	/// Remove the value.
	Remove,
}

/// Multiple form of fees
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum CompoundFee<Balance> {
	Flat(Balance),
	Percentage(Permill),
}
