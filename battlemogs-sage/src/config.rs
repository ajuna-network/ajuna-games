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

pub const MILLIARD: u64 = 1_000_000_000;

pub struct Pricing<Balance>(PhantomData<Balance>);
impl<Balance> Pricing<Balance>
where
	Balance: Member + Parameter + AtLeast32BitUnsigned + MaxEncodedLen,
{
	pub fn intrinsic_return(phase: PhaseType) -> Balance {
		match phase {
			PhaseType::None => 0,
			PhaseType::Bred => 20 * MILLIARD,
			PhaseType::Hatched => 5 * MILLIARD,
			PhaseType::Matured => 3 * MILLIARD,
			PhaseType::Mastered => 2 * MILLIARD,
			PhaseType::Exalted => MILLIARD,
		}
		.saturated_into()
	}

	pub fn pairing(rarity1: RarityType, rarity2: RarityType) -> Balance {
		let rarity_sum = rarity1 as u8 + rarity2 as u8;

		match rarity_sum {
			0 => 10 * MILLIARD,
			1 => 100 * MILLIARD,
			2 => 200 * MILLIARD,
			3 => 300 * MILLIARD,
			4 => 400 * MILLIARD,
			5 => 500 * MILLIARD,
			6 => 1000 * MILLIARD,
			7 => 1500 * MILLIARD,
			8 => 2000 * MILLIARD,
			_ => 10000 * MILLIARD,
		}
		.saturated_into()
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
}
