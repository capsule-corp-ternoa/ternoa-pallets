use super::*;

pub mod v2 {
	use super::*;
	use frame_support::{
		traits::OnRuntimeUpgrade, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
	};
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;
	use sp_std::{fmt::Debug, vec::Vec};

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
		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			let read = 450u64;
			let write = 450u64;

			// Remove these operators from the storages
			let accounts = &[
				"5GrqCRkedhRBvaTnfNxitMQrvVHhR4zj4HChvgE1WT9M5oiL",
				"5GmV6mxKpF3AWWAfgvikkKQhqnnMucTUjE1YhTt56YZ9TGQQ",
				"5Df3w3sAnVoTPtSwaoS4AZJRRXHDDxTgzDZ834K2cosBuJzn",
				"5FWSDLFmqN3z97HGi8EgrpSy9qKU316QfAQ9cQWtDFmMUP9F",
				"5C4hrfkcV8Q2SLRkqBDyWqAJaFmi66cfTtQgcwiHpRFWEULC",
				"5HXR8qQqug9JRAUDNG3UdyEvU7BzzpherctfEbDXbRzfy4Rw",
				"5C4hrfkGSWFRLkLKEkYypTQ9i9NJr2jD72euevVp28gbRDNE",
				"5HhFF5DPFg92k3bN7wibBdHhXoQEMqAToRBaZMfSPZiicd3x",
				"5FZbAfjxCpRPXXpiQ1eAXg4vhLiuwer8L4RPZGLPoh9xCWAi",
				"5GEJnKbHFFXoSYfxuMh9bXQB2vE8UuTSSH5yS2nvF4TXjwrv",
				"5C4hrfkPcF24v5ie2u8VUeaV39dRtspRQ7QJvXF6RoNxrmHL",
				"5E97ao6eksZpeAqZXUhNkWpXNcHrLw48F56wTRFy6e2hGS8J",
				"5FFMF5cSWdHfAoXiB2K598FEdbcq4RJjZhHQAiGjxNo2jBbL",
				"5GkmvC6kpiBGH1yuTBRkJsNaxsKMjfpD6Smvys2WJWLf2x89",
				"5C4hrfkeBsjXtFM2KYophPRPeyWicTTprL8kKB7Rs9gtTgyu",
				"5D5EjB9oVgxbzB8MLFX7P2buoFyGVD3frnngYA2qdEYbAwNh",
				"5C4hrfkL1yVp8J4CHi5b3qKULTXHwsG8UTr659FyctFB9Gvx",
				"5HMqTBKwHNbakaZwdgjKxjGK6tzeq3CVuprutgyB9hRZpGJx",
				"5EjLVB3TL6dz4NQoDnnfTqZrdJwbTtbHZGSz45aUrCFzGtXS",
				"5CJGHiC3LTTAwdisp1BUMi7YGvrytxxoU6JjhDS8yTf7xik4",
			];

			remove_operator_assigned_era_and_staking_ledger::<T>(accounts);

			// Insert cluster 1 data
			insert_cluster_data::<T>(
				0,
				ClusterType::Disabled,
				&[
					"5CSVucKUzY3hKxgXP25DJ642KEwDbe9TaCpeYni8AHpBJ4Wx",
					"5F3tW1dUPvd9h6B8drsY4oTk9v8HCGps8Y4hyRRa4ujZGkdf",
					"5En7dSHaRuSwk8GEiyGKwrstfWmqLCjMVcwrB8cKWsPhghak",
					"5ESA3Pm4tQWvMN3hpCR7z4TfKUGmEt7dGxYVBFs8cfmbAQGe",
					"5DLMmayCoqG5JWSwicQtfD86VZUdP2QPMiTVL5rbnfLfDNMf",
				],
			);

