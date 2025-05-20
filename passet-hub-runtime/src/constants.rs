/// Universally recognized accounts.
pub mod account {
	use frame_support::PalletId;

	/// Polkadot treasury pallet id, used to convert into AccountId
	pub const POLKADOT_TREASURY_PALLET_ID: PalletId = PalletId(*b"py/trsry");
	/// Alliance pallet ID.
	/// Used as a temporary place to deposit a slashed imbalance before teleporting to the Treasury.
	pub const ALLIANCE_PALLET_ID: PalletId = PalletId(*b"py/allia");
	/// Referenda pallet ID.
	/// Used as a temporary place to deposit a slashed imbalance before teleporting to the Treasury.
	pub const REFERENDA_PALLET_ID: PalletId = PalletId(*b"py/refer");
	/// Ambassador Referenda pallet ID.
	/// Used as a temporary place to deposit a slashed imbalance before teleporting to the Treasury.
	pub const AMBASSADOR_REFERENDA_PALLET_ID: PalletId = PalletId(*b"py/amref");
	/// Identity pallet ID.
	/// Used as a temporary place to deposit a slashed imbalance before teleporting to the Treasury.
	pub const IDENTITY_PALLET_ID: PalletId = PalletId(*b"py/ident");
	/// Fellowship treasury pallet ID
	pub const FELLOWSHIP_TREASURY_PALLET_ID: PalletId = PalletId(*b"py/feltr");
	/// Ambassador treasury pallet ID
	pub const AMBASSADOR_TREASURY_PALLET_ID: PalletId = PalletId(*b"py/ambtr");
}

/// Consensus-related.
pub mod consensus {
	use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight};

	/// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
	/// into the relay chain.
	pub const UNINCLUDED_SEGMENT_CAPACITY: u32 = 1;
	/// How many parachain blocks are processed by the relay chain per parent. Limits the
	/// number of blocks authored per slot.
	pub const BLOCK_PROCESSING_VELOCITY: u32 = 1;
	/// Relay chain slot duration, in milliseconds.
	pub const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;

	/// We allow for 2 seconds of compute with a 6 second average block.
	pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
		WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
		cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
	);

	/// This determines the average expected block time that we are targeting.
	/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
	/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
	/// up by `pallet_aura` to implement `fn slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const MILLISECS_PER_BLOCK: u64 = 6000;
	pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
}

/// Time-related
pub mod time {
	use polkadot_core_primitives::BlockNumber;

	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber =
		60_000 / (super::consensus::MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
}

/// Constants relating to DOT.
pub mod currency {
	use polkadot_core_primitives::Balance;

	/// Paseo existential deposit.
	pub const PASEO_EXISTENTIAL_DEPOSIT: Balance = 100 * CENTS;

	/// The default existential deposit for system chains. 1/10th of the Relay Chain's existential
	/// deposit. Individual system parachains may modify this in special cases.
	pub const EXISTENTIAL_DEPOSIT: Balance = PASEO_EXISTENTIAL_DEPOSIT / 10;

	/// One "DOT" that a UI would show a user.
	pub const UNITS: Balance = 10_000_000_000;
	pub const DOLLARS: Balance = UNITS; // 10_000_000_000
	pub const GRAND: Balance = DOLLARS * 1_000; // 10_000_000_000_000
	pub const CENTS: Balance = DOLLARS / 100; // 100_000_000
	pub const MILLICENTS: Balance = CENTS / 1_000; // 100_000

	// Paseo deposit function.
	const fn paseo_deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS
	}

	/// Deposit rate for stored data. 1/100th of the Relay Chain's deposit rate. `items` is the
	/// number of keys in storage and `bytes` is the size of the value.
	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		paseo_deposit(items, bytes) / 100
	}
}

/// Constants related to Polkadot fee payment.
pub mod fee {
	use frame_support::{
		pallet_prelude::Weight,
		weights::{
			constants::ExtrinsicBaseWeight, FeePolynomial, WeightToFeeCoefficient,
			WeightToFeeCoefficients, WeightToFeePolynomial,
		},
	};
	use polkadot_core_primitives::Balance;
	use smallvec::smallvec;
	pub use sp_runtime::Perbill;

	/// The block saturation level. Fees will be updates based on this value.
	pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

