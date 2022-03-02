use super::mock::*;
use crate::{tests::mock, Error, Event as NFTsEvent, NFTData, NFTSeriesDetails};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use ternoa_common::traits::NFTTrait;

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
}

fn root() -> mock::Origin {
	RawOrigin::Root.into()
}

mod create {
	use super::*;

	#[test]
	fn create_ok_with_no_series() {
		ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
			// State checks
			assert_eq!(NFTs::nft_id_generator(), 0);
			assert_eq!(NFTs::series_id_generator(), 0);

			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let serie_id = vec![50];
			let ipfs_reference = vec![1];
			let data = NFTData::new_default(ALICE, ipfs_reference.clone(), serie_id.clone());
			let alice_balance = Balances::free_balance(ALICE);

			// Create NFT with new serie id while there is no series already registered
			assert_ok!(NFTs::create(
				alice.clone(),
				data.ipfs_reference.clone(),
				Some(data.series_id.clone()),
			));
			let nft_id = NFTs::nft_id_generator() - 1;

			// Final state checks
			assert_eq!(NFTs::series_id_generator(), 0);
			assert_eq!(nft_id, 0);
			assert_eq!(
				NFTs::series(&data.series_id.clone()),
				Some(NFTSeriesDetails::new(ALICE, true))
			);
			assert_eq!(NFTs::data(0), Some(data.clone()));
			assert_eq!(Balances::free_balance(ALICE), alice_balance - NFTs::nft_mint_fee());

			// Events checks
			assert_eq!(
				System::events().last().unwrap().event,
				Event::NFTs(NFTsEvent::NFTCreated {
					nft_id,
					owner: data.owner,
					series_id: data.series_id,
					ipfs_reference,
					mint_fee: NFTs::nft_mint_fee(),
				})
			);
		})
	}

	#[test]
	fn create_ok_associated_with_existing_serie() {
		ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
			// State checks
			assert_eq!(NFTs::nft_id_generator(), 0);
			assert_eq!(NFTs::series_id_generator(), 0);

			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let serie_id = vec![50];
			let ipfs_1_reference = vec![1];
			let ipfs_2_reference = vec![2];
			let data_1 = NFTData::new_default(ALICE, ipfs_1_reference.clone(), serie_id.clone());
			let data_2 = NFTData::new_default(ALICE, ipfs_2_reference.clone(), serie_id.clone());
			let alice_balance = Balances::free_balance(ALICE);

			// Create NFT with new serie id while there is no series already registered
			assert_ok!(NFTs::create(
				alice.clone(),
				data_1.ipfs_reference.clone(),
				Some(data_1.series_id.clone()),
			));
			let nft_1_id = NFTs::nft_id_generator() - 1;

			// NFT Id check
			assert_eq!(nft_1_id, 0);

			// Events checks
			assert_eq!(
				System::events().last().unwrap().event,
				Event::NFTs(NFTsEvent::NFTCreated {
					nft_id: nft_1_id,
					owner: data_1.owner,
					series_id: data_1.series_id.clone(),
					ipfs_reference: ipfs_1_reference,
					mint_fee: NFTs::nft_mint_fee(),
				})
			);

			// Create NFT associated with existing serie
			assert_ok!(NFTs::create(
				alice.clone(),
				data_2.ipfs_reference.clone(),
				Some(data_2.series_id.clone()),
			));
			let nft_2_id = NFTs::nft_id_generator() - 1;

			// NFT Id check
			assert_eq!(nft_2_id, 1);

			// Events checks
			assert_eq!(
				System::events().last().unwrap().event,
				Event::NFTs(NFTsEvent::NFTCreated {
					nft_id: nft_2_id,
					owner: data_2.owner,
					series_id: data_2.series_id.clone(),
					ipfs_reference: ipfs_2_reference,
					mint_fee: NFTs::nft_mint_fee(),
				})
			);

			// Final state checks
			assert_eq!(NFTs::series_id_generator(), 0);
			assert_eq!(NFTs::series(&data_1.series_id), Some(NFTSeriesDetails::new(ALICE, true)));
			assert_eq!(NFTs::series(&data_2.series_id), Some(NFTSeriesDetails::new(ALICE, true)));
			assert_eq!(NFTs::data(0), Some(data_1.clone()));
			assert_eq!(NFTs::data(1), Some(data_2.clone()));
			assert_eq!(Balances::free_balance(ALICE), alice_balance - NFTs::nft_mint_fee() * 2);
		})
	}

	#[test]
	fn create_error_too_short_name() {
		ExtBuilder::default().caps(vec![(ALICE, 1)]).build().execute_with(|| {
			// State checks
			assert_eq!(NFTs::nft_id_generator(), 0);
			assert_eq!(NFTs::series_id_generator(), 0);

			// Initial state
			let alice: mock::Origin = origin(ALICE);

			// Should fail and storage should remain empty
			let ok = NFTs::create(alice.clone(), vec![], None);
			assert_noop!(ok, Error::<Test>::IPFSReferenceIsTooShort);
		})
	}

	#[test]
	fn create_error_too_long_name() {
		ExtBuilder::default().caps(vec![(ALICE, 1)]).build().execute_with(|| {
			// State checks
			assert_eq!(NFTs::nft_id_generator(), 0);
			assert_eq!(NFTs::series_id_generator(), 0);

			// Initial state
			let alice: mock::Origin = origin(ALICE);

			// Should fail and storage should remain empty
			let ok = NFTs::create(alice.clone(), vec![1, 2, 3, 4, 5, 6], None);
			assert_noop!(ok, Error::<Test>::IPFSReferenceIsTooLong);
		})
	}

	#[test]
	fn create_error_not_enough_caps_to_mint_nft() {
		ExtBuilder::default().caps(vec![(ALICE, 1)]).build().execute_with(|| {
			// State checks
			assert_eq!(NFTs::nft_id_generator(), 0);
			assert_eq!(NFTs::series_id_generator(), 0);

			// Initial state
			let alice: mock::Origin = origin(ALICE);

			// Should fail and storage should remain empty
			let ok = NFTs::create(alice.clone(), vec![1], None);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn create_error_not_the_owner_of_the_serie() {
		ExtBuilder::default()
			.caps(vec![(ALICE, 100), (BOB, 100)])
			.build()
			.execute_with(|| {
				// State checks
				assert_eq!(NFTs::nft_id_generator(), 0);
				assert_eq!(NFTs::series_id_generator(), 0);

				// Initial state
				let bob: mock::Origin = origin(BOB);

				let series_id = Some(vec![50]);
				assert_ok!(NFTs::create(origin(ALICE), vec![50], series_id.clone()));

				// Should fail and storage should remain empty
				assert_noop!(
					NFTs::create(bob.clone(), vec![1], series_id),
					Error::<Test>::NotTheSeriesOwner
				);
				assert_eq!(Balances::free_balance(BOB), 100);
			})
	}

	#[test]
	fn create_error_locked_serie() {
		ExtBuilder::default().caps(vec![(ALICE, 100)]).build().execute_with(|| {
			// State checks
			assert_eq!(NFTs::nft_id_generator(), 0);
			assert_eq!(NFTs::series_id_generator(), 0);

			// Initial state
			let alice: mock::Origin = origin(ALICE);

			let series_id = Some(vec![51]);
			assert_ok!(NFTs::create(alice.clone(), vec![50], series_id.clone()));
			NFTs::finish_series(alice.clone(), series_id.clone().unwrap()).unwrap();

			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::create(alice, vec![1], series_id.clone()),
				Error::<Test>::CannotCreateNFTsWithCompletedSeries
			);
		})
	}
}

mod transfer {
	use super::*;

	#[test]
	fn transfer_ok() {
		ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let series_id = vec![2];
			let nft_id =
				<NFTs as NFTTrait>::create_nft(ALICE, vec![1], Some(series_id.clone())).unwrap();
			NFTs::finish_series(alice.clone(), series_id).unwrap();
			let nft = NFTs::data(nft_id).unwrap();

			// NFT owner and creator check
			assert_eq!(nft.owner, ALICE);
			assert_eq!(nft.creator, ALICE);

			// Transfer nft ownership from ALICE to BOB
			assert_ok!(NFTs::transfer(alice.clone(), nft_id, BOB));

			// Events checks
			assert_eq!(
				System::events().last().unwrap().event,
				Event::NFTs(NFTsEvent::NFTTransferred { nft_id, old_owner: ALICE, new_owner: BOB })
			);

			// Final state checks
			let nft = NFTs::data(nft_id).unwrap();
			assert_eq!(nft.owner, BOB);
			assert_eq!(nft.creator, ALICE);
		})
	}

	#[test]
	fn transfer_error_unknown_nft() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);

			// Try to transfer with an unknown nft id
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(alice.clone(), INVALID_NFT_ID, BOB),
				Error::<Test>::NFTNotFound
			);
		})
	}

	#[test]
	fn transfer_error_uncompleted_serie() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();

			// Try to transfer an nft that is part of an uncompleted serie
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(alice.clone(), nft_id, BOB),
				Error::<Test>::CannotTransferNFTsInUncompletedSeries
			);
		})
	}

	#[test]
	fn transfer_error_listed_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			<NFTs as NFTTrait>::set_listed_for_sale(nft_id, true).unwrap();

			// Try to transfer an nft that is listed for sale
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(alice.clone(), nft_id, BOB),
				Error::<Test>::CannotTransferNFTsListedForSale
			);
		})
	}

	#[test]
	fn transfer_error_converted_to_capsule() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			<NFTs as NFTTrait>::set_converted_to_capsule(nft_id, true).unwrap();

			// Try to transfer an nft that is converted to capsule
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(alice.clone(), nft_id, BOB),
				Error::<Test>::CannotTransferCapsules
			);
		})
	}

	#[test]
	fn transfer_error_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			<NFTs as NFTTrait>::set_in_transmission(nft_id, true).unwrap();

			// Try to transfer an nft that is in transmission
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(alice.clone(), nft_id, BOB),
				Error::<Test>::CannotTransferNFTsInTransmission
			);
		})
	}

	#[test]
	fn transfer_error_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			assert_ok!(NFTs::set_viewer(nft_id, Some(BOB)));

			// Try to transfer a delegated nft
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(origin(ALICE), nft_id, BOB),
				Error::<Test>::CannotTransferDelegatedNFTs
			);
		})
	}
}