			// Insert cluster 2 data
			insert_cluster_data::<T>(
				1,
				ClusterType::Admin,
				&[
					"5ELdU85Eh9TyRXyWzbSwQvPHGpG2vKLYFsA8sCizx6mHWevg",
					"5EEz4Yg8bxdn5BpM5XdCFmGh7gBHLyVgssAmvBpnqrbD5BwX",
					"5GekFCenV6z5hyu4L8yuWS2F3H9A5B6jfKmFeKzvZCQWkkEM",
					"5F7T7smXKyVaXGDbeurH3iC5kx8srXY3RC6tbV3PQ2X4bMnM",
					"5GHtmbtASxU2nrz62wT8VBmz6SJm3tKZUXgAyCBRmg81wBes",
				],
			);

			// Insert cluster 3 data
			insert_cluster_data::<T>(
				2,
				ClusterType::Admin,
				&[
					"5CkfFDaSNiEYpbfTKfyXVPU1Yp5ziTKhGD4LBBVMmGaRLayp",
					"5EePft2cvAWrQLaqXNmC1zhdkLWjuWD5Atw7xNXUCkY1XEPC",
					"5HhBaok16yFsfQaG8HWDfNzLqFGDTQbD7dbKMAm8c9JdvADN",
					"5GsS2zStdsJAPFTFkStgvPuCP2FRMGC2TDNW6SV27MAesRrs",
					"5Fsy7iHE1c8Lw1Gjgfph6baqXVHPVPiieEVjY7raWJdLC3vS",
				],
			);

			// Insert cluster 4 data
			insert_cluster_data::<T>(
				3,
				ClusterType::Public,
				&[
					"5GddBSC9121ZTGzkfozwb5JqkFwnWaNPB8WmLpJUxdTdabTS",
					"5GhJ2RVC23zkx2ZhA8fBHLmbSPp4isyMQ5XDoibmVVe3HQ6d",
					"5FRhBg7PizTCXx1HdrtT31vMe6jbhkRM1yvSfyBi5UUumzBY",
					"5FbdGaMSKms16jfLX4n4tvfCgRFuG8dLtoLBiQqBJtomH1TU",
					"5Dd5jXUAU3Gxt3oyeSoQ4qy9w3M2hNrjhGnGwtqs1Bwx9BjT",
				],
			);

			// Insert cluster 5 data
			insert_cluster_data::<T>(
				4,
				ClusterType::Public,
				&[
					"5FTaoaJ38Vp5txJ1aeR8oksLiNt2o7TopSwTsdWMhb7c7v6n",
					"5GW9rkdj6qfW1YukvYXSxxNL8aiiGsNV3kz1hT2eJnCy3dLj",
					"5EvYGK8scFtc926ADqmAy2h5jzxy4MV7VjyT7ioFG8a6eUd2",
					"5FnXiYJSjhtFL5UnBKqo99MH7PSNgjUAt2iPxwnqtfUDPDpz",
					"5ENWWScYzUyfxR9vV28DWUfSx6jsZ8ccn992Q4RAshKRmtq8",
				],
			);

			// Insert cluster 6 data
			insert_cluster_data::<T>(5, ClusterType::Public, &[]);

