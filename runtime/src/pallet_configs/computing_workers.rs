use crate::*;
use sp_core::{ConstBool, ConstU128, ConstU32};

impl pallet_computing_workers::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UnixTime = Timestamp;
	type MaxPendingOfflineWorkers = ConstU32<1024>;
	type ReservedDeposit = ConstU128<{ 100 * UNITS }>;
	type CollectingHeartbeatsDuration = ConstU32<120>; // 120 block * 6 sec = 720 sec
	type AttestationValidityDuration = ConstU32<43200>; // 10 b/s * 60 min * 24 hour * 3 days = 43200
	type DisallowOptOutAttestation = ConstBool<true>;
	type DisallowNonTEEAttestation = ConstBool<false>;
}
