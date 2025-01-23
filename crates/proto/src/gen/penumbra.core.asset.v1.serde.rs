impl serde::Serialize for AssetId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.inner.is_empty() {
            len += 1;
        }
        if !self.alt_bech32m.is_empty() {
            len += 1;
        }
        if !self.alt_base_denom.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.AssetId", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        if !self.alt_bech32m.is_empty() {
            struct_ser.serialize_field("altBech32m", &self.alt_bech32m)?;
        }
        if !self.alt_base_denom.is_empty() {
            struct_ser.serialize_field("altBaseDenom", &self.alt_base_denom)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "inner",
            "alt_bech32m",
            "altBech32m",
            "alt_base_denom",
            "altBaseDenom",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Inner,
            AltBech32m,
            AltBaseDenom,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "inner" => Ok(GeneratedField::Inner),
                            "altBech32m" | "alt_bech32m" => Ok(GeneratedField::AltBech32m),
                            "altBaseDenom" | "alt_base_denom" => Ok(GeneratedField::AltBaseDenom),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AssetId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.AssetId")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AssetId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                let mut alt_bech32m__ = None;
                let mut alt_base_denom__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AltBech32m => {
                            if alt_bech32m__.is_some() {
                                return Err(serde::de::Error::duplicate_field("altBech32m"));
                            }
                            alt_bech32m__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AltBaseDenom => {
                            if alt_base_denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("altBaseDenom"));
                            }
                            alt_base_denom__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AssetId {
                    inner: inner__.unwrap_or_default(),
                    alt_bech32m: alt_bech32m__.unwrap_or_default(),
                    alt_base_denom: alt_base_denom__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.AssetId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AssetImage {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.png.is_empty() {
            len += 1;
        }
        if !self.svg.is_empty() {
            len += 1;
        }
        if self.theme.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.AssetImage", len)?;
        if !self.png.is_empty() {
            struct_ser.serialize_field("png", &self.png)?;
        }
        if !self.svg.is_empty() {
            struct_ser.serialize_field("svg", &self.svg)?;
        }
        if let Some(v) = self.theme.as_ref() {
            struct_ser.serialize_field("theme", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetImage {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "png",
            "svg",
            "theme",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Png,
            Svg,
            Theme,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "png" => Ok(GeneratedField::Png),
                            "svg" => Ok(GeneratedField::Svg),
                            "theme" => Ok(GeneratedField::Theme),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AssetImage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.AssetImage")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AssetImage, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut png__ = None;
                let mut svg__ = None;
                let mut theme__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Png => {
                            if png__.is_some() {
                                return Err(serde::de::Error::duplicate_field("png"));
                            }
                            png__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Svg => {
                            if svg__.is_some() {
                                return Err(serde::de::Error::duplicate_field("svg"));
                            }
                            svg__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Theme => {
                            if theme__.is_some() {
                                return Err(serde::de::Error::duplicate_field("theme"));
                            }
                            theme__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AssetImage {
                    png: png__.unwrap_or_default(),
                    svg: svg__.unwrap_or_default(),
                    theme: theme__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.AssetImage", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for asset_image::Theme {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.primary_color_hex.is_empty() {
            len += 1;
        }
        if self.circle {
            len += 1;
        }
        if self.dark_mode {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.AssetImage.Theme", len)?;
        if !self.primary_color_hex.is_empty() {
            struct_ser.serialize_field("primaryColorHex", &self.primary_color_hex)?;
        }
        if self.circle {
            struct_ser.serialize_field("circle", &self.circle)?;
        }
        if self.dark_mode {
            struct_ser.serialize_field("darkMode", &self.dark_mode)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for asset_image::Theme {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "primary_color_hex",
            "primaryColorHex",
            "circle",
            "dark_mode",
            "darkMode",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrimaryColorHex,
            Circle,
            DarkMode,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "primaryColorHex" | "primary_color_hex" => Ok(GeneratedField::PrimaryColorHex),
                            "circle" => Ok(GeneratedField::Circle),
                            "darkMode" | "dark_mode" => Ok(GeneratedField::DarkMode),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = asset_image::Theme;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.AssetImage.Theme")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<asset_image::Theme, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut primary_color_hex__ = None;
                let mut circle__ = None;
                let mut dark_mode__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrimaryColorHex => {
                            if primary_color_hex__.is_some() {
                                return Err(serde::de::Error::duplicate_field("primaryColorHex"));
                            }
                            primary_color_hex__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Circle => {
                            if circle__.is_some() {
                                return Err(serde::de::Error::duplicate_field("circle"));
                            }
                            circle__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DarkMode => {
                            if dark_mode__.is_some() {
                                return Err(serde::de::Error::duplicate_field("darkMode"));
                            }
                            dark_mode__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(asset_image::Theme {
                    primary_color_hex: primary_color_hex__.unwrap_or_default(),
                    circle: circle__.unwrap_or_default(),
                    dark_mode: dark_mode__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.AssetImage.Theme", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Balance {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.values.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.Balance", len)?;
        if !self.values.is_empty() {
            struct_ser.serialize_field("values", &self.values)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Balance {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "values",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Values,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "values" => Ok(GeneratedField::Values),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Balance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.Balance")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Balance, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut values__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Values => {
                            if values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("values"));
                            }
                            values__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Balance {
                    values: values__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.Balance", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for balance::SignedValue {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        if self.negated {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.Balance.SignedValue", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if self.negated {
            struct_ser.serialize_field("negated", &self.negated)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for balance::SignedValue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "negated",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Negated,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "value" => Ok(GeneratedField::Value),
                            "negated" => Ok(GeneratedField::Negated),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = balance::SignedValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.Balance.SignedValue")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<balance::SignedValue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut negated__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::Negated => {
                            if negated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("negated"));
                            }
                            negated__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(balance::SignedValue {
                    value: value__,
                    negated: negated__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.Balance.SignedValue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BalanceCommitment {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.inner.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.BalanceCommitment", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BalanceCommitment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "inner",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Inner,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "inner" => Ok(GeneratedField::Inner),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BalanceCommitment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.BalanceCommitment")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BalanceCommitment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(BalanceCommitment {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.BalanceCommitment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Denom {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.denom.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.Denom", len)?;
        if !self.denom.is_empty() {
            struct_ser.serialize_field("denom", &self.denom)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Denom {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "denom",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Denom,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "denom" => Ok(GeneratedField::Denom),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Denom;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.Denom")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Denom, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut denom__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Denom => {
                            if denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denom"));
                            }
                            denom__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Denom {
                    denom: denom__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.Denom", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DenomUnit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.denom.is_empty() {
            len += 1;
        }
        if self.exponent != 0 {
            len += 1;
        }
        if !self.aliases.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.DenomUnit", len)?;
        if !self.denom.is_empty() {
            struct_ser.serialize_field("denom", &self.denom)?;
        }
        if self.exponent != 0 {
            struct_ser.serialize_field("exponent", &self.exponent)?;
        }
        if !self.aliases.is_empty() {
            struct_ser.serialize_field("aliases", &self.aliases)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DenomUnit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "denom",
            "exponent",
            "aliases",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Denom,
            Exponent,
            Aliases,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "denom" => Ok(GeneratedField::Denom),
                            "exponent" => Ok(GeneratedField::Exponent),
                            "aliases" => Ok(GeneratedField::Aliases),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DenomUnit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.DenomUnit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DenomUnit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut denom__ = None;
                let mut exponent__ = None;
                let mut aliases__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Denom => {
                            if denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denom"));
                            }
                            denom__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Exponent => {
                            if exponent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exponent"));
                            }
                            exponent__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Aliases => {
                            if aliases__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aliases"));
                            }
                            aliases__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DenomUnit {
                    denom: denom__.unwrap_or_default(),
                    exponent: exponent__.unwrap_or_default(),
                    aliases: aliases__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.DenomUnit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EquivalentValue {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.equivalent_amount.is_some() {
            len += 1;
        }
        if self.numeraire.is_some() {
            len += 1;
        }
        if self.as_of_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.EquivalentValue", len)?;
        if let Some(v) = self.equivalent_amount.as_ref() {
            struct_ser.serialize_field("equivalentAmount", v)?;
        }
        if let Some(v) = self.numeraire.as_ref() {
            struct_ser.serialize_field("numeraire", v)?;
        }
        if self.as_of_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("asOfHeight", ToString::to_string(&self.as_of_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EquivalentValue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "equivalent_amount",
            "equivalentAmount",
            "numeraire",
            "as_of_height",
            "asOfHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EquivalentAmount,
            Numeraire,
            AsOfHeight,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "equivalentAmount" | "equivalent_amount" => Ok(GeneratedField::EquivalentAmount),
                            "numeraire" => Ok(GeneratedField::Numeraire),
                            "asOfHeight" | "as_of_height" => Ok(GeneratedField::AsOfHeight),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EquivalentValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.EquivalentValue")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EquivalentValue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut equivalent_amount__ = None;
                let mut numeraire__ = None;
                let mut as_of_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EquivalentAmount => {
                            if equivalent_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("equivalentAmount"));
                            }
                            equivalent_amount__ = map_.next_value()?;
                        }
                        GeneratedField::Numeraire => {
                            if numeraire__.is_some() {
                                return Err(serde::de::Error::duplicate_field("numeraire"));
                            }
                            numeraire__ = map_.next_value()?;
                        }
                        GeneratedField::AsOfHeight => {
                            if as_of_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asOfHeight"));
                            }
                            as_of_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EquivalentValue {
                    equivalent_amount: equivalent_amount__,
                    numeraire: numeraire__,
                    as_of_height: as_of_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.EquivalentValue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EstimatedPrice {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.priced_asset.is_some() {
            len += 1;
        }
        if self.numeraire.is_some() {
            len += 1;
        }
        if self.numeraire_per_unit != 0. {
            len += 1;
        }
        if self.as_of_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.EstimatedPrice", len)?;
        if let Some(v) = self.priced_asset.as_ref() {
            struct_ser.serialize_field("pricedAsset", v)?;
        }
        if let Some(v) = self.numeraire.as_ref() {
            struct_ser.serialize_field("numeraire", v)?;
        }
        if self.numeraire_per_unit != 0. {
            struct_ser.serialize_field("numerairePerUnit", &self.numeraire_per_unit)?;
        }
        if self.as_of_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("asOfHeight", ToString::to_string(&self.as_of_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EstimatedPrice {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "priced_asset",
            "pricedAsset",
            "numeraire",
            "numeraire_per_unit",
            "numerairePerUnit",
            "as_of_height",
            "asOfHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PricedAsset,
            Numeraire,
            NumerairePerUnit,
            AsOfHeight,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "pricedAsset" | "priced_asset" => Ok(GeneratedField::PricedAsset),
                            "numeraire" => Ok(GeneratedField::Numeraire),
                            "numerairePerUnit" | "numeraire_per_unit" => Ok(GeneratedField::NumerairePerUnit),
                            "asOfHeight" | "as_of_height" => Ok(GeneratedField::AsOfHeight),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EstimatedPrice;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.EstimatedPrice")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EstimatedPrice, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut priced_asset__ = None;
                let mut numeraire__ = None;
                let mut numeraire_per_unit__ = None;
                let mut as_of_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PricedAsset => {
                            if priced_asset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pricedAsset"));
                            }
                            priced_asset__ = map_.next_value()?;
                        }
                        GeneratedField::Numeraire => {
                            if numeraire__.is_some() {
                                return Err(serde::de::Error::duplicate_field("numeraire"));
                            }
                            numeraire__ = map_.next_value()?;
                        }
                        GeneratedField::NumerairePerUnit => {
                            if numeraire_per_unit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("numerairePerUnit"));
                            }
                            numeraire_per_unit__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AsOfHeight => {
                            if as_of_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asOfHeight"));
                            }
                            as_of_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EstimatedPrice {
                    priced_asset: priced_asset__,
                    numeraire: numeraire__,
                    numeraire_per_unit: numeraire_per_unit__.unwrap_or_default(),
                    as_of_height: as_of_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.EstimatedPrice", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Metadata {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.description.is_empty() {
            len += 1;
        }
        if !self.denom_units.is_empty() {
            len += 1;
        }
        if !self.base.is_empty() {
            len += 1;
        }
        if !self.display.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.symbol.is_empty() {
            len += 1;
        }
        if self.penumbra_asset_id.is_some() {
            len += 1;
        }
        if !self.images.is_empty() {
            len += 1;
        }
        if self.priority_score != 0 {
            len += 1;
        }
        if !self.badges.is_empty() {
            len += 1;
        }
        if !self.coingecko_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.Metadata", len)?;
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if !self.denom_units.is_empty() {
            struct_ser.serialize_field("denomUnits", &self.denom_units)?;
        }
        if !self.base.is_empty() {
            struct_ser.serialize_field("base", &self.base)?;
        }
        if !self.display.is_empty() {
            struct_ser.serialize_field("display", &self.display)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.symbol.is_empty() {
            struct_ser.serialize_field("symbol", &self.symbol)?;
        }
        if let Some(v) = self.penumbra_asset_id.as_ref() {
            struct_ser.serialize_field("penumbraAssetId", v)?;
        }
        if !self.images.is_empty() {
            struct_ser.serialize_field("images", &self.images)?;
        }
        if self.priority_score != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("priorityScore", ToString::to_string(&self.priority_score).as_str())?;
        }
        if !self.badges.is_empty() {
            struct_ser.serialize_field("badges", &self.badges)?;
        }
        if !self.coingecko_id.is_empty() {
            struct_ser.serialize_field("coingeckoId", &self.coingecko_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Metadata {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "description",
            "denom_units",
            "denomUnits",
            "base",
            "display",
            "name",
            "symbol",
            "penumbra_asset_id",
            "penumbraAssetId",
            "images",
            "priority_score",
            "priorityScore",
            "badges",
            "coingecko_id",
            "coingeckoId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Description,
            DenomUnits,
            Base,
            Display,
            Name,
            Symbol,
            PenumbraAssetId,
            Images,
            PriorityScore,
            Badges,
            CoingeckoId,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "description" => Ok(GeneratedField::Description),
                            "denomUnits" | "denom_units" => Ok(GeneratedField::DenomUnits),
                            "base" => Ok(GeneratedField::Base),
                            "display" => Ok(GeneratedField::Display),
                            "name" => Ok(GeneratedField::Name),
                            "symbol" => Ok(GeneratedField::Symbol),
                            "penumbraAssetId" | "penumbra_asset_id" => Ok(GeneratedField::PenumbraAssetId),
                            "images" => Ok(GeneratedField::Images),
                            "priorityScore" | "priority_score" => Ok(GeneratedField::PriorityScore),
                            "badges" => Ok(GeneratedField::Badges),
                            "coingeckoId" | "coingecko_id" => Ok(GeneratedField::CoingeckoId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Metadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.Metadata")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Metadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut description__ = None;
                let mut denom_units__ = None;
                let mut base__ = None;
                let mut display__ = None;
                let mut name__ = None;
                let mut symbol__ = None;
                let mut penumbra_asset_id__ = None;
                let mut images__ = None;
                let mut priority_score__ = None;
                let mut badges__ = None;
                let mut coingecko_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DenomUnits => {
                            if denom_units__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denomUnits"));
                            }
                            denom_units__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Base => {
                            if base__.is_some() {
                                return Err(serde::de::Error::duplicate_field("base"));
                            }
                            base__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Display => {
                            if display__.is_some() {
                                return Err(serde::de::Error::duplicate_field("display"));
                            }
                            display__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Symbol => {
                            if symbol__.is_some() {
                                return Err(serde::de::Error::duplicate_field("symbol"));
                            }
                            symbol__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PenumbraAssetId => {
                            if penumbra_asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("penumbraAssetId"));
                            }
                            penumbra_asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::Images => {
                            if images__.is_some() {
                                return Err(serde::de::Error::duplicate_field("images"));
                            }
                            images__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PriorityScore => {
                            if priority_score__.is_some() {
                                return Err(serde::de::Error::duplicate_field("priorityScore"));
                            }
                            priority_score__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Badges => {
                            if badges__.is_some() {
                                return Err(serde::de::Error::duplicate_field("badges"));
                            }
                            badges__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CoingeckoId => {
                            if coingecko_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("coingeckoId"));
                            }
                            coingecko_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Metadata {
                    description: description__.unwrap_or_default(),
                    denom_units: denom_units__.unwrap_or_default(),
                    base: base__.unwrap_or_default(),
                    display: display__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    symbol: symbol__.unwrap_or_default(),
                    penumbra_asset_id: penumbra_asset_id__,
                    images: images__.unwrap_or_default(),
                    priority_score: priority_score__.unwrap_or_default(),
                    badges: badges__.unwrap_or_default(),
                    coingecko_id: coingecko_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.Metadata", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Value {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.amount.is_some() {
            len += 1;
        }
        if self.asset_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.Value", len)?;
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Value {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
            "asset_id",
            "assetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Amount,
            AssetId,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "amount" => Ok(GeneratedField::Amount),
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.Value")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Value, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut asset_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map_.next_value()?;
                        }
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Value {
                    amount: amount__,
                    asset_id: asset_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.Value", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValueView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.ValueView", len)?;
        if let Some(v) = self.value_view.as_ref() {
            match v {
                value_view::ValueView::KnownAssetId(v) => {
                    struct_ser.serialize_field("knownAssetId", v)?;
                }
                value_view::ValueView::UnknownAssetId(v) => {
                    struct_ser.serialize_field("unknownAssetId", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValueView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "known_asset_id",
            "knownAssetId",
            "unknown_asset_id",
            "unknownAssetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            KnownAssetId,
            UnknownAssetId,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "knownAssetId" | "known_asset_id" => Ok(GeneratedField::KnownAssetId),
                            "unknownAssetId" | "unknown_asset_id" => Ok(GeneratedField::UnknownAssetId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValueView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.ValueView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValueView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value_view__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::KnownAssetId => {
                            if value_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("knownAssetId"));
                            }
                            value_view__ = map_.next_value::<::std::option::Option<_>>()?.map(value_view::ValueView::KnownAssetId)
;
                        }
                        GeneratedField::UnknownAssetId => {
                            if value_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unknownAssetId"));
                            }
                            value_view__ = map_.next_value::<::std::option::Option<_>>()?.map(value_view::ValueView::UnknownAssetId)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ValueView {
                    value_view: value_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.ValueView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for value_view::KnownAssetId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.amount.is_some() {
            len += 1;
        }
        if self.metadata.is_some() {
            len += 1;
        }
        if !self.equivalent_values.is_empty() {
            len += 1;
        }
        if self.extended_metadata.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.ValueView.KnownAssetId", len)?;
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if let Some(v) = self.metadata.as_ref() {
            struct_ser.serialize_field("metadata", v)?;
        }
        if !self.equivalent_values.is_empty() {
            struct_ser.serialize_field("equivalentValues", &self.equivalent_values)?;
        }
        if let Some(v) = self.extended_metadata.as_ref() {
            struct_ser.serialize_field("extendedMetadata", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for value_view::KnownAssetId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
            "metadata",
            "equivalent_values",
            "equivalentValues",
            "extended_metadata",
            "extendedMetadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Amount,
            Metadata,
            EquivalentValues,
            ExtendedMetadata,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "amount" => Ok(GeneratedField::Amount),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "equivalentValues" | "equivalent_values" => Ok(GeneratedField::EquivalentValues),
                            "extendedMetadata" | "extended_metadata" => Ok(GeneratedField::ExtendedMetadata),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = value_view::KnownAssetId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.ValueView.KnownAssetId")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<value_view::KnownAssetId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut metadata__ = None;
                let mut equivalent_values__ = None;
                let mut extended_metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map_.next_value()?;
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = map_.next_value()?;
                        }
                        GeneratedField::EquivalentValues => {
                            if equivalent_values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("equivalentValues"));
                            }
                            equivalent_values__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExtendedMetadata => {
                            if extended_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extendedMetadata"));
                            }
                            extended_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(value_view::KnownAssetId {
                    amount: amount__,
                    metadata: metadata__,
                    equivalent_values: equivalent_values__.unwrap_or_default(),
                    extended_metadata: extended_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.ValueView.KnownAssetId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for value_view::UnknownAssetId {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.amount.is_some() {
            len += 1;
        }
        if self.asset_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.asset.v1.ValueView.UnknownAssetId", len)?;
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for value_view::UnknownAssetId {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
            "asset_id",
            "assetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Amount,
            AssetId,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "amount" => Ok(GeneratedField::Amount),
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = value_view::UnknownAssetId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.asset.v1.ValueView.UnknownAssetId")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<value_view::UnknownAssetId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut asset_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map_.next_value()?;
                        }
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(value_view::UnknownAssetId {
                    amount: amount__,
                    asset_id: asset_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.asset.v1.ValueView.UnknownAssetId", FIELDS, GeneratedVisitor)
    }
}
