use sp_core::{ConstU128, ConstBool};
use crate::*;

impl pallet_computing_workers::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ReservedDeposit = ConstU128<{ 100 * UNITS }>;
	type AllowNoneAttestation = ConstBool<true>;
}
