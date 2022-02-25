#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::BoundedVec;
use sp_std::vec::Vec;

pub type TextFormat = Vec<u8>;
pub type StringData<Limit> = BoundedVec<u8, Limit>;
