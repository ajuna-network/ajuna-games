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

use crate::{asset, error::*, BattleMogsTransition};

use ajuna_payment_handler::NativeId;
use ajuna_primitives::sage_api::SageApi;
use sage_api::{traits::TransitionOutput, TransitionError};

use frame_support::{
	ensure,
	pallet_prelude::{Decode, Encode, TypeInfo},
	Parameter,
};
use parity_scale_codec::{Codec, MaxEncodedLen};
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
	pub(crate) fn new_asset_id(subject: &[u8], nonce: u64) -> asset::BattleMogsId {
		Sage::random_hash(subject).to_low_u64_be().saturating_add(nonce)
	}

	pub(crate) fn ensure_not_max_mogwais(account: &AccountId) -> Result<(), TransitionError> {
		let mogwai_count =
			Sage::iter_assets_from(account).filter(|(_, asset)| asset.is_mogwai()).count();
		let max_mogwais = Sage::get_transition_config().max_mogwais;
		ensure!(
			mogwai_count <= max_mogwais as usize,
			TransitionError::Transition { code: MOGWAI_LIMIT_REACHED }
		);

		Ok(())
	}

	pub(crate) fn ensure_ownership(
		owner: &AccountId,
		mogwai_id: &asset::BattleMogsId,
	) -> Result<asset::BattleMogsAsset<BlockNumber>, TransitionError> {
		Sage::ensure_ownership(owner, mogwai_id).map_err(|_| TransitionError::AssetOwnership)
	}

	pub(crate) fn ensure_mogwai(
		asset: &asset::BattleMogsAsset<BlockNumber>,
	) -> Result<(), TransitionError> {
		ensure!(asset.is_mogwai(), TransitionError::Transition { code: ASSET_IS_NOT_MOGWAI });
		Ok(())
	}

	pub(crate) fn get_mogwai(
		mogwai_id: &asset::BattleMogsId,
	) -> Result<asset::BattleMogsAsset<BlockNumber>, TransitionError> {
		let asset = Sage::get_asset(mogwai_id)
			.map_err(|_| TransitionError::Transition { code: ASSET_NOT_FOUND })?;
		Self::ensure_mogwai(&asset)?;
		Ok(asset)
	}

	pub(crate) fn get_owned_mogwai(
		owner: &AccountId,
		mogwai_id: &asset::BattleMogsId,
	) -> Result<asset::BattleMogsAsset<BlockNumber>, TransitionError> {
		Self::ensure_ownership(owner, mogwai_id)?;
		let asset = Sage::get_asset(mogwai_id)
			.map_err(|_| TransitionError::Transition { code: ASSET_NOT_FOUND })?;
		Self::ensure_mogwai(&asset)?;
		Ok(asset)
	}

	pub(crate) fn get_payment_id(
		payment_asset: Option<Sage::FungiblesAssetId>,
	) -> Sage::FungiblesAssetId {
		if let Some(payment) = payment_asset {
			payment
		} else {
			Sage::FungiblesAssetId::get_native_id()
		}
	}

	pub(crate) fn inspect_asset_funds(
		asset_id: &asset::BattleMogsId,
		payment_asset: Option<Sage::FungiblesAssetId>,
	) -> Balance {
		let fund_id = Self::get_payment_id(payment_asset);
		Sage::inspect_asset_funds(asset_id, &fund_id)
	}

	pub(crate) fn deposit_funds_to_asset(
		asset_id: &asset::BattleMogsId,
		from: &AccountId,
		payment_asset: Option<Sage::FungiblesAssetId>,
		amount: Balance,
	) -> Result<(), TransitionError> {
		let fund_id = Self::get_payment_id(payment_asset);
		Sage::deposit_funds_to_asset(asset_id, from, fund_id, amount)
			.map_err(|_| TransitionError::Transition { code: ASSET_COULD_NOT_RECEIVE_FUNDS })
	}

	pub(crate) fn withdraw_funds_from_asset(
		asset_id: &asset::BattleMogsId,
		to: &AccountId,
		payment_asset: Option<Sage::FungiblesAssetId>,
		amount: Balance,
	) -> Result<(), TransitionError> {
		let fund_id = Self::get_payment_id(payment_asset);
		Sage::transfer_funds_from_asset(asset_id, to, fund_id, amount)
			.map_err(|_| TransitionError::Transition { code: ASSET_COULD_NOT_WITHDRAW_FUNDS })
	}
}
