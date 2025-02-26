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
	asset,
	asset::mogwai::PhaseType,
	config::Pricing,
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
	pub(crate) fn sacrifice_mogwai(
		owner: &AccountId,
		mogwai_id: &asset::BattleMogsId,
		payment_asset: Option<Sage::FungiblesAssetId>,
	) -> Result<BattleMogsTransitionOutput<BlockNumber>, TransitionError> {
		let mut asset = Self::get_owned_mogwai(owner, mogwai_id)?;
		let mogwai = asset.as_mogwai()?;

		ensure!(mogwai.phase != PhaseType::Bred, BattleMogsError::from(MOGWAI_STILL_IN_BRED_PHASE));

		let intrinsic_to_deposit = {
			let mogwai_funds = Self::inspect_asset_funds(mogwai_id, payment_asset.clone());

			let intrinsic_return = Pricing::<Balance>::intrinsic_return(mogwai.phase);
			mogwai_funds.checked_div(&intrinsic_return).unwrap_or(Balance::zero())
		};
		Self::withdraw_funds_from_asset(mogwai_id, owner, payment_asset, intrinsic_to_deposit)?;

		// TODO: Do something with the results
		//let _ = Self::update_achievement_for(&sender, AccountAchievement::Sacrificer, 1);

		Ok(sp_std::vec![TransitionOutput::Consumed(*mogwai_id)])
	}
}
