// This file is part of Substrate.

// Copyright (C) 2018-2022 Parity Technologies (UK) Ltd.
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

//! This module deals with the deprecated randomness API.

use crate::{Config, DispatchError, Error, PhantomData, Randomness};

/// Fallible version of [`Randomness`].
///
/// This is needed in order to signal that no randomness is to be supplied to contracts.
/// This is a sealed trait. Use the provided [`UnsafeDeprecatedRandomness`] or [`NoRandomness`].
pub trait MaybeRandomness<Output, BlockNumber>: sealed::Sealed {
	/// Same as `[Randomness:random]` but fallible.
	///
	/// When a contract queries randomness and this function fails then the execution of this
	/// contract is immediately trapped.
	fn random(subject: &[u8]) -> Result<(Output, BlockNumber), DispatchError>;
}

/// This just forwards the randomness provided by `R`.
///
/// This merely exists to support pre-existing contracts. Never use this for new
/// deployments of this pallet. Please note that even when using this type
pub struct UnsafeDeprecatedRandomness<T, R>(PhantomData<(T, R)>);

/// Do not support randomness functions. This is the safe default and should be used.
///
/// Any contract that tries to use the legacy random functions will trap if this is set.
pub struct NoRandomness<T>(PhantomData<T>);

impl<T, R> MaybeRandomness<T::Hash, T::BlockNumber> for UnsafeDeprecatedRandomness<T, R>
where
	T: Config,
	R: Randomness<T::Hash, T::BlockNumber>,
{
	fn random(subject: &[u8]) -> Result<(T::Hash, T::BlockNumber), DispatchError> {
		Ok(R::random(subject))
	}
}

impl<T: Config> MaybeRandomness<T::Hash, T::BlockNumber> for NoRandomness<T> {
	fn random(_subject: &[u8]) -> Result<(T::Hash, T::BlockNumber), DispatchError> {
		Err(Error::<T>::RandomnessUnavailable.into())
	}
}

mod sealed {
	use super::*;

	pub trait Sealed {}

	impl<T, R> Sealed for UnsafeDeprecatedRandomness<T, R> {}
	impl<T> Sealed for NoRandomness<T> {}
}
