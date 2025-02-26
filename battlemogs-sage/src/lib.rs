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

#![cfg_attr(not(feature = "std"), no_std)]

use crate::transitions::BattleMogsTransitionConfig;

use ajuna_primitives::sage_api::SageApi;
use sage_api::{traits::TransitionOutput, SageGameTransition, TransitionError};

use frame_support::pallet_prelude::*;
use parity_scale_codec::Codec;
use sp_core::H256;
use sp_runtime::traits::{AtLeast32BitUnsigned, BlockNumber as BlockNumberT};
use sp_std::marker::PhantomData;

mod algorithm;
mod asset;
mod config;
mod error;
mod transitions;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum BattleMogsAction {
	Create,
	Remove(asset::BattleMogsId),
	Hatch(asset::BattleMogsId),
	Sacrifice(asset::BattleMogsId),
	SacrificeInto(asset::BattleMogsId, asset::BattleMogsId),
	Morph(asset::BattleMogsId),
	Breed(asset::BattleMogsId, asset::BattleMogsId),
}

pub struct BattleMogsTransition<AccountId, BlockNumber, Sage> {
	_phantom: PhantomData<(AccountId, BlockNumber, Sage)>,
}

impl<AccountId, BlockNumber, Balance, Sage> SageGameTransition
	for BattleMogsTransition<AccountId, BlockNumber, Sage>
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
	type TransitionId = BattleMogsAction;
	type TransitionConfig = BattleMogsTransitionConfig;
	type AccountId = AccountId;
	type AssetId = asset::BattleMogsId;
	type Asset = asset::BattleMogsAsset<BlockNumber>;
	type Extra = ();
	type PaymentFungible = Sage::FungiblesAssetId;

	fn do_transition(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		_: &[Self::AssetId],
		_: &Self::Extra,
		payment_asset: Option<Self::PaymentFungible>,
	) -> Result<Vec<TransitionOutput<Self::AssetId, Self::Asset>>, TransitionError> {
		match transition_id {
			BattleMogsAction::Create => Self::create_mogwai(account_id),
			BattleMogsAction::Remove(mogwai_id) => Self::remove_mogwai(account_id, mogwai_id),
			BattleMogsAction::Hatch(mogwai_id) => Self::hatch_mogwai(account_id, mogwai_id),
			BattleMogsAction::Sacrifice(mogwai_id) =>
				Self::sacrifice_mogwai(account_id, mogwai_id, payment_asset),
			BattleMogsAction::SacrificeInto(sacrificed_id, into_id) =>
				Self::sacrifice_mogwai_into(account_id, sacrificed_id, into_id, payment_asset),
			BattleMogsAction::Morph(mogwai_id) =>
				Self::morph_mogwai(account_id, mogwai_id, payment_asset),
			BattleMogsAction::Breed(mogwai_id_1, mogwai_id_2) =>
				Self::breed_mogwais(account_id, mogwai_id_1, mogwai_id_2, payment_asset),
		}
	}
}
