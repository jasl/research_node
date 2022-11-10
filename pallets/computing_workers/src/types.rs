use frame_support::{
	codec::{Decode, Encode, MaxEncodedLen},
	scale_info::TypeInfo,
};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;

/// Worker's status
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum WorkerStatus {
	/// Initial status for a new registered worker.
	Registered,
	/// The worker is online so it can accept job
	/// Transit from `Registered` and `Offline`
	Online,
	/// The worker is offline so it can't accept job.
	/// Transit from `Online`
	Offline,
	/// The worker is pending to deregister
	Deregistering,
}

impl Default for WorkerStatus {
	fn default() -> Self { WorkerStatus::Registered }
}

/// Worker's info.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug, Clone, PartialEq, Eq, Default)]
pub struct WorkerInfo<Account> {
	/// Account that owning the worker, can manage current_account.
	pub(crate) owner: Account,
	/// Account that has permission to operate the worker's working state.
	pub(crate) controller: Account,
	/// Account that holds income and slash,
	/// if its balance lower than `ExistentialDeposit`,
	/// the registration will be revoked, and remaining balance will return to the owner.
	pub(crate) current_account: Account,
	/// Status
	pub(crate) status: WorkerStatus,
}
