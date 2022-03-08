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
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let data = NFTData::new_default(ALICE, vec![1], vec![50]);
			let alice_balance = Balances::free_balance(ALICE);

			// Create NFT with new serie id while there is no series already registered
			let ok = NFTs::create(
				origin(ALICE),
				data.ipfs_reference.clone(),
				Some(data.series_id.clone()),
			);
			assert_ok!(ok);
			let nft_id = NFTs::nft_id_generator() - 1;

			// Final state checks
			assert_eq!(NFTs::series_id_generator(), 0);
			assert_eq!(nft_id, 0);
			assert_eq!(NFTs::series(&data.series_id), Some(NFTSeriesDetails::new(ALICE, true)));
			assert_eq!(NFTs::data(0), Some(data.clone()));
			assert_eq!(Balances::free_balance(ALICE), alice_balance - NFTs::nft_mint_fee());

			// Events checks
			let event = NFTsEvent::NFTCreated {
				nft_id,
				owner: data.owner,
				series_id: data.series_id,
				ipfs_reference: data.ipfs_reference,
				mint_fee: NFTs::nft_mint_fee(),
			};
			let event = Event::NFTs(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn create_ok_associated_with_existing_serie() {
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let data_1 = NFTData::new_default(ALICE, vec![1], vec![50]);
			let data_2 = NFTData::new_default(ALICE, vec![2], vec![50]);
			let alice_balance = Balances::free_balance(ALICE);

			// Create NFT with new serie id while there is no series already registered
			let ok = NFTs::create(
				alice.clone(),
				data_1.ipfs_reference.clone(),
				Some(data_1.series_id.clone()),
			);
			assert_ok!(ok);
			let nft_1_id = NFTs::nft_id_generator() - 1;

			// NFT Id check
			assert_eq!(nft_1_id, 0);

			// Events checks
			let event = NFTsEvent::NFTCreated {
				nft_id: nft_1_id,
				owner: data_1.owner,
				series_id: data_1.series_id.clone(),
				ipfs_reference: data_1.ipfs_reference.clone(),
				mint_fee: NFTs::nft_mint_fee(),
			};
			let event = Event::NFTs(event);
			assert_eq!(System::events().last().unwrap().event, event);

			// Create NFT associated with existing serie
			let ok = NFTs::create(
				alice.clone(),
				data_2.ipfs_reference.clone(),
				Some(data_2.series_id.clone()),
			);
			assert_ok!(ok);
			let nft_2_id = NFTs::nft_id_generator() - 1;

			// NFT Id check
			assert_eq!(nft_2_id, 1);

			// Final state checks
			assert_eq!(NFTs::series_id_generator(), 0);
			let nft_series_details = Some(NFTSeriesDetails::new(ALICE, true));
			assert_eq!(NFTs::series(&data_1.series_id), nft_series_details);
			assert_eq!(NFTs::series(&data_2.series_id), nft_series_details.clone());
			assert_eq!(NFTs::data(0), Some(data_1));
			assert_eq!(NFTs::data(1), Some(data_2.clone()));
			assert_eq!(Balances::free_balance(ALICE), alice_balance - NFTs::nft_mint_fee() * 2);

			// Events checks
			let event = NFTsEvent::NFTCreated {
				nft_id: nft_2_id,
				owner: data_2.owner,
				series_id: data_2.series_id,
				ipfs_reference: data_2.ipfs_reference,
				mint_fee: NFTs::nft_mint_fee(),
			};
			let event = Event::NFTs(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn ipfs_reference_is_too_short() {
		ExtBuilder::new_build(vec![(ALICE, 1)]).execute_with(|| {
			// Should fail and storage should remain empty
			let ok = NFTs::create(origin(ALICE), vec![], None);
			assert_noop!(ok, Error::<Test>::IPFSReferenceIsTooShort);
		})
	}

	#[test]
	fn ipfs_reference_is_too_long() {
		ExtBuilder::new_build(vec![(ALICE, 1)]).execute_with(|| {
			// Should fail and storage should remain empty
			let ok = NFTs::create(origin(ALICE), vec![1, 2, 3, 4, 5, 6], None);
			assert_noop!(ok, Error::<Test>::IPFSReferenceIsTooLong);
		})
	}

	#[test]
	fn insufficient_balance() {
		ExtBuilder::new_build(vec![(ALICE, 1)]).execute_with(|| {
			// Should fail and storage should remain empty
			let ok = NFTs::create(origin(ALICE), vec![1], None);
			assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);
		})
	}

	#[test]
	fn not_the_series_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100), (BOB, 100)]).execute_with(|| {
			let series_id = Some(vec![50]);
			let ok = NFTs::create(origin(ALICE), vec![50], series_id.clone());
			assert_ok!(ok);

			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::create(origin(BOB), vec![1], series_id),
				Error::<Test>::NotTheSeriesOwner
			);
			assert_eq!(Balances::free_balance(BOB), 100);
		})
	}

	#[test]
	fn cannot_create_nft_with_completed_series() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);

			let series_id = Some(vec![51]);
			let ok = NFTs::create(alice.clone(), vec![50], series_id.clone());
			assert_ok!(ok);
			let ok = NFTs::finish_series(alice.clone(), series_id.clone().unwrap());
			assert_ok!(ok);

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
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
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
			let ok = NFTs::transfer(alice.clone(), nft_id, BOB);
			assert_ok!(ok);

			// Final state checks
			let nft = NFTs::data(nft_id).unwrap();
			assert_eq!(nft.owner, BOB);
			assert_eq!(nft.creator, ALICE);

			// Events checks
			let event = NFTsEvent::NFTTransferred { nft_id, old_owner: ALICE, new_owner: BOB };
			let event = Event::NFTs(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Try to transfer with an unknown nft id
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(origin(ALICE), INVALID_NFT_ID, BOB),
				Error::<Test>::NFTNotFound
			);
		})
	}

	#[test]
	fn cannot_transfer_nfts_in_uncompleted_series() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();

			// Try to transfer an nft that is part of an uncompleted serie
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(origin(ALICE), nft_id, BOB),
				Error::<Test>::CannotTransferNFTsInUncompletedSeries
			);
		})
	}

	#[test]
	fn cannot_transfer_nfts_listed_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			<NFTs as NFTTrait>::set_listed_for_sale(nft_id, true).unwrap();

			// Try to transfer an nft that is listed for sale
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(origin(ALICE), nft_id, BOB),
				Error::<Test>::CannotTransferNFTsListedForSale
			);
		})
	}

	#[test]
	fn cannot_transfer_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			<NFTs as NFTTrait>::set_converted_to_capsule(nft_id, true).unwrap();

			// Try to transfer an nft that is converted to capsule
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(origin(ALICE), nft_id, BOB),
				Error::<Test>::CannotTransferCapsules
			);
		})
	}

	#[test]
	fn cannot_transfer_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			<NFTs as NFTTrait>::set_in_transmission(nft_id, true).unwrap();

			// Try to transfer an nft that is in transmission
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::transfer(origin(ALICE), nft_id, BOB),
				Error::<Test>::CannotTransferNFTsInTransmission
			);
		})
	}

	#[test]
	fn cannot_transfer_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			let ok = NFTs::set_viewer(nft_id, Some(BOB));
			assert_ok!(ok);

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
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![1], Some(vec![2])).unwrap();

			// NFT check
			assert_eq!(NFTs::data(nft_id).is_some(), true);

			// Burning the nft
			let ok = NFTs::burn(origin(ALICE), nft_id);
			assert_ok!(ok);

			// Final state checks
			assert_eq!(NFTs::data(nft_id).is_some(), false);

			// Events checks
			let event = NFTsEvent::NFTBurned { nft_id };
			let event = Event::NFTs(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Try to burn an unknown nft
			// Should fail and storage should remain empty
			assert_noop!(NFTs::burn(origin(ALICE), INVALID_NFT_ID), Error::<Test>::NFTNotFound);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100), (BOB, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(BOB, vec![1], Some(vec![3])).unwrap();

			// Try to burn an nft but is not the owner
			// Should fail and storage should remain empty
			assert_noop!(NFTs::burn(origin(ALICE), nft_id), Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_burn_nfts_listed_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
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
	fn cannot_burn_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![1], Some(vec![2])).unwrap();
			<NFTs as NFTTrait>::set_converted_to_capsule(nft_id, true).unwrap();

			// Try to burn an nft that is converted to capsule
			// Should fail and storage should remain empty
			assert_noop!(NFTs::burn(origin(ALICE), nft_id), Error::<Test>::CannotBurnCapsules);
		})
	}

	#[test]
	fn cannot_burn_delegated_nfts() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			let ok = NFTs::set_viewer(nft_id, Some(BOB));
			assert_ok!(ok);

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
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			let mut nft = NFTs::data(nft_id).unwrap();
			let viewer = Some(BOB);

			// Delegating nft to another account
			let ok = NFTs::delegate(origin(ALICE), nft_id, viewer.clone());
			assert_ok!(ok);

			// Final state checks
			nft.viewer = viewer.clone();
			assert_eq!(NFTs::data(nft_id), Some(nft));

			// Events checks
			let event = NFTsEvent::NFTDelegated { nft_id, viewer };
			let event = Event::NFTs(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn nft_not_found() {
		ExtBuilder::new_build(vec![]).execute_with(|| {
			// Try to delegate an unknown nft
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::delegate(origin(ALICE), INVALID_NFT_ID, None),
				Error::<Test>::NFTNotFound
			);
		})
	}

	#[test]
	fn not_the_nft_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();

			// Try to delegate an nft but caller is not the owner
			// Should fail and storage should remain empty
			assert_noop!(NFTs::delegate(origin(BOB), nft_id, None), Error::<Test>::NotTheNFTOwner);
		})
	}

	#[test]
	fn cannot_delegate_nfts_listed_for_sale() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			let ok = NFTs::set_listed_for_sale(nft_id, true);
			assert_ok!(ok);

			// Try to delegate an nft that is listed for sale
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::delegate(origin(ALICE), nft_id, None),
				Error::<Test>::CannotDelegateNFTsListedForSale
			);
		})
	}

	#[test]
	fn cannot_delegate_capsules() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			let ok = NFTs::set_converted_to_capsule(nft_id, true);
			assert_ok!(ok);

			// Try to delegate an nft that has been converted to capsule
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::delegate(origin(ALICE), nft_id, None),
				Error::<Test>::CannotDelegateCapsules
			);
		})
	}

	#[test]
	fn cannot_delegate_nfts_in_transmission() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let nft_id = <NFTs as NFTTrait>::create_nft(ALICE, vec![0], None).unwrap();
			let ok = NFTs::set_in_transmission(nft_id, true);
			assert_ok!(ok);

			// Try to delegate an nft that is in transmission
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::delegate(origin(ALICE), nft_id, None),
				Error::<Test>::CannotDelegateNFTsInTransmission
			);
		})
	}

	#[test]
	fn cannot_delegate_nfts_to_yourself() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
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
		ExtBuilder::new_build(vec![(ALICE, 1000)]).execute_with(|| {
			let alice: mock::Origin = origin(ALICE);
			let series_id = vec![50];
			let ok = NFTs::create(alice.clone(), vec![1], Some(series_id.clone()));
			assert_ok!(ok);
			assert_eq!(NFTs::series(&series_id).unwrap().draft, true);

			// Finish the serie
			let ok = NFTs::finish_series(alice.clone(), series_id.clone());
			assert_ok!(ok);

			// Final state checks
			assert_eq!(NFTs::series(&series_id).unwrap().draft, false);

			// Events checks
			let event = NFTsEvent::SeriesFinished { series_id: series_id.clone() };
			let event = Event::NFTs(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn series_not_found() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			// Try to finish unknown serie
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::finish_series(origin(ALICE), vec![123]),
				Error::<Test>::SeriesNotFound
			);
		})
	}

	#[test]
	fn not_the_series_owner() {
		ExtBuilder::new_build(vec![(ALICE, 100), (BOB, 100)]).execute_with(|| {
			let series_id = vec![3];
			let ok = NFTs::create(origin(BOB), vec![1], Some(series_id.clone()));
			assert_ok!(ok);

			// Try to finish serie as no owner
			// Should fail and storage should remain empty
			assert_noop!(
				NFTs::finish_series(origin(ALICE), series_id),
				Error::<Test>::NotTheSeriesOwner
			);
		})
	}
}

mod set_nft_mint_fee {
	use super::*;

	#[test]
	fn set_nft_mint_fee_ok() {
		ExtBuilder::new_build(vec![]).execute_with(|| {
			let old_mint_fee = NFTs::nft_mint_fee();
			let new_mint_fee = 654u64;
			assert_eq!(NFTs::nft_mint_fee(), old_mint_fee);

			// Change the mint fee
			let ok = NFTs::set_nft_mint_fee(root(), new_mint_fee);
			assert_ok!(ok);

			// Final state checks
			assert_eq!(NFTs::nft_mint_fee(), new_mint_fee);

			// Events checks
			let event = NFTsEvent::NFTMintFeeUpdated { fee: new_mint_fee };
			let event = Event::NFTs(event);
			assert_eq!(System::events().last().unwrap().event, event);
		})
	}

	#[test]
	fn bad_origin() {
		ExtBuilder::new_build(vec![(ALICE, 10000)]).execute_with(|| {
			// Try to change nft mint fee as not root
			// Should fail and storage should remain empty
			assert_noop!(NFTs::set_nft_mint_fee(origin(ALICE), 654), BadOrigin);
		})
	}
}
