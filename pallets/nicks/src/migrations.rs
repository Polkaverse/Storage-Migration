use super::*;
pub use pallet::*;
use sp_std::prelude::*;
use frame_support::{
	weights::Weight,
	pallet_prelude::*,
	traits::{Get, StorageVersion},
	storage::migration,
};

pub fn migrate<T: Config>() -> Weight {
	let version = StorageVersion::get::<Pallet<T>>();
	let mut weight: Weight = 0;

	// Upgrade from a version lower than 1.
	if version < 1 {
		// Calculate the weight of the upgraded version and perform the version upgrade.
		weight = weight.saturating_add(v1::migrate::<T>());
	}

	weight
}

pub mod v1 {
	use super::*;
	
	pub(crate) fn migrate<T: Config>() -> Weight {
		let mut reads_writes = 0;

		let module_name = <crate::Pallet<T>>::name().as_bytes();
		let item_name = b"NameOf";

		// Count the number of entries in nicks.
		// Save it in the CountForNames field.
		let name_count = migration::storage_key_iter::<T::AccountId, (Vec<u8>,BalanceOf<T>), Twox64Concat>(module_name, item_name).count() as u32;
		CountForNames::<T>::put(name_count);
		reads_writes += 1;

		// Get the iter of the old field.
		let iter = migration::storage_key_iter::<T::AccountId, (Vec<u8>,BalanceOf<T>), Twox64Concat>(module_name, item_name);
		// Split the original name into last name and first name by " ".
		// Save it in the Realname field.
		for item in iter {
			if let Some(take_item) = migration::take_storage_item::<T::AccountId, (Vec<u8>,BalanceOf<T>), Twox64Concat>(module_name, item_name, item.0.clone()) {
				reads_writes += 1;
				let (nick, deposit) = take_item;
				let value = match nick.iter().rposition(|&x| x == b" "[0]) {
					Some(ndx) => (Nickname {
						first: nick[0..ndx].to_vec(),
						last: Some(nick[ndx + 1..].to_vec())
					}, deposit),
					None => (Nickname { first: nick, last: None }, deposit)
				};
				RealnameOf::<T>::insert(item.0, value);
			}
		}

		// Upgrade version number.
		StorageVersion::new(1).put::<crate::Pallet<T>>();
		reads_writes += 1;

		// Calculate weight.
		T::DbWeight::get().reads_writes(reads_writes, reads_writes)
	}
}
