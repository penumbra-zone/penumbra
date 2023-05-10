use comfy_table::{presets, Table};
use penumbra_crypto::{
    asset,
    dex::{lp::position::Position, DirectedUnitPair},
    Amount,
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
        let denom_1 = asset_cache.get(&trading_pair.asset_1());
        let denom_2 = asset_cache.get(&trading_pair.asset_2());

        // if either of the assets is unknown, just show the asset IDs
        let (d_display, v_display) = if denom_1.is_none() || denom_2.is_none() {
            (
                format!("({}, {})", trading_pair.asset_1(), trading_pair.asset_2()),
                format!(
                    "({} {}, {} {})",
                    position.reserves.r1,
                    trading_pair.asset_1(),
                    position.reserves.r2,
                    trading_pair.asset_2(),
                ),
            )
        } else {
            let display_denom_1 = denom_1.unwrap().default_unit();
            let display_denom_2 = denom_2.unwrap().default_unit();
            (
                format!("({}, {})", display_denom_1, display_denom_2),
                format!(
                    "({}, {})",
                    display_denom_1.format_value(position.reserves.r1),
                    display_denom_2.format_value(position.reserves.r2)
                ),
            )
        };

        table.add_row(vec![
            format!("{}", position.id()),
            d_display,
            position.state.to_string(),
            v_display,
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
        "State",
        "Reserves",
        "Trading Function",
    ]);

    for position in positions {
        let trading_pair = position.phi.pair;

        let well_directed_phi = position.phi.orient_end(pair.end.id()).unwrap();
        let price_a_to_b = well_directed_phi.effective_price();
        let price_b_to_a = well_directed_phi.effective_price_inv();

        let price_and_pair_entry =
            if position.reserves_for(pair.start.id()).unwrap() == Amount::zero() {
                format!(
                    "{}: {:.5}, fee: {}bps",
                    pair.flip().to_string(),
                    price_b_to_a,
                    position.phi.component.fee
                )
            } else {
                format!(
                    "{}: {:.5}, fee: {}bps",
                    pair.to_string(),
                    price_a_to_b,
                    position.phi.component.fee
                )
            };

        table.add_row(vec![
            format!("{}", position.id()),
            format!("{}", pair.to_string()),
            position.state.to_string(),
            format!(
                "({}, {})",
                pair.start
                    .format_value(position.reserves_for(pair.start.id()).unwrap()),
                pair.end
                    .format_value(position.reserves_for(pair.end.id()).unwrap()),
            ),
            price_and_pair_entry,
        ]);
    }

    format!("{table}")
}
