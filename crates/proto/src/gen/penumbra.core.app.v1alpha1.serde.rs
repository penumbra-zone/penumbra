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
        if self.community_pool_params.is_some() {
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
        if self.distributions_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.AppParameters", len)?;
        if let Some(v) = self.chain_params.as_ref() {
            struct_ser.serialize_field("chainParams", v)?;
        }
        if let Some(v) = self.community_pool_params.as_ref() {
            struct_ser.serialize_field("communityPoolParams", v)?;
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
        if let Some(v) = self.distributions_params.as_ref() {
            struct_ser.serialize_field("distributionsParams", v)?;
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
            "community_pool_params",
            "communityPoolParams",
            "governance_params",
            "governanceParams",
            "ibc_params",
            "ibcParams",
            "stake_params",
            "stakeParams",
            "fee_params",
            "feeParams",
            "distributions_params",
            "distributionsParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainParams,
            CommunityPoolParams,
            GovernanceParams,
            IbcParams,
            StakeParams,
            FeeParams,
            DistributionsParams,
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
                            "communityPoolParams" | "community_pool_params" => Ok(GeneratedField::CommunityPoolParams),
                            "governanceParams" | "governance_params" => Ok(GeneratedField::GovernanceParams),
                            "ibcParams" | "ibc_params" => Ok(GeneratedField::IbcParams),
                            "stakeParams" | "stake_params" => Ok(GeneratedField::StakeParams),
                            "feeParams" | "fee_params" => Ok(GeneratedField::FeeParams),
                            "distributionsParams" | "distributions_params" => Ok(GeneratedField::DistributionsParams),
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

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AppParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_params__ = None;
                let mut community_pool_params__ = None;
                let mut governance_params__ = None;
                let mut ibc_params__ = None;
                let mut stake_params__ = None;
                let mut fee_params__ = None;
                let mut distributions_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainParams => {
                            if chain_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainParams"));
                            }
                            chain_params__ = map_.next_value()?;
                        }
                        GeneratedField::CommunityPoolParams => {
                            if community_pool_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolParams"));
                            }
                            community_pool_params__ = map_.next_value()?;
                        }
                        GeneratedField::GovernanceParams => {
                            if governance_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("governanceParams"));
                            }
                            governance_params__ = map_.next_value()?;
                        }
                        GeneratedField::IbcParams => {
                            if ibc_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcParams"));
                            }
                            ibc_params__ = map_.next_value()?;
                        }
                        GeneratedField::StakeParams => {
                            if stake_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakeParams"));
                            }
                            stake_params__ = map_.next_value()?;
                        }
                        GeneratedField::FeeParams => {
                            if fee_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeParams"));
                            }
                            fee_params__ = map_.next_value()?;
                        }
                        GeneratedField::DistributionsParams => {
                            if distributions_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("distributionsParams"));
                            }
                            distributions_params__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AppParameters {
                    chain_params: chain_params__,
                    community_pool_params: community_pool_params__,
                    governance_params: governance_params__,
                    ibc_params: ibc_params__,
                    stake_params: stake_params__,
                    fee_params: fee_params__,
                    distributions_params: distributions_params__,
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

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AppParametersRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
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

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AppParametersResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut app_parameters__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AppParameters => {
                            if app_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appParameters"));
                            }
                            app_parameters__ = map_.next_value()?;
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
                    #[allow(clippy::needless_borrow)]
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

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisAppState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut genesis_app_state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::GenesisContent => {
                            if genesis_app_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("genesisContent"));
                            }
                            genesis_app_state__ = map_.next_value::<::std::option::Option<_>>()?.map(genesis_app_state::GenesisAppState::GenesisContent)
