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

use crate::{asset, BattleMogsTransition};

use ajuna_primitives::sage_api::SageApi;

use frame_support::{
	pallet_prelude::{Decode, Encode, TypeInfo},
	Parameter,
};
use parity_scale_codec::{Codec, MaxEncodedLen};
use sage_api::traits::TransitionOutput;
use sp_core::H256;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, BlockNumber as BlockNumberT, Member},
	SaturatedConversion,
};

mod breed;
mod create;
mod hatch;
mod morph;
mod remove;
mod sacrifice;
mod sarifice_into;

pub(crate) type BattleMogsTransitionOutput<BlockNumber> =
	Vec<TransitionOutput<asset::BattleMogsId, asset::BattleMogsAsset<BlockNumber>>>;

#[derive(Encode, Decode, Debug, Copy, Clone, PartialEq, Eq, TypeInfo)]
pub enum BreedType {
	DomDom = 0,
	DomRez = 1,
	RezDom = 2,
	RezRez = 3,
}

impl BreedType {
	fn calculate_breed_type<BlockNumber>(block_number: BlockNumber) -> BreedType
	where
		BlockNumber: BlockNumberT,
	{
		let mod_value: u32 = 80;
		let modulo_80 = (block_number % mod_value.into()).saturated_into::<u32>();

		match modulo_80 {
			0..=19 => BreedType::DomDom,
			20..=39 => BreedType::DomRez,
			40..=59 => BreedType::RezDom,
			_ => BreedType::RezRez,
		}
	}
}

pub const DEFAULT_MAX_MOGWAIS: u16 = 10;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct BattleMogsTransitionConfig {
	pub max_mogwais: u16,
}

impl Default for BattleMogsTransitionConfig {
	fn default() -> Self {
		Self { max_mogwais: DEFAULT_MAX_MOGWAIS }
	}
}

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
}
