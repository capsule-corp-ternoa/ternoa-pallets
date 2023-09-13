use super::*;

pub mod v2 {
	use super::*;
	use frame_support::{
		traits::OnRuntimeUpgrade, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
	};
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_arithmetic::traits::AtLeast32BitUnsigned;
	use sp_std::fmt::Debug;

	#[derive(
		PartialEqNoBound, CloneNoBound, Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
	)]
	#[codec(mel_bound(AccountId: MaxEncodedLen, BlockNumber: MaxEncodedLen))]
	pub struct OldTeeStakingLedger<AccountId, BlockNumber>
	where
		AccountId: Clone + PartialEq + Debug,
		BlockNumber:
			Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
	{
		/// The operator account whose balance is actually locked and at stake.
		pub operator: AccountId,
		/// State variable to know whether the staked amount is unbonded
		pub is_unlocking: bool,
		/// Block Number of when unbonded happened
		pub unbonded_at: BlockNumber,
	}

	pub struct MigrationV2<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrationV2<T> {
		// #[cfg(feature = "try-runtime")]
		// fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// 	log::info!("Pre-upgrade inside MigrationV2");
		// 	Ok(Vec::new())
		// }

		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			let mut read = 0u64;
			let mut write = 0u64;

			let current_active_era = Staking::<T>::active_era().map(|e| e.index).unwrap();

			let stake_amount: u128 = 250_000_000_000_000_000_000_000u128;
			let stake_amount: BalanceOf<T> = stake_amount.saturated_into::<BalanceOf<T>>();

			// Translate the old StakingLedger storage to the new format
			StakingLedger::<T>::translate(
				|_id, old: OldTeeStakingLedger<T::AccountId, T::BlockNumber>| {
					let new_staking_ledger =
						TeeStakingLedger::<T::AccountId, T::BlockNumber, BalanceOf<T>> {
							operator: old.operator.clone(),
							staked_amount: stake_amount, /* Initialize with default value */
							is_unlocking: old.is_unlocking,
							unbonded_at: old.unbonded_at,
						};
					read += 1;
					write += 1;
					OperatorAssignedEra::<T>::insert(old.operator, current_active_era);
					Some(new_staking_ledger)
				},
			);

			T::DbWeight::get().reads_writes(read, write)
		}

		// #[cfg(feature = "try-runtime")]
		// fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
		// 	log::info!("Post-upgrade inside MigrationV2");
		// 	Ok(())
		// }
	}
}
