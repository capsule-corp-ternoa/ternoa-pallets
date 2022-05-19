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

pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{
		Currency, ExistenceRequirement::KeepAlive, OnUnbalanced, StorageVersion, WithdrawReasons,
	},
	transactional,
};
use frame_system::pallet_prelude::*;
use primitives::{
	nfts::{Collection, CollectionId, NFTId, NFT},
	U8BoundedVec,
};
use sp_arithmetic::per_things::Permill;
use sp_runtime::traits::StaticLookup;
use ternoa_common::{
	traits,
	helpers::check_bounds
};

pub use weights::WeightInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for pallet.
		// type WeightInfo: WeightInfo;

		/// Currency type.
		type Currency: Currency<Self::AccountId>;

		/// What we do with additional fees
		type FeesCollector: OnUnbalanced<NegativeImbalanceOf<Self>>;

		// Constants
		/// Default fee for minting NFTs.
		#[pallet::constant]
		type InitialMintFee: Get<BalanceOf<Self>>;

		/// Maximum offchain data length.
		#[pallet::constant]
		type OffchainDataLimit: Get<u32>;

		/// Maximum collection length.
		#[pallet::constant]
		type CollectionSizeLimit: Get<u32>;

		/// Maximum collection name length.
		#[pallet::constant]
		type CollectionNameLimit: Get<u32>;

		/// Maximum collection description length.
		#[pallet::constant]
		type CollectionDescriptionLimit: Get<u32>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Host much does it cost to mint a NFT (extra fee on top of the tx fees)
	#[pallet::storage]
	#[pallet::getter(fn nft_mint_fee)]
	pub(super) type NFTMintFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery, T::InitialMintFee>;

	/// The number of NFTs managed by this pallet
	#[pallet::storage]
	#[pallet::getter(fn next_nft_id)]
	pub type NextNFTId<T: Config> = StorageValue<_, NFTId, ValueQuery>;

	/// The number of Collections managed by this pallet
	#[pallet::storage]
	#[pallet::getter(fn next_collection_id)]
	pub type NextCollectionId<T: Config> = StorageValue<_, CollectionId, ValueQuery>;

	/// Data related to NFTs.
	#[pallet::storage]
	#[pallet::getter(fn nfts)]
	pub type NFTs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NFTId,
		NFT<T::AccountId, T::OffchainDataLimit>,
		OptionQuery,
	>;

	/// Data related to collections.
	#[pallet::storage]
	#[pallet::getter(fn collections)]
	pub type Collections<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		CollectionId,
		Collection<
			T::AccountId,
			T::CollectionNameLimit,
			T::CollectionDescriptionLimit,
			T::CollectionSizeLimit,
		>,
		OptionQuery,
	>;

	/// Host a map of delegated NFTs and the recipient
	#[pallet::storage]
	#[pallet::getter(fn delegated_nfts)]
	pub type DelegatedNFTs<T: Config> =
		StorageMap<_, Blake2_128Concat, NFTId, T::AccountId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new NFT was created.
		NFTCreated {
			nft_id: NFTId,
			owner: T::AccountId,
			offchain_data: U8BoundedVec<T::OffchainDataLimit>,
			collection_id: Option<CollectionId>,
			royalty: Permill,
		},
		/// An NFT was burned.
		NFTBurned { nft_id: NFTId },
		/// An NFT was transferred to someone else.
		NFTTransferred { nft_id: NFTId, sender: T::AccountId, recipient: T::AccountId },
		/// An NFT was delegated to someone else
		NFTDelegated { nft_id: NFTId, recipient: Option<T::AccountId> },
		/// Royalty has been changed for an NFT
		NFTRoyaltySet { nft_id: NFTId, royalty: Permill },
		/// NFT mint fee changed.
		NFTMintFeeSet { fee: BalanceOf<T> },
		/// A collection was created
		CollectionCreated {
			collection_id: CollectionId,
			owner: T::AccountId,
			name: U8BoundedVec<T::CollectionNameLimit>,
			description: U8BoundedVec<T::CollectionDescriptionLimit>,
			limit: Option<u32>,
		},
		/// A collection was burned
		CollectionBurned { collection_id: CollectionId },
		/// A collection was closed
		CollectionClosed { collection_id: CollectionId },
		/// A collection has limit set
		CollectionLimited { collection_id: CollectionId },
		/// An NFT has been added to a collection
		NFTAddedToCollection { nft_id: NFTId, collection_id: CollectionId },
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
		/// Operation is not allowed because the NFT is a capsule.
		CannotSetRoyaltyForCapsuleNFTs,
		/// Operation is not allowed because the NFT owner is self.
		CannotTransferNFTsToYourself,
		/// Operation is not allowed because the NFT owner is self.
		CannotDelegateNFTsToYourself,
		/// Operation is not allowed because the nft offchain data is too short
		NFTOffchainDataIsTooShort,
		/// Operation is not allowed because the nft offchain data is too long
		NFTOffchainDataIsTooLong,
		/// Operation is not allowed because the collection name is too short
		CollectionNameIsTooShort,
		/// Operation is not allowed because the collection name is too long
		CollectionNameIsTooLong,
		/// Operation is not allowed because the collection description is too short
		CollectionDescriptionIsTooShort,
		/// Operation is not allowed because the collection description is too long
		CollectionDescriptionIsTooLong,
		/// Operation is not allowed because the collection limit is too low
		CollectionLimitIsTooLow,
		/// Operation is not allowed because the collection limit is too high
		CollectionLimitIsTooHigh,
		/// No NFT was found with that NFT id.
		NFTNotFound,
		/// No NFT was found with that NFT id in the specified collection.
		NFTNotFoundInCollection,
		/// Operation is not allowed because the nft belong to a collection.
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
		/// Operation is not allowed because the collection has reached limit.
		CollectionHasReachedLimit,
		/// Operation is not allowed because the collection has reached hard limit.
		CollectionHasReachedMax,
		/// Operation is not allowed because the collection is not empty.
		CollectionIsNotEmpty,
		/// Operation is not permitted because the collection does not contains any NFTs.
		CollectionIsEmpty,
		/// Operation is not permitted because the collection's limit is already set.
		CollectionLimitAlreadySet,
		/// Operation is not permitted because the nfts number in the collection are greater than
		/// the new limit.
		CollectionNFTsNumberGreaterThanLimit,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new NFT with the provided details. An ID will be auto
		/// generated and logged as an event, The caller of this function
		/// will become the owner of the new NFT.
		// #[pallet::weight(T::WeightInfo::create_nft())]
		#[pallet::weight(100_000)]
		// have to be transactional otherwise we could make people pay the mint
		// even if the creation fails.
		#[transactional]
		pub fn create_nft(
			origin: OriginFor<T>,
			offchain_data: U8BoundedVec<T::OffchainDataLimit>,
			royalty: Permill,
			collection_id: Option<CollectionId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Checks
			// Check offchain_data length
			check_bounds(
				offchain_data.len(),
				(1, Error::<T>::NFTOffchainDataIsTooShort),
				(T::OffchainDataLimit::get(), Error::<T>::NFTOffchainDataIsTooLong),
			)?;

			// The Caller needs to pay the NFT Mint fee.
			let mint_fee = NFTMintFee::<T>::get();
			let reason = WithdrawReasons::FEE;
			let imbalance = T::Currency::withdraw(&who, mint_fee, reason, KeepAlive)?;
			T::FeesCollector::on_unbalanced(imbalance);

			let mut nft_id = None;

			// Check if the collection exists.
			// Throws an error if specified collection does not exist, signer is now owner,
			// collection is close, collection has reached limit.
			if let Some(collection_id) = &collection_id {
				Collections::<T>::try_mutate(collection_id, |x| -> DispatchResult {
					let collection = x.as_mut().ok_or(Error::<T>::CollectionNotFound)?;

					ensure!(collection.owner == who, Error::<T>::NotTheCollectionOwner);
					ensure!(!collection.is_closed, Error::<T>::CollectionIsClosed);
					ensure!(collection.nfts.len() < T::CollectionSizeLimit::get() as usize, Error::<T>::CollectionHasReachedMax);
					if let Some(limit) = &collection.limit {
						ensure!(
							collection.nfts.len() < *limit as usize,
							Error::<T>::CollectionHasReachedLimit
						);
					}
					// Execute
					let tmp_nft_id = Self::get_next_nft_id();
					collection.nfts.try_push(tmp_nft_id).expect("Cannot happen.");
					nft_id = Some(tmp_nft_id);
					Ok(().into())
				})?;
			}

			// Execute
			let nft_id = nft_id.unwrap_or_else(|| Self::get_next_nft_id());

			let nft = NFT::new_default(
				who.clone(),
				offchain_data.clone(),
				royalty,
				collection_id.clone(),
			);

			// Save
			NFTs::<T>::insert(nft_id, nft);

			let event =
				Event::NFTCreated { nft_id, owner: who, offchain_data, collection_id, royalty };

			Self::deposit_event(event);

			Ok(().into())
		}

		/// Remove an NFT from the storage. This operation is irreversible which means
		/// once the NFT is removed (burned) from the storage there is no way to
		/// get it back.
		/// Must be called by the owner of the NFT.
		// #[pallet::weight(T::WeightInfo::burn_nft())]
		#[pallet::weight(100_000)]
		pub fn burn_nft(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let nft = NFTs::<T>::get(nft_id).ok_or(Error::<T>::NFTNotFound)?;

			// Checks
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(!nft.state.listed_for_sale, Error::<T>::CannotBurnListedNFTs);
			ensure!(!nft.state.is_capsule, Error::<T>::CannotBurnCapsuleNFTs);
			ensure!(!nft.state.is_delegated, Error::<T>::CannotBurnDelegatedNFTs);

			// Check for collection to remove nft
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

			// Execute
			// Remove nft
			NFTs::<T>::remove(nft_id);

			Self::deposit_event(Event::NFTBurned { nft_id });

			Ok(().into())
		}

		/// Transfer an NFT from an account to another one. Must be called by the
		/// owner of the NFT.
		// #[pallet::weight(T::WeightInfo::transfer_nft())]
		#[pallet::weight(100_000)]
		pub fn transfer_nft(
			origin: OriginFor<T>,
			nft_id: NFTId,
			recipient: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let recipient = T::Lookup::lookup(recipient)?;

			NFTs::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let nft = x.as_mut().ok_or(Error::<T>::NFTNotFound)?;

				// Checks
				ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
				ensure!(nft.owner != recipient, Error::<T>::CannotTransferNFTsToYourself);
				ensure!(!nft.state.listed_for_sale, Error::<T>::CannotTransferListedNFTs);
				ensure!(!nft.state.is_capsule, Error::<T>::CannotTransferCapsuleNFTs);
				ensure!(!nft.state.is_delegated, Error::<T>::CannotTransferDelegatedNFTs);

				// Execute
				nft.owner = recipient.clone();
				Ok(().into())
			})?;

			let event = Event::NFTTransferred { nft_id, sender: who, recipient };

			Self::deposit_event(event);

			Ok(().into())
		}

		/// Delegate an NFT to a recipient, does not change ownership
		/// Must be called by NFT owner
		// #[pallet::weight(T::WeightInfo::delegate_nft())]
		#[pallet::weight(100_000)]
		pub fn delegate_nft(
			origin: OriginFor<T>,
			nft_id: NFTId,
			recipient: Option<<T::Lookup as StaticLookup>::Source>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut recipient_account_id: Option<T::AccountId> = None;

			NFTs::<T>::try_mutate(nft_id, |maybe_nft| -> DispatchResult {
				let nft = maybe_nft.as_mut().ok_or(Error::<T>::NFTNotFound)?;

				// Checks
				ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
				ensure!(!nft.state.listed_for_sale, Error::<T>::CannotDelegateListedNFTs);
				ensure!(!nft.state.is_capsule, Error::<T>::CannotDelegateCapsuleNFTs);

				if let Some(recipient) = &recipient {
					let recipient = T::Lookup::lookup(recipient.clone())?;
					ensure!(who != recipient, Error::<T>::CannotDelegateNFTsToYourself);
					recipient_account_id = Some(recipient);
				}

				// Execute
				nft.state.is_delegated = recipient_account_id.is_some();

				Ok(().into())
			})?;

			// Execute
			match recipient_account_id.as_ref() {
				Some(v) => DelegatedNFTs::<T>::insert(nft_id, v),
				None => DelegatedNFTs::<T>::remove(nft_id),
			}

			let event = Event::NFTDelegated { nft_id, recipient: recipient_account_id };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Set the royalty of an NFT
		/// Can only be called from creator if creator is the owner.
		// #[pallet::weight(T::WeightInfo::set_royalty())]
		#[pallet::weight(100_000)]
		pub fn set_royalty(
			origin: OriginFor<T>,
			nft_id: NFTId,
			royalty: Permill,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			NFTs::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let nft = x.as_mut().ok_or(Error::<T>::NFTNotFound)?;

				// Checks
				ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
				ensure!(nft.creator == who, Error::<T>::NotTheNFTCreator);
				ensure!(!nft.state.listed_for_sale, Error::<T>::CannotSetRoyaltyForListedNFTs);
				ensure!(!nft.state.is_capsule, Error::<T>::CannotSetRoyaltyForCapsuleNFTs);
				ensure!(!nft.state.is_delegated, Error::<T>::CannotSetRoyaltyForDelegatedNFTs);

				// Execute
				nft.royalty = royalty;

				Ok(().into())
			})?;

			let event = Event::NFTRoyaltySet { nft_id, royalty };

			Self::deposit_event(event);

			Ok(().into())
		}

		/// Set the fee for minting an NFT
		/// Can only be called from root access.
		// #[pallet::weight(T::WeightInfo::set_nft_mint_fee())]
		#[pallet::weight(100_000)]
		pub fn set_nft_mint_fee(
			origin: OriginFor<T>,
			fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			NFTMintFee::<T>::put(fee);

			let event = Event::NFTMintFeeSet { fee };

			Self::deposit_event(event);

			Ok(().into())
		}

		/// Create a new collection with the provided details. An ID will be auto
		/// generated and logged as an event, The caller of this function
		/// will become the owner of the new collection.
		// #[pallet::weight(T::WeightInfo::create_collection())]
		#[pallet::weight(100_000)]
		pub fn create_collection(
			origin: OriginFor<T>,
			name: U8BoundedVec<T::CollectionNameLimit>,
			description: U8BoundedVec<T::CollectionDescriptionLimit>,
			limit: Option<u32>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Checks
			// Check name length
			check_bounds(
				name.len(),
				(1, Error::<T>::CollectionNameIsTooShort),
				(T::CollectionNameLimit::get(), Error::<T>::CollectionNameIsTooLong),
			)?;
			// Check description length
			check_bounds(
				description.len(),
				(1, Error::<T>::CollectionDescriptionIsTooShort),
				(T::CollectionDescriptionLimit::get(), Error::<T>::CollectionDescriptionIsTooLong),
			)?;
			// Check size limit if it exists
			if let Some(limit) = &limit {
				check_bounds(
					*limit as usize,
					(1, Error::<T>::CollectionLimitIsTooLow),
					(T::CollectionSizeLimit::get(), Error::<T>::CollectionLimitIsTooHigh),
				)?;
			}

			// Execute
			let collection_id = Self::get_next_collection_id();

			let collection = Collection::new(who.clone(), name.clone(), description.clone(), limit);

			// Save
			Collections::<T>::insert(collection_id, collection);

			let event =
				Event::CollectionCreated { collection_id, owner: who, name, description, limit };

			Self::deposit_event(event);

			Ok(().into())
		}

		/// Remove a collection from the storage. This operation is irreversible which means
		/// once the collection is removed (burned) from the storage there is no way to
		/// get it back.
		/// Must be called by the owner of the collection and collection must be empty.
		// #[pallet::weight(T::WeightInfo::burn_collection())]
		#[pallet::weight(100_000)]
		pub fn burn_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let collection =
				Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;

			// Checks
			ensure!(collection.owner == who, Error::<T>::NotTheCollectionOwner);
			ensure!(collection.nfts.len() == 0, Error::<T>::CollectionIsNotEmpty);

			// Execute
			// Remove collection
			Collections::<T>::remove(collection_id);

			Self::deposit_event(Event::CollectionBurned { collection_id });

			Ok(().into())
		}

		/// Makes the series closed. This means that it is not anymore
		/// possible to add new NFTs to the series.
		/// Can only be called by owner of the collection if collection is not empty
		// #[pallet::weight(T::WeightInfo::close_collection())]
		#[pallet::weight(100_000)]
		pub fn close_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Collections::<T>::try_mutate(collection_id, |x| -> DispatchResult {
				let collection = x.as_mut().ok_or(Error::<T>::CollectionNotFound)?;

				// Checks
				ensure!(collection.owner == who, Error::<T>::NotTheCollectionOwner);
				ensure!(collection.nfts.len() != 0, Error::<T>::CollectionIsEmpty);

				// Execute
				collection.is_closed = true;

				Ok(().into())
			})?;

			Self::deposit_event(Event::CollectionClosed { collection_id });

			Ok(().into())
		}

		/// Set the maximum number (limit) of nfts in the collection
		/// Can only be called by owner of the collection, if number of nfts is not greater than new
		/// limit
		// #[pallet::weight(T::WeightInfo::limit_collection())]
		#[pallet::weight(100_000)]
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
					collection.nfts.len() > limit as usize,
					Error::<T>::CollectionNFTsNumberGreaterThanLimit
				);
				check_bounds(
					limit as usize,
					(1, Error::<T>::CollectionLimitIsTooLow),
					(T::CollectionSizeLimit::get(), Error::<T>::CollectionLimitIsTooHigh),
				)?;

				// Execute
				collection.limit = Some(limit);

				Ok(().into())
			})?;

			Self::deposit_event(Event::CollectionLimited { collection_id });

			Ok(().into())
		}

		/// Add an NFT to a collection
		/// Can only be called by owner of the collection and NFT
		/// NFT must not be in collection
		/// Collection must not be closed or has reached limit
		// #[pallet::weight(T::WeightInfo::add_nft_to_collection())]
		#[pallet::weight(100_000)]
		pub fn add_nft_to_collection(
			origin: OriginFor<T>,
			nft_id: NFTId,
			collection_id: CollectionId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Collections::<T>::try_mutate(collection_id, |x| -> DispatchResult {
				let collection = x.as_mut().ok_or(Error::<T>::CollectionNotFound)?;

				// Checks
				ensure!(collection.owner == who, Error::<T>::NotTheCollectionOwner);
				ensure!(!collection.is_closed, Error::<T>::CollectionIsClosed);
				ensure!(collection.nfts.len() < T::CollectionSizeLimit::get() as usize, Error::<T>::CollectionHasReachedMax);
				if let Some(limit) = &collection.limit {
					ensure!(
						collection.nfts.len() < *limit as usize,
						Error::<T>::CollectionHasReachedLimit
					);
				}

				NFTs::<T>::try_mutate(nft_id, |y| -> DispatchResult {
					let nft = y.as_mut().ok_or(Error::<T>::NFTNotFound)?;

					//Checks
					ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
					ensure!(nft.collection_id == None, Error::<T>::NFTBelongToACollection);

					//Execution
					nft.collection_id = Some(collection_id);

					Ok(().into())
				})?;

				// Execute
				collection.nfts.try_push(nft_id).expect("Cannot happen.");

				Ok(().into())
			})?;

			Self::deposit_event(Event::NFTAddedToCollection { nft_id, collection_id });

			Ok(().into())
		}
	}
}

