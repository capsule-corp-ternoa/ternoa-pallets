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
	traits::{
		tokens::Balance, Currency, ExistenceRequirement::KeepAlive, Get, OnUnbalanced,
		StorageVersion, WithdrawReasons,
	},
	transactional, BoundedVec, PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AccountIdConversion, CheckedSub, Saturating, StaticLookup};
use sp_std::prelude::*;

use primitives::{
	nfts::{NFTId, NFTState},
	CompoundFee, ConfigOp, U8BoundedVec,
};
use ternoa_common::{config_op_field_exp, traits::NFTExt};
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

	/// Data related to rent contracts
	#[pallet::storage]
	#[pallet::getter(fn contracts)]
	pub type Contracts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NFTId,
		RentContractData<T::AccountId, T::BlockNumber, BalanceOf<T>, T::AccountSizeLimit>,
		OptionQuery,
	>;

	/// Data related to fixed contract deadlines
	#[pallet::storage]
	#[pallet::getter(fn fixed_queue)]
	pub type FixedQueue<T: Config> =
		StorageValue<_, Queue<T::BlockNumber, T::SimultaneousContractLimit>, ValueQuery>;

	/// Data related to subscription contract deadlines
	#[pallet::storage]
	#[pallet::getter(fn subscription_queue)]
	pub type SubscriptionQueue<T: Config> =
		StorageValue<_, Queue<T::BlockNumber, T::SimultaneousContractLimit>, ValueQuery>;

	/// Data related to available for rent contract deadlines
	#[pallet::storage]
	#[pallet::getter(fn available_queue)]
	pub type AvailableQueue<T: Config> =
		StorageValue<_, Queue<T::BlockNumber, T::SimultaneousContractLimit>, ValueQuery>;

	/// Data related to rent contracts offers
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
			revocation_type: RevocationType<BalanceOf<T>, T::BlockNumber>,
			rent_fee: RentFee<BalanceOf<T>>,
			renter_cancellation_fee: Option<CancellationFee<BalanceOf<T>>>,
			rentee_cancellation_fee: Option<CancellationFee<BalanceOf<T>>>,
		},
		/// Contract was accepted and has started.
		ContractStarted { nft_id: NFTId, rentee: T::AccountId },
		/// Contract was revoked by either renter or rentee.
		ContractRevoked {
			nft_id: NFTId,
			revoked_by: T::AccountId,
		},
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
		/// A contract new subscription's terms were declined by rentee.
		ContractSubscriptionTermsDeclined { nft_id: NFTId },
		/// A contract has ended.
		ContractEnded { nft_id: NFTId },
		/// A contract's subscription period has started.
		ContractSubscriptionPeriodStarted { nft_id: NFTId },
		/// A contract available for sale was expired before its acceptance
		ContractAvailableExpired { nft_id: NFTId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// NFT not found.
		NFTNotFound,
		/// Not the owner of the NFT.
		NotTheNFTOwner,
		/// Operation is not permitted because NFT is listed.
		CannotRentOutListedNFTs,
		/// Operation is not permitted because NFT is capsule.
		CannotRentOutCapsuleNFTs,
		/// Operation is not permitted because NFT is delegated.
		CannotRentOutDelegatedNFTs,
		/// Operation is not permitted because NFT is delegated.
		CannotRentOutSoulboundNFTs,
		/// Operation is not permitted because NFT is auctioned.
		CannotRentOutAuctionedNFTs,
		/// Operation is not permitted because NFT is rented.
		CannotRentOutRentedNFTs,
		/// Operation is not permitted because NFT for cancellation fee was not found.
		NFTNotFoundForCancellationFee,
		/// Operation is not permitted because NFT for cancellation fee is not owned by caller.
		NotTheNFTOwnerForCancellationFee,
		/// Operation is not permitted because NFT for cancellation fee is listed.
		CannotPutListedNFTsForCancellationFee,
		/// Operation is not permitted because NFT for cancellation fee is capsule.
		CannotPutCapsuleNFTsForCancellationFee,
		/// Operation is not permitted because NFT for cancellation fee is delegated.
		CannotPutDelegatedNFTsForCancellationFee,
		/// Operation is not permitted because NFT for cancellation fee is soulbound.
		CannotPutSoulboundNFTsForCancellationFee,
		/// Operation is not permitted because NFT for cancellation fee is auctioned.
		CannotPutAuctionedNFTsForCancellationFee,
		/// Operation is not permitted because NFT for cancellation fee is rented.
		CannotPutRentedNFTsForCancellationFee,
		/// Operation is not permitted because the maximum number au parallel rent contract has
		/// been reached.
		MaxSimultaneousContractReached,
		/// Cannot create a contract with fixed duration and onSubscriptionChange revocation type.
		NoFixedWithSubscriptionChanges,
		/// Cannot create a contract with infinite duration and onSubscriptionChange revocation
		/// type.
		NoInfiniteWithSubscriptionChanges,
		/// Cannot create a contract with infinite duration and flexible tokens cancellation fee
		NoInfiniteWithFlexibleFee,
		/// Cannot create a contract with no revocation type and a renter cancellation fee.
		CannotSetRenterCancellationFeeWithNoRevocation,
		/// The contract was not found for the given nft_id.
		ContractNotFound,
		/// The caller is neither the renter or rentee.
		NotTheRenterOrRentee,
		/// Operation is not permitted because revocation type is not anytime.
		CannotRevoke,
		/// The owner was not found to return flexible fee.
		FlexibleFeeOwnerNotFound,
		/// The total duration was not found to return flexible fee
		FlexibleFeeEndTotalDurationNotFound,
		/// The endblock was not found to return flexible fee.
		FlexibleFeeEndBlockNotFound,
		/// Math error
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

			// Checks
			let is_fixed = matches!(duration, Duration::Fixed { .. });
			let is_on_subscription_change = matches!(revocation_type, RevocationType::OnSubscriptionChange { .. });
			ensure!(
				!(is_fixed && is_on_subscription_change),
				Error::<T>::NoFixedWithSubscriptionChanges
			);
			ensure!(
				!(duration == Duration::Infinite && is_on_subscription_change),
				Error::<T>::NoInfiniteWithSubscriptionChanges
			);
			ensure!(
				!(revocation_type == RevocationType::NoRevocation &&
					renter_cancellation_fee.is_some()),
				Error::<T>::CannotSetRenterCancellationFeeWithNoRevocation
			);
			let is_flexible_token_renter = matches!(renter_cancellation_fee, Some(CancellationFee::FlexibleTokens { .. }));
			let is_flexible_token_rentee = matches!(rentee_cancellation_fee, Some(CancellationFee::FlexibleTokens { .. }));
			ensure!(
				!(duration == Duration::Infinite &&
					(is_flexible_token_renter || is_flexible_token_rentee)),
				Error::<T>::NoInfiniteWithFlexibleFee
			);

			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(!nft.state.is_listed, Error::<T>::CannotRentOutListedNFTs);
			ensure!(!nft.state.is_capsule, Error::<T>::CannotRentOutCapsuleNFTs);
			ensure!(!nft.state.is_delegated, Error::<T>::CannotRentOutDelegatedNFTs);
			ensure!(!nft.state.is_soulbound, Error::<T>::CannotRentOutSoulboundNFTs);
			ensure!(!nft.state.is_auctioned, Error::<T>::CannotRentOutAuctionedNFTs);
			ensure!(!nft.state.is_rented, Error::<T>::CannotRentOutRentedNFTs);

			// Take cancellation fee for renter if it exist.
			if let Some(cancellation_fee) = &renter_cancellation_fee {
				Self::take_cancellation_fee(who.clone(), cancellation_fee)?;
			}

			// Insert in available queue with expiration
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let expiration_duration = T::ContractExpirationDuration::get();
			let expiration_block = current_block_number + expiration_duration.into();
			AvailableQueue::<T>::mutate(|x| -> DispatchResult {
				x.insert(nft_id, expiration_block)
					.map_err(|_| Error::<T>::MaxSimultaneousContractReached)?;
				Ok(())
			})?;

			// Set NFT state
			nft.state.is_rented = true;

			// Inser new contract
			let contract = RentContractData::new(
				false,
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
			T::NFTExt::set_nft(nft_id, nft);

			// Deposit event
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

		/// Revoke a rent contract, cancel it if it has not started
		#[pallet::weight(T::WeightInfo::transfer_nft())]
		#[transactional]
		pub fn revoke_contract(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			let contract = Contracts::<T>::get(nft_id).ok_or(Error::<T>::ContractNotFound)?;
			ensure!(
				contract.renter == who || contract.rentee == Some(who.clone()),
				Error::<T>::NotTheRenterOrRentee
			);
			if contract.renter == who {
				if !contract.has_started {
					// Give back cancellation fee for renter if it exist and contract has not
					// started yet.
					if let Some(cancellation_fee) = &contract.renter_cancellation_fee {
						//TODO
						// Self::move_cancellation_fee(who.clone(), cancellation_fee)?;
					}

					// Remove from available queue
					AvailableQueue::<T>::mutate(|x| x.remove(nft_id));
				} else {
					ensure!(
						contract.revocation_type == RevocationType::Anytime,
						Error::<T>::CannotRevoke
					);
					// Give renter's cancellation fee to rentee if it exists
					if let Some(cancellation_fee) = &contract.rentee_cancellation_fee {
						// TODO
						// Self::move_cancellation_fee(contract.renter.clone(), cancellation_fee)?;
					}
					// Give back rentee's cancellation fee to himself if it exists
					if let Some(cancellation_fee) = &contract.renter_cancellation_fee {
						// TODO
						// Self::move_cancellation_fee(contract.renter.clone(), cancellation_fee)?;
					}


				};
			} else {
				// Give rentee's cancellation fee to renter if it exists
				if let Some(cancellation_fee) = &contract.rentee_cancellation_fee {
					// TODO
					// Self::move_cancellation_fee(contract.renter.clone(), cancellation_fee)?;
				}
				// Give back renter's cancellation fee to himself if it exists
				if let Some(cancellation_fee) = &contract.renter_cancellation_fee {
					// TODO
					// Self::move_cancellation_fee(contract.renter.clone(), cancellation_fee)?;
				}
			};

			// Remove from fixed queue
			if let Duration::Fixed(_) = contract.duration {
				FixedQueue::<T>::mutate(|x| x.remove(nft_id));
			}

			// Remove from subscription queue
			if let Duration::Fixed(_) = contract.duration {
				SubscriptionQueue::<T>::mutate(|x| x.remove(nft_id));
			}

			// Set NFT state back
			nft.state.is_rented = false;

			// Remove contract
			Contracts::<T>::remove(nft_id);

			// Deposit event
			let event = Event::ContractRevoked {
				nft_id,
				revoked_by: who,
			};
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

	/// Takes the cancellation fee
	pub fn take_cancellation_fee(
		from: T::AccountId,
		cancellation_fee: &CancellationFee<BalanceOf<T>>,
	) -> Result<(), DispatchError> {
		match cancellation_fee {
			CancellationFee::NFT(cancellation_nft_id) => {
				let mut cancellation_nft = T::NFTExt::get_nft(*cancellation_nft_id)
					.ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
				ensure!(
					cancellation_nft.owner == from,
					Error::<T>::NotTheNFTOwnerForCancellationFee
				);
				ensure!(
					!cancellation_nft.state.is_listed,
					Error::<T>::CannotPutListedNFTsForCancellationFee
				);
				ensure!(
					!cancellation_nft.state.is_capsule,
					Error::<T>::CannotPutCapsuleNFTsForCancellationFee
				);
				ensure!(
					!cancellation_nft.state.is_delegated,
					Error::<T>::CannotPutDelegatedNFTsForCancellationFee
				);
				ensure!(
					!cancellation_nft.state.is_soulbound,
					Error::<T>::CannotPutSoulboundNFTsForCancellationFee
				);
				ensure!(
					!cancellation_nft.state.is_auctioned,
					Error::<T>::CannotPutAuctionedNFTsForCancellationFee
				);
				ensure!(
					!cancellation_nft.state.is_rented,
					Error::<T>::CannotPutRentedNFTsForCancellationFee
				);
				cancellation_nft.owner = Self::account_id();
				T::NFTExt::set_nft(*cancellation_nft_id, cancellation_nft)?;
			},
			CancellationFee::FixedTokens(amount) => {
				T::Currency::transfer(&from, &Self::account_id(), *amount, KeepAlive)?;
			},
			CancellationFee::FlexibleTokens(amount) => {
				T::Currency::transfer(&from, &Self::account_id(), *amount, KeepAlive)?;
			},
		};
		Ok(())
	}

	/// Give back the cancellation fee
	pub fn move_cancellation_fee(
		to: T::AccountId,
		cancellation_fee: &CancellationFee<BalanceOf<T>>,
		flexible_fee_origin: Option<T::AccountId>,
		total_duration: Option<T::BlockNumber>,
		end_block: Option<T::BlockNumber>,
	) -> Result<(), DispatchError> {
		match cancellation_fee {
			CancellationFee::NFT(cancellation_nft_id) => {
				let mut cancellation_nft = T::NFTExt::get_nft(*cancellation_nft_id)
					.ok_or(Error::<T>::NFTNotFoundForCancellationFee)?;
				ensure!(
					cancellation_nft.owner == Self::account_id(),
					Error::<T>::NotTheNFTOwnerForCancellationFee
				);
				cancellation_nft.owner = to;
			},
			CancellationFee::FixedTokens(amount) => {
				T::Currency::transfer(&Self::account_id(), &to, *amount, KeepAlive)?;
			},
			CancellationFee::FlexibleTokens(amount) => {
				//TODO
				// ensure!(flexible_fee_origin.is_some(), Error::<T>::FlexibleFeeOwnerNotFound);
				// ensure!(total_duration.is_some(), Error::<T>::FlexibleFeeEndTotalDurationNotFound);
				// ensure!(end_block.is_some(), Error::<T>::FlexibleFeeEndBlockNotFound);
				// let block_number = <frame_system::Pallet<T>>::block_number();
				// // price = price.checked_sub(&commission_fee).ok_or(Error::<T>::InternalMathError)?;
				// if let Some(end_block) = end_block {
				// 	let remaining = end_block
				// 		.checked_sub(&block_number)
				// 		.ok_or(Error::<T>::InternalMathError)?;
				// 	if let Some(total_duration) = total_duration {
				// 		let passed = total_duration.checked_sub(&remaining).ok_or(Error::<T>::InternalMathError)?;
				// 		//TODO
				// 		// let amount = remaining.saturating_mul(amount.into()).ok_or(Error::<T>::InternalMathError)?;
				// 		// let ratio = remaining. / total_duration;
				// 		// T::Currency::transfer(&Self::account_id(), &to, *amount, KeepAlive)?;
				// 	}
				// }
				T::Currency::transfer(&Self::account_id(), &to, *amount, KeepAlive)?;
			},
		};
		Ok(())
	}
}
