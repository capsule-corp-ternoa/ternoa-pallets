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
use crate::Pallet as TEE;
use frame_benchmarking::{account as benchmark_account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	traits::{Currency, Get},
	BoundedVec,
};
use frame_system::RawOrigin;

use sp_runtime::traits::Bounded;
use sp_std::prelude::*;

use parity_scale_codec::{Decode, Encode};

pub fn get_account<T: Config>(name: &'static str) -> T::AccountId {
	let account: T::AccountId = benchmark_account(name, 0, 0);
	account
}
pub fn origin<T: Config>(name: &'static str) -> RawOrigin<T::AccountId> {
	RawOrigin::Signed(get_account::<T>(name))
}

pub fn prepare_benchmarks<T: Config>() {
	let alice: T::AccountId = get_account::<T>("ALICE");
	let bob: T::AccountId = get_account::<T>("BOB");

	T::Currency::make_free_balance_be(&alice, BalanceOf::<T>::max_value() / 5u32.into());
	T::Currency::make_free_balance_be(&bob, BalanceOf::<T>::max_value() / 5u32.into());

	let auction_account = TEE::<T>::account_id();
	T::Currency::make_free_balance_be(&auction_account, BalanceOf::<T>::max_value() / 5u32.into());
}

benchmarks! {
	register_enclave {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let enclave_address: T::AccountId = get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

	}: _(origin::<T>("ALICE"), enclave_address.clone(), uri)
	verify {
		assert_eq!(EnclaveRegistrations::<T>::get(alice), Some(enclave));
	}

	unregister_enclave {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		let raw = (4 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();
	}: _(origin::<T>("ALICE"))

	update_enclave {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		let raw = (4 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();

		let bob: T::AccountId = get_account::<T>("BOB");
		let new_enclave_address: T::AccountId= get_account::<T>("BOB");
		let new_uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let new_enclave = Enclave::new(new_enclave_address.clone(), new_uri.clone());
	}: _(origin::<T>("ALICE"), new_enclave_address.clone(), new_uri)
	verify {
		assert_eq!(EnclaveUpdates::<T>::get(alice), Some(new_enclave));
	}

	cancel_update {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let new_enclave_address: T::AccountId= get_account::<T>("BOB");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		let raw = (4 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();
		TEE::<T>::update_enclave(origin::<T>("ALICE").into(), new_enclave_address.clone(), uri.clone()).unwrap();

	}: _(origin::<T>("ALICE"))
	verify {
		assert_eq!(EnclaveUpdates::<T>::get(alice), None);
	}

	assign_enclave {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		let raw = (4 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();

	}: _(RawOrigin::Root, alice.clone(), cluster_id, slot_id)
	verify {
		assert_eq!(EnclaveAccountOperator::<T>::get(enclave_address), Some(alice.clone()));
		assert_eq!(EnclaveData::<T>::get(alice.clone()), Some(enclave));
		assert_eq!(EnclaveClusterId::<T>::get(alice.clone()), Some(cluster_id));
		assert_eq!(EnclaveRegistrations::<T>::get(alice), None);
	}

	remove_registration {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
	}: _(RawOrigin::Root, alice.clone())
	verify {
		assert_eq!(EnclaveRegistrations::<T>::get(alice), None);
	}

	reject_update {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		let raw = (4 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();

		let bob: T::AccountId = get_account::<T>("BOB");
		let new_enclave_address: T::AccountId= get_account::<T>("BOB");
		let new_uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let new_enclave = Enclave::new(new_enclave_address.clone(), new_uri.clone());
		TEE::<T>::update_enclave(origin::<T>("ALICE").into(), new_enclave_address.clone(), new_uri).unwrap();
	}: _(RawOrigin::Root, alice.clone())
	verify {
		assert_eq!(EnclaveUpdates::<T>::get(alice), None);
	}

	force_remove_enclave {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		let raw = (4 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();
	}: _(RawOrigin::Root, alice.clone())
	verify {
		assert_eq!(EnclaveAccountOperator::<T>::get(enclave_address), None);
		assert_eq!(EnclaveData::<T>::get(alice.clone()), None);
		assert_eq!(EnclaveClusterId::<T>::get(alice.clone()), None);
		assert_eq!(ClusterData::<T>::get(cluster_id).unwrap().enclaves, vec![]);
		assert_eq!(EnclaveRegistrations::<T>::get(alice), None);
	}

	force_update_enclave {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		let raw = (4 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();

		let new_enclave_address: T::AccountId= get_account::<T>("BOB");
		let new_uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let new_enclave = Enclave::new(new_enclave_address.clone(), new_uri.clone());
		TEE::<T>::update_enclave(origin::<T>("ALICE").into(), new_enclave_address.clone(), new_uri.clone()).unwrap();

	}: _(RawOrigin::Root, alice.clone(), Some(new_enclave_address.clone()), Some(new_uri))
	verify {
		assert_eq!(EnclaveData::<T>::get(alice.clone()), Some(new_enclave));
		assert_eq!(EnclaveUpdates::<T>::get(alice), None);
	}

	create_cluster {
		let cluster_id: ClusterId = 0;
		let cluster = Cluster::new(Default::default(), ClusterType::Public);
	}: _(RawOrigin::Root, ClusterType::Public)
	verify {
		assert_eq!(ClusterData::<T>::get(cluster_id), Some(cluster));
	}

	update_cluster {
		let cluster_id: ClusterId = 0;
		let cluster_type: ClusterType = ClusterType::Private;
		let cluster = Cluster::new(Default::default(), ClusterType::Private);
		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
	}: _(RawOrigin::Root, cluster_id, cluster_type)
	verify {
		assert_eq!(ClusterData::<T>::get(cluster_id), Some(cluster));
	}

	remove_cluster {
		let cluster_id: ClusterId = 0;
		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
	}: _(RawOrigin::Root, cluster_id)
	verify {
		assert_eq!(ClusterData::<T>::get(cluster_id), None);
	}

	withdraw_unbonded {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");

		let staked_amount: BalanceOf<T> = 20u32.into();

		let stake_details = TeeStakingLedger::new(alice.clone(), staked_amount, true, Default::default());
		StakingLedger::<T>::insert(alice.clone(), stake_details);
		frame_system::Pallet::<T>::set_block_number(200u32.into());

	}: _(RawOrigin::Signed(alice.clone()))
	verify {
		assert_eq!(StakingLedger::<T>::get(alice), None);
	}

	register_metrics_server {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");

		let metrics_server: MetricsServer<T::AccountId> = MetricsServer::new(alice.clone(), ClusterType::Public);

	}: _(RawOrigin::Root, metrics_server.clone())
	verify {
		assert_eq!(MetricsServers::<T>::get(), vec![metrics_server]);
	}

	unregister_metrics_server {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");

		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();

		let raw = (4 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();

		let metrics_server: MetricsServer<T::AccountId> = MetricsServer::new(alice.clone(), ClusterType::Public);
		TEE::<T>::register_metrics_server(RawOrigin::Root.into(), metrics_server).unwrap();

	}: _(RawOrigin::Root, alice.clone())
	verify {
		assert_eq!(MetricsServers::<T>::get(), vec![]);
	}

	force_update_metrics_server_type {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");

		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();

		let raw = (4 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();

		let metrics_server: MetricsServer<T::AccountId> = MetricsServer::new(alice.clone(), ClusterType::Public);
		TEE::<T>::register_metrics_server(RawOrigin::Root.into(), metrics_server).unwrap();
		let updated_metrics_server: MetricsServer<T::AccountId> = MetricsServer::new(alice.clone(), ClusterType::Private);


	}: _(RawOrigin::Root, alice.clone(), ClusterType::Private)
	verify {
		assert_eq!(MetricsServers::<T>::get(), vec![updated_metrics_server]);
	}

	submit_metrics_server_report {
		prepare_benchmarks::<T>();
		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");

		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		let raw = (4 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();

		let metrics_server: MetricsServer<T::AccountId> = MetricsServer::new(alice.clone(), ClusterType::Public);
		TEE::<T>::register_metrics_server(RawOrigin::Root.into(), metrics_server).unwrap();

		let metrics_server_report: MetricsServerReport<T::AccountId> = MetricsServerReport {
			param_1: 20,
			param_2: 20,
			param_3: 20,
			param_4: 20,
			param_5: 20,
			submitted_by: alice.clone(),
		};


	}: _(origin::<T>("ALICE"), alice.clone(), metrics_server_report.clone())
	verify {
		assert_eq!(MetricsReports::<T>::get(3, alice).unwrap(), vec![metrics_server_report]);
	}

	set_report_params_weightage {
		let report_params_weightage = ReportParamsWeightage {
			param_1_weightage: 20,
			param_2_weightage: 20,
			param_3_weightage: 20,
			param_4_weightage: 20,
			param_5_weightage: 20,
		};
	}: _(RawOrigin::Root, report_params_weightage.clone())
	verify {
		assert_eq!(ReportParamsWeightages::<T>::get(), report_params_weightage);
	}

	set_staking_amount {
		let staking_amount: BalanceOf<T> = 100u32.into();
	}: _(RawOrigin::Root, staking_amount)
	verify {
		assert_eq!(StakingAmount::<T>::get(), staking_amount);
	}

	set_daily_reward_pool {
		let reward_amount: BalanceOf<T> = 100u32.into();
	}: _(RawOrigin::Root, reward_amount)
	verify {
		assert_eq!(DailyRewardPool::<T>::get(), reward_amount);
	}

	claim_rewards {
		prepare_benchmarks::<T>();
		let raw = (10 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");

		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();

		let metrics_server: MetricsServer<T::AccountId> = MetricsServer::new(alice.clone(), ClusterType::Public);
		TEE::<T>::register_metrics_server(RawOrigin::Root.into(), metrics_server).unwrap();

		let report_params_weightage = ReportParamsWeightage {
			param_1_weightage: 20,
			param_2_weightage: 20,
			param_3_weightage: 20,
			param_4_weightage: 20,
			param_5_weightage: 20,
		};
		TEE::<T>::set_report_params_weightage(RawOrigin::Root.into(), report_params_weightage).unwrap();

		let metrics_server_report: MetricsServerReport<T::AccountId> = MetricsServerReport {
			param_1: 20,
			param_2: 20,
			param_3: 20,
			param_4: 20,
			param_5: 20,
			submitted_by: alice.clone(),
		};
		TEE::<T>::submit_metrics_server_report(origin::<T>("ALICE").into(), alice.clone(), metrics_server_report).unwrap();

		let raw = (15 as sp_staking::EraIndex, Some(15u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

	}: _(origin::<T>("ALICE"), 10)

	update_operator_assigned_era {
		prepare_benchmarks::<T>();
		let raw = (10 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");

		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();
	}: _(RawOrigin::Root, alice.clone(), 11)
	verify {
		assert_eq!(OperatorAssignedEra::<T>::get(alice).unwrap(), 11);
	}

	bond_extra {
		prepare_benchmarks::<T>();
		let raw = (10 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");

		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());
		let stake_amount: BalanceOf<T> = 30u32.into();

		TEE::<T>::set_staking_amount(RawOrigin::Root.into(), stake_amount).unwrap();
		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();
		let stake_amount: BalanceOf<T> = 60u32.into();

		TEE::<T>::set_staking_amount(RawOrigin::Root.into(), stake_amount).unwrap();
	}: _(origin::<T>("ALICE"))
	verify {
		assert_eq!(OperatorAssignedEra::<T>::get(alice).unwrap(), 10);
	}

	refund_excess {
		prepare_benchmarks::<T>();
		let raw = (10 as sp_staking::EraIndex, Some(10u64)).encode();
		let info = pallet_staking::ActiveEraInfo::decode(&mut &raw[..]).unwrap();
		pallet_staking::ActiveEra::<T>::put(&info);

		let alice: T::AccountId = get_account::<T>("ALICE");
		let bob: T::AccountId = get_account::<T>("BOB");

		let cluster_id: ClusterId = 0;
		let slot_id: SlotId = 0;
		let enclave_address: T::AccountId= get_account::<T>("ALICE_ENCLAVE");
		let uri: BoundedVec<u8, T::MaxUriLen> = BoundedVec::try_from(vec![1; T::MaxUriLen::get() as usize]).unwrap();
		let enclave = Enclave::new(enclave_address.clone(), uri.clone());

		let stake_amount: BalanceOf<T> = 30u32.into();

		TEE::<T>::set_staking_amount(RawOrigin::Root.into(), stake_amount).unwrap();

		TEE::<T>::create_cluster(RawOrigin::Root.into(), ClusterType::Public).unwrap();
		TEE::<T>::register_enclave(origin::<T>("ALICE").into(), enclave_address.clone(), uri.clone()).unwrap();
		TEE::<T>::assign_enclave(RawOrigin::Root.into(), alice.clone(), cluster_id, slot_id).unwrap();
		let stake_amount: BalanceOf<T> = 10u32.into();

		TEE::<T>::set_staking_amount(RawOrigin::Root.into(), stake_amount).unwrap();
	}: _(origin::<T>("ALICE"))
	verify {
		assert_eq!(OperatorAssignedEra::<T>::get(alice).unwrap(), 10);
	}
}

impl_benchmark_test_suite!(TEE, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
