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
pub use weights::WeightInfo;

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
use primitives::nfts::{
	NFTId,
	NFTStateModifiers::{self, *},
};
use sp_runtime::{
	traits::{AccountIdConversion, CheckedSub, Saturating},
	Permill,
};
use sp_std::prelude::*;
use ternoa_common::traits::NFTExt;

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
		type MaximumContractAvailabilityLimit: Get<u32>;

		/// Maximum number of blocks that a contract can last for.
		#[pallet::constant]
		type MaximumContractDurationLimit: Get<u32>;
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
			renter_can_revoke: bool,
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
			max_duration: T::BlockNumber,
			is_changeable: bool,
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
		/// NFT was not found.
		NFTNotFound,
		/// Rented NFT was not found.
		RentNFTNotFound,
		/// Cancellation NFT was not found.
		CancellationNFTNotFound,
		/// The caller is not the renter or rentee.
		NotAContractParticipant,
		/// Cannot revoke a non running contract.
		CannotRevokeNonRunningContract,
		/// Cannot cancel a running contract.
		CannotCancelRunningContract,
		/// Contract is not allowed to be canceled by renter.
		ContractCannotBeCanceledByRenter,
		/// Contract does not support automatic rent.
		ContractDoesNotSupportAutomaticRent,
		/// Contract does not allow for offers to be made.
		ContractDoesNotSupportOffers,
		/// The caller is not whitelisted.
		NotWhitelisted,
		/// The caller does not own the rent NFT.
		CallerDoesNotOwnRentNFT,
		/// The caller does not own the cancellation NFT.
		CallerDoesNotOwnCancellationNFT,
		/// Rentee does not own the rent NFT.
		RenteeDoesNotOwnTheRentNFT,
		/// Rentee does not own the cancellation NFT.
		RenteeDoesNotOwnTheCancellationNFT,
		/// Not Enough funds for rent fee.
		NotEnoughFundsForRentFee,
		/// Not Enough funds for cancellation fee.
		NotEnoughFundsForCancellationFee,
		/// Not enough funds for cancellation fee + rent fee.
		NotEnoughFundsForFees,
		/// The caller is not the contract owner.
		NotTheContractOwner,
		/// The caller is not the contract rentee.
		NotTheContractRentee,
		/// Contract NFT is not in a valid state.
		ContractNFTNotInAValidState,
		/// Rent NFT is not in a valid state.
		RentNFTNotInValidState,
		/// Cancellation NFT is not in a valid state.
		CancellationNFTNotInValidState,
		/// The caller is not the owner of the contract NFT.
		NotTheNFTOwner,
		/// The chain cannot accept new NFT contracts. Maximum limit reached.
		MaxSimultaneousContractReached,
		/// The contract was not found.
		ContractNotFound,
		/// Cannot Rent your own contract.
		CannotRentOwnContract,
		/// The contract cannot accept new offers. Maximum limit reached.
		MaximumOffersReached,
		/// Operation is not permitted because contract terms are already accepted.
		ContractTermsAlreadyAccepted,
		/// No offers were found for that contract.
		NoOffersForThisContract,
		/// No offers were found for that address.
		NoOfferFromThisAddress,
		/// Duration and revocation mismatch.
		DurationAndRentFeeMismatch,
		/// Duration and cancellation mismatch.
		DurationAndCancellationFeeMismatch,
		/// Cannot adjust subscription Terms.
		CannotAdjustSubscriptionTerms,
		/// Contract is not running.
		ContractIsNotRunning,
		/// Duration exceeds maximum limit
		DurationExceedsMaximumLimit,
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
					_ = queues.subscription_queue.insert(nft_id, block_number);
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
			duration: DurationInput<T::BlockNumber>,
			acceptance_type: AcceptanceType<AccountList<T::AccountId, T::AccountSizeLimit>>,
			renter_can_revoke: bool,
			rent_fee: RentFee<BalanceOf<T>>,
			renter_cancellation_fee: CancellationFee<BalanceOf<T>>,
			rentee_cancellation_fee: CancellationFee<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let pallet = Self::account_id();
			let mut queues = Queues::<T>::get();

			// Checks ✅
			queues.can_be_increased(1).ok_or(Error::<T>::MaxSimultaneousContractReached)?;

			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(
				nft.not_in_state(&Self::invalid_state()).is_ok(),
				Error::<T>::ContractNFTNotInAValidState
			);

			let duration_limit: T::BlockNumber = T::MaximumContractDurationLimit::get().into();
			let duration = duration.to_duration(duration_limit.clone());
			ensure!(
				*duration.get_full_duration() <= duration_limit,
				Error::<T>::DurationExceedsMaximumLimit
			);
			duration
				.allows_rent_fee(&rent_fee)
				.ok_or(Error::<T>::DurationAndRentFeeMismatch)?;
			duration
				.allows_cancellation(&renter_cancellation_fee)
				.ok_or(Error::<T>::DurationAndCancellationFeeMismatch)?;
			duration
				.allows_cancellation(&rentee_cancellation_fee)
				.ok_or(Error::<T>::DurationAndCancellationFeeMismatch)?;

			if let Some(id) = rent_fee.get_nft() {
				ensure!(T::NFTExt::exists(id), Error::<T>::RentNFTNotFound);
			}
			if let Some(id) = rentee_cancellation_fee.get_nft() {
				ensure!(T::NFTExt::exists(id), Error::<T>::CancellationNFTNotFound);
			}

			// Storage Activity 📦
			// 1. Transfer Cancellation NFT or Tokens to Pallet.
			// 2. Add Contract to the Queue
			// 3. Add Contract to Storage
			// 4. Set NFT to Rented State
			let amount = renter_cancellation_fee.get_balance().unwrap_or(0u32.into());
			T::Currency::transfer(&who, &pallet, amount, KeepAlive)
				.map_err(|_| Error::<T>::NotEnoughFundsForCancellationFee)?;

			if let Some(id) = renter_cancellation_fee.get_nft() {
				T::NFTExt::mutate_nft(id, |x| -> DispatchResult {
					let nft = x.as_mut().ok_or(Error::<T>::CancellationNFTNotFound)?;
					let is_valid = nft.not_in_state(&Self::invalid_state()).is_ok();
					ensure!(nft.owner == who, Error::<T>::CallerDoesNotOwnCancellationNFT);
					ensure!(is_valid, Error::<T>::CancellationNFTNotInValidState);

					nft.owner = pallet;
					Ok(())
				})?;
			}

			let now = frame_system::Pallet::<T>::block_number();
			let expiration_block = now + T::MaximumContractAvailabilityLimit::get().into();
			queues
				.insert(nft_id, expiration_block, QueueKind::Available)
				.map_err(|_| Error::<T>::MaxSimultaneousContractReached)?; // This should never happen since we already did the check.
			Queues::<T>::set(queues);

			let contract = RentContractData::new(
				None,
				who.clone(),
				None,
				duration.clone(),
				acceptance_type.clone(),
				renter_can_revoke,
				rent_fee.clone(),
				renter_cancellation_fee.clone(),
				rentee_cancellation_fee.clone(),
			);
			Contracts::<T>::insert(nft_id, contract);

			_ = nft.set_state(Rented, true);
			T::NFTExt::set_nft(nft_id, nft)?;

			// Event 🎁
			let event = Event::ContractCreated {
				nft_id,
				renter: who,
				duration,
				acceptance_type,
				renter_can_revoke,
				rent_fee,
				renter_cancellation_fee,
				rentee_cancellation_fee,
			};
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Cancel a contract that is not running.
		#[pallet::weight(T::WeightInfo::cancel_contract(Queues::<T>::get().size() as u32))]
		pub fn cancel_contract(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;

			// Checks ✅
			ensure!(contract.renter == who, Error::<T>::NotTheContractOwner);
			ensure!(contract.rentee.is_none(), Error::<T>::CannotCancelRunningContract);

			// Storage Activity 📦
			// 1. Remove Contract From Queue
			// 2. Remove Offers from Storage
			// 3. Return Renter Cancellation NFT & Tokens
			// 4. Remove Contract from Storage
			// 5. Remove Rented state from NFT
			Queues::<T>::mutate(|x| {
				x.remove(nft_id, QueueKind::Available);
			});
			Offers::<T>::remove(nft_id);
			Self::handle_finished_or_unused_contract(nft_id)?;

			// Event 🎁
			let event = Event::ContractCanceled { nft_id };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Revoke a running contract.
		#[pallet::weight(T::WeightInfo::revoke_contract(Queues::<T>::get().size() as u32))]
		pub fn revoke_contract(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();
			let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;
			let rentee_cancellation = &contract.rentee_cancellation_fee;
			let renter_cancellation = &contract.renter_cancellation_fee;
			let renter = &contract.renter;

			// Checks ✅
			let rentee =
				contract.rentee.as_ref().ok_or(Error::<T>::CannotRevokeNonRunningContract)?;
			let is_renter = contract.renter == who;
			let is_rentee = rentee == &who;
			ensure!(is_renter || is_rentee, Error::<T>::NotAContractParticipant);

			if is_renter {
				ensure!(contract.renter_can_revoke, Error::<T>::ContractCannotBeCanceledByRenter);
			}

			// Storage Activity 📦
			// Here we do storage activity without checking first. This can go horribly wrong if
			// `create_contract`, `rent` or `make_rent_offer` mess up things!!!
			// 1. Return damaged party cancellation fee
			// 2. Send the offender cancellation fee to damaged party
			// 3. Remove Contract from Queue
			// 4. Remove Contract from Storage
			// 5. Remove Rented state from NFT
			let (deposited_fee, price_to_pay) = match is_renter {
				true => ((rentee_cancellation, rentee), (renter_cancellation, rentee)),
				false => ((renter_cancellation, renter), (rentee_cancellation, renter)),
			};

			Self::return_cancellation_fee(deposited_fee.0, deposited_fee.1)?;
			if let Some(full_amount) = price_to_pay.0.as_flexible() {
				let percent = contract.percentage_of_completion(&now);
				Self::return_flexible_fee(&who, price_to_pay.1, percent, full_amount)?;
			} else {
				Self::return_cancellation_fee(price_to_pay.0, price_to_pay.1)?;
			}

			Queues::<T>::mutate(|queues| {
				queues.remove(nft_id, contract.duration.queue_kind());
			});
			Contracts::<T>::remove(nft_id);
			T::NFTExt::mutate_nft(nft_id, |x| -> DispatchResult {
				let nft = x.as_mut().ok_or(Error::<T>::NFTNotFound)?;
				_ = nft.set_state(Rented, false);
				Ok(())
			})?;

			// Event 🎁
			let event = Event::ContractRevoked { nft_id, revoked_by: who };
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

				// Checks ✅
				ensure!(contract.renter != who, Error::<T>::CannotRentOwnContract);
				ensure!(
					!contract.is_manual_acceptance(),
					Error::<T>::ContractDoesNotSupportAutomaticRent
				);
				if let Some(list) = contract.acceptance_type.get_allow_list() {
					ensure!(list.contains(&who), Error::<T>::NotWhitelisted);
				}

				// Storage Activity 📦
				// 1. Take Rent and Cancellation Fee from Caller
				// 2. Move Contract Queue from Available to Active
				// 3. Remove Offers
				// 4. Set Contract Start Block and Rentee
				Self::take_rent_and_cancellation_fee(&who, &pallet, &contract)?;

				Queues::<T>::mutate(|queues| -> DispatchResult {
					queues.remove(nft_id, QueueKind::Available);

					let target_block = now + *contract.duration.get_duration_or_period();
					let queue_kind = contract.duration.queue_kind();
					queues
						.insert(nft_id, target_block, queue_kind)
						.map_err(|_| Error::<T>::MaxSimultaneousContractReached)?; // This should never happen.

					Ok(())
				})?;
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
			let rent_fee = &contract.rent_fee;
			let cancellation_fee = &contract.rentee_cancellation_fee;

			// Checks ✅
			ensure!(contract.renter != who, Error::<T>::CannotRentOwnContract);
			ensure!(contract.is_manual_acceptance(), Error::<T>::ContractDoesNotSupportOffers);
			if let Some(list) = contract.acceptance_type.get_allow_list() {
				ensure!(list.contains(&who), Error::<T>::NotWhitelisted);
			}

			let rent_balance = rent_fee.get_balance().unwrap_or(0u32.into());
			let cancel_balance = cancellation_fee.get_balance().unwrap_or(0u32.into());
			ensure!(Self::balance_check(&who, rent_balance), Error::<T>::NotEnoughFundsForRentFee);
			ensure!(
				Self::balance_check(&who, cancel_balance),
				Error::<T>::NotEnoughFundsForCancellationFee
			);
			ensure!(
				Self::balance_check(&who, rent_balance + cancel_balance),
				Error::<T>::NotEnoughFundsForFees
			);

			let maybe_rent_nft = rent_fee.get_nft();
			let maybe_cancel_nft = cancellation_fee.get_nft();
			if let Some(nft_id) = &maybe_rent_nft {
				let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::RentNFTNotFound)?;
				ensure!(nft.owner == who, Error::<T>::CallerDoesNotOwnRentNFT);
				ensure!(
					nft.not_in_state(&Self::invalid_state()).is_ok(),
					Error::<T>::RentNFTNotInValidState
				);
			}
			if let Some(nft_id) = &maybe_cancel_nft {
				let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::CancellationNFTNotFound)?;
				ensure!(nft.owner == who, Error::<T>::CallerDoesNotOwnCancellationNFT);
				ensure!(
					nft.not_in_state(&Self::invalid_state()).is_ok(),
					Error::<T>::CancellationNFTNotInValidState
				);
			}

			// Storage Activity 📦
			// 1. Add Offer to Offer list
			Offers::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				if x.is_none() {
					*x = Some(BoundedVec::default());
				}
				let offers = x.as_mut().ok_or(Error::<T>::NoOffersForThisContract)?; // This should never happen.
				offers.try_push(who.clone()).map_err(|_| Error::<T>::MaximumOffersReached)?;

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
				let pallet = Self::account_id();
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;
				let offers = Offers::<T>::get(nft_id).ok_or(Error::<T>::NoOffersForThisContract)?;
				let offer_found = offers.contains(&rentee);

				// Checks ✅
				ensure!(contract.renter == who, Error::<T>::NotTheContractOwner);
				ensure!(offer_found, Error::<T>::NoOfferFromThisAddress);

				// Storage Activity 📦
				// 1. Take Rent and Cancellation Fee from Caller
				// 2. Move Contract Queue from Available to Active
				// 3. Remove Offers
				// 4. Set Contract Start Block and Rentee
				Self::take_rent_and_cancellation_fee(&rentee, &pallet, &contract)?;

				Queues::<T>::mutate(|queues| -> DispatchResult {
					queues.remove(nft_id, QueueKind::Available);
					let target_block = now + *contract.duration.get_duration_or_period();
					let queue_kind = contract.duration.queue_kind();
					queues
						.insert(nft_id, target_block, queue_kind)
						.map_err(|_| Error::<T>::MaxSimultaneousContractReached)?; // This should never happen.

					Ok(())
				})?;
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
				// Checks ✅
				let offers = x.as_mut().ok_or(Error::<T>::NoOffersForThisContract)?;
				let index = offers.iter().position(|x| *x == who);
				let index = index.ok_or(Error::<T>::NoOfferFromThisAddress)?;

				// Storage Activity 📦
				// 1. Remove offer from Offers
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
			rent_fee: BalanceOf<T>,
			period: T::BlockNumber,
			max_duration: Option<T::BlockNumber>,
			is_changeable: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let duration_limit: T::BlockNumber = T::MaximumContractDurationLimit::get().into();
			let max_duration = max_duration.unwrap_or_else(|| duration_limit.clone());

			Contracts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;
				let contract_active = contract.rentee.is_some();

				// Checks ✅
				ensure!(who == contract.renter, Error::<T>::NotTheContractOwner);
				ensure!(
					contract.can_adjust_subscription(),
					Error::<T>::CannotAdjustSubscriptionTerms
				);
				ensure!(max_duration <= duration_limit, Error::<T>::DurationExceedsMaximumLimit);
				if !contract_active {
					Offers::<T>::remove(nft_id);
				}

				// Storage Activity 📦
				// 1. Change Contract Duration, Rent Fee
				// 2. Set Contract Terms Changed field to true (or false if there is no rentee)
				contract.duration = Duration::Subscription(Subscription {
					period_length: period,
					max_duration,
					is_changeable,
					new_terms: contract_active,
				});
				contract.rent_fee = RentFee::Tokens(rent_fee);
				contract.duration.set_terms_changed(true);

				Ok(())
			})?;

			// Event 🎁
			let event = Event::ContractSubscriptionTermsChanged {
				nft_id,
				period,
				max_duration,
				is_changeable,
				rent_fee,
			};
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

			Contracts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;

				// Checks ✅
				ensure!(Some(who) == contract.rentee, Error::<T>::NotTheContractRentee);
				ensure!(
					contract.duration.terms_changed(),
					Error::<T>::ContractTermsAlreadyAccepted
				);

				// Storage Activity 📦
				// 1. Set Contract Terms Changed field to false
				contract.duration.set_terms_changed(false);

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

	pub fn return_flexible_fee(
		offender: &T::AccountId,
		damaged_party: &T::AccountId,
		percent: Permill,
		full_amount: BalanceOf<T>,
	) -> DispatchResult {
		let to_offender = percent * full_amount;
		let to_damaged_party = full_amount.saturating_sub(to_offender);

		let src = &Self::account_id();
		T::Currency::transfer(src, &offender, to_offender, AllowDeath)?;
		T::Currency::transfer(src, &damaged_party, to_damaged_party, AllowDeath)?;
		Ok(())
	}

	pub fn return_cancellation_fee(
		fee: &CancellationFee<BalanceOf<T>>,
		dst: &T::AccountId,
	) -> DispatchResult {
		let amount = fee.get_balance().unwrap_or(0u32.into());
		T::Currency::transfer(&Self::account_id(), dst, amount, AllowDeath)?;

		if let Some(nft_id) = fee.get_nft() {
			Self::change_nft_ownership(nft_id, &dst)?;
		}

		Ok(())
	}

	pub fn handle_subscription_contract(
		nft_id: NFTId,
		now: &T::BlockNumber,
	) -> Option<T::BlockNumber> {
		let contract = Contracts::<T>::get(nft_id)?;
		let rentee = contract.rentee.as_ref()?;
		let rent_fee = contract.rent_fee.get_balance()?;

		let mut cancel_subscription = contract.duration.terms_changed() || contract.has_ended(now);
		if !cancel_subscription {
			cancel_subscription =
				T::Currency::transfer(rentee, &contract.renter, rent_fee, KeepAlive).is_err();
		}

		if cancel_subscription {
			return None
		}

		let sub_duration = contract.duration.get_duration_or_period();

		// TODO This is not fully correct.
		// It can happen that this rent contract is processed later than it should so we need to
		// adjust for that.
		Some(*now + *sub_duration)
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
			let amount = renter_cancellation.get_balance().unwrap_or(0u32.into());
			T::Currency::transfer(src, renter, amount, AllowDeath)?;

			if let Some(rentee) = &contract.rentee {
				let amount = rentee_cancellation.get_balance().unwrap_or(0u32.into());
				T::Currency::transfer(src, rentee, amount, AllowDeath)?;
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

			_ = nft.set_state(Rented, false);
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
		let amount = contract.rent_fee.get_balance().unwrap_or(0u32.into());
		T::Currency::transfer(rentee, renter, amount, KeepAlive)
			.map_err(|_| Error::<T>::NotEnoughFundsForRentFee)?;

		let amount = cancellation_fee.get_balance().unwrap_or(0u32.into());
		T::Currency::transfer(rentee, pallet, amount, KeepAlive)
			.map_err(|_| Error::<T>::NotEnoughFundsForCancellationFee)?;

		// Let's take source's NFTs.
		let maybe_rent_nft = contract.rent_fee.get_nft();
		let maybe_cancel_nft = cancellation_fee.get_nft();

		// Rent and Rentee Cancellation NFT Check ✅
		if let Some(nft_id) = &maybe_rent_nft {
			let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::RentNFTNotFound)?;
			ensure!(nft.owner == *rentee, Error::<T>::RenteeDoesNotOwnTheRentNFT);
			ensure!(
				nft.not_in_state(&Self::invalid_state()).is_ok(),
				Error::<T>::RentNFTNotInValidState
			);
		}
		if let Some(nft_id) = &maybe_cancel_nft {
			let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::CancellationNFTNotFound)?;
			ensure!(nft.owner == *rentee, Error::<T>::RenteeDoesNotOwnTheCancellationNFT);
			ensure!(
				nft.not_in_state(&Self::invalid_state()).is_ok(),
				Error::<T>::CancellationNFTNotInValidState
			);
		}

		// Rent and Rentee Cancellation NFT Taken 📦
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
		vec![Capsule, IsListed, Delegated, Soulbound, SecretSyncing, Rented]
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
		let duration = DurationInput::Fixed(100u32.into());
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
