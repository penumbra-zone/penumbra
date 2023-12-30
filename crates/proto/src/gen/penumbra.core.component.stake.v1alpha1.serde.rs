impl serde::Serialize for BaseRateData {
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
        if self.base_reward_rate != 0 {
            len += 1;
        }
        if self.base_exchange_rate != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.BaseRateData", len)?;
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if self.base_reward_rate != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("baseRewardRate", ToString::to_string(&self.base_reward_rate).as_str())?;
        }
        if self.base_exchange_rate != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("baseExchangeRate", ToString::to_string(&self.base_exchange_rate).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BaseRateData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch_index",
            "epochIndex",
            "base_reward_rate",
            "baseRewardRate",
            "base_exchange_rate",
            "baseExchangeRate",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EpochIndex,
            BaseRewardRate,
            BaseExchangeRate,
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
                            "baseRewardRate" | "base_reward_rate" => Ok(GeneratedField::BaseRewardRate),
                            "baseExchangeRate" | "base_exchange_rate" => Ok(GeneratedField::BaseExchangeRate),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BaseRateData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.BaseRateData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BaseRateData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch_index__ = None;
                let mut base_reward_rate__ = None;
                let mut base_exchange_rate__ = None;
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
                        GeneratedField::BaseRewardRate => {
                            if base_reward_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseRewardRate"));
                            }
                            base_reward_rate__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BaseExchangeRate => {
                            if base_exchange_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseExchangeRate"));
                            }
                            base_exchange_rate__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(BaseRateData {
                    epoch_index: epoch_index__.unwrap_or_default(),
                    base_reward_rate: base_reward_rate__.unwrap_or_default(),
                    base_exchange_rate: base_exchange_rate__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.BaseRateData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BondingState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.state != 0 {
            len += 1;
        }
        if self.unbonding_epoch != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.BondingState", len)?;
        if self.state != 0 {
            let v = bonding_state::BondingStateEnum::try_from(self.state)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.state)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        if self.unbonding_epoch != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("unbondingEpoch", ToString::to_string(&self.unbonding_epoch).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BondingState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "state",
            "unbonding_epoch",
            "unbondingEpoch",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            State,
            UnbondingEpoch,
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
                            "state" => Ok(GeneratedField::State),
                            "unbondingEpoch" | "unbonding_epoch" => Ok(GeneratedField::UnbondingEpoch),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BondingState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.BondingState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BondingState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state__ = None;
                let mut unbonding_epoch__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = Some(map_.next_value::<bonding_state::BondingStateEnum>()? as i32);
                        }
                        GeneratedField::UnbondingEpoch => {
                            if unbonding_epoch__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingEpoch"));
                            }
                            unbonding_epoch__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(BondingState {
                    state: state__.unwrap_or_default(),
                    unbonding_epoch: unbonding_epoch__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.BondingState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for bonding_state::BondingStateEnum {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "BONDING_STATE_ENUM_UNSPECIFIED",
            Self::Bonded => "BONDING_STATE_ENUM_BONDED",
            Self::Unbonding => "BONDING_STATE_ENUM_UNBONDING",
            Self::Unbonded => "BONDING_STATE_ENUM_UNBONDED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for bonding_state::BondingStateEnum {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "BONDING_STATE_ENUM_UNSPECIFIED",
            "BONDING_STATE_ENUM_BONDED",
            "BONDING_STATE_ENUM_UNBONDING",
            "BONDING_STATE_ENUM_UNBONDED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = bonding_state::BondingStateEnum;

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
                    "BONDING_STATE_ENUM_UNSPECIFIED" => Ok(bonding_state::BondingStateEnum::Unspecified),
                    "BONDING_STATE_ENUM_BONDED" => Ok(bonding_state::BondingStateEnum::Bonded),
                    "BONDING_STATE_ENUM_UNBONDING" => Ok(bonding_state::BondingStateEnum::Unbonding),
                    "BONDING_STATE_ENUM_UNBONDED" => Ok(bonding_state::BondingStateEnum::Unbonded),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for CurrentConsensusKeys {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.consensus_keys.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.CurrentConsensusKeys", len)?;
        if !self.consensus_keys.is_empty() {
            struct_ser.serialize_field("consensusKeys", &self.consensus_keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CurrentConsensusKeys {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "consensus_keys",
            "consensusKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConsensusKeys,
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
                            "consensusKeys" | "consensus_keys" => Ok(GeneratedField::ConsensusKeys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CurrentConsensusKeys;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.CurrentConsensusKeys")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CurrentConsensusKeys, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut consensus_keys__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ConsensusKeys => {
                            if consensus_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusKeys"));
                            }
                            consensus_keys__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CurrentConsensusKeys {
                    consensus_keys: consensus_keys__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.CurrentConsensusKeys", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CurrentValidatorRateRequest {
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
        if self.identity_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.CurrentValidatorRateRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CurrentValidatorRateRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "identity_key",
            "identityKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            IdentityKey,
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
                            "identityKey" | "identity_key" => Ok(GeneratedField::IdentityKey),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CurrentValidatorRateRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.CurrentValidatorRateRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CurrentValidatorRateRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut identity_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CurrentValidatorRateRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    identity_key: identity_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.CurrentValidatorRateRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CurrentValidatorRateResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.CurrentValidatorRateResponse", len)?;
        if let Some(v) = self.data.as_ref() {
            struct_ser.serialize_field("data", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CurrentValidatorRateResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Data,
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
                            "data" => Ok(GeneratedField::Data),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CurrentValidatorRateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.CurrentValidatorRateResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CurrentValidatorRateResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CurrentValidatorRateResponse {
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.CurrentValidatorRateResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Delegate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator_identity.is_some() {
            len += 1;
        }
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.unbonded_amount.is_some() {
            len += 1;
        }
        if self.delegation_amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.Delegate", len)?;
        if let Some(v) = self.validator_identity.as_ref() {
            struct_ser.serialize_field("validatorIdentity", v)?;
        }
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.unbonded_amount.as_ref() {
            struct_ser.serialize_field("unbondedAmount", v)?;
        }
        if let Some(v) = self.delegation_amount.as_ref() {
            struct_ser.serialize_field("delegationAmount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Delegate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_identity",
            "validatorIdentity",
            "epoch_index",
            "epochIndex",
            "unbonded_amount",
            "unbondedAmount",
            "delegation_amount",
            "delegationAmount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorIdentity,
            EpochIndex,
            UnbondedAmount,
            DelegationAmount,
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
                            "validatorIdentity" | "validator_identity" => Ok(GeneratedField::ValidatorIdentity),
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "unbondedAmount" | "unbonded_amount" => Ok(GeneratedField::UnbondedAmount),
                            "delegationAmount" | "delegation_amount" => Ok(GeneratedField::DelegationAmount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Delegate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.Delegate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Delegate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_identity__ = None;
                let mut epoch_index__ = None;
                let mut unbonded_amount__ = None;
                let mut delegation_amount__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidatorIdentity => {
                            if validator_identity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorIdentity"));
                            }
                            validator_identity__ = map_.next_value()?;
                        }
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UnbondedAmount => {
                            if unbonded_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondedAmount"));
                            }
                            unbonded_amount__ = map_.next_value()?;
                        }
                        GeneratedField::DelegationAmount => {
                            if delegation_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegationAmount"));
                            }
                            delegation_amount__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Delegate {
                    validator_identity: validator_identity__,
                    epoch_index: epoch_index__.unwrap_or_default(),
                    unbonded_amount: unbonded_amount__,
                    delegation_amount: delegation_amount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.Delegate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegationChanges {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.delegations.is_empty() {
            len += 1;
        }
        if !self.undelegations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.DelegationChanges", len)?;
        if !self.delegations.is_empty() {
            struct_ser.serialize_field("delegations", &self.delegations)?;
        }
        if !self.undelegations.is_empty() {
            struct_ser.serialize_field("undelegations", &self.undelegations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegationChanges {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegations",
            "undelegations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Delegations,
            Undelegations,
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
                            "delegations" => Ok(GeneratedField::Delegations),
                            "undelegations" => Ok(GeneratedField::Undelegations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegationChanges;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.DelegationChanges")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DelegationChanges, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegations__ = None;
                let mut undelegations__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Delegations => {
                            if delegations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegations"));
                            }
                            delegations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Undelegations => {
                            if undelegations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegations"));
                            }
                            undelegations__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DelegationChanges {
                    delegations: delegations__.unwrap_or_default(),
                    undelegations: undelegations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.DelegationChanges", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FundingStream {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.recipient.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.FundingStream", len)?;
        if let Some(v) = self.recipient.as_ref() {
            match v {
                funding_stream::Recipient::ToAddress(v) => {
                    struct_ser.serialize_field("toAddress", v)?;
                }
                funding_stream::Recipient::ToCommunityPool(v) => {
                    struct_ser.serialize_field("toCommunityPool", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FundingStream {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "to_address",
            "toAddress",
            "to_community_pool",
            "toCommunityPool",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ToAddress,
            ToCommunityPool,
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
                            "toAddress" | "to_address" => Ok(GeneratedField::ToAddress),
                            "toCommunityPool" | "to_community_pool" => Ok(GeneratedField::ToCommunityPool),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FundingStream;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.FundingStream")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FundingStream, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut recipient__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ToAddress => {
                            if recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toAddress"));
                            }
                            recipient__ = map_.next_value::<::std::option::Option<_>>()?.map(funding_stream::Recipient::ToAddress)
;
                        }
                        GeneratedField::ToCommunityPool => {
                            if recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toCommunityPool"));
                            }
                            recipient__ = map_.next_value::<::std::option::Option<_>>()?.map(funding_stream::Recipient::ToCommunityPool)
;
                        }
                    }
                }
                Ok(FundingStream {
                    recipient: recipient__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.FundingStream", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for funding_stream::ToAddress {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        if self.rate_bps != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.FundingStream.ToAddress", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if self.rate_bps != 0 {
            struct_ser.serialize_field("rateBps", &self.rate_bps)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for funding_stream::ToAddress {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "rate_bps",
            "rateBps",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            RateBps,
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
                            "address" => Ok(GeneratedField::Address),
                            "rateBps" | "rate_bps" => Ok(GeneratedField::RateBps),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = funding_stream::ToAddress;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.FundingStream.ToAddress")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<funding_stream::ToAddress, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut rate_bps__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RateBps => {
                            if rate_bps__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rateBps"));
                            }
                            rate_bps__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(funding_stream::ToAddress {
                    address: address__.unwrap_or_default(),
                    rate_bps: rate_bps__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.FundingStream.ToAddress", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for funding_stream::ToCommunityPool {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.rate_bps != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.FundingStream.ToCommunityPool", len)?;
        if self.rate_bps != 0 {
            struct_ser.serialize_field("rateBps", &self.rate_bps)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for funding_stream::ToCommunityPool {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rate_bps",
            "rateBps",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RateBps,
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
                            "rateBps" | "rate_bps" => Ok(GeneratedField::RateBps),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = funding_stream::ToCommunityPool;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.FundingStream.ToCommunityPool")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<funding_stream::ToCommunityPool, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rate_bps__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RateBps => {
                            if rate_bps__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rateBps"));
                            }
                            rate_bps__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(funding_stream::ToCommunityPool {
                    rate_bps: rate_bps__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.FundingStream.ToCommunityPool", FIELDS, GeneratedVisitor)
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
        if self.stake_params.is_some() {
            len += 1;
        }
        if !self.validators.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.GenesisContent", len)?;
        if let Some(v) = self.stake_params.as_ref() {
            struct_ser.serialize_field("stakeParams", v)?;
        }
        if !self.validators.is_empty() {
            struct_ser.serialize_field("validators", &self.validators)?;
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
            "stake_params",
            "stakeParams",
            "validators",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StakeParams,
            Validators,
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
                            "stakeParams" | "stake_params" => Ok(GeneratedField::StakeParams),
                            "validators" => Ok(GeneratedField::Validators),
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
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut stake_params__ = None;
                let mut validators__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StakeParams => {
                            if stake_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakeParams"));
                            }
                            stake_params__ = map_.next_value()?;
                        }
                        GeneratedField::Validators => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validators"));
                            }
                            validators__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(GenesisContent {
                    stake_params: stake_params__,
                    validators: validators__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Penalty {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.inner.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.Penalty", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Penalty {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "inner",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Inner,
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
                            "inner" => Ok(GeneratedField::Inner),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Penalty;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.Penalty")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Penalty, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Penalty {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.Penalty", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RateData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.identity_key.is_some() {
            len += 1;
        }
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.validator_reward_rate != 0 {
            len += 1;
        }
        if self.validator_exchange_rate != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.RateData", len)?;
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if self.validator_reward_rate != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("validatorRewardRate", ToString::to_string(&self.validator_reward_rate).as_str())?;
        }
        if self.validator_exchange_rate != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("validatorExchangeRate", ToString::to_string(&self.validator_exchange_rate).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RateData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "identity_key",
            "identityKey",
            "epoch_index",
            "epochIndex",
            "validator_reward_rate",
            "validatorRewardRate",
            "validator_exchange_rate",
            "validatorExchangeRate",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IdentityKey,
            EpochIndex,
            ValidatorRewardRate,
            ValidatorExchangeRate,
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
                            "identityKey" | "identity_key" => Ok(GeneratedField::IdentityKey),
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "validatorRewardRate" | "validator_reward_rate" => Ok(GeneratedField::ValidatorRewardRate),
                            "validatorExchangeRate" | "validator_exchange_rate" => Ok(GeneratedField::ValidatorExchangeRate),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RateData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.RateData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RateData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut identity_key__ = None;
                let mut epoch_index__ = None;
                let mut validator_reward_rate__ = None;
                let mut validator_exchange_rate__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValidatorRewardRate => {
                            if validator_reward_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorRewardRate"));
                            }
                            validator_reward_rate__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValidatorExchangeRate => {
                            if validator_exchange_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorExchangeRate"));
                            }
                            validator_exchange_rate__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(RateData {
                    identity_key: identity_key__,
                    epoch_index: epoch_index__.unwrap_or_default(),
                    validator_reward_rate: validator_reward_rate__.unwrap_or_default(),
                    validator_exchange_rate: validator_exchange_rate__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.RateData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StakeParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.unbonding_epochs != 0 {
            len += 1;
        }
        if self.active_validator_limit != 0 {
            len += 1;
        }
        if self.base_reward_rate != 0 {
            len += 1;
        }
        if self.slashing_penalty_misbehavior != 0 {
            len += 1;
        }
        if self.slashing_penalty_downtime != 0 {
            len += 1;
        }
        if self.signed_blocks_window_len != 0 {
            len += 1;
        }
        if self.missed_blocks_maximum != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.StakeParameters", len)?;
        if self.unbonding_epochs != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("unbondingEpochs", ToString::to_string(&self.unbonding_epochs).as_str())?;
        }
        if self.active_validator_limit != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("activeValidatorLimit", ToString::to_string(&self.active_validator_limit).as_str())?;
        }
        if self.base_reward_rate != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("baseRewardRate", ToString::to_string(&self.base_reward_rate).as_str())?;
        }
        if self.slashing_penalty_misbehavior != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("slashingPenaltyMisbehavior", ToString::to_string(&self.slashing_penalty_misbehavior).as_str())?;
        }
        if self.slashing_penalty_downtime != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("slashingPenaltyDowntime", ToString::to_string(&self.slashing_penalty_downtime).as_str())?;
        }
        if self.signed_blocks_window_len != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("signedBlocksWindowLen", ToString::to_string(&self.signed_blocks_window_len).as_str())?;
        }
        if self.missed_blocks_maximum != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("missedBlocksMaximum", ToString::to_string(&self.missed_blocks_maximum).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StakeParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "unbonding_epochs",
            "unbondingEpochs",
            "active_validator_limit",
            "activeValidatorLimit",
            "base_reward_rate",
            "baseRewardRate",
            "slashing_penalty_misbehavior",
            "slashingPenaltyMisbehavior",
            "slashing_penalty_downtime",
            "slashingPenaltyDowntime",
            "signed_blocks_window_len",
            "signedBlocksWindowLen",
            "missed_blocks_maximum",
            "missedBlocksMaximum",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UnbondingEpochs,
            ActiveValidatorLimit,
            BaseRewardRate,
            SlashingPenaltyMisbehavior,
            SlashingPenaltyDowntime,
            SignedBlocksWindowLen,
            MissedBlocksMaximum,
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
                            "unbondingEpochs" | "unbonding_epochs" => Ok(GeneratedField::UnbondingEpochs),
                            "activeValidatorLimit" | "active_validator_limit" => Ok(GeneratedField::ActiveValidatorLimit),
                            "baseRewardRate" | "base_reward_rate" => Ok(GeneratedField::BaseRewardRate),
                            "slashingPenaltyMisbehavior" | "slashing_penalty_misbehavior" => Ok(GeneratedField::SlashingPenaltyMisbehavior),
                            "slashingPenaltyDowntime" | "slashing_penalty_downtime" => Ok(GeneratedField::SlashingPenaltyDowntime),
                            "signedBlocksWindowLen" | "signed_blocks_window_len" => Ok(GeneratedField::SignedBlocksWindowLen),
                            "missedBlocksMaximum" | "missed_blocks_maximum" => Ok(GeneratedField::MissedBlocksMaximum),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StakeParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.StakeParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<StakeParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut unbonding_epochs__ = None;
                let mut active_validator_limit__ = None;
                let mut base_reward_rate__ = None;
                let mut slashing_penalty_misbehavior__ = None;
                let mut slashing_penalty_downtime__ = None;
                let mut signed_blocks_window_len__ = None;
                let mut missed_blocks_maximum__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UnbondingEpochs => {
                            if unbonding_epochs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingEpochs"));
                            }
                            unbonding_epochs__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ActiveValidatorLimit => {
                            if active_validator_limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("activeValidatorLimit"));
                            }
                            active_validator_limit__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BaseRewardRate => {
                            if base_reward_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseRewardRate"));
                            }
                            base_reward_rate__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SlashingPenaltyMisbehavior => {
                            if slashing_penalty_misbehavior__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slashingPenaltyMisbehavior"));
                            }
                            slashing_penalty_misbehavior__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SlashingPenaltyDowntime => {
                            if slashing_penalty_downtime__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slashingPenaltyDowntime"));
                            }
                            slashing_penalty_downtime__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SignedBlocksWindowLen => {
                            if signed_blocks_window_len__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signedBlocksWindowLen"));
                            }
                            signed_blocks_window_len__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MissedBlocksMaximum => {
                            if missed_blocks_maximum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("missedBlocksMaximum"));
                            }
                            missed_blocks_maximum__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(StakeParameters {
                    unbonding_epochs: unbonding_epochs__.unwrap_or_default(),
                    active_validator_limit: active_validator_limit__.unwrap_or_default(),
                    base_reward_rate: base_reward_rate__.unwrap_or_default(),
                    slashing_penalty_misbehavior: slashing_penalty_misbehavior__.unwrap_or_default(),
                    slashing_penalty_downtime: slashing_penalty_downtime__.unwrap_or_default(),
                    signed_blocks_window_len: signed_blocks_window_len__.unwrap_or_default(),
                    missed_blocks_maximum: missed_blocks_maximum__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.StakeParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Undelegate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator_identity.is_some() {
            len += 1;
        }
        if self.start_epoch_index != 0 {
            len += 1;
        }
        if self.unbonded_amount.is_some() {
            len += 1;
        }
        if self.delegation_amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.Undelegate", len)?;
        if let Some(v) = self.validator_identity.as_ref() {
            struct_ser.serialize_field("validatorIdentity", v)?;
        }
        if self.start_epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startEpochIndex", ToString::to_string(&self.start_epoch_index).as_str())?;
        }
        if let Some(v) = self.unbonded_amount.as_ref() {
            struct_ser.serialize_field("unbondedAmount", v)?;
        }
        if let Some(v) = self.delegation_amount.as_ref() {
            struct_ser.serialize_field("delegationAmount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Undelegate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_identity",
            "validatorIdentity",
            "start_epoch_index",
            "startEpochIndex",
            "unbonded_amount",
            "unbondedAmount",
            "delegation_amount",
            "delegationAmount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorIdentity,
            StartEpochIndex,
            UnbondedAmount,
            DelegationAmount,
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
                            "validatorIdentity" | "validator_identity" => Ok(GeneratedField::ValidatorIdentity),
                            "startEpochIndex" | "start_epoch_index" => Ok(GeneratedField::StartEpochIndex),
                            "unbondedAmount" | "unbonded_amount" => Ok(GeneratedField::UnbondedAmount),
                            "delegationAmount" | "delegation_amount" => Ok(GeneratedField::DelegationAmount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Undelegate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.Undelegate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Undelegate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_identity__ = None;
                let mut start_epoch_index__ = None;
                let mut unbonded_amount__ = None;
                let mut delegation_amount__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidatorIdentity => {
                            if validator_identity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorIdentity"));
                            }
                            validator_identity__ = map_.next_value()?;
                        }
                        GeneratedField::StartEpochIndex => {
                            if start_epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startEpochIndex"));
                            }
                            start_epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UnbondedAmount => {
                            if unbonded_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondedAmount"));
                            }
                            unbonded_amount__ = map_.next_value()?;
                        }
                        GeneratedField::DelegationAmount => {
                            if delegation_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegationAmount"));
                            }
                            delegation_amount__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Undelegate {
                    validator_identity: validator_identity__,
                    start_epoch_index: start_epoch_index__.unwrap_or_default(),
                    unbonded_amount: unbonded_amount__,
                    delegation_amount: delegation_amount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.Undelegate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UndelegateClaim {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.body.is_some() {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.UndelegateClaim", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if !self.proof.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UndelegateClaim {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body",
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Body,
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
                            "body" => Ok(GeneratedField::Body),
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
            type Value = UndelegateClaim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.UndelegateClaim")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UndelegateClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut proof__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map_.next_value()?;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(UndelegateClaim {
                    body: body__,
                    proof: proof__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.UndelegateClaim", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UndelegateClaimBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator_identity.is_some() {
            len += 1;
        }
        if self.start_epoch_index != 0 {
            len += 1;
        }
        if self.penalty.is_some() {
            len += 1;
        }
        if self.balance_commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.UndelegateClaimBody", len)?;
        if let Some(v) = self.validator_identity.as_ref() {
            struct_ser.serialize_field("validatorIdentity", v)?;
        }
        if self.start_epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startEpochIndex", ToString::to_string(&self.start_epoch_index).as_str())?;
        }
        if let Some(v) = self.penalty.as_ref() {
            struct_ser.serialize_field("penalty", v)?;
        }
        if let Some(v) = self.balance_commitment.as_ref() {
            struct_ser.serialize_field("balanceCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UndelegateClaimBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_identity",
            "validatorIdentity",
            "start_epoch_index",
            "startEpochIndex",
            "penalty",
            "balance_commitment",
            "balanceCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorIdentity,
            StartEpochIndex,
            Penalty,
            BalanceCommitment,
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
                            "validatorIdentity" | "validator_identity" => Ok(GeneratedField::ValidatorIdentity),
                            "startEpochIndex" | "start_epoch_index" => Ok(GeneratedField::StartEpochIndex),
                            "penalty" => Ok(GeneratedField::Penalty),
                            "balanceCommitment" | "balance_commitment" => Ok(GeneratedField::BalanceCommitment),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UndelegateClaimBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.UndelegateClaimBody")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UndelegateClaimBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_identity__ = None;
                let mut start_epoch_index__ = None;
                let mut penalty__ = None;
                let mut balance_commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidatorIdentity => {
                            if validator_identity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorIdentity"));
                            }
                            validator_identity__ = map_.next_value()?;
                        }
                        GeneratedField::StartEpochIndex => {
                            if start_epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startEpochIndex"));
                            }
                            start_epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Penalty => {
                            if penalty__.is_some() {
                                return Err(serde::de::Error::duplicate_field("penalty"));
                            }
                            penalty__ = map_.next_value()?;
                        }
                        GeneratedField::BalanceCommitment => {
                            if balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceCommitment"));
                            }
                            balance_commitment__ = map_.next_value()?;
                        }
                    }
                }
                Ok(UndelegateClaimBody {
                    validator_identity: validator_identity__,
                    start_epoch_index: start_epoch_index__.unwrap_or_default(),
                    penalty: penalty__,
                    balance_commitment: balance_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.UndelegateClaimBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UndelegateClaimPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator_identity.is_some() {
            len += 1;
        }
        if self.start_epoch_index != 0 {
            len += 1;
        }
        if self.penalty.is_some() {
            len += 1;
        }
        if self.unbonding_amount.is_some() {
            len += 1;
        }
        if !self.balance_blinding.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_r.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_s.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.UndelegateClaimPlan", len)?;
        if let Some(v) = self.validator_identity.as_ref() {
            struct_ser.serialize_field("validatorIdentity", v)?;
        }
        if self.start_epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startEpochIndex", ToString::to_string(&self.start_epoch_index).as_str())?;
        }
        if let Some(v) = self.penalty.as_ref() {
            struct_ser.serialize_field("penalty", v)?;
        }
        if let Some(v) = self.unbonding_amount.as_ref() {
            struct_ser.serialize_field("unbondingAmount", v)?;
        }
        if !self.balance_blinding.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("balanceBlinding", pbjson::private::base64::encode(&self.balance_blinding).as_str())?;
        }
        if !self.proof_blinding_r.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proofBlindingR", pbjson::private::base64::encode(&self.proof_blinding_r).as_str())?;
        }
        if !self.proof_blinding_s.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proofBlindingS", pbjson::private::base64::encode(&self.proof_blinding_s).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UndelegateClaimPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_identity",
            "validatorIdentity",
            "start_epoch_index",
            "startEpochIndex",
            "penalty",
            "unbonding_amount",
            "unbondingAmount",
            "balance_blinding",
            "balanceBlinding",
            "proof_blinding_r",
            "proofBlindingR",
            "proof_blinding_s",
            "proofBlindingS",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorIdentity,
            StartEpochIndex,
            Penalty,
            UnbondingAmount,
            BalanceBlinding,
            ProofBlindingR,
            ProofBlindingS,
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
                            "validatorIdentity" | "validator_identity" => Ok(GeneratedField::ValidatorIdentity),
                            "startEpochIndex" | "start_epoch_index" => Ok(GeneratedField::StartEpochIndex),
                            "penalty" => Ok(GeneratedField::Penalty),
                            "unbondingAmount" | "unbonding_amount" => Ok(GeneratedField::UnbondingAmount),
                            "balanceBlinding" | "balance_blinding" => Ok(GeneratedField::BalanceBlinding),
                            "proofBlindingR" | "proof_blinding_r" => Ok(GeneratedField::ProofBlindingR),
                            "proofBlindingS" | "proof_blinding_s" => Ok(GeneratedField::ProofBlindingS),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UndelegateClaimPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.UndelegateClaimPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UndelegateClaimPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_identity__ = None;
                let mut start_epoch_index__ = None;
                let mut penalty__ = None;
                let mut unbonding_amount__ = None;
                let mut balance_blinding__ = None;
                let mut proof_blinding_r__ = None;
                let mut proof_blinding_s__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidatorIdentity => {
                            if validator_identity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorIdentity"));
                            }
                            validator_identity__ = map_.next_value()?;
                        }
                        GeneratedField::StartEpochIndex => {
                            if start_epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startEpochIndex"));
                            }
                            start_epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Penalty => {
                            if penalty__.is_some() {
                                return Err(serde::de::Error::duplicate_field("penalty"));
                            }
                            penalty__ = map_.next_value()?;
                        }
                        GeneratedField::UnbondingAmount => {
                            if unbonding_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingAmount"));
                            }
                            unbonding_amount__ = map_.next_value()?;
                        }
                        GeneratedField::BalanceBlinding => {
                            if balance_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceBlinding"));
                            }
                            balance_blinding__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofBlindingR => {
                            if proof_blinding_r__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofBlindingR"));
                            }
                            proof_blinding_r__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofBlindingS => {
                            if proof_blinding_s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofBlindingS"));
                            }
                            proof_blinding_s__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(UndelegateClaimPlan {
                    validator_identity: validator_identity__,
                    start_epoch_index: start_epoch_index__.unwrap_or_default(),
                    penalty: penalty__,
                    unbonding_amount: unbonding_amount__,
                    balance_blinding: balance_blinding__.unwrap_or_default(),
                    proof_blinding_r: proof_blinding_r__.unwrap_or_default(),
                    proof_blinding_s: proof_blinding_s__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.UndelegateClaimPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Uptime {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.as_of_block_height != 0 {
            len += 1;
        }
        if self.window_len != 0 {
            len += 1;
        }
        if !self.bitvec.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.Uptime", len)?;
        if self.as_of_block_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("asOfBlockHeight", ToString::to_string(&self.as_of_block_height).as_str())?;
        }
        if self.window_len != 0 {
            struct_ser.serialize_field("windowLen", &self.window_len)?;
        }
        if !self.bitvec.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("bitvec", pbjson::private::base64::encode(&self.bitvec).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Uptime {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "as_of_block_height",
            "asOfBlockHeight",
            "window_len",
            "windowLen",
            "bitvec",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AsOfBlockHeight,
            WindowLen,
            Bitvec,
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
                            "asOfBlockHeight" | "as_of_block_height" => Ok(GeneratedField::AsOfBlockHeight),
                            "windowLen" | "window_len" => Ok(GeneratedField::WindowLen),
                            "bitvec" => Ok(GeneratedField::Bitvec),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Uptime;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.Uptime")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Uptime, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut as_of_block_height__ = None;
                let mut window_len__ = None;
                let mut bitvec__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AsOfBlockHeight => {
                            if as_of_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asOfBlockHeight"));
                            }
                            as_of_block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::WindowLen => {
                            if window_len__.is_some() {
                                return Err(serde::de::Error::duplicate_field("windowLen"));
                            }
                            window_len__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Bitvec => {
                            if bitvec__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bitvec"));
                            }
                            bitvec__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Uptime {
                    as_of_block_height: as_of_block_height__.unwrap_or_default(),
                    window_len: window_len__.unwrap_or_default(),
                    bitvec: bitvec__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.Uptime", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Validator {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.identity_key.is_some() {
            len += 1;
        }
        if !self.consensus_key.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.website.is_empty() {
            len += 1;
        }
        if !self.description.is_empty() {
            len += 1;
        }
        if self.enabled {
            len += 1;
        }
        if !self.funding_streams.is_empty() {
            len += 1;
        }
        if self.sequence_number != 0 {
            len += 1;
        }
        if self.governance_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.Validator", len)?;
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        if !self.consensus_key.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("consensusKey", pbjson::private::base64::encode(&self.consensus_key).as_str())?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.website.is_empty() {
            struct_ser.serialize_field("website", &self.website)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if self.enabled {
            struct_ser.serialize_field("enabled", &self.enabled)?;
        }
        if !self.funding_streams.is_empty() {
            struct_ser.serialize_field("fundingStreams", &self.funding_streams)?;
        }
        if self.sequence_number != 0 {
            struct_ser.serialize_field("sequenceNumber", &self.sequence_number)?;
        }
        if let Some(v) = self.governance_key.as_ref() {
            struct_ser.serialize_field("governanceKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Validator {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "identity_key",
            "identityKey",
            "consensus_key",
            "consensusKey",
            "name",
            "website",
            "description",
            "enabled",
            "funding_streams",
            "fundingStreams",
            "sequence_number",
            "sequenceNumber",
            "governance_key",
            "governanceKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IdentityKey,
            ConsensusKey,
            Name,
            Website,
            Description,
            Enabled,
            FundingStreams,
            SequenceNumber,
            GovernanceKey,
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
                            "identityKey" | "identity_key" => Ok(GeneratedField::IdentityKey),
                            "consensusKey" | "consensus_key" => Ok(GeneratedField::ConsensusKey),
                            "name" => Ok(GeneratedField::Name),
                            "website" => Ok(GeneratedField::Website),
                            "description" => Ok(GeneratedField::Description),
                            "enabled" => Ok(GeneratedField::Enabled),
                            "fundingStreams" | "funding_streams" => Ok(GeneratedField::FundingStreams),
                            "sequenceNumber" | "sequence_number" => Ok(GeneratedField::SequenceNumber),
                            "governanceKey" | "governance_key" => Ok(GeneratedField::GovernanceKey),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Validator;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.Validator")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Validator, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut identity_key__ = None;
                let mut consensus_key__ = None;
                let mut name__ = None;
                let mut website__ = None;
                let mut description__ = None;
                let mut enabled__ = None;
                let mut funding_streams__ = None;
                let mut sequence_number__ = None;
                let mut governance_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                        GeneratedField::ConsensusKey => {
                            if consensus_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusKey"));
                            }
                            consensus_key__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Website => {
                            if website__.is_some() {
                                return Err(serde::de::Error::duplicate_field("website"));
                            }
                            website__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FundingStreams => {
                            if funding_streams__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fundingStreams"));
                            }
                            funding_streams__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SequenceNumber => {
                            if sequence_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sequenceNumber"));
                            }
                            sequence_number__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::GovernanceKey => {
                            if governance_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("governanceKey"));
                            }
                            governance_key__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Validator {
                    identity_key: identity_key__,
                    consensus_key: consensus_key__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    website: website__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    enabled: enabled__.unwrap_or_default(),
                    funding_streams: funding_streams__.unwrap_or_default(),
                    sequence_number: sequence_number__.unwrap_or_default(),
                    governance_key: governance_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.Validator", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorDefinition {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator.is_some() {
            len += 1;
        }
        if !self.auth_sig.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorDefinition", len)?;
        if let Some(v) = self.validator.as_ref() {
            struct_ser.serialize_field("validator", v)?;
        }
        if !self.auth_sig.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("authSig", pbjson::private::base64::encode(&self.auth_sig).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorDefinition {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator",
            "auth_sig",
            "authSig",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Validator,
            AuthSig,
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
                            "validator" => Ok(GeneratedField::Validator),
                            "authSig" | "auth_sig" => Ok(GeneratedField::AuthSig),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorDefinition;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorDefinition")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorDefinition, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator__ = None;
                let mut auth_sig__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Validator => {
                            if validator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validator"));
                            }
                            validator__ = map_.next_value()?;
                        }
                        GeneratedField::AuthSig => {
                            if auth_sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authSig"));
                            }
                            auth_sig__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ValidatorDefinition {
                    validator: validator__,
                    auth_sig: auth_sig__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorDefinition", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator.is_some() {
            len += 1;
        }
        if self.status.is_some() {
            len += 1;
        }
        if self.rate_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorInfo", len)?;
        if let Some(v) = self.validator.as_ref() {
            struct_ser.serialize_field("validator", v)?;
        }
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        if let Some(v) = self.rate_data.as_ref() {
            struct_ser.serialize_field("rateData", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator",
            "status",
            "rate_data",
            "rateData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Validator,
            Status,
            RateData,
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
                            "validator" => Ok(GeneratedField::Validator),
                            "status" => Ok(GeneratedField::Status),
                            "rateData" | "rate_data" => Ok(GeneratedField::RateData),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator__ = None;
                let mut status__ = None;
                let mut rate_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Validator => {
                            if validator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validator"));
                            }
                            validator__ = map_.next_value()?;
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = map_.next_value()?;
                        }
                        GeneratedField::RateData => {
                            if rate_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rateData"));
                            }
                            rate_data__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ValidatorInfo {
                    validator: validator__,
                    status: status__,
                    rate_data: rate_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorInfoRequest {
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
        if self.show_inactive {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorInfoRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.show_inactive {
            struct_ser.serialize_field("showInactive", &self.show_inactive)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorInfoRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "show_inactive",
            "showInactive",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            ShowInactive,
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
                            "showInactive" | "show_inactive" => Ok(GeneratedField::ShowInactive),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorInfoRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorInfoRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorInfoRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut show_inactive__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ShowInactive => {
                            if show_inactive__.is_some() {
                                return Err(serde::de::Error::duplicate_field("showInactive"));
                            }
                            show_inactive__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ValidatorInfoRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    show_inactive: show_inactive__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorInfoRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorInfoResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorInfoResponse", len)?;
        if let Some(v) = self.validator_info.as_ref() {
            struct_ser.serialize_field("validatorInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorInfoResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_info",
            "validatorInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorInfo,
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
                            "validatorInfo" | "validator_info" => Ok(GeneratedField::ValidatorInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorInfoResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorInfoResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorInfoResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_info__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidatorInfo => {
                            if validator_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorInfo"));
                            }
                            validator_info__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ValidatorInfoResponse {
                    validator_info: validator_info__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorInfoResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorList {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.validator_keys.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorList", len)?;
        if !self.validator_keys.is_empty() {
            struct_ser.serialize_field("validatorKeys", &self.validator_keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorList {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_keys",
            "validatorKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorKeys,
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
                            "validatorKeys" | "validator_keys" => Ok(GeneratedField::ValidatorKeys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorList;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorList")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorList, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_keys__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidatorKeys => {
                            if validator_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorKeys"));
                            }
                            validator_keys__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ValidatorList {
                    validator_keys: validator_keys__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorList", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorPenaltyRequest {
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
        if self.identity_key.is_some() {
            len += 1;
        }
        if self.start_epoch_index != 0 {
            len += 1;
        }
        if self.end_epoch_index != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorPenaltyRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        if self.start_epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startEpochIndex", ToString::to_string(&self.start_epoch_index).as_str())?;
        }
        if self.end_epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("endEpochIndex", ToString::to_string(&self.end_epoch_index).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorPenaltyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "identity_key",
            "identityKey",
            "start_epoch_index",
            "startEpochIndex",
            "end_epoch_index",
            "endEpochIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            IdentityKey,
            StartEpochIndex,
            EndEpochIndex,
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
                            "identityKey" | "identity_key" => Ok(GeneratedField::IdentityKey),
                            "startEpochIndex" | "start_epoch_index" => Ok(GeneratedField::StartEpochIndex),
                            "endEpochIndex" | "end_epoch_index" => Ok(GeneratedField::EndEpochIndex),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorPenaltyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorPenaltyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorPenaltyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut identity_key__ = None;
                let mut start_epoch_index__ = None;
                let mut end_epoch_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                        GeneratedField::StartEpochIndex => {
                            if start_epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startEpochIndex"));
                            }
                            start_epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndEpochIndex => {
                            if end_epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endEpochIndex"));
                            }
                            end_epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ValidatorPenaltyRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    identity_key: identity_key__,
                    start_epoch_index: start_epoch_index__.unwrap_or_default(),
                    end_epoch_index: end_epoch_index__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorPenaltyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorPenaltyResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.penalty.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorPenaltyResponse", len)?;
        if let Some(v) = self.penalty.as_ref() {
            struct_ser.serialize_field("penalty", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorPenaltyResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "penalty",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Penalty,
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
                            "penalty" => Ok(GeneratedField::Penalty),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorPenaltyResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorPenaltyResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorPenaltyResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut penalty__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Penalty => {
                            if penalty__.is_some() {
                                return Err(serde::de::Error::duplicate_field("penalty"));
                            }
                            penalty__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ValidatorPenaltyResponse {
                    penalty: penalty__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorPenaltyResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.state != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorState", len)?;
        if self.state != 0 {
            let v = validator_state::ValidatorStateEnum::try_from(self.state)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.state)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "state",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            State,
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
                            "state" => Ok(GeneratedField::State),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = Some(map_.next_value::<validator_state::ValidatorStateEnum>()? as i32);
                        }
                    }
                }
                Ok(ValidatorState {
                    state: state__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for validator_state::ValidatorStateEnum {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "VALIDATOR_STATE_ENUM_UNSPECIFIED",
            Self::Inactive => "VALIDATOR_STATE_ENUM_INACTIVE",
            Self::Active => "VALIDATOR_STATE_ENUM_ACTIVE",
            Self::Jailed => "VALIDATOR_STATE_ENUM_JAILED",
            Self::Tombstoned => "VALIDATOR_STATE_ENUM_TOMBSTONED",
            Self::Disabled => "VALIDATOR_STATE_ENUM_DISABLED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for validator_state::ValidatorStateEnum {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "VALIDATOR_STATE_ENUM_UNSPECIFIED",
            "VALIDATOR_STATE_ENUM_INACTIVE",
            "VALIDATOR_STATE_ENUM_ACTIVE",
            "VALIDATOR_STATE_ENUM_JAILED",
            "VALIDATOR_STATE_ENUM_TOMBSTONED",
            "VALIDATOR_STATE_ENUM_DISABLED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = validator_state::ValidatorStateEnum;

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
                    "VALIDATOR_STATE_ENUM_UNSPECIFIED" => Ok(validator_state::ValidatorStateEnum::Unspecified),
                    "VALIDATOR_STATE_ENUM_INACTIVE" => Ok(validator_state::ValidatorStateEnum::Inactive),
                    "VALIDATOR_STATE_ENUM_ACTIVE" => Ok(validator_state::ValidatorStateEnum::Active),
                    "VALIDATOR_STATE_ENUM_JAILED" => Ok(validator_state::ValidatorStateEnum::Jailed),
                    "VALIDATOR_STATE_ENUM_TOMBSTONED" => Ok(validator_state::ValidatorStateEnum::Tombstoned),
                    "VALIDATOR_STATE_ENUM_DISABLED" => Ok(validator_state::ValidatorStateEnum::Disabled),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.identity_key.is_some() {
            len += 1;
        }
        if self.state.is_some() {
            len += 1;
        }
        if self.voting_power != 0 {
            len += 1;
        }
        if self.bonding_state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorStatus", len)?;
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        if let Some(v) = self.state.as_ref() {
            struct_ser.serialize_field("state", v)?;
        }
        if self.voting_power != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("votingPower", ToString::to_string(&self.voting_power).as_str())?;
        }
        if let Some(v) = self.bonding_state.as_ref() {
            struct_ser.serialize_field("bondingState", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "identity_key",
            "identityKey",
            "state",
            "voting_power",
            "votingPower",
            "bonding_state",
            "bondingState",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IdentityKey,
            State,
            VotingPower,
            BondingState,
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
                            "identityKey" | "identity_key" => Ok(GeneratedField::IdentityKey),
                            "state" => Ok(GeneratedField::State),
                            "votingPower" | "voting_power" => Ok(GeneratedField::VotingPower),
                            "bondingState" | "bonding_state" => Ok(GeneratedField::BondingState),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorStatus")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorStatus, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut identity_key__ = None;
                let mut state__ = None;
                let mut voting_power__ = None;
                let mut bonding_state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map_.next_value()?;
                        }
                        GeneratedField::VotingPower => {
                            if voting_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingPower"));
                            }
                            voting_power__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BondingState => {
                            if bonding_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bondingState"));
                            }
                            bonding_state__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ValidatorStatus {
                    identity_key: identity_key__,
                    state: state__,
                    voting_power: voting_power__.unwrap_or_default(),
                    bonding_state: bonding_state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorStatus", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorStatusRequest {
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
        if self.identity_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorStatusRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorStatusRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "identity_key",
            "identityKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            IdentityKey,
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
                            "identityKey" | "identity_key" => Ok(GeneratedField::IdentityKey),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorStatusRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorStatusRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorStatusRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut identity_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ValidatorStatusRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    identity_key: identity_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorStatusRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorStatusResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.status.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorStatusResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorStatusResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
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
                            "status" => Ok(GeneratedField::Status),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorStatusResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ValidatorStatusResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorStatusResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ValidatorStatusResponse {
                    status: status__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ValidatorStatusResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZkUndelegateClaimProof {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.inner.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.stake.v1alpha1.ZKUndelegateClaimProof", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZkUndelegateClaimProof {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "inner",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Inner,
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
                            "inner" => Ok(GeneratedField::Inner),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZkUndelegateClaimProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.stake.v1alpha1.ZKUndelegateClaimProof")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZkUndelegateClaimProof, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ZkUndelegateClaimProof {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.stake.v1alpha1.ZKUndelegateClaimProof", FIELDS, GeneratedVisitor)
    }
}
