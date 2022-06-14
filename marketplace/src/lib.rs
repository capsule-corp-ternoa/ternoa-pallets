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
	dispatch::{DispatchErrorWithPostInfo, DispatchResult},
	ensure,
	pallet_prelude::DispatchResultWithPostInfo,
	traits::{
		Currency, ExistenceRequirement::KeepAlive, Get, OnUnbalanced, StorageVersion,
		WithdrawReasons,
	},
	transactional, BoundedVec,
};
use frame_system::{pallet_prelude::*, Origin};
use sp_runtime::traits::{CheckedDiv, CheckedSub, StaticLookup};
use sp_std::{cmp::max, prelude::*};

use primitives::{
	marketplace::{MarketplaceData, MarketplaceFee, MarketplaceId, MarketplaceType},
	nfts::NFTId,
	ConfigOp, U8BoundedVec,
};
use ternoa_common::traits::NFTExt;
use weights::WeightInfo;

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

		/// Place where the marketplace fees go.
		type FeesCollector: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Link to the NFT pallet.
		type NFTExt: NFTExt<AccountId = Self::AccountId>;

		// Constants
		/// Default fee for minting Marketplaces.
		#[pallet::constant]
		type InitialMintFee: Get<BalanceOf<Self>>;

		/// The maximum number of accounts that can be stored inside the account list.
		#[pallet::constant]
		type AccountSizeLimit: Get<u32>;

		/// Maximum offchain data length.
		#[pallet::constant]
		type OffchainDataLimit: Get<u32>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// How much does it cost to create a marketplace.
	#[pallet::storage]
	#[pallet::getter(fn marketplace_mint_fee)]
	pub type MarketplaceMintFee<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, T::InitialMintFee>;

	/// Counter for marketplace ids.
	#[pallet::storage]
	#[pallet::getter(fn next_marketplace_id)]
	pub type NextMarketplaceId<T: Config> = StorageValue<_, MarketplaceId, ValueQuery>;

	/// Data related to marketplaces
	#[pallet::storage]
	#[pallet::getter(fn marketplaces)]
	pub type Marketplaces<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		MarketplaceId,
		MarketplaceData<T::AccountId, BalanceOf<T>, T::AccountSizeLimit, T::OffchainDataLimit>,
		OptionQuery,
	>;

	/// Data related to sales
	#[pallet::storage]
	#[pallet::getter(fn nft_for_sale)]
	pub type NftsForSale<T: Config> =
		StorageMap<_, Blake2_128Concat, NFTId, Sale<T::AccountId, BalanceOf<T>>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Marketplace created
		MarketplaceCreated {
			marketplace_id: MarketplaceId,
			owner: T::AccountId,
			kind: MarketplaceType,
			commission_fee: Option<MarketplaceFee<BalanceOf<T>>>,
			listing_fee: Option<MarketplaceFee<BalanceOf<T>>>,
			account_list: Option<BoundedVec<T::AccountId, T::AccountSizeLimit>>,
			offchain_data: Option<U8BoundedVec<T::OffchainDataLimit>>,
		},
		/// Marketplace owner set
		MarketplaceOwnerSet { marketplace_id: MarketplaceId, owner: T::AccountId },
		/// Marketplace kind set
		MarketplaceKindSet { marketplace_id: MarketplaceId, kind: MarketplaceType },
		/// Marketplace config set
		MarketplaceConfigSet {
			marketplace_id: MarketplaceId,
			commission_fee: Option<MarketplaceFee<BalanceOf<T>>>,
			listing_fee: Option<MarketplaceFee<BalanceOf<T>>>,
			account_list: Option<BoundedVec<T::AccountId, T::AccountSizeLimit>>,
			offchain_data: Option<U8BoundedVec<T::OffchainDataLimit>>,
		},
		/// Marketplace mint fee set
		MarketplaceMintFeeSet { fee: BalanceOf<T> },
		/// Nft listed
		NftListed {
			nft_id: NFTId,
			marketplace_id: MarketplaceId,
			price: BalanceOf<T>,
			commission_fee: Option<MarketplaceFee<BalanceOf<T>>>,
		},
		/// Nft unlisted
		NftUnlisted { nft_id: NFTId },
		/// Nft sold
		NftSold {
			buyer: T::AccountId,
			nft_id: NFTId,
			marketplace_id: MarketplaceId,
			price: BalanceOf<T>,
			commission_fee: Option<MarketplaceFee<BalanceOf<T>>>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account not allowed to list NFTs on that marketplace.
		AccountNotAllowedToList,
		/// Cannot list delegated NFTs.
		CannotListDelegatedNFTs,
		/// Cannot list capsule NFTs.
		CannotListCapsuleNFTs,
		/// Cannot list soulbound NFTs.
		CannotListSoulboundNFTs,
		/// Cannot buy owned NFT
		CannotBuyOwnedNFT,
		/// Sender is already the marketplace owner
		CannotTransferMarketplaceToYourself,
		/// Not enough funds for listing fee
		NotEnoughFundsForListingFee,
		/// The selected price is too low for commission fee
		PriceTooLowForCommissionFee,
		/// NFT already listed
		NFTAlreadyListed,
		/// Marketplace not found
		MarketplaceNotFound,
		/// NFT not found
		NFTNotFound,
		/// This function can only be called by the owner of the NFT.
		NotTheNFTOwner,
		/// This function can only be called by the owner of the marketplace.
		NotTheMarketplaceOwner,
		/// NFT is not for sale
		NFTNotForSale,
		/// Marketplaces data are full
		MarketpalceIdOverflow,
		/// Math operations errors
		InternalMathError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new marketplace with the provided details. An ID will be auto
		/// generated and logged as an event, The caller of this function
		/// will become the owner of the new marketplace.
		#[pallet::weight(T::WeightInfo::create_marketplace())]
		// have to be transactional otherwise we could make people pay the mint fee
		// even if the creation fails.
		#[transactional]
		pub fn create_marketplace(
			origin: OriginFor<T>,
			kind: MarketplaceType,
			commission_fee: Option<MarketplaceFee<BalanceOf<T>>>,
			listing_fee: Option<MarketplaceFee<BalanceOf<T>>>,
			offchain_data: Option<U8BoundedVec<T::OffchainDataLimit>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Checks
			// The Caller needs to pay the Marketplace Mint fee.
			let mint_fee = MarketplaceMintFee::<T>::get();
			let reason = WithdrawReasons::FEE;
			let imbalance = T::Currency::withdraw(&who, mint_fee, reason, KeepAlive)?;
			T::FeesCollector::on_unbalanced(imbalance);

			let marketplace_id = Self::get_next_marketplace_id();
			let marketplace = MarketplaceData::new(
				who.clone(),
				kind,
				commission_fee,
				listing_fee,
				None,
				offchain_data.clone(),
			);

			//execute
			Marketplaces::<T>::insert(marketplace_id, marketplace);
			let event = Event::MarketplaceCreated {
				marketplace_id,
				owner: who,
				kind,
				commission_fee,
				listing_fee,
				account_list: None,
				offchain_data,
			};
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Transfer the ownership of the marketplace to the recipient. Must be called by the
		/// owner of the marketplace.
		#[pallet::weight(T::WeightInfo::create_marketplace())]
		pub fn set_marketplace_owner(
			origin: OriginFor<T>,
			marketplace_id: MarketplaceId,
			recipient: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let recipient = T::Lookup::lookup(recipient)?;

			Marketplaces::<T>::try_mutate(marketplace_id, |x| -> DispatchResult {
				// Checks
				let marketplace = x.as_mut().ok_or(Error::<T>::MarketplaceNotFound)?;
				ensure!(marketplace.owner == who, Error::<T>::NotTheMarketplaceOwner);
				ensure!(recipient.clone() == who, Error::<T>::CannotTransferMarketplaceToYourself);

				// Execute
				marketplace.owner = recipient.clone();
				Ok(())
			})?;

			let event = Event::MarketplaceOwnerSet { marketplace_id, owner: recipient };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Change the kind of the marketplace, can be private or public.
		/// Must be called by the owner of the marketplace.
		#[pallet::weight(T::WeightInfo::create_marketplace())]
		pub fn set_marketplace_kind(
			origin: OriginFor<T>,
			marketplace_id: MarketplaceId,
			kind: MarketplaceType,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Marketplaces::<T>::try_mutate(marketplace_id, |x| -> DispatchResult {
				// Checks
				let marketplace = x.as_mut().ok_or(Error::<T>::MarketplaceNotFound)?;
				ensure!(marketplace.owner == who, Error::<T>::NotTheMarketplaceOwner);

				// Execute
				marketplace.kind = kind;
				Ok(())
			})?;

			let event = Event::MarketplaceKindSet { marketplace_id, kind };
			Self::deposit_event(event);

			Ok(().into())
		}

		// /// Set the configuration parameters of the marketplace (eg. commission_fee, listing_fee,
		// /// account_list, offchain_data). Must be called by the owner of the marketplace.
		// #[pallet::weight(max(
		// 	T::WeightInfo::create_marketplace(),//all set
		// 	T::WeightInfo::create_marketplace(),//all remove
		// ))]
		// pub fn set_marketplace_configuration(
		// 	origin: OriginFor<T>,
		// 	marketplace_id: MarketplaceId,
		// 	commission_fee: ConfigOp<MarketplaceFee>,
		// 	listing_fee: ConfigOp<MarketplaceFee>,
		// 	account_list: ConfigOp<BoundedVec<T::AccountId, T::AccountSizeLimit>>,
		// 	offchain_data: ConfigOp<BoundedVec<u8, T::OffchainDataLimit>>
		// ) -> DispatchResultWithPostInfo {
		// 	let who = ensure_signed(origin)?;

		// 	//??????
		// 	macro_rules! config_op_exp {
		//         ($object:ident, $field:ident, $op:ident) => {
		//             match $op {
		//                 ConfigOp::Noop => (),
		//                 ConfigOp::Set(v) => $object[$field] = (Some(v)), // ???
		//                 ConfigOp::Remove => $object.$field = (None), // ???
		//             }
		//         };
		//     }

		// 	Marketplaces::<T>::try_mutate(marketplace_id, |x| -> DispatchResult {
		// 		let marketplace = x.as_mut().ok_or(Error::<T>::MarketplaceNotFound)?;
		// 		ensure!(marketplace.owner == who, Error::<T>::NotTheMarketplaceOwner);

		// 		config_op_exp!(marketplace, "commission_fee", commission_fee);
		// 		Ok(())
		// 	})?;

		// 	let event = Event::MarketplaceKindSet { marketplace_id, kind };
		// 	Self::deposit_event(event);

		// 	Ok(().into())
		// }

		/// Set the fee for minting a marketplace if the caller is root.
		#[pallet::weight(T::WeightInfo::create_marketplace())]
		pub fn set_marketplace_mint_fee(
			origin: OriginFor<T>,
			fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			MarketplaceMintFee::<T>::put(fee);
			Self::deposit_event(Event::MarketplaceMintFeeSet { fee });

			Ok(().into())
		}

		/// Put an NFT on sale on a marketplace
		#[pallet::weight(T::WeightInfo::create_marketplace())]
		#[transactional]
		pub fn list_nft(
			origin: OriginFor<T>,
			nft_id: NFTId,
			price: BalanceOf<T>,
			marketplace_id: MarketplaceId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Checks
			let nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(!nft.state.listed_for_sale, Error::<T>::NFTAlreadyListed);
			ensure!(!nft.state.is_capsule, Error::<T>::CannotListCapsuleNFTs);
			ensure!(!nft.state.is_delegated, Error::<T>::CannotListDelegatedNFTs);
			ensure!(!nft.state.is_soulbound, Error::<T>::CannotListSoulboundNFTs);

			let marketplace =
				Marketplaces::<T>::get(marketplace_id).ok_or(Error::<T>::MarketplaceNotFound)?;

			// Check if the user is allowed to list on this marketplace
			let mut is_allowed_to_list = false;
			if marketplace.kind == MarketplaceType::Private {
				if let Some(account_list) = &marketplace.account_list {
					is_allowed_to_list = account_list.contains(&who);
				}
			} else {
				if let Some(account_list) = &marketplace.account_list {
					is_allowed_to_list = !account_list.contains(&who);
				} else {
					is_allowed_to_list = true;
				}
			}
			ensure!(is_allowed_to_list, Error::<T>::AccountNotAllowedToList);

			// Check if the selected price can cover the marketplace commission_fee if it exists
			if let Some(commission_fee) = &marketplace.commission_fee {
				if let MarketplaceFee::Flat(flat_commission) = commission_fee {
					ensure!(price > *flat_commission, Error::<T>::PriceTooLowForCommissionFee);
				}
			}

			// The Caller needs to pay the listing fee if it exists.
			let mut listing_fee_value: BalanceOf<T> = 0u32.into();
			if let Some(listing_fee) = &marketplace.listing_fee {
				match *listing_fee {
					MarketplaceFee::Flat(flat_listing_fee) => {
						listing_fee_value = flat_listing_fee;
					},
					MarketplaceFee::Percentage(percentage_listing_fee) => {
						listing_fee_value = percentage_listing_fee * price;
					},
				}
				if listing_fee_value > 0u32.into() {
					let reason = WithdrawReasons::FEE;
					let imbalance = T::Currency::withdraw(&who, listing_fee_value, reason, KeepAlive)?;
					T::FeesCollector::on_unbalanced(imbalance);
				}
			}

			// Execute
			let sale = Sale::new(who, marketplace_id, price, marketplace.commission_fee);
			NftsForSale::<T>::insert(nft_id, sale);
			T::NFTExt::set_nft_state(
				nft_id,
				nft.state.is_capsule,
				true,
				nft.state.is_secret,
				nft.state.is_delegated,
				nft.state.is_soulbound,
			)?;

			let event = Event::NftListed {
				nft_id,
				marketplace_id,
				price,
				commission_fee: marketplace.commission_fee,
			};
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Remove an NFT from sale
		#[pallet::weight(T::WeightInfo::create_marketplace())]
		pub fn unlist_nft(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;

			// Checks
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(NftsForSale::<T>::contains_key(nft_id), Error::<T>::NFTNotForSale);

			// Execute
			T::NFTExt::set_nft_state(
				nft_id,
				nft.state.is_capsule,
				false,
				nft.state.is_secret,
				nft.state.is_delegated,
				nft.state.is_soulbound,
			)?;
			NftsForSale::<T>::remove(nft_id);
			Self::deposit_event(Event::NftUnlisted { nft_id });

			Ok(().into())
		}

		/// Buy a listed nft
		#[pallet::weight(T::WeightInfo::create_marketplace())]
		#[transactional]
		pub fn buy_nft(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			let sale = NftsForSale::<T>::get(nft_id).ok_or(Error::<T>::NFTNotForSale)?;
			let commission_fee = sale.commission_fee;
			let price = sale.price;

			// Checks
			ensure!(sale.account_id != who, Error::<T>::CannotBuyOwnedNFT);

			
			let mut commission_fee_value: BalanceOf<T> = 0u32.into();
			if let Some(commission_fee) = &sale.commission_fee {
				match *commission_fee {
					MarketplaceFee::Flat(flat_commission_fee) => {
						commission_fee_value = flat_commission_fee;
					},
					MarketplaceFee::Percentage(percentage_commission_fee) => {
						commission_fee_value = percentage_commission_fee * price;
					},
				}
				if commission_fee_value > 0u32.into() {
					let reason = WithdrawReasons::FEE;
					let imbalance = T::Currency::withdraw(&who, commission_fee_value, reason, KeepAlive)?;
					T::FeesCollector::on_unbalanced(imbalance);
				}
			}








			// // KeepAlive because they need to be able to use the NFT later on
			// if commission_fee != 0 {
			// 	let tmp = 100u8.checked_div(commission_fee).ok_or(Error::<T>::InternalMathError)?;

			// 	let fee = price.checked_div(&(tmp.into())).ok_or(Error::<T>::InternalMathError)?;

			// 	price = price.checked_sub(&fee).ok_or(Error::<T>::InternalMathError)?;

			// 	T::Currency::transfer(&caller, &market.owner, fee, KeepAlive)?;
			// }

			// T::Currency::transfer(&caller, &sale.account_id, price, KeepAlive)?;

			// T::NFTExt::set_listed_for_sale(nft_id, false)?;
			// T::NFTExt::set_owner(nft_id, &caller)?;

			// NftsForSale::<T>::remove(nft_id);

			// let event = Event::NFTSold { nft_id, owner: caller };
			// Self::deposit_event(event);

			Ok(().into())
		}
	}
}

// impl<T: Config> MarketplaceExt for Pallet<T> {
// 	type AccountId = T::AccountId;
// 	type AccountCountLimit = T::AccountCountLimit;
// 	type NameLengthLimit = T::NameLengthLimit;
// 	type URILengthLimit = T::URILengthLimit;
// 	type DescriptionLengthLimit = T::DescriptionLengthLimit;

// 	// Return if an account is permitted to list on given marketplace
// 	fn is_allowed_to_list(
// 		marketplace_id: MarketplaceId,
// 		account_id: Self::AccountId,
// 	) -> DispatchResult {
// 		let market =
// 			Marketplaces::<T>::get(marketplace_id).ok_or(Error::<T>::MarketplaceNotFound)?;

// 		if market.kind == MarketplaceType::Private {
// 			let is_on_list = market.allow_list.contains(&account_id);
// 			ensure!(is_on_list, Error::<T>::AccountNotAllowedToList);
// 			Ok(())
// 		} else {
// 			let is_on_list = market.disallow_list.contains(&account_id);
// 			ensure!(!is_on_list, Error::<T>::AccountNotAllowedToList);
// 			Ok(())
// 		}
// 	}

// 	// Return the owner account and commision for marketplace with `marketplace_id`
// 	fn get_marketplace(
// 		marketplace_id: MarketplaceId,
// 	) -> Option<
// 		MarketplaceData<
// 			Self::AccountId,
// 			Self::AccountCountLimit,
// 			Self::NameLengthLimit,
// 			Self::URILengthLimit,
// 			Self::DescriptionLengthLimit,
// 		>,
// 	> {
// 		match Marketplaces::<T>::get(marketplace_id) {
// 			Some(marketplace) => Some(marketplace),
// 			None => None,
// 		}
// 	}

// 	// create a new marketplace
// 	fn create(
// 		caller_id: Self::AccountId,
// 		kind: MarketplaceType,
// 		commission_fee: u8,
// 		name: NameVec<T>,
// 		uri: URIVec<T>,
// 		logo_uri: URIVec<T>,
// 		description: DescriptionVec<T>,
// 	) -> Result<MarketplaceId, DispatchErrorWithPostInfo> {
// 		Self::create_marketplace(
// 			Origin::<T>::Signed(caller_id).into(),
// 			kind,
// 			commission_fee,
// 			name,
// 			uri,
// 			logo_uri,
// 			description,
// 		)?;

// 		Ok(MarketplaceIdGenerator::<T>::get())
// 	}
// }

impl<T: Config> Pallet<T> {
	fn get_next_marketplace_id() -> MarketplaceId {
		let marketplace_id = NextMarketplaceId::<T>::get();
		let next_id = marketplace_id
			.checked_add(1)
			.expect("If u32 is not enough we should crash for safety; qed.");
		NextMarketplaceId::<T>::put(next_id);

		marketplace_id
	}
}