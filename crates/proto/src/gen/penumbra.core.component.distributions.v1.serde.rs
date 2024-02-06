impl serde::Serialize for DistributionsParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.staking_issuance_per_block != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.distributions.v1.DistributionsParameters", len)?;
        if self.staking_issuance_per_block != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("stakingIssuancePerBlock", ToString::to_string(&self.staking_issuance_per_block).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DistributionsParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "staking_issuance_per_block",
            "stakingIssuancePerBlock",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StakingIssuancePerBlock,
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
                            "stakingIssuancePerBlock" | "staking_issuance_per_block" => Ok(GeneratedField::StakingIssuancePerBlock),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DistributionsParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.distributions.v1.DistributionsParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DistributionsParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut staking_issuance_per_block__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StakingIssuancePerBlock => {
                            if staking_issuance_per_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakingIssuancePerBlock"));
                            }
                            staking_issuance_per_block__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DistributionsParameters {
                    staking_issuance_per_block: staking_issuance_per_block__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.distributions.v1.DistributionsParameters", FIELDS, GeneratedVisitor)
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
        if self.distributions_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.distributions.v1.GenesisContent", len)?;
        if let Some(v) = self.distributions_params.as_ref() {
            struct_ser.serialize_field("distributionsParams", v)?;
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
            "distributions_params",
            "distributionsParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DistributionsParams,
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
                            "distributionsParams" | "distributions_params" => Ok(GeneratedField::DistributionsParams),
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
                formatter.write_str("struct penumbra.core.component.distributions.v1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut distributions_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DistributionsParams => {
                            if distributions_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("distributionsParams"));
                            }
                            distributions_params__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    distributions_params: distributions_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.distributions.v1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
