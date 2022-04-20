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

mod types;
mod weights;

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{
		Currency, EnsureOrigin,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		Get, OnUnbalanced, WithdrawReasons,
	},
	transactional, BoundedVec, PalletId,
};
use frame_system::ensure_signed;
use sp_core::U256;
use sp_runtime::{
	traits::{AccountIdConversion, StaticLookup},
	SaturatedConversion,
};
use sp_std::prelude::*;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Associated type for Event enum
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;

		/// Currency type.
		type Currency: Currency<Self::AccountId>;

		/// What we do with additional fees
		type FeesCollector: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Origin that can control this pallet.
		type ExternalOrigin: EnsureOrigin<Self::Origin>;

		/// The identifier for this chain.
		/// This must be unique and must not collide with existing IDs within a set of bridged
		/// chains.
		#[pallet::constant]
		type ChainId: Get<ChainId>;

		/// Constant configuration parameter to store the module identifier for the pallet.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type ProposalLifetime: Get<Self::BlockNumber>;

		/// Type for setting initial number of votes required for a proposal to be executed (see
		/// [RelayerVoteThreshold] in storage section).
		#[pallet::constant]
		type RelayerVoteThreshold: Get<u32>;

		/// Total amount of accounts that can be in the bidder list.
		#[pallet::constant]
		type RelayerCountLimit: Get<u32>;

		/// Total amount of accounts that can be in the bidder list.
		#[pallet::constant]
		type InitialBridgeFee: Get<BalanceOf<Self>>;
	}

	/// All whitelisted chains and their respective transaction counts
	#[pallet::storage]
	#[pallet::getter(fn chain_nonces)]
	pub type ChainNonces<T: Config> = StorageMap<_, Blake2_256, ChainId, DepositNonce, OptionQuery>;

	/// Number of votes required for a proposal to execute
	#[pallet::storage]
	#[pallet::getter(fn relayer_vote_threshold)]
	pub type RelayerVoteThreshold<T: Config> =
		StorageValue<_, u32, ValueQuery, T::RelayerVoteThreshold>;

	/// Tracks current relayer set
	#[pallet::storage]
	#[pallet::getter(fn relayers)]
	pub type Relayers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::RelayerCountLimit>, ValueQuery>;

	/// All known proposals.
	/// The key is the hash of the call and the deposit ID, to ensure it's unique.
	#[pallet::storage]
	#[pallet::getter(fn get_votes)]
	pub type Votes<T: Config> = StorageDoubleMap<
		_,
		Blake2_256,
		ChainId,
		Blake2_256,
		(DepositNonce, T::AccountId, BalanceOf<T>),
		Proposal<T::AccountId, T::BlockNumber, T::RelayerCountLimit>,
		OptionQuery,
	>;

	/// Host much does it cost to transfer Native through the bridge (extra fee on top of the tx
	/// fees)
	#[pallet::storage]
	#[pallet::getter(fn bridge_fee)]
	pub type BridgeFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery, T::InitialBridgeFee>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Vote threshold has changed (new_threshold)
		RelayerThresholdUpdated {
			threshold: u32,
		},
		/// Chain now available for transfers (chain_id)
		ChainWhitelisted {
			chain_id: ChainId,
		},
		/// Relayer added to set
		RelayersUpdated {
			relayers: BoundedVec<T::AccountId, T::RelayerCountLimit>,
		},
		/// FunglibleTransfer is for relaying fungibles (dest_id, nonce, amount, recipient)
		FungibleTransfer(ChainId, DepositNonce, U256, Vec<u8>),
		/// Vote submitted in favour of proposal
		RelayerVoted {
			chain_id: ChainId,
			nonce: DepositNonce,
			account: T::AccountId,
			in_favour: bool,
		},
		/// Voting successful for a proposal
		ProposalApproved {
			chain_id: ChainId,
			nonce: DepositNonce,
		},
		/// Voting rejected a proposal
		ProposalRejected {
			chain_id: ChainId,
			nonce: DepositNonce,
		},
		BridgeFeeUpdated {
			fee: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// TODO!
		ThresholdCannotBeZero,
		/// Provided chain Id is not valid
		InvalidChainId,
		/// Relayer threshold cannot be 0
		InvalidThreshold,
		/// Interactions with this chain is not permitted
		ChainNotWhitelisted,
		/// Chain has already been enabled
		ChainAlreadyWhitelisted,
		/// Protected operation, must be performed by relayer
		MustBeRelayer,
		/// Relayer has already submitted some vote for this proposal
		RelayerAlreadyVoted,
		/// Proposal has either failed or succeeded
		ProposalAlreadyComplete,
		/// Lifetime of proposal has been exceeded
		ProposalExpired,
		/// TODO!
		MaximumVoteLimitExceeded,
		/// TODO!
		InvalidTransfer,
		/// TODO!
		RemovalImpossible,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sets the vote threshold for proposals.
		///
		/// This threshold is used to determine how many votes are required
		/// before a proposal is executed.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_threshold())]
		pub fn set_threshold(origin: OriginFor<T>, threshold: u32) -> DispatchResultWithPostInfo {
			T::ExternalOrigin::ensure_origin(origin)?;
			ensure!(threshold > 0, Error::<T>::ThresholdCannotBeZero);

			RelayerVoteThreshold::<T>::put(threshold);
			Self::deposit_event(Event::RelayerThresholdUpdated { threshold });

			Ok(().into())
		}

		/// Enables a chain ID as a source or destination for a bridge transfer.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::whitelist_chain())]
		pub fn whitelist_chain(
			origin: OriginFor<T>,
			chain_id: ChainId,
		) -> DispatchResultWithPostInfo {
			T::ExternalOrigin::ensure_origin(origin)?;

			ensure!(chain_id != T::ChainId::get(), Error::<T>::InvalidChainId);
			ensure!(!Self::chain_whitelisted(chain_id), Error::<T>::ChainAlreadyWhitelisted);

			ChainNonces::<T>::insert(&chain_id, 0);
			Self::deposit_event(Event::ChainWhitelisted { chain_id });

			Ok(().into())
		}

		/// Adds a new relayer to the relayer set.
		#[pallet::weight(100)]
		pub fn set_relayers(
			origin: OriginFor<T>,
			relayers: BoundedVec<T::AccountId, T::RelayerCountLimit>,
		) -> DispatchResultWithPostInfo {
			T::ExternalOrigin::ensure_origin(origin)?;

			Relayers::<T>::put(relayers.clone());
			Self::deposit_event(Event::RelayersUpdated { relayers });

			Ok(().into())
		}

		/// Commits a vote in favour of the provided proposal.
		///
		/// If a proposal with the given nonce and source chain ID does not already exist, it will
		/// be created with an initial vote in favour from the caller.
		#[pallet::weight(1000)]
		pub fn vote_proposal(
			origin: OriginFor<T>,
			chain_id: ChainId,
			nonce: DepositNonce,
			recipient: <T::Lookup as StaticLookup>::Source,
			amount: BalanceOf<T>,
			in_favour: bool,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;
			let recipient = T::Lookup::lookup(recipient)?;

			ensure!(Self::is_relayer(&account), Error::<T>::MustBeRelayer);
			ensure!(Self::chain_whitelisted(chain_id), Error::<T>::ChainNotWhitelisted);

			let now = frame_system::Pallet::<T>::block_number();
			let threshold = Self::relayer_vote_threshold();
			let mut result = None;
			let error = Error::<T>::MaximumVoteLimitExceeded;

			Votes::<T>::try_mutate(
				chain_id,
				(nonce, recipient.clone(), amount.clone()),
				|proposal| -> DispatchResult {
					if let Some(proposal) = proposal {
						ensure!(!proposal.is_complete(), Error::<T>::ProposalAlreadyComplete);
						ensure!(!proposal.is_expired(now), Error::<T>::ProposalExpired);
						ensure!(!proposal.has_voted(&account), Error::<T>::RelayerAlreadyVoted);

						proposal.votes.try_push((account.clone(), in_favour)).map_err(|_| error)?;
						result = proposal.try_to_complete(threshold);
					} else {
						let lifetime = T::ProposalLifetime::get();
						let initial = BoundedVec::try_from(vec![(account.clone(), in_favour)])
							.map_err(|_| error)?;
						let mut new_proposal = Proposal::new(initial, now + lifetime);
						result = new_proposal.try_to_complete(threshold);
						*proposal = Some(new_proposal);
					}

					// Send Vote Event
					let event = Event::RelayerVoted { chain_id, nonce, account, in_favour };
					Self::deposit_event(event);

					Ok(())
				},
			)?;

			// Let's see if the proposal is already completed
			if let Some(result) = result {
				match result {
					ProposalStatus::Approved => {
						let negative_imbalance = <T as Config>::Currency::issue(amount);
						<T as Config>::Currency::resolve_creating(&recipient, negative_imbalance);
						Self::deposit_event(Event::ProposalApproved { chain_id, nonce });
					},
					ProposalStatus::Rejected => {
						Self::deposit_event(Event::ProposalRejected { chain_id, nonce });
					},
					_ => {},
				}
			}

			Ok(().into())
		}

		/// Transfers some amount of the native token to some recipient on a (whitelisted)
		/// destination chain.
		#[pallet::weight(100)]
		#[transactional]
		pub fn transfer(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			recipient: Vec<u8>,
			dest_id: ChainId,
		) -> DispatchResultWithPostInfo {
			let source = ensure_signed(origin)?;
			let bridge_fee = Self::bridge_fee();
			let total = bridge_fee + amount;

			ensure!(Self::chain_whitelisted(dest_id), Error::<T>::ChainNotWhitelisted);
			ensure!(T::Currency::free_balance(&source) >= total, Error::<T>::RemovalImpossible);

			let imbalance =
				T::Currency::withdraw(&source, bridge_fee, WithdrawReasons::all(), KeepAlive)?;
			T::FeesCollector::on_unbalanced(imbalance);

			T::Currency::withdraw(&source, amount, WithdrawReasons::TRANSFER, AllowDeath)?;
			T::Currency::burn(amount);

			// Bump nonce
			let nonce = Self::chain_nonces(dest_id).unwrap_or_default() + 1;
			ChainNonces::<T>::insert(dest_id, nonce);

			let amount = U256::from(amount.saturated_into::<u128>());
			Self::deposit_event(Event::FungibleTransfer(dest_id, nonce, amount, recipient));

			Ok(().into())
		}

		/// Update the bridge fee value
		#[pallet::weight(100)]
		pub fn set_bridge_fee(
			origin: OriginFor<T>,
			bridge_fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ExternalOrigin::ensure_origin(origin)?;

			BridgeFee::<T>::put(bridge_fee);
			Self::deposit_event(Event::BridgeFeeUpdated { fee: bridge_fee });

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	// Public immutables and private mutables functions

	/// Provides an AccountId for the pallet.
	/// This is used both as an origin check and deposit/withdrawal account.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	/// Checks if who is a relayer
	pub fn is_relayer(who: &T::AccountId) -> bool {
		Self::relayers().iter().position(|x| x == who).is_some()
	}

	/// Checks if a chain exists as a whitelisted destination
	pub fn chain_whitelisted(id: ChainId) -> bool {
		Self::chain_nonces(id) != None
	}
}
