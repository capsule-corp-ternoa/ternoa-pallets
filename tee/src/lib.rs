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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

mod migrations;
mod types;
mod weights;

use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo},
	traits::ExistenceRequirement::AllowDeath,
	BoundedVec, PalletId,
};

pub use pallet::*;
pub use types::*;

use frame_support::traits::{Get, LockIdentifier, OnRuntimeUpgrade, StorageVersion};
use sp_std::vec;
use sp_std::vec::Vec;

use primitives::tee::{ClusterId, SlotId};
use sp_runtime::{
	traits::{AccountIdConversion, CheckedSub, SaturatedConversion},
	Perbill, Saturating,
};
use ternoa_common::traits;
pub use weights::WeightInfo;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);
const TEE_STAKING_ID: LockIdentifier = *b"teestake";
use pallet_staking::Pallet as Staking;
use sp_staking::EraIndex;

use parity_scale_codec::Decode;
use sp_core::crypto::AccountId32;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, LockableCurrency, WithdrawReasons},
	};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	pub type BalanceOf<T> = <<T as pallet_staking::Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_staking::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for pallet.
		type TeeWeightInfo: WeightInfo;

		// Constants
		/// Size of a cluster
		#[pallet::constant]
		type ClusterSize: Get<u32>;

		/// Max Uri length
		#[pallet::constant]
		type MaxUriLen: Get<u32>;

		/// Size limit for lists
		#[pallet::constant]
		type ListSizeLimit: Get<u32>;

		/// Default staking amount for TEE.
		#[pallet::constant]
		type InitialStakingAmount: Get<BalanceOf<Self>>;

		/// Bonding duration in block numbers.
		#[pallet::constant]
		type TeeBondingDuration: Get<u32>;

		/// The tee pallet id - will be used to generate account id.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Default staking amount for TEE.
		#[pallet::constant]
		type InitalDailyRewardPool: Get<BalanceOf<Self>>;

		/// Number of eras to keep in history for the metrics report.
		#[pallet::constant]
		type TeeHistoryDepth: Get<u32>;
	}

	/// Mapping of operator addresses who want to be registered as enclaves
	#[pallet::storage]
	#[pallet::getter(fn enclaves_to_register)]
	pub type EnclaveRegistrations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Enclave<T::AccountId, T::MaxUriLen>,
		OptionQuery,
	>;

	/// List of registered operator addresses who want to be unregistered
	#[pallet::storage]
	#[pallet::getter(fn enclaves_to_unregister)]
	pub type EnclaveUnregistrations<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::ListSizeLimit>, ValueQuery>;

	/// Mapping of operator addresses to the new values they want for their enclave.
	#[pallet::storage]
	#[pallet::getter(fn enclaves_to_update)]
	pub type EnclaveUpdates<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Enclave<T::AccountId, T::MaxUriLen>,
		OptionQuery,
	>;

	/// Mapping of operator addresses to their enclave data
	#[pallet::storage]
	#[pallet::getter(fn enclaves)]
	pub type EnclaveData<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Enclave<T::AccountId, T::MaxUriLen>,
		OptionQuery,
	>;

	/// Mapping of enclave address to enclave operator address
	#[pallet::storage]
	#[pallet::getter(fn enclave_account_operator)]
	pub type EnclaveAccountOperator<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, OptionQuery>;

	/// Map stores Cluster information
	#[pallet::storage]
	#[pallet::getter(fn clusters)]
	pub type ClusterData<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ClusterId,
		Cluster<T::AccountId, T::ClusterSize>,
		OptionQuery,
	>;

	/// Holds generated ClusterIds
	#[pallet::storage]
	#[pallet::getter(fn next_cluster_id)]
	pub type NextClusterId<T: Config> = StorageValue<_, ClusterId, ValueQuery>;

	/// Map stores Enclave operator | ClusterId
	#[pallet::storage]
	#[pallet::getter(fn enclave_slot_id)]
	pub type EnclaveClusterId<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, ClusterId, OptionQuery>;

	/// Metrics Server accounts storage.
	#[pallet::storage]
	#[pallet::getter(fn metrics_servers)]
	pub(super) type MetricsServers<T: Config> =
		StorageValue<_, BoundedVec<MetricsServer<T::AccountId>, T::ListSizeLimit>, ValueQuery>;

	/// Staking amount for TEE operator.
	#[pallet::storage]
	#[pallet::getter(fn staking_amount)]
	pub(super) type StakingAmount<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, T::InitialStakingAmount>;

	/// Tee Staking details mapped to operator address
	#[pallet::storage]
	#[pallet::getter(fn tee_staking_ledger)]
	pub type StakingLedger<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		TeeStakingLedger<T::AccountId, T::BlockNumber, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn metrics_reports)]
	pub type MetricsReports<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		EraIndex,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<MetricsServerReport<T::AccountId>, T::ListSizeLimit>,
		OptionQuery,
	>;

	/// Report params weightage
	#[pallet::storage]
	#[pallet::getter(fn report_params_weightages)]
	pub type ReportParamsWeightages<T: Config> = StorageValue<_, ReportParamsWeightage, ValueQuery>;

	/// Daily reward amount for TEE operator.
	#[pallet::storage]
	#[pallet::getter(fn daily_reward_pool)]
	pub(super) type DailyRewardPool<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, T::InitalDailyRewardPool>;

	#[pallet::storage]
	#[pallet::getter(fn rewards)]
	pub type ClaimedRewards<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		EraIndex,
		Blake2_128Concat,
		T::AccountId,
		BalanceOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn operator_assigned_block_number)]
	pub type OperatorAssignedEra<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, EraIndex, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			let mut weight = Weight::zero();

			let version = StorageVersion::get::<Pallet<T>>();
			if version == StorageVersion::new(1) {
				weight = <migrations::v2::MigrationV2<T> as OnRuntimeUpgrade>::on_runtime_upgrade();

				StorageVersion::put::<Pallet<T>>(&StorageVersion::new(2));
			}

			weight
		}

		fn on_initialize(now: T::BlockNumber) -> frame_support::weights::Weight {
			let mut read = 0u64;
			let write = 0u64;

			let current_active_era: Option<EraIndex> = match Staking::<T>::active_era() {
				Some(era) => {
					read += 1;
					Some(era.index)
				},
				None => {
					let error_event = Event::FailedToGetActiveEra { block_number: now };
					Self::deposit_event(error_event);
					None
				},
			};

			if let Some(current_active_era) = current_active_era {
				// Clean old era information.
				if let Some(old_era) = current_active_era.checked_sub(T::TeeHistoryDepth::get()) {
					Self::clear_old_era(old_era);
				}
			}
			T::DbWeight::get().reads_writes(read, write)
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New Enclave account got added
		EnclaveAddedForRegistration {
			operator_address: T::AccountId,
			enclave_address: T::AccountId,
			api_uri: BoundedVec<u8, T::MaxUriLen>,
		},
		/// An enclave got unregistered
		RegistrationRemoved { operator_address: T::AccountId },
		/// An enclave update request was cancelled by operator
		UpdateRequestCancelled { operator_address: T::AccountId },
		/// An enclave update request was removed
		UpdateRequestRemoved { operator_address: T::AccountId },
		/// An enclave moved for unregistration to a queue
		MovedForUnregistration { operator_address: T::AccountId },
		/// An enclave got assigned to a cluster
		EnclaveAssigned { operator_address: T::AccountId, cluster_id: ClusterId },
		/// An enclave got removed
		EnclaveRemoved { operator_address: T::AccountId },
		/// An enclave was added to the update list
		MovedForUpdate {
			operator_address: T::AccountId,
			new_enclave_address: T::AccountId,
			new_api_uri: BoundedVec<u8, T::MaxUriLen>,
		},
		/// Enclave updated
		EnclaveUpdated {
			operator_address: T::AccountId,
			new_enclave_address: T::AccountId,
			new_api_uri: BoundedVec<u8, T::MaxUriLen>,
		},
		/// Enclave updated
		EnclaveForceUpdated {
			operator_address: T::AccountId,
			new_enclave_address: Option<T::AccountId>,
			new_api_uri: Option<BoundedVec<u8, T::MaxUriLen>>,
		},
		/// New cluster got added
		ClusterAdded { cluster_id: ClusterId, cluster_type: ClusterType },
		///Cluster got update
		ClusterUpdated { cluster_id: ClusterId, cluster_type: ClusterType },
		/// Cluster got removed
		ClusterRemoved { cluster_id: ClusterId },
		/// Staking amount changed.
		StakingAmountSet { staking_amount: BalanceOf<T> },
		/// Bonded while enclave registration
		Bonded { operator_address: T::AccountId, amount: BalanceOf<T> },
		/// An account has unbonded this amount.
		Unbonded { operator_address: T::AccountId, amount: BalanceOf<T> },
		/// Withdrawn the bonded amount
		Withdrawn { operator_address: T::AccountId, amount: BalanceOf<T> },
		/// New metrics server got added
		MetricsServerAdded { metrics_server: MetricsServer<T::AccountId> },
		/// Updated metrics server cluster type
		MetricsServerTypeUpdated {
			metrics_server_address: T::AccountId,
			new_supported_cluster_type: ClusterType,
		},
		/// Removed a metrics server
		MetricsServerRemoved { metrics_server_address: T::AccountId },
		/// Metrics server report submitted
		MetricsServerReportSubmitted {
			era: EraIndex,
			operator_address: T::AccountId,
			metrics_server_report: MetricsServerReport<T::AccountId>,
		},
		/// Report parameters weightage modified
		ReportParamsWeightageModified {
			param_1_weightage: u8,
			param_2_weightage: u8,
			param_3_weightage: u8,
			param_4_weightage: u8,
			param_5_weightage: u8,
		},
		/// Rewards claimed by operator
		RewardsClaimed { era: EraIndex, operator_address: T::AccountId, amount: BalanceOf<T> },
		/// Fetching active era during the last session in an era
		FailedToGetActiveEra { block_number: T::BlockNumber },
		/// Staking amount is set
		StakingAmountIsSet { amount: BalanceOf<T> },
		/// Reward amount is set
		RewardAmountIsSet { amount: BalanceOf<T> },
		/// Cluster got update
		OperatorAssignedEraUpdated { operator_address: T::AccountId, new_era: EraIndex },
		/// Bonded extra to match default staking amount
		BondedExtra { operator_address: T::AccountId, amount: BalanceOf<T> },
		/// Bonded extra to match default staking amount
		RefundedExcess { operator_address: T::AccountId, amount: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Enclave was not found in storage
		EnclaveNotFound,
		/// The registration already exists
		RegistrationAlreadyExists,
		/// The operator is already linked to an enclave
		OperatorAlreadyExists,
		/// The enclave address is already linked to an operator
		EnclaveAddressAlreadyExists,
		/// Unregistration already exists
		UnregistrationAlreadyExists,
		/// The maximum simultaneous unregistration has been reached
		UnregistrationLimitReached,
		/// The registration does not exist
		RegistrationNotFound,
		/// Enclave address does not exists
		EnclaveAddressNotFound,
		/// Slot id does not exist for this address
		SlotIdNotFound,
		/// The cluster does not exists
		ClusterNotFound,
		/// Cluster id does not exist for this address
		ClusterIdNotFound,
		/// The cluster still have enclaves associated to it
		ClusterIsNotEmpty,
		/// Cluster is already full, cannot assign any enclaves
		ClusterIsFull,
		/// The given api uri is empty
		ApiUriIsEmpty,
		/// The given operator account and enclave account are same
		OperatorAndEnclaveAreSame,
		/// The operator already asked for request
		UpdateRequestAlreadyExists,
		/// The update request was not found in storage
		UpdateRequestNotFound,
		/// The update is not allowed for unassigned enclave
		UpdateProhibitedForUnassignedEnclave,
		/// Staking details not found
		StakingNotFound,
		/// Withdraw can not be done without unbonding
		UnbondingNotStarted,
		/// Withdraw is not allowed till the unbonding period is done
		WithdrawProhibited,
		/// Metrics server already registered
		MetricsServerAlreadyExists,
		/// Metrics server not found
		MetricsServerNotFound,
		/// Metrics server limit reached
		MetricsServerLimitReached,
		/// Metrics server address not found
		MetricsServerAddressNotFound,
		/// Unsupported cluster type for a metrics server to submit report
		MetricsServerUnsupportedClusterType,
		/// Enclave address not found for the operator
		EnclaveNotFoundForTheOperator,
		/// Operator not found in unregistration list for approving unregistration
		UnregistrationNotFound,
		/// Failed to get the active era from the staking pallet
		FailedToGetActiveEra,
		/// Metrics reports limit reached
		MetricsReportsLimitReached,
		/// Invalid era to claim rewards
		InvalidEraToClaimRewards,
		/// Rewards already claimed for the era
		RewardsAlreadyClaimedForEra,
		/// Insuffience Balance to Bond
		InsufficientBalanceToBond,
		/// Operator assigned era not found
		OperatorAssignedEraNotFound,
		/// Bond extra not allowed since the current staked amount of operator is not less than
		/// default staking amount
		BondExtraNotAllowed,
		/// Refund excess not allowed since the current staked amount of operator is not higher
		/// than the default staking amount
		RefundExcessNotAllowed,
		/// Force update should have either new enclave address or new api uri to be updated
		NoUpdatesProvided,
		/// Registration not allowed while unbonding period
		UnbondingPeriod,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Ask for an enclave registration
		#[pallet::weight(T::TeeWeightInfo::register_enclave())]
		pub fn register_enclave(
			origin: OriginFor<T>,
			enclave_address: T::AccountId,
			api_uri: BoundedVec<u8, T::MaxUriLen>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(!api_uri.is_empty(), Error::<T>::ApiUriIsEmpty);
			ensure!(who != enclave_address, Error::<T>::OperatorAndEnclaveAreSame);
			ensure!(
				EnclaveRegistrations::<T>::get(&who).is_none(),
				Error::<T>::RegistrationAlreadyExists
			);
			ensure!(EnclaveData::<T>::get(&who).is_none(), Error::<T>::OperatorAlreadyExists);
			ensure!(
				EnclaveAccountOperator::<T>::get(&enclave_address).is_none(),
				Error::<T>::EnclaveAddressAlreadyExists
			);
			ensure!(StakingLedger::<T>::get(&who).is_none(), Error::<T>::UnbondingPeriod);
			let default_staking_amount = StakingAmount::<T>::get();

			let operator_balance = T::Currency::free_balance(&who);
			let new_operator_balance = operator_balance
				.checked_sub(&default_staking_amount)
				.ok_or(Error::<T>::InsufficientBalanceToBond)?;

			T::Currency::ensure_can_withdraw(
				&who,
				default_staking_amount.clone(),
				WithdrawReasons::all(),
				new_operator_balance,
			)?;

			let stake_details = TeeStakingLedger::new(
				who.clone(),
				default_staking_amount.clone(),
				false,
				Default::default(),
			);
			StakingLedger::<T>::insert(who.clone(), stake_details);
			T::Currency::set_lock(
				TEE_STAKING_ID,
				&who,
				default_staking_amount,
				WithdrawReasons::all(),
			);

			let enclave = Enclave::new(enclave_address.clone(), api_uri.clone());
			EnclaveRegistrations::<T>::insert(who.clone(), enclave);

			Self::deposit_event(Event::EnclaveAddedForRegistration {
				operator_address: who.clone(),
				enclave_address,
				api_uri,
			});
			Self::deposit_event(Event::Bonded {
				operator_address: who,
				amount: default_staking_amount,
			});

			Ok(().into())
		}

		/// Ask for an enclave to be removed.
		/// No need for approval if the enclave registration was not approved yet.
		#[pallet::weight(T::TeeWeightInfo::unregister_enclave())]
		pub fn unregister_enclave(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let default_staking_amount = StakingAmount::<T>::get();
			match EnclaveData::<T>::get(&who) {
				Some(_) => {
					EnclaveUnregistrations::<T>::try_mutate(|x| -> DispatchResult {
						ensure!(!x.contains(&who), Error::<T>::UnregistrationAlreadyExists);
						x.try_push(who.clone())
							.map_err(|_| Error::<T>::UnregistrationLimitReached)?;
						Ok(())
					})?;
					let now = frame_system::Pallet::<T>::block_number();

					StakingLedger::<T>::try_mutate(
						&who,
						|maybe_stake_details| -> DispatchResult {
							let stake_details =
								maybe_stake_details.as_mut().ok_or(Error::<T>::StakingNotFound)?;

							stake_details.is_unlocking = true;
							stake_details.unbonded_at = now;

							Self::deposit_event(Event::MovedForUnregistration {
								operator_address: who.clone(),
							});
							Self::deposit_event(Event::Unbonded {
								operator_address: who.clone(),
								amount: stake_details.staked_amount,
							});

							Ok(())
						},
					)?;
				},
				None => {
					EnclaveRegistrations::<T>::try_mutate(
						&who,
						|maybe_registration| -> DispatchResult {
							let _ = maybe_registration
								.as_mut()
								.ok_or(Error::<T>::RegistrationNotFound)?;
							*maybe_registration = None;
							Ok(())
						},
					)?;
					StakingLedger::<T>::remove(who.clone());
					T::Currency::remove_lock(TEE_STAKING_ID, &who);

					Self::deposit_event(Event::RegistrationRemoved {
						operator_address: who.clone(),
					});
					Self::deposit_event(Event::Withdrawn {
						operator_address: who,
						amount: default_staking_amount,
					});
				},
			}

			Ok(().into())
		}

		/// Ask for enclave update
		#[pallet::weight(T::TeeWeightInfo::update_enclave())]
		pub fn update_enclave(
			origin: OriginFor<T>,
			new_enclave_address: T::AccountId,
			new_api_uri: BoundedVec<u8, T::MaxUriLen>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(!new_api_uri.is_empty(), Error::<T>::ApiUriIsEmpty);
			ensure!(who != new_enclave_address, Error::<T>::OperatorAndEnclaveAreSame);

			let enclave = EnclaveData::<T>::get(&who)
				.ok_or(Error::<T>::UpdateProhibitedForUnassignedEnclave)?;
			ensure!(
				EnclaveUpdates::<T>::get(&who).is_none(),
				Error::<T>::UpdateRequestAlreadyExists
			);

			ensure!(
				enclave.enclave_address == new_enclave_address ||
					EnclaveAccountOperator::<T>::get(&new_enclave_address).is_none(),
				Error::<T>::EnclaveAddressAlreadyExists
			);

			let enclave = Enclave::new(new_enclave_address.clone(), new_api_uri.clone());
			EnclaveUpdates::<T>::insert(who.clone(), enclave);

			Self::deposit_event(Event::MovedForUpdate {
				operator_address: who,
				new_enclave_address,
				new_api_uri,
			});
			Ok(().into())
		}

		/// Remove the operator update request
		#[pallet::weight(T::TeeWeightInfo::cancel_update())]
		pub fn cancel_update(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			EnclaveUpdates::<T>::try_mutate(&who, |maybe_update| -> DispatchResult {
				let _ = maybe_update.as_mut().ok_or(Error::<T>::UpdateRequestNotFound)?;
				*maybe_update = None;
				Ok(())
			})?;

			Self::deposit_event(Event::UpdateRequestCancelled { operator_address: who });
			Ok(().into())
		}

		/// Assign an enclave to a cluster
		#[pallet::weight(T::TeeWeightInfo::assign_enclave())]
		pub fn assign_enclave(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
			cluster_id: ClusterId,
			slot_id: SlotId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveRegistrations::<T>::try_mutate(
				&operator_address,
				|maybe_registration| -> DispatchResult {
					let registration =
						maybe_registration.as_mut().ok_or(Error::<T>::RegistrationNotFound)?;

					ClusterData::<T>::try_mutate(cluster_id, |maybe_cluster| -> DispatchResult {
						let cluster = maybe_cluster.as_mut().ok_or(Error::<T>::ClusterNotFound)?;

						ensure!(
							cluster.enclaves.len() < T::ClusterSize::get() as usize,
							Error::<T>::ClusterIsFull
						);

						ensure!(
							EnclaveAccountOperator::<T>::get(&registration.enclave_address)
								.is_none(),
							Error::<T>::EnclaveAddressAlreadyExists
						);
						ensure!(
							EnclaveData::<T>::get(&operator_address).is_none(),
							Error::<T>::OperatorAlreadyExists
						);

						// Add enclave account to operator
						EnclaveAccountOperator::<T>::insert(
							registration.enclave_address.clone(),
							operator_address.clone(),
						);

						// Add enclave data
						EnclaveData::<T>::insert(operator_address.clone(), registration);

						// Add enclave to cluster id
						EnclaveClusterId::<T>::insert(operator_address.clone(), cluster_id);

						let current_active_era = Staking::<T>::active_era()
							.map(|e| e.index)
							.ok_or(Error::<T>::FailedToGetActiveEra)?;

						OperatorAssignedEra::<T>::insert(
							operator_address.clone(),
							current_active_era,
						);

						// Add enclave operator to cluster
						cluster
							.enclaves
							.try_push((operator_address.clone(), slot_id))
							.map_err(|_| Error::<T>::ClusterIsFull)?;

						Ok(())
					})?;

					*maybe_registration = None;
					Ok(())
				},
			)?;

			Self::deposit_event(Event::EnclaveAssigned { operator_address, cluster_id });
			Ok(().into())
		}

		/// Remove a registration from storage
		#[pallet::weight(T::TeeWeightInfo::remove_registration())]
		pub fn remove_registration(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveRegistrations::<T>::try_mutate(
				&operator_address,
				|maybe_registration| -> DispatchResult {
					let _ = maybe_registration.as_mut().ok_or(Error::<T>::RegistrationNotFound)?;
					*maybe_registration = None;
					StakingLedger::<T>::remove(&operator_address);
					T::Currency::remove_lock(TEE_STAKING_ID, &operator_address);
					Ok(())
				},
			)?;

			Self::deposit_event(Event::RegistrationRemoved { operator_address });
			Ok(().into())
		}

		/// Remove an enclave update request from storage
		#[pallet::weight(T::TeeWeightInfo::reject_update())]
		pub fn reject_update(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveUpdates::<T>::try_mutate(&operator_address, |maybe_update| -> DispatchResult {
				let _ = maybe_update.as_mut().ok_or(Error::<T>::UpdateRequestNotFound)?;
				*maybe_update = None;
				Ok(())
			})?;

			Self::deposit_event(Event::UpdateRequestRemoved { operator_address });
			Ok(().into())
		}

		/// Unassign an enclave from a cluster and remove all information
		#[pallet::weight(T::TeeWeightInfo::force_remove_enclave())]
		pub fn approve_enclave_unregistration(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveUnregistrations::<T>::try_mutate(|unregistrations| -> DispatchResult {
				ensure!(
					unregistrations.contains(&operator_address),
					Error::<T>::UnregistrationNotFound
				);

				EnclaveData::<T>::try_mutate(
					&operator_address,
					|maybe_enclave| -> DispatchResult {
						let enclave = maybe_enclave.as_mut().ok_or(Error::<T>::EnclaveNotFound)?;

						ensure!(
							EnclaveAccountOperator::<T>::get(enclave.enclave_address.clone())
								.is_some(),
							Error::<T>::EnclaveAddressNotFound
						);

						let cluster_id = EnclaveClusterId::<T>::get(&operator_address)
							.ok_or(Error::<T>::ClusterIdNotFound)?;

						ClusterData::<T>::try_mutate(
							cluster_id,
							|maybe_cluster| -> DispatchResult {
								let cluster =
									maybe_cluster.as_mut().ok_or(Error::<T>::ClusterNotFound)?;

								EnclaveUpdates::<T>::try_mutate(
									&operator_address,
									|maybe_update| -> DispatchResult {
										if maybe_update.is_some() {
											*maybe_update = None;
										}
										Ok(())
									},
								)?;

								// Remove the operator from cluster
								if let Some(index) =
									cluster.enclaves.iter().position(|(account_id, _slot_id)| {
										*account_id == operator_address.clone()
									}) {
									cluster.enclaves.swap_remove(index);
								}

								// Remove the mapping between operator to cluster id
								EnclaveClusterId::<T>::remove(&operator_address);

								// Remove the mapping between operator to assigned block number
								OperatorAssignedEra::<T>::remove(&operator_address);

								// Remove the mapping between enclave address to operator address
								EnclaveAccountOperator::<T>::remove(&enclave.enclave_address);

								Ok(())
							},
						)?;

						// Remove the enclave data
						*maybe_enclave = None;
						Ok(())
					},
				)?;
				if let Some(index) =
					unregistrations.iter().position(|x| *x == operator_address.clone())
				{
					unregistrations.swap_remove(index);
				}
				Ok(())
			})?;

			Self::deposit_event(Event::EnclaveRemoved { operator_address });
			Ok(().into())
		}

		/// Unassign an enclave from a cluster and remove all information
		#[pallet::weight(T::TeeWeightInfo::force_remove_enclave())]
		pub fn force_remove_enclave(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveData::<T>::try_mutate(&operator_address, |maybe_enclave| -> DispatchResult {
				let enclave = maybe_enclave.as_mut().ok_or(Error::<T>::EnclaveNotFound)?;

				ensure!(
					EnclaveAccountOperator::<T>::get(enclave.enclave_address.clone()).is_some(),
					Error::<T>::EnclaveAddressNotFound
				);

				let cluster_id = EnclaveClusterId::<T>::get(&operator_address)
					.ok_or(Error::<T>::ClusterIdNotFound)?;

				ClusterData::<T>::try_mutate(cluster_id, |maybe_cluster| -> DispatchResult {
					let cluster = maybe_cluster.as_mut().ok_or(Error::<T>::ClusterNotFound)?;

					// Remove enclave from unregistration list
					EnclaveUnregistrations::<T>::try_mutate(|x| -> DispatchResult {
						if let Some(index) = x.iter().position(|x| *x == operator_address.clone()) {
							x.swap_remove(index);
						}
						Ok(())
					})?;

					EnclaveUpdates::<T>::try_mutate(
						&operator_address,
						|maybe_update| -> DispatchResult {
							if maybe_update.is_some() {
								*maybe_update = None;
							}
							Ok(())
						},
					)?;

					// Remove the operator from cluster
					if let Some(index) = cluster
						.enclaves
						.iter()
						.position(|(account_id, _slot_id)| *account_id == operator_address.clone())
					{
						cluster.enclaves.swap_remove(index);
					}

					// Remove the mapping between operator to cluster id
					EnclaveClusterId::<T>::remove(&operator_address);

					// Remove the mapping between operator to assigned block number
					OperatorAssignedEra::<T>::remove(&operator_address);

					// Remove the mapping between enclave address to operator address
					EnclaveAccountOperator::<T>::remove(&enclave.enclave_address);

					Ok(())
				})?;

				let now = frame_system::Pallet::<T>::block_number();
				StakingLedger::<T>::try_mutate(
					&operator_address,
					|maybe_stake_details| -> DispatchResult {
						let stake_details =
							maybe_stake_details.as_mut().ok_or(Error::<T>::StakingNotFound)?;
						stake_details.is_unlocking = true;
						stake_details.unbonded_at = now;

						Self::deposit_event(Event::Unbonded {
							operator_address: operator_address.clone(),
							amount: stake_details.staked_amount,
						});

						Ok(())
					},
				)?;

				// Remove the enclave data
				*maybe_enclave = None;
				Ok(())
			})?;

			Self::deposit_event(Event::EnclaveRemoved { operator_address });
			Ok(().into())
		}

		/// Update an enclave and clean the enclaves to update if needed
		#[pallet::weight(T::TeeWeightInfo::force_update_enclave())]
		pub fn approve_update_enclave(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveUpdates::<T>::try_mutate(&operator_address, |maybe_update| -> DispatchResult {
				let enclave_update =
					maybe_update.as_mut().ok_or(Error::<T>::UpdateRequestNotFound)?;
				let new_enclave_address = enclave_update.enclave_address.clone();
				let new_api_uri = enclave_update.api_uri.clone();
				ensure!(!new_api_uri.is_empty(), Error::<T>::ApiUriIsEmpty);
				ensure!(
					operator_address != new_enclave_address,
					Error::<T>::OperatorAndEnclaveAreSame
				);

				EnclaveData::<T>::try_mutate(
					&operator_address,
					|maybe_enclave| -> DispatchResult {
						let enclave = maybe_enclave.as_mut().ok_or(Error::<T>::EnclaveNotFound)?;

						if enclave.enclave_address != new_enclave_address {
							ensure!(
								EnclaveAccountOperator::<T>::get(&new_enclave_address).is_none(),
								Error::<T>::EnclaveAddressAlreadyExists
							);
							EnclaveAccountOperator::<T>::insert(
								new_enclave_address.clone(),
								operator_address.clone(),
							);
						}

						enclave.enclave_address = new_enclave_address.clone();
						enclave.api_uri = new_api_uri.clone();
						Ok(())
					},
				)?;

				*maybe_update = None;

				Self::deposit_event(Event::EnclaveUpdated {
					operator_address: operator_address.clone(),
					new_enclave_address,
					new_api_uri,
				});

				Ok(())
			})?;

			Ok(().into())
		}

		/// Update an enclave and clean the enclaves to update if needed
		#[pallet::weight(T::TeeWeightInfo::force_update_enclave())]
		pub fn force_update_enclave(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
			new_enclave_address: Option<T::AccountId>,
			new_api_uri: Option<BoundedVec<u8, T::MaxUriLen>>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Ensure at least one of the optional parameters is provided
			ensure!(
				new_enclave_address.is_some() || new_api_uri.is_some(),
				Error::<T>::NoUpdatesProvided
			);

			EnclaveData::<T>::try_mutate(&operator_address, |maybe_enclave| -> DispatchResult {
				let enclave = maybe_enclave.as_mut().ok_or(Error::<T>::EnclaveNotFound)?;

				// Update enclave address if provided
				if let Some(address) = new_enclave_address.clone() {
					ensure!(operator_address != address, Error::<T>::OperatorAndEnclaveAreSame);

					if enclave.enclave_address != address {
						ensure!(
							EnclaveAccountOperator::<T>::get(&address).is_none(),
							Error::<T>::EnclaveAddressAlreadyExists
						);
						// Remove the mapping between enclave address to operator address
						EnclaveAccountOperator::<T>::remove(&enclave.enclave_address);

						EnclaveAccountOperator::<T>::insert(
							address.clone(),
							operator_address.clone(),
						);
					}

					enclave.enclave_address = address.clone();
				}

				// Update API URI if provided
				if let Some(uri) = new_api_uri.clone() {
					ensure!(!uri.is_empty(), Error::<T>::ApiUriIsEmpty);
					enclave.api_uri = uri.clone();
				}

				Ok(())
			})?;

			EnclaveUpdates::<T>::try_mutate(&operator_address, |maybe_update| -> DispatchResult {
				if maybe_update.is_some() {
					*maybe_update = None;
				}
				Ok(())
			})?;

			Self::deposit_event(Event::EnclaveForceUpdated {
				operator_address,
				new_enclave_address,
				new_api_uri,
			});
			Ok(().into())
		}

		// Creates an empty Cluster
		#[pallet::weight(T::TeeWeightInfo::create_cluster())]
		pub fn create_cluster(
			origin: OriginFor<T>,
			cluster_type: ClusterType,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let id = Self::get_next_cluster_id();
			let cluster = Cluster::new(Default::default(), cluster_type.clone());
			ClusterData::<T>::insert(id, cluster);
			Self::deposit_event(Event::ClusterAdded { cluster_id: id, cluster_type });
			Ok(().into())
		}

		// Updates the cluster type
		#[pallet::weight(T::TeeWeightInfo::update_cluster())]
		pub fn update_cluster(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
			cluster_type: ClusterType,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			ClusterData::<T>::try_mutate(cluster_id, |maybe_cluster| -> DispatchResult {
				let cluster = maybe_cluster.as_mut().ok_or(Error::<T>::ClusterNotFound)?;
				cluster.cluster_type = cluster_type.clone();
				Self::deposit_event(Event::ClusterUpdated { cluster_id, cluster_type });
				Ok(())
			})?;
			Ok(().into())
		}

		/// Removes an empty cluster
		#[pallet::weight(T::TeeWeightInfo::remove_cluster())]
		pub fn remove_cluster(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ClusterData::<T>::try_mutate(cluster_id, |maybe_cluster| -> DispatchResult {
				let cluster = maybe_cluster.as_mut().ok_or(Error::<T>::ClusterNotFound)?;
				ensure!(cluster.enclaves.len() == 0, Error::<T>::ClusterIsNotEmpty);
				*maybe_cluster = None;
				Ok(())
			})?;

			Self::deposit_event(Event::ClusterRemoved { cluster_id });
			Ok(().into())
		}

		/// Withdraw the unbonded amount
		#[pallet::weight(T::TeeWeightInfo::withdraw_unbonded())]
		pub fn withdraw_unbonded(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			StakingLedger::<T>::try_mutate(&who, |maybe_staking| -> DispatchResult {
				let staking_details = maybe_staking.as_mut().ok_or(Error::<T>::StakingNotFound)?;
				ensure!(staking_details.is_unlocking, Error::<T>::UnbondingNotStarted);
				let now = frame_system::Pallet::<T>::block_number();
				let bonding_duration = T::TeeBondingDuration::get();
				let unbonded_at = staking_details.unbonded_at;
				let duration: u32 = (now - unbonded_at).saturated_into();
				ensure!(duration >= bonding_duration, Error::<T>::WithdrawProhibited);
				T::Currency::remove_lock(TEE_STAKING_ID, &who);
				// Remove the staking data
				*maybe_staking = None;
				Ok(())
			})?;

			let default_staking_amount = StakingAmount::<T>::get();
			Self::deposit_event(Event::Withdrawn {
				operator_address: who,
				amount: default_staking_amount,
			});

			Ok(().into())
		}

		/// Metrics server registration by Technical Committee.
		#[pallet::weight(T::TeeWeightInfo::register_metrics_server())]
		pub fn register_metrics_server(
			origin: OriginFor<T>,
			metrics_server: MetricsServer<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			MetricsServers::<T>::try_mutate(|metrics_servers| -> DispatchResult {
				ensure!(
					!metrics_servers.iter().any(|server| server.metrics_server_address ==
						metrics_server.metrics_server_address),
					Error::<T>::MetricsServerAlreadyExists
				);
				metrics_servers
					.try_push(metrics_server.clone())
					.map_err(|_| Error::<T>::MetricsServerLimitReached)?;
				Self::deposit_event(Event::MetricsServerAdded { metrics_server });
				Ok(())
			})?;
			Ok(().into())
		}

		#[pallet::weight(T::TeeWeightInfo::unregister_metrics_server())]
		pub fn unregister_metrics_server(
			origin: OriginFor<T>,
			metrics_server_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			MetricsServers::<T>::try_mutate(|metrics_servers| -> DispatchResult {
				if let Some(index) = metrics_servers
					.iter()
					.position(|server| server.metrics_server_address == metrics_server_address)
				{
					// Remove the metrics server registration at the found index
					metrics_servers.swap_remove(index);
					Self::deposit_event(Event::MetricsServerRemoved { metrics_server_address });
				} else {
					return Err(Error::<T>::MetricsServerNotFound.into())
				}

				Ok(())
			})?;

			Ok(().into())
		}

		#[pallet::weight(T::TeeWeightInfo::force_update_metrics_server_type())]
		pub fn force_update_metrics_server_type(
			origin: OriginFor<T>,
			metrics_server_address: T::AccountId,
			new_supported_cluster_type: ClusterType,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			MetricsServers::<T>::try_mutate(|metrics_servers| -> DispatchResult {
				if let Some(server) = metrics_servers
					.iter_mut()
					.find(|s| s.metrics_server_address == metrics_server_address)
				{
					// Update the supported_cluster_type for the metrics server
					server.supported_cluster_type = new_supported_cluster_type.clone();
					Self::deposit_event(Event::MetricsServerTypeUpdated {
						metrics_server_address,
						new_supported_cluster_type,
					});
				} else {
					return Err(Error::<T>::MetricsServerNotFound.into())
				}

				Ok(())
			})?;

			Ok(().into())
		}

		#[pallet::weight(T::TeeWeightInfo::submit_metrics_server_report())]
		pub fn submit_metrics_server_report(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
			metrics_server_report: MetricsServerReport<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let found_server = MetricsServers::<T>::get()
				.iter()
				.find(|server| server.metrics_server_address == who)
				.cloned();

			if let Some(server) = found_server {
				if server.supported_cluster_type != ClusterType::Public {
					return Err(Error::<T>::MetricsServerUnsupportedClusterType.into())
				}
			} else {
				return Err(Error::<T>::MetricsServerAddressNotFound.into())
			}

			EnclaveData::<T>::get(&operator_address)
				.ok_or(Error::<T>::EnclaveNotFoundForTheOperator)?;

			// Retrieve the era index
			let era_index = Staking::<T>::active_era()
				.map(|e| e.index)
				.ok_or(Error::<T>::FailedToGetActiveEra)?
				.saturating_sub(1);

			let existing_reports = MetricsReports::<T>::get(&era_index, &operator_address);

			if let Some(mut existing_reports) = existing_reports {
				if let Some((index, _)) = existing_reports
					.iter()
					.enumerate()
					.find(|(_, report)| report.submitted_by == metrics_server_report.submitted_by)
				{
					existing_reports[index] = metrics_server_report.clone();
				} else {
					existing_reports
						.try_push(metrics_server_report.clone())
						.map_err(|_| Error::<T>::MetricsReportsLimitReached)?;
				}

				MetricsReports::<T>::insert(&era_index, &operator_address, existing_reports);
			} else {
				let mut reports =
					BoundedVec::<MetricsServerReport<T::AccountId>, T::ListSizeLimit>::default();
				reports
					.try_push(metrics_server_report.clone())
					.map_err(|_| Error::<T>::MetricsReportsLimitReached)?;

				MetricsReports::<T>::insert(&era_index, &operator_address, reports);
			}

			// // Emit an event for the successful submission
			Self::deposit_event(Event::MetricsServerReportSubmitted {
				era: era_index,
				operator_address,
				metrics_server_report,
			});

			Ok(().into())
		}

		/// Report parameters weightage modification which can be done by Technical Committee.
		#[pallet::weight(T::TeeWeightInfo::set_report_params_weightage())]
		pub fn set_report_params_weightage(
			origin: OriginFor<T>,
			report_params_weightage: ReportParamsWeightage,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ReportParamsWeightages::<T>::put(report_params_weightage.clone());

			Self::deposit_event(Event::ReportParamsWeightageModified {
				param_1_weightage: report_params_weightage.param_1_weightage,
				param_2_weightage: report_params_weightage.param_2_weightage,
				param_3_weightage: report_params_weightage.param_3_weightage,
				param_4_weightage: report_params_weightage.param_4_weightage,
				param_5_weightage: report_params_weightage.param_5_weightage,
			});
			Ok(().into())
		}

		/// Set staking amount for operators by Technical Committee
		#[pallet::weight(T::TeeWeightInfo::set_staking_amount())]
		pub fn set_staking_amount(
			origin: OriginFor<T>,
			staking_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			StakingAmount::<T>::put(staking_amount);

			Self::deposit_event(Event::StakingAmountIsSet { amount: staking_amount });
			Ok(().into())
		}

		/// Set reward pool amount for operators by Technical Committee
		#[pallet::weight(T::TeeWeightInfo::set_daily_reward_pool())]
		pub fn set_daily_reward_pool(
			origin: OriginFor<T>,
			reward_amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			DailyRewardPool::<T>::put(reward_amount);

			Self::deposit_event(Event::RewardAmountIsSet { amount: reward_amount });
			Ok(().into())
		}

		/// Claim rewards by Era
		#[pallet::weight(T::TeeWeightInfo::claim_rewards())]
		pub fn claim_rewards(origin: OriginFor<T>, era: EraIndex) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				ClaimedRewards::<T>::get(&era, &who).is_none(),
				Error::<T>::RewardsAlreadyClaimedForEra
			);

			let current_active_era = Staking::<T>::active_era()
				.map(|e| e.index)
				.ok_or(Error::<T>::FailedToGetActiveEra)?;

			let operator_assigned_era = OperatorAssignedEra::<T>::get(&who)
				.ok_or(Error::<T>::OperatorAssignedEraNotFound)?;

			ensure!(
				era < current_active_era.saturating_sub(2) &&
					era > current_active_era.saturating_sub(T::TeeHistoryDepth::get()) &&
					era >= operator_assigned_era,
				Error::<T>::InvalidEraToClaimRewards
			);

			EnclaveData::<T>::get(&who).ok_or(Error::<T>::EnclaveNotFoundForTheOperator)?;

			let total_operators = EnclaveData::<T>::iter_keys().count();
			let share_fraction = Perbill::from_rational(1, total_operators as u32);
			let reward_pool = Self::daily_reward_pool();

			let reward_per_operator: BalanceOf<T> = share_fraction * reward_pool;

			let submitted_metrics_report = MetricsReports::<T>::get(&era, &who);

			if let Some(submitted_metrics_report) = submitted_metrics_report {
				let variance = Self::calculate_highest_params(&submitted_metrics_report);

				let report_params_weightage = Self::report_params_weightages();

				let weighted_sum =
					Self::calculate_weighted_sum(&variance, &report_params_weightage);
				let percent = Perbill::from_rational(weighted_sum, 10000 as u32);

				let weighted_reward_amount = percent * reward_per_operator;

				T::Currency::transfer(
					&Self::account_id(),
					&who,
					weighted_reward_amount,
					AllowDeath,
				)?;
				ClaimedRewards::<T>::insert(era, who.clone(), weighted_reward_amount.clone());
				Self::deposit_event(Event::RewardsClaimed {
					era,
					operator_address: who.clone(),
					amount: weighted_reward_amount,
				});
			} else {
				T::Currency::transfer(&Self::account_id(), &who, reward_per_operator, AllowDeath)?;
				ClaimedRewards::<T>::insert(era, who.clone(), reward_per_operator.clone());
				Self::deposit_event(Event::RewardsClaimed {
					era,
					operator_address: who.clone(),
					amount: reward_per_operator,
				});
			}
			Ok(().into())
		}

		// Updates assigned era for an operator
		#[pallet::weight(T::TeeWeightInfo::update_operator_assigned_era())]
		pub fn update_operator_assigned_era(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
			new_era: EraIndex,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			OperatorAssignedEra::<T>::try_mutate(
				operator_address.clone(),
				|maybe_operator| -> DispatchResult {
					let operator_assigned_era =
						maybe_operator.as_mut().ok_or(Error::<T>::OperatorAssignedEraNotFound)?;
					*operator_assigned_era = new_era.clone();
					Self::deposit_event(Event::OperatorAssignedEraUpdated {
						operator_address,
						new_era,
					});
					Ok(())
				},
			)?;
			Ok(().into())
		}

		// Bond extra if the default staking amount is increased
		#[pallet::weight(T::TeeWeightInfo::bond_extra())]
		pub fn bond_extra(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let default_staking_amount = StakingAmount::<T>::get();

			StakingLedger::<T>::try_mutate(&who, |maybe_stake_details| -> DispatchResult {
				let stake_details =
					maybe_stake_details.as_mut().ok_or(Error::<T>::StakingNotFound)?;

				ensure!(
					stake_details.staked_amount < default_staking_amount,
					Error::<T>::BondExtraNotAllowed
				);

				let extra_bond_required =
					default_staking_amount.saturating_sub(stake_details.staked_amount);

				let operator_balance = T::Currency::free_balance(&who);
				let new_operator_balance = operator_balance
					.checked_sub(&extra_bond_required)
					.ok_or(Error::<T>::InsufficientBalanceToBond)?;

				T::Currency::ensure_can_withdraw(
					&who,
					extra_bond_required.clone(),
					WithdrawReasons::all(),
					new_operator_balance,
				)?;

				stake_details.staked_amount = default_staking_amount.clone();

				T::Currency::set_lock(
					TEE_STAKING_ID,
					&who,
					default_staking_amount.clone(),
					WithdrawReasons::all(),
				);

				Self::deposit_event(Event::BondedExtra {
					operator_address: who.clone(),
					amount: extra_bond_required,
				});

				Ok(())
			})?;
			Ok(().into())
		}

		// Bond extra if the default staking amount is increased
		#[pallet::weight(T::TeeWeightInfo::refund_excess())]
		pub fn refund_excess(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let default_staking_amount = StakingAmount::<T>::get();

			StakingLedger::<T>::try_mutate(&who, |maybe_stake_details| -> DispatchResult {
				let stake_details =
					maybe_stake_details.as_mut().ok_or(Error::<T>::StakingNotFound)?;

				ensure!(
					stake_details.staked_amount > default_staking_amount,
					Error::<T>::RefundExcessNotAllowed
				);

				let extra_bond_to_be_refunded =
					stake_details.staked_amount.saturating_sub(default_staking_amount);

				stake_details.staked_amount = default_staking_amount.clone();

				T::Currency::set_lock(
					TEE_STAKING_ID,
					&who,
					default_staking_amount.clone(),
					WithdrawReasons::all(),
				);

				Self::deposit_event(Event::RefundedExcess {
					operator_address: who.clone(),
					amount: extra_bond_to_be_refunded,
				});

				Ok(())
			})?;
			Ok(().into())
		}

		#[pallet::weight(T::TeeWeightInfo::refund_excess())]
		pub fn force_fix(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			// Remove these operators from the storages
			let accounts = &[
				"5GrqCRkedhRBvaTnfNxitMQrvVHhR4zj4HChvgE1WT9M5oiL",
				"5GmV6mxKpF3AWWAfgvikkKQhqnnMucTUjE1YhTt56YZ9TGQQ",
				"5Df3w3sAnVoTPtSwaoS4AZJRRXHDDxTgzDZ834K2cosBuJzn",
				"5FWSDLFmqN3z97HGi8EgrpSy9qKU316QfAQ9cQWtDFmMUP9F",
				"5C4hrfkcV8Q2SLRkqBDyWqAJaFmi66cfTtQgcwiHpRFWEULC",
				"5HXR8qQqug9JRAUDNG3UdyEvU7BzzpherctfEbDXbRzfy4Rw",
				"5C4hrfkGSWFRLkLKEkYypTQ9i9NJr2jD72euevVp28gbRDNE",
				"5HhFF5DPFg92k3bN7wibBdHhXoQEMqAToRBaZMfSPZiicd3x",
				"5FZbAfjxCpRPXXpiQ1eAXg4vhLiuwer8L4RPZGLPoh9xCWAi",
				"5GEJnKbHFFXoSYfxuMh9bXQB2vE8UuTSSH5yS2nvF4TXjwrv",
				"5C4hrfkPcF24v5ie2u8VUeaV39dRtspRQ7QJvXF6RoNxrmHL",
				"5E97ao6eksZpeAqZXUhNkWpXNcHrLw48F56wTRFy6e2hGS8J",
				"5FFMF5cSWdHfAoXiB2K598FEdbcq4RJjZhHQAiGjxNo2jBbL",
				"5GkmvC6kpiBGH1yuTBRkJsNaxsKMjfpD6Smvys2WJWLf2x89",
				"5C4hrfkeBsjXtFM2KYophPRPeyWicTTprL8kKB7Rs9gtTgyu",
				"5D5EjB9oVgxbzB8MLFX7P2buoFyGVD3frnngYA2qdEYbAwNh",
				"5C4hrfkL1yVp8J4CHi5b3qKULTXHwsG8UTr659FyctFB9Gvx",
				"5HMqTBKwHNbakaZwdgjKxjGK6tzeq3CVuprutgyB9hRZpGJx",
				"5EjLVB3TL6dz4NQoDnnfTqZrdJwbTtbHZGSz45aUrCFzGtXS",
				"5CJGHiC3LTTAwdisp1BUMi7YGvrytxxoU6JjhDS8yTf7xik4",
			];

			Self::remove_operator_assigned_era_and_staking_ledger(accounts);

			// Insert cluster 1 data
			Self::insert_cluster_data(
				0,
				ClusterType::Disabled,
				&[
					"5CSVucKUzY3hKxgXP25DJ642KEwDbe9TaCpeYni8AHpBJ4Wx",
					"5F3tW1dUPvd9h6B8drsY4oTk9v8HCGps8Y4hyRRa4ujZGkdf",
					"5En7dSHaRuSwk8GEiyGKwrstfWmqLCjMVcwrB8cKWsPhghak",
					"5ESA3Pm4tQWvMN3hpCR7z4TfKUGmEt7dGxYVBFs8cfmbAQGe",
					"5DLMmayCoqG5JWSwicQtfD86VZUdP2QPMiTVL5rbnfLfDNMf",
				],
			);

			// Insert cluster 2 data
			Self::insert_cluster_data(
				1,
				ClusterType::Admin,
				&[
					"5ELdU85Eh9TyRXyWzbSwQvPHGpG2vKLYFsA8sCizx6mHWevg",
					"5EEz4Yg8bxdn5BpM5XdCFmGh7gBHLyVgssAmvBpnqrbD5BwX",
					"5GekFCenV6z5hyu4L8yuWS2F3H9A5B6jfKmFeKzvZCQWkkEM",
					"5F7T7smXKyVaXGDbeurH3iC5kx8srXY3RC6tbV3PQ2X4bMnM",
					"5GHtmbtASxU2nrz62wT8VBmz6SJm3tKZUXgAyCBRmg81wBes",
				],
			);

			// Insert cluster 3 data
			Self::insert_cluster_data(
				2,
				ClusterType::Admin,
				&[
					"5CkfFDaSNiEYpbfTKfyXVPU1Yp5ziTKhGD4LBBVMmGaRLayp",
					"5EePft2cvAWrQLaqXNmC1zhdkLWjuWD5Atw7xNXUCkY1XEPC",
					"5HhBaok16yFsfQaG8HWDfNzLqFGDTQbD7dbKMAm8c9JdvADN",
					"5GsS2zStdsJAPFTFkStgvPuCP2FRMGC2TDNW6SV27MAesRrs",
					"5Fsy7iHE1c8Lw1Gjgfph6baqXVHPVPiieEVjY7raWJdLC3vS",
				],
			);

			// Insert cluster 4 data
			Self::insert_cluster_data(
				3,
				ClusterType::Public,
				&[
					"5GddBSC9121ZTGzkfozwb5JqkFwnWaNPB8WmLpJUxdTdabTS",
					"5GhJ2RVC23zkx2ZhA8fBHLmbSPp4isyMQ5XDoibmVVe3HQ6d",
					"5FRhBg7PizTCXx1HdrtT31vMe6jbhkRM1yvSfyBi5UUumzBY",
					"5FbdGaMSKms16jfLX4n4tvfCgRFuG8dLtoLBiQqBJtomH1TU",
					"5Dd5jXUAU3Gxt3oyeSoQ4qy9w3M2hNrjhGnGwtqs1Bwx9BjT",
				],
			);

			// Insert cluster 5 data
			Self::insert_cluster_data(
				4,
				ClusterType::Public,
				&[
					"5FTaoaJ38Vp5txJ1aeR8oksLiNt2o7TopSwTsdWMhb7c7v6n",
					"5GW9rkdj6qfW1YukvYXSxxNL8aiiGsNV3kz1hT2eJnCy3dLj",
					"5EvYGK8scFtc926ADqmAy2h5jzxy4MV7VjyT7ioFG8a6eUd2",
					"5FnXiYJSjhtFL5UnBKqo99MH7PSNgjUAt2iPxwnqtfUDPDpz",
					"5ENWWScYzUyfxR9vV28DWUfSx6jsZ8ccn992Q4RAshKRmtq8",
				],
			);

			// Insert cluster 6 data
			Self::insert_cluster_data(5, ClusterType::Public, &[]);

			let data_to_insert = vec![
				("5F7T7smXKyVaXGDbeurH3iC5kx8srXY3RC6tbV3PQ2X4bMnM", 539u32.into()),
				("5FbdGaMSKms16jfLX4n4tvfCgRFuG8dLtoLBiQqBJtomH1TU", 539u32.into()),
				("5EEz4Yg8bxdn5BpM5XdCFmGh7gBHLyVgssAmvBpnqrbD5BwX", 539u32.into()),
				("5HhBaok16yFsfQaG8HWDfNzLqFGDTQbD7dbKMAm8c9JdvADN", 539u32.into()),
				("5GekFCenV6z5hyu4L8yuWS2F3H9A5B6jfKmFeKzvZCQWkkEM", 539u32.into()),
				("5GHtmbtASxU2nrz62wT8VBmz6SJm3tKZUXgAyCBRmg81wBes", 539u32.into()),
				("5FnXiYJSjhtFL5UnBKqo99MH7PSNgjUAt2iPxwnqtfUDPDpz", 539u32.into()),
				("5F3tW1dUPvd9h6B8drsY4oTk9v8HCGps8Y4hyRRa4ujZGkdf", 539u32.into()),
				("5EvYGK8scFtc926ADqmAy2h5jzxy4MV7VjyT7ioFG8a6eUd2", 539u32.into()),
				("5FRhBg7PizTCXx1HdrtT31vMe6jbhkRM1yvSfyBi5UUumzBY", 539u32.into()),
				("5GhJ2RVC23zkx2ZhA8fBHLmbSPp4isyMQ5XDoibmVVe3HQ6d", 539u32.into()),
				("5Fsy7iHE1c8Lw1Gjgfph6baqXVHPVPiieEVjY7raWJdLC3vS", 539u32.into()),
				("5Dd5jXUAU3Gxt3oyeSoQ4qy9w3M2hNrjhGnGwtqs1Bwx9BjT", 539u32.into()),
				("5ESA3Pm4tQWvMN3hpCR7z4TfKUGmEt7dGxYVBFs8cfmbAQGe", 539u32.into()),
				("5DLMmayCoqG5JWSwicQtfD86VZUdP2QPMiTVL5rbnfLfDNMf", 539u32.into()),
				("5GsS2zStdsJAPFTFkStgvPuCP2FRMGC2TDNW6SV27MAesRrs", 539u32.into()),
				("5En7dSHaRuSwk8GEiyGKwrstfWmqLCjMVcwrB8cKWsPhghak", 539u32.into()),
				("5ELdU85Eh9TyRXyWzbSwQvPHGpG2vKLYFsA8sCizx6mHWevg", 539u32.into()),
				("5FTaoaJ38Vp5txJ1aeR8oksLiNt2o7TopSwTsdWMhb7c7v6n", 539u32.into()),
				("5CSVucKUzY3hKxgXP25DJ642KEwDbe9TaCpeYni8AHpBJ4Wx", 539u32.into()),
				("5CkfFDaSNiEYpbfTKfyXVPU1Yp5ziTKhGD4LBBVMmGaRLayp", 539u32.into()),
				("5GddBSC9121ZTGzkfozwb5JqkFwnWaNPB8WmLpJUxdTdabTS", 539u32.into()),
				("5EePft2cvAWrQLaqXNmC1zhdkLWjuWD5Atw7xNXUCkY1XEPC", 539u32.into()),
				("5ENWWScYzUyfxR9vV28DWUfSx6jsZ8ccn992Q4RAshKRmtq8", 539u32.into()),
				("5GW9rkdj6qfW1YukvYXSxxNL8aiiGsNV3kz1hT2eJnCy3dLj", 539u32.into()),
			];

			Self::insert_operator_assigned_era_data(&data_to_insert);

			let ledger_data_to_insert = vec![
				(
					"5F7T7smXKyVaXGDbeurH3iC5kx8srXY3RC6tbV3PQ2X4bMnM",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5FbdGaMSKms16jfLX4n4tvfCgRFuG8dLtoLBiQqBJtomH1TU",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5EEz4Yg8bxdn5BpM5XdCFmGh7gBHLyVgssAmvBpnqrbD5BwX",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5HhBaok16yFsfQaG8HWDfNzLqFGDTQbD7dbKMAm8c9JdvADN",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GekFCenV6z5hyu4L8yuWS2F3H9A5B6jfKmFeKzvZCQWkkEM",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GHtmbtASxU2nrz62wT8VBmz6SJm3tKZUXgAyCBRmg81wBes",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5FnXiYJSjhtFL5UnBKqo99MH7PSNgjUAt2iPxwnqtfUDPDpz",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5F3tW1dUPvd9h6B8drsY4oTk9v8HCGps8Y4hyRRa4ujZGkdf",
					0u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5EvYGK8scFtc926ADqmAy2h5jzxy4MV7VjyT7ioFG8a6eUd2",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5FRhBg7PizTCXx1HdrtT31vMe6jbhkRM1yvSfyBi5UUumzBY",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GhJ2RVC23zkx2ZhA8fBHLmbSPp4isyMQ5XDoibmVVe3HQ6d",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5Fsy7iHE1c8Lw1Gjgfph6baqXVHPVPiieEVjY7raWJdLC3vS",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5Dd5jXUAU3Gxt3oyeSoQ4qy9w3M2hNrjhGnGwtqs1Bwx9BjT",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5ESA3Pm4tQWvMN3hpCR7z4TfKUGmEt7dGxYVBFs8cfmbAQGe",
					0u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5DLMmayCoqG5JWSwicQtfD86VZUdP2QPMiTVL5rbnfLfDNMf",
					0u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GsS2zStdsJAPFTFkStgvPuCP2FRMGC2TDNW6SV27MAesRrs",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5En7dSHaRuSwk8GEiyGKwrstfWmqLCjMVcwrB8cKWsPhghak",
					0u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5ELdU85Eh9TyRXyWzbSwQvPHGpG2vKLYFsA8sCizx6mHWevg",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5FTaoaJ38Vp5txJ1aeR8oksLiNt2o7TopSwTsdWMhb7c7v6n",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5CSVucKUzY3hKxgXP25DJ642KEwDbe9TaCpeYni8AHpBJ4Wx",
					0u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5CkfFDaSNiEYpbfTKfyXVPU1Yp5ziTKhGD4LBBVMmGaRLayp",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GddBSC9121ZTGzkfozwb5JqkFwnWaNPB8WmLpJUxdTdabTS",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5EePft2cvAWrQLaqXNmC1zhdkLWjuWD5Atw7xNXUCkY1XEPC",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5ENWWScYzUyfxR9vV28DWUfSx6jsZ8ccn992Q4RAshKRmtq8",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GW9rkdj6qfW1YukvYXSxxNL8aiiGsNV3kz1hT2eJnCy3dLj",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
			];

			Self::insert_tee_staking_ledger_data(&ledger_data_to_insert);

			Ok(().into())
		}
	}
}

