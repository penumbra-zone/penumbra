use penumbra_crypto::dex::TradingPair;

pub fn output_data(height: u64, trading_pair: TradingPair) -> String {
    format!(
        "dex/output/{}/{}/{}",
        height,
        &trading_pair.asset_1(),
        &trading_pair.asset_2()
    )
}
