#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod traits;
pub mod types;
mod weights;

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{EnsureOrigin, Get},
	PalletId,
};
use frame_system::ensure_signed;
use sp_core::U256;
use sp_runtime::traits::AccountIdConversion;
use sp_std::prelude::*;

pub use constants::*;
pub use pallet::*;
pub use traits::WeightInfo;
pub use types::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Associated type for Event enum
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;

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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Vote threshold has changed (new_threshold)
		RelayerThresholdChanged { threshold: u32 },
		/// Chain now available for transfers (chain_id)
		ChainWhitelisted { chain_id: ChainId },
		/// Relayer added to set
		RelayerAdded { account: T::AccountId },
		/// Relayer removed from set
		RelayerRemoved { account: T::AccountId },
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
		ProposalApproved { chain_id: ChainId, nonce: DepositNonce },
		/// Voting rejected a proposal
		ProposalRejected { chain_id: ChainId, nonce: DepositNonce },
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
	pub type Relayers<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	/// All known proposals.
	/// The key is the hash of the call and the deposit ID, to ensure it's unique.
	#[pallet::storage]
	#[pallet::getter(fn get_votes)]
	pub type Votes<T: Config> = StorageDoubleMap<
		_,
		Blake2_256,
		ChainId,
		Blake2_256,
		DepositNonce,
		ProposalVotes<T::AccountId, T::BlockNumber>,
		OptionQuery,
	>;

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
			Self::deposit_event(Event::RelayerThresholdChanged { threshold });

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
		#[pallet::weight(<T as Config>::WeightInfo::add_relayer())]
		pub fn add_relayer(
			origin: OriginFor<T>,
			account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::ExternalOrigin::ensure_origin(origin)?;

			Relayers::<T>::mutate(|relayers| {
				let found = relayers.iter().find(|x| **x == account);
				if found.is_none() {
					Relayers::<T>::mutate(|relayers| relayers.push(account.clone()));
					Self::deposit_event(Event::RelayerAdded { account });
				}
			});

			Ok(().into())
		}

		/// Removes an existing relayer from the set.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::remove_relayer())]
		pub fn remove_relayer(
			origin: OriginFor<T>,
			account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::ExternalOrigin::ensure_origin(origin)?;

			Relayers::<T>::mutate(|relayers| {
				let pos = relayers.iter().position(|x| *x == account);
				if let Some(pos) = pos {
					relayers.remove(pos);
					Self::deposit_event(Event::RelayerRemoved { account });
				}
			});

			Ok(().into())
		}

		/// Commits a vote in favour of the provided proposal.
		///
		/// If a proposal with the given nonce and source chain ID does not already exist, it will
		/// be created with an initial vote in favour from the caller.
		#[pallet::weight(1000)]
		pub fn vote_proposal(
			origin: OriginFor<T>,
			nonce: DepositNonce,
			chain_id: ChainId,
			in_favour: bool,
		) -> DispatchResultWithPostInfo {
			let account = ensure_signed(origin)?;
			ensure!(Self::is_relayer(&account), Error::<T>::MustBeRelayer);
			ensure!(Self::chain_whitelisted(chain_id), Error::<T>::ChainNotWhitelisted);

			let now = <frame_system::Pallet<T>>::block_number();
			let threshold = Self::relayer_vote_threshold();
			let mut result = None;

			Votes::<T>::try_mutate(chain_id, nonce, |proposal| -> DispatchResult {
				if let Some(proposal) = proposal {
					ensure!(!proposal.is_complete(), Error::<T>::ProposalAlreadyComplete);
					ensure!(!proposal.is_expired(now), Error::<T>::ProposalExpired);
					ensure!(!proposal.has_voted(&account), Error::<T>::RelayerAlreadyVoted);
					proposal.votes.push((account.clone(), in_favour));
					result = proposal.try_to_complete(threshold);
				} else {
					let lifetime = T::ProposalLifetime::get();
					let mut new_proposal =
						ProposalVotes::new(vec![(account.clone(), in_favour)], now + lifetime);
					result = new_proposal.try_to_complete(threshold);
					*proposal = Some(new_proposal);
				}

				// Send Vote Event
				let event = Event::RelayerVoted { chain_id, nonce, account, in_favour };
				Self::deposit_event(event);

				Ok(())
			})?;

			// Let's see if the proposal is already completed
			if let Some(result) = result {
				match result {
					ProposalStatus::Approved => {
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

	/// Initiates a transfer of a fungible asset out of the chain. This should be called by another
	/// pallet.
	pub fn bridge_funds(dest_id: ChainId, to: Vec<u8>, amount: U256) -> DispatchResult {
		ensure!(Self::chain_whitelisted(dest_id), Error::<T>::ChainNotWhitelisted);

		// Bump nonce
		let nonce = Self::chain_nonces(dest_id).unwrap_or_default() + 1;
		<ChainNonces<T>>::insert(dest_id, nonce);

		Self::deposit_event(Event::FungibleTransfer(dest_id, nonce, amount, to));
		Ok(())
	}
}
