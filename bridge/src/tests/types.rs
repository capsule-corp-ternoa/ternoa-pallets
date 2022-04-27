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

use super::mock::*;
use frame_support::bounded_vec;

use crate::{
	tests::mock::{ExtBuilder, ProposalLifetime, RELAYER_A, RELAYER_B},
	types::{Proposal, ProposalStatus},
};

pub mod proposal {
	pub use super::*;

	#[test]
	fn try_to_complete_approved() {
		ExtBuilder::build().execute_with(|| {
			let mut prop: Proposal<_, _, RelayerCountLimit> = Proposal {
				votes: bounded_vec![(RELAYER_A, true)],
				status: ProposalStatus::Initiated,
				expiry: ProposalLifetime::get(),
			};

			prop.try_to_complete(1);
			assert_eq!(prop.status, ProposalStatus::Approved);
		});
	}

	#[test]
	fn try_to_complete_bad_threshold() {
		ExtBuilder::build().execute_with(|| {
			let mut prop: Proposal<_, _, RelayerCountLimit> = Proposal {
				votes: bounded_vec![(RELAYER_A, true), (RELAYER_B, true)],
				status: ProposalStatus::Initiated,
				expiry: ProposalLifetime::get(),
			};

			prop.try_to_complete(3);
			assert_eq!(prop.status, ProposalStatus::Initiated);
		});
	}
}
