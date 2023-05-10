use comfy_table::{presets, Table};
use penumbra_crypto::{asset, dex::lp::position::Position};

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
                "fee: {}bps, p/q: {:.6}, q/p: {:.6}",
                position.phi.component.fee,
                f64::from(position.phi.component.p) / f64::from(position.phi.component.q),
                f64::from(position.phi.component.q) / f64::from(position.phi.component.p),
            ),
        ]);
    }

    format!("{table}")
}