	/// Cost of every transaction byte at Polkadot system parachains.
	///
	/// It is the Relay Chain (Paseo) `TransactionByteFee` / 20.
	pub const TRANSACTION_BYTE_FEE: Balance = super::currency::MILLICENTS / 2;

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl frame_support::weights::WeightToFee for WeightToFee {
		type Balance = Balance;

		fn weight_to_fee(weight: &Weight) -> Self::Balance {
			let time_poly: FeePolynomial<Balance> = RefTimeToFee::polynomial().into();
			let proof_poly: FeePolynomial<Balance> = ProofSizeToFee::polynomial().into();

			// Take the maximum instead of the sum to charge by the more scarce resource.
			time_poly.eval(weight.ref_time()).max(proof_poly.eval(weight.proof_size()))
		}
	}

	/// Maps the reference time component of `Weight` to a fee.
	pub struct RefTimeToFee;
	impl WeightToFeePolynomial for RefTimeToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// In Polkadot, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
			// The standard system parachain configuration is 1/20 of that, as in 1/200 CENT.
			let p = super::currency::CENTS;
			let q = 200 * Balance::from(ExtrinsicBaseWeight::get().ref_time());

			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}

	/// Maps the proof size component of `Weight` to a fee.
	pub struct ProofSizeToFee;
	impl WeightToFeePolynomial for ProofSizeToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// Map 20kb proof to 1 CENT.
			let p = super::currency::CENTS;
			let q = 20_000;

			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}

	pub fn calculate_weight_to_fee(weight: &Weight) -> Balance {
		<WeightToFee as frame_support::weights::WeightToFee>::weight_to_fee(weight)
	}
}

pub mod snowbridge {
	use cumulus_primitives_core::ParaId;
	use frame_support::parameter_types;
	use xcm::prelude::{Location, NetworkId};

	/// The pallet index of the Ethereum inbound queue pallet in the bridge hub runtime.
	pub const INBOUND_QUEUE_PALLET_INDEX_V1: u8 = 80;
	pub const INBOUND_QUEUE_PALLET_INDEX_V2: u8 = 91;

	pub const FRONTEND_PALLET_INDEX: u8 = 36;

	parameter_types! {
		/// Network and location for the Ethereum chain. On Paseo, the Ethereum chain bridged
		/// to is the Sepolia Ethereum testnet, with chain ID 11155111.
		/// <https://chainlist.org/chain/11155111>
		/// <https://ethereum.org/en/developers/docs/apis/json-rpc/#net_version>
		pub EthereumNetwork: NetworkId = NetworkId::Ethereum { chain_id: 11155111 };
		pub EthereumLocation: Location = Location::new(2, EthereumNetwork::get());
		pub AssetHubParaId: ParaId = ParaId::from(super::system_parachain::ASSET_HUB_ID);
	}
}

pub mod xcm_version {
	/// The default XCM version to set in genesis config.
	pub const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
}

/// System Parachains.
pub mod system_parachain {
	use polkadot_primitives::Id;
	use xcm_builder::IsChildSystemParachain;

	/// Asset Hub parachain ID.
	pub const ASSET_HUB_ID: u32 = 1000;
	/// Collectives parachain ID.
	pub const COLLECTIVES_ID: u32 = 1001;
	/// Bridge Hub parachain ID.
	pub const BRIDGE_HUB_ID: u32 = 1002;
	/// People parachain ID.
	pub const PEOPLE_ID: u32 = 1004;
	/// Coretime Chain ID.
	pub const BROKER_ID: u32 = 1005;

	// System parachains from Polkadot point of view.
	pub type SystemParachains = IsChildSystemParachain<Id>;

	/// Coretime constants
	pub mod coretime {
		/// Coretime timeslice period in blocks
		/// WARNING: This constant is used across chains, so additional care should be taken
		/// when changing it.
		#[cfg(feature = "fast-runtime")]
		pub const TIMESLICE_PERIOD: u32 = 20;
		#[cfg(not(feature = "fast-runtime"))]
		pub const TIMESLICE_PERIOD: u32 = 80;
	}
}

/// Paseo Treasury pallet instance.
pub const TREASURY_PALLET_ID: u8 = 19;
