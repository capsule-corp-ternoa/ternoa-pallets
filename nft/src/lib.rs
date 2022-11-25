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

mod migrations;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

pub use pallet::*;

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{
		Currency, ExistenceRequirement::KeepAlive, Get, OnRuntimeUpgrade, OnUnbalanced,
		StorageVersion, WithdrawReasons,
	},
	BoundedVec,
};
use frame_system::pallet_prelude::*;
use primitives::{
	nfts::{Collection, CollectionId, NFTData, NFTId, NFTState},
	U8BoundedVec,
};
use sp_arithmetic::per_things::Permill;
use sp_runtime::traits::{CheckedSub, StaticLookup};
use sp_std::prelude::*;
use ternoa_common::traits;

pub use weights::WeightInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for pallet.
		type WeightInfo: WeightInfo;

		/// Currency type.
		type Currency: Currency<Self::AccountId>;

		/// What we do with additional fees.
		type FeesCollector: OnUnbalanced<NegativeImbalanceOf<Self>>;

		// Constants
		/// Default fee for minting NFTs.
		#[pallet::constant]
		type InitialMintFee: Get<BalanceOf<Self>>;

		/// Maximum offchain data length.
		#[pallet::constant]
		type NFTOffchainDataLimit: Get<u32>;

		/// Maximum collection length.
		#[pallet::constant]
		type CollectionSizeLimit: Get<u32>;

		/// Maximum collection offchain data length.
		#[pallet::constant]
		type CollectionOffchainDataLimit: Get<u32>;

		/// Default fee for minting secret NFTs.
		#[pallet::constant]
		type InitialSecretMintFee: Get<BalanceOf<Self>>;

		/// The number of necessary shards to consider the Secret NFT valid.
		#[pallet::constant]
		type ShardsNumber: Get<u32>;
	}

	/// How much does it cost to mint a NFT (extra fee on top of the tx fees).
	#[pallet::storage]
	#[pallet::getter(fn nft_mint_fee)]
	pub(super) type NftMintFee<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, T::InitialMintFee>;

	/// Counter for NFT ids.
	#[pallet::storage]
	#[pallet::getter(fn next_nft_id)]
	pub type NextNFTId<T: Config> = StorageValue<_, NFTId, ValueQuery>;

	/// Counter for collection ids.
	#[pallet::storage]
	#[pallet::getter(fn next_collection_id)]
	pub type NextCollectionId<T: Config> = StorageValue<_, CollectionId, ValueQuery>;

	/// Data related to NFTs.
	#[pallet::storage]
	#[pallet::getter(fn nfts)]
	pub type Nfts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NFTId,
		NFTData<T::AccountId, T::NFTOffchainDataLimit>,
		OptionQuery,
	>;

	/// Data related to collections.
	#[pallet::storage]
	#[pallet::getter(fn collections)]
	pub type Collections<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		CollectionId,
		Collection<T::AccountId, T::CollectionOffchainDataLimit, T::CollectionSizeLimit>,
		OptionQuery,
	>;

	/// Host a map of delegated NFTs and the recipient.
	#[pallet::storage]
	#[pallet::getter(fn delegated_nfts)]
	pub type DelegatedNFTs<T: Config> =
		StorageMap<_, Blake2_128Concat, NFTId, T::AccountId, OptionQuery>;

	/// How much does it cost to mint a secret NFT (extra fee on top of the tx fees and basic NFT
	/// fee).
	#[pallet::storage]
	#[pallet::getter(fn secret_nft_mint_fee)]
	pub(super) type SecretNftMintFee<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, T::InitialSecretMintFee>;

	/// Host a map of secret NFTs and their secret_offchain_data.
	#[pallet::storage]
	#[pallet::getter(fn secret_nfts_offchain_data)]
	pub type SecretNftsOffchainData<T: Config> =
		StorageMap<_, Blake2_128Concat, NFTId, U8BoundedVec<T::NFTOffchainDataLimit>, OptionQuery>;

	/// Host a map of secret NFTs and a vector of enclave addresses that sent a shard.
	#[pallet::storage]
	#[pallet::getter(fn secret_nfts_shards_count)]
	pub type SecretNftsShardsCount<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NFTId,
		BoundedVec<T::AccountId, T::ShardsNumber>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new NFT was created.
		NFTCreated {
			nft_id: NFTId,
			owner: T::AccountId,
			offchain_data: U8BoundedVec<T::NFTOffchainDataLimit>,
			royalty: Permill,
			collection_id: Option<CollectionId>,
			is_soulbound: bool,
			mint_fee: BalanceOf<T>,
		},
		/// An NFT was burned.
		NFTBurned { nft_id: NFTId },
		/// An NFT was transferred to someone else.
		NFTTransferred { nft_id: NFTId, sender: T::AccountId, recipient: T::AccountId },
		/// An NFT was delegated to someone else.
		NFTDelegated { nft_id: NFTId, recipient: Option<T::AccountId> },
		/// Royalty has been changed for an NFT.
		NFTRoyaltySet { nft_id: NFTId, royalty: Permill },
		/// NFT mint fee changed.
		NFTMintFeeSet { fee: BalanceOf<T> },
		/// A collection was created.
		CollectionCreated {
			collection_id: CollectionId,
			owner: T::AccountId,
			offchain_data: U8BoundedVec<T::CollectionOffchainDataLimit>,
			limit: Option<u32>,
		},
		/// A collection was burned.
		CollectionBurned { collection_id: CollectionId },
		/// A collection was closed.
		CollectionClosed { collection_id: CollectionId },
		/// A collection has limit set.
		CollectionLimited { collection_id: CollectionId, limit: u32 },
		/// An NFT has been added to a collection.
		NFTAddedToCollection { nft_id: NFTId, collection_id: CollectionId },
		/// A secret was added to a basic NFT.
		SecretAddedToNFT { nft_id: NFTId, offchain_data: U8BoundedVec<T::NFTOffchainDataLimit> },
		/// A shard was added for a secret NFT.
		ShardAdded { nft_id: NFTId, enclave: T::AccountId },
		/// A secret NFT has finished syncing shards.
		SecretNFTSynced { nft_id: NFTId },
		/// Secret NFT mint fee changed.
		SecretNFTMintFeeSet { fee: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Operation not allowed because the NFT is listed for sale.
		CannotTransferListedNFTs,
		/// Operation not allowed because the NFT is listed for sale.
		CannotBurnListedNFTs,
		/// Operation not allowed because the NFT is listed for sale.
		CannotDelegateListedNFTs,
		/// Operation not allowed because the NFT is listed for sale.
		CannotSetRoyaltyForListedNFTs,
		/// Operation is not allowed because the NFT is delegated.
		CannotTransferDelegatedNFTs,
		/// Operation is not allowed because the NFT secret is not synced.
		CannotTransferNotSyncedSecretNFTs,
		/// Operation is not allowed because the NFT is delegated.
		CannotBurnDelegatedNFTs,
		/// Operation is not allowed because the NFT is delegated.
		CannotSetRoyaltyForDelegatedNFTs,
		/// Operation is not allowed because the NFT is a capsule.
		CannotTransferCapsuleNFTs,
		/// Operation is not allowed because the NFT is a capsule.
		CannotBurnCapsuleNFTs,
		/// Operation is not allowed because the NFT is a capsule.
		CannotDelegateCapsuleNFTs,
		/// Operation is not allowed because the NFT is  and signer is not the creator.
		CannotTransferNotCreatedSoulboundNFTs,
		/// Operation is not allowed because the NFT is a capsule.
		CannotSetRoyaltyForCapsuleNFTs,
		/// Operation is not allowed because the NFT is owned by the caller.
		CannotTransferNFTsToYourself,
		/// Operation is not allowed because the NFT is rented
		CannotTransferRentedNFTs,
		/// Operation is not allowed because the NFT is rented
		CannotBurnRentedNFTs,
		/// Operation is not allowed because the NFT is rented
		CannotSetRoyaltyForRentedNFTs,
		/// Operation is not allowed because the NFT is rented
		CannotDelegateRentedNFTs,
		/// Operation is not allowed because the collection limit is too low.
		CollectionLimitExceededMaximumAllowed,
		/// No NFT was found with that NFT id.
		NFTNotFound,
		/// NFT id not found in collection nft list.
		NFTNotFoundInCollection,
		/// NFT already belong to a collection.
		NFTBelongToACollection,
		/// This function can only be called by the owner of the NFT.
		NotTheNFTOwner,
		/// This function can only be called by the creator of the NFT.
		NotTheNFTCreator,
		/// This function can only be called by the owner of the collection.
		NotTheCollectionOwner,
		/// No Collection was found with that NFT id.
		CollectionNotFound,
		/// Operation is not allowed because the collection is closed.
		CollectionIsClosed,
		/// Collection nft list has reached the selected limit.
		CollectionHasReachedLimit,
		/// Operation is not allowed because the collection is not empty.
		CollectionIsNotEmpty,
		/// Operation is not permitted because the collection's limit is already set.
		CollectionLimitAlreadySet,
		/// Operation is not permitted because the nfts number in the collection are greater than
		/// the new limit.
		CollectionHasTooManyNFTs,
		/// Operation is not permitted because collection nfts is full.
		CannotAddMoreNFTsToCollection,
		/// Operation is not permitted because caller is not a registered TEE enclave.
		NotARegisteredEnclave,
		/// Operation is not permitted because NFT is not a secret.
		NFTIsNotSecret,
		/// Operation is not permitted because NFT secret is already synced.
		NFTAlreadySynced,
		/// Operation is not permitted because NFT has already received all shards.
		NFTHasReceivedAllShards,
		/// Operation is not permitted because Enclave has already added its shard.
		EnclaveAlreadyAddedShard,
		/// Insufficient balance
		InsufficientBalance,
		/// Operation is not permitted because the NFT is listed.
		CannotAddSecretToListedNFTs,
		/// Operation is not permitted because the NFT is a capsule.
		CannotAddSecretToCapsuleNFTs,
		/// Operation is not permitted because the NFT is already a secret.
		CannotAddSecretToSecretNFTs,
		/// Feature is not available yet
		ComingSoon,
	}

	// TODO Write Tests for Runtime upgrade
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			<migrations::v2::MigrationV2<T> as OnRuntimeUpgrade>::pre_upgrade()
		}

		// This function is called when a runtime upgrade is called. We need to make sure that
		// what ever we do here won't brick the chain or leave the data in a invalid state.
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			let mut weight = Weight::zero();

			// Seems like we start with version 1 and not 0.
			let version = StorageVersion::get::<Pallet<T>>();
			if version == StorageVersion::new(1) || version == StorageVersion::new(0) {
				weight = <migrations::v2::MigrationV2<T> as OnRuntimeUpgrade>::on_runtime_upgrade();

				// Update the storage version.
				StorageVersion::put::<Pallet<T>>(&StorageVersion::new(2));
			}

			weight
		}

		// This function is called after a runtime upgrade is executed. Here we can
		// test if the new state of blockchain data is valid. It's important to say that
		// post_upgrade won't be called when a real runtime upgrade is executed.
		#[cfg(feature = "try-runtime")]
		fn post_upgrade(v: Vec<u8>) -> Result<(), &'static str> {
			<migrations::v2::MigrationV2<T> as OnRuntimeUpgrade>::post_upgrade(v)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new NFT with the provided details. An ID will be auto
		/// generated and logged as an event, The caller of this function
		/// will become the owner of the new NFT.
		#[pallet::weight((
            {
				if let Some(collection_id) = &collection_id {
					let collection = Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound);
					if let Ok(collection) = collection {
						let s = collection.nfts.len();
						T::WeightInfo::create_nft(s as u32)
					} else {
						T::WeightInfo::create_nft(1)
					}
				} else {
					T::WeightInfo::create_nft(1)
				}
            },
			DispatchClass::Normal
        ))]
		pub fn create_nft(
			origin: OriginFor<T>,
			offchain_data: U8BoundedVec<T::NFTOffchainDataLimit>,
			royalty: Permill,
			collection_id: Option<CollectionId>,
			is_soulbound: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut next_nft_id = None;

			// Checks
			// The Caller needs to pay the NFT mint fee.
			let mint_fee = NftMintFee::<T>::get();
			let reason = WithdrawReasons::FEE;
			let imbalance = T::Currency::withdraw(&who, mint_fee, reason, KeepAlive)?;
			T::FeesCollector::on_unbalanced(imbalance);

			// Throws an error if specified collection does not exist, signer is not owner,
			// collection is close, collection has reached limit.
			if let Some(collection_id) = &collection_id {
				Collections::<T>::try_mutate(collection_id, |x| -> DispatchResult {
					let collection = x.as_mut().ok_or(Error::<T>::CollectionNotFound)?;
					let limit =
						collection.limit.unwrap_or_else(|| T::CollectionSizeLimit::get()) as usize;
					ensure!(collection.owner == who, Error::<T>::NotTheCollectionOwner);
					ensure!(!collection.is_closed, Error::<T>::CollectionIsClosed);
					ensure!(collection.nfts.len() < limit, Error::<T>::CollectionHasReachedLimit);

					let tmp_nft_id = Self::get_next_nft_id();
					collection
						.nfts
						.try_push(tmp_nft_id)
						.map_err(|_| Error::<T>::CannotAddMoreNFTsToCollection)?;
					next_nft_id = Some(tmp_nft_id);
					Ok(().into())
				})?;
			}

			let nft_id = next_nft_id.unwrap_or_else(|| Self::get_next_nft_id());
			let nft = NFTData::new_default(
				who.clone(),
				offchain_data.clone(),
				royalty,
				collection_id.clone(),
				is_soulbound,
			);
			// Execute
			Nfts::<T>::insert(nft_id, nft);
			let event = Event::NFTCreated {
				nft_id,
				owner: who,
				offchain_data,
				royalty,
				collection_id,
				is_soulbound,
				mint_fee,
			};
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Remove an NFT from the storage. This operation is irreversible which means
		/// once the NFT is removed (burned) from the storage there is no way to
		/// get it back.
		/// Must be called by the owner of the NFT.
		#[pallet::weight((
            {
				let nft = Nfts::<T>::get(nft_id).ok_or(Error::<T>::NFTNotFound);
				if let Ok(nft) = nft {
					if let Some(collection_id) = &nft.collection_id {
						let collection = Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound);
						if let Ok(collection) = collection {
							let s = collection.nfts.len();
							T::WeightInfo::burn_nft(s as u32)
						} else {
							T::WeightInfo::burn_nft(1)
						}
					} else {
						T::WeightInfo::burn_nft(1)
					}
				} else {
					T::WeightInfo::burn_nft(1)
				}
            },
			DispatchClass::Normal
        ))]
		pub fn burn_nft(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let nft = Nfts::<T>::get(nft_id).ok_or(Error::<T>::NFTNotFound)?;

			// Checks
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(!nft.state.is_listed, Error::<T>::CannotBurnListedNFTs);
			ensure!(!nft.state.is_capsule, Error::<T>::CannotBurnCapsuleNFTs);
			ensure!(!nft.state.is_delegated, Error::<T>::CannotBurnDelegatedNFTs);
			ensure!(!nft.state.is_rented, Error::<T>::CannotBurnRentedNFTs);

			// Check for collection to remove nft.
			if let Some(collection_id) = &nft.collection_id {
				Collections::<T>::try_mutate(collection_id, |x| -> DispatchResult {
					let collection = x.as_mut().ok_or(Error::<T>::CollectionNotFound)?;
					let index = collection
						.nfts
						.iter()
						.position(|y| *y == nft_id)
						.ok_or(Error::<T>::NFTNotFoundInCollection)?;
					// Execute
					collection.nfts.swap_remove(index);
					Ok(().into())
				})?;
			}

			// Check for secret nft to remove secret offchain data and shards count.
			if nft.state.is_secret {
				SecretNftsOffchainData::<T>::remove(nft_id);
				if nft.state.is_syncing {
					SecretNftsShardsCount::<T>::remove(nft_id);
				}
			}

			// Execute
			Nfts::<T>::remove(nft_id);
			Self::deposit_event(Event::NFTBurned { nft_id });

			Ok(().into())
		}

		/// Transfer an NFT from an account to another one. Must be called by the
		/// owner of the NFT.
		#[pallet::weight(T::WeightInfo::transfer_nft())]
		pub fn transfer_nft(
			origin: OriginFor<T>,
			nft_id: NFTId,
			recipient: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let recipient = T::Lookup::lookup(recipient)?;

			Nfts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let nft = x.as_mut().ok_or(Error::<T>::NFTNotFound)?;

				// Checks
				ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
				ensure!(nft.owner != recipient, Error::<T>::CannotTransferNFTsToYourself);
				ensure!(!nft.state.is_listed, Error::<T>::CannotTransferListedNFTs);
				ensure!(!nft.state.is_capsule, Error::<T>::CannotTransferCapsuleNFTs);
				ensure!(!nft.state.is_delegated, Error::<T>::CannotTransferDelegatedNFTs);
				ensure!(
					!(nft.state.is_soulbound && nft.creator != nft.owner),
					Error::<T>::CannotTransferNotCreatedSoulboundNFTs
				);
				ensure!(!nft.state.is_syncing, Error::<T>::CannotTransferNotSyncedSecretNFTs);
				ensure!(!nft.state.is_rented, Error::<T>::CannotTransferRentedNFTs);

				// Execute
				nft.owner = recipient.clone();

				Ok(().into())
			})?;
			// Execute
			let event = Event::NFTTransferred { nft_id, sender: who, recipient };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Delegate an NFT to a recipient, does not change ownership.
		/// Must be called by NFT owner.
		#[pallet::weight(T::WeightInfo::delegate_nft())]
		pub fn delegate_nft(
			origin: OriginFor<T>,
			nft_id: NFTId,
			recipient: Option<<T::Lookup as StaticLookup>::Source>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let recipient_account_id = if let Some(recipient) = recipient {
				T::Lookup::lookup(recipient)?
			} else {
				who.clone()
			};
			let is_delegated = recipient_account_id != who;

			Nfts::<T>::try_mutate(nft_id, |maybe_nft| -> DispatchResult {
				let nft = maybe_nft.as_mut().ok_or(Error::<T>::NFTNotFound)?;

				// Checks
				ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
				ensure!(!nft.state.is_listed, Error::<T>::CannotDelegateListedNFTs);
				ensure!(!nft.state.is_capsule, Error::<T>::CannotDelegateCapsuleNFTs);
				ensure!(!nft.state.is_rented, Error::<T>::CannotDelegateRentedNFTs);

				// Execute
				nft.state.is_delegated = is_delegated;

				Ok(().into())
			})?;

			// Execute
			if is_delegated {
				DelegatedNFTs::<T>::insert(nft_id, recipient_account_id.clone());
			} else {
				DelegatedNFTs::<T>::remove(nft_id);
			}
			let recipient_event = if is_delegated { Some(recipient_account_id) } else { None };
			let event = Event::NFTDelegated { nft_id, recipient: recipient_event };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Set the royalty of an NFT.
		/// Can only be called if the NFT is owned and has been created by the caller.
		#[pallet::weight(T::WeightInfo::set_royalty())]
		pub fn set_royalty(
			origin: OriginFor<T>,
			nft_id: NFTId,
			royalty: Permill,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Nfts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let nft = x.as_mut().ok_or(Error::<T>::NFTNotFound)?;

				// Checks
				ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
				ensure!(nft.creator == who, Error::<T>::NotTheNFTCreator);
				ensure!(!nft.state.is_listed, Error::<T>::CannotSetRoyaltyForListedNFTs);
				ensure!(!nft.state.is_capsule, Error::<T>::CannotSetRoyaltyForCapsuleNFTs);
				ensure!(!nft.state.is_delegated, Error::<T>::CannotSetRoyaltyForDelegatedNFTs);
				ensure!(!nft.state.is_rented, Error::<T>::CannotSetRoyaltyForRentedNFTs);

				// Execute
				nft.royalty = royalty;

				Ok(().into())
			})?;

			let event = Event::NFTRoyaltySet { nft_id, royalty };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Set the fee for minting an NFT if the caller is root.
		#[pallet::weight(T::WeightInfo::set_nft_mint_fee())]
		pub fn set_nft_mint_fee(
			origin: OriginFor<T>,
			fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			NftMintFee::<T>::put(fee);
			let event = Event::NFTMintFeeSet { fee };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Create a new collection with the provided details. An ID will be auto
		/// generated and logged as an event, the caller of this function
		/// will become the owner of the new collection.
		#[pallet::weight(T::WeightInfo::create_collection())]
		pub fn create_collection(
			origin: OriginFor<T>,
			offchain_data: U8BoundedVec<T::CollectionOffchainDataLimit>,
			limit: Option<u32>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check size limit if it exists.
			if let Some(limit) = &limit {
				ensure!(
					*limit <= T::CollectionSizeLimit::get(),
					Error::<T>::CollectionLimitExceededMaximumAllowed
				);
			}

			// Execute
			let collection_id = Self::get_next_collection_id();
			let collection = Collection::new(who.clone(), offchain_data.clone(), limit);

			// Save
			Collections::<T>::insert(collection_id, collection);
			let event =
				Event::CollectionCreated { collection_id, owner: who, offchain_data, limit };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Remove a collection from the storage. This operation is irreversible which means
		/// once the collection is removed (burned) from the storage there is no way to
		/// get it back.
		/// Must be called by the owner of the collection and collection must be empty.
		#[pallet::weight(T::WeightInfo::burn_collection())]
		pub fn burn_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let collection =
				Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;

			// Checks
			ensure!(collection.owner == who, Error::<T>::NotTheCollectionOwner);
			ensure!(collection.nfts.is_empty(), Error::<T>::CollectionIsNotEmpty);

			// Execute
			// Remove collection
			Collections::<T>::remove(collection_id);
			Self::deposit_event(Event::CollectionBurned { collection_id });

			Ok(().into())
		}

		/// Makes the collection closed. This means that it is not anymore
		/// possible to add new NFTs to the collection.
		/// Can only be called by owner of the collection.
		#[pallet::weight(T::WeightInfo::close_collection())]
		pub fn close_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Collections::<T>::try_mutate(collection_id, |x| -> DispatchResult {
				let collection = x.as_mut().ok_or(Error::<T>::CollectionNotFound)?;
				ensure!(collection.owner == who, Error::<T>::NotTheCollectionOwner);
				collection.is_closed = true;

				Ok(().into())
			})?;

			Self::deposit_event(Event::CollectionClosed { collection_id });

			Ok(().into())
		}

		/// Set the maximum amount of nfts in the collection.
		/// Caller must be owner of collection, nfts in that collection must be lower or equal to
		/// new limit.
		#[pallet::weight(T::WeightInfo::limit_collection())]
		pub fn limit_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			limit: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Collections::<T>::try_mutate(collection_id, |x| -> DispatchResult {
				let collection = x.as_mut().ok_or(Error::<T>::CollectionNotFound)?;

				// Checks
				ensure!(collection.owner == who, Error::<T>::NotTheCollectionOwner);
				ensure!(collection.limit == None, Error::<T>::CollectionLimitAlreadySet);
				ensure!(!collection.is_closed, Error::<T>::CollectionIsClosed);
				ensure!(
					collection.nfts.len() <= limit as usize,
					Error::<T>::CollectionHasTooManyNFTs
				);
				ensure!(
					limit <= T::CollectionSizeLimit::get(),
					Error::<T>::CollectionLimitExceededMaximumAllowed
				);

				// Execute
				collection.limit = Some(limit);

				Ok(().into())
			})?;

			Self::deposit_event(Event::CollectionLimited { collection_id, limit });

			Ok(().into())
		}

		/// Add an NFT to a collection.
		/// Can only be called by owner of the collection, NFT
		/// must not be in collection and collection must not be closed or has reached limit.
		#[pallet::weight((
            {
				let collection = Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound);
				if let Ok(collection) = collection {
					let s = collection.nfts.len();
					T::WeightInfo::add_nft_to_collection(s as u32)
				} else {
					T::WeightInfo::add_nft_to_collection(1)
				}
            },
			DispatchClass::Normal
        ))]
		pub fn add_nft_to_collection(
			origin: OriginFor<T>,
			nft_id: NFTId,
			collection_id: CollectionId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Collections::<T>::try_mutate(collection_id, |x| -> DispatchResult {
				let collection = x.as_mut().ok_or(Error::<T>::CollectionNotFound)?;
				let limit =
					collection.limit.unwrap_or_else(|| T::CollectionSizeLimit::get()) as usize;

				// Checks
				ensure!(collection.owner == who, Error::<T>::NotTheCollectionOwner);
				ensure!(!collection.is_closed, Error::<T>::CollectionIsClosed);
				ensure!(collection.nfts.len() < limit, Error::<T>::CollectionHasReachedLimit);

				Nfts::<T>::try_mutate(nft_id, |y| -> DispatchResult {
					let nft = y.as_mut().ok_or(Error::<T>::NFTNotFound)?;

					//Checks
					ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
					ensure!(nft.collection_id == None, Error::<T>::NFTBelongToACollection);

					//Execution
					nft.collection_id = Some(collection_id);

					Ok(().into())
				})?;

				// Execute
				collection
					.nfts
					.try_push(nft_id)
					.map_err(|_| Error::<T>::CannotAddMoreNFTsToCollection)?;

				Ok(().into())
			})?;

			Self::deposit_event(Event::NFTAddedToCollection { nft_id, collection_id });

			Ok(().into())
		}

		/// Add a secret to a basic NFT.
		/// Must be called by NFT owner.
		#[pallet::weight(T::WeightInfo::add_secret())]
		pub fn add_secret(
			origin: OriginFor<T>,
			nft_id: NFTId,
			offchain_data: U8BoundedVec<T::NFTOffchainDataLimit>,
		) -> DispatchResultWithPostInfo {
			ensure!(false, Error::<T>::ComingSoon);
			let who = ensure_signed(origin)?;

			Nfts::<T>::try_mutate(nft_id, |maybe_nft| -> DispatchResult {
				let nft = maybe_nft.as_mut().ok_or(Error::<T>::NFTNotFound)?;

				// Checks
				ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
				ensure!(!nft.state.is_listed, Error::<T>::CannotAddSecretToListedNFTs);
				ensure!(!nft.state.is_capsule, Error::<T>::CannotAddSecretToCapsuleNFTs);
				ensure!(!nft.state.is_secret, Error::<T>::CannotAddSecretToSecretNFTs);

				// The Caller needs to pay the Secret NFT Mint fee.
				let secret_nft_mint_fee = SecretNftMintFee::<T>::get();
				let reason = WithdrawReasons::FEE;
				let imbalance =
					T::Currency::withdraw(&who, secret_nft_mint_fee, reason, KeepAlive)?;
				T::FeesCollector::on_unbalanced(imbalance);

				// Execute
				nft.state.is_secret = true;
				nft.state.is_syncing = true;

				SecretNftsOffchainData::<T>::insert(nft_id, offchain_data.clone());

				Ok(().into())
			})?;

			let event = Event::SecretAddedToNFT { nft_id, offchain_data };
			Self::deposit_event(event);
			Ok(().into())
		}

		/// Create a new secret NFT with the provided details. An ID will be auto
		/// generated and logged as an event, The caller of this function
		/// will become the owner of the new NFT.
		#[pallet::weight((
            {
				if let Some(collection_id) = &collection_id {
					let collection = Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound);
					if let Ok(collection) = collection {
						let s = collection.nfts.len();
						T::WeightInfo::create_secret_nft(s as u32)
					} else {
						T::WeightInfo::create_secret_nft(1)
					}
				} else {
					T::WeightInfo::create_secret_nft(1)
				}
            },
			DispatchClass::Normal
        ))]
		pub fn create_secret_nft(
			origin: OriginFor<T>,
			offchain_data: U8BoundedVec<T::NFTOffchainDataLimit>,
			secret_offchain_data: U8BoundedVec<T::NFTOffchainDataLimit>,
			royalty: Permill,
			collection_id: Option<CollectionId>,
			is_soulbound: bool,
		) -> DispatchResultWithPostInfo {
			ensure!(false, Error::<T>::ComingSoon);
			let who = ensure_signed(origin.clone())?;

			// Check balance
			ensure!(
				Self::balance_check(&who, NftMintFee::<T>::get() + SecretNftMintFee::<T>::get()),
				Error::<T>::InsufficientBalance
			);

			// Create NFT
			Self::create_nft(origin.clone(), offchain_data, royalty, collection_id, is_soulbound)?;
			let nft_id = NextNFTId::<T>::get() - 1;

			// Add a secret to the NFT
			Self::add_secret(origin.clone(), nft_id, secret_offchain_data)?;

			Ok(().into())
		}

		/// Extrinsic called by TEE enclaves to indicate that a shard was received.
		/// Must be called by registered enclaves.
		#[pallet::weight(T::WeightInfo::add_secret_shard())]
		pub fn add_secret_shard(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			ensure!(false, Error::<T>::ComingSoon);
			let who = ensure_signed(origin)?;
			ensure!(
				/* TODO is_registered_enclave(who) */ true,
				Error::<T>::NotARegisteredEnclave
			);
			let mut has_finished_sync = false;

			Nfts::<T>::try_mutate(nft_id, |maybe_nft| -> DispatchResult {
				let nft = maybe_nft.as_mut().ok_or(Error::<T>::NFTNotFound)?;

				// Checks
				ensure!(nft.state.is_secret, Error::<T>::NFTIsNotSecret);
				ensure!(nft.state.is_syncing, Error::<T>::NFTAlreadySynced);

				SecretNftsShardsCount::<T>::try_mutate(nft_id, |maybe_shards| -> DispatchResult {
					if let Some(shards) = maybe_shards {
						ensure!(
							shards.len() < T::ShardsNumber::get() as usize,
							Error::<T>::NFTHasReceivedAllShards
						);
						ensure!(!shards.contains(&who), Error::<T>::EnclaveAlreadyAddedShard);
						shards
							.try_push(who.clone())
							.map_err(|_| Error::<T>::NFTHasReceivedAllShards)?;
						if shards.len() == T::ShardsNumber::get() as usize {
							has_finished_sync = true;
							*maybe_shards = None;
						}
					} else {
						let mut shards: BoundedVec<T::AccountId, T::ShardsNumber> =
							BoundedVec::default();
						shards
							.try_push(who.clone())
							.map_err(|_| Error::<T>::NFTHasReceivedAllShards)?;
						if shards.len() == T::ShardsNumber::get() as usize {
							has_finished_sync = true;
						} else {
							*maybe_shards = Some(shards);
						}
					}

					Ok(().into())
				})?;

				if has_finished_sync {
					nft.state.is_syncing = false;
				}

				Ok(().into())
			})?;

			let event = Event::ShardAdded { nft_id, enclave: who };
			Self::deposit_event(event);

			if has_finished_sync {
				let event = Event::SecretNFTSynced { nft_id };
				Self::deposit_event(event);
			}

			Ok(().into())
		}

		/// Set the fee for minting a secret NFT if the caller is root.
		#[pallet::weight(T::WeightInfo::set_secret_nft_mint_fee())]
		pub fn set_secret_nft_mint_fee(
			origin: OriginFor<T>,
			fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure!(false, Error::<T>::ComingSoon);
			ensure_root(origin)?;
			SecretNftMintFee::<T>::put(fee);
			let event = Event::SecretNFTMintFeeSet { fee };
			Self::deposit_event(event);

			Ok(().into())
		}
	}
}

