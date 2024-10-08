use anyhow::{anyhow, Context as _};
use penumbra_app::genesis::{AppState, Content};
use serde_json::Value;

const GENESIS_NO_CONTENT_ERROR: &'static str = r#"
Error: using an upgrade genesis file instead of an initial genesis file.

This genesis file only contains a checkpoint hash of the state,
rather than information about how the initial state of the chain was initialized,
at the very first genesis.

Make sure that you're using the very first genesis file, before any upgrades.
"#;

/// Attempt to parse content from a value.
///
/// This is useful to get the initial chain state for app views.
///
/// This has a nice error message, so you should use this.
pub fn parse_content(data: Value) -> anyhow::Result<Content> {
    let app_state: AppState = serde_json::from_value(data)
        .context("error decoding app_state json: make sure that this is a penumbra genesis file")?;
    let content = app_state
        .content()
        .ok_or(anyhow!(GENESIS_NO_CONTENT_ERROR))?;
    Ok(content.clone())
}
