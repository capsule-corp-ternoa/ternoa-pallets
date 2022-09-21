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

pub use pallet::*;
pub use types::*;

use core::convert::TryFrom;
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	pallet_prelude::DispatchResultWithPostInfo,
	traits::{
		Currency,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		Get, StorageVersion, WithdrawReasons,
	},
	BoundedVec, PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits::{AccountIdConversion, CheckedSub, Saturating},
	Permill,
};
use sp_std::prelude::*;

use primitives::nfts::{
	NFTId,
	NFTStateModifiers::{self, *},
};
use ternoa_common::traits::NFTExt;
pub use weights::WeightInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

pub type RentContractDataOf<T> = RentContractData<
	<T as frame_system::Config>::AccountId,
	<T as frame_system::Config>::BlockNumber,
	BalanceOf<T>,
	<T as Config>::AccountSizeLimit,
>;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

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
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for pallet.
		type WeightInfo: WeightInfo;

		/// Currency type.
		type Currency: Currency<Self::AccountId>;

		/// Link to the NFT pallet.
		type NFTExt: NFTExt<AccountId = Self::AccountId>;

		// Constants
		/// The auctions pallet id - will be used to generate account id.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The maximum number of accounts that can be stored inside the account list.
		#[pallet::constant]
		type AccountSizeLimit: Get<u32>;

		/// Maximum number of simultaneous rent contract.
		#[pallet::constant]
		type SimultaneousContractLimit: Get<u32>;

		/// Maximum number of related automatic rent actions in block.
		#[pallet::constant]
		type ActionsInBlockLimit: Get<u32>;

		/// Maximum number of blocks during which a rent contract is available for acceptance.
		#[pallet::constant]
		type ContractExpirationDuration: Get<u32>;
	}

	/// Data related to rent contracts.
	#[pallet::storage]
	#[pallet::getter(fn contracts)]
	pub type Contracts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NFTId,
		RentContractData<T::AccountId, T::BlockNumber, BalanceOf<T>, T::AccountSizeLimit>,
		OptionQuery,
	>;

	/// Data related to contracts queues.
	#[pallet::storage]
	#[pallet::getter(fn queues)]
	pub type Queues<T: Config> =
		StorageValue<_, RentingQueues<T::BlockNumber, T::SimultaneousContractLimit>, ValueQuery>;

	/// Data related to rent contracts offers.
	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NFTId,
		AccountList<T::AccountId, T::AccountSizeLimit>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Contract Created.
		ContractCreated {
			nft_id: NFTId,
			renter: T::AccountId,
			duration: Duration<T::BlockNumber>,
			acceptance_type: AcceptanceType<AccountList<T::AccountId, T::AccountSizeLimit>>,
			renter_can_cancel: bool,
			rent_fee: RentFee<BalanceOf<T>>,
			renter_cancellation_fee: CancellationFee<BalanceOf<T>>,
			rentee_cancellation_fee: CancellationFee<BalanceOf<T>>,
		},
		/// Contract was accepted and has started.
		ContractStarted { nft_id: NFTId, rentee: T::AccountId },
		/// Contract was revoked by either renter or rentee.
		ContractRevoked { nft_id: NFTId, revoked_by: T::AccountId },
		/// An offer was made for manual acceptance rent contract.
		ContractOfferCreated { nft_id: NFTId, rentee: T::AccountId },
		/// An offer was retracted for manual acceptance rent contract.
		ContractOfferRetracted { nft_id: NFTId, rentee: T::AccountId },
		/// A contract subscription's terms were changed by renter.
		ContractSubscriptionTermsChanged {
			nft_id: NFTId,
			period: T::BlockNumber,
			max_duration: Option<T::BlockNumber>,
			rent_fee: BalanceOf<T>,
		},
		/// A contract new subscription's terms were accpeted by rentee.
		ContractSubscriptionTermsAccepted { nft_id: NFTId },
		/// A contract has ended.
		ContractEnded { nft_id: NFTId, revoked_by: Option<T::AccountId> },
		/// A contract's subscription period has started.
		ContractSubscriptionPeriodStarted { nft_id: NFTId },
		/// A contract available for sale was expired before its acceptance.
		ContractExpired { nft_id: NFTId },
		/// Contract was canceled.
		ContractCanceled { nft_id: NFTId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// NFT not found.
		NFTNotFound,
		/// TODO
		RentNFTNotFound,
		/// TODO
		CancellationNFTNotFound,
		/// TODO
		NotAContractParticipant,
		/// TODO
		ContractIsNotRunning,
		/// TODO
		ContractIsRunning,
		/// TODO
		ContractCannotBeCanceledByRenter,
		/// TODO
		ContractDoesNotSupportAutomaticRent,
		/// TODO
		ContractDoesNotSupportOffers,
		/// TODO
		NotWhitelisted,
		/// TODO
		NotTheRentedNFTOwner,
		/// TODO
		RenteeDoesNotOwnTheRentNFT,
		/// TODO
		RenteeDoesNotOwnTheCancellationNFT,
		/// TODO
		RentedNFTNotInValidState,
		/// TODO
		NotTheCancellationNFTOwner,
		/// TODO
		CancellationNFTNotInValidState,
		/// TODO
		NotEnoughFundsForRentFee,
		/// TODO
		NotEnoughFundsForCancellationFee,
		/// TODO
		NotTheContractOwner,
		/// TODO
		NotTheContractRentee,
		/// TODO
		ContractNFTNotInAValidState,
		/// TODO
		/// Not the owner of the NFT.
		NotTheNFTOwner,
		/// Operation is not permitted because NFT is in invalid state.
		NFTInInvalidState,
		/// Operation is not permitted because the maximum number au parallel rent contract has
		/// been reached.
		MaxSimultaneousContractReached,
		/// The contract was not found for the given nft_id.
		ContractNotFound,
		/// The caller is neither the renter or rentee.
		NotTheRenterOrRentee,
		/// Cannot Rent your own contract.
		CannotRentOwnContract,
		/// Maximum offers reached.
		MaximumOffersReached,
		/// Operation is not permitted because contract terms are already accepted
		ContractTermsAlreadyAccepted,
		/// No offers was found for the contract
		NoOffersForThisContract,
		/// TODO
		NoOfferFoundForThatRenteeAddress,
		/// Offer not found.
		OfferNotFound,
		/// Duration and Revocation Mismatch
		DurationAndRentFeeMismatch,
		/// Duration and Cancellation Mismatch
		DurationAndCancellationFeeMismatch,
		/// Cannot adjust subscription Terms.
		CannotAdjustSubscriptionTerms,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Weight: see `begin_block`
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut read = 1u64;
			let mut write = 0u64;
			let mut current_actions = 0;
			let max_actions = T::ActionsInBlockLimit::get();

			let mut queues = Queues::<T>::get();
			// Fixed queue management
			while let Some(nft_id) = queues.fixed_queue.pop_next(now) {
				_ = Self::handle_finished_or_unused_contract(nft_id);
				// Deposit event.
				let event = Event::ContractEnded { nft_id, revoked_by: None };
				Self::deposit_event(event);

				read += 3;
				write += 4;
				current_actions += 1;
				if current_actions >= max_actions {
					break
				}
			}

			// Subscription queue management
			while let Some(nft_id) = queues.subscription_queue.pop_next(now) {
				if let Some(block_number) = Self::handle_subscription_contract(nft_id, &now) {
					queues
						.subscription_queue
						.insert(nft_id, block_number)
						.expect("This cannot happen. qed");
					let event = Event::ContractSubscriptionPeriodStarted { nft_id };
					Self::deposit_event(event);
				} else {
					_ = Self::handle_finished_or_unused_contract(nft_id);
					let event = Event::ContractEnded { nft_id, revoked_by: None };
					Self::deposit_event(event);
				};

				read += 3;
				write += 2;
				current_actions += 1;
				if current_actions >= max_actions {
					break
				}
			}

			// Available queue management
			while let Some(nft_id) = queues.available_queue.pop_next(now) {
				_ = Self::handle_finished_or_unused_contract(nft_id);

				let event = Event::ContractExpired { nft_id };
				Self::deposit_event(event);

				read += 3;
				write += 5;
				current_actions += 1;
				if current_actions >= max_actions {
					break
				}
			}

			if current_actions > 0 {
				Queues::<T>::set(queues);
				write += 1;
			}

			T::DbWeight::get().reads_writes(read, write)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new rent contract with the provided details.
		#[pallet::weight(T::WeightInfo::create_contract(Queues::<T>::get().size() as u32))]
		pub fn create_contract(
			origin: OriginFor<T>,
			nft_id: NFTId,
			duration: Duration<T::BlockNumber>,
			acceptance_type: AcceptanceType<AccountList<T::AccountId, T::AccountSizeLimit>>,
			renter_can_cancel: bool,
			rent_fee: RentFee<BalanceOf<T>>,
			renter_cancellation_fee: CancellationFee<BalanceOf<T>>,
			rentee_cancellation_fee: CancellationFee<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let pallet = Self::account_id();
			let mut queues = Queues::<T>::get();

			// Queue ✅
			queues.can_be_increased(1).ok_or(Error::<T>::MaxSimultaneousContractReached)?;

			// Contract NFT Check ✅
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(
				nft.not_in_state(&Self::invalid_state()).is_ok(),
				Error::<T>::ContractNFTNotInAValidState
			);

			// Duration Check ✅
			duration
				.allows_rent_fee(&rent_fee)
				.ok_or(Error::<T>::DurationAndRentFeeMismatch)?;
			duration
				.allows_cancellation(&renter_cancellation_fee)
				.ok_or(Error::<T>::DurationAndCancellationFeeMismatch)?;
			duration
				.allows_cancellation(&rentee_cancellation_fee)
				.ok_or(Error::<T>::DurationAndCancellationFeeMismatch)?;

			// Rent Fee Check ✅
			if let Some(id) = rent_fee.get_nft() {
				ensure!(T::NFTExt::exists(id), Error::<T>::RentNFTNotFound);
			}

			// Rentee Cancellation Check  ✅
			if let Some(id) = rentee_cancellation_fee.get_nft() {
				ensure!(T::NFTExt::exists(id), Error::<T>::CancellationNFTNotFound);
			}

			// Checking done, time to change the storage 📦
			// Renter Cancellation fee ✅ taken  📦
			// Renter Cancellation NFT Check and Taken  📦
			if let Some(amount) = renter_cancellation_fee.get_balance() {
				T::Currency::transfer(&who, &pallet, amount, KeepAlive)?;
			}

			if let Some(id) = renter_cancellation_fee.get_nft() {
				T::NFTExt::mutate_nft(id, |x| -> DispatchResult {
					let nft = x.as_mut().ok_or(Error::<T>::CancellationNFTNotFound)?;
					ensure!(nft.owner == who, Error::<T>::NotTheCancellationNFTOwner);
					ensure!(
						nft.not_in_state(&Self::invalid_state()).is_ok(),
						Error::<T>::CancellationNFTNotInValidState
					);
					nft.owner = pallet;
					Ok(())
				})?;
			}

			// Queue Updated  📦
			let now = frame_system::Pallet::<T>::block_number();
			let expiration_block = now + T::ContractExpirationDuration::get().into();
			queues
				.insert(nft_id, expiration_block, QueueKind::Available)
				.expect("Checked on line 317. qed");
			Queues::<T>::set(queues);

			// Contract Created 📦
			let contract = RentContractData::new(
				None,
				who.clone(),
				None,
				duration.clone(),
				acceptance_type.clone(),
				renter_can_cancel,
				rent_fee.clone(),
				false,
				renter_cancellation_fee.clone(),
				rentee_cancellation_fee.clone(),
			);
			Contracts::<T>::insert(nft_id, contract);

			// NFT Updated 📦
			nft.state.is_rented = true;
			T::NFTExt::set_nft(nft_id, nft)?;

			// Event 🎁
			let event = Event::ContractCreated {
				nft_id,
				renter: who,
				duration,
				acceptance_type,
				renter_can_cancel,
				rent_fee,
				renter_cancellation_fee,
				rentee_cancellation_fee,
			};
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Revoke a running contract.
		#[pallet::weight(T::WeightInfo::revoke_contract(Queues::<T>::get().size() as u32))]
		pub fn revoke_contract(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();
			let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;

			let rentee = contract.rentee.as_ref().ok_or(Error::<T>::ContractIsNotRunning)?;
			let is_renter = contract.is_renter(&who).is_some();
			let is_rentee = rentee == &who;
			ensure!(is_renter || is_rentee, Error::<T>::NotAContractParticipant);

			if is_renter {
				ensure!(contract.renter_can_cancel, Error::<T>::ContractCannotBeCanceledByRenter);
			}

			// Let's first return the cancellation fee of the damaged party. 📦
			let return_fee = if is_renter {
				(contract.rentee_cancellation_fee.clone(), rentee.clone())
			} else {
				(contract.renter_cancellation_fee.clone(), contract.renter.clone())
			};
			Self::return_cancellation_fee(&return_fee.0, &return_fee.1)?;

			// Now let's move the revoker cancellation fee to the damaged party. 📦
			// Since we have flexible tokens we need to calculate how much we need to return.
			let return_fee = if is_renter {
				(contract.renter_cancellation_fee.clone(), rentee.clone())
			} else {
				(contract.rentee_cancellation_fee.clone(), contract.renter.clone())
			};

			// God help us if it is flexible
			if let Some(full_amount) = return_fee.0.as_flexible() {
				let completion = contract.completion(&now);
				let to_damaged_party = completion * full_amount;
				let to_caller = full_amount.saturating_sub(to_damaged_party);

				let src = &Self::account_id();
				T::Currency::transfer(src, &return_fee.1, to_damaged_party, AllowDeath)?;
				T::Currency::transfer(src, &who, to_caller, AllowDeath)?;
			} else {
				Self::return_cancellation_fee(&return_fee.0, &return_fee.1)?;
			}

			// Remove from corresponding queues / mappings. 📦
			Queues::<T>::mutate(|queues| {
				queues.remove(nft_id, contract.duration.queue_kind());
			});
			nft.state.is_rented = false;
			T::NFTExt::set_nft(nft_id, nft)?;
			Contracts::<T>::remove(nft_id);

			// Event 🎁
			let event = Event::ContractRevoked { nft_id, revoked_by: who };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Cancel a contract that is not running.
		#[pallet::weight(T::WeightInfo::cancel_contract(Queues::<T>::get().size() as u32))]
		pub fn cancel_contract(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;

			// Caller check ✅
			ensure!(contract.renter == who, Error::<T>::NotTheContractOwner);
			ensure!(contract.rentee.is_none(), Error::<T>::ContractIsRunning);

			// Queue updated 📦
			Queues::<T>::mutate(|x| {
				x.remove(nft_id, QueueKind::Available);
			});

			// Sent NFT back to original owner. Remove Contract and Offers 📦
			Self::handle_finished_or_unused_contract(nft_id)?;
			Offers::<T>::remove(nft_id);

			// Event 🎁
			let event = Event::ContractCanceled { nft_id };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Rent an NFT.
		#[pallet::weight(T::WeightInfo::rent(Queues::<T>::get().size() as u32))]
		pub fn rent(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let pallet = Self::account_id();
			let now = frame_system::Pallet::<T>::block_number();

			Contracts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;

				ensure!(contract.renter != who, Error::<T>::CannotRentOwnContract);
				ensure!(
					!contract.is_manual_acceptance(),
					Error::<T>::ContractDoesNotSupportAutomaticRent
				);

				// Caller needs to be whitelisted if such a list exists. ✅
				if let Some(list) = contract.acceptance_type.get_allow_list() {
					ensure!(list.contains(&who), Error::<T>::NotWhitelisted);
				}

				// Rent and Cancellation Fees are taken 📦
				Self::take_rent_and_cancellation_fee(&who, &pallet, &contract)?;

				// Queue and Offers updated 📦
				Queues::<T>::mutate(|queues| {
					queues.remove(nft_id, QueueKind::Available);

					let target_block = now + *contract.duration.get_duration_or_period();
					let queue_kind = contract.duration.queue_kind();
					queues.insert(nft_id, target_block, queue_kind).expect("qed");
				});
				Offers::<T>::remove(nft_id);

				contract.rentee = Some(who.clone());
				contract.start_block = Some(now);

				Ok(())
			})?;

			// Event 🎁
			let event = Event::ContractStarted { nft_id, rentee: who };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Make a offer.
		#[pallet::weight(T::WeightInfo::make_rent_offer(Queues::<T>::get().size() as u32))]
		pub fn make_rent_offer(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;

			ensure!(contract.renter != who, Error::<T>::CannotRentOwnContract);
			ensure!(contract.is_manual_acceptance(), Error::<T>::ContractDoesNotSupportOffers);

			let rent_fee = &contract.rent_fee;
			let cancellation_fee = &contract.rentee_cancellation_fee;

			// Let's see if he is on the allowed list ✅
			if let Some(list) = contract.acceptance_type.get_allow_list() {
				ensure!(list.contains(&who), Error::<T>::NotWhitelisted);
			}

			// Balance Check  ✅
			if let Some(amount) = rent_fee.get_balance() {
				ensure!(Self::balance_check(&who, amount), Error::<T>::NotEnoughFundsForRentFee);
			}

			// Balance Check  ✅
			if let Some(amount) = cancellation_fee.get_balance() {
				ensure!(
					Self::balance_check(&who, amount),
					Error::<T>::NotEnoughFundsForCancellationFee
				);
			}

			// Rent and Renter Cancellation NFT Check ✅
			let maybe_rent_nft = contract.rent_fee.get_nft();
			let maybe_cancel_nft = cancellation_fee.get_nft();

			if let Some(nft_id) = &maybe_rent_nft {
				let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::RentNFTNotFound)?;
				ensure!(nft.owner == who, Error::<T>::NotTheRentedNFTOwner);
				ensure!(
					nft.not_in_state(&Self::invalid_state()).is_ok(),
					Error::<T>::RentedNFTNotInValidState
				);
			}
			if let Some(nft_id) = &maybe_cancel_nft {
				let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::CancellationNFTNotFound)?;
				ensure!(nft.owner == who, Error::<T>::NotTheCancellationNFTOwner);
				ensure!(
					nft.not_in_state(&Self::invalid_state()).is_ok(),
					Error::<T>::CancellationNFTNotInValidState
				);
			}

			// Offers Updated 📦
			Offers::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				if let Some(offers) = x.as_mut() {
					offers.try_push(who.clone()).map_err(|_| Error::<T>::MaximumOffersReached)?;
				} else {
					let offers = BoundedVec::try_from(vec![who.clone()])
						.map_err(|_| Error::<T>::MaximumOffersReached)?;
					*x = Some(offers);
				}

				Ok(())
			})?;

			// Event 🎁
			let event = Event::ContractOfferCreated { nft_id, rentee: who };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Accept a rent offer for manual acceptance contract.
		#[pallet::weight(T::WeightInfo::accept_rent_offer(Queues::<T>::get().size() as u32))]
		pub fn accept_rent_offer(
			origin: OriginFor<T>,
			nft_id: NFTId,
			rentee: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			Contracts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;
				let pallet = Self::account_id();

				ensure!(contract.renter == who, Error::<T>::NotTheContractOwner);
				let offers = Offers::<T>::get(nft_id).ok_or(Error::<T>::NoOffersForThisContract)?;
				let offer_found = offers.contains(&rentee);
				ensure!(offer_found, Error::<T>::NoOfferFoundForThatRenteeAddress);

				// Rent and Cancellation Fees are taken 📦
				Self::take_rent_and_cancellation_fee(&rentee, &pallet, &contract)?;

				// All good ☀️
				// Queue and Offers updated 📦
				Queues::<T>::mutate(|queues| {
					queues.remove(nft_id, QueueKind::Available);
					let target_block = now + *contract.duration.get_duration_or_period();
					let queue_kind = contract.duration.queue_kind();
					queues.insert(nft_id, target_block, queue_kind).expect("qed");
				});

				Offers::<T>::remove(nft_id);

				contract.rentee = Some(rentee.clone());
				contract.start_block = Some(now);

				Ok(())
			})?;

			// Event 🎁
			let event = Event::ContractStarted { nft_id, rentee };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Retract a rent offer for manual acceptance contract.
		#[pallet::weight(T::WeightInfo::retract_rent_offer(Offers::<T>::get(nft_id).map_or_else(|| 0, |o| o.len()) as u32))]
		pub fn retract_rent_offer(
			origin: OriginFor<T>,
			nft_id: NFTId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			Offers::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				// TODO Error naming.
				let offers = x.as_mut().ok_or(Error::<T>::OfferNotFound)?;
				let index =
					offers.iter().position(|x| *x == who).ok_or(Error::<T>::OfferNotFound)?;
				offers.remove(index);

				Ok(())
			})?;

			// Event 🎁
			let event = Event::ContractOfferRetracted { nft_id, rentee: who };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Change the subscription terms for subscription contracts.
		#[pallet::weight(T::WeightInfo::change_subscription_terms(Queues::<T>::get().size() as u32))]
		pub fn change_subscription_terms(
			origin: OriginFor<T>,
			nft_id: NFTId,
			period: T::BlockNumber,
			max_duration: Option<T::BlockNumber>,
			rent_fee: BalanceOf<T>,
			changeable: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Changing Contracts and Cleared Offers 📦
			Contracts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;

				ensure!(who == contract.renter, Error::<T>::NotTheContractOwner);
				ensure!(
					contract.can_adjust_subscription(),
					Error::<T>::CannotAdjustSubscriptionTerms
				);

				if contract.rentee.is_none() {
					Offers::<T>::remove(nft_id);
				}

				contract.duration = Duration::Subscription(period, max_duration, changeable);
				contract.rent_fee = RentFee::Tokens(rent_fee);
				contract.terms_changed = contract.rentee.is_some();

				Ok(())
			})?;

			// Event 🎁
			let event =
				Event::ContractSubscriptionTermsChanged { nft_id, period, max_duration, rent_fee };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Accept the new contract terms.
		#[pallet::weight(T::WeightInfo::accept_subscription_terms(Queues::<T>::get().size() as u32))]
		pub fn accept_subscription_terms(
			origin: OriginFor<T>,
			nft_id: NFTId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Changing Contracts 📦
			Contracts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;

				ensure!(Some(who) == contract.rentee, Error::<T>::NotTheContractRentee);
				ensure!(contract.terms_changed, Error::<T>::ContractTermsAlreadyAccepted);

				contract.terms_changed = false;

				Ok(())
			})?;

			// Event 🎁
			let event = Event::ContractSubscriptionTermsAccepted { nft_id };
			Self::deposit_event(event);

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The account ID of the cancellation fee pot.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	pub fn change_nft_ownership(id: NFTId, new_owner: &T::AccountId) -> DispatchResult {
		T::NFTExt::mutate_nft(id, |x| -> DispatchResult {
			let nft = x.as_mut().ok_or(Error::<T>::NFTNotFound)?;
			nft.owner = new_owner.clone();
			Ok(())
		})
	}

	pub fn return_cancellation_fee(
		fee: &CancellationFee<BalanceOf<T>>,
		dst: &T::AccountId,
	) -> DispatchResult {
		if let Some(amount) = fee.get_balance() {
			let src = &Self::account_id();
			T::Currency::transfer(src, dst, amount, AllowDeath)?
		}

		if let Some(nft_id) = fee.get_nft() {
			T::NFTExt::mutate_nft(nft_id, |x| -> DispatchResult {
				let nft = x.as_mut().ok_or(Error::<T>::NFTNotFound)?;
				nft.owner = dst.clone();
				Ok(())
			})?;
		}

		Ok(())
	}

	pub fn handle_subscription_contract(
		nft_id: NFTId,
		now: &T::BlockNumber,
	) -> Option<T::BlockNumber> {
		let contract = Contracts::<T>::get(nft_id).expect("Should not happen. qed");
		let rentee = contract.rentee.clone().expect("Should not happen. qed");
		let rent_fee = contract.rent_fee.get_balance().expect("This cannot happen. qed");

		let mut cancel_subscription = contract.terms_changed || contract.has_ended(now);
		if !cancel_subscription {
			cancel_subscription =
				T::Currency::transfer(&rentee, &contract.renter, rent_fee, KeepAlive).is_err();
		}

		if cancel_subscription {
			return None
		}

		let sub_duration = contract.duration.get_sub_period().expect("This cannot happen. qed");

		// TODO This is not correct.
		// It can happen that this rent contract is processed later than it should so we need to
		// adjust for that.
		Some(*now + sub_duration)
	}

	pub fn handle_finished_or_unused_contract(nft_id: NFTId) -> DispatchResult {
		let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;
		T::NFTExt::mutate_nft(nft_id, |x| -> DispatchResult {
			let nft = x.as_mut().ok_or(Error::<T>::NFTNotFound)?;
			let src = &Self::account_id();

			let renter = &contract.renter;
			let renter_cancellation = &contract.renter_cancellation_fee;
			let rentee_cancellation = &contract.rentee_cancellation_fee;

			// Let's first do the transactions
			if let Some(amount) = renter_cancellation.get_balance() {
				T::Currency::transfer(src, renter, amount, AllowDeath)?;
			}

			if let Some(rentee) = &contract.rentee {
				if let Some(amount) = rentee_cancellation.get_balance() {
					T::Currency::transfer(src, rentee, amount, AllowDeath)?;
				}
			}

			// Now lets get the NFTs
			if let Some(nft_id) = renter_cancellation.get_nft() {
				Self::change_nft_ownership(nft_id, &renter)?;
			}

			if let Some(rentee) = &contract.rentee {
				if let Some(nft_id) = rentee_cancellation.get_nft() {
					Self::change_nft_ownership(nft_id, rentee)?;
				}
			}

			nft.state.is_rented = false;
			Ok(())
		})?;

		Contracts::<T>::remove(nft_id);
		Ok(())
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

	pub fn take_rent_and_cancellation_fee(
		rentee: &T::AccountId,
		pallet: &T::AccountId,
		contract: &RentContractDataOf<T>,
	) -> DispatchResult {
		let renter = &contract.renter;
		let cancellation_fee = contract.rentee_cancellation_fee.clone();

		// Let's take rentee's token. In case an error happens those balance transactions
		// will be reverted. ✅ 📦
		if let Some(amount) = contract.rent_fee.get_balance() {
			T::Currency::transfer(rentee, renter, amount, KeepAlive)
				.map_err(|_| Error::<T>::NotEnoughFundsForRentFee)?;
		}

		if let Some(amount) = cancellation_fee.get_balance() {
			T::Currency::transfer(rentee, pallet, amount, KeepAlive)
				.map_err(|_| Error::<T>::NotEnoughFundsForCancellationFee)?;
		}

		// Let's take source's NFTs.
		let maybe_rent_nft = contract.rent_fee.get_nft();
		let maybe_cancel_nft = cancellation_fee.get_nft();

		// Rent and Renter Cancellation NFT Check ✅
		if let Some(nft_id) = &maybe_rent_nft {
			let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::RentNFTNotFound)?;
			ensure!(nft.owner == *renter, Error::<T>::RenteeDoesNotOwnTheRentNFT);
			ensure!(
				nft.not_in_state(&Self::invalid_state()).is_ok(),
				Error::<T>::RentedNFTNotInValidState
			);
		}
		if let Some(nft_id) = &maybe_cancel_nft {
			let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::CancellationNFTNotFound)?;
			ensure!(nft.owner == *renter, Error::<T>::RenteeDoesNotOwnTheCancellationNFT);
			ensure!(
				nft.not_in_state(&Self::invalid_state()).is_ok(),
				Error::<T>::CancellationNFTNotInValidState
			);
		}

		// Rent and Renter Cancellation NFT Taken 📦
		if let Some(nft_id) = &maybe_rent_nft {
			Self::change_nft_ownership(*nft_id, &renter)
				.map_err(|_| Error::<T>::RentNFTNotFound)?;
		}
		if let Some(nft_id) = &maybe_cancel_nft {
			Self::change_nft_ownership(*nft_id, &pallet)
				.map_err(|_| Error::<T>::RentNFTNotFound)?;
		}

		Ok(())
	}

	pub fn invalid_state() -> Vec<NFTStateModifiers> {
		vec![Capsule, IsListed, Delegated, Soulbound, Rented]
	}
}

