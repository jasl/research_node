use crate::*;
use frame_support::{
	parameter_types,
	traits::{ConstBool, ConstU32, Nothing},
	weights::Weight,
};
use pallet_contracts::{
	weights::{SubstrateWeight, WeightInfo},
	Config, DefaultAddressGenerator, Frame, Schedule,
};

parameter_types! {
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub const MaxCodeLen: u32 = 128 * 1024;
	// The lazy deletion runs inside on_initialize.
	pub DeletionWeightLimit: Weight = AVERAGE_ON_INITIALIZE_RATIO *
		RuntimeBlockWeights::get().max_block;
	// The weight needed for decoding the queue should be less or equal than a fifth
	// of the overall weight dedicated to the lazy deletion.
	pub DeletionQueueDepth: u32 = ((DeletionWeightLimit::get().ref_time() / (
			<Runtime as Config>::WeightInfo::on_initialize_per_queue_item(1).ref_time() -
			<Runtime as Config>::WeightInfo::on_initialize_per_queue_item(0).ref_time()
		)) / 5) as u32;
	pub MySchedule: Schedule<Runtime> = Default::default();
}

impl Config for Runtime {
	type Time = Timestamp;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	/// The safest default is to allow no calls at all.
	///
	/// Runtimes should whitelist dispatchables that are allowed to be called from contracts
	/// and make sure they are stable. Dispatchables exposed to contracts are not allowed to
	/// change because that would break already deployed contracts. The `Call` structure itself
	/// is not allowed to change the indices of existing pallet_configs, too.
	type CallFilter = Nothing;
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = SubstrateWeight<Self>;
	type ChainExtension = ();
	type Schedule = MySchedule;
	type CallStack = [Frame<Self>; 31];
	type DeletionQueueDepth = DeletionQueueDepth;
	type DeletionWeightLimit = DeletionWeightLimit;
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type AddressGenerator = DefaultAddressGenerator;
	type MaxCodeLen = MaxCodeLen;
	type MaxStorageKeyLen = ConstU32<128>;
	type UnsafeUnstableInterface = ConstBool<false>;
}
