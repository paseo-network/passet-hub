// Copyright (C) Parity Technologies and the various Polkadot contributors, see Contributions.md
// for a list of specific contributors.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use serde::{Deserialize, Serialize};

/// Generic extensions for Parachain ChainSpecs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

pub type PassethubChainSpec = sc_chain_spec::GenericChainSpec<Extensions>;

pub fn passet_hub_local_config() -> Result<Box<dyn sc_chain_spec::ChainSpec>, String> {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("ss58Format".into(), 0.into());
	properties.insert("tokenSymbol".into(), "PAS".into());
	properties.insert("tokenDecimals".into(), 10.into());

	Ok(Box::new(
		PassethubChainSpec::builder(
			passet_hub_runtime::WASM_BINARY.expect("PAssetHub wasm not available!"),
			Extensions { relay_chain: "paseo-local".into(), para_id: 1111 },
		)
		.with_name("PAssetHub Local")
		.with_id("passet-hub-local")
		.with_chain_type(sc_chain_spec::ChainType::Local)
		.with_genesis_config_preset_name("development")
		.with_protocol_id("phub-local")
		.with_properties(properties)
		.build(),
	))
}
