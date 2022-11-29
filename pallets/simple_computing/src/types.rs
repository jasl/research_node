use scale_codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use frame_support::{
	sp_std::prelude::*,
	sp_runtime::Saturating,
	BoundedVec,
	RuntimeDebug,
};

use crate::macros::impl_auto_incremental;

pub trait AutoIncremental {
	fn increment(&self) -> Self;
	fn initial_value() -> Self;
}
impl_auto_incremental!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum JobStatus {
	Created,
	Enqueued,
	Started,
	Success,
	Failed,
	Timeout,
	Cancelled,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct Job<Account, BlockNumber> {
	pub status: JobStatus,
	pub created_by: Account,
	pub created_at: Option<BlockNumber>,
	pub enqueued_at: Option<BlockNumber>,
	pub started_at: Option<BlockNumber>,
	pub completed_at: Option<BlockNumber>,
	// pub payload
}
