// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::asset::mogwai::{PhaseType, RarityType};

use frame_support::{
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	Parameter,
};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Member},
	SaturatedConversion,
};
use sp_std::marker::PhantomData;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum FeeType {
	#[default]
	Default = 0,
	Remove = 1,
}

pub const MILLIMOGS: u64 = 1_000_000_000;
pub const DMOGS: u64 = 1_000 * MILLIMOGS;

pub struct Pricing<Balance>(PhantomData<Balance>);
impl<Balance> Pricing<Balance>
where
	Balance: Member + Parameter + AtLeast32BitUnsigned + MaxEncodedLen,
{
	pub fn config_update_price(index: u8, value: u8) -> Balance {
		match index {
			1 => Self::config_max_mogwais(value),
			_ => 0_u32.into(),
		}
	}

	fn config_max_mogwais(value: u8) -> Balance {
		match value {
			1 => 5 * DMOGS,
			2 => 10 * DMOGS,
			3 => 20 * DMOGS,
			_ => 0,
		}
		.saturated_into()
	}

	pub fn fee_price(fee: FeeType) -> Balance {
		match fee {
			FeeType::Default => MILLIMOGS,
			FeeType::Remove => 50 * MILLIMOGS,
		}
		.saturated_into()
	}

	pub fn intrinsic_return(phase: PhaseType) -> Balance {
		match phase {
			PhaseType::None => 0,
			PhaseType::Bred => 20 * MILLIMOGS,
			PhaseType::Hatched => 5 * MILLIMOGS,
			PhaseType::Matured => 3 * MILLIMOGS,
			PhaseType::Mastered => 2 * MILLIMOGS,
			PhaseType::Exalted => MILLIMOGS,
		}
		.saturated_into()
	}

	pub fn pairing(rarity1: RarityType, rarity2: RarityType) -> Balance {
		let rarity_sum = rarity1 as u8 + rarity2 as u8;

		match rarity_sum {
			0 => 10 * MILLIMOGS,
			1 => 100 * MILLIMOGS,
			2 => 200 * MILLIMOGS,
			3 => 300 * MILLIMOGS,
			4 => 400 * MILLIMOGS,
			5 => 500 * MILLIMOGS,
			6 => 1000 * MILLIMOGS,
			7 => 1500 * MILLIMOGS,
			8 => 2000 * MILLIMOGS,
			_ => 10000 * MILLIMOGS,
		}
		.saturated_into()
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct GameConfig {
	pub parameters: [u8; GameConfig::PARAM_COUNT],
}

impl GameConfig {
	pub const PARAM_COUNT: usize = 10;

	pub fn new() -> Self {
		GameConfig { parameters: [0; GameConfig::PARAM_COUNT] }
	}

	pub fn config_value(index: u8, value: u8) -> u32 {
		let result: u32;
		match index {
			// MaxMogwaisInAccount
			1 => match value {
				0 => result = 6,
				1 => result = 12,
				2 => result = 18,
				3 => result = 24,
				_ => result = 0,
			},
			_ => result = 0,
		}
		result
	}

	pub fn verify_update(index: u8, value: u8, update_value_opt: Option<u8>) -> u8 {
		let mut result: u8;
		match index {
			// MaxMogwaisInAccount
			1 => match value {
				0 => result = 1,
				1 => result = 2,
				2 => result = 3,
				_ => result = 0,
			},
			_ => result = 0,
		}
		// don't allow bad requests
		if update_value_opt.is_some() && result != update_value_opt.unwrap() {
			result = 0;
		}
		result
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum GameEventType {
	#[default]
	Default = 0,
	Hatch = 1,
}

impl GameEventType {
	pub fn time_till(game_type: GameEventType) -> u16 {
		match game_type {
			GameEventType::Hatch => 100,
			GameEventType::Default => 0,
		}
	}

	pub fn duration(game_type: GameEventType) -> u16 {
		match game_type {
			GameEventType::Hatch => 0,
			GameEventType::Default => 0,
		}
	}
}
