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
	pallet_prelude::*,
	traits::{
		Currency,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		Get, StorageVersion,
	},
	PalletId,
};
use primitives::nfts::NFTId;
use sp_runtime::traits::{AccountIdConversion, Saturating};
use ternoa_common::traits::{MarketplaceExt, NFTExt};
use types::{AuctionData, AuctionsGenesis, BidderList, DeadlineList};
pub use weights::WeightInfo;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, transactional};
	use frame_system::{ensure_root, pallet_prelude::*, RawOrigin};
	use primitives::marketplace::MarketplaceId;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
		type MarketplaceExt: MarketplaceExt<AccountId = Self::AccountId>;

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
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Weight: see `begin_block`
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut read = 0;
			let mut write = 0;

			loop {
				let deadlines = Deadlines::<T>::get();
				read += 1;

				if let Some(nft_id) = deadlines.next(now) {
					let ok = Self::complete_auction(RawOrigin::Root.into(), nft_id);
					debug_assert_eq!(ok, Ok(().into()));
				} else {
					break
				}

				read += 1;
				write += 1;
			}

			if write == 0 {
				T::DbWeight::get().reads(read)
			} else {
				T::DbWeight::get().reads_writes(read, write)
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(T::WeightInfo::create_auction())]
		#[transactional]
		pub fn create_auction(
			origin: OriginFor<T>,
			nft_id: NFTId,
			marketplace_id: MarketplaceId,
			#[pallet::compact] start_block: T::BlockNumber,
			#[pallet::compact] end_block: T::BlockNumber,
			start_price: BalanceOf<T>,
			buy_it_price: Option<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;
			let current_block = frame_system::Pallet::<T>::block_number();

			ensure!(start_block >= current_block, Error::<T>::AuctionCannotStartInThePast);
			ensure!(start_block < end_block, Error::<T>::AuctionCannotEndBeforeItHasStarted);

			let duration = end_block.saturating_sub(start_block);
			let buffer = start_block.saturating_sub(current_block);

			ensure!(duration <= T::MaxAuctionDuration::get(), Error::<T>::AuctionDurationIsTooLong);
			ensure!(
				duration >= T::MinAuctionDuration::get(),
				Error::<T>::AuctionDurationIsTooShort
			);
			ensure!(buffer <= T::MaxAuctionDelay::get(), Error::<T>::AuctionStartIsTooFarAway);

			if let Some(price) = buy_it_price {
				ensure!(
					price > start_price,
					Error::<T>::BuyItPriceCannotBeLowerOrEqualThanStartPrice
				);
			}

			// fetch the data of given nftId
			let nft_data = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTDoesNotExist)?;
			let is_nft_in_completed_series = T::NFTExt::is_nft_in_completed_series(nft_id);

			ensure!(nft_data.owner == creator.clone(), Error::<T>::CannotAuctionNotOwnedNFTs);
			ensure!(nft_data.listed_for_sale == false, Error::<T>::CannotAuctionNFTsListedForSale);
			ensure!(
				nft_data.is_in_transmission == false,
				Error::<T>::CannotAuctionNFTsInTransmission
			);
			ensure!(nft_data.is_capsule == false, Error::<T>::CannotAuctionCapsules);
			ensure!(!nft_data.is_delegated, Error::<T>::CannotAuctionDelegatedNFTs);
			ensure!(
				is_nft_in_completed_series == Some(true),
				Error::<T>::CannotAuctionNFTsInUncompletedSeries
			);

			T::MarketplaceExt::is_allowed_to_list(marketplace_id, creator.clone())?;
			T::NFTExt::set_listed_for_sale(nft_id, true)?;

			let bidders: BidderList<T::AccountId, BalanceOf<T>, T::BidderListLengthLimit> =
				BidderList::new();
			let auction_data = AuctionData {
				creator: creator.clone(),
				start_block,
				end_block,
				start_price,
				buy_it_price,
				bidders,
				marketplace_id,
				is_extended: false,
			};

			// Add auction to storage and insert an entry to deadlines
			Deadlines::<T>::mutate(|x| -> DispatchResult {
				x.insert(nft_id, end_block)
					.map_err(|_| Error::<T>::MaximumAuctionsLimitReached)?;
				Ok(())
			})?;
			Auctions::<T>::insert(nft_id, auction_data);

			// Emit AuctionCreated event
			let event = Event::AuctionCreated {
				nft_id,
				marketplace_id,
				creator,
				start_price,
				buy_it_price,
				start_block,
				end_block,
			};
			Self::deposit_event(event);

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::cancel_auction())]
		#[transactional]
		pub fn cancel_auction(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let current_block = frame_system::Pallet::<T>::block_number();

			let auction = Auctions::<T>::get(nft_id).ok_or(Error::<T>::AuctionDoesNotExist)?;

			ensure!(auction.creator == who, Error::<T>::NotTheAuctionCreator);
			ensure!(
				!Self::has_started(current_block, auction.start_block),
				Error::<T>::CannotCancelAuctionInProgress
			);

			T::NFTExt::set_listed_for_sale(nft_id, false)?;
			Self::remove_auction(nft_id, &auction);

			Self::deposit_event(Event::AuctionCancelled { nft_id });

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::end_auction())]
		#[transactional]
		pub fn end_auction(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let auction = Auctions::<T>::get(nft_id).ok_or(Error::<T>::AuctionDoesNotExist)?;

			ensure!(auction.creator == who, Error::<T>::NotTheAuctionCreator);
			ensure!(auction.is_extended, Error::<T>::CannotEndAuctionThatWasNotExtended);

			Self::complete_auction(RawOrigin::Root.into(), nft_id)?;

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::add_bid())]
		#[transactional]
		pub fn add_bid(
			origin: OriginFor<T>,
			nft_id: NFTId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let current_block = frame_system::Pallet::<T>::block_number();

			// add bid to storage
			Auctions::<T>::try_mutate(nft_id, |maybe_auction| -> DispatchResult {
				let auction = maybe_auction.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

				// ensure the caller is not the owner of NFT
				ensure!(auction.creator != who.clone(), Error::<T>::CannotAddBidToYourOwnAuctions);

				// ensure the auction period has commenced
				ensure!(
					Self::has_started(current_block, auction.start_block),
					Error::<T>::AuctionNotStarted
				);

				// ensure the bid is larger than the current highest bid
				if let Some(highest_bid) = auction.bidders.get_highest_bid() {
					ensure!(amount > highest_bid.1, Error::<T>::CannotBidLessThanTheHighestBid);
				} else {
					// ensure the bid amount is greater than start price
					ensure!(
						amount > auction.start_price,
						Error::<T>::CannotBidLessThanTheStartingPrice
					);
				}
				let remaining_blocks = auction.end_block.saturating_sub(current_block);

				if let Some(existing_bid) = auction.bidders.find_bid(who.clone()) {
					let amount_difference = amount.saturating_sub(existing_bid.1);
					T::Currency::transfer(&who, &Self::account_id(), amount_difference, KeepAlive)?;

					auction.bidders.remove_bid(who.clone());
				} else {
					// transfer funds from caller
					T::Currency::transfer(&who, &Self::account_id(), amount, KeepAlive)?;
				}

				// replace top bidder with caller
				// if bidder has been removed, refund removed user
				if let Some(bid) = auction.bidders.insert_new_bid(who.clone(), amount) {
					Self::add_claim(&bid.0, bid.1);
				}

				let grace_period = T::AuctionGracePeriod::get();
				// extend auction by grace period if in ending period
				if remaining_blocks < grace_period {
					let blocks_to_add = grace_period.saturating_sub(remaining_blocks);

					auction.end_block = auction.end_block.saturating_add(blocks_to_add);
					auction.is_extended = true;

					// Update deadline
					Deadlines::<T>::mutate(|x| x.update(nft_id, auction.end_block));
				}

				Ok(())
			})?;

			Self::deposit_event(Event::BidAdded { nft_id, bidder: who, amount });

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::remove_bid())]
		#[transactional]
		pub fn remove_bid(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let current_block = frame_system::Pallet::<T>::block_number();

			// remove bid from storage
			Auctions::<T>::try_mutate(nft_id, |maybe_auction| -> DispatchResult {
				// should not panic when unwrap since already checked above
				let auction = maybe_auction.as_mut().ok_or(Error::<T>::AuctionDoesNotExist)?;

				let remaining_blocks = auction.end_block.saturating_sub(current_block);
				// ensure the auction period has not ended
				ensure!(
					remaining_blocks > T::AuctionEndingPeriod::get(),
					Error::<T>::CannotRemoveBidAtTheEndOfAuction
				);

				let bid = auction
					.bidders
					.find_bid(who.clone())
					.ok_or(Error::<T>::BidDoesNotExist)?
					.clone();

				T::Currency::transfer(&Self::account_id(), &bid.0, bid.1, AllowDeath)?;

				auction.bidders.remove_bid(who.clone());

				Self::deposit_event(Event::BidRemoved { nft_id, bidder: who, amount: bid.1 });

				Ok(())
			})?;

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::buy_it_now())]
		#[transactional]
		pub fn buy_it_now(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let current_block = frame_system::Pallet::<T>::block_number();

			let auction = Auctions::<T>::get(nft_id).ok_or(Error::<T>::AuctionDoesNotExist)?;
			let amount = auction.buy_it_price.ok_or(Error::<T>::AuctionDoesNotSupportBuyItNow)?;

			// ensure the auction period has commenced
			ensure!(
				Self::has_started(current_block, auction.start_block),
				Error::<T>::AuctionNotStarted
			);

			if let Some(highest_bid) = auction.bidders.get_highest_bid() {
				ensure!(
					amount > highest_bid.1,
					Error::<T>::CannotBuyItWhenABidIsHigherThanBuyItPrice
				);
			}

			Self::close_auction(nft_id, &auction, &who, amount, Some(who.clone()))?;
			Self::remove_auction(nft_id, &auction);

			Self::deposit_event(Event::AuctionCompleted {
				nft_id,
				new_owner: Some(who),
				amount: Some(amount),
			});

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::complete_auction())]
		#[transactional]
		pub fn complete_auction(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let _who = ensure_root(origin)?;

			let mut auction = Auctions::<T>::get(nft_id).ok_or(Error::<T>::AuctionDoesNotExist)?;

			let mut new_owner = None;
			let mut amount = None;
			// assign to highest bidder if exists
			if let Some(bidder) = auction.bidders.remove_highest_bid() {
				new_owner = Some(bidder.0.clone());
				amount = Some(bidder.1.clone());

				Self::close_auction(nft_id, &auction, &bidder.0, bidder.1, None)?;
			}

			Self::remove_auction(nft_id, &auction);

			Self::deposit_event(Event::AuctionCompleted { nft_id, new_owner, amount });

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::claim())]
		#[transactional]
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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new auction was created
		AuctionCreated {
			nft_id: NFTId,
			marketplace_id: MarketplaceId,
			creator: T::AccountId,
			start_price: BalanceOf<T>,
			buy_it_price: Option<BalanceOf<T>>,
			start_block: T::BlockNumber,
			end_block: T::BlockNumber,
		},
		/// An existing auction was cancelled
		AuctionCancelled { nft_id: NFTId },
		/// An auction has completed and no more bids can be placed
		AuctionCompleted {
			nft_id: NFTId,
			new_owner: Option<T::AccountId>,
			amount: Option<BalanceOf<T>>,
		},
		/// A new bid was created
		BidAdded { nft_id: NFTId, bidder: T::AccountId, amount: BalanceOf<T> },
		/// An exising bid was removed
		BidRemoved { nft_id: NFTId, bidder: T::AccountId, amount: BalanceOf<T> },
		/// An exising bid was updated
		BidUpdated { nft_id: NFTId, bidder: T::AccountId, amount: BalanceOf<T> },
		/// Balance claimed
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
		BuyItPriceCannotBeLowerOrEqualThanStartPrice,
		/// The specified bid does not exist.
		BidDoesNotExist,
		/// Auction owner cannot add a bid to his own auction.
		CannotAddBidToYourOwnAuctions,
		/// Auction cannot be canceled if the auction has started.
		CannotCancelAuctionInProgress,
		/// Cannot add a bid that is less than the current highest bid.
		CannotBidLessThanTheHighestBid,
		/// Cannot add a bid that is less than the current starting price.
		CannotBidLessThanTheStartingPrice,
		/// Cannot pay the buy-it-now price if a higher bid exists.
		CannotBuyItWhenABidIsHigherThanBuyItPrice,
		/// Cannot auction NFTs that are in a uncompleted series.
		CannotAuctionNFTsInUncompletedSeries,
		/// Cannot remove bid if the auction is soon to end.
		CannotRemoveBidAtTheEndOfAuction,
		/// Cannot end the auction if it was not extended.
		CannotEndAuctionThatWasNotExtended,
		/// Cannot auction NFTs that are listed for sale.
		CannotAuctionNFTsListedForSale,
		/// Cannot auction NFTs that are in transmission.
		CannotAuctionNFTsInTransmission,
		/// Cannot auction capsules.
		CannotAuctionCapsules,
		/// Cannot auction NFTs that are not owned by the caller.
		CannotAuctionNotOwnedNFTs,
		/// Cannot auction delegated NFTs.
		CannotAuctionDelegatedNFTs,
		/// Cannot claim if the claim does not exist.
		ClaimDoesNotExist,
		/// Cannot auction NFTs that do not exit.
		NFTDoesNotExist,
		/// Operation not allowed because the caller is not the owner of the auction.
		NotTheAuctionCreator,
		/// Unknown Marketplace found. This should never happen.
		MarketplaceNotFound,
		/// The Maximum amount of auctions that can be active at the same time has been reached.
		MaximumAuctionsLimitReached,
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

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub auctions: Vec<AuctionsGenesis<T::AccountId, T::BlockNumber, BalanceOf<T>>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { auctions: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for auction in self.auctions.clone() {
				let nft_id = auction.0;
				Deadlines::<T>::mutate(|x| {
					x.insert(nft_id, auction.3)
						.map_err(|_| Error::<T>::MaximumAuctionsLimitReached)
						.expect("It will never happen.");
				});

				let bidders = BidderList::from_raw(auction.6);
				let data = AuctionData {
					creator: auction.1,
					start_block: auction.2,
					end_block: auction.3,
					start_price: auction.4,
					buy_it_price: auction.5,
					bidders,
					marketplace_id: auction.7,
					is_extended: auction.8,
				};
				Auctions::<T>::insert(nft_id, data);
			}
		}
	}
}

