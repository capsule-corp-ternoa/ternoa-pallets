use super::*;

pub mod v2 {
	use super::*;
	use frame_support::{
		traits::OnRuntimeUpgrade, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
	};
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_std::fmt::Debug;

	#[derive(
		Encode,
		Decode,
		CloneNoBound,
		PartialEqNoBound,
		Eq,
		RuntimeDebugNoBound,
		TypeInfo,
		MaxEncodedLen,
	)]
	#[scale_info(skip_type_params(ClusterSize))]
	#[codec(mel_bound(AccountId: MaxEncodedLen))]
	pub struct OldClusterData<AccountId, ClusterSize>
	where
		AccountId: Clone + PartialEq + Debug,
		ClusterSize: Get<u32>,
	{
		pub enclaves: BoundedVec<AccountId, ClusterSize>,
	}

	pub struct MigrationV2<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrationV2<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			log::info!("Pre-upgrade inside MigrationV2");
			Ok(Vec::new())
		}

		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			let mut read = 0u64;
			let mut write = 0u64;

			ClusterData::<T>::translate(
				|_id, old: OldClusterData<T::AccountId, T::ClusterSize>| {
					let new_cluster_data = Cluster::new(old.enclaves, true);
					read += 1;
					write += 1;

					Some(new_cluster_data)
				},
			);

			T::DbWeight::get().reads_writes(read, write)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
			log::info!("Post-upgrade inside MigrationV2");
			Ok(())
		}
	}
}
