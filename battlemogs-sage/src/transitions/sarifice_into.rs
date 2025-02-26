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
	algorithm::Breeding,
	asset,
	asset::mogwai::{PhaseType, RarityType},
	error::*,
	transitions::{BattleMogsTransitionConfig, BattleMogsTransitionOutput},
	BattleMogsTransition,
};

use ajuna_primitives::sage_api::SageApi;
use sage_api::{traits::TransitionOutput, TransitionError};

use frame_support::pallet_prelude::*;
use parity_scale_codec::Codec;
use sp_core::H256;
use sp_runtime::traits::{AtLeast32BitUnsigned, BlockNumber as BlockNumberT, Member};

impl<AccountId, BlockNumber, Balance, Sage> BattleMogsTransition<AccountId, BlockNumber, Sage>
where
	AccountId: Member + Codec,
	BlockNumber: BlockNumberT,
	Balance: Member + Parameter + AtLeast32BitUnsigned + MaxEncodedLen,
	Sage: SageApi<
		AccountId = AccountId,
		AssetId = asset::BattleMogsId,
		Asset = asset::BattleMogsAsset<BlockNumber>,
		Balance = Balance,
		BlockNumber = BlockNumber,
		TransitionConfig = BattleMogsTransitionConfig,
		HashOutput = H256,
	>,
{
	pub(crate) fn sacrifice_mogwai_into(
		owner: &AccountId,
		sacrificed_mogwai_id: &asset::BattleMogsId,
		into_mogwai_id: &asset::BattleMogsId,
		payment_asset: Option<Sage::FungiblesAssetId>,
	) -> Result<BattleMogsTransitionOutput<BlockNumber>, TransitionError> {
		let mut sacrificed_asset = Self::get_owned_mogwai(owner, sacrificed_mogwai_id)?;
		let sacrificed_mogwai = sacrificed_asset.as_mogwai()?;
		ensure!(
			sacrificed_mogwai.phase != PhaseType::Bred,
			BattleMogsError::from(MOGWAI_STILL_IN_BRED_PHASE)
		);
		ensure!(
			sacrificed_mogwai.rarity != RarityType::Common,
			BattleMogsError::from(MOGWAI_HAS_INVALID_RARITY)
		);

		let mut into_asset = Self::get_owned_mogwai(owner, into_mogwai_id)?;
		let into_mogwai = into_asset.as_mogwai()?;
		ensure!(
			into_mogwai.phase != PhaseType::Bred,
			BattleMogsError::from(MOGWAI_STILL_IN_BRED_PHASE)
		);
		ensure!(
			into_mogwai.rarity != RarityType::Common,
			BattleMogsError::from(MOGWAI_HAS_INVALID_RARITY)
		);

		let gen_jump = Breeding::sacrifice(
			sacrificed_mogwai.generation,
			sacrificed_mogwai.rarity,
			&sacrificed_mogwai.dna,
			into_mogwai.generation,
			into_mogwai.rarity,
			&into_mogwai.dna,
		) as u16;

		if gen_jump > 0 && (into_mogwai.generation as u16 + gen_jump) <= 16 {
			let sacrifice_funds =
				Self::inspect_asset_funds(sacrificed_mogwai_id, payment_asset.clone());
			Self::withdraw_funds_from_asset(
				sacrificed_mogwai_id,
				owner,
				payment_asset.clone(),
				sacrifice_funds.clone(),
			)?;

			Self::deposit_funds_to_asset(into_mogwai_id, owner, payment_asset, sacrifice_funds)?;
		}

		// TODO: Do something with the results
		//let _ = Self::update_achievement_for(&sender, AccountAchievement::Sacrificer, 1);

		Ok(sp_std::vec![
			TransitionOutput::Consumed(*sacrificed_mogwai_id),
			TransitionOutput::Mutated(*into_mogwai_id, into_asset)
		])
	}
}