impl<T: Config> Pallet<T> {
	pub fn prep_benchmark_0(
		account: &T::AccountId,
		origin: frame_system::Origin<T>,
		contract_amount: u32,
	) -> Result<(), DispatchError> {
		let text = "I like to drink milk, eat sugar and dance the orange dance".as_bytes().to_vec();
		let offchain_data = BoundedVec::try_from(text).unwrap();
		let royalty = Permill::from_percent(0);
		let duration = Duration::Fixed(100u32.into());
		let acceptance_type = AcceptanceType::AutoAcceptance(None);
		let rent_fee = RentFee::Tokens(200u32.into());

		for _i in 0..contract_amount {
			let nft_id = T::NFTExt::create_nft(
				account.clone(),
				offchain_data.clone(),
				royalty.clone(),
				None,
				false,
			);
			if let Err(res) = nft_id {
				if let Err(err) = res {
					return Err(err.into())
				}
			}
			let nft_id = nft_id.unwrap();

			Self::create_contract(
				origin.clone().into(),
				nft_id,
				duration.clone(),
				acceptance_type.clone(),
				false,
				rent_fee.clone(),
				CancellationFee::None,
				CancellationFee::None,
			)
			.map_err(|x| x.error)?;
		}

		Ok(())
	}

	/// Fill offers vector with any number of data.
	pub fn prep_benchmark_1(
		number: u32,
		nft_id: NFTId,
		account: T::AccountId,
	) -> Result<(), DispatchError> {
		let offers: AccountList<T::AccountId, T::AccountSizeLimit> =
			BoundedVec::try_from(vec![account; number as usize])
				.map_err(|_| Error::<T>::MaximumOffersReached)?;
		Offers::<T>::insert(nft_id, offers);
		Ok(())
	}
}
