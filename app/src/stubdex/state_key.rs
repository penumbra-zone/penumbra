use penumbra_crypto::dex::TradingPair;

pub fn stub_cpmm_reserves(trading_pair: &TradingPair) -> String {
    format!(
        "dex/stub_cpmm_reserves/{}/{}",
        &trading_pair.asset_1(),
        &trading_pair.asset_2()
    )
}

pub fn output_data(height: u64, trading_pair: TradingPair) -> String {
    format!(
        "dex/output/{}/{}/{}",
        height,
        &trading_pair.asset_1(),
        &trading_pair.asset_2()
    )
}

pub fn swap_flows() -> &'static str {
    "dex/swap_flows"
}
