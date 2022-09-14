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
	traits::{AccountIdConversion, CheckedDiv, CheckedSub, Saturating},
	SaturatedConversion,
};
use sp_std::prelude::*;

use primitives::nfts::{NFTData, NFTId, NFTStateModifiers::*};
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
			revocation_type: RevocationType,
			rent_fee: RentFee<BalanceOf<T>>,
			renter_cancellation_fee: Option<CancellationFee<BalanceOf<T>>>,
			rentee_cancellation_fee: Option<CancellationFee<BalanceOf<T>>>,
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
		ContractAvailableExpired { nft_id: NFTId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// NFT not found.
		NFTNotFound,
		/// Not the owner of the NFT.
		NotTheNFTOwner,
		/// Operation is not permitted because NFT is listed.
		CannotUseListedNFTs,
		/// Operation is not permitted because NFT is capsule.
		CannotUseCapsuleNFTs,
		/// Operation is not permitted because NFT is delegated.
		CannotUseDelegatedNFTs,
		/// Operation is not permitted because NFT is delegated.
		CannotUseSoulboundNFTs,
		/// Operation is not permitted because NFT is rented.
		CannotUseRentedNFTs,
		/// Operation is not permitted because NFT for cancellation fee was not found.
		NFTNotFoundForCancellationFee,
		/// Operation is not permitted because NFT for cancellation fee is not owned by caller.
		NotTheNFTOwnerForCancellationFee,
		/// Operation is not permitted because NFT for rent_fee fee is not owned by caller.
		NotTheNFTOwnerForRentFee,
		/// Operation is not permitted because the maximum number au parallel rent contract has
		/// been reached.
		MaxSimultaneousContractReached,
		/// Cannot create a contract with fixed / infinite duration and onSubscriptionChange
		/// revocation type.
		SubscriptionChangeForSubscriptionOnly,
		/// Cannot create a contract with infinite or subscription duration and flexible tokens
		/// cancellation fee.
		FlexibleFeeOnlyForFixedDuration,
		/// Cannot create a contract with no revocation type and a renter cancellation fee.
		NoRenterCancellationFeeWithNoRevocation,
		/// Cannot create a contract with NFT id rent fee type and subscription duration.
		NoNFTRentFeeWithSubscription,
		/// The contract was not found for the given nft_id.
		ContractNotFound,
		/// The caller is neither the renter or rentee.
		NotTheRenterOrRentee,
		/// The caller is not the renter.
		NotTheRenter,
		/// The caller is not the rentee.
		NotTheRentee,
		/// Operation is not permitted because revocation type is not anytime.
		CannotRevoke,
		/// End block not found for flexible fee calculation.
		FlexibleFeeEndBlockNotFound,
		/// Cannot Rent your own contract.
		CannotRentOwnContract,
		/// Rentee is not on authorized account list.
		NotAuthorizedForRent,
		/// Operation is not permitted because user has not enough funds.
		NotEnoughBalance,
		/// Operation is not permitted because user has not enough funds for cancellation fee.
		NotEnoughBalanceForCancellationFee,
		/// Operation is not permitted because user has not enough funds for rent fee.
		NotEnoughBalanceForRentFee,
		/// Fee NFT not found.
		NFTNotFoundForRentFee,
		/// Maximum offers reached.
		MaximumOffersReached,
		/// Operation is not permitted because acceptance type is not manual.
		CannotAcceptOfferForAutoAcceptance,
		/// Operation is not permitted because acceptance type is not manual.
		CannotRetractOfferForAutoAcceptance,
		/// Operation is not permitted because contract has started.
		ContractHasStarted,
		/// Operation is not permitted because contract has not started yet.
		ContractHasNotStarted,
		/// Terms can only be set for subscription duration and OnSubscriptionChange revocation
		/// type.
		CanChangeTermForSubscriptionOnly,
		/// New term must be suvscription.
		CanSetTermsForSubscriptionOnly,
		/// Operation is not permitted because contract terms are already accepted
		ContractTermsAlreadyAccepted,
		/// Operation is allowed only for subscription
		RenewalOnlyForSubscription,
		/// Operation is not allowed because same nft was used for contract /
		/// renter_cancellation_fee / rentee_cancellation_fee / rent_fee
		InvalidFeeNFT,
		/// No offers was found for the contract
		NoOffersForThisContract,
		/// No offer was made by the rentee
		NoOfferFromRentee,
		/// Math error.
		InternalMathError,
		/// Offer not found.
		OfferNotFound,
		/// Duration and Revocation Mismatch
		DurationAndRevocationMismatch,
		/// Duration and Revocation Mismatch
		DurationAndRentFeeMismatch,
		/// Duration and Cancellation Mismatch
		DurationAndCancellationFeeMismatch,
		/// Revocation and Cancellation Mismatch
		RevocationAndCancellationFeeMismatch,
		/// Cannot adjust subscription Terms.
		CannotAdjustSubscriptionTerms,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Weight: see `begin_block`
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut read = 0u64;
			let mut write = 0u64;
			let mut current_actions = 0;
			let max_actions = T::ActionsInBlockLimit::get();

			let mut queues = Queues::<T>::get();
			read += 1;
			// Fixed queue management
			while let Some(nft_id) = queues.fixed_queue.pop_next(now) {
				Self::handle_finished_or_unused_contract(nft_id);
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
					Self::handle_finished_or_unused_contract(nft_id);
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
				Self::handle_finished_or_unused_contract(nft_id);

				let event = Event::ContractAvailableExpired { nft_id };
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
		#[pallet::weight((
			{
				let s = Queues::<T>::get().available_queue.size();
				T::WeightInfo::create_contract(s as u32)
			},
			DispatchClass::Normal
		))]
		pub fn create_contract(
			origin: OriginFor<T>,
			nft_id: NFTId,
			duration: Duration<T::BlockNumber>,
			acceptance_type: AcceptanceType<AccountList<T::AccountId, T::AccountSizeLimit>>,
			revocation_type: RevocationType,
			rent_fee: RentFee<BalanceOf<T>>,
			renter_cancellation_fee: Option<CancellationFee<BalanceOf<T>>>,
			rentee_cancellation_fee: Option<CancellationFee<BalanceOf<T>>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut queues = Queues::<T>::get();

			// Queue ✅
			ensure!(
				queues.total_size() + 1 <= queues.limit(),
				Error::<T>::MaxSimultaneousContractReached
			);

			// NFT Check ✅
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			Self::check_nft_state_validity(&nft)?;

			// Duration and Revocation Check ✅
			duration
				.allows_revocation(&revocation_type)
				.ok_or(Error::<T>::DurationAndRevocationMismatch)?;
			duration
				.allows_rent_fee(&rent_fee)
				.ok_or(Error::<T>::DurationAndRentFeeMismatch)?;
			if let Some(x) = &renter_cancellation_fee {
				duration
					.allows_cancellation(x)
					.ok_or(Error::<T>::DurationAndCancellationFeeMismatch)?;
				revocation_type
					.allows_cancellation(x)
					.ok_or(Error::<T>::RevocationAndCancellationFeeMismatch)?;
			}
			if let Some(x) = &rentee_cancellation_fee {
				duration
					.allows_cancellation(x)
					.ok_or(Error::<T>::DurationAndCancellationFeeMismatch)?;
			}

			// Rent Fee Check ✅
			if let Some(nft_id) = rent_fee.get_nft() {
				T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFoundForRentFee)?;
			}

			// Renter Cancellation Check  ✅
			if let Some(id) = renter_cancellation_fee.clone().and_then(|x| x.get_nft()) {
				let nft =
					T::NFTExt::get_nft(id).ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
				ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
				Self::check_nft_state_validity(&nft)?;
			}

			// Rentee Cancellation Check  ✅
			if let Some(id) = rentee_cancellation_fee.clone().and_then(|x| x.get_nft()) {
				T::NFTExt::get_nft(id).ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
			}

			// Checking done, time to change the storage 📦
			// Renter Cancellation fee taken  📦
			if let Some(fee) = rentee_cancellation_fee.clone() {
				if let Some(amount) = fee.get_balance() {
					T::Currency::transfer(&who, &Self::account_id(), amount, KeepAlive)?;
				}

				if let Some(id) = fee.get_nft() {
					let mut nft = T::NFTExt::get_nft(id).expect("Checked before. qed");
					nft.owner = Self::account_id();
					T::NFTExt::set_nft(nft_id, nft)?;
				}
			}

			// Queue Updated  📦
			let expiration_block = frame_system::Pallet::<T>::block_number() +
				T::ContractExpirationDuration::get().into();
			queues
				.insert_in_available_queue(nft_id, expiration_block)
				.expect("We already checked for this. qed");
			Queues::<T>::set(queues);

			// Contract Created 📦
			let contract = RentContractData::new(
				false,
				None,
				who.clone(),
				None,
				duration.clone(),
				acceptance_type.clone(),
				revocation_type.clone(),
				rent_fee.clone(),
				false,
				renter_cancellation_fee.clone(),
				rentee_cancellation_fee.clone(),
			);
			Contracts::<T>::insert(nft_id, contract);

			// NFT Updated 📦
			nft.state.is_rented = true;
			T::NFTExt::set_nft(nft_id, nft)?;

			// Deposit event.
			let event = Event::ContractCreated {
				nft_id,
				renter: who,
				duration,
				acceptance_type,
				revocation_type,
				rent_fee,
				renter_cancellation_fee,
				rentee_cancellation_fee,
			};
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Revoke a rent contract, cancel it if it has not started.
		#[pallet::weight((
			{
				let s = Contracts::<T>::get(nft_id)
					.map_or_else(|| 0, |c| {
						let mut queues = Queues::<T>::get();
						if !c.has_started {
							queues.available_queue.size()
						} else {
							match c.duration {
								Duration::Subscription(_, _) => queues.subscription_queue.size(),
								Duration::Fixed(_) => queues.fixed_queue.size(),
							}
						}
					});
				T::WeightInfo::revoke_contract(s as u32)
			},
			DispatchClass::Normal
		))]
		pub fn revoke_contract(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;

			ensure!(
				contract.renter == who || contract.rentee == Some(who.clone()),
				Error::<T>::NotTheRenterOrRentee
			);
			ensure!(
				!(contract.renter == who &&
					contract.has_started && contract.revocation_type == RevocationType::NoRevocation),
				Error::<T>::CannotRevoke
			);

			// Apply cancel_fees transfers.
			Self::process_cancellation_fees(nft_id, &contract, Some(who.clone()))?;

			// Remove from corresponding queues / mappings.
			let mut queues = Queues::<T>::get();
			queues.remove_from_queue(nft_id, contract.has_started, &contract.duration);
			Queues::<T>::set(queues);

			// Remove offers
			if !contract.has_started {
				Offers::<T>::remove(nft_id);
			}

			// Set NFT state back.
			nft.state.is_rented = false;
			T::NFTExt::set_nft(nft_id, nft)?;

			// Remove contract.
			Contracts::<T>::remove(nft_id);

			// Deposit event.
			let event = Event::ContractRevoked { nft_id, revoked_by: who };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Rent an nft if contract exist, makes an offer if it's manual acceptance.
		#[pallet::weight((
			{
				let (s, t) = Contracts::<T>::get(nft_id)
					.map_or_else(|| (0, 0), |c| {
						let mut queues = Queues::<T>::get();
						let available_size = queues.available_queue.size();
						match c.acceptance_type {
							AcceptanceType::AutoAcceptance(_) => {
								match c.duration {
									Duration::Subscription(_, _) => (available_size, queues.subscription_queue.size()),
									Duration::Fixed(_) =>  (available_size, queues.fixed_queue.size()),
								}
							},
							AcceptanceType::ManualAcceptance(_) => (0, 0)
						}
					});
				T::WeightInfo::rent(s as u32, t as u32)
			},
			DispatchClass::Normal
		))]
		pub fn rent(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let pallet = Self::account_id();
			let now = frame_system::Pallet::<T>::block_number();

			Contracts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;

				ensure!(contract.renter != who, Error::<T>::CannotRentOwnContract);
				ensure!(!contract.is_manual_acceptance(), Error::<T>::CannotRentOwnContract);

				let rent_fee = contract.rent_fee.clone();
				let cancellation_fee = contract.rentee_cancellation_fee.clone();

				// Let's see if he is on the allowed list
				if let Some(list) = contract.acceptance_type.get_allow_list() {
					ensure!(list.contains(&who), Error::<T>::NotAuthorizedForRent);
				}

				// Balance Check  ✅
				if let Some(amount) = rent_fee.get_balance() {
					T::Currency::transfer(&who, &contract.renter, amount, KeepAlive)?;
				}

				// Rent and Renter Cancellation fee Check  ✅
				let rent_nft =
					contract.rent_fee.get_nft().and_then(|x| Some((x, contract.renter.clone())));
				let cancellation_nft = cancellation_fee
					.and_then(|x| x.get_nft())
					.and_then(|x| Some((x, pallet.clone())));
				let nfts: Vec<(NFTId, T::AccountId)> =
					vec![rent_nft, cancellation_nft].into_iter().flatten().collect();

				// Rent and Renter Cancellation fee Check  ✅
				for (nft_id, _) in &nfts {
					let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::NFTNotFound)?;
					ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
					Self::check_nft_state_validity(&nft)?;
				}

				// Rent and Renter Cancellation fee Taken 📦
				for (nft_id, dst) in &nfts {
					let mut nft = T::NFTExt::get_nft(*nft_id).expect("qed");
					nft.owner = dst.clone();
					T::NFTExt::set_nft(*nft_id, nft).expect("qed");
				}

				// Queue and Offers updated 📦
				let mut queues = Queues::<T>::get();
				queues.available_queue.remove(nft_id);

				match &contract.duration {
					Duration::Fixed(x) => queues.fixed_queue.insert(nft_id, x.clone()),
					Duration::Subscription(x, _) =>
						queues.subscription_queue.insert(nft_id, now + *x),
				}
				.expect("qed");

				Queues::<T>::set(queues);
				Offers::<T>::remove(nft_id);

				contract.rentee = Some(who.clone());
				contract.start_block = Some(now);

				Ok(())
			})?;

			let event = Event::ContractStarted { nft_id, rentee: who };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Rent an nft if contract exist, makes an offer if it's manual acceptance.
		#[pallet::weight(1)]
		pub fn make_rent_offer(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;

			ensure!(contract.renter != who, Error::<T>::CannotRentOwnContract);
			ensure!(contract.is_manual_acceptance(), Error::<T>::CannotRentOwnContract);

			let rent_fee = contract.rent_fee.clone();
			let cancellation_fee = contract.rentee_cancellation_fee.clone();

			// Let's see if he is on the allowed list
			if let Some(list) = contract.acceptance_type.get_allow_list() {
				ensure!(list.contains(&who), Error::<T>::NotAuthorizedForRent);
			}

			// Balance Check  ✅
			if let Some(amount) = rent_fee.get_balance() {
				Self::balance_check(&who, amount).ok_or(Error::<T>::NFTNotFound)?;
			}

			// Rent and Renter Cancellation fee Check  ✅
			let nft_ids = vec![rent_fee.get_nft(), cancellation_fee.and_then(|x| x.get_nft())];
			let nft_ids = nft_ids.iter().flatten();
			for nft_id in nft_ids {
				let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::NFTNotFound)?;
				ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
				Self::check_nft_state_validity(&nft)?;
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

			// Deposit event.
			let event = Event::ContractOfferCreated { nft_id, rentee: who };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Accept a rent offer for manual acceptance contract.
		#[pallet::weight((
			{
				let (s, t, u) = Contracts::<T>::get(nft_id)
					.map_or_else(|| (0, 0, 0), |c| {
						match c.acceptance_type {
							AcceptanceType::ManualAcceptance(_) => {
								let mut queues = Queues::<T>::get();
								let available_size = queues.available_queue.size();
								let offers_size = Offers::<T>::get(nft_id)
									.map_or_else(|| 0, |o| {
										o.len()
									});
								match c.duration {
									Duration::Subscription(_, _) => (available_size, queues.subscription_queue.size(), offers_size),
									Duration::Fixed(_) => (available_size, queues.fixed_queue.size(), offers_size),
								}
							},
							AcceptanceType::AutoAcceptance(_) => (0,0,0)
						}

					});
				T::WeightInfo::accept_rent_offer(s as u32, t as u32, u as u32)
			},
			DispatchClass::Normal
		))]
		pub fn accept_rent_offer(
			origin: OriginFor<T>,
			nft_id: NFTId,
			rentee: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			Contracts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;
				let cancellation_fee = contract.rentee_cancellation_fee.clone();
				let pallet = Self::account_id();

				ensure!(contract.renter == who, Error::<T>::NotTheRenter);
				let offers = Offers::<T>::get(nft_id).ok_or(Error::<T>::NoOffersForThisContract)?;
				let offer_found = offers.contains(&rentee);
				ensure!(offer_found, Error::<T>::NoOffersForThisContract);

				// Let's take rentee's token. In case an error happens those balance transactions
				// will be reverted.
				if let Some(amount) = contract.rent_fee.get_balance() {
					T::Currency::transfer(&rentee, &who, amount, KeepAlive)?;
				}

				if let Some(amount) = cancellation_fee.clone().and_then(|x| x.get_balance()) {
					T::Currency::transfer(&rentee, &pallet, amount, KeepAlive)?;
				}

				// Let's see if those NFTs are OK to be taken.
				let rent_nft =
					contract.rent_fee.get_nft().and_then(|x| Some((x, contract.renter.clone())));
				let cancellation_nft = cancellation_fee
					.and_then(|x| x.get_nft())
					.and_then(|x| Some((x, pallet.clone())));
				let nfts: Vec<(NFTId, T::AccountId)> =
					vec![rent_nft, cancellation_nft].into_iter().flatten().collect();

				// Rent and Renter Cancellation fee Check  ✅
				for (nft_id, _) in &nfts {
					let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::NFTNotFound)?;
					ensure!(nft.owner == rentee, Error::<T>::NotTheNFTOwner);
					Self::check_nft_state_validity(&nft)?;
				}

				// Rent and Renter Cancellation fee Taken 📦
				for (nft_id, dst) in &nfts {
					let mut nft = T::NFTExt::get_nft(*nft_id).expect("qed");
					nft.owner = dst.clone();
					T::NFTExt::set_nft(*nft_id, nft).expect("qed");
				}

				// All good ☀️
				// Queue and Offers updated 📦
				let mut queues = Queues::<T>::get();
				queues.available_queue.remove(nft_id);

				match &contract.duration {
					Duration::Fixed(x) => queues.fixed_queue.insert(nft_id, x.clone()),
					Duration::Subscription(x, _) =>
						queues.subscription_queue.insert(nft_id, now + *x),
				}
				.expect("qed");

				Queues::<T>::set(queues);
				Offers::<T>::remove(nft_id);

				contract.rentee = Some(rentee.clone());
				contract.start_block = Some(now);

				Ok(())
			})?;

			// Deposit event.
			let event = Event::ContractStarted { nft_id, rentee };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Retract a rent offer for manual acceptance contract.
		#[pallet::weight((
			{
				let s = Offers::<T>::get(nft_id)
					.map_or_else(|| 0, |o| o.len());
				T::WeightInfo::retract_rent_offer(s as u32)
			},
			DispatchClass::Normal
		))]
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

			// Deposit event.
			let event = Event::ContractOfferRetracted { nft_id, rentee: who };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Change the subscription terms for subscription contracts.
		#[pallet::weight(T::WeightInfo::change_subscription_terms())]
		pub fn change_subscription_terms(
			origin: OriginFor<T>,
			nft_id: NFTId,
			period: T::BlockNumber,
			max_duration: Option<T::BlockNumber>,
			rent_fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Contracts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;

				ensure!(who == contract.renter, Error::<T>::NotTheRenter);
				ensure!(
					contract.can_adjust_subscription(),
					Error::<T>::CannotAdjustSubscriptionTerms
				);

				contract.duration = Duration::Subscription(period, max_duration);
				contract.rent_fee = RentFee::Tokens(rent_fee);
				contract.terms_changed = contract.rentee.is_some();

				Ok(())
			})?;

			// Deposit event.
			let event =
				Event::ContractSubscriptionTermsChanged { nft_id, period, max_duration, rent_fee };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Accept the new contract terms.
		#[pallet::weight(T::WeightInfo::accept_subscription_terms())]
		pub fn accept_subscription_terms(
			origin: OriginFor<T>,
			nft_id: NFTId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Contracts::<T>::try_mutate(nft_id, |x| -> DispatchResult {
				let contract = x.as_mut().ok_or(Error::<T>::ContractNotFound)?;

				ensure!(Some(who) == contract.rentee, Error::<T>::NotTheRentee);
				ensure!(!contract.terms_changed, Error::<T>::ContractTermsAlreadyAccepted);

				contract.terms_changed = false;

				Ok(())
			})?;

			// Deposit event.
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

	/// Check that an NFT is available for rent.
	pub fn check_nft_state_validity(
		nft: &NFTData<T::AccountId, <T::NFTExt as NFTExt>::NFTOffchainDataLimit>,
	) -> Result<(), pallet::Error<T>> {
		nft.not_in_state(vec![Capsule, IsListed, Delegated, Soulbound, Rented])
			.map_err(|err| {
				return match err {
					Capsule => Error::<T>::CannotUseCapsuleNFTs,
					IsListed => Error::<T>::CannotUseListedNFTs,
					Delegated => Error::<T>::CannotUseDelegatedNFTs,
					Soulbound => Error::<T>::CannotUseSoulboundNFTs,
					Rented => Error::<T>::CannotUseRentedNFTs,
					_ => panic!("This should never happen"),
				}
			})
	}

	/// Get contract total blocks.
	pub fn get_contract_total_blocks(duration: &Duration<T::BlockNumber>) -> T::BlockNumber {
		match duration {
			Duration::Fixed(value) => *value,
			Duration::Subscription(value, max_value) =>
				if let Some(max_value) = *max_value {
					max_value
				} else {
					*value
				},
		}
	}

	/// Get contract end block.
	pub fn get_contract_end_block(
		nft_id: NFTId,
		duration: &Duration<T::BlockNumber>,
		queues: &mut RentingQueues<T::BlockNumber, T::SimultaneousContractLimit>,
	) -> Option<T::BlockNumber> {
		match duration {
			Duration::Fixed(_) => queues.fixed_queue.get(nft_id),
			Duration::Subscription(_, _) => queues.subscription_queue.get(nft_id),
		}
	}

	/// Return all the cancellation fee to recipient without any calculation.
	pub fn return_full_cancellation_fee(
		to: T::AccountId,
		cancellation_fee: &Option<CancellationFee<BalanceOf<T>>>,
	) -> Result<(), DispatchError> {
		let cancellation_fee = match cancellation_fee {
			Some(x) => x,
			None => return Ok(()),
		};

		if let Some(amount) = cancellation_fee.get_balance() {
			T::Currency::transfer(&Self::account_id(), &to, amount, AllowDeath)?;
		}

		if let Some(nft_id) = cancellation_fee.get_nft() {
			let mut cancellation_nft =
				T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
			ensure!(
				cancellation_nft.owner == Self::account_id(),
				Error::<T>::NotTheNFTOwnerForCancellationFee
			);
			cancellation_nft.owner = to.clone();
			T::NFTExt::set_nft(nft_id, cancellation_nft)?;
		}
		Ok(())
	}

	/// Pay the cancellation fee to renter or rentee impacted by revoker's cancellation.
	pub fn pay_cancellation_fee(
		from: &T::AccountId,
		to: T::AccountId,
		cancellation_fee: &Option<CancellationFee<BalanceOf<T>>>,
		nft_id: NFTId,
		duration: &Duration<T::BlockNumber>,
	) -> Result<(), DispatchError> {
		if let Some(cancellation_fee) = cancellation_fee {
			match cancellation_fee {
				CancellationFee::NFT(cancellation_nft_id) => {
					let mut cancellation_nft = T::NFTExt::get_nft(*cancellation_nft_id)
						.ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
					ensure!(
						cancellation_nft.owner == Self::account_id(),
						Error::<T>::NotTheNFTOwnerForCancellationFee
					);
					cancellation_nft.owner = to;
					T::NFTExt::set_nft(*cancellation_nft_id, cancellation_nft)?;
				},
				CancellationFee::FixedTokens(amount) => {
					T::Currency::transfer(&Self::account_id(), &to, *amount, AllowDeath)?;
				},
				CancellationFee::FlexibleTokens(amount) => {
					let mut queues = Queues::<T>::get();
					let end_block = Self::get_contract_end_block(nft_id, &duration, &mut queues);
					let total_blocks = Self::get_contract_total_blocks(&duration);
					ensure!(end_block.is_some(), Error::<T>::FlexibleFeeEndBlockNotFound);
					if let Some(end_block) = end_block {
						let current_block: u128 =
							<frame_system::Pallet<T>>::block_number().saturated_into();
						let end_block: u128 = end_block.saturated_into();
						let remaining_blocks: u128 = end_block
							.checked_sub(current_block)
							.ok_or(Error::<T>::InternalMathError)?;
						let total_blocks: u128 = total_blocks.saturated_into();
						let taken = amount
							.saturating_mul(remaining_blocks.saturated_into::<BalanceOf<T>>())
							.checked_div(&total_blocks.saturated_into::<BalanceOf<T>>())
							.ok_or(Error::<T>::InternalMathError)?;
						let returned =
							amount.checked_sub(&taken).ok_or(Error::<T>::InternalMathError)?;

						T::Currency::transfer(&Self::account_id(), &to, taken, AllowDeath)?;
						T::Currency::transfer(&Self::account_id(), &from, returned, AllowDeath)?;
					}
				},
			};
		}
		Ok(())
	}

	/// Give back the cancellation fees depending on revoker and revocation timing.
	pub fn process_cancellation_fees(
		nft_id: NFTId,
		contract: &RentContractData<
			T::AccountId,
			T::BlockNumber,
			BalanceOf<T>,
			T::AccountSizeLimit,
		>,
		revoker: Option<T::AccountId>,
	) -> Result<(), DispatchError> {
		if !contract.has_started {
			Self::return_full_cancellation_fee(
				contract.renter.clone(),
				&contract.renter_cancellation_fee,
			)?;

			return Ok(())
		}

		if let Some(revoker) = revoker {
			if revoker == contract.renter {
				if let Some(rentee) = &contract.rentee {
					// Revoked by renter, rentee receive renter's cancellation fee.
					Self::pay_cancellation_fee(
						&contract.renter,
						rentee.clone(),
						&contract.renter_cancellation_fee,
						nft_id,
						&contract.duration,
					)?;

					// Rentee gets back his cancellation fee.
					Self::return_full_cancellation_fee(
						rentee.clone(),
						&contract.rentee_cancellation_fee,
					)?;
				};
			} else if let Some(rentee) = &contract.rentee {
				if revoker == *rentee {
					// Revoked by rentee or Subscription payment stopped, renters receive
					// rentees's cancellation fee.
					Self::pay_cancellation_fee(
						&rentee,
						contract.renter.clone(),
						&contract.rentee_cancellation_fee,
						nft_id,
						&contract.duration,
					)?;

					// Renter gets back his cancellation fee.
					Self::return_full_cancellation_fee(
						contract.renter.clone(),
						&contract.renter_cancellation_fee,
					)?;
				}
			}
		} else {
			// Revoked cause contract ended (fixed, or maxSubscription reached).
			Self::return_full_cancellation_fee(
				contract.renter.clone(),
				&contract.renter_cancellation_fee,
			)?;
			if let Some(rentee) = &contract.rentee {
				Self::return_full_cancellation_fee(
					rentee.clone(),
					&contract.rentee_cancellation_fee,
				)?;
			};
		};
		Ok(())
	}

	pub fn return_cancellation_fee(
		fee: &CancellationFee<BalanceOf<T>>,
		dst: &T::AccountId,
	) -> Result<(), DispatchError> {
		if let Some(amount) = fee.get_balance() {
			let src = &Self::account_id();
			T::Currency::transfer(src, dst, amount, AllowDeath)?
		}

		if let Some(nft_id) = fee.get_nft() {
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			nft.owner = dst.clone();
			T::NFTExt::set_nft(nft_id, nft)?;
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

	pub fn handle_finished_or_unused_contract(nft_id: NFTId) {
		let contract = Contracts::<T>::get(nft_id).expect("Should not happen. qed");
		let mut nft = T::NFTExt::get_nft(nft_id).expect("Should not happen. qed");

		// Return Cancellation fees
		if let Some(fee) = &contract.renter_cancellation_fee {
			Self::return_cancellation_fee(fee, &contract.renter).expect("This cannot happen. qed");
		}

		if let Some(rentee) = &contract.rentee {
			if let Some(fee) = &contract.rentee_cancellation_fee {
				Self::return_cancellation_fee(fee, rentee).expect("This cannot happen. qed");
			}
		}

		nft.state.is_rented = false;
		T::NFTExt::set_nft(nft_id, nft).expect("This cannot happen. qed");
		Contracts::<T>::remove(nft_id);
	}

	pub fn balance_check(account: &T::AccountId, amount: BalanceOf<T>) -> Option<()> {
		let current_balance = T::Currency::free_balance(account);
		let new_balance = current_balance.checked_sub(&amount)?;
		T::Currency::ensure_can_withdraw(&account, amount, WithdrawReasons::FEE, new_balance).ok()
	}
}

impl<T: Config> Pallet<T> {
	/// Fill available queue with any number of data.
	pub fn fill_available_queue(
		number: u32,
		nft_id: NFTId,
		block_number: T::BlockNumber,
	) -> Result<(), DispatchError> {
		let mut queues = Queues::<T>::get();
		for _i in 0..number {
			queues
				.available_queue
				.insert(nft_id, block_number)
				.map_err(|_| Error::<T>::MaxSimultaneousContractReached)?;
		}
		Queues::<T>::set(queues);
		Ok(())
	}

	/// Fill fixed queue with any number of data.
	pub fn fill_fixed_queue(
		number: u32,
		nft_id: NFTId,
		block_number: T::BlockNumber,
	) -> Result<(), DispatchError> {
		let mut queues = Queues::<T>::get();
		for _i in 0..number {
			queues
				.fixed_queue
				.insert(nft_id, block_number)
				.map_err(|_| Error::<T>::MaxSimultaneousContractReached)?;
		}
		Queues::<T>::set(queues);
		Ok(())
	}

	/// Fill subscription queue with any number of data.
	pub fn fill_subscription_queue(
		number: u32,
		nft_id: NFTId,
		block_number: T::BlockNumber,
	) -> Result<(), DispatchError> {
		let mut queues = Queues::<T>::get();
		for _i in 0..number {
			queues
				.subscription_queue
				.insert(nft_id, block_number)
				.map_err(|_| Error::<T>::MaxSimultaneousContractReached)?;
		}
		Queues::<T>::set(queues);
		Ok(())
	}

	/// Fill offers vector with any number of data.
	pub fn fill_offers_vector(
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
