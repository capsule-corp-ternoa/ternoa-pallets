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

use frame_support::{
	pallet_prelude::*,
	traits::{
		Currency,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		Get, StorageVersion,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;
use primitives::{
	common::CompoundFee,
	nfts::{NFTData, NFTId},
};
use sp_runtime::traits::{AccountIdConversion, Saturating};
use ternoa_common::traits::{MarketplaceExt, NFTExt};
use types::{AuctionData, BidderList, DeadlineList};
pub use weights::WeightInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::dispatch::DispatchResultWithPostInfo;
	use primitives::marketplace::MarketplaceId;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
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

		/// Link to the Marketplace pallet.
		type MarketplaceExt: MarketplaceExt<AccountId = Self::AccountId, Balance = BalanceOf<Self>>;

		// Constants
		/// Minimum required length of auction.
		#[pallet::constant]
		type MinAuctionDuration: Get<Self::BlockNumber>;

		/// Maximum permitted length of auction.
		#[pallet::constant]
		type MaxAuctionDuration: Get<Self::BlockNumber>;

		/// Maximum distance between the current block and the start block of an auction.
		#[pallet::constant]
		type MaxAuctionDelay: Get<Self::BlockNumber>;

		/// Grace period to extend auction by if new bid received.
		#[pallet::constant]
		type AuctionGracePeriod: Get<Self::BlockNumber>;

		/// Ending period during which an auction can be extended.
		#[pallet::constant]
		type AuctionEndingPeriod: Get<Self::BlockNumber>;

		/// The auctions pallet id - will be used to generate account id.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Total amount of accounts that can be in the bidder list.
		#[pallet::constant]
		type BidderListLengthLimit: Get<u32>;

		/// Maximum amount of auctions that can be active at the same time.
		#[pallet::constant]
		type ParallelAuctionLimit: Get<u32>;

		/// Maximum number of related automatic auction actions in block.
		#[pallet::constant]
		type ActionsInBlockLimit: Get<u32>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Weight: see `begin_block`.
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut read = 1u64;
			let mut write = 0u64;

			// Lets get all the deadlines
			let mut deadlines = Deadlines::<T>::get();
			let max_actions = T::ActionsInBlockLimit::get();
			let mut actions = 0;

			// As long as we have deadlines (or we hit the wall) to finish we should complete them
			while let Some(nft_id) = deadlines.pop_next(now) {
				let mut auction = match Auctions::<T>::get(nft_id) {
					Some(x) => x,
					None => continue,
				};
				let mut nft = match T::NFTExt::get_nft(nft_id) {
					Some(x) => x,
					None => continue,
				};

				let highest_bid = auction.pop_highest_bid();
				if let Some((new_owner, paid)) = highest_bid {
					// Pay the fee
					let cut = match Self::pay_for_nft(&Self::account_id(), paid, &nft, &auction) {
						Ok(x) => x,
						Err(_x) => continue,
					};

					// Handle bidders
					read += auction.get_bidders().iter().count() as u64;
					auction.for_each_bidder(&|(owner, amount)| Self::add_claim(owner, *amount));

					// Change the owner
					nft.owner = new_owner.clone();

					Self::emit_auction_completed_event(
						nft_id,
						Some(new_owner),
						Some(paid),
						Some(cut),
					)
				} else {
					Self::emit_auction_completed_event(nft_id, None, None, None);
				}

				nft.state.is_listed = false;
				_ = T::NFTExt::set_nft(nft_id, nft);
				Auctions::<T>::remove(nft_id);

				read += 3;
				write += 2;
				actions += 1;

				if actions >= max_actions {
					break
				}
			}

			Deadlines::<T>::set(deadlines);
			T::DbWeight::get().reads_writes(read, write)
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn auctions)]
	pub type Auctions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		NFTId,
		AuctionData<T::AccountId, T::BlockNumber, BalanceOf<T>, T::BidderListLengthLimit>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn deadlines)]
	pub type Deadlines<T: Config> =
		StorageValue<_, DeadlineList<T::BlockNumber, T::ParallelAuctionLimit>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn claims)]
	pub type Claims<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new auction was created.
		AuctionCreated {
			nft_id: NFTId,
			marketplace_id: MarketplaceId,
			creator: T::AccountId,
			start_price: BalanceOf<T>,
			buy_it_price: Option<BalanceOf<T>>,
			start_block: T::BlockNumber,
			end_block: T::BlockNumber,
		},
		/// An existing auction was cancelled.
		AuctionCancelled { nft_id: NFTId },
		/// An auction has completed and no more bids can be placed.
		AuctionCompleted {
			nft_id: NFTId,
			new_owner: Option<T::AccountId>,
			paid_amount: Option<BalanceOf<T>>,
			marketplace_cut: Option<BalanceOf<T>>,
			royalty_cut: Option<BalanceOf<T>>,
			auctioneer_cut: Option<BalanceOf<T>>,
		},
		/// A new bid was created.
		BidAdded { nft_id: NFTId, bidder: T::AccountId, amount: BalanceOf<T> },
		/// An existing bid was removed.
		BidRemoved { nft_id: NFTId, bidder: T::AccountId, amount: BalanceOf<T> },
		/// An existing bid was updated.
		BidUpdated { nft_id: NFTId, bidder: T::AccountId, amount: BalanceOf<T> },
		/// An existing bid was dropped.
		BidDropped { nft_id: NFTId, bidder: T::AccountId, amount: BalanceOf<T> },
		/// Balance claimed.
		BalanceClaimed { account: T::AccountId, amount: BalanceOf<T> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Operation not allowed because the auction has not started yet.
		AuctionNotStarted,
		/// Operation not allowed because the auction does not exists.
		AuctionDoesNotExist,
		/// Buy-It-Now option is not available.
		AuctionDoesNotSupportBuyItNow,
		/// Auction start block cannot be lower than current block.
		AuctionCannotStartInThePast,
		/// Auction end block cannot be lower than start block.
		AuctionCannotEndBeforeItHasStarted,
		/// Auction duration exceeds the maximum allowed duration.
		AuctionDurationIsTooLong,
		/// Auction duration is lower than the minimum allowed duration.
		AuctionDurationIsTooShort,
		/// Auction start block cannot be exceed the maximum allowed start delay.
		AuctionStartIsTooFarAway,
		/// Buy-it-now price cannot be lower or equal tah the auction start price.
		BuyItPriceCannotBeLessOrEqualThanStartPrice,
		/// The specified bid does not exist.
		BidDoesNotExist,
		/// Auction owner cannot add a bid to his own auction.
		CannotAddBidToYourOwnAuctions,
		/// Auction owner cannot use buy it now feature to his own auction.
		CannotBuyItNowToYourOwnAuctions,
		/// Auction cannot be canceled if the auction has started.
		CannotCancelAuctionInProgress,
		/// Cannot add a bid that is less than the current highest bid.
		CannotBidLessThanTheHighestBid,
		/// Cannot add a bid that is less than the current starting price.
		CannotBidLessThanTheStartingPrice,
		/// Cannot pay the buy-it-now price if a higher bid exists.
		CannotBuyItWhenABidIsHigherThanBuyItPrice,
		/// Cannot remove bid if the auction is soon to end.
		CannotRemoveBidAtTheEndOfAuction,
		/// Cannot end the auction if it was not extended.
		CannotEndAuctionThatWasNotExtended,
		/// Cannot auction NFTs that are listed for sale.
		CannotListListedNFTs,
		/// Cannot auction capsules.
		CannotListCapsulesNFTs,
		/// Cannot list because the NFT secret is not synced.
		CannotListNotSyncedSecretNFTs,
		/// Cannot auction NFTs that are not owned by the caller.
		CannotListNotOwnedNFTs,
		/// Cannot auction delegated NFTs.
		CannotListDelegatedNFTs,
		/// Cannot auction non-created soulbound NFTs.
		CannotListNotCreatedSoulboundNFTs,
		/// Cannot auction rented NFTs.
		CannotListRentedNFTs,
		/// Cannot claim if the claim does not exist.
		ClaimDoesNotExist,
		/// Cannot auction NFTs that do not exit.
		NFTNotFound,
		/// Operation not allowed because the caller is not the owner of the auction.
		NotTheAuctionCreator,
		/// Unknown Marketplace found. This should never happen.
		MarketplaceNotFound,
		/// The Maximum amount of auctions that can be active at the same time has been reached.
		MaximumAuctionsLimitReached,
		/// The Maximum amount of bids for an auction.
		MaximumBidLimitReached,
		/// Operation is not permitted because price cannot cover marketplace fee.
		PriceCannotCoverMarketplaceFee,
		/// Not Allowed To List On MP
		NotAllowedToList,
		/// Cannot end auction without bids
		CannotEndAuctionWithoutBids,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_auction(Deadlines::<T>::get().len() as u32))]
		pub fn create_auction(
			origin: OriginFor<T>,
			nft_id: NFTId,
			marketplace_id: MarketplaceId,
			start_block: T::BlockNumber,
			end_block: T::BlockNumber,
			start_price: BalanceOf<T>,
			buy_it_price: Option<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			ensure!(start_block >= now, Error::<T>::AuctionCannotStartInThePast);
			ensure!(start_block < end_block, Error::<T>::AuctionCannotEndBeforeItHasStarted);

			let duration = end_block.saturating_sub(start_block);
			let buffer = start_block.saturating_sub(now);

			ensure!(duration <= T::MaxAuctionDuration::get(), Error::<T>::AuctionDurationIsTooLong);
			ensure!(
				duration >= T::MinAuctionDuration::get(),
				Error::<T>::AuctionDurationIsTooShort
			);
			ensure!(buffer <= T::MaxAuctionDelay::get(), Error::<T>::AuctionStartIsTooFarAway);

			if let Some(price) = buy_it_price {
				ensure!(
					price > start_price,
					Error::<T>::BuyItPriceCannotBeLessOrEqualThanStartPrice
				);
			}

			// fetch the data of given nftId.
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::CannotListNotOwnedNFTs);
			ensure!(!nft.state.is_listed, Error::<T>::CannotListListedNFTs);
			ensure!(!nft.state.is_capsule, Error::<T>::CannotListCapsulesNFTs);
			ensure!(!nft.state.is_delegated, Error::<T>::CannotListDelegatedNFTs);
			ensure!(
				!(nft.state.is_soulbound && nft.creator != nft.owner),
				Error::<T>::CannotListNotCreatedSoulboundNFTs
			);
			ensure!(
				!(nft.state.is_secret && !nft.state.is_secret_synced),
				Error::<T>::CannotListNotSyncedSecretNFTs
			);

			let marketplace = T::MarketplaceExt::get_marketplace(marketplace_id)
				.ok_or(Error::<T>::MarketplaceNotFound)?;

			marketplace
				.allowed_to_list(&who, nft.collection_id)
				.ok_or(Error::<T>::NotAllowedToList)?;

			// Check if the start price can cover the marketplace commission_fee if it exists.
			if let Some(commission_fee) = &marketplace.commission_fee {
				if let CompoundFee::Flat(flat_commission) = commission_fee {
					ensure!(
						start_price >= *flat_commission,
						Error::<T>::PriceCannotCoverMarketplaceFee
					);
				}
			}

			// Add NFT ID to deadlines
			Deadlines::<T>::try_mutate(|x| -> DispatchResult {
				x.insert(nft_id, end_block)
					.map_err(|_| Error::<T>::MaximumAuctionsLimitReached)?;
				Ok(())
			})?;

			nft.state.is_listed = true;
			T::NFTExt::set_nft(nft_id, nft)?;

			let bidders: BidderList<T::AccountId, BalanceOf<T>, T::BidderListLengthLimit> =
				BidderList::new();
			let auction_data = AuctionData {
				creator: who.clone(),
				start_block,
				end_block,
				start_price,
				buy_it_price,
				bidders,
				marketplace_id,
				is_extended: false,
			};

			Auctions::<T>::insert(nft_id, auction_data);

			// Emit AuctionCreated event.
			let event = Event::AuctionCreated {
				nft_id,
				marketplace_id,
				creator: who,
				start_price,
				buy_it_price,
				start_block,
				end_block,
			};
			Self::deposit_event(event);

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::cancel_auction(Deadlines::<T>::get().len() as u32))]
		pub fn cancel_auction(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			let auction = Auctions::<T>::get(nft_id).ok_or(Error::<T>::AuctionDoesNotExist)?;

			ensure!(auction.is_creator(&who), Error::<T>::NotTheAuctionCreator);
			ensure!(!auction.has_started(now), Error::<T>::CannotCancelAuctionInProgress);

			// Remove bidders
			auction.for_each_bidder(&|(owner, amount)| Self::add_claim(owner, *amount));

			nft.state.is_listed = false;
			T::NFTExt::set_nft(nft_id, nft)?;
			Auctions::<T>::remove(nft_id);
			Deadlines::<T>::mutate(|x| x.remove(nft_id));

			Self::deposit_event(Event::AuctionCancelled { nft_id });

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::end_auction(Deadlines::<T>::get().len() as u32))]
		pub fn end_auction(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			let mut auction = Auctions::<T>::get(nft_id).ok_or(Error::<T>::AuctionDoesNotExist)?;

			ensure!(auction.is_creator(&who), Error::<T>::NotTheAuctionCreator);
			ensure!(auction.is_extended, Error::<T>::CannotEndAuctionThatWasNotExtended);

			let (new_owner, paid) =
				auction.pop_highest_bid().ok_or(Error::<T>::CannotEndAuctionWithoutBids)?;

			let cut = Self::pay_for_nft(&Self::account_id(), paid, &nft, &auction)?;
			auction.for_each_bidder(&|(owner, amount)| Self::add_claim(owner, *amount));

			// Change the owner
			nft.owner = new_owner.clone();
			nft.state.is_listed = false;

			T::NFTExt::set_nft(nft_id, nft)?;
			Auctions::<T>::remove(nft_id);
			Deadlines::<T>::mutate(|x| x.remove(nft_id));

			Self::emit_auction_completed_event(nft_id, Some(new_owner), Some(paid), Some(cut));

			Ok(().into())
		}

		#[pallet::weight((
            {
				let s = Auctions::<T>::get(nft_id).map_or_else(|| 0, |x| x.get_bidders().len());
				T::WeightInfo::add_bid(s as u32)
            },
			DispatchClass::Normal
        ))]
		pub fn add_bid(
			origin: OriginFor<T>,
			nft_id: NFTId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			// add bid to storage.
			Auctions::<T>::try_mutate(nft_id, |maybe_auction| -> DispatchResult {
				let auction = maybe_auction.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

				ensure!(!auction.is_creator(&who), Error::<T>::CannotAddBidToYourOwnAuctions);
				ensure!(auction.has_started(now), Error::<T>::AuctionNotStarted);

				// ensure the bid is larger than the current highest bid.
				if let Some(highest_bid) = auction.get_highest_bid() {
					ensure!(amount > highest_bid.1, Error::<T>::CannotBidLessThanTheHighestBid);
				} else {
					ensure!(
						amount > auction.start_price,
						Error::<T>::CannotBidLessThanTheStartingPrice
					);
				}

				if let Some(existing_bid) = auction.find_bid(&who) {
					let amount_difference = amount.saturating_sub(existing_bid.1);
					T::Currency::transfer(&who, &Self::account_id(), amount_difference, KeepAlive)?;

					auction.remove_bid(&who);
				} else {
					// transfer funds from caller.
					T::Currency::transfer(&who, &Self::account_id(), amount, KeepAlive)?;
				}

				// replace top bidder with caller.
				// if bidder has been removed, refund removed user.
				if let Some(bid) = auction.insert_new_bid(who.clone(), amount) {
					Self::add_claim(&bid.0, bid.1);
					Self::deposit_event(Event::BidDropped { nft_id, bidder: bid.0, amount: bid.1 });
				}

				// extend auction by grace period if in ending period.
				let grace_period = T::AuctionGracePeriod::get();
				if let Some(new_end_block) = auction.extend_if_necessary(now, grace_period) {
					Deadlines::<T>::mutate(|x| x.update(nft_id, new_end_block));
				}
				Ok(())
			})?;

			Self::deposit_event(Event::BidAdded { nft_id, bidder: who, amount });

			Ok(().into())
		}

		#[pallet::weight((
            {
				let s = Auctions::<T>::get(nft_id).map_or_else(|| 0, |x| x.get_bidders().len());
				T::WeightInfo::remove_bid(s as u32)
            },
			DispatchClass::Normal
        ))]
		pub fn remove_bid(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			// remove bid from storage.
			Auctions::<T>::try_mutate(nft_id, |maybe_auction| -> DispatchResult {
				// should not panic when unwrap since already checked above.
				let auction = maybe_auction.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

				let remaining_blocks = auction.end_block.saturating_sub(now);
				// ensure the auction period has not ended.
				ensure!(
					remaining_blocks > T::AuctionEndingPeriod::get(),
					Error::<T>::CannotRemoveBidAtTheEndOfAuction
				);

				let bid = auction.find_bid(&who).ok_or(Error::<T>::BidDoesNotExist)?.clone();
				T::Currency::transfer(&Self::account_id(), &bid.0, bid.1, AllowDeath)?;

				auction.remove_bid(&who);
				Self::deposit_event(Event::BidRemoved { nft_id, bidder: who, amount: bid.1 });

				Ok(())
			})?;

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::buy_it_now(Deadlines::<T>::get().len() as u32))]
		pub fn buy_it_now(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = frame_system::Pallet::<T>::block_number();

			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			let auction = Auctions::<T>::get(nft_id).ok_or(Error::<T>::AuctionDoesNotExist)?;
			let paid_amount =
				auction.buy_it_price.ok_or(Error::<T>::AuctionDoesNotSupportBuyItNow)?;

			ensure!(!auction.is_creator(&who), Error::<T>::CannotBuyItNowToYourOwnAuctions);
			ensure!(auction.has_started(now), Error::<T>::AuctionNotStarted);
			if let Some(bid) = auction.get_highest_bid() {
				ensure!(paid_amount > bid.1, Error::<T>::CannotBuyItWhenABidIsHigherThanBuyItPrice);
			}

			// Pay for NFT
			let cut = Self::pay_for_nft(&who, paid_amount, &nft, &auction)?;
			// Handle Bidders
			auction.for_each_bidder(&|(owner, amount)| Self::add_claim(owner, *amount));

			nft.owner = who.clone();
			nft.state.is_listed = false;

			T::NFTExt::set_nft(nft_id, nft)?;
			Auctions::<T>::remove(nft_id);
			Deadlines::<T>::mutate(|x| x.remove(nft_id));

			Self::emit_auction_completed_event(nft_id, Some(who), Some(paid_amount), Some(cut));

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::claim())]
		pub fn claim(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let claim = Claims::<T>::get(&who).ok_or(Error::<T>::ClaimDoesNotExist)?;

			T::Currency::transfer(&Self::account_id(), &who, claim, AllowDeath)?;
			Claims::<T>::remove(&who);

			let event = Event::BalanceClaimed { account: who, amount: claim };
			Self::deposit_event(event);

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The account ID of the auctions pot.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	pub fn pay_for_nft(
		from: &T::AccountId,
		amount: BalanceOf<T>,
		nft: &NFTData<T::AccountId, <<T as Config>::NFTExt as NFTExt>::NFTOffchainDataLimit>,
		auction: &AuctionData<T::AccountId, T::BlockNumber, BalanceOf<T>, T::BidderListLengthLimit>,
	) -> Result<(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>), DispatchError> {
		let nft_creator = &nft.creator;
		let nft_royalty = nft.royalty;
		let auction_creator = &auction.creator;
		let marketplace_id = auction.marketplace_id;

		let marketplace = T::MarketplaceExt::get_marketplace(marketplace_id)
			.ok_or(Error::<T>::MarketplaceNotFound)?;

		let commission_fee_amount = marketplace.commission_fee.map_or_else(
			|| 0u32.into(),
			|x| match x {
				CompoundFee::Flat(x) => x,
				CompoundFee::Percentage(x) => x * amount,
			},
		);

		let to_marketplace = commission_fee_amount;
		let to_nft_creator = nft_royalty * amount.saturating_sub(to_marketplace);
		let to_auction_creator =
			amount.saturating_sub(to_marketplace).saturating_sub(to_nft_creator);

		let exist = if from == &Self::account_id() { AllowDeath } else { KeepAlive };
		T::Currency::transfer(from, &marketplace.owner, to_marketplace, exist)?;
		T::Currency::transfer(from, nft_creator, to_nft_creator, exist)?;
		T::Currency::transfer(from, auction_creator, to_auction_creator, exist)?;

		Ok((to_marketplace, to_nft_creator, to_auction_creator))
	}

	pub fn add_claim(account: &T::AccountId, amount: BalanceOf<T>) {
		Claims::<T>::mutate(account, |x| {
			*x = Some(x.unwrap_or(0u32.into()).saturating_add(amount));
		})
	}

	pub fn has_started(now: T::BlockNumber, start_block: T::BlockNumber) -> bool {
		now >= start_block
	}

	pub fn emit_auction_completed_event(
		nft_id: NFTId,
		new_owner: Option<T::AccountId>,
		paid_amount: Option<BalanceOf<T>>,
		cut: Option<(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>)>,
	) {
		Self::deposit_event(Event::AuctionCompleted {
			nft_id,
			new_owner,
			paid_amount,
			marketplace_cut: cut.and_then(|x| Some(x.0)),
			royalty_cut: cut.and_then(|x| Some(x.1)),
			auctioneer_cut: cut.and_then(|x| Some(x.2)),
		});
	}

	/// Fill Deadline queue with any number of data
	pub fn fill_deadline_queue(
		number: u32,
		nft_id: NFTId,
		block_number: T::BlockNumber,
	) -> Result<(), DispatchError> {
		Deadlines::<T>::try_mutate(|x| -> DispatchResult {
			for _i in 0..number {
				x.insert(nft_id, block_number)
					.map_err(|_| Error::<T>::MaximumAuctionsLimitReached)?;
			}
			Ok(())
		})?;
		Ok(())
	}

	/// Fill the bidders list of an auction
	pub fn fill_bidders_list(
		number: u32,
		nft_id: NFTId,
		account: T::AccountId,
		amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		Auctions::<T>::try_mutate(nft_id, |x| -> DispatchResult {
			let auction = x.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;
			for _i in 0..number {
				auction
					.bidders
					.list
					.try_push((account.clone(), amount))
					.map_err(|_| Error::<T>::MaximumBidLimitReached)?;
			}
			Ok(())
		})?;
		Ok(())
	}
}
