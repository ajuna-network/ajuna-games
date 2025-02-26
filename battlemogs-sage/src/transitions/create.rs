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
	algorithm::{Breeding, Generation},
	asset::{
		mogwai::{Mogwai as MogwaiVariant, MogwaiGeneration, PhaseType, RarityType},
		BattleMogsAsset, BattleMogsId, BattleMogsVariant,
	},
	transitions::{BattleMogsTransitionConfig, BattleMogsTransitionOutput, BreedType},
	BattleMogsTransition,
};

use ajuna_primitives::sage_api::SageApi;
use sage_api::{traits::TransitionOutput, TransitionError};

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
	pub(crate) fn create_mogwai(
		owner: &AccountId,
	) -> Result<BattleMogsTransitionOutput<BlockNumber>, TransitionError> {
		Self::ensure_not_max_mogwais(owner)?;

		let block_number = Sage::get_current_block_number();

		let mogwai_id = Self::new_asset_id(b"mogwai_id", block_number.saturated_into());

		let random_hash_1 = Sage::random_hash(b"create_mogwai");
		let random_hash_2 = Sage::random_hash(b"extend_mogwai");

		let (rarity, next_gen, max_rarity) = Generation::next_gen(
			MogwaiGeneration::First,
			RarityType::Common,
			MogwaiGeneration::First,
			RarityType::Common,
			random_hash_1.as_ref(),
		);
		let rarity = RarityType::from(((max_rarity as u8) << 4) + rarity as u8);

		let breed_type = BreedType::calculate_breed_type::<BlockNumber>(block_number);

		let dx = unsafe { &*(&random_hash_1.as_ref()[0..32] as *const [u8] as *const [u8; 32]) };
		let dy = unsafe { &*(&random_hash_2.as_ref()[0..32] as *const [u8] as *const [u8; 32]) };

		let final_dna = Breeding::pairing(breed_type, dx, dy);

		let mogwai =
			MogwaiVariant { dna: final_dna, generation: next_gen, rarity, phase: PhaseType::Bred };

		let asset = BattleMogsAsset {
			id: mogwai_id,
			genesis: block_number,
			variant: BattleMogsVariant::Mogwai(mogwai),
		};

		Ok(sp_std::vec![TransitionOutput::Minted(asset)])
	}
}
