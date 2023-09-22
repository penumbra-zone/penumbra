impl serde::Serialize for ChainParameters {
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
        if self.epoch_duration != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.ChainParameters", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.epoch_duration != 0 {
            struct_ser.serialize_field("epochDuration", ToString::to_string(&self.epoch_duration).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChainParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "epoch_duration",
            "epochDuration",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            EpochDuration,
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
                            "epochDuration" | "epoch_duration" => Ok(GeneratedField::EpochDuration),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ChainParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.ChainParameters")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ChainParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut epoch_duration__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::EpochDuration => {
                            if epoch_duration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochDuration"));
                            }
                            epoch_duration__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ChainParameters {
                    chain_id: chain_id__.unwrap_or_default(),
                    epoch_duration: epoch_duration__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.ChainParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EffectHash {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.EffectHash", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EffectHash {
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
            type Value = EffectHash;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.EffectHash")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EffectHash, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(EffectHash {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.EffectHash", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Epoch {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.index != 0 {
            len += 1;
        }
        if self.start_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.Epoch", len)?;
        if self.index != 0 {
            struct_ser.serialize_field("index", ToString::to_string(&self.index).as_str())?;
        }
        if self.start_height != 0 {
            struct_ser.serialize_field("startHeight", ToString::to_string(&self.start_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Epoch {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "index",
            "start_height",
            "startHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Index,
            StartHeight,
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
                            "index" => Ok(GeneratedField::Index),
                            "startHeight" | "start_height" => Ok(GeneratedField::StartHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Epoch;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.Epoch")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Epoch, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut index__ = None;
                let mut start_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::StartHeight => {
                            if start_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startHeight"));
                            }
                            start_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Epoch {
                    index: index__.unwrap_or_default(),
                    start_height: start_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.Epoch", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EpochByHeightRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.EpochByHeightRequest", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EpochByHeightRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
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
                            "height" => Ok(GeneratedField::Height),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EpochByHeightRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.EpochByHeightRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EpochByHeightRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(EpochByHeightRequest {
                    height: height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.EpochByHeightRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EpochByHeightResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.epoch.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.EpochByHeightResponse", len)?;
        if let Some(v) = self.epoch.as_ref() {
            struct_ser.serialize_field("epoch", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EpochByHeightResponse {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EpochByHeightResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.EpochByHeightResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EpochByHeightResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Epoch => {
                            if epoch__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epoch"));
                            }
                            epoch__ = map.next_value()?;
                        }
                    }
                }
                Ok(EpochByHeightResponse {
                    epoch: epoch__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.EpochByHeightResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FmdParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.precision_bits != 0 {
            len += 1;
        }
        if self.as_of_block_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.FmdParameters", len)?;
        if self.precision_bits != 0 {
            struct_ser.serialize_field("precisionBits", &self.precision_bits)?;
        }
        if self.as_of_block_height != 0 {
            struct_ser.serialize_field("asOfBlockHeight", ToString::to_string(&self.as_of_block_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FmdParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "precision_bits",
            "precisionBits",
            "as_of_block_height",
            "asOfBlockHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrecisionBits,
            AsOfBlockHeight,
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
                            "precisionBits" | "precision_bits" => Ok(GeneratedField::PrecisionBits),
                            "asOfBlockHeight" | "as_of_block_height" => Ok(GeneratedField::AsOfBlockHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FmdParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.FmdParameters")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FmdParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut precision_bits__ = None;
                let mut as_of_block_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PrecisionBits => {
                            if precision_bits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("precisionBits"));
                            }
                            precision_bits__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AsOfBlockHeight => {
                            if as_of_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asOfBlockHeight"));
                            }
                            as_of_block_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(FmdParameters {
                    precision_bits: precision_bits__.unwrap_or_default(),
                    as_of_block_height: as_of_block_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.FmdParameters", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.GenesisAppState", len)?;
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
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.GenesisAppState")
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
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.GenesisAppState", FIELDS, GeneratedVisitor)
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
        if self.chain_params.is_some() {
            len += 1;
        }
        if !self.validators.is_empty() {
            len += 1;
        }
        if !self.allocations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.GenesisContent", len)?;
        if let Some(v) = self.chain_params.as_ref() {
            struct_ser.serialize_field("chainParams", v)?;
        }
        if !self.validators.is_empty() {
            struct_ser.serialize_field("validators", &self.validators)?;
        }
        if !self.allocations.is_empty() {
            struct_ser.serialize_field("allocations", &self.allocations)?;
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
            "chain_params",
            "chainParams",
            "validators",
            "allocations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainParams,
            Validators,
            Allocations,
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
                            "validators" => Ok(GeneratedField::Validators),
                            "allocations" => Ok(GeneratedField::Allocations),
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
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.GenesisContent")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_params__ = None;
                let mut validators__ = None;
                let mut allocations__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChainParams => {
                            if chain_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainParams"));
                            }
                            chain_params__ = map.next_value()?;
                        }
                        GeneratedField::Validators => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validators"));
                            }
                            validators__ = Some(map.next_value()?);
                        }
                        GeneratedField::Allocations => {
                            if allocations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allocations"));
                            }
                            allocations__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GenesisContent {
                    chain_params: chain_params__,
                    validators: validators__.unwrap_or_default(),
                    allocations: allocations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for genesis_content::Allocation {
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
        if !self.denom.is_empty() {
            len += 1;
        }
        if self.address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.GenesisContent.Allocation", len)?;
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if !self.denom.is_empty() {
            struct_ser.serialize_field("denom", &self.denom)?;
        }
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for genesis_content::Allocation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
            "denom",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Amount,
            Denom,
            Address,
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
                            "denom" => Ok(GeneratedField::Denom),
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = genesis_content::Allocation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.GenesisContent.Allocation")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<genesis_content::Allocation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut denom__ = None;
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map.next_value()?;
                        }
                        GeneratedField::Denom => {
                            if denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denom"));
                            }
                            denom__ = Some(map.next_value()?);
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map.next_value()?;
                        }
                    }
                }
                Ok(genesis_content::Allocation {
                    amount: amount__,
                    denom: denom__.unwrap_or_default(),
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.GenesisContent.Allocation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for KnownAssets {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.assets.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.KnownAssets", len)?;
        if !self.assets.is_empty() {
            struct_ser.serialize_field("assets", &self.assets)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for KnownAssets {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "assets",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Assets,
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
                            "assets" => Ok(GeneratedField::Assets),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = KnownAssets;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.KnownAssets")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<KnownAssets, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut assets__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Assets => {
                            if assets__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assets"));
                            }
                            assets__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(KnownAssets {
                    assets: assets__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.KnownAssets", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NoteSource {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.NoteSource", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NoteSource {
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
            type Value = NoteSource;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.NoteSource")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NoteSource, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(NoteSource {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.NoteSource", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Ratio {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.numerator != 0 {
            len += 1;
        }
        if self.denominator != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.Ratio", len)?;
        if self.numerator != 0 {
            struct_ser.serialize_field("numerator", ToString::to_string(&self.numerator).as_str())?;
        }
        if self.denominator != 0 {
            struct_ser.serialize_field("denominator", ToString::to_string(&self.denominator).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Ratio {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "numerator",
            "denominator",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Numerator,
            Denominator,
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
                            "numerator" => Ok(GeneratedField::Numerator),
                            "denominator" => Ok(GeneratedField::Denominator),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Ratio;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.Ratio")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Ratio, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut numerator__ = None;
                let mut denominator__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Numerator => {
                            if numerator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("numerator"));
                            }
                            numerator__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Denominator => {
                            if denominator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denominator"));
                            }
                            denominator__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Ratio {
                    numerator: numerator__.unwrap_or_default(),
                    denominator: denominator__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.Ratio", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_source.is_some() {
            len += 1;
        }
        if self.spend_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.chain.v1alpha1.SpendInfo", len)?;
        if let Some(v) = self.note_source.as_ref() {
            struct_ser.serialize_field("noteSource", v)?;
        }
        if self.spend_height != 0 {
            struct_ser.serialize_field("spendHeight", ToString::to_string(&self.spend_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_source",
            "noteSource",
            "spend_height",
            "spendHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteSource,
            SpendHeight,
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
                            "noteSource" | "note_source" => Ok(GeneratedField::NoteSource),
                            "spendHeight" | "spend_height" => Ok(GeneratedField::SpendHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.chain.v1alpha1.SpendInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SpendInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_source__ = None;
                let mut spend_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NoteSource => {
                            if note_source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteSource"));
                            }
                            note_source__ = map.next_value()?;
                        }
                        GeneratedField::SpendHeight => {
                            if spend_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendHeight"));
                            }
                            spend_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SpendInfo {
                    note_source: note_source__,
                    spend_height: spend_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.chain.v1alpha1.SpendInfo", FIELDS, GeneratedVisitor)
    }
}