mod burn {
	use super::*;

	#[test]
	fn burn_ok() {
		ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![1], Some(vec![2])).unwrap();

			// NFT check
			assert_eq!(NFTs::data(nft_id).is_some(), true);

			// Burning the nft
			assert_ok!(NFTs::burn(alice.clone(), nft_id));

			// Events checks
			assert_eq!(
				System::events().last().unwrap().event,
				Event::NFTs(NFTsEvent::NFTBurned { nft_id })
			);

			// Final state checks
			assert_eq!(NFTs::data(nft_id).is_some(), false);
		})
	}

	#[test]
	fn burn_error_unknown_nft() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);

			// Try to burn an unknown nft
			// Should fail and storage should remain empty
			assert_noop!(NFTs::burn(alice.clone(), INVALID_NFT_ID), Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn burn_error_not_the_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100), (BOB, 100)]).execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = <NFTs as NFTTrait>::create_nft(BOB, vec![1], Some(vec![3])).unwrap();

			// Try to burn an nft but is not the owner
			// Should fail and storage should remain empty
			assert_noop!(NFTs::burn(alice.clone(), nft_id), Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn burn_error_lited_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![1], Some(vec![2])).unwrap();
			<NFTs as NFTTrait>::set_listed_for_sale(nft_id, true).unwrap();

			// Try to burn an nft that is listed for sale
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::burn(alice.clone(), nft_id),
				Error::<Test>::CannotBurnNFTsListedForSale
			);
		})
	}

	#[test]
	fn burn_error_converted_to_capsule() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![1], Some(vec![2])).unwrap();
			<NFTs as NFTTrait>::set_converted_to_capsule(nft_id, true).unwrap();

			// Try to burn an nft that is converted to capsule
			// Should fail and storage should remain empty
			assert_noop!(NFTs::burn(alice.clone(), nft_id), Error::<Test>::CannotBurnCapsules);
		})
	}

	#[test]
	fn burn_error_delegated_nft() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			assert_ok!(NFTs::set_viewer(nft_id, Some(BOB)));

			// Try to burn a delegated nft
			// Should fail and storage should remain empty
			assert_noop!(NFTs::burn(origin(ALICE), nft_id), Error::<Test>::CannotBurnDelegatedNFTs);
		})
	}
}

