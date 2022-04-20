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

use frame_support::{traits::Get, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub type ChainId = u8;
pub type DepositNonce = u64;

/// Enumeration of proposal status.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProposalStatus {
	Initiated,
	Approved,
	Rejected,
}

/// Proposal votes data structure.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[codec(mel_bound(AccountId: MaxEncodedLen, BlockNumber: MaxEncodedLen))]
#[scale_info(skip_type_params(VoteCountLimit))]
pub struct Proposal<AccountId, BlockNumber, VoteCountLimit>
where
	VoteCountLimit: Get<u32>,
{
	pub votes: BoundedVec<(AccountId, bool), VoteCountLimit>,
	pub status: ProposalStatus,
	pub expiry: BlockNumber,
}

impl<AccountId, BlockNumber, VoteCountLimit> Proposal<AccountId, BlockNumber, VoteCountLimit>
where
	AccountId: PartialEq,
	BlockNumber: PartialOrd + Default,
	VoteCountLimit: Get<u32>,
{
	pub fn new(
		initial_votes: BoundedVec<(AccountId, bool), VoteCountLimit>,
		block_expiry: BlockNumber,
	) -> Self {
		Self { votes: initial_votes, status: ProposalStatus::Initiated, expiry: block_expiry }
	}

	/// Attempts to mark the proposal as approve or rejected.
	/// Returns true if the status changes from active.
	/// TODO!
	pub fn try_to_complete(&mut self, threshold: u32) -> Option<ProposalStatus> {
		let for_count = self.votes.iter().filter(|x| x.1 == true).count() as u32;
		let against_count = self.votes.iter().count() as u32 - for_count;

		if for_count >= threshold {
			self.status = ProposalStatus::Approved;
			return Some(ProposalStatus::Approved)
		}
		if against_count >= threshold {
			self.status = ProposalStatus::Rejected;
			return Some(ProposalStatus::Rejected)
		}

		None
	}

	/// Returns true if the proposal has been rejected or approved, otherwise false.
	pub fn is_complete(&self) -> bool {
		self.status != ProposalStatus::Initiated
	}

	/// Returns true if `who` has voted for or against the proposal
	pub fn has_voted(&self, who: &AccountId) -> bool {
		self.votes.iter().find(|x| x.0 == *who).is_some()
	}

	/// Return true if the expiry time has been reached
	pub fn is_expired(&self, now: BlockNumber) -> bool {
		self.expiry <= now
	}
}