impl<T: Config> traits::NFTExt for Pallet<T> {
	type AccountId = T::AccountId;
	// type OffchainDataLimit: T::OffchainDataLimit;
	// type CollectionSizeLimit: T::CollectionSizeLimit;
	// type CollectionNameLimit: T::CollectionNameLimit;
	// type CollectionDescriptionLimit: T::CollectionDescriptionLimit;
	// type InitialMintFee: T::InitialMintFee;

	// fn set_owner(id: NFTId, owner: &Self::AccountId) -> DispatchResult {
	// 	Data::<T>::try_mutate(id, |data| -> DispatchResult {
	// 		let data = data.as_mut().ok_or(Error::<T>::NFTNotFound)?;
	// 		data.owner = owner.clone();
	// 		Ok(())
	// 	})?;

	// 	Ok(())
	// }

	// fn owner(id: NFTId) -> Option<Self::AccountId> {
	// 	Some(Data::<T>::get(id)?.owner)
	// }

	// fn is_nft_in_completed_series(id: NFTId) -> Option<bool> {
	// 	let series_id = Data::<T>::get(id)?.series_id;
	// 	Some(!Series::<T>::get(series_id)?.draft)
	// }

	// fn create_nft(
	// 	owner: Self::AccountId,
	// 	ipfs_reference: IPFSReference<T>,
	// 	series_id: Option<NFTSeriesId>,
	// ) -> Result<NFTId, DispatchErrorWithPostInfo> {
	// 	Self::create(Origin::<T>::Signed(owner).into(), ipfs_reference, series_id)?;
	// 	return Ok(Self::nft_id_generator() - 1)
	// }

