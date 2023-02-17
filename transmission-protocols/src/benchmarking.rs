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
// #![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as TransmissionProtocols;
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	traits::{Currency, Get},
	BoundedVec,
};
use frame_system::RawOrigin;

use sp_arithmetic::Permill;
use sp_runtime::traits::Bounded;
use sp_std::prelude::*;

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}
pub fn origin<T: Config>(name: &'static str) -> RawOrigin<T::AccountId> {
	RawOrigin::Signed(get_account::<T>(name))
}

pub struct BenchmarkData {
	pub alice_nft_id: NFTId,
	pub bob_nft_id: NFTId,
}

const PERCENT_0: Permill = Permill::from_parts(0);

pub fn prepare_benchmarks<T: Config>() -> BenchmarkData {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value());

	// Create NFTs
	let alice_nft_id =
		T::NFTExt::create_nft(alice, BoundedVec::default(), PERCENT_0, None, false).unwrap();
	let bob_nft_id =
		T::NFTExt::create_nft(bob, BoundedVec::default(), PERCENT_0, None, false).unwrap();
	BenchmarkData { alice_nft_id, bob_nft_id }
}

benchmarks! {
	set_transmission_protocol {
		let s in 0 .. T::SimultaneousTransmissionLimit::get() - 1;
		let benchmark_data = prepare_benchmarks::<T>();
		let alice_origin = origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let protocol:TransmissionProtocol<
			T::BlockNumber,
			ConsentList<T::AccountId, T::MaxConsentListSize>,
		> = TransmissionProtocol::AtBlock(10u32.into());
		let cancellation = CancellationPeriod::None;
		TransmissionProtocols::<T>::fill_queue(s, benchmark_data.bob_nft_id, 100u32.into()).unwrap();
	}: _(alice_origin, benchmark_data.alice_nft_id, bob, protocol, cancellation)
	verify {
		let nft = T::NFTExt::get_nft(benchmark_data.alice_nft_id).unwrap();
		assert!(nft.state.is_transmission);
		assert!(TransmissionProtocols::<T>::transmissions(benchmark_data.alice_nft_id).is_some());
		assert_eq!(TransmissionProtocols::<T>::at_block_queue().get(benchmark_data.alice_nft_id), Some(10u32.into()));
	}

	remove_transmission_protocol {
		let s in 0 .. T::SimultaneousTransmissionLimit::get() - 1;
		let benchmark_data = prepare_benchmarks::<T>();
		let alice_origin = origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let protocol:TransmissionProtocol<
			T::BlockNumber,
			ConsentList<T::AccountId, T::MaxConsentListSize>,
		> = TransmissionProtocol::AtBlock(10u32.into());
		let cancellation = CancellationPeriod::UntilBlock(20u32.into());
		TransmissionProtocols::<T>::fill_queue(s, benchmark_data.bob_nft_id, 100u32.into()).unwrap();
		TransmissionProtocols::<T>::set_transmission_protocol(alice_origin.clone().into(), benchmark_data.alice_nft_id, bob, protocol, cancellation).unwrap();
	}: _(alice_origin, benchmark_data.alice_nft_id)
	verify {
		let nft = T::NFTExt::get_nft(benchmark_data.alice_nft_id).unwrap();
		assert!(!nft.state.is_transmission);
		assert!(TransmissionProtocols::<T>::transmissions(benchmark_data.alice_nft_id).is_none());
		assert_eq!(TransmissionProtocols::<T>::at_block_queue().get(benchmark_data.alice_nft_id), None);
	}

	reset_timer {
		let s in 0 .. T::SimultaneousTransmissionLimit::get() - 1;
		let benchmark_data = prepare_benchmarks::<T>();
		let alice_origin = origin::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let protocol:TransmissionProtocol<
			T::BlockNumber,
			ConsentList<T::AccountId, T::MaxConsentListSize>,
		> = TransmissionProtocol::AtBlockWithReset(10u32.into());
		let cancellation = CancellationPeriod::None;
		TransmissionProtocols::<T>::fill_queue(s, benchmark_data.bob_nft_id, 100u32.into()).unwrap();
		TransmissionProtocols::<T>::set_transmission_protocol(alice_origin.clone().into(), benchmark_data.alice_nft_id, bob, protocol, cancellation).unwrap();
	}: _(alice_origin, benchmark_data.alice_nft_id, 200u32.into())
	verify {
		let nft = T::NFTExt::get_nft(benchmark_data.alice_nft_id).unwrap();
		assert!(nft.state.is_transmission);
		assert!(TransmissionProtocols::<T>::transmissions(benchmark_data.alice_nft_id).is_some());
		assert_eq!(TransmissionProtocols::<T>::at_block_queue().get(benchmark_data.alice_nft_id), Some(200u32.into()));
	}

	add_consent {
		let s in 0 .. T::SimultaneousTransmissionLimit::get() - 1;
		let benchmark_data = prepare_benchmarks::<T>();
		let alice_origin = origin::<T>("ALICE");
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");
		let charlie: T::AccountId = get_account::<T>("CHARLIE");
		let consent_list = BoundedVec::try_from(vec![alice.clone(), bob.clone(), charlie]).unwrap();
		let protocol:TransmissionProtocol<
			T::BlockNumber,
			ConsentList<T::AccountId, T::MaxConsentListSize>,
		> = TransmissionProtocol::OnConsentAtBlock {
			consent_list,
			threshold: 2u8,
			block: 10u32.into(),
		};
		let cancellation = CancellationPeriod::None;
		TransmissionProtocols::<T>::fill_queue(s, benchmark_data.bob_nft_id, 100u32.into()).unwrap();
		TransmissionProtocols::<T>::set_transmission_protocol(alice_origin.clone().into(), benchmark_data.alice_nft_id, bob.clone(), protocol, cancellation).unwrap();
		TransmissionProtocols::<T>::fill_consent_list(1, benchmark_data.alice_nft_id, bob).unwrap();
	}: _(alice_origin, benchmark_data.alice_nft_id)
	verify {
		let nft = T::NFTExt::get_nft(benchmark_data.alice_nft_id).unwrap();
		assert!(nft.state.is_transmission);
		assert!(TransmissionProtocols::<T>::transmissions(benchmark_data.alice_nft_id).is_some());
		assert_eq!(TransmissionProtocols::<T>::at_block_queue().get(benchmark_data.alice_nft_id), Some(10u32.into()));
	}

	set_protocol_fee {
		let old_fee = TransmissionProtocols::<T>::at_block_fee();
		let new_fee = 20u32;
	}: _(RawOrigin::Root, TransmissionProtocolKind::AtBlock, new_fee.into())
	verify {
		assert_ne!(old_fee, new_fee.into());
		assert_eq!(TransmissionProtocols::<T>::at_block_fee(), new_fee.into());
	}
}

impl_benchmark_test_suite!(
	TransmissionProtocols,
	crate::tests::mock::new_test_ext(),
	crate::tests::mock::Test
);