;
                        }
                        GeneratedField::GenesisCheckpoint => {
                            if genesis_app_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("genesisCheckpoint"));
                            }
                            genesis_app_state__ = map_.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| genesis_app_state::GenesisAppState::GenesisCheckpoint(x.0));
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
        if self.community_pool_content.is_some() {
            len += 1;
        }
        if self.fee_content.is_some() {
            len += 1;
        }
        if self.distributions_content.is_some() {
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
        if let Some(v) = self.community_pool_content.as_ref() {
            struct_ser.serialize_field("communityPoolContent", v)?;
        }
        if let Some(v) = self.fee_content.as_ref() {
            struct_ser.serialize_field("feeContent", v)?;
        }
        if let Some(v) = self.distributions_content.as_ref() {
            struct_ser.serialize_field("distributionsContent", v)?;
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
            "community_pool_content",
            "communityPoolContent",
            "fee_content",
            "feeContent",
            "distributions_content",
            "distributionsContent",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StakeContent,
            ShieldedPoolContent,
            GovernanceContent,
            IbcContent,
            ChainContent,
            CommunityPoolContent,
            FeeContent,
            DistributionsContent,
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
                            "communityPoolContent" | "community_pool_content" => Ok(GeneratedField::CommunityPoolContent),
                            "feeContent" | "fee_content" => Ok(GeneratedField::FeeContent),
                            "distributionsContent" | "distributions_content" => Ok(GeneratedField::DistributionsContent),
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

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut stake_content__ = None;
                let mut shielded_pool_content__ = None;
                let mut governance_content__ = None;
                let mut ibc_content__ = None;
                let mut chain_content__ = None;
                let mut community_pool_content__ = None;
                let mut fee_content__ = None;
                let mut distributions_content__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StakeContent => {
                            if stake_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakeContent"));
                            }
                            stake_content__ = map_.next_value()?;
                        }
                        GeneratedField::ShieldedPoolContent => {
                            if shielded_pool_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shieldedPoolContent"));
                            }
                            shielded_pool_content__ = map_.next_value()?;
                        }
                        GeneratedField::GovernanceContent => {
                            if governance_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("governanceContent"));
                            }
                            governance_content__ = map_.next_value()?;
                        }
                        GeneratedField::IbcContent => {
                            if ibc_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcContent"));
                            }
                            ibc_content__ = map_.next_value()?;
                        }
                        GeneratedField::ChainContent => {
                            if chain_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainContent"));
                            }
                            chain_content__ = map_.next_value()?;
                        }
                        GeneratedField::CommunityPoolContent => {
                            if community_pool_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolContent"));
                            }
                            community_pool_content__ = map_.next_value()?;
                        }
                        GeneratedField::FeeContent => {
                            if fee_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeContent"));
                            }
                            fee_content__ = map_.next_value()?;
                        }
                        GeneratedField::DistributionsContent => {
                            if distributions_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("distributionsContent"));
                            }
                            distributions_content__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    stake_content: stake_content__,
                    shielded_pool_content: shielded_pool_content__,
                    governance_content: governance_content__,
                    ibc_content: ibc_content__,
                    chain_content: chain_content__,
                    community_pool_content: community_pool_content__,
                    fee_content: fee_content__,
                    distributions_content: distributions_content__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionsByHeightRequest {
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
        if self.block_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.TransactionsByHeightRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.block_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("blockHeight", ToString::to_string(&self.block_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionsByHeightRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "block_height",
            "blockHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            BlockHeight,
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
                            "blockHeight" | "block_height" => Ok(GeneratedField::BlockHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionsByHeightRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.TransactionsByHeightRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionsByHeightRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut block_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BlockHeight => {
                            if block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHeight"));
                            }
                            block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(TransactionsByHeightRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    block_height: block_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.TransactionsByHeightRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionsByHeightResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.transactions.is_empty() {
            len += 1;
        }
        if self.block_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1alpha1.TransactionsByHeightResponse", len)?;
        if !self.transactions.is_empty() {
            struct_ser.serialize_field("transactions", &self.transactions)?;
        }
        if self.block_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("blockHeight", ToString::to_string(&self.block_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionsByHeightResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "transactions",
            "block_height",
            "blockHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Transactions,
            BlockHeight,
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
                            "transactions" => Ok(GeneratedField::Transactions),
                            "blockHeight" | "block_height" => Ok(GeneratedField::BlockHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionsByHeightResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1alpha1.TransactionsByHeightResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionsByHeightResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut transactions__ = None;
                let mut block_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Transactions => {
                            if transactions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactions"));
                            }
                            transactions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BlockHeight => {
                            if block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHeight"));
                            }
                            block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(TransactionsByHeightResponse {
                    transactions: transactions__.unwrap_or_default(),
                    block_height: block_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1alpha1.TransactionsByHeightResponse", FIELDS, GeneratedVisitor)
    }
}
