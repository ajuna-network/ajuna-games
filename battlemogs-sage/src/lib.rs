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

use crate::{
	asset::{BattleMogsAsset, BattleMogsId},
	transitions::BattleMogsTransitionConfig,
};

use ajuna_primitives::sage_api::SageApi;
use sage_api::{traits::TransitionOutput, SageGameTransition, TransitionError};

use frame_support::pallet_prelude::*;
use parity_scale_codec::Codec;
use sp_core::H256;
use sp_runtime::traits::{AtLeast32BitUnsigned, BlockNumber as BlockNumberT};
use sp_std::marker::PhantomData;

mod algorithm;
pub mod asset;
pub mod config;
pub mod error;
pub mod transitions;

pub mod prelude {
	pub use crate::{
		asset::{
			achievement_table::*, mogwai::*, BattleMogsAsset, BattleMogsId, BattleMogsVariant,
		},
		error::*,
		transitions::BattleMogsTransitionConfig,
		BattleMogsTransition,
	};
}

pub mod sage_dependencies {
	pub use ajuna_primitives::sage_api::SageApi;
	pub use sage_api::{traits::TransitionOutput, SageGameTransition, TransitionError};
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum BattleMogsAction {
	RegisterPlayer,
	CreateMogwai,
	Remove { mogwai: BattleMogsId },
	Hatch { mogwai: BattleMogsId, table: BattleMogsId },
	Sacrifice { mogwai: BattleMogsId, table: BattleMogsId },
	SacrificeInto { mogwai: BattleMogsId, into: BattleMogsId, table: BattleMogsId },
	Morph { mogwai: BattleMogsId, table: BattleMogsId },
	Breed { mogwai_1: BattleMogsId, mogwai_2: BattleMogsId, table: BattleMogsId },
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
		AssetId = BattleMogsId,
		Asset = BattleMogsAsset<BlockNumber>,
		Balance = Balance,
		BlockNumber = BlockNumber,
		TransitionConfig = BattleMogsTransitionConfig,
		HashOutput = H256,
	>,
{
	type TransitionId = BattleMogsAction;
	type TransitionConfig = BattleMogsTransitionConfig;
	type AccountId = AccountId;
	type AssetId = BattleMogsId;
	type Asset = BattleMogsAsset<BlockNumber>;
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
			BattleMogsAction::RegisterPlayer => Self::register_player(account_id),
			BattleMogsAction::CreateMogwai => Self::create_mogwai(account_id),
			BattleMogsAction::Remove { mogwai } => Self::remove_mogwai(account_id, mogwai),
			BattleMogsAction::Hatch { mogwai, table } =>
				Self::hatch_mogwai(account_id, mogwai, table),
			BattleMogsAction::Sacrifice { mogwai, table } =>
				Self::sacrifice_mogwai(account_id, mogwai, table, payment_asset),
			BattleMogsAction::SacrificeInto { mogwai, into, table } =>
				Self::sacrifice_mogwai_into(account_id, mogwai, into, table, payment_asset),
			BattleMogsAction::Morph { mogwai, table } =>
				Self::morph_mogwai(account_id, mogwai, table, payment_asset),
			BattleMogsAction::Breed { mogwai_1, mogwai_2, table } =>
				Self::breed_mogwais(account_id, mogwai_1, mogwai_2, table, payment_asset),
		}
	}
}
