#[allow(unused)]
use crate::{mock::*, Error};
#[allow(unused)]
use frame_support::{assert_noop, assert_ok, assert_err};
use frame_system::Account;
use crate::primitives::WorkerInfo;

#[allow(unused)]
const ALICE: AccountId = 1;
#[allow(unused)]
const ALICE_CONTROLLER: AccountId = 2;
#[allow(unused)]
const BOB: AccountId = 3;
#[allow(unused)]
const BOB_CONTROLLER: AccountId = 4;

fn register_worker_for(
	owner: AccountId,
	controller: AccountId,
	initial_deposit: Balance
) -> WorkerInfo<AccountId> {
	let owner_balance = Balances::free_balance(owner);

	assert_ok!(
		ComputingWorkers::register(
			RuntimeOrigin::signed(owner),
			controller,
			initial_deposit
		)
	);

	let worker_info = ComputingWorkers::workers(controller).unwrap();
	let current_account = worker_info.current_account;

	assert_eq!(Balances::free_balance(owner), owner_balance - initial_deposit);
	assert_eq!(Balances::free_balance(controller), 0);
	assert_eq!(Balances::free_balance(current_account), initial_deposit);

	worker_info
}

#[test]
fn register_works() {
	new_test_ext().execute_with(|| {
		set_balance(ALICE, 101 * DOLLARS, 0);

		register_worker_for(ALICE, ALICE_CONTROLLER, 100 * DOLLARS);

		run_to_block(1);
		set_balance(ALICE, 101 * DOLLARS, 0);

		assert_noop!(
			ComputingWorkers::register(
				RuntimeOrigin::signed(ALICE),
				ALICE_CONTROLLER,
				10 * DOLLARS
			),
			Error::<Test>::InitialDepositTooLow
		);

		assert_noop!(
			ComputingWorkers::register(
				RuntimeOrigin::signed(ALICE),
				ALICE_CONTROLLER,
				100 * DOLLARS
			),
			Error::<Test>::AlreadyRegistered
		);
	});
}

#[test]
fn deregister_works() {
	new_test_ext().execute_with(|| {
		set_balance(ALICE, 101 * DOLLARS, 0);

		let alice_worker = register_worker_for(ALICE, ALICE_CONTROLLER, 100 * DOLLARS);
		let alice_worker_current_account = alice_worker.current_account;

		run_to_block(1);

		assert_ok!(
			ComputingWorkers::deregister(
				RuntimeOrigin::signed(ALICE),
				ALICE_CONTROLLER,
			)
		);

		assert_eq!(Balances::free_balance(ALICE), 101 * DOLLARS);
		assert_eq!(Balances::free_balance(ALICE_CONTROLLER), 0);
		assert!(!Account::<Test>::contains_key(&alice_worker_current_account));
	});
}
