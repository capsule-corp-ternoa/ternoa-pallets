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
// #[cfg(test)]
// mod tests;
mod types;
mod weights;

pub use pallet::*;
pub use types::*;

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	pallet_prelude::DispatchResultWithPostInfo,
	traits::{Currency, ExistenceRequirement::KeepAlive, Get, StorageVersion},
	transactional, BoundedVec, PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::AccountIdConversion; /* , CheckedDiv, CheckedSub, Saturating */
use sp_std::prelude::*;

use primitives::nfts::{NFTData, NFTId};
use ternoa_common::traits::NFTExt;
pub use weights::WeightInfo;

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NegativeImbalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

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
		type SimultaneousContractLimit: Get<u32>; //TODO use to limit contracts mapping

		// /// Maximum number of simultaneous fixed rent contract.
		// #[pallet::constant]
		// type FixedQueueLimit: Get<u32>;

		// /// Maximum number of simultaneous subscription rent contract.
		// #[pallet::constant]
		// type SubscriptionQueueLimit: Get<u32>;

		// /// Maximum number of simultaneous rent contract waiting for acceptance.
		// #[pallet::constant]
		// type AvailableQueueLimit: Get<u32>;

		/// Maximum number of related automatic rent actions in block.
		#[pallet::constant]
		type ActionsInBlockLimit: Get<u32>;

		/// Maximum number of blocks during which a rent contract is available for acceptance.
		#[pallet::constant]
		type ContractExpirationDuration: Get<u32>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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

	/// Data related to fixed contract deadlines.
	#[pallet::storage]
	#[pallet::getter(fn fixed_queue)]
	pub type FixedQueue<T: Config> = StorageValue<_, Queue<T::BlockNumber, T::SimultaneousContractLimit>, ValueQuery>;

	/// Data related to subscription contract deadlines.
	#[pallet::storage]
	#[pallet::getter(fn subscription_queue)]
	pub type SubscriptionQueue<T: Config> =
		StorageValue<_, Queue<T::BlockNumber, T::SimultaneousContractLimit>, ValueQuery>;

	/// Data related to available for rent contract deadlines.
	#[pallet::storage]
	#[pallet::getter(fn available_queue)]
	pub type AvailableQueue<T: Config> =
		StorageValue<_, Queue<T::BlockNumber, T::SimultaneousContractLimit>, ValueQuery>;

	/// Data related to rent contracts offers.
	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> =
		StorageMap<_, Blake2_128Concat, NFTId, AccountList<T::AccountId, T::AccountSizeLimit>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Contract Created.
		ContractCreated {
			nft_id: NFTId,
			renter: T::AccountId,
			duration: Duration<T::BlockNumber>,
			acceptance_type: AcceptanceType<AccountList<T::AccountId, T::AccountSizeLimit>>,
			revocation_type: RevocationType<BalanceOf<T>, T::BlockNumber>,
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
			duration: Duration<T::BlockNumber>,
			rent_fee: RentFee<BalanceOf<T>>,
		},
		/// A contract new subscription's terms were accpeted by rentee.
		ContractSubscriptionTermsAccepted { nft_id: NFTId },
		/// A contract has ended.
		ContractEnded { nft_id: NFTId },
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
		/// Operation is not permitted because NFT is auctioned.
		CannotUseAuctionedNFTs,
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
		/// Cannot create a contract with infinite duration and flexible tokens cancellation fee.
		NoInfiniteWithFlexibleFee,
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
		/// No offers were found.
		NoOffersForThisContract,
		/// Addresss not found in offers.
		AddressNotFoundInOffers,
		/// Terms can only be set for subscription duration.
		CanChangeTermForSubscriptionOnly,
		/// New term must be suvscription.
		CanSetTermsForSubscriptionOnly,
		/// Operation is not permitted because revocation type is not OnSubscriptionChange.
		CanChangeTermForOnSubscriptionChangeOnly,
		/// Operation is not permitted because contract terms are already accepted
		ContractTermsAlreadyAccepted,
		/// Math error.
		InternalMathError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new rent contract with the provided details.
		#[pallet::weight(T::WeightInfo::transfer_nft())]
		#[transactional]
		pub fn create_contract(
			origin: OriginFor<T>,
			nft_id: NFTId,
			duration: Duration<T::BlockNumber>,
			acceptance_type: AcceptanceType<AccountList<T::AccountId, T::AccountSizeLimit>>,
			revocation_type: RevocationType<BalanceOf<T>, T::BlockNumber>,
			rent_fee: RentFee<BalanceOf<T>>,
			renter_cancellation_fee: Option<CancellationFee<BalanceOf<T>>>,
			rentee_cancellation_fee: Option<CancellationFee<BalanceOf<T>>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check contract parameters.
			Self::check_contract_parameters(
				&duration,
				&revocation_type,
				&rent_fee,
				&renter_cancellation_fee,
				&rentee_cancellation_fee,
			)?;

			// Check nft data.
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			Self::ensure_nft_available(&nft)?;

			Self::ensure_enough_for_cancellation_fee(&renter_cancellation_fee, &who)?;

			// Take cancellation fee for renter if it exist.
			Self::take_cancellation_fee(&who, &renter_cancellation_fee)?;

			// Insert in available queue with expiration.
			Self::insert_in_available_queue(nft_id)?;

			// Insert new contract.
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

			// Set NFT state.
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
		#[pallet::weight(T::WeightInfo::transfer_nft())]
		#[transactional]
		pub fn revoke_contract(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;

			ensure!(contract.renter == who || contract.rentee == Some(who.clone()), Error::<T>::NotTheRenterOrRentee);
			ensure!(
				!(contract.renter == who &&
					contract.has_started && contract.revocation_type == RevocationType::Anytime),
				Error::<T>::CannotRevoke
			);

			// Apply cancel_fees transfers.
			Self::process_cancellation_fees(nft_id, &contract, Some(&who))?;

			// Remove from corresponding queues / mappings.
			Self::remove_from_queues(nft_id, &contract)?;

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
		#[pallet::weight(T::WeightInfo::transfer_nft())]
		#[transactional]
		pub fn rent(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;
			ensure!(contract.renter != who, Error::<T>::CannotRentOwnContract);

			Self::ensure_enough_for_rent(&who, &contract.rent_fee, &contract.rentee_cancellation_fee)?;

			match contract.acceptance_type.clone() {
				AcceptanceType::ManualAcceptance(accounts) => {
					if let Some(accounts) = accounts {
						ensure!(accounts.contains(&who), Error::<T>::NotAuthorizedForRent);
					}

					Self::insert_offer(nft_id, who.clone())?;

					// Deposit event.
					let event = Event::ContractOfferCreated { nft_id, rentee: who };
					Self::deposit_event(event);
				},
				AcceptanceType::AutoAcceptance(accounts) => {
					if let Some(accounts) = accounts {
						ensure!(accounts.contains(&who), Error::<T>::NotAuthorizedForRent);
					}

					Self::take_rent_fee(&who, contract.renter.clone(), &contract.rent_fee)?;
					Self::take_cancellation_fee(&who, &contract.rentee_cancellation_fee)?;
					Self::insert_in_queue(nft_id, &contract.duration)?;

					contract.terms_accepted = true;
					contract.rentee = Some(who.clone());
					contract.has_started = true;
					contract.start_block = Some(<frame_system::Pallet<T>>::block_number());

					Contracts::<T>::insert(nft_id, contract);

					// Deposit event.
					let event = Event::ContractStarted { nft_id, rentee: who };
					Self::deposit_event(event);
				},
			};

			Ok(().into())
		}

		/// Accept a rent offer for manual acceptance contract.
		#[pallet::weight(T::WeightInfo::transfer_nft())]
		#[transactional]
		pub fn accept_rent_offer(
			origin: OriginFor<T>,
			nft_id: NFTId,
			rentee: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;
			ensure!(contract.renter == who, Error::<T>::NotTheRenter);
			let is_manual_acceptance = matches!(contract.acceptance_type, AcceptanceType::ManualAcceptance { .. });
			ensure!(is_manual_acceptance, Error::<T>::CannotAcceptOfferForAutoAcceptance);
			match contract.acceptance_type.clone() {
				AcceptanceType::AutoAcceptance(_) => (),
				AcceptanceType::ManualAcceptance(accounts) =>
					if let Some(accounts) = accounts {
						ensure!(accounts.contains(&who), Error::<T>::NotAuthorizedForRent);
					},
			}

			Self::ensure_enough_for_rent(&rentee, &contract.rent_fee, &contract.rentee_cancellation_fee)?;
			Self::take_rent_fee(&rentee, contract.renter.clone(), &contract.rent_fee)?;
			Self::take_cancellation_fee(&rentee, &contract.rentee_cancellation_fee)?;
			Self::insert_in_queue(nft_id, &contract.duration)?;

			contract.terms_accepted = true;
			contract.rentee = Some(rentee.clone());
			contract.has_started = true;
			contract.start_block = Some(<frame_system::Pallet<T>>::block_number());

			Self::remove_from_available_queue(nft_id)?;
			Contracts::<T>::insert(nft_id, contract);
			Offers::<T>::remove(nft_id);

			// Deposit event.
			let event = Event::ContractStarted { nft_id, rentee: who };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Retract a rent offer for manual acceptance contract.
		#[pallet::weight(T::WeightInfo::transfer_nft())]
		pub fn retract_rent_offer(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;
			let is_manual_acceptance = matches!(contract.acceptance_type, AcceptanceType::ManualAcceptance { .. });
			ensure!(is_manual_acceptance, Error::<T>::CannotRetractOfferForAutoAcceptance);
			Self::remove_offer(nft_id, who.clone())?;

			// Deposit event.
			let event = Event::ContractOfferRetracted { nft_id, rentee: who };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Change the subscription terms for subscription contracts.
		#[pallet::weight(T::WeightInfo::transfer_nft())]
		pub fn change_subscription_terms(
			origin: OriginFor<T>,
			nft_id: NFTId,
			duration: Duration<T::BlockNumber>,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;

			ensure!(who == contract.renter, Error::<T>::NotTheRenter);
			let is_on_subscription_change = matches!(contract.revocation_type, RevocationType::OnSubscriptionChange {..});
			ensure!(is_on_subscription_change, Error::<T>::CanChangeTermForOnSubscriptionChangeOnly);
			let is_subscription = matches!(contract.duration, Duration::Subscription { .. });
			ensure!(is_subscription, Error::<T>::CanChangeTermForSubscriptionOnly);
			let is_new_term_subscription = matches!(duration, Duration::Subscription { .. });
			ensure!(is_new_term_subscription, Error::<T>::CanSetTermsForSubscriptionOnly);

			contract.duration = duration.clone();
			contract.rent_fee = RentFee::Tokens(amount);
			contract.terms_accepted = false;

			Contracts::<T>::insert(nft_id, contract);

			// Deposit event.
			let event = Event::ContractSubscriptionTermsChanged { nft_id, duration, rent_fee: RentFee::Tokens(amount) };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Accept the new contract terms.
		#[pallet::weight(T::WeightInfo::transfer_nft())]
		pub fn accept_subscription_terms(
			origin: OriginFor<T>,
			nft_id: NFTId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;

			ensure!(Some(who) == contract.rentee, Error::<T>::NotTheRentee);
			ensure!(!contract.terms_accepted, Error::<T>::ContractTermsAlreadyAccepted);

			contract.terms_accepted = true;

			Contracts::<T>::insert(nft_id, contract);

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
		T::PalletId::get().into_account()
	}

	/// Check that an NFT is available for rent.
	pub fn ensure_nft_available(
		nft: &NFTData<T::AccountId, <T::NFTExt as NFTExt>::NFTOffchainDataLimit>,
	) -> Result<(), DispatchError> {
		ensure!(!nft.state.is_listed, Error::<T>::CannotUseListedNFTs);
		ensure!(!nft.state.is_capsule, Error::<T>::CannotUseCapsuleNFTs);
		ensure!(!nft.state.is_delegated, Error::<T>::CannotUseDelegatedNFTs);
		ensure!(!nft.state.is_soulbound, Error::<T>::CannotUseSoulboundNFTs);
		ensure!(!nft.state.is_auctioned, Error::<T>::CannotUseAuctionedNFTs);
		ensure!(!nft.state.is_rented, Error::<T>::CannotUseRentedNFTs);
		Ok(())
	}

	/// Check that address has NFT / Balance to cover cancellation fee.
	pub fn ensure_enough_for_cancellation_fee(
		cancellation_fee: &Option<CancellationFee<BalanceOf<T>>>,
		from: &T::AccountId,
	) -> Result<(), DispatchError> {
		if let Some(cancellation_fee) = cancellation_fee {
			match cancellation_fee {
				CancellationFee::FixedTokens(amount) => {
					ensure!(T::Currency::free_balance(from) > *amount, Error::<T>::NotEnoughBalanceForCancellationFee);
				},
				CancellationFee::FlexibleTokens(amount) => {
					ensure!(T::Currency::free_balance(from) > *amount, Error::<T>::NotEnoughBalanceForCancellationFee);
				},
				CancellationFee::NFT(nft_id) => {
					let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
					ensure!(nft.owner == *from, Error::<T>::NotTheNFTOwnerForCancellationFee);
					Self::ensure_nft_available(&nft)?;
				},
			}
		}
		Ok(())
	}

	/// Check that address has NFT / Balance to cover first rent fee.
	pub fn ensure_enough_for_rent_fee(
		rent_fee: &RentFee<BalanceOf<T>>,
		from: &T::AccountId,
	) -> Result<(), DispatchError> {
		match rent_fee {
			RentFee::Tokens(amount) => {
				ensure!(T::Currency::free_balance(from) > *amount, Error::<T>::NotEnoughBalanceForRentFee);
			},
			RentFee::NFT(nft_id) => {
				let nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
				ensure!(nft.owner == *from, Error::<T>::NotTheNFTOwnerForCancellationFee);
				Self::ensure_nft_available(&nft)?;
			},
		}
		Ok(())
	}

	/// Check that address has NFTs / Balance to cover cancellation fee and first rent fee.
	pub fn ensure_enough_for_rent(
		from: &T::AccountId,
		rent_fee: &RentFee<BalanceOf<T>>,
		cancellation_fee: &Option<CancellationFee<BalanceOf<T>>>,
	) -> Result<(), DispatchError> {
		Self::ensure_enough_for_cancellation_fee(cancellation_fee, from)?;
		Self::ensure_enough_for_rent_fee(rent_fee, from)?;

		if let Some(cancellation_fee) = cancellation_fee {
			let is_cancellation_fee_balance = !matches!(*cancellation_fee, CancellationFee::NFT { .. });
			let is_rent_fee_balance = matches!(*rent_fee, RentFee::Tokens { .. });
			if is_rent_fee_balance && is_cancellation_fee_balance {
				let mut total: BalanceOf<T> = 0u32.into();
				match rent_fee {
					RentFee::Tokens(amount) => total = total + *amount,
					RentFee::NFT(_) => (),
				}
				match cancellation_fee {
					CancellationFee::FixedTokens(amount) => total = total + *amount,
					CancellationFee::FlexibleTokens(amount) => total = total + *amount,
					CancellationFee::NFT(_) => (),
				}
				ensure!(T::Currency::free_balance(from) > total, Error::<T>::NotEnoughBalance);
			}
		}

		Ok(())
	}

	/// Check contract parameters.
	pub fn check_contract_parameters(
		duration: &Duration<T::BlockNumber>,
		revocation_type: &RevocationType<BalanceOf<T>, T::BlockNumber>,
		rent_fee: &RentFee<BalanceOf<T>>,
		renter_cancellation_fee: &Option<CancellationFee<BalanceOf<T>>>,
		rentee_cancellation_fee: &Option<CancellationFee<BalanceOf<T>>>,
	) -> Result<(), DispatchError> {
		// Checks
		let is_subscription = matches!(*duration, Duration::Subscription { .. });
		let is_on_subscription_change = matches!(*revocation_type, RevocationType::OnSubscriptionChange { .. });
		let is_nft_rent_fee = matches!(rent_fee, RentFee::NFT { .. });
		let is_flexible_token_renter = matches!(*renter_cancellation_fee, Some(CancellationFee::FlexibleTokens { .. }));
		let is_flexible_token_rentee = matches!(*rentee_cancellation_fee, Some(CancellationFee::FlexibleTokens { .. }));
		ensure!(!(is_on_subscription_change && !is_subscription), Error::<T>::SubscriptionChangeForSubscriptionOnly);
		ensure!(!(is_nft_rent_fee && is_subscription), Error::<T>::NoNFTRentFeeWithSubscription);
		ensure!(
			!(*revocation_type == RevocationType::NoRevocation && renter_cancellation_fee.is_some()),
			Error::<T>::NoRenterCancellationFeeWithNoRevocation
		);
		ensure!(
			!(*duration == Duration::Infinite && (is_flexible_token_renter || is_flexible_token_rentee)),
			Error::<T>::NoInfiniteWithFlexibleFee
		);
		Ok(())
	}

	/// Set contract as available for rent.
	pub fn insert_in_available_queue(nft_id: NFTId) -> Result<(), DispatchError> {
		let current_block_number = <frame_system::Pallet<T>>::block_number();
		let expiration_duration = T::ContractExpirationDuration::get();
		let expiration_block = current_block_number + expiration_duration.into();
		AvailableQueue::<T>::mutate(|x| -> DispatchResult {
			x.insert(nft_id, expiration_block)
				.map_err(|_| Error::<T>::MaxSimultaneousContractReached)?;
			Ok(())
		})?;
		Ok(())
	}

	/// Remove contract from available for rent queue.
	pub fn remove_from_available_queue(nft_id: NFTId) -> Result<(), DispatchError> {
		AvailableQueue::<T>::mutate(|x| -> DispatchResult {
			x.remove(nft_id);
			Ok(())
		})?;
		Ok(())
	}

	/// Put contract deadlines in respective queues.
	pub fn insert_in_queue(nft_id: NFTId, duration: &Duration<T::BlockNumber>) -> Result<(), DispatchError> {
		let current_block_number = <frame_system::Pallet<T>>::block_number();
		match duration {
			Duration::Fixed(blocks) => {
				let expiration_block = current_block_number + *blocks;
				FixedQueue::<T>::mutate(|x| -> DispatchResult {
					x.insert(nft_id, expiration_block)
						.map_err(|_| Error::<T>::MaxSimultaneousContractReached)?;
					Ok(())
				})?;
			},
			Duration::Subscription(blocks, _) => {
				let expiration_block = current_block_number + *blocks;
				SubscriptionQueue::<T>::mutate(|x| -> DispatchResult {
					x.insert(nft_id, expiration_block)
						.map_err(|_| Error::<T>::MaxSimultaneousContractReached)?;
					Ok(())
				})?;
			},
			Duration::Infinite => (),
		}
		Ok(())
	}

	/// Remove a contract from all queues and remove offers if some exist.
	pub fn remove_from_queues(
		nft_id: NFTId,
		contract: &RentContractData<T::AccountId, T::BlockNumber, BalanceOf<T>, T::AccountSizeLimit>,
	) -> Result<(), DispatchError> {
		if !contract.has_started {
			// Remove from available queue
			AvailableQueue::<T>::mutate(|x| x.remove(nft_id));

			// Clear offers
			if Offers::<T>::get(nft_id).is_some() {
				Offers::<T>::remove(nft_id);
			};
		} else {
			// Remove from fixed queue
			if let Duration::Fixed(_) = contract.duration {
				FixedQueue::<T>::mutate(|x| x.remove(nft_id));
			}

			// Remove from subscription queue
			if let Duration::Fixed(_) = contract.duration {
				SubscriptionQueue::<T>::mutate(|x| x.remove(nft_id));
			}
		};
		Ok(())
	}

	/// Get contract total blocks.
	pub fn get_contract_total_blocks(duration: &Duration<T::BlockNumber>) -> T::BlockNumber {
		match duration {
			Duration::Fixed(value) => *value,
			Duration::Subscription(value, _) => *value,
			Duration::Infinite => T::BlockNumber::from(0u32),
		}
	}

	/// Get contract end block.
	pub fn get_contract_end_block(nft_id: NFTId, duration: &Duration<T::BlockNumber>) -> Option<T::BlockNumber> {
		match duration {
			Duration::Fixed(_) => FixedQueue::<T>::get().get(nft_id),
			Duration::Subscription(_, _) => SubscriptionQueue::<T>::get().get(nft_id),
			Duration::Infinite => Some(T::BlockNumber::from(0u32)),
		}
	}

	/// Takes the rent fee from the rentee.
	pub fn take_rent_fee(
		from: &T::AccountId,
		to: T::AccountId,
		rent_fee: &RentFee<BalanceOf<T>>,
	) -> Result<(), DispatchError> {
		match rent_fee {
			RentFee::NFT(nft_id) => {
				let mut fee_nft = T::NFTExt::get_nft(*nft_id).ok_or(Error::<T>::NFTNotFoundForRentFee)?;
				ensure!(fee_nft.owner == *from, Error::<T>::NotTheNFTOwnerForRentFee);
				Self::ensure_nft_available(&fee_nft)?;
				fee_nft.owner = to;
				T::NFTExt::set_nft(*nft_id, fee_nft)?;
			},
			RentFee::Tokens(amount) => {
				ensure!(T::Currency::free_balance(&from) >= *amount, Error::<T>::NotEnoughBalanceForRentFee);
				T::Currency::transfer(&from, &to, *amount, KeepAlive)?;
			},
		}
		Ok(())
	}

	/// Takes the cancellation fee from renter or rentee.
	pub fn take_cancellation_fee(
		from: &T::AccountId,
		cancellation_fee: &Option<CancellationFee<BalanceOf<T>>>,
	) -> Result<(), DispatchError> {
		if let Some(cancellation_fee) = cancellation_fee {
			match cancellation_fee {
				CancellationFee::NFT(cancellation_nft_id) => {
					let mut cancellation_nft =
						T::NFTExt::get_nft(*cancellation_nft_id).ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
					ensure!(cancellation_nft.owner == *from, Error::<T>::NotTheNFTOwnerForCancellationFee);
					Self::ensure_nft_available(&cancellation_nft)?;
					cancellation_nft.owner = Self::account_id();
					T::NFTExt::set_nft(*cancellation_nft_id, cancellation_nft)?;
				},
				CancellationFee::FixedTokens(amount) => {
					ensure!(T::Currency::free_balance(from) >= *amount, Error::<T>::NotEnoughBalanceForCancellationFee);
					T::Currency::transfer(from, &Self::account_id(), *amount, KeepAlive)?;
				},
				CancellationFee::FlexibleTokens(amount) => {
					ensure!(T::Currency::free_balance(from) >= *amount, Error::<T>::NotEnoughBalanceForCancellationFee);
					T::Currency::transfer(from, &Self::account_id(), *amount, KeepAlive)?;
				},
			};
		}
		Ok(())
	}

	/// Return all the cancellation fee to recipient without any calculation.
	pub fn return_full_cancellation_fee(
		to: T::AccountId,
		cancellation_fee: &Option<CancellationFee<BalanceOf<T>>>,
	) -> Result<(), DispatchError> {
		if let Some(cancellation_fee) = cancellation_fee {
			match cancellation_fee {
				CancellationFee::NFT(cancellation_nft_id) => {
					let mut cancellation_nft =
						T::NFTExt::get_nft(*cancellation_nft_id).ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
					ensure!(cancellation_nft.owner == Self::account_id(), Error::<T>::NotTheNFTOwnerForCancellationFee);
					cancellation_nft.owner = to.clone();
					T::NFTExt::set_nft(*cancellation_nft_id, cancellation_nft)?;
				},
				CancellationFee::FixedTokens(amount) => {
					T::Currency::transfer(&Self::account_id(), &to, *amount, KeepAlive)?;
				},
				CancellationFee::FlexibleTokens(amount) => {
					T::Currency::transfer(&Self::account_id(), &to, *amount, KeepAlive)?;
				},
			};
		}
		Ok(())
	}

	/// Pay the cancellation fee to renter or rentee impacted by revoker's cancellation.
	pub fn pay_cancellation_fee(
		from: &T::AccountId,
		to: T::AccountId,
		cancellation_fee: &Option<CancellationFee<BalanceOf<T>>>,
		end_block: Option<T::BlockNumber>,
		total_blocks: T::BlockNumber,
	) -> Result<(), DispatchError> {
		if let Some(cancellation_fee) = cancellation_fee {
			match cancellation_fee {
				CancellationFee::NFT(cancellation_nft_id) => {
					let mut cancellation_nft =
						T::NFTExt::get_nft(*cancellation_nft_id).ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
					ensure!(cancellation_nft.owner == Self::account_id(), Error::<T>::NotTheNFTOwnerForCancellationFee);
					cancellation_nft.owner = to;
					T::NFTExt::set_nft(*cancellation_nft_id, cancellation_nft)?;
				},
				CancellationFee::FixedTokens(amount) => {
					T::Currency::transfer(&Self::account_id(), &to, *amount, KeepAlive)?;
				},
				CancellationFee::FlexibleTokens(amount) => {
					//TODO
					ensure!(end_block.is_some(), Error::<T>::FlexibleFeeEndBlockNotFound);
					if let Some(end_block) = end_block {
						// let current_block = <frame_system::Pallet<T>>::block_number();
						// let remaining_blocks =
						// 	end_block.checked_sub(&current_block).ok_or(Error::<T>::InternalMathError);
						// let part = amount
						// 	.saturating_mul(remaining_blocks.into())
						// 	.checked_div(total_blocks)
						// 	.ok_or(Error::<T>::InternalMathError);
						// let other_part = amount
						// 	.checked_sub(&part)
						// 	.ok_or(Error::<T>::InternalMathError);
						T::Currency::transfer(&Self::account_id(), &to, *amount, KeepAlive)?;
					}
				},
			};
		}
		Ok(())
	}

	/// Give back the cancellation fees depending on revoker and revocation timing.
	pub fn process_cancellation_fees(
		nft_id: NFTId,
		contract: &RentContractData<T::AccountId, T::BlockNumber, BalanceOf<T>, T::AccountSizeLimit>,
		revoker: Option<&T::AccountId>,
	) -> Result<(), DispatchError> {
		if !contract.has_started {
			Self::return_full_cancellation_fee(contract.renter.clone(), &contract.renter_cancellation_fee)?;
		} else {
			if let Some(revoker) = revoker {
				if *revoker == contract.renter {
					if let Some(rentee) = &contract.rentee {
						// Revoked by renter, rentee receive renter's cancellation fee
						let end_block = Self::get_contract_end_block(nft_id, &contract.duration);
						let total_blocks = Self::get_contract_total_blocks(&contract.duration);
						Self::pay_cancellation_fee(
							&contract.renter,
							rentee.clone(),
							&contract.renter_cancellation_fee,
							end_block,
							total_blocks,
						)?;

						// Rentee gets back his cancellation fee
						Self::return_full_cancellation_fee(rentee.clone(), &contract.rentee_cancellation_fee)?;
					};
				} else if let Some(rentee) = &contract.rentee {
					if *revoker == *rentee {
						// Revoked by rentee or Subscription payment stopped, renters receive rentees's cancellation fee
						let end_block = Self::get_contract_end_block(nft_id, &contract.duration);
						let total_blocks = Self::get_contract_total_blocks(&contract.duration);
						Self::pay_cancellation_fee(
							&rentee,
							contract.renter.clone(),
							&contract.renter_cancellation_fee,
							end_block,
							total_blocks,
						)?;

						// Renter gets back his cancellation fee
						Self::return_full_cancellation_fee(contract.renter.clone(), &contract.renter_cancellation_fee)?;
					}
				}
			} else {
				// Revoked cause contract ended (fixed, or maxSubscription reached)
				Self::return_full_cancellation_fee(contract.renter.clone(), &contract.renter_cancellation_fee)?;
				if let Some(rentee) = &contract.rentee {
					Self::return_full_cancellation_fee(rentee.clone(), &contract.rentee_cancellation_fee)?;
				};
			};
		};
		Ok(())
	}

	/// Insert an address into contract offers account list.
	pub fn insert_offer(nft_id: NFTId, rentee: T::AccountId) -> Result<(), DispatchError> {
		if let Some(mut offers) = Offers::<T>::get(nft_id) {
			if !offers.contains(&rentee) {
				offers.try_push(rentee.clone()).map_err(|_| Error::<T>::MaximumOffersReached)?;
				Offers::<T>::insert(nft_id, offers);
			}
		} else {
			let mut offers: AccountList<T::AccountId, T::AccountSizeLimit> = BoundedVec::default();
			offers.try_push(rentee.clone()).map_err(|_| Error::<T>::MaximumOffersReached)?;
			Offers::<T>::insert(nft_id, offers);
		}
		Ok(())
	}

	/// Remove an address from contract offers account list.
	pub fn remove_offer(nft_id: NFTId, rentee: T::AccountId) -> Result<(), DispatchError> {
		if let Some(mut offers) = Offers::<T>::get(nft_id) {
			if offers.contains(&rentee) {
				let index = offers.iter().position(|x| *x == rentee);
				if let Some(index) = index {
					offers.remove(index);
					Offers::<T>::insert(nft_id, offers);
				}
			}
		}
		Ok(())
	}
}