impl<T: Config> traits::NFTExt for Pallet<T> {
	type AccountId = T::AccountId;
	type NFTOffchainDataLimit = T::NFTOffchainDataLimit;
	type CollectionOffchainDataLimit = T::CollectionOffchainDataLimit;
	type CollectionSizeLimit = T::CollectionSizeLimit;
	type ShardsNumber = T::ShardsNumber;

	fn set_nft_state(nft_id: NFTId, nft_state: NFTState) -> DispatchResult {
		Nfts::<T>::try_mutate(nft_id, |data| -> DispatchResult {
			let data = data.as_mut().ok_or(Error::<T>::NFTNotFound)?;
			data.state = nft_state;

			Ok(())
		})?;

		Ok(())
	}

	fn create_filled_collection(
		owner: Self::AccountId,
		collection_id: CollectionId,
		start_nft_id: NFTId,
		amount_in_collection: u32,
	) -> DispatchResult {
		//Create full collection
		let collection_offchain_data: U8BoundedVec<Self::CollectionOffchainDataLimit> =
			U8BoundedVec::try_from(vec![
				1;
				Self::CollectionOffchainDataLimit::get()
					.try_into()
					.unwrap()
			])
			.expect("It will never happen.");

		let mut collection = Collection::<
			Self::AccountId,
			Self::CollectionOffchainDataLimit,
			Self::CollectionSizeLimit,
		>::new(owner.clone(), collection_offchain_data, None);

		let ids: Vec<u32> = (start_nft_id..amount_in_collection + start_nft_id).collect();
		let nft_ids: BoundedVec<u32, Self::CollectionSizeLimit> =
			BoundedVec::try_from(ids).expect("It will never happen.");

		collection.nfts = nft_ids;
		Collections::<T>::insert(collection_id, collection);

		Ok(())
	}

