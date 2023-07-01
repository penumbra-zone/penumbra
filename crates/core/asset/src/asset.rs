//! Asset types and identifiers.
mod cache;
mod denom;
mod denom_metadata;
mod id;
mod r1cs;
mod registry;

pub use cache::Cache;
pub use denom::Denom;
pub use denom_metadata::{DenomMetadata, Unit};
pub use id::{Id, VALUE_GENERATOR_DOMAIN_SEP};
pub use r1cs::AssetIdVar;
pub use registry::{Registry, REGISTRY};

// #[derive(Clone, Debug, Serialize, Deserialize)]
// #[serde(try_from = "pb::Asset", into = "pb::Asset")]
// pub struct Asset {
//     pub id: Id,
//     pub denom: DenomMetadata,
// }

// impl TypeUrl for Asset {
//     const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.Asset";
// }

// impl DomainType for Asset {
//     type Proto = pb::Asset;
// }

// impl TryFrom<pb::Asset> for Asset {
//     type Error = anyhow::Error;
//     fn try_from(asset: pb::Asset) -> anyhow::Result<Self> {
//         let denom = asset
//             .denom
//             .ok_or_else(|| anyhow::anyhow!("missing denom field in proto"))?
//             .try_into()?;

//         let dm = DenomMetadata::default_for(&denom)
//             .ok_or_else(|| anyhow::anyhow!("error generating metadata for denom"))?;

//         Ok(Self {
//             id: asset
//                 .id
//                 .ok_or_else(|| anyhow::anyhow!("missing id field in proto"))?
//                 .try_into()?,
//             denom: dm,
//         })
//     }
// }

// impl From<Asset> for pb::Asset {
//     fn from(asset: Asset) -> Self {
//         Self {
//             id: Some(asset.id.into()),
//             denom: Some(asset.denom.base_denom().into()),
//         }
//     }
// }

// impl TryFrom<AssetsResponse> for Asset {
//     type Error = anyhow::Error;

//     fn try_from(response: AssetsResponse) -> Result<Self, Self::Error> {
//         response
//             .asset
//             .ok_or_else(|| anyhow::anyhow!("empty AssetsResponse message"))?
//             .try_into()
//     }
// }

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn test_registry_native_token() {
        // We should be able to use `parse_base` with the valid base denomination.
        let base_denom = Cache::with_known_assets()
            .get_unit("upenumbra")
            .unwrap()
            .base();
        assert_eq!(format!("{base_denom}"), "upenumbra".to_string());

        // If we try to use `parse_base` with a display denomination, we should get `None`.
        let display_denoms = vec!["mpenumbra", "penumbra"];
        for display_denom in &display_denoms {
            assert!(REGISTRY.parse_denom(display_denom).is_none());
        }

        // We should be able to use the display denominations with `parse_display` however.
        for display_denom in display_denoms {
            let parsed_display_denom = REGISTRY.parse_unit(display_denom);

            assert_eq!(
                format!("{}", parsed_display_denom.base()),
                "upenumbra".to_string()
            );

            assert_eq!(format!("{parsed_display_denom}"), display_denom)
        }

        // The base denomination (upenumbra) can also be used for display purposes.
        let parsed_display_denom = REGISTRY.parse_unit("upenumbra");
        assert_eq!(
            format!("{}", parsed_display_denom.base()),
            "upenumbra".to_string()
        );
    }

    #[test]
    fn test_displaydenom_format_value() {
        // with exponent 6, 1782000 formats to 1.782
        let penumbra_display_denom = REGISTRY.parse_unit("penumbra");
        assert_eq!(
            penumbra_display_denom.format_value(1782000u64.into()),
            "1.782"
        );
        assert_eq!(
            penumbra_display_denom.format_value(6700001u64.into()),
            "6.700001"
        );
        assert_eq!(penumbra_display_denom.format_value(1u64.into()), "0.000001");

        // with exponent 3, 1782000 formats to 1782
        let mpenumbra_display_denom = REGISTRY.parse_unit("mpenumbra");
        assert_eq!(
            mpenumbra_display_denom.format_value(1782000u64.into()),
            "1782"
        );

        // with exponent 0, 1782000 formats to 1782000
        let upenumbra_display_denom = REGISTRY.parse_unit("upenumbra");
        assert_eq!(
            upenumbra_display_denom.format_value(1782000u64.into()),
            "1782000"
        );
    }

    #[test]
    fn best_unit_for() {
        let base_denom = Cache::with_known_assets()
            .get_unit("upenumbra")
            .unwrap()
            .base();

        assert_eq!(
            base_denom.best_unit_for(0u64.into()).to_string(),
            "penumbra"
        );
        assert_eq!(
            base_denom.best_unit_for(999u64.into()).to_string(),
            "upenumbra"
        );
        assert_eq!(
            base_denom.best_unit_for(1_000u64.into()).to_string(),
            "mpenumbra"
        );
        assert_eq!(
            base_denom.best_unit_for(999_999u64.into()).to_string(),
            "mpenumbra"
        );
        assert_eq!(
            base_denom.best_unit_for(1_000_000u64.into()).to_string(),
            "penumbra"
        );
    }

    #[test]
    fn test_displaydenom_parse_value() {
        let penumbra_display_denom = REGISTRY.parse_unit("penumbra");
        assert!(penumbra_display_denom.parse_value("1.2.3").is_err());

        assert_eq!(
            penumbra_display_denom.parse_value("1.782").unwrap(),
            1782000u64.into()
        );
        assert_eq!(
            penumbra_display_denom.parse_value("6.700001").unwrap(),
            6700001u64.into()
        );

        let mpenumbra_display_denom = REGISTRY.parse_unit("mpenumbra");
        assert_eq!(
            mpenumbra_display_denom.parse_value("1782").unwrap(),
            1782000u64.into()
        );
        assert!(mpenumbra_display_denom.parse_value("1782.0001").is_err());

        let upenumbra_display_denom = REGISTRY.parse_unit("upenumbra");
        assert_eq!(
            upenumbra_display_denom.parse_value("1782000").unwrap(),
            1782000u64.into()
        );
    }

    #[test]
    fn test_get_unit() {
        let unit = Cache::with_known_assets().get_unit("cube").unwrap();

        assert_eq!(format!("{unit}"), "cube".to_string());
    }

    proptest! {
        #[test]
        fn displaydenom_parsing_formatting_roundtrip(
            v: u128
        ) {
            let penumbra_display_denom = REGISTRY.parse_unit("penumbra");
            let formatted = penumbra_display_denom.format_value(v.into());
            let parsed = penumbra_display_denom.parse_value(&formatted);
            assert_eq!(v, u128::from(parsed.unwrap()));

            let mpenumbra_display_denom = REGISTRY.parse_unit("mpenumbra");
            let formatted = mpenumbra_display_denom.format_value(v.into());
            let parsed = mpenumbra_display_denom.parse_value(&formatted);
            assert_eq!(v, u128::from(parsed.unwrap()));
        }
    }
}
