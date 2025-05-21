// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # PAsset Hub Runtime genesis config presets

use crate::*;
use alloc::{vec, vec::Vec};
use constants::{currency::UNITS as PAS, xcm_version::SAFE_XCM_VERSION};
use cumulus_primitives_core::ParaId;
use frame_support::build_struct_json_patch;
use hex_literal::hex;
use parachains_common::{AccountId, AuraId};
use sp_core::crypto::{Ss58Codec, UncheckedInto};
use sp_genesis_builder::PresetId;
use sp_keyring::Sr25519Keyring;

const PASSET_HUB_ED: Balance = ExistentialDeposit::get();
const PASSET_HUB_PARA_ID: u32 = 1111;
fn get_passet_hub_sudo() -> Option<AccountId> {
	AccountId::from_ss58check("15GWrigTkvLHjE8y3gFEXthb5Ux17v7c7MsW9sgYez1y9XWg").ok()
}
fn passet_hub_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	endowment: Balance,
	id: ParaId,
	sudo_key: Option<AccountId>,
) -> serde_json::Value {
	build_struct_json_patch!(RuntimeGenesisConfig {
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, endowment)).collect(),
		},
		parachain_info: ParachainInfoConfig { parachain_id: id },
		collator_selection: CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: PASSET_HUB_ED * 16,
		},
		session: SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),          // account id
						acc,                  // validator id
						SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		sudo: SudoConfig { key: sudo_key },
		polkadot_xcm: PolkadotXcmConfig { safe_xcm_version: Some(SAFE_XCM_VERSION) },
	})
}

/// Encapsulates names of predefined presets.
mod preset_names {
	pub const PRESET_GENESIS: &str = "genesis";
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	use preset_names::*;
	let patch = match id.as_ref() {
		PRESET_GENESIS => passet_hub_genesis(
			// One collator at genesis.
			vec![(
				hex!("ba665b203800a42e7bf67dcf77c77fb2ff7ece897fb32d395a074f786a7cbb23").into(),
				hex!("ba665b203800a42e7bf67dcf77c77fb2ff7ece897fb32d395a074f786a7cbb23")
					.unchecked_into(),
			)],
			Vec::new(),
			PASSET_HUB_ED * 4096,
			PASSET_HUB_PARA_ID.into(),
			get_passet_hub_sudo(),
		),
		sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => passet_hub_genesis(
			// initial collators.
			vec![
				(Sr25519Keyring::Alice.to_account_id(), Sr25519Keyring::Alice.public().into()),
				(Sr25519Keyring::Bob.to_account_id(), Sr25519Keyring::Bob.public().into()),
			],
			Sr25519Keyring::well_known().map(|k| k.to_account_id()).collect(),
			PAS * 1_000_000,
			PASSET_HUB_PARA_ID.into(),
			Some(Sr25519Keyring::Alice.to_account_id()),
		),
		sp_genesis_builder::DEV_RUNTIME_PRESET => passet_hub_genesis(
			// initial collators.
			vec![(Sr25519Keyring::Alice.to_account_id(), Sr25519Keyring::Alice.public().into())],
			vec![
				Sr25519Keyring::Alice.to_account_id(),
				Sr25519Keyring::Bob.to_account_id(),
				Sr25519Keyring::AliceStash.to_account_id(),
				Sr25519Keyring::BobStash.to_account_id(),
			],
			PAS * 1_000_000,
			PASSET_HUB_PARA_ID.into(),
			Some(Sr25519Keyring::Alice.to_account_id()),
		),
		_ => return None,
	};

	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work. qed.")
			.into_bytes(),
	)
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
	use preset_names::*;
	vec![
		PresetId::from(PRESET_GENESIS),
		PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
		PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
	]
}
