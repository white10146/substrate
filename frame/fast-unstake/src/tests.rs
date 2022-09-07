// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for pallet-fast-unstake.

use super::*;
use crate::{mock::*, weights::WeightInfo, Event};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*, traits::Currency};
use pallet_nomination_pools::{BondedPools, LastPoolId, RewardPools};
use pallet_staking::CurrentEra;

use sp_runtime::{
	traits::BadOrigin,
	DispatchError, ModuleError,
};
use sp_staking::StakingInterface;
use sp_std::prelude::*;

#[test]
fn test_setup_works() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(BondedPools::<T>::count(), 1);
		assert_eq!(RewardPools::<T>::count(), 1);
		assert_eq!(Staking::bonding_duration(), 3);
		let last_pool = LastPoolId::<T>::get();
		assert_eq!(last_pool, 1);
	});
}

#[test]
fn register_works() {
	ExtBuilder::default().build_and_execute(|| {
		// Controller account registers for fast unstake.
		assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(1_u32)));
		// Ensure stash is in the queue.
		assert_ne!(Queue::<T>::get(1), None);
	});
}

#[test]
fn cannot_register_if_not_bonded() {
	ExtBuilder::default().build_and_execute(|| {
		// Mint accounts 1 and 2 with 200 tokens.
		for _ in 1..2 {
			let _ = Balances::make_free_balance_be(&1, 200);
		}
		// Attempt to fast unstake.
		assert_noop!(
			FastUnstake::register_fast_unstake(Origin::signed(1), Some(1_u32)),
			Error::<T>::NotController
		);
	});
}

#[test]
fn cannot_register_if_in_queue() {
	ExtBuilder::default().build_and_execute(|| {
		// Insert some Queue item
		Queue::<T>::insert(1, Some(1_u32));
		// Cannot re-register, already in queue
		assert_noop!(
			FastUnstake::register_fast_unstake(Origin::signed(2), Some(1_u32)),
			Error::<T>::AlreadyQueued
		);
	});
}

#[test]
fn cannot_register_if_head() {
	ExtBuilder::default().build_and_execute(|| {
		// Insert some Head item for stash
		Head::<T>::put(UnstakeRequest { stash: 1.clone(), checked: vec![], maybe_pool_id: None });
		// Controller attempts to regsiter
		assert_noop!(
			FastUnstake::register_fast_unstake(Origin::signed(2), Some(1_u32)),
			Error::<T>::AlreadyHead
		);
	});
}

#[test]
fn cannot_register_if_has_unlocking_chunks() {
	ExtBuilder::default().build_and_execute(|| {
		// Start unbonding half of staked tokens
		assert_ok!(Staking::unbond(Origin::signed(2), 50_u128));
		// Cannot register for fast unstake with unlock chunks active
		assert_noop!(
			FastUnstake::register_fast_unstake(Origin::signed(2), Some(1_u32)),
			Error::<T>::NotFullyBonded
		);
	});
}

#[test]
fn deregister_works() {
	ExtBuilder::default().build_and_execute(|| {
		// Controller account registers for fast unstake.
		assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(1_u32)));
		// Controller then changes mind and deregisters.
		assert_ok!(FastUnstake::deregister(Origin::signed(2)));
		// Ensure stash no longer exists in the queue.
		assert_eq!(Queue::<T>::get(1), None);
	});
}

#[test]
fn cannot_deregister_if_not_controller() {
	ExtBuilder::default().build_and_execute(|| {
		// Controller account registers for fast unstake.
		assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(1_u32)));
		// Stash tries to deregister.
		assert_noop!(FastUnstake::deregister(Origin::signed(1)), Error::<T>::NotController);
	});
}

#[test]
fn cannot_deregister_if_not_queued() {
	ExtBuilder::default().build_and_execute(|| {
		// Controller tries to deregister without first registering
		assert_noop!(FastUnstake::deregister(Origin::signed(2)), Error::<T>::NotQueued);
	});
}

#[test]
fn cannot_deregister_already_head() {
	ExtBuilder::default().build_and_execute(|| {
		// Controller attempts to register, should fail
		assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(1_u32)));
		// Insert some Head item for stash.
		Head::<T>::put(UnstakeRequest { stash: 1.clone(), checked: vec![], maybe_pool_id: None });
		// Controller attempts to deregister
		assert_noop!(FastUnstake::deregister(Origin::signed(2)), Error::<T>::AlreadyHead);
	});
}

#[test]
fn control_works() {
	ExtBuilder::default().build_and_execute(|| {
		// account with control (root) origin wants to only check 1 era per block.
		assert_ok!(FastUnstake::control(Origin::root(), 1_u32));
	});
}

