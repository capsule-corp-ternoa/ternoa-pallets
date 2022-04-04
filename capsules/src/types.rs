use frame_support::{traits::Get, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use primitives::nfts::NFTId;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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

pub type CapsuleLedger<Balance, CapsuleCountLimit> =
	BoundedVec<(NFTId, Balance), CapsuleCountLimit>;