			let data_to_insert = vec![
				("5F7T7smXKyVaXGDbeurH3iC5kx8srXY3RC6tbV3PQ2X4bMnM", 539u32.into()),
				("5FbdGaMSKms16jfLX4n4tvfCgRFuG8dLtoLBiQqBJtomH1TU", 539u32.into()),
				("5EEz4Yg8bxdn5BpM5XdCFmGh7gBHLyVgssAmvBpnqrbD5BwX", 539u32.into()),
				("5HhBaok16yFsfQaG8HWDfNzLqFGDTQbD7dbKMAm8c9JdvADN", 539u32.into()),
				("5GekFCenV6z5hyu4L8yuWS2F3H9A5B6jfKmFeKzvZCQWkkEM", 539u32.into()),
				("5GHtmbtASxU2nrz62wT8VBmz6SJm3tKZUXgAyCBRmg81wBes", 539u32.into()),
				("5FnXiYJSjhtFL5UnBKqo99MH7PSNgjUAt2iPxwnqtfUDPDpz", 539u32.into()),
				("5F3tW1dUPvd9h6B8drsY4oTk9v8HCGps8Y4hyRRa4ujZGkdf", 539u32.into()),
				("5EvYGK8scFtc926ADqmAy2h5jzxy4MV7VjyT7ioFG8a6eUd2", 539u32.into()),
				("5FRhBg7PizTCXx1HdrtT31vMe6jbhkRM1yvSfyBi5UUumzBY", 539u32.into()),
				("5GhJ2RVC23zkx2ZhA8fBHLmbSPp4isyMQ5XDoibmVVe3HQ6d", 539u32.into()),
				("5Fsy7iHE1c8Lw1Gjgfph6baqXVHPVPiieEVjY7raWJdLC3vS", 539u32.into()),
				("5Dd5jXUAU3Gxt3oyeSoQ4qy9w3M2hNrjhGnGwtqs1Bwx9BjT", 539u32.into()),
				("5ESA3Pm4tQWvMN3hpCR7z4TfKUGmEt7dGxYVBFs8cfmbAQGe", 539u32.into()),
				("5DLMmayCoqG5JWSwicQtfD86VZUdP2QPMiTVL5rbnfLfDNMf", 539u32.into()),
				("5GsS2zStdsJAPFTFkStgvPuCP2FRMGC2TDNW6SV27MAesRrs", 539u32.into()),
				("5En7dSHaRuSwk8GEiyGKwrstfWmqLCjMVcwrB8cKWsPhghak", 539u32.into()),
				("5ELdU85Eh9TyRXyWzbSwQvPHGpG2vKLYFsA8sCizx6mHWevg", 539u32.into()),
				("5FTaoaJ38Vp5txJ1aeR8oksLiNt2o7TopSwTsdWMhb7c7v6n", 539u32.into()),
				("5CSVucKUzY3hKxgXP25DJ642KEwDbe9TaCpeYni8AHpBJ4Wx", 539u32.into()),
				("5CkfFDaSNiEYpbfTKfyXVPU1Yp5ziTKhGD4LBBVMmGaRLayp", 539u32.into()),
				("5GddBSC9121ZTGzkfozwb5JqkFwnWaNPB8WmLpJUxdTdabTS", 539u32.into()),
				("5EePft2cvAWrQLaqXNmC1zhdkLWjuWD5Atw7xNXUCkY1XEPC", 539u32.into()),
				("5ENWWScYzUyfxR9vV28DWUfSx6jsZ8ccn992Q4RAshKRmtq8", 539u32.into()),
				("5GW9rkdj6qfW1YukvYXSxxNL8aiiGsNV3kz1hT2eJnCy3dLj", 539u32.into()),
			];

			insert_operator_assigned_era_data::<T>(&data_to_insert);