// Helper Methods for Storage
impl<T: Config> Pallet<T> {
	/// Increment the cluster id generator and return the id
	fn get_next_cluster_id() -> ClusterId {
		let id = NextClusterId::<T>::get();
		let next_id = id
			.checked_add(1)
			.expect("If u32 is not enough we should crash for safety; qed.");
		NextClusterId::<T>::put(next_id);

		id
	}

	/// The account ID of the tee pot.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	pub fn calculate_highest_params(
		enclave_reports: &BoundedVec<MetricsServerReport<T::AccountId>, T::ListSizeLimit>,
	) -> HighestParamsResponse {
		let mut highest_params =
			HighestParamsResponse { param_1: 0, param_2: 0, param_3: 0, param_4: 0, param_5: 0 };

		for report in enclave_reports.iter() {
			highest_params.param_1 = highest_params.param_1.max(report.param_1);
			highest_params.param_2 = highest_params.param_2.max(report.param_2);
			highest_params.param_3 = highest_params.param_3.max(report.param_3);
			highest_params.param_4 = highest_params.param_4.max(report.param_4);
			highest_params.param_5 = highest_params.param_5.max(report.param_5);
		}

		highest_params
	}

	pub fn calculate_weighted_sum(
		variances: &HighestParamsResponse,
		weightages: &ReportParamsWeightage,
	) -> u32 {
		// Calculate the weighted sum for each index
		let weighted_sum: u32 = (variances.param_1 as u32)
			.saturating_mul(weightages.param_1_weightage as u32) +
			(variances.param_2 as u32).saturating_mul(weightages.param_2_weightage as u32) +
			(variances.param_3 as u32).saturating_mul(weightages.param_3_weightage as u32) +
			(variances.param_4 as u32).saturating_mul(weightages.param_4_weightage as u32) +
			(variances.param_5 as u32).saturating_mul(weightages.param_5_weightage as u32);

		weighted_sum
	}

	fn clear_old_era(old_era: EraIndex) {
		let mut cursor = ClaimedRewards::<T>::clear_prefix(old_era, u32::MAX, None);
		debug_assert!(cursor.maybe_cursor.is_none());

		cursor = MetricsReports::<T>::clear_prefix(old_era, u32::MAX, None);
		debug_assert!(cursor.maybe_cursor.is_none());
	}

	///fn convert_str_to_valid_account_id(account_address: &str) -> Result<T::AccountId, Error<T>>
	///This function is to convert given string of SS58 address to AccountId type.
	pub fn convert_str_to_valid_account_id(
		account_address: &str,
	) -> Result<T::AccountId, Error<T>>
