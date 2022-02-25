#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{traits::ConstU32, BoundedVec};
use sp_std::vec::Vec;

pub type TextFormat = Vec<u8>;
pub type IPFSString = BoundedVec<u8, ConstU32<50>>;
//pub type IPFSString = BoundedVec<u8, ConstU32<64>>;
