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

use frame_support::{traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::fmt::Debug;

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, Eq, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(MaxUriLen))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct Enclave<AccountId, MaxUriLen>
where
	AccountId: Clone + PartialEq + Debug,
	MaxUriLen: Get<u32>,
{
	pub enclave_address: AccountId,
	pub api_uri: BoundedVec<u8, MaxUriLen>,
}

impl<AccountId, MaxUriLen> Enclave<AccountId, MaxUriLen>
where
	AccountId: Clone + PartialEq + Debug,
	MaxUriLen: Get<u32>,
{
	pub fn new(enclave_address: AccountId, api_uri: BoundedVec<u8, MaxUriLen>) -> Self {
		Self { enclave_address, api_uri }
	}
}

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, Eq, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(ClusterSize))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct Cluster<AccountId, ClusterSize>
where
	AccountId: Clone + PartialEq + Debug,
	ClusterSize: Get<u32>,
{
	pub enclaves: BoundedVec<AccountId, ClusterSize>,
}

impl<AccountId, ClusterSize> Cluster<AccountId, ClusterSize>
where
	AccountId: Clone + PartialEq + Debug,
	ClusterSize: Get<u32>,
{
	pub fn new(enclaves: BoundedVec<AccountId, ClusterSize>) -> Self {
		Self { enclaves }
	}
}
