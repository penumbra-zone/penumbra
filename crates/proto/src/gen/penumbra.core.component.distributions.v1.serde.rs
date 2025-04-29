impl serde::Serialize for CurrentLqtPoolSizeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.component.distributions.v1.CurrentLqtPoolSizeRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CurrentLqtPoolSizeRequest {
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
            type Value = CurrentLqtPoolSizeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.distributions.v1.CurrentLqtPoolSizeRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CurrentLqtPoolSizeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(CurrentLqtPoolSizeRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.distributions.v1.CurrentLqtPoolSizeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CurrentLqtPoolSizeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.pool_size.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.distributions.v1.CurrentLqtPoolSizeResponse", len)?;
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.pool_size.as_ref() {
            struct_ser.serialize_field("poolSize", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CurrentLqtPoolSizeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch_index",
            "epochIndex",
            "pool_size",
            "poolSize",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EpochIndex,
            PoolSize,
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
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "poolSize" | "pool_size" => Ok(GeneratedField::PoolSize),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CurrentLqtPoolSizeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.distributions.v1.CurrentLqtPoolSizeResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CurrentLqtPoolSizeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch_index__ = None;
                let mut pool_size__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PoolSize => {
                            if pool_size__.is_some() {
                                return Err(serde::de::Error::duplicate_field("poolSize"));
                            }
                            pool_size__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CurrentLqtPoolSizeResponse {
                    epoch_index: epoch_index__.unwrap_or_default(),
                    pool_size: pool_size__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.distributions.v1.CurrentLqtPoolSizeResponse", FIELDS, GeneratedVisitor)
    }
}
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
        if self.liquidity_tournament_incentive_per_block != 0 {
            len += 1;
        }
        if self.liquidity_tournament_end_block != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.distributions.v1.DistributionsParameters", len)?;
        if self.staking_issuance_per_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("stakingIssuancePerBlock", ToString::to_string(&self.staking_issuance_per_block).as_str())?;
        }
        if self.liquidity_tournament_incentive_per_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("liquidityTournamentIncentivePerBlock", ToString::to_string(&self.liquidity_tournament_incentive_per_block).as_str())?;
        }
        if self.liquidity_tournament_end_block != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("liquidityTournamentEndBlock", ToString::to_string(&self.liquidity_tournament_end_block).as_str())?;
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
            "liquidity_tournament_incentive_per_block",
            "liquidityTournamentIncentivePerBlock",
            "liquidity_tournament_end_block",
            "liquidityTournamentEndBlock",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StakingIssuancePerBlock,
            LiquidityTournamentIncentivePerBlock,
            LiquidityTournamentEndBlock,
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
                            "liquidityTournamentIncentivePerBlock" | "liquidity_tournament_incentive_per_block" => Ok(GeneratedField::LiquidityTournamentIncentivePerBlock),
                            "liquidityTournamentEndBlock" | "liquidity_tournament_end_block" => Ok(GeneratedField::LiquidityTournamentEndBlock),
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
                let mut liquidity_tournament_incentive_per_block__ = None;
                let mut liquidity_tournament_end_block__ = None;
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
                        GeneratedField::LiquidityTournamentIncentivePerBlock => {
                            if liquidity_tournament_incentive_per_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("liquidityTournamentIncentivePerBlock"));
                            }
                            liquidity_tournament_incentive_per_block__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LiquidityTournamentEndBlock => {
                            if liquidity_tournament_end_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("liquidityTournamentEndBlock"));
                            }
                            liquidity_tournament_end_block__ = 
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
                    liquidity_tournament_incentive_per_block: liquidity_tournament_incentive_per_block__.unwrap_or_default(),
                    liquidity_tournament_end_block: liquidity_tournament_end_block__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.distributions.v1.DistributionsParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventLqtPoolSizeIncrease {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.epoch != 0 {
            len += 1;
        }
        if self.increase.is_some() {
            len += 1;
        }
        if self.new_total.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.distributions.v1.EventLqtPoolSizeIncrease", len)?;
        if self.epoch != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epoch", ToString::to_string(&self.epoch).as_str())?;
        }
        if let Some(v) = self.increase.as_ref() {
            struct_ser.serialize_field("increase", v)?;
        }
        if let Some(v) = self.new_total.as_ref() {
            struct_ser.serialize_field("newTotal", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventLqtPoolSizeIncrease {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch",
            "increase",
            "new_total",
            "newTotal",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Epoch,
            Increase,
            NewTotal,
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
                            "epoch" => Ok(GeneratedField::Epoch),
                            "increase" => Ok(GeneratedField::Increase),
                            "newTotal" | "new_total" => Ok(GeneratedField::NewTotal),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventLqtPoolSizeIncrease;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.distributions.v1.EventLqtPoolSizeIncrease")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventLqtPoolSizeIncrease, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch__ = None;
                let mut increase__ = None;
                let mut new_total__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Epoch => {
                            if epoch__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epoch"));
                            }
                            epoch__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Increase => {
                            if increase__.is_some() {
                                return Err(serde::de::Error::duplicate_field("increase"));
                            }
                            increase__ = map_.next_value()?;
                        }
                        GeneratedField::NewTotal => {
                            if new_total__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newTotal"));
                            }
                            new_total__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventLqtPoolSizeIncrease {
                    epoch: epoch__.unwrap_or_default(),
                    increase: increase__,
                    new_total: new_total__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.distributions.v1.EventLqtPoolSizeIncrease", FIELDS, GeneratedVisitor)
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
impl serde::Serialize for LqtPoolSizeByEpochRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.epoch != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.distributions.v1.LqtPoolSizeByEpochRequest", len)?;
        if self.epoch != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epoch", ToString::to_string(&self.epoch).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LqtPoolSizeByEpochRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Epoch,
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
                            "epoch" => Ok(GeneratedField::Epoch),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LqtPoolSizeByEpochRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.distributions.v1.LqtPoolSizeByEpochRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LqtPoolSizeByEpochRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Epoch => {
                            if epoch__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epoch"));
                            }
                            epoch__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LqtPoolSizeByEpochRequest {
                    epoch: epoch__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.distributions.v1.LqtPoolSizeByEpochRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LqtPoolSizeByEpochResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.pool_size.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.distributions.v1.LqtPoolSizeByEpochResponse", len)?;
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.pool_size.as_ref() {
            struct_ser.serialize_field("poolSize", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LqtPoolSizeByEpochResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch_index",
            "epochIndex",
            "pool_size",
            "poolSize",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EpochIndex,
            PoolSize,
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
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "poolSize" | "pool_size" => Ok(GeneratedField::PoolSize),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LqtPoolSizeByEpochResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.distributions.v1.LqtPoolSizeByEpochResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LqtPoolSizeByEpochResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch_index__ = None;
                let mut pool_size__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PoolSize => {
                            if pool_size__.is_some() {
                                return Err(serde::de::Error::duplicate_field("poolSize"));
                            }
                            pool_size__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LqtPoolSizeByEpochResponse {
                    epoch_index: epoch_index__.unwrap_or_default(),
                    pool_size: pool_size__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.distributions.v1.LqtPoolSizeByEpochResponse", FIELDS, GeneratedVisitor)
    }
}
