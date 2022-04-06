use frame_support::{traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use primitives::nfts::NFTId;
use scale_info::TypeInfo;
use sp_std::fmt::Debug;

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, Eq, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
#[scale_info(skip_type_params(IPFSLengthLimit))]
pub struct CapsuleData<AccountId, IPFSLengthLimit>
where
	AccountId: Clone + PartialEq + Debug,
	IPFSLengthLimit: Get<u32>,
{
	pub owner: AccountId,
	pub ipfs_reference: BoundedVec<u8, IPFSLengthLimit>,
}

impl<AccountId, IPFSLengthLimit> CapsuleData<AccountId, IPFSLengthLimit>
where
	AccountId: Clone + PartialEq + Debug,
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
