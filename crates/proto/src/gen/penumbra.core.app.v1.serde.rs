impl serde::Serialize for AppParameters {
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
        if self.sct_params.is_some() {
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
        if self.funding_params.is_some() {
            len += 1;
        }
        if self.shielded_pool_params.is_some() {
            len += 1;
        }
        if self.dex_params.is_some() {
            len += 1;
        }
        if self.auction_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1.AppParameters", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if let Some(v) = self.sct_params.as_ref() {
            struct_ser.serialize_field("sctParams", v)?;
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
        if let Some(v) = self.funding_params.as_ref() {
            struct_ser.serialize_field("fundingParams", v)?;
        }
        if let Some(v) = self.shielded_pool_params.as_ref() {
            struct_ser.serialize_field("shieldedPoolParams", v)?;
        }
        if let Some(v) = self.dex_params.as_ref() {
            struct_ser.serialize_field("dexParams", v)?;
        }
        if let Some(v) = self.auction_params.as_ref() {
            struct_ser.serialize_field("auctionParams", v)?;
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
            "chain_id",
            "chainId",
            "sct_params",
            "sctParams",
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
            "funding_params",
            "fundingParams",
            "shielded_pool_params",
            "shieldedPoolParams",
            "dex_params",
            "dexParams",
            "auction_params",
            "auctionParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            SctParams,
            CommunityPoolParams,
            GovernanceParams,
            IbcParams,
            StakeParams,
            FeeParams,
            DistributionsParams,
            FundingParams,
            ShieldedPoolParams,
            DexParams,
            AuctionParams,
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
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "sctParams" | "sct_params" => Ok(GeneratedField::SctParams),
                            "communityPoolParams" | "community_pool_params" => Ok(GeneratedField::CommunityPoolParams),
                            "governanceParams" | "governance_params" => Ok(GeneratedField::GovernanceParams),
                            "ibcParams" | "ibc_params" => Ok(GeneratedField::IbcParams),
                            "stakeParams" | "stake_params" => Ok(GeneratedField::StakeParams),
                            "feeParams" | "fee_params" => Ok(GeneratedField::FeeParams),
                            "distributionsParams" | "distributions_params" => Ok(GeneratedField::DistributionsParams),
                            "fundingParams" | "funding_params" => Ok(GeneratedField::FundingParams),
                            "shieldedPoolParams" | "shielded_pool_params" => Ok(GeneratedField::ShieldedPoolParams),
                            "dexParams" | "dex_params" => Ok(GeneratedField::DexParams),
                            "auctionParams" | "auction_params" => Ok(GeneratedField::AuctionParams),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.app.v1.AppParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AppParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut sct_params__ = None;
                let mut community_pool_params__ = None;
                let mut governance_params__ = None;
                let mut ibc_params__ = None;
                let mut stake_params__ = None;
                let mut fee_params__ = None;
                let mut distributions_params__ = None;
                let mut funding_params__ = None;
                let mut shielded_pool_params__ = None;
                let mut dex_params__ = None;
                let mut auction_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SctParams => {
                            if sct_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sctParams"));
                            }
                            sct_params__ = map_.next_value()?;
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
                        GeneratedField::FundingParams => {
                            if funding_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fundingParams"));
                            }
                            funding_params__ = map_.next_value()?;
                        }
                        GeneratedField::ShieldedPoolParams => {
                            if shielded_pool_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shieldedPoolParams"));
                            }
                            shielded_pool_params__ = map_.next_value()?;
                        }
                        GeneratedField::DexParams => {
                            if dex_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dexParams"));
                            }
                            dex_params__ = map_.next_value()?;
                        }
                        GeneratedField::AuctionParams => {
                            if auction_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("auctionParams"));
                            }
                            auction_params__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AppParameters {
                    chain_id: chain_id__.unwrap_or_default(),
                    sct_params: sct_params__,
                    community_pool_params: community_pool_params__,
                    governance_params: governance_params__,
                    ibc_params: ibc_params__,
                    stake_params: stake_params__,
                    fee_params: fee_params__,
                    distributions_params: distributions_params__,
                    funding_params: funding_params__,
                    shielded_pool_params: shielded_pool_params__,
                    dex_params: dex_params__,
                    auction_params: auction_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1.AppParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AppParametersRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.app.v1.AppParametersRequest", len)?;
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
            type Value = AppParametersRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1.AppParametersRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AppParametersRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(AppParametersRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1.AppParametersRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1.AppParametersResponse", len)?;
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
                            "appParameters" | "app_parameters" => Ok(GeneratedField::AppParameters),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.app.v1.AppParametersResponse")
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AppParametersResponse {
                    app_parameters: app_parameters__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1.AppParametersResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventAppParametersChange {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.new_parameters.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1.EventAppParametersChange", len)?;
        if let Some(v) = self.new_parameters.as_ref() {
            struct_ser.serialize_field("newParameters", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventAppParametersChange {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "new_parameters",
            "newParameters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NewParameters,
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
                            "newParameters" | "new_parameters" => Ok(GeneratedField::NewParameters),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventAppParametersChange;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.app.v1.EventAppParametersChange")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventAppParametersChange, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut new_parameters__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NewParameters => {
                            if new_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newParameters"));
                            }
                            new_parameters__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventAppParametersChange {
                    new_parameters: new_parameters__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1.EventAppParametersChange", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1.GenesisAppState", len)?;
        if let Some(v) = self.genesis_app_state.as_ref() {
            match v {
                genesis_app_state::GenesisAppState::GenesisContent(v) => {
                    struct_ser.serialize_field("genesisContent", v)?;
                }
                genesis_app_state::GenesisAppState::GenesisCheckpoint(v) => {
                    #[allow(clippy::needless_borrow)]
                    #[allow(clippy::needless_borrows_for_generic_args)]
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
                            "genesisContent" | "genesis_content" => Ok(GeneratedField::GenesisContent),
                            "genesisCheckpoint" | "genesis_checkpoint" => Ok(GeneratedField::GenesisCheckpoint),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.app.v1.GenesisAppState")
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenesisAppState {
                    genesis_app_state: genesis_app_state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1.GenesisAppState", FIELDS, GeneratedVisitor)
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
        if !self.chain_id.is_empty() {
            len += 1;
        }
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
        if self.sct_content.is_some() {
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
        if self.funding_content.is_some() {
            len += 1;
        }
        if self.dex_content.is_some() {
            len += 1;
        }
        if self.auction_content.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1.GenesisContent", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
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
        if let Some(v) = self.sct_content.as_ref() {
            struct_ser.serialize_field("sctContent", v)?;
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
        if let Some(v) = self.funding_content.as_ref() {
            struct_ser.serialize_field("fundingContent", v)?;
        }
        if let Some(v) = self.dex_content.as_ref() {
            struct_ser.serialize_field("dexContent", v)?;
        }
        if let Some(v) = self.auction_content.as_ref() {
            struct_ser.serialize_field("auctionContent", v)?;
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
            "chain_id",
            "chainId",
            "stake_content",
            "stakeContent",
            "shielded_pool_content",
            "shieldedPoolContent",
            "governance_content",
            "governanceContent",
            "ibc_content",
            "ibcContent",
            "sct_content",
            "sctContent",
            "community_pool_content",
            "communityPoolContent",
            "fee_content",
            "feeContent",
            "distributions_content",
            "distributionsContent",
            "funding_content",
            "fundingContent",
            "dex_content",
            "dexContent",
            "auction_content",
            "auctionContent",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            StakeContent,
            ShieldedPoolContent,
            GovernanceContent,
            IbcContent,
            SctContent,
            CommunityPoolContent,
            FeeContent,
            DistributionsContent,
            FundingContent,
            DexContent,
            AuctionContent,
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
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "stakeContent" | "stake_content" => Ok(GeneratedField::StakeContent),
                            "shieldedPoolContent" | "shielded_pool_content" => Ok(GeneratedField::ShieldedPoolContent),
                            "governanceContent" | "governance_content" => Ok(GeneratedField::GovernanceContent),
                            "ibcContent" | "ibc_content" => Ok(GeneratedField::IbcContent),
                            "sctContent" | "sct_content" => Ok(GeneratedField::SctContent),
                            "communityPoolContent" | "community_pool_content" => Ok(GeneratedField::CommunityPoolContent),
                            "feeContent" | "fee_content" => Ok(GeneratedField::FeeContent),
                            "distributionsContent" | "distributions_content" => Ok(GeneratedField::DistributionsContent),
                            "fundingContent" | "funding_content" => Ok(GeneratedField::FundingContent),
                            "dexContent" | "dex_content" => Ok(GeneratedField::DexContent),
                            "auctionContent" | "auction_content" => Ok(GeneratedField::AuctionContent),
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
                formatter.write_str("struct penumbra.core.app.v1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut stake_content__ = None;
                let mut shielded_pool_content__ = None;
                let mut governance_content__ = None;
                let mut ibc_content__ = None;
                let mut sct_content__ = None;
                let mut community_pool_content__ = None;
                let mut fee_content__ = None;
                let mut distributions_content__ = None;
                let mut funding_content__ = None;
                let mut dex_content__ = None;
                let mut auction_content__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
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
                        GeneratedField::SctContent => {
                            if sct_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sctContent"));
                            }
                            sct_content__ = map_.next_value()?;
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
                        GeneratedField::FundingContent => {
                            if funding_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fundingContent"));
                            }
                            funding_content__ = map_.next_value()?;
                        }
                        GeneratedField::DexContent => {
                            if dex_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dexContent"));
                            }
                            dex_content__ = map_.next_value()?;
                        }
                        GeneratedField::AuctionContent => {
                            if auction_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("auctionContent"));
                            }
                            auction_content__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    chain_id: chain_id__.unwrap_or_default(),
                    stake_content: stake_content__,
                    shielded_pool_content: shielded_pool_content__,
                    governance_content: governance_content__,
                    ibc_content: ibc_content__,
                    sct_content: sct_content__,
                    community_pool_content: community_pool_content__,
                    fee_content: fee_content__,
                    distributions_content: distributions_content__,
                    funding_content: funding_content__,
                    dex_content: dex_content__,
                    auction_content: auction_content__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1.GenesisContent", FIELDS, GeneratedVisitor)
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
        if self.block_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1.TransactionsByHeightRequest", len)?;
        if self.block_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
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
            "block_height",
            "blockHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockHeight,
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
                            "blockHeight" | "block_height" => Ok(GeneratedField::BlockHeight),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.app.v1.TransactionsByHeightRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionsByHeightRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BlockHeight => {
                            if block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHeight"));
                            }
                            block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionsByHeightRequest {
                    block_height: block_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1.TransactionsByHeightRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.app.v1.TransactionsByHeightResponse", len)?;
        if !self.transactions.is_empty() {
            struct_ser.serialize_field("transactions", &self.transactions)?;
        }
        if self.block_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
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
                            "transactions" => Ok(GeneratedField::Transactions),
                            "blockHeight" | "block_height" => Ok(GeneratedField::BlockHeight),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.app.v1.TransactionsByHeightResponse")
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionsByHeightResponse {
                    transactions: transactions__.unwrap_or_default(),
                    block_height: block_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.app.v1.TransactionsByHeightResponse", FIELDS, GeneratedVisitor)
    }
}
