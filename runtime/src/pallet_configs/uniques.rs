use crate::*;
use frame_support::{parameter_types, traits::AsEnsureOriginWithArg};
use frame_system::{EnsureRoot, EnsureSigned};

parameter_types! {
	pub const CollectionDeposit: Balance = 10 * UNITS; // 10 UNIT deposit to create uniques class
	pub const ItemDeposit: Balance = UNITS / 100; // 1 / 100 UNIT deposit to create uniques instance
	pub const KeyLimit: u32 = 32; // Max 32 bytes per key
	pub const ValueLimit: u32 = 64; // Max 64 bytes per value
	pub const UniquesMetadataDepositBase: Balance = deposit(1, 129);
	pub const AttributeDepositBase: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub const UniquesStringLimit: u32 = 128;
}

impl pallet_uniques::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = CollectionId;
	type ItemId = ItemId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = UniquesMetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = UniquesStringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type WeightInfo = pallet_uniques::weights::SubstrateWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type Locker = ();
}
