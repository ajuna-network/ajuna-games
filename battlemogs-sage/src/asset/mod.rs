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

use frame_support::pallet_prelude::*;
use sage_api::traits::GetId;

pub mod achievement_table;
pub mod mogwai;

pub type BattleMogsId = u64;

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum BattleMogsVariant {
	Mogwai(mogwai::Mogwai),
	AchievementTable(achievement_table::AchievementTable),
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct BattleMogsAsset<BlockNumber> {
	pub id: BattleMogsId,
	pub genesis: BlockNumber,
	pub variant: BattleMogsVariant,
}

impl<BlockNumber> GetId<BattleMogsId> for BattleMogsAsset<BlockNumber> {
	fn get_id(&self) -> BattleMogsId {
		self.id
	}
}

impl<BlockNumber> BattleMogsAsset<BlockNumber> {
	pub fn is_mogwai(&self) -> bool {
		matches!(self.variant, BattleMogsVariant::Mogwai(_))
	}

	pub fn is_achievement(&self) -> bool {
		matches!(self.variant, BattleMogsVariant::AchievementTable(_))
	}

	pub fn as_mogwai(&mut self) -> Option<&mut mogwai::Mogwai> {
		match &mut self.variant {
			BattleMogsVariant::Mogwai(mogwai) => Some(mogwai),
			BattleMogsVariant::AchievementTable(_) => None,
		}
	}

	pub fn as_achievement(&mut self) -> Option<&mut achievement_table::AchievementTable> {
		match &mut self.variant {
			BattleMogsVariant::AchievementTable(achievement_table) => Some(achievement_table),
			BattleMogsVariant::Mogwai(_) => None,
		}
	}
}
