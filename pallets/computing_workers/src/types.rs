use frame_support::{
	codec::{Decode, Encode, MaxEncodedLen},
	scale_info::TypeInfo,
	traits::ConstU32,
	BoundedVec,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub const ATTESTATION_MATERIAL_ISSUED_PERIOD_OF_VALIDITY: u64 = 60 * 60 * 1000; // 1 hour
pub const MAX_ATTESTATION_PAYLOAD_SIZE: u32 = 64 * 1000; // limit to 64KB
pub const MAX_CUSTOM_PAYLOAD_SIZE: u32 = 64 * 1000; // limit to 64KB

pub type AttestationPayload = BoundedVec<u8, ConstU32<MAX_ATTESTATION_PAYLOAD_SIZE>>;

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct NonTEEAttestationMaterial {
	pub issued_at: u64,
	pub payload: AttestationPayload,
}

/// The type of how the worker do attestation
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AttestationMethod {
	/// Attest by dev mode
	NonTEE,
	/// Attest by root authority
	Root,
	// TODO: Intel SGX (EPID, ECDSA), AMD SEV, etc.
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum AttestationError {
	Invalid,
	Expired,
}

/// Worker's attestation
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Attestation {
	NonTEE(NonTEEAttestationMaterial),
	// TODO: Intel SGX (EPID, ECDSA), AMD SEV, etc.
}
impl Attestation {
	pub fn method(&self) -> AttestationMethod {
		match self {
			Attestation::NonTEE(..) => AttestationMethod::NonTEE,
		}
	}

	pub fn verify(&self, now: u64) -> Result<VerifiedAttestation, AttestationError> {
		match self {
			Attestation::NonTEE(material) =>
				verify_non_tee_attestation(material, now).map(|_| Ok(VerifiedAttestation(self)))?,
		}
	}

	pub(self) fn payload(&self) -> &[u8] {
		match self {
			Attestation::NonTEE(material) => material.payload.as_slice(),
		}
	}
}

#[derive(Clone, PartialEq, RuntimeDebug)]
pub struct VerifiedAttestation<'a>(&'a Attestation);
impl VerifiedAttestation<'_> {
	pub fn method(&self) -> AttestationMethod {
		self.0.method()
	}

	pub fn payload(&self) -> &[u8] {
		self.0.payload()
	}
}

fn verify_non_tee_attestation(material: &NonTEEAttestationMaterial, now: u64) -> Result<(), AttestationError> {
	let period = now - material.issued_at;
	if period > ATTESTATION_MATERIAL_ISSUED_PERIOD_OF_VALIDITY {
		Err(AttestationError::Expired)
	} else {
		Ok(())
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct OnlinePayload {
	pub spec_version: u32,
	pub extra: BoundedVec<u8, ConstU32<MAX_CUSTOM_PAYLOAD_SIZE>>,
}

/// Worker's status
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum WorkerStatus {
	/// Initial status for a new registered worker.
	Registered,
	/// The worker's attestation expired that need to do a refreshing,
	/// the worker won't accept new job, accepted jobs will still processing.
	/// Transit from `Online`
	AttestationExpired,
	/// The worker is requesting offline,
	/// the worker won't accept new job, accepted jobs will still processing,
	/// when accepted jobs processed it can be transited to `Offline` safely without slashing.
	/// Transit from `Online`
	RequestingOffline,
	/// The worker not sent heartbeat during a period, It shall be moved to pending offline queue
	/// Transit from `Online`
	Unresponsive,
	/// The worker is online so it can accept job
	/// Transit from `Registered`, `Offline`, `RefreshRegistrationRequired`, and `Unresponsive`
	/// not sure `RequestingOffline`
	Online,
	/// The worker is offline so it can't accept job.
	/// Transit from `RequestingOffline`, `RefreshRegistrationRequired` when job queue cleared,
	/// and `Unresponsive` (will be slashed), `Online` (when force)
	Offline,
}

impl Default for WorkerStatus {
	fn default() -> Self {
		WorkerStatus::Registered
	}
}

/// Worker's info.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug, Clone, PartialEq, Eq, Default)]
pub struct WorkerInfo<Account, Balance, BlockNumber> {
	/// Account that generated by the worker app, used for identity the worker and send extrinsic.
	/// This field is readonly once set
	pub account: Account,
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
	/// Attestation method,
	/// This field is readonly once set
	pub attestation_method: Option<AttestationMethod>,
	/// A block number of when the worker refresh its attestation.
	/// It may be 0 in case the worker hasn't submit
	pub attested_at: BlockNumber,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug, Copy, Clone, PartialEq, Eq)]
pub enum FlipFlopStage {
	Flip,
	Flop,
	FlipToFlop,
	FlopToFlip,
}

impl Default for FlipFlopStage {
	fn default() -> Self {
		FlipFlopStage::Flip
	}
}