#[test]
fn control_must_be_control_origin() {
	ExtBuilder::default().build_and_execute(|| {
		// account without control (root) origin wants to only check 1 era per block.
		assert_noop!(FastUnstake::control(Origin::signed(1), 1_u32), BadOrigin);
	});
}

mod on_idle {
	use super::*;

	#[test]
	fn early_exit() {
		ExtBuilder::default().build_and_execute(|| {
			ErasToCheckPerBlock::<T>::put(BondingDuration::get() + 1);
			CurrentEra::<T>::put(BondingDuration::get());

			// set up Queue item
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(1)));
			assert_eq!(Queue::<T>::get(1), Some(Some(1)));

			// call on_idle with no remaining weight
			FastUnstake::on_idle(System::block_number(), Weight::from_ref_time(0));

			// assert nothing changed in Queue and Head
			assert_eq!(Head::<T>::get(), None);
			assert_eq!(Queue::<T>::get(1), Some(Some(1)));
		});
	}

	#[test]
	fn respects_weight() {
		ExtBuilder::default().build_and_execute(|| {
			// we want to check all eras in one block...
			ErasToCheckPerBlock::<T>::put(BondingDuration::get() + 1);
			CurrentEra::<T>::put(BondingDuration::get());

			// given
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(1)));
			assert_eq!(Queue::<T>::get(1), Some(Some(1)));

			assert_eq!(Queue::<T>::count(), 1);
			assert_eq!(Head::<T>::get(), None);

			// when: call fast unstake with not enough weight to process the whole thing, just one
			// era.
			let remaining_weight = <T as Config>::WeightInfo::on_idle_check(
				pallet_staking::ValidatorCount::<T>::get() * 1,
			);
			assert_eq!(FastUnstake::on_idle(0, remaining_weight), remaining_weight);

			// then
			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![Event::Checked { stash: 1, eras: vec![3] }]
			);
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3], maybe_pool_id: Some(1) })
			);

			// when: another 1 era.
			let remaining_weight = <T as Config>::WeightInfo::on_idle_check(
				pallet_staking::ValidatorCount::<T>::get() * 1,
			);
			assert_eq!(FastUnstake::on_idle(0, remaining_weight), remaining_weight);

			// then:
			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![Event::Checked { stash: 1, eras: vec![2] }]
			);
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3, 2], maybe_pool_id: Some(1) })
			);

			// when: then 5 eras, we only need 2 more.
			let remaining_weight = <T as Config>::WeightInfo::on_idle_check(
				pallet_staking::ValidatorCount::<T>::get() * 5,
			);
			assert_eq!(
				FastUnstake::on_idle(0, remaining_weight),
				// note the amount of weight consumed: 2 eras worth of weight.
				<T as Config>::WeightInfo::on_idle_check(
					pallet_staking::ValidatorCount::<T>::get() * 2,
				)
			);

			// then:
			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![Event::Checked { stash: 1, eras: vec![1, 0] }]
			);
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest {
					stash: 1,
					checked: vec![3, 2, 1, 0],
					maybe_pool_id: Some(1)
				})
			);

			// when: not enough weight to unstake:
			let remaining_weight =
				<T as Config>::WeightInfo::on_idle_unstake() - Weight::from_ref_time(1);
			assert_eq!(FastUnstake::on_idle(0, remaining_weight), Weight::from_ref_time(0));

			// then nothing happens:
			assert_eq!(fast_unstake_events_since_last_call(), vec![]);
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest {
					stash: 1,
					checked: vec![3, 2, 1, 0],
					maybe_pool_id: Some(1)
				})
			);

			// when: enough weight to get over at least one iteration: then we are unblocked and can
			// unstake.
			let remaining_weight = <T as Config>::WeightInfo::on_idle_check(
				pallet_staking::ValidatorCount::<T>::get() * 1,
			);
			assert_eq!(
				FastUnstake::on_idle(0, remaining_weight),
				<T as Config>::WeightInfo::on_idle_unstake()
			);

			// then we finish the unbonding:
			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![Event::Unstaked { stash: 1, maybe_pool_id: Some(1), result: Ok(()) }]
			);
			assert_eq!(Head::<T>::get(), None,);

			assert_unstaked(&1);
		});
	}

	#[test]
	fn if_head_not_set_one_random_fetched_from_queue() {
		ExtBuilder::default().build_and_execute(|| {
			ErasToCheckPerBlock::<T>::put(BondingDuration::get() + 1);
			CurrentEra::<T>::put(BondingDuration::get());

			// given
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), None));
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(4), None));
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(6), None));
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(8), None));
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(10), None));

			assert_eq!(Queue::<T>::count(), 5);
			assert_eq!(Head::<T>::get(), None);

			// when
			next_block(true);

			// then
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3, 2, 1, 0], maybe_pool_id: None })
			);
			assert_eq!(Queue::<T>::count(), 4);

			// when
			next_block(true);

			// then
			assert_eq!(Head::<T>::get(), None,);
			assert_eq!(Queue::<T>::count(), 4);

			// when
			next_block(true);

			// then
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 5, checked: vec![3, 2, 1, 0], maybe_pool_id: None }),
			);
			assert_eq!(Queue::<T>::count(), 3);

			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![
					Event::Checked { stash: 1, eras: vec![3, 2, 1, 0] },
					Event::Unstaked { stash: 1, maybe_pool_id: None, result: Ok(()) },
					Event::Checked { stash: 5, eras: vec![3, 2, 1, 0] }
				]
			);
		});
	}

	#[test]
	fn successful_multi_queue() {
		ExtBuilder::default().build_and_execute(|| {
			ErasToCheckPerBlock::<T>::put(BondingDuration::get() + 1);
			CurrentEra::<T>::put(BondingDuration::get());

			// register multi accounts for fast unstake
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(1)));
			assert_eq!(Queue::<T>::get(1), Some(Some(1)));
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(4), Some(1)));
			assert_eq!(Queue::<T>::get(3), Some(Some(1)));

			// assert 2 queue items are in Queue & None in Head to start with
			assert_eq!(Queue::<T>::count(), 2);
			assert_eq!(Head::<T>::get(), None);

			// process on idle and check eras for next Queue item
			next_block(true);

			// process on idle & let go of current Head
			next_block(true);

			// confirm Head / Queue items remaining
			assert_eq!(Head::<T>::get(), None);
			assert_eq!(Queue::<T>::count(), 1);

			// process on idle and check eras for next Queue item
			next_block(true);

			// process on idle & let go of current Head
			next_block(true);

			// Head & Queue should now be empty
			assert_eq!(Head::<T>::get(), None);
			assert_eq!(Queue::<T>::count(), 0);

			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![
					Event::Checked { stash: 1, eras: vec![3, 2, 1, 0] },
					Event::Unstaked { stash: 1, maybe_pool_id: Some(1), result: Ok(()) },
					Event::Checked { stash: 3, eras: vec![3, 2, 1, 0] },
					Event::Unstaked { stash: 3, maybe_pool_id: Some(1), result: Ok(()) },
				]
			);

			assert_unstaked(&1);
			assert_unstaked(&3);
		});
	}

	#[test]
	fn successful_unstake_without_pool_join() {
		ExtBuilder::default().build_and_execute(|| {
			ErasToCheckPerBlock::<T>::put(BondingDuration::get() + 1);
			CurrentEra::<T>::put(BondingDuration::get());

			// register for fast unstake
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), None));
			assert_eq!(Queue::<T>::get(1), Some(None));

			// process on idle
			next_block(true);

			// assert queue item has been moved to head
			assert_eq!(Queue::<T>::get(1), None);

			// assert head item present
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3, 2, 1, 0], maybe_pool_id: None })
			);

			next_block(true);
			assert_eq!(Head::<T>::get(), None,);

			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![
					Event::Checked { stash: 1, eras: vec![3, 2, 1, 0] },
					Event::Unstaked { stash: 1, maybe_pool_id: None, result: Ok(()) }
				]
			);
			assert_unstaked(&1);
		});
	}

	#[test]
	fn successful_unstake_joining_bad_pool() {
		ExtBuilder::default().build_and_execute(|| {
			ErasToCheckPerBlock::<T>::put(BondingDuration::get() + 1);
			CurrentEra::<T>::put(BondingDuration::get());

			// register for fast unstake
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(0)));
			assert_eq!(Queue::<T>::get(1), Some(Some(0)));

			// process on idle
			next_block(true);

			// assert queue item has been moved to head
			assert_eq!(Queue::<T>::get(1), None);

			// assert head item present
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest {
					stash: 1,
					checked: vec![3, 2, 1, 0],
					maybe_pool_id: Some(0)
				})
			);

			next_block(true);
			assert_eq!(Head::<T>::get(), None,);

			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![
					Event::Checked { stash: 1, eras: vec![3, 2, 1, 0] },
					Event::Unstaked {
						stash: 1,
						maybe_pool_id: Some(0),
						result: Err(DispatchError::Module(ModuleError {
							index: 4,
							error: [0, 0, 0, 0],
							message: None
						}))
					}
				]
			);
			assert_unstaked(&1);
		});
	}

	#[test]
	fn successful_unstake_all_eras_per_block() {
		ExtBuilder::default().build_and_execute(|| {
			ErasToCheckPerBlock::<T>::put(BondingDuration::get() + 1);
			CurrentEra::<T>::put(BondingDuration::get());

			// register for fast unstake
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(1_u32)));
			assert_eq!(Queue::<T>::get(1), Some(Some(1)));

			// process on idle
			next_block(true);

			// assert queue item has been moved to head
			assert_eq!(Queue::<T>::get(1), None);

			// assert head item present
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest {
					stash: 1,
					checked: vec![3, 2, 1, 0],
					maybe_pool_id: Some(1)
				})
			);

			next_block(true);
			assert_eq!(Head::<T>::get(), None,);

			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![
					Event::Checked { stash: 1, eras: vec![3, 2, 1, 0] },
					Event::Unstaked { stash: 1, maybe_pool_id: Some(1), result: Ok(()) }
				]
			);
			assert_unstaked(&1);
			assert!(pallet_nomination_pools::PoolMembers::<T>::contains_key(&1));
		});
	}

	#[test]
	fn successful_unstake_one_era_per_block() {
		ExtBuilder::default().build_and_execute(|| {
			// put 1 era per block
			ErasToCheckPerBlock::<T>::put(1);
			CurrentEra::<T>::put(BondingDuration::get());

			// register for fast unstake
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(1_u32)));
			assert_eq!(Queue::<T>::get(1), Some(Some(1)));

			// process on idle
			next_block(true);

			// assert queue item has been moved to head
			assert_eq!(Queue::<T>::get(1), None);

			// assert head item present
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3], maybe_pool_id: Some(1) })
			);

			next_block(true);

			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3, 2], maybe_pool_id: Some(1) })
			);

			next_block(true);

			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3, 2, 1], maybe_pool_id: Some(1) })
			);

			next_block(true);

			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest {
					stash: 1,
					checked: vec![3, 2, 1, 0],
					maybe_pool_id: Some(1)
				})
			);

			next_block(true);

			assert_eq!(Head::<T>::get(), None,);

			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![
					Event::Checked { stash: 1, eras: vec![3] },
					Event::Checked { stash: 1, eras: vec![2] },
					Event::Checked { stash: 1, eras: vec![1] },
					Event::Checked { stash: 1, eras: vec![0] },
					Event::Unstaked { stash: 1, maybe_pool_id: Some(1), result: Ok(()) }
				]
			);
			assert_unstaked(&1);
			assert!(pallet_nomination_pools::PoolMembers::<T>::contains_key(&1));
		});
	}

	#[test]
	fn unstake_paused_mid_election() {
		ExtBuilder::default().build_and_execute(|| {
			// give: put 1 era per block
			ErasToCheckPerBlock::<T>::put(1);
			CurrentEra::<T>::put(BondingDuration::get());

			// register for fast unstake
			assert_ok!(FastUnstake::register_fast_unstake(Origin::signed(2), Some(1_u32)));

			// process 2 blocks
			next_block(true);
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3], maybe_pool_id: Some(1) })
			);

			next_block(true);
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3, 2], maybe_pool_id: Some(1) })
			);

			// when
			Ongoing::set(true);

			// then nothing changes
			next_block(true);
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3, 2], maybe_pool_id: Some(1) })
			);

			next_block(true);
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3, 2], maybe_pool_id: Some(1) })
			);

			// then we register a new era.
			Ongoing::set(false);
			CurrentEra::<T>::put(CurrentEra::<T>::get().unwrap() + 1);
			ExtBuilder::register_stakers_for_era(
				CurrentEra::<T>::get().unwrap(),
				VALIDATORS_PER_ERA,
				NOMINATORS_PER_VALIDATOR_PER_ERA,
			);

			// then we can progress again, but notice that the new era that had to be checked.
			next_block(true);
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest { stash: 1, checked: vec![3, 2, 4], maybe_pool_id: Some(1) })
			);

			// progress to end
			next_block(true);
			assert_eq!(
				Head::<T>::get(),
				Some(UnstakeRequest {
					stash: 1,
					checked: vec![3, 2, 4, 1],
					maybe_pool_id: Some(1)
				})
			);

			// but notice that we don't care about era 0 instead anymore! we're done.
			next_block(true);
			assert_eq!(Head::<T>::get(), None);

			assert_eq!(
				fast_unstake_events_since_last_call(),
				vec![
					Event::Checked { stash: 1, eras: vec![3] },
					Event::Checked { stash: 1, eras: vec![2] },
					Event::Checked { stash: 1, eras: vec![4] },
					Event::Checked { stash: 1, eras: vec![1] },
					Event::Unstaked { stash: 1, maybe_pool_id: Some(1), result: Ok(()) }
				]
			);

			assert_unstaked(&1);
			assert!(pallet_nomination_pools::PoolMembers::<T>::contains_key(&1));
		});
	}
}

mod signed_extension {
	use super::*;
	// TODO:
}
