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
	BoundedVec,
};
pub use pallet::*;
pub use types::*;

use frame_support::traits::{Get, LockIdentifier, OnRuntimeUpgrade, StorageVersion};
use sp_std::vec;

use primitives::tee::{ClusterId, SlotId};
use sp_runtime::SaturatedConversion;
use ternoa_common::traits;
pub use weights::WeightInfo;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);
const TEE_STAKING_ID: LockIdentifier = *b"teestake";
use sp_staking::{EraIndex};
use pallet_staking::{Pallet as Staking};

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

	pub type BalanceOf<T> =
    <<T as pallet_staking::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_staking::Config	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for pallet.
		type TeeWeightInfo: WeightInfo;

		// /// Currency type.
		// type Currency: LockableCurrency<
		// 	Self::AccountId,
		// 	Moment = Self::BlockNumber,
		// 	Balance = Self::CurrencyBalance,
		// >;

		// /// Just the `Currency::Balance` type; we have this item to allow us to constrain it to
		// /// `From<u64>`.
		// type CurrencyBalance: sp_runtime::traits::AtLeast32BitUnsigned
		// 	+ parity_scale_codec::FullCodec
		// 	+ Copy
		// 	+ MaybeSerializeDeserialize
		// 	+ sp_std::fmt::Debug
		// 	+ Default
		// 	+ From<u64>
		// 	+ TypeInfo
		// 	+ MaxEncodedLen;
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

		/// Default staking amount for TEE.
		#[pallet::constant]
		type InitialDailyRewards: Get<BalanceOf<Self>>;

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
	#[pallet::getter(fn nft_mint_fee)]
	pub(super) type MetricsServer<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::ListSizeLimit>, ValueQuery>;

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
		TeeStakingLedger<T::AccountId, T::BlockNumber>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn metrics_reports)]
	pub type MetricsReports<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, EraIndex>,
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::AccountId>,
		),
		MetricsServerReport<T::AccountId>,
	>;

	/// Report params weightage
	#[pallet::storage]
	#[pallet::getter(fn report_params_weightages)]
	pub type ReportParamsWeightages<T: Config> = StorageValue<_, ReportParamsWeightage, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn rewards)]
	pub type Rewards<T: Config> = StorageDoubleMap<_, Twox64Concat, EraIndex, Twox64Concat, T::AccountId, u64, OptionQuery>;


	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// #[cfg(feature = "try-runtime")]
		// fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// 	<migrations::v2::MigrationV2<T> as OnRuntimeUpgrade>::pre_upgrade()
		// }

		// fn on_initialize(now: T::BlockNumber) -> Weight {
		// 	// let sessions_per_era = T::SessionsPerEra::get();
		// 	// let mut weight = Weight::zero();

		// 	// let current_session_index = <frame_system::Pallet<T>>::block_number() % sessions_per_era.into();

		// 	// // if current_session_index == sessions_per_era - 1 {
		// 	// // 	// Last session of the era
		// 	// // 	// Perform your logic here
		// 	// // }
		// 	// weight
		// }

		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			let mut weight = Weight::zero();

			let version = StorageVersion::get::<Pallet<T>>();
			if version == StorageVersion::new(1) {
				weight = <migrations::v2::MigrationV2<T> as OnRuntimeUpgrade>::on_runtime_upgrade();

				StorageVersion::put::<Pallet<T>>(&StorageVersion::new(2));
			}

			weight
		}

		// #[cfg(feature = "try-runtime")]
		// fn post_upgrade(v: Vec<u8>) -> Result<(), &'static str> {
		// 	<migrations::v2::MigrationV2<T> as OnRuntimeUpgrade>::post_upgrade(v)
		// }
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
		/// New cluster got added
		ClusterAdded { cluster_id: ClusterId },
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
		MetricsServerAdded { metrics_server_address: T::AccountId },
		/// Metrics server report submitted
		MetricsServerReportSubmitted { era: EraIndex, metrics_server_address: T::AccountId, metrics_server_report: MetricsServerReport<T::AccountId> },
		/// Report parameters weightage modified
		ReportParamsWeightageModified {
			param_1_weightage: u8,
			param_2_weightage: u8,
			param_3_weightage: u8,
			param_4_weightage: u8,
			param_5_weightage: u8,
		 },
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
		/// Metrics server limit reached
		MetricsServerLimitReached,
		/// Metrics server address not found
		MetricsServerAddressNotFound,
		EnclaveNotFoundForTheOperator,
		FailedToGetActiveEra,
		ReportAlreadySubmittedForEra,
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

			ensure!(who.clone() != enclave_address.clone(), Error::<T>::OperatorAndEnclaveAreSame);
			ensure!(
				EnclaveRegistrations::<T>::get(&who).is_none(),
				Error::<T>::RegistrationAlreadyExists
			);
			ensure!(EnclaveData::<T>::get(&who).is_none(), Error::<T>::OperatorAlreadyExists);
			ensure!(
				EnclaveAccountOperator::<T>::get(&enclave_address).is_none(),
				Error::<T>::EnclaveAddressAlreadyExists
			);

			let default_staking_amount = StakingAmount::<T>::get();
			let stake_details = TeeStakingLedger::new(who.clone(), false, Default::default());
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
					let stake_details = TeeStakingLedger::new(who.clone(), true, now);
					StakingLedger::<T>::insert(who.clone(), stake_details);
					Self::deposit_event(Event::MovedForUnregistration {
						operator_address: who.clone(),
					});
					Self::deposit_event(Event::Unbonded {
						operator_address: who,
						amount: default_staking_amount,
					});
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

			ensure!(
				who.clone() != new_enclave_address.clone(),
				Error::<T>::OperatorAndEnclaveAreSame
			);

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
					if let Some(_) = maybe_registration {
						*maybe_registration = None;
					}
					Ok(())
				},
			)?;

			Self::deposit_event(Event::RegistrationRemoved { operator_address });
			Ok(().into())
		}

		/// Remove an enclave update request from storage
		#[pallet::weight(T::TeeWeightInfo::remove_update())]
		pub fn remove_update(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveUpdates::<T>::try_mutate(&operator_address, |maybe_update| -> DispatchResult {
				if let Some(_) = maybe_update {
					*maybe_update = None;
				}
				Ok(())
			})?;

			Self::deposit_event(Event::UpdateRequestRemoved { operator_address });
			Ok(().into())
		}

		/// Unassign an enclave from a cluster and remove all information
		#[pallet::weight(T::TeeWeightInfo::remove_enclave())]
		pub fn remove_enclave(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveData::<T>::try_mutate(&operator_address, |maybe_enclave| -> DispatchResult {
				let enclave = maybe_enclave.as_mut().ok_or(Error::<T>::EnclaveNotFound)?;

				ensure!(
					EnclaveAccountOperator::<T>::get(&operator_address).is_some(),
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
							if let Some(_) = maybe_update {
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

					// Remove the mapping between enclave address to operator address
					EnclaveAccountOperator::<T>::remove(&enclave.enclave_address);

					Ok(())
				})?;

				// Remove the enclave data
				*maybe_enclave = None;
				Ok(())
			})?;

			Self::deposit_event(Event::EnclaveRemoved { operator_address });
			Ok(().into())
		}

		/// Update an enclave and clean the enclaves to update if needed
		#[pallet::weight(T::TeeWeightInfo::force_update_enclave())]
		pub fn force_update_enclave(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
			new_enclave_address: T::AccountId,
			new_api_uri: BoundedVec<u8, T::MaxUriLen>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(
				operator_address.clone() != new_enclave_address.clone(),
				Error::<T>::OperatorAndEnclaveAreSame
			);

			EnclaveData::<T>::try_mutate(&operator_address, |maybe_enclave| -> DispatchResult {
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
			})?;
			EnclaveUpdates::<T>::try_mutate(&operator_address, |maybe_update| -> DispatchResult {
				if let Some(_) = maybe_update {
					*maybe_update = None;
				}
				Ok(())
			})?;

			Self::deposit_event(Event::EnclaveUpdated {
				operator_address,
				new_enclave_address,
				new_api_uri,
			});
			Ok(().into())
		}

		// Creates an empty Cluster
		#[pallet::weight(T::TeeWeightInfo::create_cluster())]
		pub fn create_cluster(origin: OriginFor<T>, is_public: bool) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let id = Self::get_next_cluster_id();
			let cluster = Cluster::new(Default::default(), is_public);
			ClusterData::<T>::insert(id, cluster);
			Self::deposit_event(Event::ClusterAdded { cluster_id: id });
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

		/// Ask for an enclave to be removed.
		/// No need for approval if the enclave registration was not approved yet.
		#[pallet::weight(T::TeeWeightInfo::unregister_enclave())]
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
		#[pallet::weight(T::TeeWeightInfo::unregister_enclave())]
		pub fn register_metrics_server(
			origin: OriginFor<T>,
			metrics_server_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			MetricsServer::<T>::try_mutate(|metrics_server| -> DispatchResult {
				ensure!(
					!metrics_server.contains(&metrics_server_address),
					Error::<T>::MetricsServerAlreadyExists
				);
				metrics_server
					.try_push(metrics_server_address.clone())
					.map_err(|_| Error::<T>::MetricsServerLimitReached)?;
				Ok(())
			})?;
			Self::deposit_event(Event::MetricsServerAdded { metrics_server_address });
			Ok(().into())
		}

		#[pallet::weight(T::TeeWeightInfo::unregister_enclave())]
		pub fn submit_metrics_server_report(
			origin: OriginFor<T>,
			era_index: Option<EraIndex>,
			metrics_server_report: MetricsServerReport<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
	
			// Check if the account is in the Metrics Server accounts storage
			if !MetricsServer::<T>::get().contains(&who) {
				return Err(Error::<T>::MetricsServerAddressNotFound.into());
			}
	
			// Check the validity of the operator address
			EnclaveData::<T>::get(&who)
			.ok_or(Error::<T>::EnclaveNotFoundForTheOperator)?;


			// Retrieve the era index
			let era_index = match era_index {
				Some(era) => era,
				None => {
					Staking::<T>::active_era()
						.map(|e| e.index)
						.ok_or(Error::<T>::FailedToGetActiveEra)?
						.saturating_sub(1)
				}
			};

			// Check if the report for the era, metrics server address, and operator address already exists
			let report_exists = MetricsReports::<T>::contains_key(&(era_index, &who, &metrics_server_report.operator_address));
			if report_exists {
				return Err(Error::<T>::ReportAlreadySubmittedForEra.into());
			}
	
			 // Store the metrics server report
			 MetricsReports::<T>::insert(&(era_index, &who, &metrics_server_report.operator_address), metrics_server_report.clone());

			// Emit an event for the successful submission
			 Self::deposit_event(Event::MetricsServerReportSubmitted {era: era_index, metrics_server_address: who.clone(), metrics_server_report: metrics_server_report});

			Ok(().into())

		}	
		
		/// Report parameters weightage modification which can be done by Technical Committee.
		#[pallet::weight(T::TeeWeightInfo::unregister_enclave())]
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

		/// Claim rewards by Era
		#[pallet::weight(T::TeeWeightInfo::unregister_enclave())]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			era: EraIndex,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;


			// Self::deposit_event(Event::ReportParamsWeightageModified { report_params_weightage });
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