//where <T as frame_system::Config>::AccountId: sp_std::default::Default
	{
		let mut output = [0xFF; 48];
		let checksum_len = 2; //for substrate address
		let decoded = bs58::decode(account_address).into(&mut output).unwrap();
		let address_32: sp_core::crypto::AccountId32 =
			AccountId32::try_from(&output[1..decoded - checksum_len]).unwrap();
		let account_id: T::AccountId =
			T::AccountId::decode(&mut AccountId32::as_ref(&address_32)).unwrap();
		Ok(account_id)
	}

	pub fn insert_cluster_data(
		cluster_id: ClusterId,
		cluster_type: ClusterType,
		enclave_addresses: &[&str],
	) {
		let mut temp_enclaves = Vec::new(); // Regular Vec for easier manipulation

		for (index, &address) in enclave_addresses.iter().enumerate() {
			let account_id = <crate::Pallet<T>>::convert_str_to_valid_account_id(address).unwrap();
			temp_enclaves.push((account_id, index as SlotId));
		}

		// Convert Vec to BoundedVec
		let enclaves: BoundedVec<(T::AccountId, SlotId), T::ClusterSize> =
			BoundedVec::try_from(temp_enclaves).unwrap();

		let cluster = Cluster::new(enclaves, cluster_type.clone());
		ClusterData::<T>::insert(cluster_id, cluster);
	}

	pub fn insert_operator_assigned_era_data(operator_era_pairs: &[(&str, EraIndex)]) {
		for (operator, era) in operator_era_pairs.iter() {
			let account_id = <crate::Pallet<T>>::convert_str_to_valid_account_id(operator).unwrap();
			OperatorAssignedEra::<T>::insert(account_id, era);
		}
	}

	pub fn insert_tee_staking_ledger_data(ledger_data: &[(&str, BalanceOf<T>, bool, u32)]) {
		for &(operator_str, staked_amount, is_unlocking, unbonded_at) in ledger_data.iter() {
			let operator =
				<crate::Pallet<T>>::convert_str_to_valid_account_id(operator_str).unwrap();

			let ledger = TeeStakingLedger::new(
				operator.clone(),
				staked_amount.into(),
				is_unlocking,
				unbonded_at.into(),
			);

			// Assuming you have a storage named `StakingLedgers` for this ledger type.
			StakingLedger::<T>::insert(operator.clone(), ledger);
		}
	}

	pub fn remove_operator_assigned_era_and_staking_ledger(accounts: &[&str]) {
		for operator in accounts.iter() {
			let account_id = <crate::Pallet<T>>::convert_str_to_valid_account_id(operator).unwrap();
			OperatorAssignedEra::<T>::remove(account_id.clone());
			StakingLedger::<T>::remove(account_id);
		}
	}
}

