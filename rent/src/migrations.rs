use super::*;

pub mod v2 {
	use super::*;
	use frame_support::{
		traits::OnRuntimeUpgrade, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
	};
	use frame_system::Pallet as System;
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_arithmetic::traits::AtLeast32BitUnsigned;
	use sp_std::fmt::Debug;

	#[derive(
		Encode,
		Decode,
		CloneNoBound,
		Eq,
		PartialEqNoBound,
		RuntimeDebugNoBound,
		TypeInfo,
		MaxEncodedLen,
	)]
	#[scale_info(skip_type_params(AccountSizeLimit))]
	#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen, BlockNumber: MaxEncodedLen))]
	pub struct OldRentContractData<AccountId, BlockNumber, Balance, AccountSizeLimit>
	where
		AccountId: Clone + PartialEq + Debug,
		Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
		BlockNumber:
			Clone + PartialEq + Debug + sp_std::cmp::PartialOrd + AtLeast32BitUnsigned + Copy,
		AccountSizeLimit: Get<u32>,
	{
		/// Start block of the contract.
		pub start_block: Option<BlockNumber>,
		/// Renter of the NFT.
		pub renter: AccountId,
		/// Rentee of the NFT.
		pub rentee: Option<AccountId>,
		/// Duration of the renting contract.
		pub duration: Duration<BlockNumber>,
		/// Acceptance type of the renting contract.
		pub acceptance_type: AcceptanceType<AccountList<AccountId, AccountSizeLimit>>,
		/// Renter can cancel.
		pub renter_can_revoke: bool,
		/// Rent fee paid by rentee.
		pub rent_fee: RentFee<Balance>,
		/// Optional cancellation fee for renter.
		pub renter_cancellation_fee: CancellationFee<Balance>,
		/// Optional cancellation fee for rentee.
		pub rentee_cancellation_fee: CancellationFee<Balance>,
	}

	pub struct MigrationV2<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrationV2<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			log::info!("Pre-upgrade inside MigrationV2");
			Ok(Vec::new())
		}

		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			let now = System::<T>::block_number();
			let mut read = 0u64;
    		let mut write = 0u64;
			Contracts::<T>::translate(
				|_id,
				 old: OldRentContractData<
					T::AccountId,
					T::BlockNumber,
					BalanceOf<T>,
					T::AccountSizeLimit,
				>| {
					let new_rent_contract_data = RentContractData::new(
						old.start_block,
						old.renter,
						old.rentee,
						old.duration,
						old.acceptance_type,
						old.renter_can_revoke,
						old.rent_fee,
						old.renter_cancellation_fee,
						old.rentee_cancellation_fee,
						now,
					);
					read += 1;
            		write += 1;

					Some(new_rent_contract_data)
				},
			);

			T::DbWeight::get().reads_writes(read_ops, write_ops)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
			log::info!("Post-upgrade inside MigrationV2");
			Ok(())
		}
	}
}
