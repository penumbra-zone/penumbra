impl serde::Serialize for AppParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.chain_params.is_some() {
            len += 1;
        }
        if self.dao_params.is_some() {
            len += 1;
        }
        if self.governance_params.is_some() {
            len += 1;
        }
        if self.ibc_params.is_some() {
            len += 1;
        }
        if self.stake_params.is_some() {
            len += 1;
        }
        if self.fee_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.AppParameters", len)?;
        if let Some(v) = self.chain_params.as_ref() {
            struct_ser.serialize_field("chainParams", v)?;
        }
        if let Some(v) = self.dao_params.as_ref() {
            struct_ser.serialize_field("daoParams", v)?;
        }
        if let Some(v) = self.governance_params.as_ref() {
            struct_ser.serialize_field("governanceParams", v)?;
        }
        if let Some(v) = self.ibc_params.as_ref() {
            struct_ser.serialize_field("ibcParams", v)?;
        }
        if let Some(v) = self.stake_params.as_ref() {
            struct_ser.serialize_field("stakeParams", v)?;
        }
        if let Some(v) = self.fee_params.as_ref() {
            struct_ser.serialize_field("feeParams", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AppParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_params",
            "chainParams",
            "dao_params",
            "daoParams",
            "governance_params",
            "governanceParams",
            "ibc_params",
            "ibcParams",
            "stake_params",
            "stakeParams",
            "fee_params",
            "feeParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainParams,
            DaoParams,
            GovernanceParams,
            IbcParams,
            StakeParams,
            FeeParams,
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
                            "chainParams" | "chain_params" => Ok(GeneratedField::ChainParams),
                            "daoParams" | "dao_params" => Ok(GeneratedField::DaoParams),
                            "governanceParams" | "governance_params" => Ok(GeneratedField::GovernanceParams),
                            "ibcParams" | "ibc_params" => Ok(GeneratedField::IbcParams),
                            "stakeParams" | "stake_params" => Ok(GeneratedField::StakeParams),
                            "feeParams" | "fee_params" => Ok(GeneratedField::FeeParams),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AppParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.AppParameters")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AppParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_params__ = None;
                let mut dao_params__ = None;
                let mut governance_params__ = None;
                let mut ibc_params__ = None;
                let mut stake_params__ = None;
                let mut fee_params__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChainParams => {
                            if chain_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainParams"));
                            }
                            chain_params__ = map.next_value()?;
                        }
                        GeneratedField::DaoParams => {
                            if dao_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoParams"));
                            }
                            dao_params__ = map.next_value()?;
                        }
                        GeneratedField::GovernanceParams => {
                            if governance_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("governanceParams"));
                            }
                            governance_params__ = map.next_value()?;
                        }
                        GeneratedField::IbcParams => {
                            if ibc_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcParams"));
                            }
                            ibc_params__ = map.next_value()?;
                        }
                        GeneratedField::StakeParams => {
                            if stake_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakeParams"));
                            }
                            stake_params__ = map.next_value()?;
                        }
                        GeneratedField::FeeParams => {
                            if fee_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeParams"));
                            }
                            fee_params__ = map.next_value()?;
                        }
                    }
                }
                Ok(AppParameters {
                    chain_params: chain_params__,
                    dao_params: dao_params__,
                    governance_params: governance_params__,
                    ibc_params: ibc_params__,
                    stake_params: stake_params__,
                    fee_params: fee_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.AppParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AppParametersRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.AppParametersRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AppParametersRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AppParametersRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.AppParametersRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AppParametersRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AppParametersRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.AppParametersRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AppParametersResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.app_parameters.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.AppParametersResponse", len)?;
        if let Some(v) = self.app_parameters.as_ref() {
            struct_ser.serialize_field("appParameters", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AppParametersResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "app_parameters",
            "appParameters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AppParameters,
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
                            "appParameters" | "app_parameters" => Ok(GeneratedField::AppParameters),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AppParametersResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.AppParametersResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AppParametersResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut app_parameters__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AppParameters => {
                            if app_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appParameters"));
                            }
                            app_parameters__ = map.next_value()?;
                        }
                    }
                }
                Ok(AppParametersResponse {
                    app_parameters: app_parameters__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.AppParametersResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenesisAppState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.genesis_app_state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.GenesisAppState", len)?;
        if let Some(v) = self.genesis_app_state.as_ref() {
            match v {
                genesis_app_state::GenesisAppState::GenesisContent(v) => {
                    struct_ser.serialize_field("genesisContent", v)?;
                }
                genesis_app_state::GenesisAppState::GenesisCheckpoint(v) => {
                    struct_ser.serialize_field("genesisCheckpoint", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenesisAppState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "genesis_content",
            "genesisContent",
            "genesis_checkpoint",
            "genesisCheckpoint",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GenesisContent,
            GenesisCheckpoint,
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
                            "genesisContent" | "genesis_content" => Ok(GeneratedField::GenesisContent),
                            "genesisCheckpoint" | "genesis_checkpoint" => Ok(GeneratedField::GenesisCheckpoint),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenesisAppState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.GenesisAppState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisAppState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut genesis_app_state__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GenesisContent => {
                            if genesis_app_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("genesisContent"));
                            }
                            genesis_app_state__ = map.next_value::<::std::option::Option<_>>()?.map(genesis_app_state::GenesisAppState::GenesisContent)
;
                        }
                        GeneratedField::GenesisCheckpoint => {
                            if genesis_app_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("genesisCheckpoint"));
                            }
                            genesis_app_state__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| genesis_app_state::GenesisAppState::GenesisCheckpoint(x.0));
                        }
                    }
                }
                Ok(GenesisAppState {
                    genesis_app_state: genesis_app_state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.GenesisAppState", FIELDS, GeneratedVisitor)
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
        if self.stake_content.is_some() {
            len += 1;
        }
        if self.shielded_pool_content.is_some() {
            len += 1;
        }
        if self.governance_content.is_some() {
            len += 1;
        }
        if self.ibc_content.is_some() {
            len += 1;
        }
        if self.chain_content.is_some() {
            len += 1;
        }
        if self.dao_content.is_some() {
            len += 1;
        }
        if self.fee_content.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.GenesisContent", len)?;
        if let Some(v) = self.stake_content.as_ref() {
            struct_ser.serialize_field("stakeContent", v)?;
        }
        if let Some(v) = self.shielded_pool_content.as_ref() {
            struct_ser.serialize_field("shieldedPoolContent", v)?;
        }
        if let Some(v) = self.governance_content.as_ref() {
            struct_ser.serialize_field("governanceContent", v)?;
        }
        if let Some(v) = self.ibc_content.as_ref() {
            struct_ser.serialize_field("ibcContent", v)?;
        }
        if let Some(v) = self.chain_content.as_ref() {
            struct_ser.serialize_field("chainContent", v)?;
        }
        if let Some(v) = self.dao_content.as_ref() {
            struct_ser.serialize_field("daoContent", v)?;
        }
        if let Some(v) = self.fee_content.as_ref() {
            struct_ser.serialize_field("feeContent", v)?;
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
            "stake_content",
            "stakeContent",
            "shielded_pool_content",
            "shieldedPoolContent",
            "governance_content",
            "governanceContent",
            "ibc_content",
            "ibcContent",
            "chain_content",
            "chainContent",
            "dao_content",
            "daoContent",
            "fee_content",
            "feeContent",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StakeContent,
            ShieldedPoolContent,
            GovernanceContent,
            IbcContent,
            ChainContent,
            DaoContent,
            FeeContent,
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
                            "stakeContent" | "stake_content" => Ok(GeneratedField::StakeContent),
                            "shieldedPoolContent" | "shielded_pool_content" => Ok(GeneratedField::ShieldedPoolContent),
                            "governanceContent" | "governance_content" => Ok(GeneratedField::GovernanceContent),
                            "ibcContent" | "ibc_content" => Ok(GeneratedField::IbcContent),
                            "chainContent" | "chain_content" => Ok(GeneratedField::ChainContent),
                            "daoContent" | "dao_content" => Ok(GeneratedField::DaoContent),
                            "feeContent" | "fee_content" => Ok(GeneratedField::FeeContent),
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
                formatter.write_str("struct penumbra.core.app.v1alpha1.GenesisContent")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut stake_content__ = None;
                let mut shielded_pool_content__ = None;
                let mut governance_content__ = None;
                let mut ibc_content__ = None;
                let mut chain_content__ = None;
                let mut dao_content__ = None;
                let mut fee_content__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::StakeContent => {
                            if stake_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakeContent"));
                            }
                            stake_content__ = map.next_value()?;
                        }
                        GeneratedField::ShieldedPoolContent => {
                            if shielded_pool_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shieldedPoolContent"));
                            }
                            shielded_pool_content__ = map.next_value()?;
                        }
                        GeneratedField::GovernanceContent => {
                            if governance_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("governanceContent"));
                            }
                            governance_content__ = map.next_value()?;
                        }
                        GeneratedField::IbcContent => {
                            if ibc_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcContent"));
                            }
                            ibc_content__ = map.next_value()?;
                        }
                        GeneratedField::ChainContent => {
                            if chain_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainContent"));
                            }
                            chain_content__ = map.next_value()?;
                        }
                        GeneratedField::DaoContent => {
                            if dao_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoContent"));
                            }
                            dao_content__ = map.next_value()?;
                        }
                        GeneratedField::FeeContent => {
                            if fee_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeContent"));
                            }
                            fee_content__ = map.next_value()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    stake_content: stake_content__,
                    shielded_pool_content: shielded_pool_content__,
                    governance_content: governance_content__,
                    ibc_content: ibc_content__,
                    chain_content: chain_content__,
                    dao_content: dao_content__,
                    fee_content: fee_content__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for KeyValueRequest {
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
        if !self.key.is_empty() {
            len += 1;
        }
        if self.proof {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.KeyValueRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if !self.key.is_empty() {
            struct_ser.serialize_field("key", &self.key)?;
        }
        if self.proof {
            struct_ser.serialize_field("proof", &self.proof)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for KeyValueRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "key",
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            Key,
            Proof,
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
                            "key" => Ok(GeneratedField::Key),
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = KeyValueRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.KeyValueRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<KeyValueRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut key__ = None;
                let mut proof__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = Some(map.next_value()?);
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(KeyValueRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    key: key__.unwrap_or_default(),
                    proof: proof__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.KeyValueRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for KeyValueResponse {
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
        if self.proof.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.KeyValueResponse", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.proof.as_ref() {
            struct_ser.serialize_field("proof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for KeyValueResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Proof,
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
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = KeyValueResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.KeyValueResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<KeyValueResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut proof__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map.next_value()?;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = map.next_value()?;
                        }
                    }
                }
                Ok(KeyValueResponse {
                    value: value__,
                    proof: proof__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.KeyValueResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for key_value_response::Value {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.value.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.KeyValueResponse.Value", len)?;
        if !self.value.is_empty() {
            struct_ser.serialize_field("value", pbjson::private::base64::encode(&self.value).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for key_value_response::Value {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = key_value_response::Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.KeyValueResponse.Value")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<key_value_response::Value, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(key_value_response::Value {
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.KeyValueResponse.Value", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PrefixValueRequest {
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
        if !self.prefix.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.PrefixValueRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if !self.prefix.is_empty() {
            struct_ser.serialize_field("prefix", &self.prefix)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PrefixValueRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "prefix",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            Prefix,
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
                            "prefix" => Ok(GeneratedField::Prefix),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PrefixValueRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.PrefixValueRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PrefixValueRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut prefix__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Prefix => {
                            if prefix__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prefix"));
                            }
                            prefix__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PrefixValueRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    prefix: prefix__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.PrefixValueRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PrefixValueResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.key.is_empty() {
            len += 1;
        }
        if !self.value.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.PrefixValueResponse", len)?;
        if !self.key.is_empty() {
            struct_ser.serialize_field("key", &self.key)?;
        }
        if !self.value.is_empty() {
            struct_ser.serialize_field("value", pbjson::private::base64::encode(&self.value).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PrefixValueResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "key",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Key,
            Value,
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
                            "key" => Ok(GeneratedField::Key),
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PrefixValueResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.PrefixValueResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PrefixValueResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut key__ = None;
                let mut value__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = Some(map.next_value()?);
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(PrefixValueResponse {
                    key: key__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.PrefixValueResponse", FIELDS, GeneratedVisitor)
    }
}
