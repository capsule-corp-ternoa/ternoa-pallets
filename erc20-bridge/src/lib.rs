#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;

mod weights;

use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	ensure,
	traits::{
		Currency, EnsureOrigin,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		Get, OnUnbalanced, WithdrawReasons,
	},
};

use frame_system::ensure_signed;

use sp_arithmetic::traits::SaturatedConversion;
use sp_core::U256;
use sp_std::prelude::*;
pub use weights::WeightInfo;

use chainbridge::types::{ChainId, ResourceId};

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub use pallet::*;

// ----------------------------------------------------------------------------
// Pallet module
// ----------------------------------------------------------------------------

// ERC20Bridge pallet module
//
// The name of the pallet is provided by `construct_runtime` and is used as
// the unique identifier for the pallet's storage. It is not defined in the
// pallet itself.
#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub(crate) type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;

	// ERC20Bridge pallet type declaration.
	//
	// This structure is a placeholder for traits and functions implementation
	// for the pallet.
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// ------------------------------------------------------------------------
	// Pallet configuration
	// ------------------------------------------------------------------------

	/// ERC20Bridge pallet's configuration trait.
	///
	/// Associated types and constants are declared in this trait. If the pallet
	/// depends on other super-traits, the latter must be added to this trait,
	/// such as, in this case, [`chainbridge::Config`] super-trait, for instance.
	/// Note that [`frame_system::Config`] must always be included.
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

		#[pallet::constant]
		type NativeTokenId: Get<ResourceId>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;
	}

	// ------------------------------------------------------------------------
	// Pallet storage
	// ------------------------------------------------------------------------

	/// Host much does it cost to transfer Native through the bridge (extra fee on top of the tx
	/// fees)
	#[pallet::storage]
	#[pallet::getter(fn bridge_fee)]
	pub type BridgeFee<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	// ------------------------------------------------------------------------
	// Pallet events
	// ------------------------------------------------------------------------

	// The macro generates event metadata and derive Clone, Debug, Eq, PartialEq and Codec
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		BridgeFeeUpdated { fee: BalanceOf<T> },
	}

	// ------------------------------------------------------------------------
	// Pallet genesis configuration
	// ------------------------------------------------------------------------

	// The genesis configuration type.
	#[pallet::genesis_config]
	pub struct GenesisConfig {}

	// The default value for the genesis config type.
	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {}
		}
	}

	// The build of genesis for the pallet.
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {}
	}

	// ------------------------------------------------------------------------
	// Pallet lifecycle hooks
	// ------------------------------------------------------------------------

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// ------------------------------------------------------------------------
	// Pallet errors
	// ------------------------------------------------------------------------

	#[pallet::error]
	pub enum Error<T> {
		InvalidTransfer,
		RemovalImpossible,
	}

	// ------------------------------------------------------------------------
	// Pallet dispatchable functions
	// ------------------------------------------------------------------------

	// Declare Call struct and implement dispatchable (or callable) functions.
	//
	// Dispatchable functions are transactions modifying the state of the chain. They
	// are also called extrinsics are constitute the pallet's public interface.
	// Note that each parameter used in functions must implement `Clone`, `Debug`,
	// `Eq`, `PartialEq` and `Codec` traits.
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

			let resource_id = T::NativeTokenId::get();
			<chainbridge::Pallet<T>>::transfer_fungible(
				dest_id,
				resource_id,
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
} // end of 'pallet' module
