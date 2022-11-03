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

pub(crate) mod internal {
    use super::*;

    pub mod swap_flow {
        use penumbra_proto::Protobuf;

        use super::*;

        pub fn prefix() -> &'static str {
            "dex/swap_flow/"
        }
        pub fn item(trading_pair: &TradingPair) -> String {
            format!("{}{}", prefix(), hex::encode(trading_pair.encode_to_vec()),)
        }
    }
}
