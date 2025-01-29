impl serde::Serialize for EventFundingStreamReward {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.recipient.is_empty() {
            len += 1;
        }
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.reward_amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.EventFundingStreamReward", len)?;
        if !self.recipient.is_empty() {
            struct_ser.serialize_field("recipient", &self.recipient)?;
        }
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.reward_amount.as_ref() {
            struct_ser.serialize_field("rewardAmount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventFundingStreamReward {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "recipient",
            "epoch_index",
            "epochIndex",
            "reward_amount",
            "rewardAmount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Recipient,
            EpochIndex,
            RewardAmount,
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
                            "recipient" => Ok(GeneratedField::Recipient),
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "rewardAmount" | "reward_amount" => Ok(GeneratedField::RewardAmount),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventFundingStreamReward;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.EventFundingStreamReward")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventFundingStreamReward, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut recipient__ = None;
                let mut epoch_index__ = None;
                let mut reward_amount__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Recipient => {
                            if recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recipient"));
                            }
                            recipient__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RewardAmount => {
                            if reward_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewardAmount"));
                            }
                            reward_amount__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventFundingStreamReward {
                    recipient: recipient__.unwrap_or_default(),
                    epoch_index: epoch_index__.unwrap_or_default(),
                    reward_amount: reward_amount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.EventFundingStreamReward", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FundingParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.liquidity_tournament.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.FundingParameters", len)?;
        if let Some(v) = self.liquidity_tournament.as_ref() {
            struct_ser.serialize_field("liquidityTournament", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FundingParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "liquidity_tournament",
            "liquidityTournament",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LiquidityTournament,
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
                            "liquidityTournament" | "liquidity_tournament" => Ok(GeneratedField::LiquidityTournament),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FundingParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.FundingParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FundingParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut liquidity_tournament__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::LiquidityTournament => {
                            if liquidity_tournament__.is_some() {
                                return Err(serde::de::Error::duplicate_field("liquidityTournament"));
                            }
                            liquidity_tournament__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FundingParameters {
                    liquidity_tournament: liquidity_tournament__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.FundingParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for funding_parameters::LiquidityTournament {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.gauge_threshold_percent != 0 {
            len += 1;
        }
        if self.max_positions != 0 {
            len += 1;
        }
        if self.max_delegators != 0 {
            len += 1;
        }
        if self.delegator_share_percent != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.FundingParameters.LiquidityTournament", len)?;
        if self.gauge_threshold_percent != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("gaugeThresholdPercent", ToString::to_string(&self.gauge_threshold_percent).as_str())?;
        }
        if self.max_positions != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("maxPositions", ToString::to_string(&self.max_positions).as_str())?;
        }
        if self.max_delegators != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("maxDelegators", ToString::to_string(&self.max_delegators).as_str())?;
        }
        if self.delegator_share_percent != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("delegatorSharePercent", ToString::to_string(&self.delegator_share_percent).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for funding_parameters::LiquidityTournament {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "gauge_threshold_percent",
            "gaugeThresholdPercent",
            "max_positions",
            "maxPositions",
            "max_delegators",
            "maxDelegators",
            "delegator_share_percent",
            "delegatorSharePercent",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GaugeThresholdPercent,
            MaxPositions,
            MaxDelegators,
            DelegatorSharePercent,
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
                            "gaugeThresholdPercent" | "gauge_threshold_percent" => Ok(GeneratedField::GaugeThresholdPercent),
                            "maxPositions" | "max_positions" => Ok(GeneratedField::MaxPositions),
                            "maxDelegators" | "max_delegators" => Ok(GeneratedField::MaxDelegators),
                            "delegatorSharePercent" | "delegator_share_percent" => Ok(GeneratedField::DelegatorSharePercent),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = funding_parameters::LiquidityTournament;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.FundingParameters.LiquidityTournament")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<funding_parameters::LiquidityTournament, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut gauge_threshold_percent__ = None;
                let mut max_positions__ = None;
                let mut max_delegators__ = None;
                let mut delegator_share_percent__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::GaugeThresholdPercent => {
                            if gauge_threshold_percent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gaugeThresholdPercent"));
                            }
                            gauge_threshold_percent__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxPositions => {
                            if max_positions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxPositions"));
                            }
                            max_positions__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxDelegators => {
                            if max_delegators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxDelegators"));
                            }
                            max_delegators__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DelegatorSharePercent => {
                            if delegator_share_percent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorSharePercent"));
                            }
                            delegator_share_percent__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(funding_parameters::LiquidityTournament {
                    gauge_threshold_percent: gauge_threshold_percent__.unwrap_or_default(),
                    max_positions: max_positions__.unwrap_or_default(),
                    max_delegators: max_delegators__.unwrap_or_default(),
                    delegator_share_percent: delegator_share_percent__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.FundingParameters.LiquidityTournament", FIELDS, GeneratedVisitor)
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
        if self.funding_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.GenesisContent", len)?;
        if let Some(v) = self.funding_params.as_ref() {
            struct_ser.serialize_field("fundingParams", v)?;
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
            "funding_params",
            "fundingParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FundingParams,
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
                            "fundingParams" | "funding_params" => Ok(GeneratedField::FundingParams),
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
                formatter.write_str("struct penumbra.core.component.funding.v1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut funding_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FundingParams => {
                            if funding_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fundingParams"));
                            }
                            funding_params__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    funding_params: funding_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
