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
pub enum AchievementState {
	InProgress { current: u16, target: u16 },
	Completed,
}

impl AchievementState {
	pub fn increase_by(&mut self, amount: u16) -> AchievementState {
		match self {
			AchievementState::InProgress { current, target } => {
				let new_current = current.saturating_add(amount);

				if new_current >= *target {
					AchievementState::Completed
				} else {
					AchievementState::InProgress { current: new_current, target: *target }
				}
			},
			AchievementState::Completed => AchievementState::Completed,
		}
	}
}

impl AchievementState {
	pub fn new(target: u16) -> Self {
		Self::InProgress { current: Default::default(), target }
	}

	pub fn update(self, amount: u16) -> Self {
		match self {
			AchievementState::InProgress { current, target } => {
				let new_current = current + amount;

				if new_current >= target {
					Self::Completed
				} else {
					Self::InProgress { current: new_current, target }
				}
			},
			AchievementState::Completed => Self::Completed,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct AchievementTable {
	pub egg_hatcher: AchievementState,
	pub sacrificer: AchievementState,
	pub morpheus: AchievementState,
	pub legend_breeder: AchievementState,
	pub promiscuous: AchievementState,
}
