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

/// Enumeration of Transmission protocols kind.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
pub enum ClusterType {
	Disabled,
	Admin,
	Public,
	Private,
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
	pub enclaves: BoundedVec<(AccountId, SlotId), ClusterSize>,
	pub cluster_type: ClusterType,
}

impl<AccountId, ClusterSize> Cluster<AccountId, ClusterSize>
where
	AccountId: Clone + PartialEq + Debug,
	ClusterSize: Get<u32>,
{
	pub fn new(
		enclaves: BoundedVec<(AccountId, SlotId), ClusterSize>,
		cluster_type: ClusterType,
	) -> Self {
		Self { enclaves, cluster_type }
	}
}

/// The ledger of a (bonded) operator.
#[derive(
	PartialEqNoBound, CloneNoBound, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound(AccountId: MaxEncodedLen, BlockNumber: MaxEncodedLen, Balance: MaxEncodedLen))]
pub struct TeeStakingLedger<AccountId, BlockNumber, Balance>
where
	AccountId: Clone + PartialEq + Debug,
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
{
	/// The operator account whose balance is actually locked and at stake.
	pub operator: AccountId,
	/// The total staked amount
	pub staked_amount: Balance,
	/// State variable to know whether the staked amount is unbonded
	pub is_unlocking: bool,
	/// Block Number of when unbonded happened
	pub unbonded_at: BlockNumber,
}

impl<AccountId, BlockNumber, Balance> TeeStakingLedger<AccountId, BlockNumber, Balance>
where
	AccountId: Clone + PartialEq + Debug,
	BlockNumber: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
	Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
{
	pub fn new(
		operator: AccountId,
		staked_amount: Balance,
		is_unlocking: bool,
		unbonded_at: BlockNumber,
	) -> Self {
		Self { operator, staked_amount, is_unlocking, unbonded_at }
	}
}
// #[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]

#[derive(
	Encode,
	Decode,
	CloneNoBound,
	PartialEqNoBound,
	Eq,
	RuntimeDebugNoBound,
	TypeInfo,
	MaxEncodedLen,
	Default,
)]
#[scale_info(skip_type_params(ListSizeLimit))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct MetricsServerReport<AccountId>
where
	AccountId: Clone + PartialEq + Debug,
{
	pub param_1: u8,
	pub param_2: u8,
	pub param_3: u8,
	pub param_4: u8,
	pub param_5: u8,
	pub submitted_by: AccountId,
}

/// Report Parameters weightage
#[derive(
	PartialEqNoBound, CloneNoBound, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
pub struct ReportParamsWeightage {
	pub param_1_weightage: u8,
	pub param_2_weightage: u8,
	pub param_3_weightage: u8,
	pub param_4_weightage: u8,
	pub param_5_weightage: u8,
}

impl Default for ReportParamsWeightage {
	fn default() -> Self {
		Self {
			param_1_weightage: 0,
			param_2_weightage: 0,
			param_3_weightage: 0,
			param_4_weightage: 0,
			param_5_weightage: 0,
		}
	}
}

/// Report Parameters weightage
#[derive(
	PartialEqNoBound, CloneNoBound, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
pub struct HighestParamsResponse {
	pub param_1: u8,
	pub param_2: u8,
	pub param_3: u8,
	pub param_4: u8,
	pub param_5: u8,
}

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, Eq, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub struct MetricsServer<AccountId>
where
	AccountId: Clone + PartialEq + Debug,
{
	pub metrics_server_address: AccountId,
	pub supported_cluster_type: ClusterType,
}

impl<AccountId> MetricsServer<AccountId>
where
	AccountId: Clone + PartialEq + Debug,
{
	pub fn new(metrics_server_address: AccountId, supported_cluster_type: ClusterType) -> Self {
		Self { metrics_server_address, supported_cluster_type }
	}
}
