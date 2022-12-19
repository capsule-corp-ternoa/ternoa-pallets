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

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

pub type EnclaveId = u32;
pub type ClusterId = u32;
pub type ProviderId = u32;
pub type EnclaveOperatorId = u32;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default)]
pub struct EnclaveProvider {
	pub enclave_provider_name: Vec<u8>,
}

impl EnclaveProvider {
	pub fn new(enclave_provider_name: Vec<u8>) -> Self {
		Self { enclave_provider_name }
	}
}

// account_id | enclave_id | enclave_class | public_key
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default)]
pub struct EnclaveProviderKeys<AccountId> {
	pub enclave_class: Option<Vec<u8>>,
	pub account_id: AccountId,
	pub public_key: Vec<u8>,
}

impl<AccountId> EnclaveProviderKeys<AccountId> {
	pub fn new(enclave_class: Option<Vec<u8>>, account_id: AccountId, public_key: Vec<u8>) -> Self {
		Self { enclave_class, account_id, public_key }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Enclave {
	pub api_uri: Vec<u8>,
	pub enclave_address: Vec<u8>,
}

impl Enclave {
	pub fn new(api_uri: Vec<u8>, enclave_address: Vec<u8>) -> Self {
		Self { api_uri, enclave_address }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Cluster {
	pub enclaves: Vec<EnclaveId>,
}

impl Cluster {
	pub fn new(enclaves: Vec<EnclaveId>) -> Self {
		Self { enclaves }
	}
}
