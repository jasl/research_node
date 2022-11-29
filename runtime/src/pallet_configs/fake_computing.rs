use crate::*;
use frame_support::parameter_types;

parameter_types! {
	pub const SlashingCardinal: Balance = UNITS;
}

impl pallet_fake_computing::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WorkerManageable = ComputingWorkers;
	type SlashingCardinal = SlashingCardinal;
}
