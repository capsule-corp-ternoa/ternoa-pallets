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

use frame_support::{dispatch::DispatchResultWithPostInfo, BoundedVec};
pub use pallet::*;
pub use types::*;

use frame_support::traits::StorageVersion;
use primitives::tee::{ClusterId, EnclaveId};
use sp_std::{vec, vec::Vec};
use ternoa_common::traits;
pub use weights::WeightInfo;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement::KeepAlive, OnUnbalanced, WithdrawReasons},
	};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for pallet.
		type WeightInfo: WeightInfo;

		/// Currency type.
		type Currency: Currency<Self::AccountId>;

		/// What we do with additional fees
		type FeesCollector: OnUnbalanced<NegativeImbalanceOf<Self>>;

		// Constants
		/// Host much does it cost to mint enclave (extra fee on top of the tx fees)
		#[pallet::constant]
		type EnclaveFee: Get<BalanceOf<Self>>;

		/// Size of a cluster
		#[pallet::constant]
		type ClusterSize: Get<u32>;

		/// Min Uri len
		#[pallet::constant]
		type MinUriLen: Get<u16>;

		/// Max Uri len
		#[pallet::constant]
		type MaxUriLen: Get<u16>;

		/// Max Assigned Enclaves
		#[pallet::constant]
		type MaxRegisteredEnclaves: Get<u32>;

		/// Max Unassigned Enclaves
		#[pallet::constant]
		type MaxUnRegisteredEnclaves: Get<u32>;
	}

	/// Register Enclaves
	/// List of registered EnclaveIds in a sized vec
	#[pallet::storage]
	#[pallet::getter(fn registered_enclaves)]
	pub type EnclaveRegistrationList<T: Config> =
		StorageValue<_, BoundedVec<EnclaveId, T::MaxRegisteredEnclaves>, ValueQuery>;

	/// Unregister Enclaves
	/// List of un registered EnclaveIds in a sized vec
	#[pallet::storage]
	#[pallet::getter(fn unregistered_enclaves)]
	pub type EnclaveUnregistrationList<T: Config> =
		StorageValue<_, BoundedVec<EnclaveId, T::MaxUnRegisteredEnclaves>, ValueQuery>;

	/// Enclave Registry
	/// Stores Enclaves with api_uri and enclave_address
	#[pallet::storage]
	#[pallet::getter(fn enclave_registry)]
	#[pallet::unbounded]
	pub type EnclaveData<T: Config> =
		StorageMap<_, Blake2_128Concat, EnclaveId, Enclave, OptionQuery>;

	/// Holds generated EnclaveIds
	#[pallet::storage]
	#[pallet::getter(fn enclave_id_generator)]
	pub type EnclaveIdGenerator<T: Config> = StorageValue<_, EnclaveId, ValueQuery>;

	/// Map stores AccountEnclaveId
	#[pallet::storage]
	#[pallet::getter(fn enclave_index)]
	pub type AccountEnclaveId<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, EnclaveId, OptionQuery>;

	/// Map stores Cluster information
	#[pallet::storage]
	#[pallet::getter(fn cluster_registry)]
	#[pallet::unbounded]
	pub type ClusterData<T: Config> =
		StorageMap<_, Blake2_128Concat, ClusterId, Cluster, OptionQuery>;

	/// Holds generated ClusterIds
	#[pallet::storage]
	#[pallet::getter(fn cluster_id_generator)]
	pub type ClusterIdGenerator<T: Config> = StorageValue<_, ClusterId, ValueQuery>;

	/// Map stores EnclaveId | ClusterId
	#[pallet::storage]
	#[pallet::getter(fn cluster_index)]
	pub type EnclaveClusterId<T: Config> =
		StorageMap<_, Blake2_128Concat, EnclaveId, ClusterId, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New Enclave account got added
		EnclaveAdded { account: T::AccountId, api_uri: Vec<u8>, enclave_id: EnclaveId },
		/// An enclave got unregistered
		EnclaveUnregistered { account_id: T::AccountId, enclave_id: EnclaveId },
		/// An enclave moved for unregistration to a queue
		MovedForUnregistration { account_id: T::AccountId, enclave_id: EnclaveId },
		/// An enclave got assigned to a cluster
		EnclaveAssigned { enclave_id: EnclaveId, cluster_id: ClusterId },
		/// An enclave got assigned
		EnclaveUnassigned { enclave_id: EnclaveId },
		/// An enclave got updated
		EnclaveUpdated { enclave_id: EnclaveId, api_uri: Vec<u8> },
		/// Enclave force updated
		EnclaveForceUpdated { enclave_id: EnclaveId, enclave_address: Vec<u8>, api_uri: Vec<u8> },
		/// New cluster got added
		ClusterAdded { cluster_id: ClusterId },
		/// Cluster got removed
		ClusterRemoved { cluster_id: ClusterId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Unknown enclaveId
		UnknownEnclaveId,
		/// Unknown ClusterId
		UnknownClusterId,
		/// Account does not associated with an enclave
		NotEnclaveOwner,
		/// Enclave address registered to an account
		EnclaveAddressAlreadyRegisteredtoTheAccount,
		/// Enclave URI is short
		UriTooShort,
		/// Enclave URI is long
		UriTooLong,
		/// Maximum enclaves reached
		EnclaveIdOverflow,
		/// Maximum clusters reached
		ClusterIdOverflow,
		/// Cluster is already full, cannot assign any enclaves
		ClusterIsAlreadyFull,
		/// Enclave already assigned to a cluster
		EnclaveAlreadyAssigned,
		/// Enclave not assigned to a cluster
		EnclaveNotAssigned,
		/// Enclave does not exists
		EnclaveDoesNotExists,
		/// Cannot assign to same cluster
		CannotAssignToSameCluster,
		/// Internal logical error
		InternalLogicalError,
		/// Assigning an operator to an invalid enclaveId
		AssigningOperatorForUnknownEnclaveId,
		/// Unknown enclave operator account
		UnknownEnclaveOperatorAccount,
		/// Invalid IAS sign certificate
		InvalidIASSigningCert,
		/// Error Adding To Queue
		ErrorAddingToQueue,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::register_enclave())]
		pub fn register_enclave(
			origin: OriginFor<T>,
			enclave_address: Vec<u8>,
			api_uri: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;

			ensure!(api_uri.len() < T::MaxUriLen::get().into(), Error::<T>::UriTooLong);
			ensure!(api_uri.len() > T::MinUriLen::get().into(), Error::<T>::UriTooShort);

			let result = AccountEnclaveId::<T>::get(&account)
				.and_then(|enclave_id| EnclaveData::<T>::get(enclave_id).map(|enc| enc));

			if result.is_some() {
				let enclave = result.ok_or(Error::<T>::EnclaveDoesNotExists)?;
				ensure!(
					enclave.enclave_address != enclave_address,
					Error::<T>::EnclaveAddressAlreadyRegisteredtoTheAccount
				)
			}

			let (enclave_id, new_id) = Self::new_enclave_id()?;
			// Needs to have enough money
			let imbalance = T::Currency::withdraw(
				&account,
				T::EnclaveFee::get(),
				WithdrawReasons::FEE,
				KeepAlive,
			)?;
			T::FeesCollector::on_unbalanced(imbalance);

			let enclave = Enclave::new(api_uri.clone(), enclave_address);

			AccountEnclaveId::<T>::insert(account.clone(), enclave_id);
			EnclaveData::<T>::insert(enclave_id, enclave);
			EnclaveIdGenerator::<T>::put(new_id);

			let reg_enclaves: BoundedVec<EnclaveId, T::MaxRegisteredEnclaves> =
				<EnclaveRegistrationList<T>>::get();

			let reg_enclaves = reg_enclaves
				.try_mutate(|v| v.push(enclave_id))
				.ok_or(Error::<T>::ErrorAddingToQueue)?;
			<EnclaveRegistrationList<T>>::put(reg_enclaves);

			Self::deposit_event(Event::EnclaveAdded { account, api_uri, enclave_id });
			Ok(().into())
		}

		/// Removes an enclave from the system
		/// Origin- operator account address
		/// If the enclave is assigned, it will be placed in queue for tech committee approval
		/// If enclave is not already assigned, he can exit without permission.
		#[pallet::weight(T::WeightInfo::unregister_enclave())]
		pub fn unregister_enclave(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let account = ensure_signed_or_root(origin)?;
			match account {
				Some(account_id) => {
					let enclave_id = AccountEnclaveId::<T>::get(account_id.clone())
						.ok_or(Error::<T>::UnknownEnclaveOperatorAccount)?;

					let unregistered_enclaves: BoundedVec<EnclaveId, T::MaxUnRegisteredEnclaves> =
						<EnclaveUnregistrationList<T>>::get();

					let reg_enclaves = <EnclaveRegistrationList<T>>::get();

					if !reg_enclaves.contains(&enclave_id) {
						EnclaveData::<T>::remove(enclave_id);
						Self::deposit_event(Event::EnclaveUnregistered { account_id, enclave_id });
					} else {
						let unregistered_enclaves = unregistered_enclaves
							.try_mutate(|v| v.push(enclave_id))
							.ok_or(Error::<T>::ErrorAddingToQueue)?;
						<EnclaveUnregistrationList<T>>::put(unregistered_enclaves);
						Self::deposit_event(Event::MovedForUnregistration {
							account_id,
							enclave_id,
						});
					}
				},
				_ => {},
			}

			Ok(().into())
		}

		/// Update registration can be called only BEFORE assign_enclave.
		#[pallet::weight(T::WeightInfo::register_enclave())]
		pub fn update_registration(
			origin: OriginFor<T>,
			enclave_address: Vec<u8>,
			api_uri: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;

			ensure!(api_uri.len() < T::MaxUriLen::get().into(), Error::<T>::UriTooLong);
			ensure!(api_uri.len() > T::MinUriLen::get().into(), Error::<T>::UriTooShort);

			let enclave_id =
				AccountEnclaveId::<T>::get(&account).ok_or(Error::<T>::NotEnclaveOwner)?;

			ensure!(
				!EnclaveClusterId::<T>::contains_key(enclave_id),
				Error::<T>::EnclaveAlreadyAssigned,
			);

			EnclaveData::<T>::try_mutate(enclave_id, |enc| -> DispatchResult {
				let enc = enc.as_mut().ok_or(Error::<T>::EnclaveDoesNotExists)?;
				enc.api_uri = api_uri.clone();
				enc.enclave_address = enclave_address.clone();
				Ok(())
			})?;

			Self::deposit_event(Event::EnclaveForceUpdated {
				enclave_id,
				enclave_address,
				api_uri,
			});

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::register_enclave())]
		pub fn force_update_enclave(
			origin: OriginFor<T>,
			enclave_id: EnclaveId,
			enclave_address: Vec<u8>,
			api_uri: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ensure!(api_uri.len() < T::MaxUriLen::get().into(), Error::<T>::UriTooLong);
			ensure!(api_uri.len() > T::MinUriLen::get().into(), Error::<T>::UriTooShort);

			EnclaveData::<T>::try_mutate(enclave_id, |enc| -> DispatchResult {
				let enc = enc.as_mut().ok_or(Error::<T>::EnclaveDoesNotExists)?;
				enc.api_uri = api_uri.clone();
				enc.enclave_address = enclave_address.clone();
				Ok(())
			})?;

			Self::deposit_event(Event::EnclaveForceUpdated {
				enclave_id,
				enclave_address,
				api_uri,
			});
			Ok(().into())
		}

		/// ***** For this we donot need to pass enclave_address>?
		/// `assign_enclave` assigns an enclave to a cluster
		///
		/// Arguments:
		///
		/// * `origin`: OriginFor<T> - The origin of the call.
		/// * `cluster_id`: The id of the cluster to assign the enclave to.
		///
		/// Returns:
		///
		/// DispatchResultWithPostInfo
		#[pallet::weight(T::WeightInfo::assign_enclave())]
		pub fn assign_enclave(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;
			let enclave_id =
				AccountEnclaveId::<T>::get(&account).ok_or(Error::<T>::NotEnclaveOwner)?;

			ensure!(
				!EnclaveClusterId::<T>::contains_key(enclave_id),
				Error::<T>::EnclaveAlreadyAssigned,
			);

			ClusterData::<T>::mutate(cluster_id, |cluster_opt| {
				if let Some(cluster) = cluster_opt {
					if cluster.enclaves.len() >= T::ClusterSize::get() as usize {
						return Err(Error::<T>::ClusterIsAlreadyFull)
					}

					cluster.enclaves.push(enclave_id);
					EnclaveClusterId::<T>::insert(enclave_id, cluster_id);

					Ok(())
				} else {
					Err(Error::<T>::UnknownClusterId)
				}
			})?;

			Self::deposit_event(Event::EnclaveAssigned { enclave_id, cluster_id });
			Ok(().into())
		}

		/// `unassign_enclave` removes the enclave from the cluster
		///
		/// Arguments:
		///
		/// * `origin`: OriginFor<T>
		///
		/// Returns:
		///
		/// DispatchResultWithPostInfo
		#[pallet::weight(T::WeightInfo::unassign_enclave())]
		pub fn unassign_enclave(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;
			let enclave_id =
				AccountEnclaveId::<T>::get(&account).ok_or(Error::<T>::NotEnclaveOwner)?;
			let cluster_id =
				EnclaveClusterId::<T>::get(enclave_id).ok_or(Error::<T>::EnclaveNotAssigned)?;

			ClusterData::<T>::mutate(cluster_id, |cluster_opt| {
				if let Some(cluster) = cluster_opt {
					let index = cluster
						.enclaves
						.iter()
						.position(|x| *x == enclave_id)
						.ok_or(Error::<T>::InternalLogicalError)?;
					cluster.enclaves.remove(index);

					EnclaveClusterId::<T>::remove(enclave_id);

					EnclaveData::<T>::remove(enclave_id);

					// get un registered enclaves
					let mut unregistered_enclaves: BoundedVec<
						EnclaveId,
						T::MaxUnRegisteredEnclaves,
					> = <EnclaveUnregistrationList<T>>::get();
					// get registered enclaves
					let mut reg_enclaves = <EnclaveRegistrationList<T>>::get();

					// Clean up from registered enclaves
					match reg_enclaves.binary_search(&enclave_id) {
						Ok(idx) => {
							reg_enclaves.remove(idx);
							<EnclaveRegistrationList<T>>::put(reg_enclaves);
						},
						Err(_) => {},
					}

					// Clean up from unregistered enclaves
					match unregistered_enclaves.binary_search(&enclave_id) {
						Ok(idx) => {
							unregistered_enclaves.remove(idx);
							<EnclaveUnregistrationList<T>>::put(unregistered_enclaves);
						},
						Err(_) => {},
					}

					Ok(())
				} else {
					Err(Error::<T>::UnknownClusterId)
				}
			})?;

			Self::deposit_event(Event::EnclaveUnassigned { enclave_id });
			Ok(().into())
		}

		/// `update_enclave` updates the API URI of an enclave
		///
		/// Arguments:
		///
		/// * `origin`: OriginFor<T>
		/// * `api_uri`: The URI of the enclave's API.
		///
		/// Returns:
		///
		/// DispatchResultWithPostInfo
		#[pallet::weight(T::WeightInfo::update_enclave())]
		pub fn update_enclave(
			origin: OriginFor<T>,
			api_uri: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;
			let enclave_id =
				AccountEnclaveId::<T>::get(&account).ok_or(Error::<T>::NotEnclaveOwner)?;

			ensure!(api_uri.len() < T::MaxUriLen::get().into(), Error::<T>::UriTooLong);
			ensure!(api_uri.len() > T::MinUriLen::get().into(), Error::<T>::UriTooShort);

			EnclaveData::<T>::mutate(enclave_id, |enclave| -> DispatchResult {
				let enclave = enclave.as_mut().ok_or(Error::<T>::UnknownEnclaveId)?;
				enclave.api_uri = api_uri.clone();

				Ok(())
			})?;

			Self::deposit_event(Event::EnclaveUpdated { enclave_id, api_uri });
			Ok(().into())
		}

		// Creates a Cluster
		// A given cluster has list of enclaves
		#[pallet::weight(T::WeightInfo::register_cluster())]
		pub fn register_cluster(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let id = ClusterIdGenerator::<T>::get();
			let new_id = id.checked_add(1).ok_or(Error::<T>::ClusterIdOverflow)?;
			let cluster = Cluster::new(Default::default());

			ClusterData::<T>::insert(id, cluster);
			ClusterIdGenerator::<T>::put(new_id);

			Self::deposit_event(Event::ClusterAdded { cluster_id: id });
			Ok(().into())
		}

		/// Removes a cluster
		/// Mandate call
		/// Cluster must be empty
		#[pallet::weight(T::WeightInfo::unregister_cluster())]
		pub fn unregister_cluster(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ClusterData::<T>::mutate(cluster_id, |cluster_opt| {
				if let Some(cluster) = cluster_opt {
					for enclave_id in &cluster.enclaves {
						if EnclaveClusterId::<T>::take(enclave_id).is_none() {
							return Err(Error::<T>::InternalLogicalError)
						}
					}
					Ok(())
				} else {
					Err(Error::<T>::UnknownClusterId)
				}
			})?;
			ClusterData::<T>::take(cluster_id);

			Self::deposit_event(Event::ClusterRemoved { cluster_id });
			Ok(().into())
		}
	}
}

