use crate::*;
use sp_core::{ConstBool, ConstU128, ConstU32};

impl pallet_computing_workers::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type UnixTime = Timestamp;
	type MarkingOfflinePerBlockLimit = ConstU32<20>;
	type MarkingUnresponsivePerBlockLimit = ConstU32<20>;
	type ReservedDeposit = ConstU128<{ 100 * UNITS }>;
	type CollectingHeartbeatsDuration = ConstU32<10>; // TODO: ConstU32<240>; // 240 block * 6 sec / 60 sec = 24 min
	type AttestationValidityDuration = ConstU32<43200>; // 10 B/S * 60 min * 24 hour * 3 days = 43200 block
	type DisallowOptOutAttestation = ConstBool<true>;
	type DisallowNonTEEAttestation = ConstBool<false>;
}
