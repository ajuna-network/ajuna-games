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

use sage_api::TransitionError;

pub const ASSET_NOT_FOUND: u8 = 0;
pub const MOGWAI_LIMIT_REACHED: u8 = 1;
pub const PLAYER_ALREADY_HAS_ACHIEVEMENT_TABLE: u8 = 2;
pub const ASSET_IS_NOT_MOGWAI: u8 = 3;
pub const ASSET_IS_NOT_ACHIEVEMENT_TABLE: u8 = 4;
pub const CANNOT_USE_SAME_ASSET_FOR_BREEDING: u8 = 5;
pub const MOGWAI_STILL_IN_BRED_PHASE: u8 = 6;
pub const MOGWAI_NOT_IN_BRED_PHASE: u8 = 7;
pub const MOGWAI_HAS_INVALID_RARITY: u8 = 8;

pub const ASSET_COULD_NOT_RECEIVE_FUNDS: u8 = 100;
pub const ASSET_COULD_NOT_WITHDRAW_FUNDS: u8 = 101;

pub(crate) struct BattleMogsError;

impl BattleMogsError {
	pub(crate) fn from(code: u8) -> TransitionError {
		TransitionError::Transition { code }
	}
}
