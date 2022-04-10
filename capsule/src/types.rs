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

use frame_support::{traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use primitives::nfts::{IPFSReference, NFTId};
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
	pub ipfs_reference: IPFSReference<IPFSLengthLimit>,
}

impl<AccountId, IPFSLengthLimit> CapsuleData<AccountId, IPFSLengthLimit>
where
	AccountId: Clone + PartialEq + Debug,
	IPFSLengthLimit: Get<u32>,
{
	pub fn new(
		owner: AccountId,
		ipfs_reference: IPFSReference<IPFSLengthLimit>,
	) -> CapsuleData<AccountId, IPFSLengthLimit> {
		Self { owner, ipfs_reference }
	}
}

pub type CapsuleLedger<Balance, CapsuleCountLimit> =
	BoundedVec<(NFTId, Balance), CapsuleCountLimit>;
