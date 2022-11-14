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
	/// The worker's info expired that need to do a refreshing,
	/// the worker won't accept new job, accepted jobs will still processing.
	/// Transit from `Online`
	RefreshRegistrationRequired,
	/// The worker is requesting offline,
	/// the worker won't accept new job, accepted jobs will still processing,
	/// when accepted jobs processed it can be transited to `Offline` safely without slashing.
	/// Transit from `Online`
	RequestingOffline,
	/// The worker is online so it can accept job
	/// Transit from `Registered`, `Offline`, `RefreshRegistrationRequired`,
	/// not sure `RequestingOffline`
	Online,
	/// The worker is offline so it can't accept job.
	/// Transit from `RequestingOffline`, and `Online` (when slashing)
	Offline,
	/// The worker is pending to deregister
	Deregistering,
}

impl Default for WorkerStatus {
	fn default() -> Self { WorkerStatus::Registered }
}

/// The type of how the worker do attestation
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AttestationType {
	/// Attest by root authority
	Root,
	// TODO: Intel SGX (EPID, ECDSA), AMD SEV, etc.
}

/// Worker's attestation
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Attestation {
	/// No attestation
	None,
	// TODO: Intel SGX (EPID, ECDSA), AMD SEV, etc.
}

/// Worker's info.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug, Clone, PartialEq, Eq, Default)]
pub struct WorkerInfo<Account, Balance, BlockNumber> {
	/// Account that generated by the worker app, used for identity the worker and send extrinsic.
	/// This field is readonly once set
	pub identity: Account,
	/// Account that owning the worker.
	/// This field is readonly once set
	pub owner: Account,
	/// Reserved balance on register.
	/// This field is readonly once set
	pub reserved: Balance,
	/// Status
	pub status: WorkerStatus,
	/// Not the public version exposed to end users,
	/// Spec version is a sequential number that space and use friendly
	pub spec_version: u32,
	/// Attestation type,
	/// This field is readonly once set
	pub attestation_type: Option<AttestationType>,
	/// A block number of when the worker update the info.
	/// It may be 0 in case the worker hasn't submit
	pub updated_at: BlockNumber,
	/// A block number of when the worker's info expires.
	/// The worker must refresh the info before the block
	pub expiring_at: BlockNumber,
}
