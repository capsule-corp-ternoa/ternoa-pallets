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

pub mod weights;

use frame_support::{
	pallet_prelude::*,
	traits::{
		Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced, StorageVersion, TryDrop,
		WithdrawReasons,
	},
	PalletId,
};
use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};
use sp_std::prelude::*;
pub use weights::WeightInfo;

pub use pallet::*;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default, MaxEncodedLen)]
pub struct StakingRewardsData<Balance> {
	pub session_era_payout: Balance,
	pub session_extra_reward_payout: Balance,
}

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type NegativeImbalance<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::dispatch::DispatchResultWithPostInfo;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Caps Currency
		type Currency: Currency<Self::AccountId>;

		/// The auctions pallet id - will be used to generate account id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Origin that can control this pallet.
		type ExternalOrigin: EnsureOrigin<Self::Origin>;

		/// Weight
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::set_session_extra_reward_payout())]
		pub fn set_session_extra_reward_payout(
			origin: OriginFor<T>,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ExternalOrigin::ensure_origin(origin)?;

			Data::<T>::mutate(|data| {
				data.session_extra_reward_payout = value;
			});

			let event = Event::SessionExtraRewardPayoutChanged { value };
			Self::deposit_event(event);

			Ok(().into())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SessionExtraRewardPayoutChanged { value: BalanceOf<T> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::storage]
	#[pallet::getter(fn session_era_payout)]
	pub type Data<T: Config> = StorageValue<_, StakingRewardsData<BalanceOf<T>>, ValueQuery>;
}

impl<T: Config> Pallet<T> {
	/// The account ID of the auctions pot.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}
}

impl<T: Config + pallet_authorship::Config> OnUnbalanced<NegativeImbalance<T>> for Pallet<T> {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<T>>)
	where
		NegativeImbalance<T>: frame_support::traits::Imbalance<BalanceOf<T>>,
		T: pallet_authorship::Config,
	{
		if let Some(amount) = fees_then_tips.next() {
			Data::<T>::mutate(|data| {
				data.session_era_payout = data.session_era_payout.saturating_add(amount.peek());
			});

			// for fees, 100% to staking rewards
			// T::Currency::resolve_creating(&Self::account_id(), amount);

			// This will drop the value
			amount.try_drop().unwrap_or_else(|x| drop(x));

			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 100% to author
				if let Some(author) = <pallet_authorship::Pallet<T>>::author() {
					T::Currency::resolve_creating(&author, tips);
				}
			}
		}
	}
}

impl<T: Config> pallet_staking::EraPayout<BalanceOf<T>> for Pallet<T> {
	fn era_payout(
		_total_staked: BalanceOf<T>,
		_total_issuance: BalanceOf<T>,
		_era_duration_millis: u64,
	) -> (BalanceOf<T>, BalanceOf<T>) {
		let mut stakers_pay = 0u32.into();
		let treasury_pay = 0u32.into();

		Data::<T>::mutate(|data| {
			stakers_pay = data.session_era_payout;
			data.session_era_payout = 0u32.into();

			let additional_pay = sp_std::cmp::min(
				T::Currency::free_balance(&Self::account_id()),
				data.session_extra_reward_payout,
			);

			if !additional_pay.is_zero() {
				let negative_imbalance = T::Currency::withdraw(
					&Self::account_id(),
					additional_pay,
					WithdrawReasons::all(),
					ExistenceRequirement::AllowDeath,
				)
				.unwrap_or_else(|_| NegativeImbalance::<T>::zero());
				stakers_pay += negative_imbalance.peek();
				negative_imbalance.try_drop().unwrap_or_else(|x| drop(x));
			}
		});

		(stakers_pay, treasury_pay)
	}
}