#[allow(dead_code)]
impl<T: Config> Pallet<T> {
	/// The account ID of the auctions pot.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	pub fn close_auction(
		nft_id: NFTId,
		auction: &AuctionData<T::AccountId, T::BlockNumber, BalanceOf<T>, T::BidderListLengthLimit>,
		new_owner: &T::AccountId,
		price: BalanceOf<T>,
		balance_source: Option<T::AccountId>,
	) -> DispatchResult {
		// Handle marketplace fees
		let marketplace = T::MarketplaceExt::get_marketplace(auction.marketplace_id)
			.ok_or(Error::<T>::MarketplaceNotFound)?;

		let to_marketplace =
			price.saturating_mul(marketplace.commission_fee.into()) / 100u32.into();
		let to_auctioneer = price.saturating_sub(to_marketplace);

		let existence = if balance_source.is_none() { KeepAlive } else { AllowDeath };
		let balance_source = balance_source.unwrap_or_else(|| Self::account_id());

		// Transfer fee to marketplace
		T::Currency::transfer(&balance_source, &marketplace.owner, to_marketplace, existence)?;

		// Transfer remaining to auction creator
		T::Currency::transfer(&balance_source, &auction.creator, to_auctioneer, existence)?;

		T::NFTExt::set_owner(nft_id, new_owner)?;
		T::NFTExt::set_listed_for_sale(nft_id, false)?;

		Ok(())
	}

	pub fn remove_auction(
		nft_id: NFTId,
		auction: &AuctionData<T::AccountId, T::BlockNumber, BalanceOf<T>, T::BidderListLengthLimit>,
	) {
		Deadlines::<T>::mutate(|x| x.remove(nft_id));

		for bidder in auction.bidders.list.iter() {
			Self::add_claim(&bidder.0, bidder.1);
		}

		Auctions::<T>::remove(nft_id);
	}

	pub fn add_claim(account: &T::AccountId, amount: BalanceOf<T>) {
		Claims::<T>::mutate(account, |x| {
			if let Some(claim) = x {
				claim.saturating_add(amount);
			} else {
				*x = Some(amount);
			}
		})
	}

	pub fn has_started(now: T::BlockNumber, start_block: T::BlockNumber) -> bool {
		now >= start_block
	}
}
