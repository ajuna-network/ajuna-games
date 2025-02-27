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

use crate::{
	asset::{BattleMogsAsset, BattleMogsId, BattleMogsVariant},
	transitions::{BattleMogsTransitionConfig, BattleMogsTransitionOutput},
	BattleMogsTransition,
};

use ajuna_primitives::sage_api::SageApi;
use sage_api::{traits::TransitionOutput, TransitionError};

use crate::asset::achievement_table::{AchievementState, AchievementTable};
use frame_support::pallet_prelude::*;
use parity_scale_codec::Codec;
use sp_core::H256;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, BlockNumber as BlockNumberT, Member},
	SaturatedConversion,
};

impl<AccountId, BlockNumber, Balance, Sage> BattleMogsTransition<AccountId, BlockNumber, Sage>
where
	AccountId: Member + Codec,
	BlockNumber: BlockNumberT,
	Balance: Member + Parameter + AtLeast32BitUnsigned + MaxEncodedLen,
	Sage: SageApi<
		AccountId = AccountId,
		AssetId = BattleMogsId,
		Asset = BattleMogsAsset<BlockNumber>,
		Balance = Balance,
		BlockNumber = BlockNumber,
		TransitionConfig = BattleMogsTransitionConfig,
		HashOutput = H256,
	>,
{
	pub(crate) fn register_player(
		player: &AccountId,
	) -> Result<BattleMogsTransitionOutput<BlockNumber>, TransitionError> {
		Self::ensure_has_not_achievement_table(player)?;

		let config = Sage::get_transition_config();

		let table = AchievementTable {
			egg_hatcher: AchievementState::InProgress {
				current: 0,
				target: config.target_egg_hatcher,
			},
			sacrificer: AchievementState::InProgress {
				current: 0,
				target: config.target_sacrificer,
			},
			morpheus: AchievementState::InProgress { current: 0, target: config.target_morpheus },
			legend_breeder: AchievementState::InProgress {
				current: 0,
				target: config.target_legend_breeder,
			},
			promiscuous: AchievementState::InProgress {
				current: 0,
				target: config.target_promiscuous,
			},
		};

		let block_number = Sage::get_current_block_number();
		let table_id = Self::new_asset_id(b"mogwai_id", block_number.saturated_into());

		let asset = BattleMogsAsset {
			id: table_id,
			genesis: block_number,
			variant: BattleMogsVariant::AchievementTable(table),
		};

		Ok(sp_std::vec![TransitionOutput::Minted(asset)])
	}
}
