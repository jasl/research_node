use crate::*;

impl pallet_fake_computing::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WorkerManageable = ComputingWorkers;
}
