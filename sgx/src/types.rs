use parity_scale_codec::{Decode, Encode};
use primitives::TextFormat;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

pub type EnclaveId = u32;
pub type ClusterId = u32;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Enclave {
	pub api_uri: TextFormat,
}

impl Enclave {
	pub fn new(api_uri: TextFormat) -> Self {
		Self { api_uri }
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
