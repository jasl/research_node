use crate as pallet_registrar;

use frame_support::{
	assert_ok,
	traits::{
		ConstU16, ConstU64, ConstU128,
		OnFinalize, OnInitialize,
	}
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;
pub(crate) type AccountId = u64;

pub(crate) const MILLI_CENTS: Balance = 1_000_000;
pub(crate) const CENTS: Balance = 1_000 * MILLI_CENTS;
pub(crate) const DOLLARS: Balance = 100 * CENTS;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		Registrar: pallet_registrar,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<{ 1 * CENTS }>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

impl pallet_registrar::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ExistentialDeposit = ConstU128<{ 100 * DOLLARS }>;
}

// Build genesis storage according to the mock runtime.
#[allow(unused)]
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	t.into()
}

#[allow(unused)]
pub(crate) fn run_to_block(n: BlockNumber) {
	// NOTE that this function only simulates modules of interest. Depending on new pallet may
	// require adding it here.
	assert!(System::block_number() < n);
	while System::block_number() < n {
		let b = System::block_number();

		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(b + 1);
		System::on_initialize(System::block_number());
	}
}

#[allow(unused)]
pub(crate) fn take_events() -> Vec<RuntimeEvent> {
	let events = System::events()
		.into_iter()
		.map(|i| i.event)
		.collect::<Vec<_>>();
	System::reset_events();
	events
}

#[allow(unused)]
pub(crate) fn set_balance(
	who: AccountId,
	new_free: Balance,
	new_reserved: Balance
) {
	assert_ok!(Balances::set_balance(RuntimeOrigin::root(), who.into(), new_free, new_reserved));
	assert_eq!(Balances::free_balance(&who), new_free);
	assert_eq!(Balances::reserved_balance(&who), new_reserved);
}