impl<T: Config> traits::TEEExt for Pallet<T> {
	type AccountId = T::AccountId;
	type MaxUriLen = T::MaxUriLen;

	/// Check that an enclave address is valid and associated with a cluster
	fn ensure_enclave(enclave_address: Self::AccountId) -> Option<(u32, Self::AccountId)> {
		let mut result: Option<(ClusterId, Self::AccountId)> = None;
		if let Some(operator_address) = EnclaveAccountOperator::<T>::get(enclave_address) {
			if let Some(cluster_id) = EnclaveClusterId::<T>::get(&operator_address) {
				result = Some((cluster_id, operator_address));
			}
		}
		result
	}

	/// Register and assign an enclave in a cluster
	fn register_and_assign_enclave(
		operator_address: Self::AccountId,
		enclave_address: Self::AccountId,
		cluster_id: Option<ClusterId>,
	) -> DispatchResult {
		EnclaveAccountOperator::<T>::insert(enclave_address, operator_address.clone());
		EnclaveClusterId::<T>::insert(operator_address, cluster_id.unwrap_or(0u32));
		Ok(())
	}

	fn fill_unregistration_list(address: Self::AccountId, number: u8) -> DispatchResult {
		EnclaveUnregistrations::<T>::try_mutate(|x| -> DispatchResult {
			*x = BoundedVec::try_from(vec![address.clone(); number as usize]).unwrap();
			Ok(())
		})?;
		Ok(())
	}
}