mod delegate {
	use super::*;

	#[test]
	fn delegate_ok() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			let mut nft = NFTs::data(nft_id).unwrap();
			let viewer = Some(BOB);

			// Delegating nft to another account
			assert_ok!(NFTs::delegate(origin(ALICE), nft_id, viewer.clone()));

			// Events checks
			assert_eq!(
				System::events().last().unwrap().event,
				Event::NFTs(NFTsEvent::NFTDelegated { nft_id, viewer })
			);

			// Final state checks
			nft.viewer = viewer.clone();
			assert_eq!(NFTs::data(nft_id), Some(nft));
		})
	}

	#[test]
	fn delegate_error_unknown_nft() {
		ExtBuilder::new_build(vec![]).execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);

			// Try to delegate an unknown nft
			// Should fail and storage should remain empty
			assert_noop!(NFTs::delegate(alice, INVALID_NFT_ID, None), Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn delegate_error_not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let bob: mock::Origin = origin(BOB);
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();

			// Try to delegate an nft but caller is not the owner
			// Should fail and storage should remain empty
			assert_noop!(NFTs::delegate(bob, nft_id, None), Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn delegate_error_nfts_listed_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			assert_ok!(NFTs::set_listed_for_sale(nft_id, true));

			// Try to delegate an nft that is listed for sale
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::delegate(origin(ALICE), nft_id, None),
				Error::<Test>::CannotDelegateNFTsListedForSale
			);
		})
	}

	#[test]
	fn delegate_error_converted_to_capsule() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			assert_ok!(NFTs::set_converted_to_capsule(nft_id, true));

			// Try to delegate an nft that has been converted to capsule
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::delegate(origin(ALICE), nft_id, None),
				Error::<Test>::CannotDelegateCapsules
			);
		})
	}

	#[test]
	fn delegate_error_nft_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			assert_ok!(NFTs::set_in_transmission(nft_id, true));

			// Try to delegate an nft that is in transmission
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::delegate(origin(ALICE), nft_id, None),
				Error::<Test>::CannotDelegateNFTsInTransmission
			);
		})
	}

	#[test]
	fn delegate_error_to_yourself() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Initial state
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();

			// Try to delegate an nft to yourself
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::delegate(origin(ALICE), nft_id, Some(ALICE)),
				Error::<Test>::CannotDelegateNFTsToYourself
			);
		})
	}
}

