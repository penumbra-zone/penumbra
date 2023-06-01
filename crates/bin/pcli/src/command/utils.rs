use comfy_table::{presets, Table};
use penumbra_crypto::{asset, Value};
use penumbra_dex::lp::position::Position;

pub(crate) fn render_positions(asset_cache: &asset::Cache, positions: &[Position]) -> String {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.set_header(vec!["ID", "State", "Fee", "Sell Price", "Reserves"]);
    table
        .get_column_mut(2)
        .unwrap()
        .set_cell_alignment(comfy_table::CellAlignment::Right);
    table
        .get_column_mut(3)
        .unwrap()
        .set_cell_alignment(comfy_table::CellAlignment::Right);
    table
        .get_column_mut(4)
        .unwrap()
        .set_cell_alignment(comfy_table::CellAlignment::Right);

    for position in positions {
        let trading_pair = position.phi.pair;
        let denom_1 = asset_cache.get(&trading_pair.asset_1());
        let denom_2 = asset_cache.get(&trading_pair.asset_2());

        match (denom_1, denom_2) {
            (Some(_), Some(_)) => {
                if let Some(sell_order) = position.interpret_as_sell() {
                    table.add_row(vec![
                        position.id().to_string(),
                        position.state.to_string(),
                        format!("{}bps", position.phi.component.fee),
                        format!(
                            "{}",
                            sell_order
                                .price_str(&asset_cache)
                                .expect("assets are known"),
                        ),
                        sell_order.offered.format(&asset_cache),
                    ]);
                } else if let Some((sell_order_1, sell_order_2)) = position.interpret_as_mixed() {
                    table.add_row(vec![
                        position.id().to_string(),
                        position.state.to_string(),
                        format!("{}bps", position.phi.component.fee),
                        format!(
                            "{}",
                            sell_order_1
                                .price_str(&asset_cache)
                                .expect("assets are known"),
                        ),
                        sell_order_1.offered.format(&asset_cache),
                    ]);
                    table.add_row(vec![
                        // Add a mark indicating this row is associated with the same position.
                        "└──────────────────────────────────────────────────────────────▶"
                            .to_string(),
                        String::new(),
                        format!("{}bps", position.phi.component.fee),
                        format!(
                            "{}",
                            sell_order_2
                                .price_str(&asset_cache)
                                .expect("assets are known"),
                        ),
                        sell_order_2.offered.format(&asset_cache),
                    ]);
                } else {
                    table.add_row(vec![
                        position.id().to_string(),
                        position.state.to_string(),
                        "Error interpreting position (this should not happen)".to_string(),
                    ]);
                }
            }
            (_, _) => {
                table.add_row(vec![
                    position.id().to_string(),
                    position.state.to_string(),
                    format!("{}bps", position.phi.component.fee),
                    format!("Unknown asset"),
                    Value {
                        amount: position.reserves.r1,
                        asset_id: position.phi.pair.asset_1(),
                    }
                    .format(&asset_cache),
                ]);
                table.add_row(vec![
                    String::new(),
                    String::new(),
                    format!("{}bps", position.phi.component.fee),
                    format!("Unknown asset"),
                    Value {
                        amount: position.reserves.r2,
                        asset_id: position.phi.pair.asset_2(),
                    }
                    .format(&asset_cache),
                ]);
            }
        }
    }

    format!("{table}")
}
