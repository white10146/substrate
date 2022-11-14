// This file is part of Substrate.

// Copyright (C) 2021-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for pallet_example_basic
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-03-15, STEPS: `[100, ]`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/substrate
// benchmark
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_example_basic
// --extrinsic
// *
// --steps
// 100
// --repeat
// 10
// --raw
// --output
// ./
// --template
// ./.maintain/frame-weight-template.hbs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_example_basic.
pub trait WeightInfo {
	fn set_dummy_benchmark(b: u32, ) -> Weight;
	fn accumulate_dummy(b: u32, ) -> Weight;
	fn sort_vector(x: u32, ) -> Weight;
}

/// Weights for pallet_example_basic using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn set_dummy_benchmark(b: u32, ) -> Weight {
		(5_834_000 as Weight)
			.saturating_add((24_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn accumulate_dummy(b: u32, ) -> Weight {
		(51_353_000 as Weight)
			.saturating_add((14_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn sort_vector(x: u32, ) -> Weight {
		(2_569_000 as Weight)
			// Standard Error: 0
			.saturating_add((4_000 as Weight).saturating_mul(x as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn set_dummy_benchmark(b: u32, ) -> Weight {
		(5_834_000 as Weight)
			.saturating_add((24_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn accumulate_dummy(b: u32, ) -> Weight {
		(51_353_000 as Weight)
			.saturating_add((14_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn sort_vector(x: u32, ) -> Weight {
		(2_569_000 as Weight)
			// Standard Error: 0
			.saturating_add((4_000 as Weight).saturating_mul(x as Weight))
	}
}
