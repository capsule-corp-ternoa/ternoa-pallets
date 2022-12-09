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

use frame_support::dispatch::DispatchResultWithPostInfo;
pub use pallet::*;
pub use types::*;

use frame_support::traits::StorageVersion;
use sp_runtime::traits::StaticLookup;
use ternoa_common::traits;
pub use weights::WeightInfo;
use sp_std::{vec, vec::Vec};

use primitives::{
	nfts::{ClusterId},
};

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement::KeepAlive, OnUnbalanced, WithdrawReasons},
	};
	use frame_system::pallet_prelude::*;
	// use sp_runtime::traits::StaticLookup;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;

	// Declaration Enclave Id

	pub type EnclaveProviderName = Vec<u8>;

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
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registers enclave providers on chain :- ITL, AMD
		/// Different manufacturers can provide different enclave
		#[pallet::weight(T::WeightInfo::register_enclave_provider())]
		pub fn register_enclave_provider(
			origin: OriginFor<T>,
			enclave_provider_name: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let (id, new_id) = Self::new_provider_id()?;

			let provider = EnclaveProvider::new(enclave_provider_name.clone());

			let enclave_provider_exists = !EnclaveProviderRegistry::<T>::iter_values()
				.find(|x| x.enclave_provider_name.eq(&enclave_provider_name))
				.is_some();

			ensure!(enclave_provider_exists, Error::<T>::EnclaveProviderAlreadyRegistered);

			EnclaveProviderRegistry::<T>::insert(id, provider);

			ProviderIdGenerator::<T>::put(new_id);

			// Subscriber should capture the corresponding `enclave_id` for the given provider
			Self::deposit_event(Event::RegisterEnclaveProvider { id, enclave_provider_name });

			Ok(().into())
		}

		/// Given provider may have different processor architectures (enclave_class)
		/// and for a given enclave class there can be different public keys
		#[pallet::weight(T::WeightInfo::register_provider_keys())]
		pub fn register_provider_keys(
			origin: OriginFor<T>,
			provider_id: ProviderId,
			enclave_class: Option<Vec<u8>>,
			public_key: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let account_id = ensure_signed(origin)?;

			// EnclaveId does not present in Enclave Provider Registry
			ensure!(
				EnclaveProviderRegistry::<T>::contains_key(provider_id),
				Error::<T>::UnregisteredEnclaveProvider
			);

			// Entry registered for the provider key
			ensure!(
				!ProviderKeys::<T>::contains_key(provider_id),
				Error::<T>::ProviderAlreadyRegistered
			);

			// The provided public key already assigned to another enclave provider
			let enclave_provider_exists = !ProviderKeys::<T>::iter_values()
				.find(|x| x.public_key.eq(&public_key.clone()))
				.is_some();

			ensure!(
				enclave_provider_exists,
				Error::<T>::PublicKeyRegisteredForDifferentEnclaveProvider
			);

			let record = <EnclaveProviderKeys<T::AccountId>>::new(
				enclave_class.clone(),
				account_id.clone(),
				public_key.clone(),
			);

			ProviderKeys::<T>::insert(provider_id, record);

			Self::deposit_event(Event::RegisterEnclaveProviderKeys {
				account_id,
				provider_id,
				enclave_class,
				public_key,
			});

			Ok(().into())
		}

		/// Allows to register an enclave operator account
		/// - `enclave_id`: Valid Registered EnclaveId
		/// - `operator`: Valid enclave operator account
		/// Stores in
		/// 	pub type EnclaveOperator<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId,
		/// EnclaveId, OptionQuery> Checks
		/// 	enclave operator already registered -> AccountAlreadyRegisteredForEnclave
		///		enclaveId registered -> AssigningOperatorForUnknownEnclaveId
		#[pallet::weight(T::WeightInfo::register_enclave_operator())]
		pub fn register_enclave_operator(
			origin: OriginFor<T>,
			operator: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			let (id, new_id) = Self::new_enclave_operator_id()?;
			let operator_acc = T::Lookup::lookup(operator.clone())?;

			ensure!(
				!EnclaveOperatorRegistry::<T>::contains_key(operator_acc.clone()),
				Error::<T>::EnclaveOperatorExists
			);

			EnclaveOperatorRegistry::<T>::insert(operator_acc.clone(), id);
			EnclaveOperatorIdGenerator::<T>::put(new_id);

			Self::deposit_event(Event::RegisterEnclaveOperator {
				operator: operator_acc,
				enclave_operator_id: id,
			});

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::register_enclave())]
		pub fn register_enclave(
			origin: OriginFor<T>,
			ra_report: Vec<u8>, // JSON file
			api_uri: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;
			// let mut valid = true;

			// let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

			// let attestation: serde_json::Value = serde_json::from_slice(&ra_report).unwrap();
			// let report = attestation["raReport"].as_str().unwrap().as_bytes();
			// let signature =
			// 	hex::decode(attestation["signature"].as_str().unwrap().as_bytes()).unwrap();
			// let raw_signing_cert =
			// 	hex::decode(attestation["rawSigningCert"].as_str().unwrap().as_bytes()).unwrap();

			// Do we need this confidential report?
			// let res = Self::validate_ias_report(
			// 	report,
			// 	&signature,
			// 	&raw_signing_cert,
			// 	now
			// );
			//
			// match res {
			// 	Err(_) => {valid = false;}
			// 	_ => {}
			// }
			//
			// ensure!(valid, Error::<T>::InvalidIASSigningCert);

			ensure!(
				EnclaveOperatorRegistry::<T>::contains_key(account.clone()),
				Error::<T>::UnknownEnclaveOperatorAccount
			);

			ensure!(api_uri.len() < T::MaxUriLen::get().into(), Error::<T>::UriTooLong);
			ensure!(api_uri.len() > T::MinUriLen::get().into(), Error::<T>::UriTooShort);

			ensure!(
				!EnclaveIndex::<T>::contains_key(&account),
				Error::<T>::PublicKeyAlreadyTiedToACluster
			);
			let (enclave_id, new_id) = Self::new_enclave_id()?;
			// Needs to have enough money
			let imbalance = T::Currency::withdraw(
				&account,
				T::EnclaveFee::get(),
				WithdrawReasons::FEE,
				KeepAlive,
			)?;
			T::FeesCollector::on_unbalanced(imbalance);

			let enclave = Enclave::new(api_uri.clone());

			EnclaveIndex::<T>::insert(account.clone(), enclave_id);
			EnclaveRegistry::<T>::insert(enclave_id, enclave);
			EnclaveIdGenerator::<T>::put(new_id);

			Self::deposit_event(Event::AddedEnclave { account, api_uri, enclave_id });
			Ok(().into())
		}

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
			let enclave_id = EnclaveIndex::<T>::get(&account).ok_or(Error::<T>::NotEnclaveOwner)?;

			ensure!(
				!ClusterIndex::<T>::contains_key(enclave_id),
				Error::<T>::EnclaveAlreadyAssigned,
			);

			ClusterRegistry::<T>::mutate(cluster_id, |cluster_opt| {
				if let Some(cluster) = cluster_opt {
					if cluster.enclaves.len() >= T::ClusterSize::get() as usize {
						return Err(Error::<T>::ClusterIsAlreadyFull)
					}

					cluster.enclaves.push(enclave_id);
					ClusterIndex::<T>::insert(enclave_id, cluster_id);

					Ok(())
				} else {
					Err(Error::<T>::UnknownClusterId)
				}
			})?;

			Self::deposit_event(Event::AssignedEnclave { enclave_id, cluster_id });
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
			let enclave_id = EnclaveIndex::<T>::get(&account).ok_or(Error::<T>::NotEnclaveOwner)?;
			let cluster_id =
				ClusterIndex::<T>::get(enclave_id).ok_or(Error::<T>::EnclaveNotAssigned)?;

			ClusterRegistry::<T>::mutate(cluster_id, |cluster_opt| {
				if let Some(cluster) = cluster_opt {
					let index = cluster
						.enclaves
						.iter()
						.position(|x| *x == enclave_id)
						.ok_or(Error::<T>::InternalLogicalError)?;
					cluster.enclaves.remove(index);
					ClusterIndex::<T>::remove(enclave_id);
					Ok(())
				} else {
					Err(Error::<T>::UnknownClusterId)
				}
			})?;

			Self::deposit_event(Event::UnAssignedEnclave { enclave_id });
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
			let enclave_id = EnclaveIndex::<T>::get(&account).ok_or(Error::<T>::NotEnclaveOwner)?;

			ensure!(api_uri.len() < T::MaxUriLen::get().into(), Error::<T>::UriTooLong);
			ensure!(api_uri.len() > T::MinUriLen::get().into(), Error::<T>::UriTooShort);

			EnclaveRegistry::<T>::mutate(enclave_id, |enclave| -> DispatchResult {
				let enclave = enclave.as_mut().ok_or(Error::<T>::UnknownEnclaveId)?;
				enclave.api_uri = api_uri.clone();

				Ok(())
			})?;

			Self::deposit_event(Event::UpdatedEnclave { enclave_id, api_uri });
			Ok(().into())
		}

		/// `change_enclave_owner` changes the owner of an enclave
		///
		/// Arguments:
		///
		/// * `origin`: OriginFor<T> - The origin of the call.
		/// * `new_owner`: The new owner of the enclave.
		///
		/// Returns:
		///
		/// DispatchResultWithPostInfo
		#[pallet::weight(T::WeightInfo::change_enclave_owner())]
		pub fn change_enclave_owner(
			origin: OriginFor<T>,
			new_owner: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let old_owner = ensure_signed(origin)?;
			let new_owner = T::Lookup::lookup(new_owner)?;

			let enclave_id =
				EnclaveIndex::<T>::get(old_owner.clone()).ok_or(Error::<T>::NotEnclaveOwner)?;

			ensure!(
				!EnclaveIndex::<T>::contains_key(&new_owner),
				Error::<T>::PublicKeyAlreadyTiedToACluster
			);

			ensure!(EnclaveRegistry::<T>::contains_key(enclave_id), Error::<T>::UnknownEnclaveId);

			EnclaveIndex::<T>::remove(old_owner);
			EnclaveIndex::<T>::insert(new_owner.clone(), enclave_id);

			Self::deposit_event(Event::NewEnclaveOwner { enclave_id, owner: new_owner });
			Ok(().into())
		}

		// Creates a Cluster
		// A given cluster has list of enclaves
		#[pallet::weight(T::WeightInfo::create_cluster())]
		pub fn create_cluster(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let id = ClusterIdGenerator::<T>::get();
			let new_id = id.checked_add(1).ok_or(Error::<T>::ClusterIdOverflow)?;
			let cluster = Cluster::new(Default::default());

			ClusterRegistry::<T>::insert(id, cluster);
			ClusterIdGenerator::<T>::put(new_id);

			Self::deposit_event(Event::AddedCluster { cluster_id: id });
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::remove_cluster())]
		pub fn remove_cluster(
			origin: OriginFor<T>,
			cluster_id: ClusterId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			ClusterRegistry::<T>::mutate(cluster_id, |cluster_opt| {
				if let Some(cluster) = cluster_opt {
					for enclave_id in &cluster.enclaves {
						if ClusterIndex::<T>::take(enclave_id).is_none() {
							return Err(Error::<T>::InternalLogicalError)
						}
					}
					Ok(())
				} else {
					Err(Error::<T>::UnknownClusterId)
				}
			})?;
			ClusterRegistry::<T>::take(cluster_id);

			Self::deposit_event(Event::RemovedCluster { cluster_id });
			Ok(().into())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Enclave
		AddedEnclave {
			account: T::AccountId,
			api_uri: Vec<u8>,
			enclave_id: EnclaveId,
		},
		AssignedEnclave {
			enclave_id: EnclaveId,
			cluster_id: ClusterId,
		},
		UnAssignedEnclave {
			enclave_id: EnclaveId,
		},
		UpdatedEnclave {
			enclave_id: EnclaveId,
			api_uri: Vec<u8>,
		},
		NewEnclaveOwner {
			enclave_id: EnclaveId,
			owner: T::AccountId,
		},
		// Cluster
		AddedCluster {
			cluster_id: ClusterId,
		},
		RemovedCluster {
			cluster_id: ClusterId,
		},

		RegisterEnclaveProvider {
			id: EnclaveId,
			enclave_provider_name: Vec<u8>,
		},
		RegisterEnclaveProviderKeys {
			account_id: T::AccountId,
			provider_id: ProviderId,
			enclave_class: Option<Vec<u8>>,
			public_key: Vec<u8>,
		},
		RegisterEnclaveOperator {
			operator: T::AccountId,
			enclave_operator_id: EnclaveOperatorId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		UnknownEnclaveId,
		UnknownClusterId,
		NotEnclaveOwner,
		PublicKeyAlreadyTiedToACluster,
		UriTooShort,
		UriTooLong,
		EnclaveIdOverflow,
		ClusterIdOverflow,
		ClusterIsAlreadyFull,
		EnclaveAlreadyAssigned,
		EnclaveNotAssigned,
		CannotAssignToSameCluster,
		InternalLogicalError,
		ProviderIdOverflow,
		EnclaveProviderAlreadyRegistered,
		UnregisteredEnclaveProvider,
		ProviderAlreadyRegistered,
		PublicKeyRegisteredForDifferentEnclaveProvider,
		AssigningOperatorForUnknownEnclaveId,
		EnclaveOperatorExists,
		UnknownEnclaveOperatorAccount,
		InvalidIASSigningCert,
	}

	//
	// Enclave
	//
	#[pallet::storage]
	#[pallet::getter(fn enclave_registry)]
	pub type EnclaveRegistry<T: Config> =
		StorageMap<_, Blake2_128Concat, EnclaveId, Enclave, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn enclave_id_generator)]
	pub type EnclaveIdGenerator<T: Config> = StorageValue<_, EnclaveId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn enclave_index)]
	pub type EnclaveIndex<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, EnclaveId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn enclave_operator)]
	pub type EnclaveOperatorRegistry<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, EnclaveOperatorId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn cluster_registry)]
	pub type ClusterRegistry<T: Config> =
		StorageMap<_, Blake2_128Concat, ClusterId, Cluster, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn cluster_id_generator)]
	pub type ClusterIdGenerator<T: Config> = StorageValue<_, ClusterId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn cluster_index)]
	pub type ClusterIndex<T: Config> =
		StorageMap<_, Blake2_128Concat, EnclaveId, ClusterId, OptionQuery>;

	/// Creating a storage item called ProviderIdGenerator.
	#[pallet::storage]
	#[pallet::getter(fn provider_id_generator)]
	pub type ProviderIdGenerator<T: Config> = StorageValue<_, ProviderId, ValueQuery>;

	/// Creating a storage item called EnclaveOperatorIdGenerator.
	#[pallet::storage]
	#[pallet::getter(fn enclave_operator_id_generator)]
	pub type EnclaveOperatorIdGenerator<T: Config> = StorageValue<_, EnclaveOperatorId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn enclave_provider_keys)]
	pub type ProviderKeys<T: Config> =
		StorageMap<_, Blake2_128Concat, ProviderId, EnclaveProviderKeys<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn enclave_provider)]
	pub type EnclaveProviderRegistry<T: Config> =
		StorageMap<_, Blake2_128Concat, ProviderId, EnclaveProvider, OptionQuery>;

	// #[pallet::genesis_config]
	// pub struct GenesisConfig<T: Config> {
	// 	pub enclaves: Vec<(T::AccountId, EnclaveId, Vec<u8>)>,
	// 	pub clusters: Vec<(ClusterId, Vec<EnclaveId>)>,
	// }
	//
	// #[cfg(feature = "std")]
	// impl<T: Config> Default for GenesisConfig<T> {
	// 	fn default() -> Self {
	// 		Self { enclaves: Default::default(), clusters: Default::default() }
	// 	}
	// }
	//
	// #[pallet::genesis_build]
	// impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
	// 	fn build(&self) {
	// 		let enclaves = self.enclaves.clone();
	// 		if let Some(enclave) = enclaves.last() {
	// 			EnclaveIdGenerator::<T>::put(enclave.1 + 1);
	// 		}
	//
	// 		for enclave in enclaves {
	// 			EnclaveIndex::<T>::insert(enclave.0, enclave.1);
	// 			EnclaveRegistry::<T>::insert(enclave.1, Enclave { api_uri: enclave.2 });
	// 		}
	//
	// 		let clusters = self.clusters.clone();
	// 		if let Some(cluster) = clusters.last() {
	// 			ClusterIdGenerator::<T>::put(cluster.0 + 1);
	// 		}
	//
	// 		for cluster in clusters {
	// 			for enclave_id in cluster.1.iter() {
	// 				ClusterIndex::<T>::insert(*enclave_id, cluster.0);
	// 			}
	// 			ClusterRegistry::<T>::insert(cluster.0, Cluster::new(cluster.1));
	// 		}
	// 	}
	// }
}

