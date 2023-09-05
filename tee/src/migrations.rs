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
			
			let cluster1_type = ClusterType::Admin; // Replace with your desired cluster type

			let cluster_1_enclave_1 = "5HQH3eTQuDSgutg7bgARbcKcoLksqDSvwvytwnKbwt7d3vC7";
			let cluster_1_enclave_1 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_1_enclave_1).unwrap();
			let cluster_1_enclave_2 = "5G6QNaow6wFSUt468H4Sqasr3m9iz5oErzpHuLuAJcrwS83W";
			let cluster_1_enclave_2 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_1_enclave_2).unwrap();
			let cluster_1_enclave_3 = "5HmZna5KRdXvZ9GrAtZNbU9UfQoPGjfyPYtzopnMTvSDbPVn";
			let cluster_1_enclave_3 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_1_enclave_3).unwrap();
			let cluster_1_enclave_4 = "5FeRQfUDo7JzvVvtsudP6jRaFHG9nhNLLdHyTDPS3swv9nGE";
			let cluster_1_enclave_4 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_1_enclave_4).unwrap();
			let cluster_1_enclave_5 = "5Eecn3sD5bjbHi4rMyJevDViuCNeS2AgAV9KivZxb8XvpSQe";
			let cluster_1_enclave_5 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_1_enclave_5).unwrap();
			
			let cluster_1_enclaves: BoundedVec<(T::AccountId, SlotId), T::ClusterSize> = BoundedVec::try_from(vec![
				(
					cluster_1_enclave_1, // Replace with your actual AccountId values
					0,
				),
				(
					cluster_1_enclave_2, // Replace with your actual AccountId values
					1,
				),
				(
					cluster_1_enclave_3, // Replace with your actual AccountId values
					2,
				),
				(
					cluster_1_enclave_4, // Replace with your actual AccountId values
					3,
				),
				(
					cluster_1_enclave_5, // Replace with your actual AccountId values
					4,
				),
			]).unwrap();

			let cluster_1 = Cluster::new(cluster_1_enclaves, cluster1_type.clone());

			ClusterData::<T>::insert(1, cluster_1);

			let cluster2_type = ClusterType::Public; // Replace with your desired cluster type

			let cluster_2_enclave_1 = "5CcqaTBwWvbB2MvmeteSDLVujL3oaFHtdf24pPVT3Xf8v7tC";
			let cluster_2_enclave_1 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_2_enclave_1).unwrap();
			let cluster_2_enclave_2 = "5EjxzNQPeb7dHVVSugG54ghNFzwENxh7GA6VCn7kfBfE2FNg";
			let cluster_2_enclave_2 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_2_enclave_2).unwrap();
			let cluster_2_enclave_3 = "5GP1ddEfzCqSeTfs13BoC1GkiFQUPPUYqomBDAFsMU2bBnny";
			let cluster_2_enclave_3 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_2_enclave_3).unwrap();
			let cluster_2_enclave_4 = "5C5PJTfrmaZ3gF9tPWeQMQ63akmwkNXDmerTmVLbTZWEtcBd";
			let cluster_2_enclave_4 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_2_enclave_4).unwrap();
			let cluster_2_enclave_5 = "5C4zzH1ejwptMaeqi97A65J63jB8kqtm63oNPiVVCx2eoWMB";
			let cluster_2_enclave_5 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_2_enclave_5).unwrap();
			
			let cluster_2_enclaves: BoundedVec<(T::AccountId, SlotId), T::ClusterSize> = BoundedVec::try_from(vec![
				(
					cluster_2_enclave_1, // Replace with your actual AccountId values
					0,
				),
				(
					cluster_2_enclave_2, // Replace with your actual AccountId values
					1,
				),
				(
					cluster_2_enclave_3, // Replace with your actual AccountId values
					2,
				),
				(
					cluster_2_enclave_4, // Replace with your actual AccountId values
					3,
				),
				(
					cluster_2_enclave_5, // Replace with your actual AccountId values
					4,
				),
			]).unwrap();

			let cluster_2 = Cluster::new(cluster_2_enclaves, cluster2_type.clone());

			ClusterData::<T>::insert(2, cluster_2);

			let cluster3_type = ClusterType::Public; // Replace with your desired cluster type

			let cluster_3_enclave_1 = "5G1AGcU2D8832LcRefKrPm8Zrob63vf6uQSzKGmhyV9DrzFs";
			let cluster_3_enclave_1 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_3_enclave_1).unwrap();
			let cluster_3_enclave_2 = "5C8Y5FU5bf4mAJaaTUH2hr8dz2bLrpijppzqxErqYNWgjCzK";
			let cluster_3_enclave_2 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_3_enclave_2).unwrap();
			let cluster_3_enclave_3 = "5CzNerUQtWgRX3AyWUtBgRmLHxZAg74EoTBXqefbuDHJA8tL";
			let cluster_3_enclave_3 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_3_enclave_3).unwrap();
			let cluster_3_enclave_4 = "5CqHgc11S3zUBtnM1vJchiMpGBFnQ3sQJeKcvszKmjuycUR4";
			let cluster_3_enclave_4 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_3_enclave_4).unwrap();
			let cluster_3_enclave_5 = "5F1ieotWMiq4QVkV6vahCGaqundt4JQWAjp4a3rkSGkvbpoJ";
			let cluster_3_enclave_5 = <crate::Pallet<T>>::convert_str_to_valid_account_id(cluster_3_enclave_5).unwrap();
			
			let cluster_3_enclaves: BoundedVec<(T::AccountId, SlotId), T::ClusterSize> = BoundedVec::try_from(vec![
				(
					cluster_3_enclave_1, // Replace with your actual AccountId values
					0,
				),
				(
					cluster_3_enclave_2, // Replace with your actual AccountId values
					1,
				),
				(
					cluster_3_enclave_3, // Replace with your actual AccountId values
					2,
				),
				(
					cluster_3_enclave_4, // Replace with your actual AccountId values
					3,
				),
				(
					cluster_3_enclave_5, // Replace with your actual AccountId values
					4,
				),
			]).unwrap();

			let cluster_3 = Cluster::new(cluster_3_enclaves, cluster3_type.clone());

			ClusterData::<T>::insert(3, cluster_3);

			write+=3;

			T::DbWeight::get().reads_writes(read, write)
		}

		

		// #[cfg(feature = "try-runtime")]
		// fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
		// 	log::info!("Post-upgrade inside MigrationV2");
		// 	Ok(())
		// }
	}
}
