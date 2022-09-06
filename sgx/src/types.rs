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

use frame_support::{pallet_prelude::Get, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub type EnclaveId = u32;
pub type ClusterId = u32;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(APIURILegnthLimit))]
pub struct Enclave<APIURILegnthLimit>
where
	APIURILegnthLimit: Get<u32>,
{
	pub api_uri: BoundedVec<u8, APIURILegnthLimit>,
}

impl<APIURILegnthLimit> Enclave<APIURILegnthLimit>
where
	APIURILegnthLimit: Get<u32>,
{
	pub fn new(api_uri: BoundedVec<u8, APIURILegnthLimit>) -> Self {
		Self { api_uri }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxEnclaveLimit))]
pub struct Cluster<MaxEnclaveLimit>
where
	MaxEnclaveLimit: Get<u32>,
{
	pub enclaves: BoundedVec<ClusterId, MaxEnclaveLimit>,
}

impl<MaxEnclaveLimit> Cluster<MaxEnclaveLimit>
where
	MaxEnclaveLimit: Get<u32>,
{
	pub fn new(enclaves: BoundedVec<ClusterId, MaxEnclaveLimit>) -> Self {
		Self { enclaves }
	}
}
