use crate::App;

pub mod linear;
pub mod xyk;

use linear::Linear;
use penumbra_sdk_dex::DirectedUnitPair;
use penumbra_sdk_proto::core::component::dex::v1::{
    query_service_client::QueryServiceClient as DexQueryServiceClient, SpreadRequest,
};
use xyk::ConstantProduct;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Subcommand)]
pub enum ReplicateCmd {
    /// Create a set of positions that attempt to replicate an xy=k (UniV2) AMM.
    // Hidden pending further testing & review
    #[clap(visible_alias = "xyk", hide = true)]
    ConstantProduct(ConstantProduct),
    /// Create a set of positions that allocate liquidity linearly across a price range.
    Linear(Linear),
}

impl ReplicateCmd {
    pub async fn exec(&self, app: &mut App) -> anyhow::Result<()> {
        match self {
            ReplicateCmd::ConstantProduct(xyk_cmd) => xyk_cmd.exec(app).await?,
            ReplicateCmd::Linear(linear_cmd) => linear_cmd.exec(app).await?,
        };
        Ok(())
    }

    pub fn offline(&self) -> bool {
        false
    }
}

pub async fn process_price_or_fetch_spread(
    app: &mut App,
    user_price: Option<f64>,
    directed_pair: DirectedUnitPair,
) -> anyhow::Result<f64> {
    let canonical_pair = directed_pair.into_directed_trading_pair().to_canonical();

    if let Some(user_price) = user_price {
        let adjusted = adjust_price_by_exponents(user_price, &directed_pair);
        tracing::debug!(?user_price, ?adjusted, "adjusted price by units");
        return Ok(adjusted);
    } else {
        tracing::debug!("price not provided, fetching spread");

        let mut client = DexQueryServiceClient::new(app.pd_channel().await?);
        let spread_data = client
            .spread(SpreadRequest {
                trading_pair: Some(canonical_pair.into()),
            })
            .await?
            .into_inner();

        tracing::debug!(
            ?spread_data,
            pair = canonical_pair.to_string(),
            "fetched spread for pair"
        );

        if spread_data.best_1_to_2_position.is_none() || spread_data.best_2_to_1_position.is_none()
        {
            anyhow::bail!("couldn't find a market price for the specified assets, you can manually specify a price using --current-price <price>")
        }

        // The price is the amount of asset 2 required to buy 1 unit of asset 1

        if directed_pair.start.id() == canonical_pair.asset_1() {
            Ok(spread_data.approx_effective_price_2_to_1)
        } else if directed_pair.start.id() == canonical_pair.asset_2() {
            Ok(spread_data.approx_effective_price_1_to_2)
        } else {
            anyhow::bail!("the supplied liquidity must be on the pair")
        }
    }
}

fn adjust_price_by_exponents(price: f64, pair: &DirectedUnitPair) -> f64 {
    let start_exponent = pair.start.exponent() as i32;
    let end_exponent = pair.end.exponent() as i32;

    price * 10f64.powi(start_exponent - end_exponent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjust_price_by_exponents() {
        let pair1: DirectedUnitPair = "penumbra:gm".parse().unwrap();
        let pair2: DirectedUnitPair = "upenumbra:gm".parse().unwrap();
        let pair3: DirectedUnitPair = "penumbra:ugm".parse().unwrap();
        let pair4: DirectedUnitPair = "upenumbra:ugm".parse().unwrap();

        let base_price = 1.2;

        assert_eq!(adjust_price_by_exponents(base_price, &pair1), 1.2);
        assert_eq!(adjust_price_by_exponents(base_price, &pair2), 0.0000012);
        assert_eq!(adjust_price_by_exponents(base_price, &pair3), 1200000.0);
        assert_eq!(adjust_price_by_exponents(base_price, &pair4), 1.2);
    }
}
