#[allow(unused)]
use crate::{types::*, mock::*, Error};
#[allow(unused)]
use frame_support::{assert_noop, assert_ok, assert_err};
#[allow(unused)]
use frame_system::Account;

#[allow(unused)]
const ALICE: AccountId = 1;
#[allow(unused)]
const ALICE_WORKER: AccountId = 2;
#[allow(unused)]
const BOB: AccountId = 3;
#[allow(unused)]
const BOB_WORKER: AccountId = 4;

fn register_worker_for(
	owner: AccountId,
	identity: AccountId,
	initial_deposit: Balance
) -> WorkerInfo<AccountId> {
	let owner_balance = Balances::free_balance(owner);

	assert_ok!(
		ComputingWorkers::register(
			RuntimeOrigin::signed(owner),
			identity,
			initial_deposit,
			0,
			None
		)
	);

	let worker_info = ComputingWorkers::workers(identity).unwrap();
	let stash = worker_info.stash;

	assert_eq!(worker_info.status, WorkerStatus::Registered);
	assert_eq!(Balances::free_balance(owner), owner_balance - initial_deposit);
	assert_eq!(Balances::free_balance(identity), 0);
	assert_eq!(Balances::free_balance(stash), initial_deposit);

	worker_info
}

#[test]
fn register_works() {
	new_test_ext().execute_with(|| {
		set_balance(ALICE, 101 * DOLLARS, 0);

		register_worker_for(ALICE, ALICE_WORKER, 100 * DOLLARS);

		run_to_block(1);
		set_balance(ALICE, 101 * DOLLARS, 0);

		assert_noop!(
			ComputingWorkers::register(
				RuntimeOrigin::signed(ALICE),
				ALICE_WORKER,
				10 * DOLLARS,
				0,
				None
			),
			Error::<Test>::InitialDepositTooLow
		);

		assert_noop!(
			ComputingWorkers::register(
				RuntimeOrigin::signed(ALICE),
				ALICE_WORKER,
				100 * DOLLARS,
				0,
				None
			),
			Error::<Test>::AlreadyRegistered
		);
	});
}

#[test]
fn deregister_works() {
	new_test_ext().execute_with(|| {
		set_balance(ALICE, 101 * DOLLARS, 0);

		let alice_worker = register_worker_for(ALICE, ALICE_WORKER, 100 * DOLLARS);
		let alice_worker_stash = alice_worker.stash;

		run_to_block(1);

		assert_ok!(
			ComputingWorkers::deregister(
				RuntimeOrigin::signed(ALICE),
				ALICE_WORKER,
			)
		);

		assert_eq!(Balances::free_balance(ALICE), 101 * DOLLARS);
		assert_eq!(Balances::free_balance(ALICE_WORKER), 0);
		assert!(!Account::<Test>::contains_key(&alice_worker_stash));
	});
}
