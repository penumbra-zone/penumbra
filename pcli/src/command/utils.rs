use comfy_table::{presets, Table};
use penumbra_crypto::{
    asset,
    dex::{lp::position::Position, DirectedUnitPair},
};

pub(crate) fn render_positions(asset_cache: &asset::Cache, positions: &[Position]) -> String {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.set_header(vec![
        "ID",
        "Trading Pair",
        "State",
        "Reserves",
        "Trading Function",
    ]);

    for position in positions {
        let trading_pair = position.phi.pair;
        let denom_1 = asset_cache
            .get(&trading_pair.asset_1())
            .expect("asset should be known to view service");
        let denom_2 = asset_cache
            .get(&trading_pair.asset_2())
            .expect("asset should be known to view service");

        let display_denom_1 = denom_1.default_unit();
        let display_denom_2 = denom_2.default_unit();

        table.add_row(vec![
            format!("{}", position.id()),
            format!("({}, {})", display_denom_1, display_denom_2),
            position.state.to_string(),
            format!(
                "({}, {})",
                display_denom_1.format_value(position.reserves.r1),
                display_denom_2.format_value(position.reserves.r2)
            ),
            format!(
                "fee: {}bps, p/q: {:.6}, q/p: {:.18}",
                position.phi.component.fee,
                f64::from(position.phi.component.p) / f64::from(position.phi.component.q),
                f64::from(position.phi.component.q) / f64::from(position.phi.component.p),
            ),
        ]);
    }

    format!("{table}")
}

pub(crate) fn render_xyk_approximation(pair: DirectedUnitPair, positions: &[Position]) -> String {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.set_header(vec![
        "ID",
        "Trading Pair",
        "Reserves",
        "Pool fee",
        "Price summary",
        "Buy price",
        "Sell price",
    ]);

    for position in positions {
        let fee = position.phi.component.fee;
        let well_directed_phi = position.phi.orient_end(pair.end.id()).unwrap();
        let price_a_to_b = well_directed_phi.effective_price();
        let price_b_to_a = well_directed_phi.effective_price_inv();

        let buy_at_str = format!("buy  @ {:>10.05}", price_a_to_b);
        let sell_at_str = format!("sell @ {:>10.05}", price_b_to_a);

        let price_summary = if position.reserves_for(pair.start.id()).unwrap() == 0u64.into() {
            sell_at_str.clone()
        } else {
            buy_at_str.clone()
        };

        table.add_row(vec![
            format!("{}", position.id()),
            format!("{}", pair.to_string()),
            format!(
                "({}, {})",
                pair.start
                    .format_value(position.reserves_for(pair.start.id()).unwrap()),
                pair.end
                    .format_value(position.reserves_for(pair.end.id()).unwrap()),
            ),
            format!("{fee:>5}bps"),
            price_summary,
            buy_at_str,
            sell_at_str,
        ]);
    }

    format!("{table}")
}
