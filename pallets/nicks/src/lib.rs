// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
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

//! # Nicks Pallet
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! Nicks is an example pallet for keeping track of account names on-chain. It makes no effort to
//! create a name hierarchy, be a DNS replacement or provide reverse lookups. Furthermore, the
//! weights attached to this pallet's dispatchable functions are for demonstration purposes only and
//! have not been designed to be economically secure. Do not use this pallet as-is in production.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `set_name` - Set the associated name of an account; a small deposit is reserved if not already
//!   taken.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod migrations;

use scale_info::TypeInfo;
use sp_runtime::{RuntimeDebug};
use frame_support::{
	weights::Weight,
	traits::{Currency, ReservableCurrency, StorageVersion},
};
pub use pallet::*;
use sp_std::prelude::*;
pub use log::{info};

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// The current storage version.
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

/// A nickname with a first and last part.
#[derive(codec::Encode, codec::Decode, Default, RuntimeDebug, PartialEq, TypeInfo)]
pub struct Nickname {
	first: Vec<u8>,
	last: Option<Vec<u8>>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		ensure,
		pallet_prelude::*,
		traits::{Get},
	};
	use frame_system::{ensure_signed, pallet_prelude::*};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Reservation fee.
		#[pallet::constant]
		type ReservationFee: Get<BalanceOf<Self>>;

		/// The minimum length a name may be.
		#[pallet::constant]
		type MinLength: Get<u32>;

		/// The maximum length a name may be.
		#[pallet::constant]
		type MaxLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A name was set. \[who\]
		NameSet(T::AccountId),
		/// A name was changed. \[who\]
		NameChanged(T::AccountId),
	}

	/// Error for the nicks pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// A name is too short.
		TooShort,
		/// A name is too long.
		TooLong,
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			migrations::migrate::<T>()
		}
	}

		/// The lookup table for names.
	#[pallet::storage]
	pub(super) type RealnameOf<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (Nickname,BalanceOf<T>)>;

	#[pallet::storage]
	pub(super) type CountForNames<T: Config> = StorageValue<_, u32>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(50_000_000)]
		pub fn set_name(origin: OriginFor<T>, first: Vec<u8>, last: Option<Vec<u8>>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let len = match last {
				None => first.len(),
				Some(ref last_name) => first.len() + last_name.len(),
			};

			ensure!(len <= T::MaxLength::get().try_into().unwrap(), Error::<T>::TooLong);

			let deposit = if let Some((_, deposit)) = <RealnameOf<T>>::get(&sender) {
				Self::deposit_event(Event::<T>::NameChanged(sender.clone()));
				deposit
			} else {
				let deposit = T::ReservationFee::get();
				T::Currency::reserve(&sender, deposit.clone())?;
				Self::deposit_event(Event::<T>::NameSet(sender.clone()));
				deposit
			};

			<RealnameOf<T>>::insert(&sender, (Nickname { first, last }, deposit));
			if let Some(old) = <CountForNames<T>>::get() {
				// Increment the value read from storage; will error in the event of overflow.
				let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
				// Update the value in storage with the incremented result.
				<CountForNames<T>>::put(new);
			}
			Ok(())
		}
	}
}
