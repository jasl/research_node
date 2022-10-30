use crate::*;
use frame_support::{
	parameter_types,
	traits::{ConstU32, EqualPrivilegeOnly},
};
use frame_system::EnsureRoot;

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type MaxScheduledPerBlock = ConstU32<512>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type Preimages = Preimage;
}
