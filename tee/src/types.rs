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
use primitives::tee::SlotId;
use scale_info::TypeInfo;
use sp_arithmetic::traits::AtLeast32BitUnsigned;
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
	pub is_public: bool,
}

impl<AccountId, ClusterSize> Cluster<AccountId, ClusterSize>
where
	AccountId: Clone + PartialEq + Debug,
	ClusterSize: Get<u32>,
{
	pub fn new(enclaves: BoundedVec<AccountId, ClusterSize>, is_public: bool) -> Self {
		Self { enclaves, is_public }
	}
}

/// The ledger of a (bonded) operator.
#[derive(
	PartialEqNoBound, CloneNoBound, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound(AccountId: MaxEncodedLen, BlockNumber: MaxEncodedLen))]
pub struct TeeStakingLedger<AccountId, BlockNumber>
where
	AccountId: Clone + PartialEq + Debug,
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
{
	/// The operator account whose balance is actually locked and at stake.
	pub operator: AccountId,
	/// State variable to know whether the staked amount is unbonded
	pub is_unlocking: bool,
	/// Block Number of when unbonded happened
	pub unbonded_at: BlockNumber,
}

impl<AccountId, BlockNumber> TeeStakingLedger<AccountId, BlockNumber>
where
	AccountId: Clone + PartialEq + Debug,
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
{
	pub fn new(operator: AccountId, is_unlocking: bool, unbonded_at: BlockNumber) -> Self {
		Self { operator, is_unlocking, unbonded_at }
	}
}
