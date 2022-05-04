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
	dispatch::DispatchResult,
	traits::{
		Currency, ExistenceRequirement,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		Get, StorageVersion, WithdrawReasons,
	},
	BoundedVec, PalletId,
};
use primitives::{
	nfts::{NFTId, NFTSeriesId},
	TextFormat,
};
use sp_runtime::traits::AccountIdConversion;
use sp_std::vec;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{ensure, pallet_prelude::*, transactional};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::CheckedAdd;
	use sp_std::convert::TryInto;
	use ternoa_common::traits::NFTExt;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type IPFSLengthLimitOf<T> = <<T as Config>::NFTExt as NFTExt>::IPFSLengthLimit;
	pub type CapsuleIPFSReference<T> = primitives::nfts::IPFSReference<IPFSLengthLimitOf<T>>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for pallet.
		type WeightInfo: WeightInfo;

		/// Currency type.
		type Currency: Currency<Self::AccountId>;

		/// Link to the NFT pallet.
		type NFTExt: NFTExt<AccountId = Self::AccountId>;

		// Constants
		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The Maximum amount of capsules that can be active at the same time for a user has been
		/// reached.
		#[pallet::constant]
		type CapsuleCountLimit: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			if !CapsuleMintFee::<T>::exists() {
				let fee: BalanceOf<T> = 1000000000000000000000u128.try_into().ok().unwrap();
				CapsuleMintFee::<T>::put(fee);

				return 1
			}

			0
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates an NFT and coverts it into a capsule.
		#[pallet::weight(T::WeightInfo::create())]
		#[transactional]
		pub fn create(
			origin: OriginFor<T>,
			nft_ipfs_reference: BoundedVec<u8, IPFSLengthLimitOf<T>>,
			capsule_ipfs_reference: CapsuleIPFSReference<T>,
			series_id: Option<NFTSeriesId>,
			royaltie_fee: u8,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if the user has reached the capsule count limit.
			ensure!(!Self::has_reached_limit(&who), Error::<T>::MaximumCapsuleCountReached);

			// Reserve funds
			let amount = CapsuleMintFee::<T>::get();
			Self::send_funds(&who, &Self::account_id(), amount, KeepAlive)?;

			// Create NFT and capsule
			let nft_id =
				T::NFTExt::create_nft(who.clone(), nft_ipfs_reference, series_id, royaltie_fee)?;

			Self::new_capsule(&who, nft_id, capsule_ipfs_reference.clone(), amount)?;
			T::NFTExt::set_converted_to_capsule(nft_id, true)?;

			Self::deposit_event(Event::CapsuleDeposit { balance: amount });
			let event = Event::CapsuleCreated { owner: who, nft_id, frozen_balance: amount };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Converts an existing NFT into a capsule.
		#[pallet::weight(T::WeightInfo::create_from_nft())]
		#[transactional]
		pub fn create_from_nft(
			origin: OriginFor<T>,
			nft_id: NFTId,
			ipfs_reference: CapsuleIPFSReference<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if the user has reached the capsule count limit.
			ensure!(!Self::has_reached_limit(&who), Error::<T>::MaximumCapsuleCountReached);

			let nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(!nft.listed_for_sale, Error::<T>::CannotCreateCapsulesFromNFTsListedForSale);
			ensure!(
				!nft.is_in_transmission,
				Error::<T>::CannotCreateCapsulesFromNFTsInTransmission
			);
			ensure!(!nft.is_capsule, Error::<T>::CannotCreateCapsulesFromCapsules);
			ensure!(!nft.is_delegated, Error::<T>::CannotCreateCapsulesFromDelegatedNFTs);

			// Reserve funds
			let amount = CapsuleMintFee::<T>::get();
			Self::send_funds(&who, &Self::account_id(), amount, KeepAlive)?;

			// Create capsule
			Self::new_capsule(&who, nft_id, ipfs_reference.clone(), amount)?;
			T::NFTExt::set_converted_to_capsule(nft_id, true)?;

			Self::deposit_event(Event::CapsuleDeposit { balance: amount });
			let event = Event::CapsuleCreated { owner: who, nft_id, frozen_balance: amount };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Converts a capsule into an NFT.
		#[pallet::weight(T::WeightInfo::remove())]
		#[transactional]
		pub fn remove(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut unused_funds = Default::default();

			Ledgers::<T>::try_mutate(&who, |x| -> DispatchResult {
				let data = x.as_mut().ok_or(Error::<T>::NotTheNFTOwner)?;

				let error = Error::<T>::NotTheNFTOwner;
				let index = data.iter().position(|x| x.0 == nft_id).ok_or(error)?;

				unused_funds = data[index].1;
				Self::send_funds(&Self::account_id(), &who, data[index].1, AllowDeath)?;

				data.swap_remove(index);
				if data.is_empty() {
					*x = None;
				}

				Capsules::<T>::take(nft_id).ok_or(Error::<T>::InternalError)?;

				Ok(())
			})?;

			let event = Event::CapsuleRemoved { nft_id, unfrozen_balance: unused_funds };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Adds additional funds to a capsule.
		#[pallet::weight(T::WeightInfo::add_funds())]
		#[transactional]
		pub fn add_funds(
			origin: OriginFor<T>,
			nft_id: NFTId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Ledgers::<T>::try_mutate(&who, |x| -> DispatchResult {
				let data = x.as_mut().ok_or(Error::<T>::NotTheNFTOwner)?;
				let error = Error::<T>::NotTheNFTOwner;
				let index = data.iter().position(|x| x.0 == nft_id).ok_or(error)?;

				Self::send_funds(&who, &Self::account_id(), amount, KeepAlive)?;

				let error = Error::<T>::ArithmeticError;
				data[index].1 = data[index].1.checked_add(&amount).ok_or(error)?;

				Ok(())
			})?;

			let event = Event::CapsuleFundsAdded { nft_id, balance: amount };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Changes the capsule ipfs reference.
		#[pallet::weight(T::WeightInfo::set_ipfs_reference())]
		pub fn set_ipfs_reference(
			origin: OriginFor<T>,
			nft_id: NFTId,
			ipfs_reference: CapsuleIPFSReference<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Capsules::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let data = x.as_mut().ok_or(Error::<T>::NFTNotFound)?;
				ensure!(data.owner == who, Error::<T>::NotTheNFTOwner);

				data.ipfs_reference = ipfs_reference.clone();
				Ok(())
			})?;

			let event = Event::CapsuleIpfsReferenceUpdated { nft_id, ipfs_reference };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Sets the Capsule Mint Fee.
		#[pallet::weight(T::WeightInfo::set_capsule_mint_fee())]
		pub fn set_capsule_mint_fee(
			origin: OriginFor<T>,
			capsule_fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			CapsuleMintFee::<T>::put(capsule_fee);

			Self::deposit_event(Event::CapsuleMintFeeUpdated { fee: capsule_fee });

			Ok(().into())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A capsule ipfs reference was updated.
		CapsuleIpfsReferenceUpdated { nft_id: NFTId, ipfs_reference: CapsuleIPFSReference<T> },
		/// Additional funds were added to a capsule.
		CapsuleFundsAdded { nft_id: NFTId, balance: BalanceOf<T> },
		/// A capsule was convert into an NFT.
		CapsuleRemoved { nft_id: NFTId, unfrozen_balance: BalanceOf<T> },
		/// A capsule was created.
		CapsuleCreated { owner: T::AccountId, nft_id: NFTId, frozen_balance: BalanceOf<T> },
		/// Capsule mint fee has been updated.
		CapsuleMintFeeUpdated { fee: BalanceOf<T> },
		/// Some funds have been deposited.
		CapsuleDeposit { balance: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Cannot create a capsule from a delegated NFT.
		CannotCreateCapsulesFromDelegatedNFTs,
		/// Cannot create a capsule from NFTs listed for sale.
		CannotCreateCapsulesFromNFTsListedForSale,
		/// Cannot create a capsule from NFTs listed for sale.
		CannotCreateCapsulesFromNFTsInTransmission,
		/// Cannot create a capsule from NFTs that are already capsules.
		CannotCreateCapsulesFromCapsules,
		/// This should never happen.
		ArithmeticError,
		/// Ipfs reference is too short.
		IPFSReferenceIsTooShort,
		/// Ipfs reference is too long.
		IPFSReferenceIsTooLong,
		/// This should never happen.
		InternalError,
		/// No NFT was found with that NFT id.
		NFTNotFound,
		/// Callers is not the NFT owner.
		NotTheNFTOwner,
		/// The Maximum amount of capsule that can be active at the same time for a user has been
		MaximumCapsuleCountReached,
	}

	/// Current capsule mint fee.
	#[pallet::storage]
	#[pallet::getter(fn capsule_mint_fee)]
	pub type CapsuleMintFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// List of NFTs that are capsulized.
	#[pallet::storage]
	#[pallet::getter(fn capsules)]
	pub type Capsules<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NFTId,
		CapsuleData<T::AccountId, IPFSLengthLimitOf<T>>,
		OptionQuery,
	>;

	/// List of accounts that hold capsulized NFTs.
	#[pallet::storage]
	#[pallet::getter(fn ledgers)]
	pub type Ledgers<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		CapsuleLedger<BalanceOf<T>, T::CapsuleCountLimit>,
		OptionQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub capsule_mint_fee: BalanceOf<T>,
		pub capsules: Vec<(NFTId, T::AccountId, TextFormat)>,
		pub ledgers: Vec<(T::AccountId, Vec<(NFTId, BalanceOf<T>)>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				capsule_mint_fee: Default::default(),
				capsules: Default::default(),
				ledgers: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.capsules.clone().into_iter().for_each(|(nft_id, account, reference)| {
				let ipfs_reference =
					BoundedVec::try_from(reference).expect("It will never happen.");
				Capsules::<T>::insert(nft_id, CapsuleData::new(account, ipfs_reference));
			});

			self.ledgers.clone().into_iter().for_each(|(account, data)| {
				let data = BoundedVec::try_from(data).expect("It will never happen.");
				Ledgers::<T>::insert(account, data);
			});

			CapsuleMintFee::<T>::put(self.capsule_mint_fee);
		}
	}
}

impl<T: Config> Pallet<T> {
	fn new_capsule(
		owner: &T::AccountId,
		nft_id: NFTId,
		ipfs_reference: CapsuleIPFSReference<T>,
		funds: BalanceOf<T>,
	) -> Result<(), Error<T>> {
		let data = CapsuleData::new(owner.clone(), ipfs_reference.clone());
		Capsules::<T>::insert(nft_id, data);

		Ledgers::<T>::mutate(&owner, |x| -> Result<(), Error<T>> {
			if let Some(data) = x {
				data.try_push((nft_id, funds))
					.map_err(|_| Error::<T>::MaximumCapsuleCountReached)?
			} else {
				*x = Some(
					BoundedVec::try_from(vec![(nft_id, funds)])
						.map_err(|_| Error::<T>::MaximumCapsuleCountReached)?,
				);
			}
			Ok(())
		})
	}

	fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	fn send_funds(
		sender: &T::AccountId,
		receiver: &T::AccountId,
		amount: BalanceOf<T>,
		liveness: ExistenceRequirement,
	) -> DispatchResult {
		let imbalance = T::Currency::withdraw(&sender, amount, WithdrawReasons::FEE, liveness)?;
		T::Currency::resolve_creating(receiver, imbalance);

		Ok(())
	}

	fn has_reached_limit(owner: &T::AccountId) -> bool {
		let ledger = Ledgers::<T>::try_get(owner);
		if let Ok(ledger) = ledger {
			return ledger.is_full()
		}

		false
	}
}
