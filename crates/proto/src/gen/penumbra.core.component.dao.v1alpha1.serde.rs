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

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DaoParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut dao_spend_proposals_enabled__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DaoSpendProposalsEnabled => {
                            if dao_spend_proposals_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoSpendProposalsEnabled"));
                            }
                            dao_spend_proposals_enabled__ = Some(map.next_value()?);
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

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut dao_params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DaoParams => {
                            if dao_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoParams"));
                            }
                            dao_params__ = map.next_value()?;
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
