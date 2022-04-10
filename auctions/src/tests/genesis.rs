use super::mock::*;
use crate::{
	types::{AuctionData, BidderList, DeadlineList},
	GenesisConfig,
};
use frame_support::{bounded_vec, traits::GenesisBuild};

#[test]
fn genesis() {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let auction: AuctionData<AccountId, BlockNumber, u128, BidderListLengthLimit> = AuctionData {
		creator: ALICE,
		start_block: 10,
		end_block: 20,
		start_price: 10,
		buy_it_price: Some(20),
		bidders: BidderList::new(),
		marketplace_id: ALICE_MARKET_ID,
		is_extended: false,
	};

	let deadlines = DeadlineList(bounded_vec![(ALICE_NFT_ID, auction.end_block)]);
	let auctions = vec![auction.clone().to_raw(ALICE_NFT_ID)];

	GenesisConfig::<Test> { auctions: auctions.clone() }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		for auction in auctions {
			let nft_id = auction.0;
			let auction = AuctionData::from_raw(auction);
			assert_eq!(Auctions::auctions(nft_id), Some(auction));
		}
		assert_eq!(Auctions::deadlines(), deadlines);
	});
}
