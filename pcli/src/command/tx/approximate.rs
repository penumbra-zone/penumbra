use std::path::PathBuf;

use crate::dex_utils;
use anyhow::anyhow;
use penumbra_crypto::dex::lp::position::Position;
use penumbra_crypto::dex::DirectedUnitPair;
use penumbra_crypto::Value;
use std::io::Write;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Subcommand)]
pub enum ApproximateCmd {
    #[clap(visible_alias = "xyk")]
    ConstantProduct(ConstantProduct),
}

#[derive(Debug, Clone, clap::Args)]
pub struct ConstantProduct {
    pub pair: DirectedUnitPair,
    pub input: Value,
    #[clap(short, long)]
    pub current_price: Option<f64>,
    #[clap(short, long, hide(true))]
    pub debug_file: Option<PathBuf>,
    #[clap(long, default_value = "0", hide(true))]
    pub source: u32,
}

impl ApproximateCmd {
    pub fn offline(&self) -> bool {
        false
    }
}

impl ConstantProduct {
    pub fn exec(&self, current_price: f64) -> anyhow::Result<Vec<Position>> {
        if self.input.asset_id != self.pair.start.id() && self.input.asset_id != self.pair.end.id()
        {
            anyhow::bail!("you must supply liquidity with an asset that's part of the market")
        } else if self.input.amount == 0u64.into() {
            anyhow::bail!("the quantity of liquidity supplied must be non-zero.",)
        } else {
            use crate::dex_utils::approximate::utils;
            let positions = crate::dex_utils::approximate::xyk::approximate(
                &self.pair,
                &self.input,
                current_price.try_into().expect("valid price"),
            )?;

            if let Some(file) = &self.debug_file {
                let canonical_pair_str = self.pair.to_canonical_string();

                // Ad-hoc denom scaling for debug data:
                let alphas = dex_utils::approximate::xyk::sample_points(
                    current_price,
                    dex_utils::approximate::xyk::NUM_POOLS_PRECISION,
                );

                let r1 = self.input.amount.value() as f64;
                // R2 scaled because the current_price is a ratio.
                let r2 = r1 * current_price;
                let denom_scaler = self.pair.end.unit_amount().value() as f64;
                let total_k = r1 * r2 * denom_scaler;

                // TODO(erwan): if we make `approximate` return a `Vec<PayoffPositionEntry>` we can `Into` directly
                let debug_positions: Vec<utils::PayoffPositionEntry> = positions
                    .iter()
                    .zip(alphas)
                    .enumerate()
                    .map(|(idx, (pos, alpha))| utils::PayoffPositionEntry {
                        payoff: Into::into(pos.clone()),
                        current_price,
                        index: idx,
                        canonical_pair: canonical_pair_str.clone(),
                        alpha,
                        total_k,
                    })
                    .collect();

                let mut fd = std::fs::File::create(&file).map_err(|e| {
                    anyhow!(
                        "fs error opening debug file {}: {}",
                        file.to_string_lossy(),
                        e
                    )
                })?;

                let json_data = serde_json::to_string(&debug_positions)
                    .map_err(|e| anyhow!("error serializing PayoffPositionEntry: {}", e))?;

                fd.write_all(json_data.as_bytes())
                    .map_err(|e| anyhow!("error writing {}: {}", file.to_string_lossy(), e))?;
            }
            Ok(positions)
        }
    }
}