	// fn get_nft(id: NFTId) -> Option<NFTData<Self::AccountId, Self::IPFSLengthLimit>> {
	// 	Data::<T>::get(id)
	// }

	// fn benchmark_lock_series(series_id: NFTSeriesId) {
	// 	Series::<T>::mutate(&series_id, |x| {
	// 		x.as_mut().unwrap().draft = false;
	// 	});
	// }

	// fn set_listed_for_sale(id: NFTId, value: bool) -> DispatchResult {
	// 	Data::<T>::try_mutate(id, |data| -> DispatchResult {
	// 		let data = data.as_mut().ok_or(Error::<T>::NFTNotFound)?;
	// 		data.listed_for_sale = value;
	// 		Ok(())
	// 	})?;

	// 	Ok(())
	// }

	// fn is_listed_for_sale(id: NFTId) -> Option<bool> {
	// 	let nft = Data::<T>::get(id);
	// 	if let Some(nft) = nft {
	// 		return Some(nft.listed_for_sale)
	// 	}

	// 	return None
	// }

	// fn set_series_completion(series_id: &NFTSeriesId, value: bool) -> DispatchResult {
	// 	Series::<T>::try_mutate(series_id, |x| -> DispatchResult {
	// 		let series = x.as_mut().ok_or(Error::<T>::SeriesNotFound)?;
	// 		series.draft = !value;
	// 		Ok(())
	// 	})?;

