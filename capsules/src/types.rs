use frame_support::{traits::Get, BoundedVec};
use parity_scale_codec::{Decode, Encode};
use primitives::nfts::NFTId;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct CapsuleData<AccountId, IPFSLengthLimit>
where
	AccountId: Clone,
	IPFSLengthLimit: Get<u32>,
{
	pub owner: AccountId,
	pub ipfs_reference: BoundedVec<u8, IPFSLengthLimit>,
}

impl<AccountId, IPFSLengthLimit> CapsuleData<AccountId, IPFSLengthLimit>
where
	AccountId: Clone,
	IPFSLengthLimit: Get<u32>,
{
	pub fn new(
		owner: AccountId,
		ipfs_reference: BoundedVec<u8, IPFSLengthLimit>,
	) -> CapsuleData<AccountId, IPFSLengthLimit> {
		Self { owner, ipfs_reference }
	}
}

pub type CapsuleLedger<Balance> = Vec<(NFTId, Balance)>;
