#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode};
use primitives::{nfts::NFTId, TextFormat};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CapsuleData<AccountId>
where
	AccountId: Clone,
{
	pub owner: AccountId,
	pub ipfs_reference: TextFormat,
}

impl<AccountId> CapsuleData<AccountId>
where
	AccountId: Clone,
{
	pub fn new(owner: AccountId, ipfs_reference: TextFormat) -> CapsuleData<AccountId> {
		Self { owner, ipfs_reference }
	}
}

pub type CapsuleLedger<Balance> = Vec<(NFTId, Balance)>;