mod finish_series {
	use super::*;

	#[test]
	fn finish_series_ok() {
		ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);
			let series_id = vec![50];
			assert_ok!(NFTs::create(origin(ALICE), vec![1], Some(series_id.clone())));
			assert_eq!(NFTs::series(&series_id).unwrap().draft, true);

			// Finish the serie
			assert_ok!(NFTs::finish_series(alice.clone(), series_id.clone()));

			// Events checks
			assert_eq!(
				System::events().last().unwrap().event,
				Event::NFTs(NFTsEvent::SeriesFinished { series_id: series_id.clone() })
			);

			// Final state checks
			assert_eq!(NFTs::series(&series_id).unwrap().draft, false);
		})
	}

	#[test]
	fn finish_series_error_unknown_serie() {
		ExtBuilder::default()
			.caps(vec![(ALICE, 100), (BOB, 100)])
			.build()
			.execute_with(|| {
				// Initial state
				let alice: mock::Origin = origin(ALICE);

				// Try to finish unknown serie
				// Should fail and storage should remain empty
				assert_noop!(
					NFTs::finish_series(alice.clone(), vec![123]),
					Error::<Test>::SeriesNotFound
				);
			})
	}

	#[test]
	fn finish_series_error_not_owner() {
		ExtBuilder::default()
			.caps(vec![(ALICE, 100), (BOB, 100)])
			.build()
			.execute_with(|| {
				// Initial state
				let alice: mock::Origin = origin(ALICE);
				let series_id = vec![3];
				assert_ok!(NFTs::create(origin(BOB), vec![1], Some(series_id.clone())));

				// Try to finish serie as no owner
				// Should fail and storage should remain empty
				assert_noop!(
					NFTs::finish_series(alice.clone(), series_id),
					Error::<Test>::NotTheSeriesOwner
				);
			})
	}
}

mod set_nft_mint_fee {
	use super::*;

	#[test]
	fn set_nft_mint_fee_ok() {
		ExtBuilder::default().build().execute_with(|| {
			// Initial state
			let old_mint_fee = NFTs::nft_mint_fee();
			let new_mint_fee = 654u64;
			assert_eq!(NFTs::nft_mint_fee(), old_mint_fee);

			// Change the mint fee
			assert_ok!(NFTs::set_nft_mint_fee(root(), new_mint_fee));

			// Events checks
			assert_eq!(
				System::events().last().unwrap().event,
				Event::NFTs(NFTsEvent::NFTMintFeeUpdated { fee: new_mint_fee })
			);

			// Final state checks
			assert_eq!(NFTs::nft_mint_fee(), new_mint_fee);
		})
	}

	#[test]
	fn set_nft_mint_fee_error_non_root() {
		ExtBuilder::default().caps(vec![(ALICE, 10000)]).build().execute_with(|| {
			// Initial state
			let alice: mock::Origin = origin(ALICE);

			// Try to change nft mint fee as not root
			// Should fail and storage should remain empty
			assert_noop!(NFTs::set_nft_mint_fee(alice.clone(), 654), BadOrigin);
		})
	}
}
