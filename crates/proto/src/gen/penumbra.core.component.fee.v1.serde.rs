impl serde::Serialize for CurrentGasPricesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1.CurrentGasPricesRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CurrentGasPricesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Ok(GeneratedField::__SkipField__)
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CurrentGasPricesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1.CurrentGasPricesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CurrentGasPricesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(CurrentGasPricesRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1.CurrentGasPricesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CurrentGasPricesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.gas_prices.is_some() {
            len += 1;
        }
        if !self.alt_gas_prices.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1.CurrentGasPricesResponse", len)?;
        if let Some(v) = self.gas_prices.as_ref() {
            struct_ser.serialize_field("gasPrices", v)?;
        }
        if !self.alt_gas_prices.is_empty() {
            struct_ser.serialize_field("altGasPrices", &self.alt_gas_prices)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CurrentGasPricesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "gas_prices",
            "gasPrices",
            "alt_gas_prices",
            "altGasPrices",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GasPrices,
            AltGasPrices,
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
                            "gasPrices" | "gas_prices" => Ok(GeneratedField::GasPrices),
                            "altGasPrices" | "alt_gas_prices" => Ok(GeneratedField::AltGasPrices),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CurrentGasPricesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1.CurrentGasPricesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CurrentGasPricesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut gas_prices__ = None;
                let mut alt_gas_prices__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::GasPrices => {
                            if gas_prices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gasPrices"));
                            }
                            gas_prices__ = map_.next_value()?;
                        }
                        GeneratedField::AltGasPrices => {
                            if alt_gas_prices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("altGasPrices"));
                            }
                            alt_gas_prices__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CurrentGasPricesResponse {
                    gas_prices: gas_prices__,
                    alt_gas_prices: alt_gas_prices__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1.CurrentGasPricesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventBlockFees {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swapped_fee_total.is_some() {
            len += 1;
        }
        if self.swapped_base_fee_total.is_some() {
            len += 1;
        }
        if self.swapped_tip_total.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1.EventBlockFees", len)?;
        if let Some(v) = self.swapped_fee_total.as_ref() {
            struct_ser.serialize_field("swappedFeeTotal", v)?;
        }
        if let Some(v) = self.swapped_base_fee_total.as_ref() {
            struct_ser.serialize_field("swappedBaseFeeTotal", v)?;
        }
        if let Some(v) = self.swapped_tip_total.as_ref() {
            struct_ser.serialize_field("swappedTipTotal", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventBlockFees {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swapped_fee_total",
            "swappedFeeTotal",
            "swapped_base_fee_total",
            "swappedBaseFeeTotal",
            "swapped_tip_total",
            "swappedTipTotal",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwappedFeeTotal,
            SwappedBaseFeeTotal,
            SwappedTipTotal,
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
                            "swappedFeeTotal" | "swapped_fee_total" => Ok(GeneratedField::SwappedFeeTotal),
                            "swappedBaseFeeTotal" | "swapped_base_fee_total" => Ok(GeneratedField::SwappedBaseFeeTotal),
                            "swappedTipTotal" | "swapped_tip_total" => Ok(GeneratedField::SwappedTipTotal),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventBlockFees;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1.EventBlockFees")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventBlockFees, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swapped_fee_total__ = None;
                let mut swapped_base_fee_total__ = None;
                let mut swapped_tip_total__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwappedFeeTotal => {
                            if swapped_fee_total__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swappedFeeTotal"));
                            }
                            swapped_fee_total__ = map_.next_value()?;
                        }
                        GeneratedField::SwappedBaseFeeTotal => {
                            if swapped_base_fee_total__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swappedBaseFeeTotal"));
                            }
                            swapped_base_fee_total__ = map_.next_value()?;
                        }
                        GeneratedField::SwappedTipTotal => {
                            if swapped_tip_total__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swappedTipTotal"));
                            }
                            swapped_tip_total__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventBlockFees {
                    swapped_fee_total: swapped_fee_total__,
                    swapped_base_fee_total: swapped_base_fee_total__,
                    swapped_tip_total: swapped_tip_total__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1.EventBlockFees", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventPaidFee {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fee.is_some() {
            len += 1;
        }
        if self.base_fee.is_some() {
            len += 1;
        }
        if self.tip.is_some() {
            len += 1;
        }
        if self.gas_used.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1.EventPaidFee", len)?;
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        if let Some(v) = self.base_fee.as_ref() {
            struct_ser.serialize_field("baseFee", v)?;
        }
        if let Some(v) = self.tip.as_ref() {
            struct_ser.serialize_field("tip", v)?;
        }
        if let Some(v) = self.gas_used.as_ref() {
            struct_ser.serialize_field("gasUsed", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventPaidFee {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fee",
            "base_fee",
            "baseFee",
            "tip",
            "gas_used",
            "gasUsed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fee,
            BaseFee,
            Tip,
            GasUsed,
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
                            "fee" => Ok(GeneratedField::Fee),
                            "baseFee" | "base_fee" => Ok(GeneratedField::BaseFee),
                            "tip" => Ok(GeneratedField::Tip),
                            "gasUsed" | "gas_used" => Ok(GeneratedField::GasUsed),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventPaidFee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1.EventPaidFee")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventPaidFee, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee__ = None;
                let mut base_fee__ = None;
                let mut tip__ = None;
                let mut gas_used__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map_.next_value()?;
                        }
                        GeneratedField::BaseFee => {
                            if base_fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseFee"));
                            }
                            base_fee__ = map_.next_value()?;
                        }
                        GeneratedField::Tip => {
                            if tip__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tip"));
                            }
                            tip__ = map_.next_value()?;
                        }
                        GeneratedField::GasUsed => {
                            if gas_used__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gasUsed"));
                            }
                            gas_used__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventPaidFee {
                    fee: fee__,
                    base_fee: base_fee__,
                    tip: tip__,
                    gas_used: gas_used__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1.EventPaidFee", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Fee {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1.Fee", len)?;
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Fee {
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
            type Value = Fee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1.Fee")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Fee, V::Error>
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
                Ok(Fee {
                    amount: amount__,
                    asset_id: asset_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1.Fee", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FeeParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fixed_gas_prices.is_some() {
            len += 1;
        }
        if !self.fixed_alt_gas_prices.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1.FeeParameters", len)?;
        if let Some(v) = self.fixed_gas_prices.as_ref() {
            struct_ser.serialize_field("fixedGasPrices", v)?;
        }
        if !self.fixed_alt_gas_prices.is_empty() {
            struct_ser.serialize_field("fixedAltGasPrices", &self.fixed_alt_gas_prices)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FeeParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fixed_gas_prices",
            "fixedGasPrices",
            "fixed_alt_gas_prices",
            "fixedAltGasPrices",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FixedGasPrices,
            FixedAltGasPrices,
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
                            "fixedGasPrices" | "fixed_gas_prices" => Ok(GeneratedField::FixedGasPrices),
                            "fixedAltGasPrices" | "fixed_alt_gas_prices" => Ok(GeneratedField::FixedAltGasPrices),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FeeParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1.FeeParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FeeParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fixed_gas_prices__ = None;
                let mut fixed_alt_gas_prices__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FixedGasPrices => {
                            if fixed_gas_prices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fixedGasPrices"));
                            }
                            fixed_gas_prices__ = map_.next_value()?;
                        }
                        GeneratedField::FixedAltGasPrices => {
                            if fixed_alt_gas_prices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fixedAltGasPrices"));
                            }
                            fixed_alt_gas_prices__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FeeParameters {
                    fixed_gas_prices: fixed_gas_prices__,
                    fixed_alt_gas_prices: fixed_alt_gas_prices__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1.FeeParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FeeTier {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fee_tier != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1.FeeTier", len)?;
        if self.fee_tier != 0 {
            let v = fee_tier::Tier::try_from(self.fee_tier)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.fee_tier)))?;
            struct_ser.serialize_field("feeTier", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FeeTier {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fee_tier",
            "feeTier",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FeeTier,
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
                            "feeTier" | "fee_tier" => Ok(GeneratedField::FeeTier),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FeeTier;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1.FeeTier")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FeeTier, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee_tier__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FeeTier => {
                            if fee_tier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeTier"));
                            }
                            fee_tier__ = Some(map_.next_value::<fee_tier::Tier>()? as i32);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FeeTier {
                    fee_tier: fee_tier__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1.FeeTier", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for fee_tier::Tier {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "TIER_UNSPECIFIED",
            Self::Low => "TIER_LOW",
            Self::Medium => "TIER_MEDIUM",
            Self::High => "TIER_HIGH",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for fee_tier::Tier {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "TIER_UNSPECIFIED",
            "TIER_LOW",
            "TIER_MEDIUM",
            "TIER_HIGH",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = fee_tier::Tier;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "TIER_UNSPECIFIED" => Ok(fee_tier::Tier::Unspecified),
                    "TIER_LOW" => Ok(fee_tier::Tier::Low),
                    "TIER_MEDIUM" => Ok(fee_tier::Tier::Medium),
                    "TIER_HIGH" => Ok(fee_tier::Tier::High),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Gas {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block_space != 0 {
            len += 1;
        }
        if self.compact_block_space != 0 {
            len += 1;
        }
        if self.verification != 0 {
            len += 1;
        }
        if self.execution != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1.Gas", len)?;
        if self.block_space != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("blockSpace", ToString::to_string(&self.block_space).as_str())?;
        }
        if self.compact_block_space != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("compactBlockSpace", ToString::to_string(&self.compact_block_space).as_str())?;
        }
        if self.verification != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("verification", ToString::to_string(&self.verification).as_str())?;
        }
        if self.execution != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("execution", ToString::to_string(&self.execution).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Gas {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_space",
            "blockSpace",
            "compact_block_space",
            "compactBlockSpace",
            "verification",
            "execution",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockSpace,
            CompactBlockSpace,
            Verification,
            Execution,
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
                            "blockSpace" | "block_space" => Ok(GeneratedField::BlockSpace),
                            "compactBlockSpace" | "compact_block_space" => Ok(GeneratedField::CompactBlockSpace),
                            "verification" => Ok(GeneratedField::Verification),
                            "execution" => Ok(GeneratedField::Execution),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Gas;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1.Gas")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Gas, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_space__ = None;
                let mut compact_block_space__ = None;
                let mut verification__ = None;
                let mut execution__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BlockSpace => {
                            if block_space__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockSpace"));
                            }
                            block_space__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CompactBlockSpace => {
                            if compact_block_space__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compactBlockSpace"));
                            }
                            compact_block_space__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Verification => {
                            if verification__.is_some() {
                                return Err(serde::de::Error::duplicate_field("verification"));
                            }
                            verification__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Execution => {
                            if execution__.is_some() {
                                return Err(serde::de::Error::duplicate_field("execution"));
                            }
                            execution__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Gas {
                    block_space: block_space__.unwrap_or_default(),
                    compact_block_space: compact_block_space__.unwrap_or_default(),
                    verification: verification__.unwrap_or_default(),
                    execution: execution__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1.Gas", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GasPrices {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.asset_id.is_some() {
            len += 1;
        }
        if self.block_space_price != 0 {
            len += 1;
        }
        if self.compact_block_space_price != 0 {
            len += 1;
        }
        if self.verification_price != 0 {
            len += 1;
        }
        if self.execution_price != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1.GasPrices", len)?;
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        if self.block_space_price != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("blockSpacePrice", ToString::to_string(&self.block_space_price).as_str())?;
        }
        if self.compact_block_space_price != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("compactBlockSpacePrice", ToString::to_string(&self.compact_block_space_price).as_str())?;
        }
        if self.verification_price != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("verificationPrice", ToString::to_string(&self.verification_price).as_str())?;
        }
        if self.execution_price != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("executionPrice", ToString::to_string(&self.execution_price).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GasPrices {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_id",
            "assetId",
            "block_space_price",
            "blockSpacePrice",
            "compact_block_space_price",
            "compactBlockSpacePrice",
            "verification_price",
            "verificationPrice",
            "execution_price",
            "executionPrice",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssetId,
            BlockSpacePrice,
            CompactBlockSpacePrice,
            VerificationPrice,
            ExecutionPrice,
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
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            "blockSpacePrice" | "block_space_price" => Ok(GeneratedField::BlockSpacePrice),
                            "compactBlockSpacePrice" | "compact_block_space_price" => Ok(GeneratedField::CompactBlockSpacePrice),
                            "verificationPrice" | "verification_price" => Ok(GeneratedField::VerificationPrice),
                            "executionPrice" | "execution_price" => Ok(GeneratedField::ExecutionPrice),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GasPrices;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1.GasPrices")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GasPrices, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_id__ = None;
                let mut block_space_price__ = None;
                let mut compact_block_space_price__ = None;
                let mut verification_price__ = None;
                let mut execution_price__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::BlockSpacePrice => {
                            if block_space_price__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockSpacePrice"));
                            }
                            block_space_price__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CompactBlockSpacePrice => {
                            if compact_block_space_price__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compactBlockSpacePrice"));
                            }
                            compact_block_space_price__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VerificationPrice => {
                            if verification_price__.is_some() {
                                return Err(serde::de::Error::duplicate_field("verificationPrice"));
                            }
                            verification_price__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ExecutionPrice => {
                            if execution_price__.is_some() {
                                return Err(serde::de::Error::duplicate_field("executionPrice"));
                            }
                            execution_price__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GasPrices {
                    asset_id: asset_id__,
                    block_space_price: block_space_price__.unwrap_or_default(),
                    compact_block_space_price: compact_block_space_price__.unwrap_or_default(),
                    verification_price: verification_price__.unwrap_or_default(),
                    execution_price: execution_price__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1.GasPrices", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenesisContent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fee_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1.GenesisContent", len)?;
        if let Some(v) = self.fee_params.as_ref() {
            struct_ser.serialize_field("feeParams", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenesisContent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fee_params",
            "feeParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FeeParams,
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
                            "feeParams" | "fee_params" => Ok(GeneratedField::FeeParams),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenesisContent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FeeParams => {
                            if fee_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeParams"));
                            }
                            fee_params__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    fee_params: fee_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