// Helper Methods for Storage
impl<T: Config> Pallet<T> {
// 	pub fn validate_ias_report(
// 		report: &[u8],
// 		signature: &[u8],
// 		raw_signing_cert: &[u8],
// 		report_timestamp: u64,
// 	) -> Result<ConfidentialReport, ReportError> {
// 		// Validate report
// 		let sig_cert = webpki::EndEntityCert::try_from(raw_signing_cert);
// 		let sig_cert = sig_cert.or(Err(ReportError::InvalidIASSigningCert))?;
// 		let verify_result =
// 			sig_cert.verify_signature(&webpki::RSA_PKCS1_2048_8192_SHA256, report, signature);
// 		verify_result.or(Err(ReportError::InvalidIASSigningCert))?;
//
// 		// Validate certificate
// 		let chain: Vec<&[u8]> = Vec::new();
// 		let time_now = webpki::Time::from_seconds_since_unix_epoch(report_timestamp);
// 		let tls_server_cert_valid = sig_cert.verify_is_valid_tls_server_cert(
// 			SUPPORTED_SIG_ALGS,
// 			&IAS_SERVER_ROOTS,
// 			&chain,
// 			time_now,
// 		);
// 		tls_server_cert_valid.or(Err(ReportError::InvalidIASSigningCert))?;
//
// 		let (ias_fields, _) = IasFields::from_ias_report(report)?;
//
// 		let pruntime_hash = ias_fields.extend_mrenclave();
//
// 		// Check the following fields
// 		Ok(ConfidentialReport {
// 			provider: Some(AttestationProvider::Ias),
// 			runtime_hash: pruntime_hash,
// 			confidence_level: ias_fields.confidence_level,
// 		})
// 	}

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

