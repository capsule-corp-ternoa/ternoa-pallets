use super::*;

pub mod v2 {
	use super::*;
	use frame_support::{
		traits::OnRuntimeUpgrade, CloneNoBound, PartialEqNoBound, RuntimeDebug, RuntimeDebugNoBound,
	};
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_std::fmt::Debug;

	#[derive(
		Encode, Decode, Eq, Default, TypeInfo, Clone, PartialEq, RuntimeDebug, MaxEncodedLen,
	)]
	pub struct OldNFTState {
		/// Is NFT converted to capsule
		pub is_capsule: bool,
		/// Is NFT listed for sale
		pub listed_for_sale: bool,
		/// Is NFT contains secret
		pub is_secret: bool,
		/// Is NFT delegated
		pub is_delegated: bool,
		/// Is NFT soulbound
		pub is_soulbound: bool,
	}

	#[derive(
		Encode,
		Decode,
		Eq,
		Default,
		TypeInfo,
		CloneNoBound,
		PartialEqNoBound,
		RuntimeDebugNoBound,
		MaxEncodedLen,
	)]
	#[scale_info(skip_type_params(NFTOffchainDataLimit))]
	#[codec(mel_bound(AccountId: MaxEncodedLen))]
	pub struct OldNFTData<AccountId, NFTOffchainDataLimit>
	where
		AccountId: Clone + PartialEq + Debug,
		NFTOffchainDataLimit: Get<u32>,
	{
		/// NFT owner
		pub owner: AccountId,
		/// NFT creator
		pub creator: AccountId,
		/// NFT offchain_data
		pub offchain_data: U8BoundedVec<NFTOffchainDataLimit>,
		/// Collection ID
		pub collection_id: Option<CollectionId>,
		/// Royalty
		pub royalty: Permill,
		/// NFT state
		pub state: OldNFTState,
	}

	pub struct MigrationV2<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrationV2<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			log::info!("Pre-upgrade inside MigrationV2");
			Ok(())
		}

		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			Nfts::<T>::translate(|_id, old: OldNFTData<T::AccountId, T::NFTOffchainDataLimit>| {
				let new_state = NFTState::new(
					old.state.is_capsule,
					old.state.listed_for_sale,
					old.state.is_secret,
					old.state.is_delegated,
					old.state.is_soulbound,
					false
				);

				let new_nft_data = NFTData::new(
					old.owner,
					old.creator,
					old.offchain_data,
					old.royalty,
					new_state,
					old.collection_id,
				);

				Some(new_nft_data)
			});

			frame_support::weights::Weight::MAX
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			log::info!("Post-upgrade inside MigrationV2");
			Ok(())
		}
	}
}
