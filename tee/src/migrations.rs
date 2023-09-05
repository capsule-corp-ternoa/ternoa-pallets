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
		// #[cfg(feature = "try-runtime")]
		// fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// 	log::info!("Pre-upgrade inside MigrationV2");
		// 	Ok(Vec::new())
		// }

		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			let mut read = 0u64;
			let mut write = 0u64;

			ClusterData::<T>::translate(
				|id, old: OldClusterData<T::AccountId, T::ClusterSize>| {
					let mut new_enclaves: BoundedVec<(T::AccountId, SlotId), T::ClusterSize> =
						BoundedVec::default();
					let mut slot_id_counter = 0;

					if id == 0 {
						for account_id in old.enclaves.into_iter() {
							let slot_id: SlotId = slot_id_counter;
							slot_id_counter += 1;
	
							let push_result = new_enclaves.try_push((account_id, slot_id));
							match push_result {
								Ok(_) => {
									read += 1;
									write += 1;
								},
								Err(_) => {
									// Handle the error case if the `BoundedVec` is already full
									break // Stop adding elements if the desired size is reached
								},
							}
						}
					}
				
					let new_cluster_data = Cluster::new(new_enclaves, ClusterType::Public);
					read += 1;
					write += 1;

					Some(new_cluster_data)
				},
			);
			
			let cluster_type = ClusterType::Public; // Replace with your desired cluster type

			// let cluster = Cluster::new(enclaves, cluster_type);
			let default_pdot_address = "5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM";
			let default_address = <crate::Pallet<T>>::convert_str_to_valid_account_id(default_pdot_address).unwrap();
			let enclaves: BoundedVec<(T::AccountId, SlotId), T::ClusterSize> = BoundedVec::try_from(vec![
				(
					default_address, // Replace with your actual AccountId values
					0,
				),
			]).unwrap();
			let cluster_1 = Cluster::new(enclaves, cluster_type.clone());

			ClusterData::<T>::insert(1, cluster_1);

			T::DbWeight::get().reads_writes(read, write)
		}

		

		// #[cfg(feature = "try-runtime")]
		// fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
		// 	log::info!("Post-upgrade inside MigrationV2");
		// 	Ok(())
		// }
	}
}
