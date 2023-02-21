// Copyright 2023 Capsule Corp (France) SAS.
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

use frame_support::{
	dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
	traits::{ExistenceRequirement::KeepAlive, OnUnbalanced, StorageVersion, WithdrawReasons},
	BoundedVec,
};
use primitives::nfts::NFTId;
use sp_runtime::SaturatedConversion;
use sp_std::vec;
use ternoa_common::traits::NFTExt;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	use frame_support::{pallet_prelude::*, traits::Currency};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
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

		/// What we do with additional fees.
		type FeesCollector: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Link to the NFT pallet.
		type NFTExt: NFTExt<AccountId = Self::AccountId>;

		// Constants
		/// Default fee for AtBlock protocol.
		#[pallet::constant]
		type InitialAtBlockFee: Get<BalanceOf<Self>>;

		/// Default fee for AtBlockWithReset protocol.
		#[pallet::constant]
		type InitialAtBlockWithResetFee: Get<BalanceOf<Self>>;

		/// Default fee for OnConsent protocol.
		#[pallet::constant]
		type InitialOnConsentFee: Get<BalanceOf<Self>>;

		/// Default fee for OnConsentAtBlock protocol.
		#[pallet::constant]
		type InitialOnConsentAtBlockFee: Get<BalanceOf<Self>>;

		/// Maximum block duration for a protocol.
		#[pallet::constant]
		type MaxBlockDuration: Get<u32>;

		/// Maximum size for the consent list.
		#[pallet::constant]
		type MaxConsentListSize: Get<u32>;

		/// Maximum number of simultaneous transmission protocol.
		#[pallet::constant]
		type SimultaneousTransmissionLimit: Get<u32>;

		/// Maximum number of actions in one block.
		#[pallet::constant]
		type ActionsInBlockLimit: Get<u32>;
	}

	/// How much does it cost to set an AtBlock protocol (extra fee on top of the tx fees).
	#[pallet::storage]
	#[pallet::getter(fn at_block_fee)]
	pub(super) type AtBlockFee<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, T::InitialAtBlockFee>;

	/// How much does it cost to set an AtBlockWithReset protocol (extra fee on top of the tx fees).
	#[pallet::storage]
	#[pallet::getter(fn at_block_with_reset_fee)]
	pub(super) type AtBlockWithResetFee<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, T::InitialAtBlockWithResetFee>;

	/// How much does it cost to set an OnConsent protocol (extra fee on top of the tx fees).
	#[pallet::storage]
	#[pallet::getter(fn on_consent_fee)]
	pub(super) type OnConsentFee<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, T::InitialOnConsentFee>;

	/// How much does it cost to set an OnConsentAtBlock protocol (extra fee on top of the tx fees).
	#[pallet::storage]
	#[pallet::getter(fn on_consent_at_block_fee)]
	pub(super) type OnConsentAtBlockFee<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, T::InitialOnConsentAtBlockFee>;

	/// Mapping of nft id and transmission data
	#[pallet::storage]
	#[pallet::getter(fn transmissions)]
	pub type Transmissions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NFTId,
		TransmissionData<T::AccountId, T::BlockNumber, T::MaxConsentListSize>,
		OptionQuery,
	>;

	/// Mapping of nft id and consent vectors
	#[pallet::storage]
	#[pallet::getter(fn on_consent_data)]
	pub type OnConsentData<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NFTId,
		ConsentList<T::AccountId, T::MaxConsentListSize>,
		OptionQuery,
	>;

	/// Data related to transmission queues.
	#[pallet::storage]
	#[pallet::getter(fn at_block_queue)]
	pub type AtBlockQueue<T: Config> =
		StorageValue<_, Queue<T::BlockNumber, T::SimultaneousTransmissionLimit>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Weight: see `begin_block`
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut read = 1u64;
			let mut write = 0u64;
			let mut current_actions = 0;
			let max_actions = T::ActionsInBlockLimit::get();

			let mut queue = AtBlockQueue::<T>::get();

			while let Some(nft_id) = queue.pop_next(now) {
				// Transmit the NFT
				_ = Self::transmit_nft(nft_id);

				// Deposit event.
				let event = Event::Transmitted { nft_id };
				Self::deposit_event(event);

				read += 2;
				write += 2;
				current_actions += 1;
				if current_actions >= max_actions {
					break
				}
			}

			if current_actions > 0 {
				AtBlockQueue::<T>::set(queue);
				write += 1;
			}
			T::DbWeight::get().reads_writes(read, write)
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A transmission protocol was set
		ProtocolSet {
			nft_id: NFTId,
			recipient: T::AccountId,
			protocol: TransmissionProtocol<
				T::BlockNumber,
				ConsentList<T::AccountId, T::MaxConsentListSize>,
			>,
			cancellation: CancellationPeriod<T::BlockNumber>,
		},
		/// A protocol was removed
		ProtocolRemoved {
			nft_id: NFTId,
		},
		TimerReset {
			nft_id: NFTId,
			new_block_number: T::BlockNumber,
		},
		ConsentAdded {
			nft_id: NFTId,
			from: T::AccountId,
		},
		ProtocolFeeSet {
			protocol: TransmissionProtocolKind,
			fee: BalanceOf<T>,
		},
		ThresholdReached {
			nft_id: NFTId,
		},
		Transmitted {
			nft_id: NFTId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Transmission data was not found for the NFT.
		TransmissionNotFound,
		/// An NFT was not found in storage.
		NFTNotFound,
		/// Caller is not the owner of the NFT.
		NotTheNFTOwner,
		/// Operation is not permitted because the recipient is the caller
		InvalidRecipient,
		/// Operation is not permitted because the NFT is listed.
		CannotSetTransmissionForListedNFTs,
		/// Operation is not permitted because the NFT is delegated.
		CannotSetTransmissionForDelegatedNFTs,
		/// Operation is not permitted because the NFT is rented.
		CannotSetTransmissionForRentedNFTs,
		/// Operation is not permitted because the NFT is souldbound and user is not the creator.
		CannotSetTransmissionForNotCreatedSoulboundNFTs,
		/// Operation is not permitted because the NFT secret is syncing.
		CannotSetTransmissionForSyncingSecretNFTs,
		/// Operation is not permitted because the NFT capsule is syncing.
		CannotSetTransmissionForSyncingCapsules,
		/// Operation is not permitted because the NFT is already in transmission.
		CannotSetTransmissionForNFTsInTransmission,
		/// Transmission block is in the past
		CannotSetTransmissionInThePast,
		/// Transmission duration is too long
		TransmissionIsInTooMuchTime,
		/// The maximum number of simultaneous transmission has been reached
		SimultaneousTransmissionLimitReached,
		/// Operation is not permitted because the NFT is not in transmission.
		NFTIsNotInTransmission,
		/// Operation is not permitted because the protocol was set as non cancellable.
		ProtocolIsNotCancellable,
		/// Operation is not permitted because the protocol was set as non resettable.
		ProtocolTimerCannotBeReset,
		/// Operation is not permitted because the protocol not consent based.
		ProtocolDoesNotAcceptConsent,
		/// Operation is not permitted because the consents already reached the needed threshold.
		ConsentAlreadyReachedThreshold,
		/// Operation is not permitted because the consents was already given by caller.
		AlreadyAddedConsent,
		/// Operation is not permitted because consent list is full, should never happen.
		ConsentListFull,
		/// The selected thresold is too high
		ThresholdTooHigh,
		/// The selected threshold is too low
		ThresholdTooLow,
		/// The consent list and thresold are invalid or incompatible
		InvalidConsentList,
		/// The consent list has duplicate values
		DuplicatesInConsentList,
		/// The consent is not allowed from this account
		ConsentNotAllowed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set a transmission protocol for the specified NFT
		#[pallet::weight(T::WeightInfo::set_transmission_protocol(AtBlockQueue::<T>::get().size() as u32))]
		pub fn set_transmission_protocol(
			origin: OriginFor<T>,
			nft_id: NFTId,
			recipient: T::AccountId,
			protocol: TransmissionProtocol<
				T::BlockNumber,
				ConsentList<T::AccountId, T::MaxConsentListSize>,
			>,
			cancellation: CancellationPeriod<T::BlockNumber>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Checks
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(who != recipient, Error::<T>::InvalidRecipient);
			ensure!(!nft.state.is_listed, Error::<T>::CannotSetTransmissionForListedNFTs);
			ensure!(!nft.state.is_delegated, Error::<T>::CannotSetTransmissionForDelegatedNFTs);
			ensure!(!nft.state.is_rented, Error::<T>::CannotSetTransmissionForRentedNFTs);
			ensure!(
				!(nft.state.is_soulbound && nft.owner != nft.creator),
				Error::<T>::CannotSetTransmissionForNotCreatedSoulboundNFTs
			);
			ensure!(
				!nft.state.is_syncing_secret,
				Error::<T>::CannotSetTransmissionForSyncingSecretNFTs
			);
			ensure!(
				!nft.state.is_syncing_capsule,
				Error::<T>::CannotSetTransmissionForSyncingCapsules
			);
			ensure!(
				!nft.state.is_transmission,
				Error::<T>::CannotSetTransmissionForNFTsInTransmission
			);

			if let Some(end_block) = protocol.get_end_block() {
				let now = frame_system::Pallet::<T>::block_number();
				ensure!(end_block > now, Error::<T>::CannotSetTransmissionInThePast);
				let duration: u32 = (end_block - now).saturated_into();
				ensure!(
					duration <= T::MaxBlockDuration::get(),
					Error::<T>::TransmissionIsInTooMuchTime
				)
			}

			if let Some((consent_list, threshold)) = protocol.get_consent_data() {
				let mut unique_consent_list: BoundedVec<T::AccountId, T::MaxConsentListSize> = BoundedVec::default();
				for account in consent_list {
					if !unique_consent_list.contains(&account){
						unique_consent_list.try_push(account.clone()).map_err(|_| Error::<T>::ConsentListFull)?;
					}
				}
				ensure!(consent_list.len() == unique_consent_list.len(), Error::<T>::DuplicatesInConsentList);
				ensure!(threshold > 0u8, Error::<T>::ThresholdTooLow);
				ensure!(
					(threshold as u32) <= T::MaxConsentListSize::get(),
					Error::<T>::ThresholdTooHigh
				);
				ensure!((threshold as usize) <= consent_list.len(), Error::<T>::InvalidConsentList);
			}

			let mut queue = AtBlockQueue::<T>::get();
			let mut has_change = false;

			if let Some(block_to_queue) = protocol.get_block_to_queue() {
				queue
					.can_be_increased(1)
					.ok_or(Error::<T>::SimultaneousTransmissionLimitReached)?;

				queue
					.insert(nft_id, block_to_queue)
					// This should never happen since we already did the check.
					.map_err(|_| Error::<T>::SimultaneousTransmissionLimitReached)?;

				has_change = true;
			}

			// Take protocol fee
			let protocol_fee = Self::get_protocol_fee(protocol.to_kind());
			let reason = WithdrawReasons::FEE;
			let imbalance = T::Currency::withdraw(&who, protocol_fee, reason, KeepAlive)?;
			T::FeesCollector::on_unbalanced(imbalance);

			// Execute
			let transmission_data =
				TransmissionData::new(recipient.clone(), protocol.clone(), cancellation.clone());
			Transmissions::<T>::insert(nft_id, transmission_data);

			nft.state.is_transmission = true;
			T::NFTExt::set_nft(nft_id, nft)?;

			if has_change {
				AtBlockQueue::<T>::set(queue);
			}

			let event = Event::ProtocolSet { nft_id, recipient, protocol, cancellation };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Remove a transmission protocol for the specified NFT
		#[pallet::weight(T::WeightInfo::remove_transmission_protocol(AtBlockQueue::<T>::get().size() as u32))]
		pub fn remove_transmission_protocol(
			origin: OriginFor<T>,
			nft_id: NFTId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			// Checks
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(nft.state.is_transmission, Error::<T>::NFTIsNotInTransmission);

			let transmission_data =
				Transmissions::<T>::get(nft_id).ok_or(Error::<T>::TransmissionNotFound)?;

			ensure!(
				transmission_data.cancellation.is_cancellable(now),
				Error::<T>::ProtocolIsNotCancellable
			);

			// Execute
			nft.state.is_transmission = false;
			T::NFTExt::set_nft(nft_id, nft)?;

			AtBlockQueue::<T>::mutate(|x| {
				x.remove(nft_id);
			});

			Transmissions::<T>::remove(nft_id);
			OnConsentData::<T>::remove(nft_id);

			let event = Event::ProtocolRemoved { nft_id };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Reset the timer of the specified NFT transmission
		#[pallet::weight(T::WeightInfo::reset_timer(AtBlockQueue::<T>::get().size() as u32))]
		pub fn reset_timer(
			origin: OriginFor<T>,
			nft_id: NFTId,
			block_number: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			// Checks
			let nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(nft.state.is_transmission, Error::<T>::NFTIsNotInTransmission);

			let mut transmission_data =
				Transmissions::<T>::get(nft_id).ok_or(Error::<T>::TransmissionNotFound)?;

			ensure!(
				transmission_data.protocol.can_reset_timer(),
				Error::<T>::ProtocolTimerCannotBeReset
			);

			ensure!(block_number > now, Error::<T>::CannotSetTransmissionInThePast);
			let duration: u32 = (block_number - now).saturated_into();
			ensure!(
				duration <= T::MaxBlockDuration::get(),
				Error::<T>::TransmissionIsInTooMuchTime
			);

			// Execute
			AtBlockQueue::<T>::mutate(|x| {
				x.update(nft_id, block_number);
			});

			transmission_data.protocol = TransmissionProtocol::AtBlockWithReset(block_number);
			Transmissions::<T>::insert(nft_id, transmission_data);

			let event = Event::TimerReset { nft_id, new_block_number: block_number };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Add the caller consent to trigger nft's transmission
		#[pallet::weight(T::WeightInfo::add_consent(AtBlockQueue::<T>::get().size() as u32))]
		pub fn add_consent(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			// Checks
			let nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.state.is_transmission, Error::<T>::NFTIsNotInTransmission);

			let transmission_data =
				Transmissions::<T>::get(nft_id).ok_or(Error::<T>::TransmissionNotFound)?;

			let mut queue = AtBlockQueue::<T>::get();

			ensure!(
				transmission_data.protocol.can_add_consent(),
				Error::<T>::ProtocolDoesNotAcceptConsent
			);

			if transmission_data.protocol.to_kind() == TransmissionProtocolKind::OnConsentAtBlock {
				ensure!(queue.get(nft_id).is_none(), Error::<T>::ConsentAlreadyReachedThreshold)
			}

			if let Some(consent_list) = transmission_data.protocol.get_consent_list() {
				ensure!(consent_list.contains(&who), Error::<T>::ConsentNotAllowed)
			}

			let mut consent_list: BoundedVec<T::AccountId, T::MaxConsentListSize> =
				BoundedVec::default();
			if let Some(existing_consent_list) = OnConsentData::<T>::get(nft_id) {
				ensure!(!existing_consent_list.contains(&who), Error::<T>::AlreadyAddedConsent);
				consent_list = existing_consent_list;
			}
			consent_list.try_push(who.clone()).map_err(|_| Error::<T>::ConsentListFull)?;

			let mut has_reached_threshold = false;
			let mut should_transmit = false;
			if let Some(threshold) = transmission_data.protocol.get_threshold() {
				if consent_list.len() < (threshold as usize) {
					OnConsentData::<T>::insert(nft_id, consent_list);
				} else {
					has_reached_threshold = true;
					if let Some(end_block) = transmission_data.protocol.get_end_block() {
						if end_block <= now {
							should_transmit = true;
						} else {
							AtBlockQueue::<T>::mutate(|x| -> DispatchResult {
								x.can_be_increased(1)
									.ok_or(Error::<T>::SimultaneousTransmissionLimitReached)?;
								x.insert(nft_id, end_block).map_err(|_| {
									Error::<T>::SimultaneousTransmissionLimitReached
								})?;
								Ok(())
							})?;
						}
					} else {
						should_transmit = true;
					}
				}
			}

			// If we reached threshold, we remove the consent list
			if has_reached_threshold {
				OnConsentData::<T>::remove(nft_id);
			}

			// Transmit if it's immediate
			if should_transmit {
				Self::transmit_nft(nft_id)?;
			}

			// Consent added
			let event = Event::ConsentAdded { nft_id, from: who };
			Self::deposit_event(event);

			// Threshold reached
			if has_reached_threshold {
				let event = Event::ThresholdReached { nft_id };
				Self::deposit_event(event);
			}

			// NFT Transmitted
			if should_transmit {
				let event = Event::Transmitted { nft_id };
				Self::deposit_event(event);
			}

			Ok(().into())
		}

		/// Set the fee for the specified protocol
		#[pallet::weight(T::WeightInfo::set_protocol_fee())]
		pub fn set_protocol_fee(
			origin: OriginFor<T>,
			protocol_kind: TransmissionProtocolKind,
			fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			match protocol_kind.clone() {
				TransmissionProtocolKind::AtBlock => AtBlockFee::<T>::put(fee),
				TransmissionProtocolKind::AtBlockWithReset => AtBlockWithResetFee::<T>::put(fee),
				TransmissionProtocolKind::OnConsent => OnConsentFee::<T>::put(fee),
				TransmissionProtocolKind::OnConsentAtBlock => OnConsentAtBlockFee::<T>::put(fee),
			};

			let event = Event::ProtocolFeeSet { protocol: protocol_kind, fee };
			Self::deposit_event(event);
			Ok(().into())
		}
	}
}

// Helper Methods for Storage
impl<T: Config> Pallet<T> {
	/// Transmit the nft from owner to recipient of protocol when it's due
	/// Cleans the transmissions mapping and set nft state to not in transmission
	fn transmit_nft(nft_id: NFTId) -> DispatchResult {
		let transmission_data =
			Transmissions::<T>::get(nft_id).ok_or(Error::<T>::TransmissionNotFound)?;
		T::NFTExt::mutate_nft(nft_id, |x| -> DispatchResult {
			let nft = x.as_mut().ok_or(Error::<T>::NFTNotFound)?;
			nft.owner = transmission_data.recipient;
			nft.state.is_transmission = false;
			Ok(())
		})?;
		Transmissions::<T>::remove(nft_id);
		Ok(())
	}

	/// Get the protocol additional setup fee
	fn get_protocol_fee(protocol_kind: TransmissionProtocolKind) -> BalanceOf<T> {
		match protocol_kind {
			TransmissionProtocolKind::AtBlock => AtBlockFee::<T>::get(),
			TransmissionProtocolKind::AtBlockWithReset => AtBlockWithResetFee::<T>::get(),
			TransmissionProtocolKind::OnConsent => OnConsentFee::<T>::get(),
			TransmissionProtocolKind::OnConsentAtBlock => OnConsentAtBlockFee::<T>::get(),
		}
	}

	/// Fill AtBlockQueue with any number of data
	pub fn fill_queue(
		number: u32,
		nft_id: NFTId,
		block_number: T::BlockNumber,
	) -> Result<(), DispatchError> {
		AtBlockQueue::<T>::try_mutate(|x| -> DispatchResult {
			x.bulk_insert(nft_id, block_number, number)
				.map_err(|_| Error::<T>::SimultaneousTransmissionLimitReached)?;
			Ok(())
		})?;
		Ok(())
	}

	/// Fill any consent list with any number of data
	pub fn fill_consent_list(
		number: u32,
		nft_id: NFTId,
		account: T::AccountId,
	) -> Result<(), DispatchError> {
		let accounts = BoundedVec::try_from(vec![account.clone(); number as usize])
			.map_err(|_| Error::<T>::SimultaneousTransmissionLimitReached)?;
		OnConsentData::<T>::insert(nft_id, accounts);
		Ok(())
	}
}