	/// It generates a new provider id.
	///
	/// Returns: New Provider
	///
	/// A new ProviderId
	pub fn new_provider_id() -> Result<(ProviderId, ProviderId), Error<T>> {
		let id: ProviderId = ProviderIdGenerator::<T>::get();
		let new_id: u32 = id.checked_add(1).ok_or(Error::<T>::ProviderIdOverflow)?;
		Ok((id, new_id))
	}

	pub fn new_enclave_operator_id() -> Result<(EnclaveOperatorId, EnclaveOperatorId), Error<T>> {
		let id: EnclaveOperatorId = EnclaveOperatorIdGenerator::<T>::get();
		let new_id: u32 = id.checked_add(1).ok_or(Error::<T>::ProviderIdOverflow)?;
		Ok((id, new_id))
	}
}

impl<T: Config> traits::TEEExt for Pallet<T> {
	type AccountId = T::AccountId;
	// type ClusterId = u32;
	type EnclaveId = u32;

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
	fn ensure_enclave(account: Self::AccountId) -> Option<(Self::ClusterId, Self::EnclaveId)> {
		let mut result: Option<(Self::ClusterId, Self::EnclaveId)> = None;
		let enclave_id: Option<EnclaveId> = EnclaveIndex::<T>::get(account);
		match enclave_id {
			Some(enc_id) => {
				let cluster_id = ClusterIndex::<T>::get(enc_id).unwrap();
				let cluster = ClusterRegistry::<T>::get(cluster_id).unwrap();
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
