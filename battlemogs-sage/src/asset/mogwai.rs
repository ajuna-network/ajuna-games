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

use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Mogwai {
	pub dna: [[u8; 32]; 2],
	pub generation: MogwaiGeneration,
	pub rarity: RarityType,
	pub phase: PhaseType,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum MogwaiGeneration {
	#[default]
	First = 1,
	Second = 2,
	Third = 3,
	Fourth = 4,
	Fifth = 5,
	Sixth = 6,
	Seventh = 7,
	Eighth = 8,
	Ninth = 9,
	Tenth = 10,
	Eleventh = 11,
	Twelfth = 12,
	Thirteenth = 13,
	Fourteenth = 14,
	Fifteenth = 15,
	Sixteenth = 16,
}

impl MogwaiGeneration {
	pub fn coerce_from(num: u16) -> Self {
		match num {
			0 => Self::First,
			1..=16 => Self::from(num),
			_ => Self::Sixteenth,
		}
	}
}

impl From<u8> for MogwaiGeneration {
	fn from(num: u8) -> Self {
		MogwaiGeneration::from(num as u16)
	}
}

impl From<u16> for MogwaiGeneration {
	fn from(num: u16) -> Self {
		match num {
			1 => MogwaiGeneration::First,
			2 => MogwaiGeneration::Second,
			3 => MogwaiGeneration::Third,
			4 => MogwaiGeneration::Fourth,
			5 => MogwaiGeneration::Fifth,
			6 => MogwaiGeneration::Sixth,
			7 => MogwaiGeneration::Seventh,
			8 => MogwaiGeneration::Eighth,
			9 => MogwaiGeneration::Ninth,
			10 => MogwaiGeneration::Tenth,
			11 => MogwaiGeneration::Eleventh,
			12 => MogwaiGeneration::Twelfth,
			13 => MogwaiGeneration::Thirteenth,
			14 => MogwaiGeneration::Fourteenth,
			15 => MogwaiGeneration::Fifteenth,
			16 => MogwaiGeneration::Sixteenth,
			_ => MogwaiGeneration::First,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum RarityType {
	#[default]
	Common = 0,
	Uncommon = 1,
	Rare = 2,
	Epic = 3,
	Legendary = 4,
	Mythical = 5,
}

impl From<u8> for RarityType {
	fn from(num: u8) -> Self {
		RarityType::from(num as u16)
	}
}

impl From<u16> for RarityType {
	fn from(num: u16) -> Self {
		match num {
			0 => RarityType::Common,
			1 => RarityType::Uncommon,
			2 => RarityType::Rare,
			3 => RarityType::Epic,
			4 => RarityType::Legendary,
			5 => RarityType::Mythical,
			_ => RarityType::Common,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum PhaseType {
	#[default]
	None = 0,
	Bred = 1,
	Hatched = 2,
	Matured = 3,
	Mastered = 4,
	Exalted = 5,
}
