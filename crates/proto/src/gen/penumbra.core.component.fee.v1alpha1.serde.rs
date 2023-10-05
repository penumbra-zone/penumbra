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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1alpha1.Fee", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.component.fee.v1alpha1.Fee")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Fee, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut asset_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map.next_value()?;
                        }
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map.next_value()?;
                        }
                    }
                }
                Ok(Fee {
                    amount: amount__,
                    asset_id: asset_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1alpha1.Fee", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FeeParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1alpha1.FeeParameters", len)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FeeParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.fee.v1alpha1.FeeParameters")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FeeParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(FeeParameters {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1alpha1.FeeParameters", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1alpha1.GasPrices", len)?;
        if self.block_space_price != 0 {
            struct_ser.serialize_field("blockSpacePrice", ToString::to_string(&self.block_space_price).as_str())?;
        }
        if self.compact_block_space_price != 0 {
            struct_ser.serialize_field("compactBlockSpacePrice", ToString::to_string(&self.compact_block_space_price).as_str())?;
        }
        if self.verification_price != 0 {
            struct_ser.serialize_field("verificationPrice", ToString::to_string(&self.verification_price).as_str())?;
        }
        if self.execution_price != 0 {
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
            BlockSpacePrice,
            CompactBlockSpacePrice,
            VerificationPrice,
            ExecutionPrice,
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
                            "blockSpacePrice" | "block_space_price" => Ok(GeneratedField::BlockSpacePrice),
                            "compactBlockSpacePrice" | "compact_block_space_price" => Ok(GeneratedField::CompactBlockSpacePrice),
                            "verificationPrice" | "verification_price" => Ok(GeneratedField::VerificationPrice),
                            "executionPrice" | "execution_price" => Ok(GeneratedField::ExecutionPrice),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.component.fee.v1alpha1.GasPrices")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GasPrices, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_space_price__ = None;
                let mut compact_block_space_price__ = None;
                let mut verification_price__ = None;
                let mut execution_price__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BlockSpacePrice => {
                            if block_space_price__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockSpacePrice"));
                            }
                            block_space_price__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CompactBlockSpacePrice => {
                            if compact_block_space_price__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compactBlockSpacePrice"));
                            }
                            compact_block_space_price__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VerificationPrice => {
                            if verification_price__.is_some() {
                                return Err(serde::de::Error::duplicate_field("verificationPrice"));
                            }
                            verification_price__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ExecutionPrice => {
                            if execution_price__.is_some() {
                                return Err(serde::de::Error::duplicate_field("executionPrice"));
                            }
                            execution_price__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(GasPrices {
                    block_space_price: block_space_price__.unwrap_or_default(),
                    compact_block_space_price: compact_block_space_price__.unwrap_or_default(),
                    verification_price: verification_price__.unwrap_or_default(),
                    execution_price: execution_price__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1alpha1.GasPrices", FIELDS, GeneratedVisitor)
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
        if self.gas_prices.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.fee.v1alpha1.GenesisContent", len)?;
        if let Some(v) = self.fee_params.as_ref() {
            struct_ser.serialize_field("feeParams", v)?;
        }
        if let Some(v) = self.gas_prices.as_ref() {
            struct_ser.serialize_field("gasPrices", v)?;
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
            "gas_prices",
            "gasPrices",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FeeParams,
            GasPrices,
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
                            "gasPrices" | "gas_prices" => Ok(GeneratedField::GasPrices),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.component.fee.v1alpha1.GenesisContent")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee_params__ = None;
                let mut gas_prices__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FeeParams => {
                            if fee_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeParams"));
                            }
                            fee_params__ = map.next_value()?;
                        }
                        GeneratedField::GasPrices => {
                            if gas_prices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gasPrices"));
                            }
                            gas_prices__ = map.next_value()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    fee_params: fee_params__,
                    gas_prices: gas_prices__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.fee.v1alpha1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
