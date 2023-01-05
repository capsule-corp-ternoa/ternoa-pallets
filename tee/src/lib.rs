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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

mod types;
mod weights;

use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo},
	BoundedVec,
};
pub use pallet::*;
pub use types::*;

use frame_support::traits::StorageVersion;
use primitives::tee::ClusterId;
use ternoa_common::traits;
pub use weights::WeightInfo;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	use frame_support::{pallet_prelude::*, traits::Currency};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for pallet.
		type WeightInfo: WeightInfo;

		/// Currency type.
		type Currency: Currency<Self::AccountId>;

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
	#[pallet::unbounded]
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
	#[pallet::unbounded]
	pub type ClusterData<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ClusterId,
		Cluster<T::AccountId, T::ClusterSize>,
		OptionQuery,
	>;

	/// Holds generated ClusterIds
	#[pallet::storage]
	#[pallet::getter(fn cluster_id_generator)]
	pub type ClusterIdGenerator<T: Config> = StorageValue<_, ClusterId, ValueQuery>;

	/// Map stores Enclave operator | ClusterId
	#[pallet::storage]
	#[pallet::getter(fn enclave_cluster_id)]
	pub type EnclaveClusterId<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, ClusterId, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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
		/// An enclave update request unregistered
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
		/// The cluster does not exists
		ClusterNotFound,
		/// Cluster id does not exist for this address
		ClusterIdNotFound,
		/// The cluster still have enclaves associated to it
		ClusterIsNotEmpty,
		/// Cluster is already full, cannot assign any enclaves
		ClusterIsFull,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Ask for an enclave registration
		#[pallet::weight(T::WeightInfo::register_enclave())]
		pub fn register_enclave(
			origin: OriginFor<T>,
			enclave_address: T::AccountId,
			api_uri: BoundedVec<u8, T::MaxUriLen>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				EnclaveRegistrations::<T>::get(&who).is_none(),
				Error::<T>::RegistrationAlreadyExists
			);
			ensure!(EnclaveData::<T>::get(&who).is_none(), Error::<T>::OperatorAlreadyExists);
			ensure!(
				EnclaveAccountOperator::<T>::get(&enclave_address).is_none(),
				Error::<T>::EnclaveAddressAlreadyExists
			);

			let enclave = Enclave::new(enclave_address.clone(), api_uri.clone());
			EnclaveRegistrations::<T>::insert(who.clone(), enclave);

			Self::deposit_event(Event::EnclaveAddedForRegistration {
				operator_address: who,
				enclave_address,
				api_uri,
			});
			Ok(().into())
		}

		/// Ask for an enclave to be removed.
		/// No need for approval if the enclave registration was not approved yet.
		#[pallet::weight(T::WeightInfo::unregister_enclave())]
		pub fn unregister_enclave(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			match EnclaveData::<T>::get(&who) {
				Some(_) => {
					EnclaveUnregistrations::<T>::try_mutate(|x| -> DispatchResult {
						ensure!(!x.contains(&who), Error::<T>::UnregistrationAlreadyExists);
						x.try_push(who.clone())
							.map_err(|_| Error::<T>::UnregistrationLimitReached)?;
						Ok(())
					})?;
					Self::deposit_event(Event::MovedForUnregistration { operator_address: who });
				},
				None => {
					EnclaveRegistrations::<T>::remove(who.clone());
					Self::deposit_event(Event::RegistrationRemoved { operator_address: who });
				},
			}

			Ok(().into())
		}

		/// Ask for enclave update
		#[pallet::weight(T::WeightInfo::update_enclave())]
		pub fn update_enclave(
			origin: OriginFor<T>,
			new_enclave_address: T::AccountId,
			new_api_uri: BoundedVec<u8, T::MaxUriLen>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let enclave = EnclaveData::<T>::get(&who).ok_or(Error::<T>::EnclaveNotFound)?;

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

		/// Assign an enclave to a cluster
		#[pallet::weight(T::WeightInfo::assign_enclave())]
		pub fn assign_enclave(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
			cluster_id: ClusterId,
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
							.try_push(operator_address.clone())
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
		#[pallet::weight(T::WeightInfo::remove_registration())]
		pub fn remove_registration(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveRegistrations::<T>::remove(operator_address.clone());
			Self::deposit_event(Event::RegistrationRemoved { operator_address });
			Ok(().into())
		}


		/// Remove an enclave update request from storage
		#[pallet::weight(T::WeightInfo::remove_update_request())]
		pub fn remove_update_request(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			EnclaveUpdates::<T>::remove(operator_address.clone());
			Self::deposit_event(Event::UpdateRequestRemoved { operator_address });
			Ok(().into())
		}

		/// Unassign an enclave from a cluster and remove all information
		#[pallet::weight(T::WeightInfo::remove_enclave())]
		pub fn remove_enclave(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveData::<T>::try_mutate(&operator_address, |maybe_enclave| -> DispatchResult {
				let enclave = maybe_enclave.as_mut().ok_or(Error::<T>::EnclaveNotFound)?;

				ensure!(
					EnclaveAccountOperator::<T>::get(&enclave.enclave_address).is_some(),
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

					// Remove the operator from cluster
					if let Some(index) =
						cluster.enclaves.iter().position(|x| *x == operator_address.clone())
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
		#[pallet::weight(T::WeightInfo::force_update_enclave())]
		pub fn force_update_enclave(
			origin: OriginFor<T>,
			operator_address: T::AccountId,
			new_enclave_address: T::AccountId,
			new_api_uri: BoundedVec<u8, T::MaxUriLen>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			EnclaveData::<T>::try_mutate(&operator_address, |maybe_enclave| -> DispatchResult {
				let enclave = maybe_enclave.as_mut().ok_or(Error::<T>::EnclaveNotFound)?;

				if enclave.enclave_address != new_enclave_address {
					ensure!(
						EnclaveAccountOperator::<T>::get(&new_enclave_address).is_none(),
						Error::<T>::EnclaveAddressAlreadyExists
					);
					EnclaveAccountOperator::<T>::remove(enclave.enclave_address.clone());
					EnclaveAccountOperator::<T>::insert(
						new_enclave_address.clone(),
						operator_address.clone(),
					);
				}

				enclave.enclave_address = new_enclave_address.clone();
				enclave.api_uri = new_api_uri.clone();
				Ok(())
			})?;
			EnclaveUpdates::<T>::remove(operator_address.clone());

			Self::deposit_event(Event::EnclaveUpdated {
				operator_address,
				new_enclave_address,
				new_api_uri,
			});
			Ok(().into())
		}

		// Creates an empty Cluster
		#[pallet::weight(T::WeightInfo::create_cluster())]
		pub fn create_cluster(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let id = Self::get_next_cluster_id();
			let cluster = Cluster::new(Default::default());
			ClusterData::<T>::insert(id, cluster);
			Self::deposit_event(Event::ClusterAdded { cluster_id: id });
			Ok(().into())
		}

		/// Removes an empty cluster
		#[pallet::weight(T::WeightInfo::remove_cluster())]
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
	}
}

// Helper Methods for Storage
impl<T: Config> Pallet<T> {
	/// Increment the cluster id generator and return the id
	fn get_next_cluster_id() -> ClusterId {
		let id = ClusterIdGenerator::<T>::get();
		let next_id = id
			.checked_add(1)
			.expect("If u32 is not enough we should crash for safety; qed.");
		ClusterIdGenerator::<T>::put(next_id);

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
}
