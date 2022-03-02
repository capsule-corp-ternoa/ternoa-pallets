use super::mock::*;
use crate::{tests::mock, Error, Event as NFTsEvent, NFTData, NFTSeriesDetails};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;
use ternoa_common::traits::NFTTrait;

fn origin(account: u64) -> mock::Origin {
	RawOrigin::Signed(account).into()
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

			// create() should fail and return the proper error
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

			// create() should fail and return the proper error
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

			// create() should fail and return the proper error
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

				// create() should fail and return the proper error
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

			// create() should fail and return the proper error
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
			assert_noop!(NFTs::transfer(alice.clone(), 1001, BOB), Error::<Test>::NFTNotFound);
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
	fn burn_happy() {
		ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
			let alice: mock::Origin = origin(ALICE);

			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![1], Some(vec![2])).unwrap();
			assert_eq!(NFTs::data(nft_id).is_some(), true);

			assert_ok!(NFTs::burn(alice.clone(), nft_id));
			assert_eq!(NFTs::data(nft_id).is_some(), false);
		})
	}

	#[test]
	fn burn_unhappy() {
		ExtBuilder::default()
			.caps(vec![(ALICE, 100), (BOB, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::Origin = origin(ALICE);

				// Unhappy unknown NFT
				let ok = NFTs::burn(alice.clone(), 10001);
				assert_noop!(ok, Error::<Test>::NFTNotFound);

				// Unhappy not the owner
				let nft_id = <NFTs as NFTTrait>::create_nft(BOB, vec![1], Some(vec![3])).unwrap();
				let ok = NFTs::burn(alice.clone(), nft_id);
				assert_noop!(ok, Error::<Test>::NotTheNFTOwner);

				// Unhappy listed for sale
				let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![1], Some(vec![2])).unwrap();
				<NFTs as NFTTrait>::set_listed_for_sale(nft_id, true).unwrap();

				let ok = NFTs::burn(alice.clone(), nft_id);
				assert_noop!(ok, Error::<Test>::CannotBurnNFTsListedForSale);

				// Unhappy converted to capsule
				let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![1], Some(vec![2])).unwrap();
				<NFTs as NFTTrait>::set_converted_to_capsule(nft_id, true).unwrap();

				let ok = NFTs::burn(alice.clone(), nft_id);
				assert_noop!(ok, Error::<Test>::CannotBurnCapsules);
			})
	}

	#[test]
	fn cannot_burn_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			assert_ok!(NFTs::set_viewer(nft_id, Some(BOB)));

			let ok = NFTs::burn(origin(ALICE), nft_id);
			assert_noop!(ok, Error::<Test>::CannotBurnDelegatedNFTs);
		})
	}
}

mod delegate {
	use super::*;

	#[test]
	fn delegate() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			let mut nft = NFTs::data(nft_id).unwrap();
			let viewer = Some(BOB);

			assert_ok!(NFTs::delegate(origin(ALICE), nft_id, viewer.clone()));

			// Storage
			nft.viewer = viewer.clone();
			assert_eq!(NFTs::data(nft_id), Some(nft));

			// Event
			let event = NFTsEvent::NFTDelegated { nft_id, viewer };
			let event = Event::NFTs(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![]).execute_with(|| {
			let ok = NFTs::delegate(origin(ALICE), INVALID_NFT_ID, None);
			assert_noop!(ok, Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();

			let ok = NFTs::delegate(origin(BOB), nft_id, None);
			assert_noop!(ok, Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_delegate_nfts_listed_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			assert_ok!(NFTs::set_listed_for_sale(nft_id, true));

			let ok = NFTs::delegate(origin(ALICE), nft_id, None);
			assert_noop!(ok, Error::<Test>::CannotDelegateNFTsListedForSale);
		})
	}

	#[test]
	fn cannot_delegate_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			assert_ok!(NFTs::set_converted_to_capsule(nft_id, true));

			let ok = NFTs::delegate(origin(ALICE), nft_id, None);
			assert_noop!(ok, Error::<Test>::CannotDelegateCapsules);
		})
	}

	#[test]
	fn cannot_delegate_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			assert_ok!(NFTs::set_in_transmission(nft_id, true));

			let ok = NFTs::delegate(origin(ALICE), nft_id, None);
			assert_noop!(ok, Error::<Test>::CannotDelegateNFTsInTransmission);
		})
	}

	#[test]
	fn cannot_delegate_nfts_to_yourself() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();

			let ok = NFTs::delegate(origin(ALICE), nft_id, Some(ALICE));
			assert_noop!(ok, Error::<Test>::CannotDelegateNFTsToYourself);
		})
	}
}

mod finish_series {
	use super::*;

	#[test]
	fn finish_series_happy() {
		ExtBuilder::default().caps(vec![(ALICE, 1000)]).build().execute_with(|| {
			let alice: mock::Origin = origin(ALICE);

			let series_id = vec![50];
			assert_ok!(NFTs::create(origin(ALICE), vec![1], Some(series_id.clone())));
			assert_eq!(NFTs::series(&series_id).unwrap().draft, true);

			assert_ok!(NFTs::finish_series(alice.clone(), series_id.clone()));
			assert_eq!(NFTs::series(&series_id).unwrap().draft, false);
		})
	}

	#[test]
	fn finish_series_unhappy() {
		ExtBuilder::default()
			.caps(vec![(ALICE, 100), (BOB, 100)])
			.build()
			.execute_with(|| {
				let alice: mock::Origin = origin(ALICE);

				// Unhappy series id not found
				let ok = NFTs::finish_series(alice.clone(), vec![123]);
				assert_noop!(ok, Error::<Test>::SeriesNotFound);

				// Unhappy not series owner
				let series_id = vec![3];
				assert_ok!(NFTs::create(origin(BOB), vec![1], Some(series_id.clone())));
				let ok = NFTs::finish_series(alice.clone(), series_id);
				assert_noop!(ok, Error::<Test>::NotTheSeriesOwner);
			})
	}
}

mod set_nft_mint_fee {
	use super::*;

	#[test]
	fn set_nft_mint_fee_happy() {
		ExtBuilder::default().build().execute_with(|| {
			// Happy path
			let old_mint_fee = NFTs::nft_mint_fee();
			let new_mint_fee = 654u64;
			assert_eq!(NFTs::nft_mint_fee(), old_mint_fee);

			let ok = NFTs::set_nft_mint_fee(mock::Origin::root(), new_mint_fee);
			assert_ok!(ok);
			assert_eq!(NFTs::nft_mint_fee(), new_mint_fee);
		})
	}

	#[test]
	fn set_nft_mint_fee_unhappy() {
		ExtBuilder::default().caps(vec![(ALICE, 10000)]).build().execute_with(|| {
			let alice: mock::Origin = origin(ALICE);

			// Unhappy non root user tries to modify the mint fee
			let ok = NFTs::set_nft_mint_fee(alice.clone(), 654);
			assert_noop!(ok, BadOrigin);
		})
	}
}