	fn get_nft(id: NFTId) -> Option<NFTData<Self::AccountId, Self::NFTOffchainDataLimit>> {
		Nfts::<T>::get(id)
	}

	fn set_nft(
		id: NFTId,
		nft_data: NFTData<Self::AccountId, Self::NFTOffchainDataLimit>,
	) -> DispatchResult {
		Nfts::<T>::insert(id, nft_data);

		Ok(())
	}

	fn create_nft(
		owner: Self::AccountId,
		offchain_data: BoundedVec<u8, Self::NFTOffchainDataLimit>,
		royalty: Permill,
		collection_id: Option<CollectionId>,
		is_soulbound: bool,
	) -> Result<NFTId, DispatchResult> {
		let nft_state = NFTState::new(false, false, false, false, is_soulbound, false, false);
		let nft = NFTData::new(
			owner.clone(),
			owner.clone(),
			offchain_data,
			royalty,
			nft_state,
			collection_id,
		);
		let nft_id = Self::get_next_nft_id();
		Nfts::<T>::insert(nft_id, nft);

		Ok(nft_id)
	}

	fn mutate_nft<
		R,
		E,
		F: FnOnce(&mut Option<NFTData<Self::AccountId, Self::NFTOffchainDataLimit>>) -> Result<R, E>,
	>(
		id: NFTId,
		f: F,
	) -> Result<R, E> {
		Nfts::<T>::try_mutate(id, f)
	}

	fn exists(id: NFTId) -> bool {
		Nfts::<T>::contains_key(id)
	}
}

impl<T: Config> Pallet<T> {
	fn get_next_nft_id() -> NFTId {
		let nft_id = NextNFTId::<T>::get();
		let next_id = nft_id
			.checked_add(1)
			.expect("If u32 is not enough we should crash for safety; qed.");
		NextNFTId::<T>::put(next_id);

		nft_id
	}

	fn get_next_collection_id() -> NFTId {
		let collection_id = NextCollectionId::<T>::get();
		let next_id = collection_id
			.checked_add(1)
			.expect("If u32 is not enough we should crash for safety; qed.");
		NextCollectionId::<T>::put(next_id);

		collection_id
	}

	pub fn balance_check(account: &T::AccountId, amount: BalanceOf<T>) -> bool {
		let current_balance = T::Currency::free_balance(account);
		let new_balance = current_balance.checked_sub(&amount);
		if let Some(new_balance) = new_balance {
			T::Currency::ensure_can_withdraw(&account, amount, WithdrawReasons::FEE, new_balance)
				.is_ok()
		} else {
			false
		}
	}
}