// Helper Methods for Storage
impl<T: Config> Pallet<T> {
	// TODO: Replace these functions with a generic function or a Macro

	/// `new_enclave_id` returns a tuple of the current enclave id and the next enclave id
	///
	/// Returns:
	///
	/// A tuple of the current enclave id and the next enclave id.
	pub fn new_enclave_id() -> Result<(EnclaveId, EnclaveId), Error<T>> {
		let id = EnclaveIdGenerator::<T>::get();
		let new_id = id.checked_add(1).ok_or(Error::<T>::EnclaveIdOverflow)?;

		Ok((id, new_id))
	}

	/// > This function returns a tuple of the current cluster id and the next cluster id
	///
	/// Returns:
	///
	/// A tuple of the current cluster id and the next cluster id.
	pub fn new_cluster_id() -> Result<(ClusterId, ClusterId), Error<T>> {
		let id: ClusterId = ClusterIdGenerator::<T>::get();
		let new_id: u32 = id.checked_add(1).ok_or(Error::<T>::ClusterIdOverflow)?;
		Ok((id, new_id))
	}
}

impl<T: Config> traits::TEEExt for Pallet<T> {
	type AccountId = T::AccountId;

	/// > If the account has an enclave, and the enclave is in the cluster, return the cluster and
	/// > enclave
	/// id
	///
	/// Arguments:
	///
	/// * `account`: The account that is trying to access the enclave.
	///
	/// Returns:
	///
	/// A tuple of the cluster id and the enclave id.
	fn ensure_enclave(account: Self::AccountId) -> Option<(ClusterId, EnclaveId)> {
		let mut result: Option<(ClusterId, EnclaveId)> = None;
		let enclave_id: Option<EnclaveId> = AccountEnclaveId::<T>::get(account);
		match enclave_id {
			Some(enc_id) => {
				let cluster_id =
					EnclaveClusterId::<T>::get(enc_id).ok_or(Error::<T>::UnknownEnclaveId).ok()?;
				let cluster =
					ClusterData::<T>::get(cluster_id).ok_or(Error::<T>::UnknownClusterId).ok()?;
				result = if cluster.enclaves.contains(&enc_id) {
					Some((cluster_id, enc_id))
				} else {
					None
				}
			},
			None => (),
		}
		result
	}
}
