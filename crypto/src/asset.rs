//! Asset types and identifiers.

mod denom;
mod id;
mod registry;

pub use denom::{BaseDenom, DisplayDenom};
pub use id::Id;
pub use registry::{Registry, REGISTRY};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_native_token() {
        // We should be able to use `parse_base` with the valid base denomination.
        let base_denom = REGISTRY.parse_base("upenumbra").unwrap();
        assert_eq!(format!("{}", base_denom), "upenumbra".to_string());

        // If we try to use `parse_base` with a display denomination, we should get `None`.
        let display_denoms = vec!["mpenumbra", "penumbra"];
        for display_denom in &display_denoms {
            assert!(REGISTRY.parse_base(display_denom).is_none());
        }

        // We should be able to use the display denominations with `parse_display` however.
        for display_denom in display_denoms {
            let parsed_display_denom = REGISTRY.parse_display(display_denom).unwrap();

            assert_eq!(
                format!("{}", parsed_display_denom.base()),
                "upenumbra".to_string()
            );
        }

        // The base denomination (upenumbra) can also be used for display purposes.
        let parsed_display_denom = REGISTRY.parse_display("upenumbra").unwrap();
        assert_eq!(
            format!("{}", parsed_display_denom.base()),
            "upenumbra".to_string()
        );
    }
}
