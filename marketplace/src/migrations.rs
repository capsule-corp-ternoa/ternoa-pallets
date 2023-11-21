// use super::*;

// pub mod v2 {
// 	use super::*;
// 	use frame_support::{
// 		traits::OnRuntimeUpgrade, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
// 	};
// 	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
// 	use scale_info::TypeInfo;
// 	use sp_std::fmt::Debug;

// 	#[derive(
// 		Encode,
// 		Decode,
// 		CloneNoBound,
// 		Eq,
// 		PartialEqNoBound,
// 		RuntimeDebugNoBound,
// 		TypeInfo,
// 		MaxEncodedLen,
// 	)]
// 	#[scale_info(skip_type_params(AccountSizeLimit, OffchainDataLimit))]
// 	#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen))]
// 	pub struct OldMarketplaceData<AccountId, Balance, AccountSizeLimit, OffchainDataLimit>
// 	where
// 		AccountId: Clone + PartialEq + Debug,
// 		Balance: Clone + PartialEq + Debug + sp_std::cmp::PartialOrd,
// 		AccountSizeLimit: Get<u32>,
// 		OffchainDataLimit: Get<u32>,
// 	{
// 		pub owner: AccountId,
// 		pub kind: MarketplaceType,
// 		pub commission_fee: Option<CompoundFee<Balance>>,
// 		pub listing_fee: Option<CompoundFee<Balance>>,
// 		pub account_list: Option<BoundedVec<AccountId, AccountSizeLimit>>,
// 		pub offchain_data: Option<U8BoundedVec<OffchainDataLimit>>,
// 	}

// 	pub struct MigrationV2<T>(sp_std::marker::PhantomData<T>);
// 	impl<T: Config> OnRuntimeUpgrade for MigrationV2<T> {
// 		#[cfg(feature = "try-runtime")]
// 		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
// 			log::info!("Pre-upgrade inside MigrationV2");
// 			Ok(Vec::new())
// 		}

// 		fn on_runtime_upgrade() -> frame_support::weights::Weight {
// 			Marketplaces::<T>::translate(
// 				|_id,
// 				 old: OldMarketplaceData<
// 					T::AccountId,
// 					BalanceOf<T>,
// 					T::AccountSizeLimit,
// 					T::OffchainDataLimit,
// 				>| {
// 					let new_marketplace_data = MarketplaceData::new(
// 						old.owner,
// 						old.kind,
// 						old.commission_fee,
// 						old.listing_fee,
// 						old.account_list,
// 						old.offchain_data,
// 						None,
// 					);

// 					Some(new_marketplace_data)
// 				},
// 			);

// 			frame_support::weights::Weight::MAX
// 		}

// 		#[cfg(feature = "try-runtime")]
// 		fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
// 			log::info!("Post-upgrade inside MigrationV2");
// 			Ok(())
// 		}
// 	}
// }