	// 	Ok(())
	// }

	// fn set_viewer(id: NFTId, value: Option<Self::AccountId>) -> DispatchResult {
	// 	Data::<T>::try_mutate(id, |maybe_data| -> DispatchResult {
	// 		let data = maybe_data.as_mut().ok_or(Error::<T>::NFTNotFound)?;
	// 		data.is_delegated = value.is_some();
	// 		Ok(().into())
	// 	})?;

	// 	match value {
	// 		Some(v) => DelegatedNFTs::<T>::insert(id, v),
	// 		None => DelegatedNFTs::<T>::remove(id),
	// 	}

	// 	Ok(())
	// }
}

impl<T: Config> Pallet<T> {
	fn get_next_nft_id() -> NFTId {
		let nft_id = NextNFTId::<T>::get();
		let next_id = nft_id
			.checked_add(1)
			.expect("If u32 is not enough we should crash for safety; qed.");
		NextNFTId::<T>::put(next_id);

		return nft_id
	}

	fn get_next_collection_id() -> NFTId {
		let collection_id = NextCollectionId::<T>::get();
		let next_id = collection_id
			.checked_add(1)
			.expect("If u32 is not enough we should crash for safety; qed.");
		NextCollectionId::<T>::put(next_id);

		return collection_id
	}
}