			let ledger_data_to_insert = vec![
				(
					"5F7T7smXKyVaXGDbeurH3iC5kx8srXY3RC6tbV3PQ2X4bMnM",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5FbdGaMSKms16jfLX4n4tvfCgRFuG8dLtoLBiQqBJtomH1TU",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5EEz4Yg8bxdn5BpM5XdCFmGh7gBHLyVgssAmvBpnqrbD5BwX",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5HhBaok16yFsfQaG8HWDfNzLqFGDTQbD7dbKMAm8c9JdvADN",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GekFCenV6z5hyu4L8yuWS2F3H9A5B6jfKmFeKzvZCQWkkEM",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GHtmbtASxU2nrz62wT8VBmz6SJm3tKZUXgAyCBRmg81wBes",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5FnXiYJSjhtFL5UnBKqo99MH7PSNgjUAt2iPxwnqtfUDPDpz",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5F3tW1dUPvd9h6B8drsY4oTk9v8HCGps8Y4hyRRa4ujZGkdf",
					0u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5EvYGK8scFtc926ADqmAy2h5jzxy4MV7VjyT7ioFG8a6eUd2",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5FRhBg7PizTCXx1HdrtT31vMe6jbhkRM1yvSfyBi5UUumzBY",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GhJ2RVC23zkx2ZhA8fBHLmbSPp4isyMQ5XDoibmVVe3HQ6d",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5Fsy7iHE1c8Lw1Gjgfph6baqXVHPVPiieEVjY7raWJdLC3vS",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5Dd5jXUAU3Gxt3oyeSoQ4qy9w3M2hNrjhGnGwtqs1Bwx9BjT",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5ESA3Pm4tQWvMN3hpCR7z4TfKUGmEt7dGxYVBFs8cfmbAQGe",
					0u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5DLMmayCoqG5JWSwicQtfD86VZUdP2QPMiTVL5rbnfLfDNMf",
					0u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GsS2zStdsJAPFTFkStgvPuCP2FRMGC2TDNW6SV27MAesRrs",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5En7dSHaRuSwk8GEiyGKwrstfWmqLCjMVcwrB8cKWsPhghak",
					0u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5ELdU85Eh9TyRXyWzbSwQvPHGpG2vKLYFsA8sCizx6mHWevg",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5FTaoaJ38Vp5txJ1aeR8oksLiNt2o7TopSwTsdWMhb7c7v6n",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5CSVucKUzY3hKxgXP25DJ642KEwDbe9TaCpeYni8AHpBJ4Wx",
					0u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5CkfFDaSNiEYpbfTKfyXVPU1Yp5ziTKhGD4LBBVMmGaRLayp",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GddBSC9121ZTGzkfozwb5JqkFwnWaNPB8WmLpJUxdTdabTS",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5EePft2cvAWrQLaqXNmC1zhdkLWjuWD5Atw7xNXUCkY1XEPC",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5ENWWScYzUyfxR9vV28DWUfSx6jsZ8ccn992Q4RAshKRmtq8",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
				(
					"5GW9rkdj6qfW1YukvYXSxxNL8aiiGsNV3kz1hT2eJnCy3dLj",
					1500000000000000000000000u128.saturated_into::<BalanceOf<T>>(),
					false,
					0,
				),
			];

			insert_tee_staking_ledger_data::<T>(&ledger_data_to_insert);

			T::DbWeight::get().reads_writes(read, write)
		}
	}

	fn insert_cluster_data<T: Config>(
		cluster_id: ClusterId,
		cluster_type: ClusterType,
		enclave_addresses: &[&str],
	) {
		let mut temp_enclaves = Vec::new(); // Regular Vec for easier manipulation

		for (index, &address) in enclave_addresses.iter().enumerate() {
			let account_id = <crate::Pallet<T>>::convert_str_to_valid_account_id(address).unwrap();
			temp_enclaves.push((account_id, index as SlotId));
		}

		// Convert Vec to BoundedVec
		let enclaves: BoundedVec<(T::AccountId, SlotId), T::ClusterSize> =
			BoundedVec::try_from(temp_enclaves).unwrap();

		let cluster = Cluster::new(enclaves, cluster_type.clone());
		ClusterData::<T>::insert(cluster_id, cluster);
	}

	fn insert_operator_assigned_era_data<T: Config>(operator_era_pairs: &[(&str, EraIndex)]) {
		for (operator, era) in operator_era_pairs.iter() {
			let account_id = <crate::Pallet<T>>::convert_str_to_valid_account_id(operator).unwrap();
			OperatorAssignedEra::<T>::insert(account_id, era);
		}
	}

	fn insert_tee_staking_ledger_data<T: Config>(ledger_data: &[(&str, BalanceOf<T>, bool, u32)]) {
		for &(operator_str, staked_amount, is_unlocking, unbonded_at) in ledger_data.iter() {
			let operator =
				<crate::Pallet<T>>::convert_str_to_valid_account_id(operator_str).unwrap();

			let ledger = TeeStakingLedger::new(
				operator.clone(),
				staked_amount.into(),
				is_unlocking,
				unbonded_at.into(),
			);

			// Assuming you have a storage named `StakingLedgers` for this ledger type.
			StakingLedger::<T>::insert(operator.clone(), ledger);
		}
	}

	fn remove_operator_assigned_era_and_staking_ledger<T: Config>(accounts: &[&str]) {
		for operator in accounts.iter() {
			let account_id = <crate::Pallet<T>>::convert_str_to_valid_account_id(operator).unwrap();
			OperatorAssignedEra::<T>::remove(account_id.clone());
			StakingLedger::<T>::remove(account_id);
		}
	}
}
