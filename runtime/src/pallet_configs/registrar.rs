use sp_core::ConstU128;
use crate::*;

impl pallet_registrar::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ExistentialDeposit = ConstU128<{ 100 * DOLLARS }>;
}
