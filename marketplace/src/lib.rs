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
mod migrations;
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
		Currency, ExistenceRequirement::KeepAlive, Get, OnRuntimeUpgrade, OnUnbalanced,
		StorageVersion, WithdrawReasons,
	},
	BoundedVec,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{CheckedSub, StaticLookup};
use sp_std::prelude::*;

use primitives::{
	marketplace::{MarketplaceData, MarketplaceId, MarketplaceType},
	nfts::{CollectionId, NFTId},
	CompoundFee, ConfigOp, U8BoundedVec,
};
use ternoa_common::{
	config_op_field_exp,
	traits::{MarketplaceExt, NFTExt},
};
pub use weights::WeightInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

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
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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

		/// The maximum number of collection ids that can be stored inside the collection list.
		#[pallet::constant]
		type CollectionSizeLimit: Get<u32>;
	}

	// TODO Write Tests for Runtime upgrade
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			<migrations::v2::MigrationV2<T> as OnRuntimeUpgrade>::pre_upgrade()
		}

		// This function is called when a runtime upgrade is called. We need to make sure that
		// what ever we do here won't brick the chain or leave the data in a invalid state.
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			let mut weight = Weight::zero();

			let version = StorageVersion::get::<Pallet<T>>();
			if version == StorageVersion::new(1) {
				weight = <migrations::v2::MigrationV2<T> as OnRuntimeUpgrade>::on_runtime_upgrade();

				// Update the storage version.
				StorageVersion::put::<Pallet<T>>(&StorageVersion::new(2));
			}

			weight
		}

		// This function is called after a runtime upgrade is executed. Here we can
		// test if the new state of blockchain data is valid. It's important to say that
		// post_upgrade won't be called when a real runtime upgrade is executed.
		#[cfg(feature = "try-runtime")]
		fn post_upgrade(v: Vec<u8>) -> Result<(), &'static str> {
			<migrations::v2::MigrationV2<T> as OnRuntimeUpgrade>::post_upgrade(v)
		}
	}

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
		MarketplaceData<
			T::AccountId,
			BalanceOf<T>,
			T::AccountSizeLimit,
			T::OffchainDataLimit,
			T::CollectionSizeLimit,
		>,
		OptionQuery,
	>;

	/// Data related to sales
	#[pallet::storage]
	#[pallet::getter(fn listed_nfts)]
	pub type ListedNfts<T: Config> =
		StorageMap<_, Blake2_128Concat, NFTId, Sale<T::AccountId, BalanceOf<T>>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Marketplace created
		MarketplaceCreated {
			marketplace_id: MarketplaceId,
			owner: T::AccountId,
			kind: MarketplaceType,
		},
		/// Marketplace owner set
		MarketplaceOwnerSet { marketplace_id: MarketplaceId, owner: T::AccountId },
		/// Marketplace kind set
		MarketplaceKindSet { marketplace_id: MarketplaceId, kind: MarketplaceType },
		/// Marketplace config set
		MarketplaceConfigSet {
			marketplace_id: MarketplaceId,
			commission_fee: ConfigOp<CompoundFee<BalanceOf<T>>>,
			listing_fee: ConfigOp<CompoundFee<BalanceOf<T>>>,
			account_list: ConfigOp<BoundedVec<T::AccountId, T::AccountSizeLimit>>,
			offchain_data: ConfigOp<U8BoundedVec<T::OffchainDataLimit>>,
			collection_list: ConfigOp<BoundedVec<CollectionId, T::CollectionSizeLimit>>,
		},
		/// Marketplace mint fee set
		MarketplaceMintFeeSet { fee: BalanceOf<T> },
		/// NFT listed
		NFTListed {
			nft_id: NFTId,
			marketplace_id: MarketplaceId,
			price: BalanceOf<T>,
			commission_fee: Option<CompoundFee<BalanceOf<T>>>,
		},
		/// NFT unlisted
		NFTUnlisted { nft_id: NFTId },
		/// NFT sold
		NFTSold {
			nft_id: NFTId,
			marketplace_id: MarketplaceId,
			buyer: T::AccountId,
			listed_price: BalanceOf<T>,
			marketplace_cut: BalanceOf<T>,
			royalty_cut: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not Allowed To List On MP
		NotAllowedToList,
		/// Cannot list delegated NFTs.
		CannotListDelegatedNFTs,
		/// Cannot list capsule NFTs.
		CannotListCapsuleNFTs,
		/// Cannot list soulbound NFTs that was not created from owner.
		CannotListNotCreatedSoulboundNFTs,
		/// Cannot buy owned NFT
		CannotBuyOwnedNFT,
		/// Sender is already the marketplace owner
		CannotTransferMarketplaceToYourself,
		/// NFT already listed
		CannotListAlreadytListedNFTs,
		/// The selected price is too low for commission fee
		PriceCannotCoverMarketplaceFee,
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
		/// Not enough balance for the operation
		NotEnoughBalanceToBuy,
		/// Cannot list because the NFT secret is not synced.
		CannotListNotSyncedSecretNFTs,
		/// Cannot list rented NFTs.
		CannotListRentedNFTs,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new marketplace with the provided details. An ID will be auto
		/// generated and logged as an event, The caller of this function
		/// will become the owner of the new marketplace.
		#[pallet::weight(T::WeightInfo::create_marketplace())]
		pub fn create_marketplace(
			origin: OriginFor<T>,
			kind: MarketplaceType,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Checks.
			// The Caller needs to pay the Marketplace Mint fee.
			Self::pay_mint_fee(&who)?;

			let marketplace_id = Self::get_next_marketplace_id();
			let marketplace = MarketplaceData::new(who.clone(), kind, None, None, None, None, None);

			// Execute.
			Marketplaces::<T>::insert(marketplace_id, marketplace);
			let event = Event::MarketplaceCreated { marketplace_id, owner: who, kind };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Transfer the ownership of the marketplace to the recipient. Must be called by the
		/// owner of the marketplace.
		#[pallet::weight(T::WeightInfo::set_marketplace_owner())]
		pub fn set_marketplace_owner(
			origin: OriginFor<T>,
			marketplace_id: MarketplaceId,
			recipient: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let recipient = T::Lookup::lookup(recipient)?;

			// Checks.
			ensure!(recipient.clone() != who, Error::<T>::CannotTransferMarketplaceToYourself);

			Marketplaces::<T>::try_mutate(marketplace_id, |x| -> DispatchResult {
				let marketplace = x.as_mut().ok_or(Error::<T>::MarketplaceNotFound)?;
				ensure!(marketplace.owner == who, Error::<T>::NotTheMarketplaceOwner);

				// Execute.
				marketplace.owner = recipient.clone();
				Ok(())
			})?;

			let event = Event::MarketplaceOwnerSet { marketplace_id, owner: recipient };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Change the kind of the marketplace, can be private or public.
		/// Must be called by the owner of the marketplace.
		#[pallet::weight(T::WeightInfo::set_marketplace_kind())]
		pub fn set_marketplace_kind(
			origin: OriginFor<T>,
			marketplace_id: MarketplaceId,
			kind: MarketplaceType,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Marketplaces::<T>::try_mutate(marketplace_id, |x| -> DispatchResult {
				// Checks.
				let marketplace = x.as_mut().ok_or(Error::<T>::MarketplaceNotFound)?;
				ensure!(marketplace.owner == who, Error::<T>::NotTheMarketplaceOwner);

				// Execute.
				marketplace.kind = kind;
				Ok(())
			})?;

			let event = Event::MarketplaceKindSet { marketplace_id, kind };
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Set the configuration parameters of the marketplace (eg. commission_fee, listing_fee,
		/// account_list, offchain_data). Must be called by the owner of the marketplace.
		#[pallet::weight(T::WeightInfo::set_marketplace_configuration())]
		pub fn set_marketplace_configuration(
			origin: OriginFor<T>,
			marketplace_id: MarketplaceId,
			commission_fee: ConfigOp<CompoundFee<BalanceOf<T>>>,
			listing_fee: ConfigOp<CompoundFee<BalanceOf<T>>>,
			account_list: ConfigOp<BoundedVec<T::AccountId, T::AccountSizeLimit>>,
			offchain_data: ConfigOp<BoundedVec<u8, T::OffchainDataLimit>>,
			collection_list: ConfigOp<BoundedVec<CollectionId, T::CollectionSizeLimit>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Marketplaces::<T>::try_mutate(marketplace_id, |x| -> DispatchResult {
				let marketplace = x.as_mut().ok_or(Error::<T>::MarketplaceNotFound)?;

				// Checks
				ensure!(marketplace.owner == who, Error::<T>::NotTheMarketplaceOwner);

				// Execute
				config_op_field_exp!(marketplace.commission_fee, commission_fee);
				config_op_field_exp!(marketplace.listing_fee, listing_fee);
				config_op_field_exp!(marketplace.account_list, account_list.clone());
				config_op_field_exp!(marketplace.offchain_data, offchain_data.clone());
				config_op_field_exp!(marketplace.collection_list, collection_list.clone());
				Ok(())
			})?;

			let event = Event::MarketplaceConfigSet {
				marketplace_id,
				commission_fee,
				listing_fee,
				account_list,
				offchain_data,
				collection_list,
			};
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Sets the marketplace mint fee. Can only be called by Root.
		#[pallet::weight(T::WeightInfo::set_marketplace_mint_fee())]
		pub fn set_marketplace_mint_fee(
			origin: OriginFor<T>,
			fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			MarketplaceMintFee::<T>::put(fee);
			Self::deposit_event(Event::MarketplaceMintFeeSet { fee });

			Ok(().into())
		}

		/// Put an NFT on sale on a marketplace.
		#[pallet::weight(T::WeightInfo::list_nft())]
		pub fn list_nft(
			origin: OriginFor<T>,
			nft_id: NFTId,
			marketplace_id: MarketplaceId,
			price: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Checks
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(!nft.state.is_listed, Error::<T>::CannotListAlreadytListedNFTs);
			ensure!(!nft.state.is_capsule, Error::<T>::CannotListCapsuleNFTs);
			ensure!(!nft.state.is_delegated, Error::<T>::CannotListDelegatedNFTs);
			ensure!(
				!(nft.state.is_soulbound && nft.creator != nft.owner),
				Error::<T>::CannotListNotCreatedSoulboundNFTs
			);
			ensure!(!nft.state.is_syncing, Error::<T>::CannotListNotSyncedSecretNFTs);
			ensure!(!nft.state.is_rented, Error::<T>::CannotListRentedNFTs);

			let marketplace =
				Marketplaces::<T>::get(marketplace_id).ok_or(Error::<T>::MarketplaceNotFound)?;

			marketplace
				.allowed_to_list(&who, nft.collection_id)
				.ok_or(Error::<T>::NotAllowedToList)?;

			// Check if the selected price can cover the marketplace commission_fee if it exists.
			if let Some(commission_fee) = &marketplace.commission_fee {
				if let CompoundFee::Flat(flat_commission) = commission_fee {
					ensure!(price >= *flat_commission, Error::<T>::PriceCannotCoverMarketplaceFee);
				}
			}

			// The Caller needs to pay the listing fee if it exists.
			Self::pay_listing_fee(&who, &marketplace, price)?;

			// Execute.
			let sale = Sale::new(who, marketplace_id, price, marketplace.commission_fee);
			ListedNfts::<T>::insert(nft_id, sale);
			nft.state.is_listed = true;
			T::NFTExt::set_nft_state(nft_id, nft.state)?;

			let event = Event::NFTListed {
				nft_id,
				marketplace_id,
				price,
				commission_fee: marketplace.commission_fee,
			};
			Self::deposit_event(event);

			Ok(().into())
		}

		/// Remove an NFT from sale.
		#[pallet::weight(T::WeightInfo::unlist_nft())]
		pub fn unlist_nft(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;

			// Checks.
			ensure!(nft.owner == who, Error::<T>::NotTheNFTOwner);
			ensure!(ListedNfts::<T>::contains_key(nft_id), Error::<T>::NFTNotForSale);

			// Execute.
			nft.state.is_listed = false;
			T::NFTExt::set_nft_state(nft_id, nft.state)?;
			ListedNfts::<T>::remove(nft_id);
			Self::deposit_event(Event::NFTUnlisted { nft_id });

			Ok(().into())
		}

		/// Buy a listed nft
		#[pallet::weight(T::WeightInfo::buy_nft())]
		pub fn buy_nft(origin: OriginFor<T>, nft_id: NFTId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let mut nft = T::NFTExt::get_nft(nft_id).ok_or(Error::<T>::NFTNotFound)?;
			let sale = ListedNfts::<T>::get(nft_id).ok_or(Error::<T>::NFTNotForSale)?;
			let marketplace = Marketplaces::<T>::get(sale.marketplace_id)
				.ok_or(Error::<T>::MarketplaceNotFound)?;
			let mut price = sale.price;

			// Checks
			ensure!(sale.account_id != who, Error::<T>::CannotBuyOwnedNFT);
			ensure!(T::Currency::free_balance(&who) >= price, Error::<T>::NotEnoughBalanceToBuy);

			// Caller pays for commission fee, the price is updated.
			let commission_fee = Self::pay_commission_fee(&who, &marketplace, &sale, price)?;
			price = price.checked_sub(&commission_fee).ok_or(Error::<T>::InternalMathError)?;

			// Caller pays for royalty, the price is updated.
			let royalty_value = nft.royalty * price;
			T::Currency::transfer(&who, &nft.creator, royalty_value, KeepAlive)?;
			price = price.checked_sub(&royalty_value).ok_or(Error::<T>::InternalMathError)?;

			// Caller pays the seller the updated price.
			T::Currency::transfer(&who, &sale.account_id, price, KeepAlive)?;

			//Execute.
			nft.owner = who.clone();
			nft.state.is_listed = false;
			T::NFTExt::set_nft(nft_id, nft)?;
			ListedNfts::<T>::remove(nft_id);
			let event = Event::NFTSold {
				nft_id,
				marketplace_id: sale.marketplace_id,
				buyer: who,
				listed_price: sale.price,
				marketplace_cut: commission_fee,
				royalty_cut: royalty_value,
			};
			Self::deposit_event(event);

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn get_next_marketplace_id() -> MarketplaceId {
		let marketplace_id = NextMarketplaceId::<T>::get();
		let next_id = marketplace_id
			.checked_add(1)
			.expect("If u32 is not enough we should crash for safety; qed.");
		NextMarketplaceId::<T>::put(next_id);

		marketplace_id
	}

	fn pay_mint_fee(who: &T::AccountId) -> Result<(), DispatchError> {
		let mint_fee = MarketplaceMintFee::<T>::get();
		let reason = WithdrawReasons::FEE;
		let imbalance = T::Currency::withdraw(&who, mint_fee, reason, KeepAlive)?;
		T::FeesCollector::on_unbalanced(imbalance);
		Ok(())
	}

	fn pay_listing_fee(
		who: &T::AccountId,
		marketplace: &MarketplaceData<
			T::AccountId,
			BalanceOf<T>,
			T::AccountSizeLimit,
			T::OffchainDataLimit,
			T::CollectionSizeLimit,
		>,
		price: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		if let Some(listing_fee) = &marketplace.listing_fee {
			let listing_fee = match *listing_fee {
				CompoundFee::Flat(x) => x,
				CompoundFee::Percentage(x) => x * price,
			};
			T::Currency::transfer(&who, &marketplace.owner, listing_fee, KeepAlive)?;
		}
		Ok(())
	}

	fn pay_commission_fee(
		who: &T::AccountId,
		marketplace: &MarketplaceData<
			T::AccountId,
			BalanceOf<T>,
			T::AccountSizeLimit,
			T::OffchainDataLimit,
			T::CollectionSizeLimit,
		>,
		sale: &Sale<T::AccountId, BalanceOf<T>>,
		price: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		if let Some(commission_fee) = &sale.commission_fee {
			let commission_fee = match *commission_fee {
				CompoundFee::Flat(x) => x,
				CompoundFee::Percentage(x) => x * price,
			};
			T::Currency::transfer(&who, &marketplace.owner, commission_fee, KeepAlive)?;
			return Ok(commission_fee)
		}
		Ok(0u32.into())
	}
}

impl<T: Config> MarketplaceExt for Pallet<T> {
	type AccountId = T::AccountId;
	type Balance = BalanceOf<T>;
	type OffchainDataLimit = T::OffchainDataLimit;
	type AccountSizeLimit = T::AccountSizeLimit;
	type CollectionSizeLimit = T::CollectionSizeLimit;

	fn get_marketplace(
		id: MarketplaceId,
	) -> Option<
		MarketplaceData<
			Self::AccountId,
			Self::Balance,
			Self::AccountSizeLimit,
			Self::OffchainDataLimit,
			Self::CollectionSizeLimit,
		>,
	> {
		Marketplaces::<T>::get(id)
	}

	fn set_marketplace(
		id: MarketplaceId,
		marketplace_data: MarketplaceData<
			T::AccountId,
			BalanceOf<T>,
			T::AccountSizeLimit,
			T::OffchainDataLimit,
			T::CollectionSizeLimit,
		>,
	) -> Result<(), DispatchError> {
		Marketplaces::<T>::insert(id, marketplace_data);

		Ok(())
	}
}
