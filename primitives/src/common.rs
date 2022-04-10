#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::BoundedVec;
use sp_std::vec::Vec;

pub type TextFormat = Vec<u8>;
pub type U8BoundedVec<S> = BoundedVec<u8, S>;
