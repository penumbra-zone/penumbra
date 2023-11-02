impl serde::Serialize for DaoAssetBalancesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.chain_id.is_empty() {
            len += 1;
        }
        if !self.asset_ids.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dao.v1alpha1.DaoAssetBalancesRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if !self.asset_ids.is_empty() {
            struct_ser.serialize_field("assetIds", &self.asset_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DaoAssetBalancesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "asset_ids",
            "assetIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            AssetIds,
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
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "assetIds" | "asset_ids" => Ok(GeneratedField::AssetIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DaoAssetBalancesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dao.v1alpha1.DaoAssetBalancesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DaoAssetBalancesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut asset_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AssetIds => {
                            if asset_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetIds"));
                            }
                            asset_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DaoAssetBalancesRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    asset_ids: asset_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dao.v1alpha1.DaoAssetBalancesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DaoAssetBalancesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.balance.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dao.v1alpha1.DaoAssetBalancesResponse", len)?;
        if let Some(v) = self.balance.as_ref() {
            struct_ser.serialize_field("balance", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DaoAssetBalancesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "balance",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Balance,
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
                            "balance" => Ok(GeneratedField::Balance),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DaoAssetBalancesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dao.v1alpha1.DaoAssetBalancesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DaoAssetBalancesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut balance__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Balance => {
                            if balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balance"));
                            }
                            balance__ = map_.next_value()?;
                        }
                    }
                }
                Ok(DaoAssetBalancesResponse {
                    balance: balance__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dao.v1alpha1.DaoAssetBalancesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DaoParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.dao_spend_proposals_enabled {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dao.v1alpha1.DaoParameters", len)?;
        if self.dao_spend_proposals_enabled {
            struct_ser.serialize_field("daoSpendProposalsEnabled", &self.dao_spend_proposals_enabled)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DaoParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "dao_spend_proposals_enabled",
            "daoSpendProposalsEnabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DaoSpendProposalsEnabled,
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
                            "daoSpendProposalsEnabled" | "dao_spend_proposals_enabled" => Ok(GeneratedField::DaoSpendProposalsEnabled),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DaoParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dao.v1alpha1.DaoParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DaoParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut dao_spend_proposals_enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DaoSpendProposalsEnabled => {
                            if dao_spend_proposals_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoSpendProposalsEnabled"));
                            }
                            dao_spend_proposals_enabled__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DaoParameters {
                    dao_spend_proposals_enabled: dao_spend_proposals_enabled__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dao.v1alpha1.DaoParameters", FIELDS, GeneratedVisitor)
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
        if self.dao_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dao.v1alpha1.GenesisContent", len)?;
        if let Some(v) = self.dao_params.as_ref() {
            struct_ser.serialize_field("daoParams", v)?;
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
            "dao_params",
            "daoParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DaoParams,
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
                            "daoParams" | "dao_params" => Ok(GeneratedField::DaoParams),
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
                formatter.write_str("struct penumbra.core.component.dao.v1alpha1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut dao_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DaoParams => {
                            if dao_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoParams"));
                            }
                            dao_params__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    dao_params: dao_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dao.v1alpha1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
