#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;
mod weights;

use chainbridge::types::ChainId;
use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	ensure,
	traits::{
		Currency, EnsureOrigin,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		OnUnbalanced, WithdrawReasons,
	},
};
use frame_system::ensure_signed;
use sp_arithmetic::traits::SaturatedConversion;
use sp_core::U256;
use sp_std::prelude::*;

pub use pallet::*;
pub use weights::WeightInfo;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub(crate) type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + chainbridge::Config {
		/// Associated type for Event enum
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Specifies the origin check provided by the bridge for calls that can only be called by
		/// the bridge pallet
		type BridgeOrigin: EnsureOrigin<
			<Self as frame_system::Config>::Origin,
			Success = <Self as frame_system::Config>::AccountId,
		>;

		/// The currency mechanism.
		type Currency: Currency<<Self as frame_system::Config>::AccountId>;

		/// What we do with additional fees
		type FeesCollector: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;
	}

	/// Host much does it cost to transfer Native through the bridge (extra fee on top of the tx
	/// fees)
	#[pallet::storage]
	#[pallet::getter(fn bridge_fee)]
	pub type BridgeFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	// The macro generates event metadata and derive Clone, Debug, Eq, PartialEq and Codec
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		BridgeFeeUpdated { fee: BalanceOf<T> },
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransfer,
		RemovalImpossible,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfers some amount of the native token to some recipient on a (whitelisted)
		/// destination chain.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::transfer_native())]
		pub fn transfer_native(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			recipient: Vec<u8>,
			dest_id: ChainId,
		) -> DispatchResultWithPostInfo {
			let source = ensure_signed(origin)?;
			ensure!(
				<chainbridge::Pallet<T>>::chain_whitelisted(dest_id),
				Error::<T>::InvalidTransfer
			);
			#[cfg(feature = "std")]
			{
				std::println!("origin: {:?}", source);
				std::println!("amount: {:?}", amount);
				std::println!("recipient: {:?}", recipient);
				std::println!("dest_id: {:?}", dest_id);
			}

			ensure!(
				T::Currency::free_balance(&source) >= Self::bridge_fee() + amount,
				Error::<T>::RemovalImpossible
			);
			let imbalance = T::Currency::withdraw(
				&source,
				Self::bridge_fee(),
				WithdrawReasons::FEE,
				KeepAlive,
			)?;
			T::FeesCollector::on_unbalanced(imbalance);
			if let Err(_) =
				T::Currency::withdraw(&source, amount, WithdrawReasons::TRANSFER, AllowDeath)
			{
				return Err(Error::<T>::RemovalImpossible)?
			}
			T::Currency::burn(amount);

			<chainbridge::Pallet<T>>::bridge_funds(
				dest_id,
				recipient,
				U256::from(amount.saturated_into::<u128>()),
			)?;

			Ok(().into())
		}

		/// Update the bridge fee value
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_bridge_fee())]
		pub fn set_bridge_fee(
			origin: OriginFor<T>,
			bridge_fee: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			BridgeFee::<T>::put(bridge_fee);

			Self::deposit_event(Event::BridgeFeeUpdated { fee: bridge_fee });

			Ok(().into())
		}

		/// Executes a simple currency transfer using the bridge account as the source
		#[pallet::weight(<T as pallet::Config>::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			to: <T as frame_system::Config>::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			T::BridgeOrigin::ensure_origin(origin)?;
			let negative_imbalance = T::Currency::issue(amount);
			T::Currency::resolve_creating(&to, negative_imbalance);

			Ok(().into())
		}
	}
}
