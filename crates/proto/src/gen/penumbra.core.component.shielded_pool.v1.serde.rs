impl serde::Serialize for AssetMetadataByIdRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.asset_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.AssetMetadataByIdRequest", len)?;
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetMetadataByIdRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_id",
            "assetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssetId,
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
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AssetMetadataByIdRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.AssetMetadataByIdRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AssetMetadataByIdRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AssetMetadataByIdRequest {
                    asset_id: asset_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.AssetMetadataByIdRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AssetMetadataByIdResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.denom_metadata.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.AssetMetadataByIdResponse", len)?;
        if let Some(v) = self.denom_metadata.as_ref() {
            struct_ser.serialize_field("denomMetadata", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetMetadataByIdResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "denom_metadata",
            "denomMetadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DenomMetadata,
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
                            "denomMetadata" | "denom_metadata" => Ok(GeneratedField::DenomMetadata),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AssetMetadataByIdResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.AssetMetadataByIdResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AssetMetadataByIdResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut denom_metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DenomMetadata => {
                            if denom_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denomMetadata"));
                            }
                            denom_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AssetMetadataByIdResponse {
                    denom_metadata: denom_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.AssetMetadataByIdResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AssetMetadataByIdsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.asset_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.AssetMetadataByIdsRequest", len)?;
        if !self.asset_id.is_empty() {
            struct_ser.serialize_field("assetId", &self.asset_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetMetadataByIdsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_id",
            "assetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssetId,
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
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AssetMetadataByIdsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.AssetMetadataByIdsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AssetMetadataByIdsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AssetMetadataByIdsRequest {
                    asset_id: asset_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.AssetMetadataByIdsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AssetMetadataByIdsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.denom_metadata.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.AssetMetadataByIdsResponse", len)?;
        if let Some(v) = self.denom_metadata.as_ref() {
            struct_ser.serialize_field("denomMetadata", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetMetadataByIdsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "denom_metadata",
            "denomMetadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DenomMetadata,
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
                            "denomMetadata" | "denom_metadata" => Ok(GeneratedField::DenomMetadata),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AssetMetadataByIdsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.AssetMetadataByIdsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AssetMetadataByIdsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut denom_metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DenomMetadata => {
                            if denom_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denomMetadata"));
                            }
                            denom_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AssetMetadataByIdsResponse {
                    denom_metadata: denom_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.AssetMetadataByIdsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventBroadcastClue {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.clue.is_some() {
            len += 1;
        }
        if self.tx.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.EventBroadcastClue", len)?;
        if let Some(v) = self.clue.as_ref() {
            struct_ser.serialize_field("clue", v)?;
        }
        if let Some(v) = self.tx.as_ref() {
            struct_ser.serialize_field("tx", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventBroadcastClue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "clue",
            "tx",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Clue,
            Tx,
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
                            "clue" => Ok(GeneratedField::Clue),
                            "tx" => Ok(GeneratedField::Tx),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventBroadcastClue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.EventBroadcastClue")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventBroadcastClue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut clue__ = None;
                let mut tx__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Clue => {
                            if clue__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clue"));
                            }
                            clue__ = map_.next_value()?;
                        }
                        GeneratedField::Tx => {
                            if tx__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tx"));
                            }
                            tx__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventBroadcastClue {
                    clue: clue__,
                    tx: tx__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.EventBroadcastClue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventInboundFungibleTokenTransfer {
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
        if !self.sender.is_empty() {
            len += 1;
        }
        if self.receiver.is_some() {
            len += 1;
        }
        if self.meta.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.EventInboundFungibleTokenTransfer", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if !self.sender.is_empty() {
            struct_ser.serialize_field("sender", &self.sender)?;
        }
        if let Some(v) = self.receiver.as_ref() {
            struct_ser.serialize_field("receiver", v)?;
        }
        if let Some(v) = self.meta.as_ref() {
            struct_ser.serialize_field("meta", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventInboundFungibleTokenTransfer {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "sender",
            "receiver",
            "meta",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Sender,
            Receiver,
            Meta,
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
                            "value" => Ok(GeneratedField::Value),
                            "sender" => Ok(GeneratedField::Sender),
                            "receiver" => Ok(GeneratedField::Receiver),
                            "meta" => Ok(GeneratedField::Meta),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventInboundFungibleTokenTransfer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.EventInboundFungibleTokenTransfer")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventInboundFungibleTokenTransfer, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut sender__ = None;
                let mut receiver__ = None;
                let mut meta__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::Sender => {
                            if sender__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sender"));
                            }
                            sender__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Receiver => {
                            if receiver__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiver"));
                            }
                            receiver__ = map_.next_value()?;
                        }
                        GeneratedField::Meta => {
                            if meta__.is_some() {
                                return Err(serde::de::Error::duplicate_field("meta"));
                            }
                            meta__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventInboundFungibleTokenTransfer {
                    value: value__,
                    sender: sender__.unwrap_or_default(),
                    receiver: receiver__,
                    meta: meta__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.EventInboundFungibleTokenTransfer", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventOutboundFungibleTokenRefund {
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
        if self.sender.is_some() {
            len += 1;
        }
        if !self.receiver.is_empty() {
            len += 1;
        }
        if self.reason != 0 {
            len += 1;
        }
        if self.meta.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.EventOutboundFungibleTokenRefund", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.sender.as_ref() {
            struct_ser.serialize_field("sender", v)?;
        }
        if !self.receiver.is_empty() {
            struct_ser.serialize_field("receiver", &self.receiver)?;
        }
        if self.reason != 0 {
            let v = event_outbound_fungible_token_refund::Reason::try_from(self.reason)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.reason)))?;
            struct_ser.serialize_field("reason", &v)?;
        }
        if let Some(v) = self.meta.as_ref() {
            struct_ser.serialize_field("meta", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventOutboundFungibleTokenRefund {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "sender",
            "receiver",
            "reason",
            "meta",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Sender,
            Receiver,
            Reason,
            Meta,
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
                            "value" => Ok(GeneratedField::Value),
                            "sender" => Ok(GeneratedField::Sender),
                            "receiver" => Ok(GeneratedField::Receiver),
                            "reason" => Ok(GeneratedField::Reason),
                            "meta" => Ok(GeneratedField::Meta),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventOutboundFungibleTokenRefund;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.EventOutboundFungibleTokenRefund")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventOutboundFungibleTokenRefund, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut sender__ = None;
                let mut receiver__ = None;
                let mut reason__ = None;
                let mut meta__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::Sender => {
                            if sender__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sender"));
                            }
                            sender__ = map_.next_value()?;
                        }
                        GeneratedField::Receiver => {
                            if receiver__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiver"));
                            }
                            receiver__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map_.next_value::<event_outbound_fungible_token_refund::Reason>()? as i32);
                        }
                        GeneratedField::Meta => {
                            if meta__.is_some() {
                                return Err(serde::de::Error::duplicate_field("meta"));
                            }
                            meta__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventOutboundFungibleTokenRefund {
                    value: value__,
                    sender: sender__,
                    receiver: receiver__.unwrap_or_default(),
                    reason: reason__.unwrap_or_default(),
                    meta: meta__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.EventOutboundFungibleTokenRefund", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for event_outbound_fungible_token_refund::Reason {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "REASON_UNSPECIFIED",
            Self::Timeout => "REASON_TIMEOUT",
            Self::Error => "REASON_ERROR",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for event_outbound_fungible_token_refund::Reason {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "REASON_UNSPECIFIED",
            "REASON_TIMEOUT",
            "REASON_ERROR",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = event_outbound_fungible_token_refund::Reason;

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
                    "REASON_UNSPECIFIED" => Ok(event_outbound_fungible_token_refund::Reason::Unspecified),
                    "REASON_TIMEOUT" => Ok(event_outbound_fungible_token_refund::Reason::Timeout),
                    "REASON_ERROR" => Ok(event_outbound_fungible_token_refund::Reason::Error),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for EventOutboundFungibleTokenTransfer {
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
        if self.sender.is_some() {
            len += 1;
        }
        if !self.receiver.is_empty() {
            len += 1;
        }
        if self.meta.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.EventOutboundFungibleTokenTransfer", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.sender.as_ref() {
            struct_ser.serialize_field("sender", v)?;
        }
        if !self.receiver.is_empty() {
            struct_ser.serialize_field("receiver", &self.receiver)?;
        }
        if let Some(v) = self.meta.as_ref() {
            struct_ser.serialize_field("meta", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventOutboundFungibleTokenTransfer {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "sender",
            "receiver",
            "meta",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Sender,
            Receiver,
            Meta,
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
                            "value" => Ok(GeneratedField::Value),
                            "sender" => Ok(GeneratedField::Sender),
                            "receiver" => Ok(GeneratedField::Receiver),
                            "meta" => Ok(GeneratedField::Meta),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventOutboundFungibleTokenTransfer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.EventOutboundFungibleTokenTransfer")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventOutboundFungibleTokenTransfer, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut sender__ = None;
                let mut receiver__ = None;
                let mut meta__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::Sender => {
                            if sender__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sender"));
                            }
                            sender__ = map_.next_value()?;
                        }
                        GeneratedField::Receiver => {
                            if receiver__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiver"));
                            }
                            receiver__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Meta => {
                            if meta__.is_some() {
                                return Err(serde::de::Error::duplicate_field("meta"));
                            }
                            meta__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventOutboundFungibleTokenTransfer {
                    value: value__,
                    sender: sender__,
                    receiver: receiver__.unwrap_or_default(),
                    meta: meta__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.EventOutboundFungibleTokenTransfer", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventOutput {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.EventOutput", len)?;
        if let Some(v) = self.note_commitment.as_ref() {
            struct_ser.serialize_field("noteCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventOutput {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_commitment",
            "noteCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteCommitment,
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
                            "noteCommitment" | "note_commitment" => Ok(GeneratedField::NoteCommitment),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventOutput;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.EventOutput")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventOutput, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NoteCommitment => {
                            if note_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitment"));
                            }
                            note_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventOutput {
                    note_commitment: note_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.EventOutput", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventSpend {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.nullifier.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.EventSpend", len)?;
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventSpend {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "nullifier",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Nullifier,
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
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventSpend;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.EventSpend")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventSpend, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut nullifier__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventSpend {
                    nullifier: nullifier__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.EventSpend", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FmdMetaParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fmd_grace_period_blocks != 0 {
            len += 1;
        }
        if self.algorithm.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.FmdMetaParameters", len)?;
        if self.fmd_grace_period_blocks != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("fmdGracePeriodBlocks", ToString::to_string(&self.fmd_grace_period_blocks).as_str())?;
        }
        if let Some(v) = self.algorithm.as_ref() {
            match v {
                fmd_meta_parameters::Algorithm::FixedPrecisionBits(v) => {
                    struct_ser.serialize_field("fixedPrecisionBits", v)?;
                }
                fmd_meta_parameters::Algorithm::SlidingWindow(v) => {
                    struct_ser.serialize_field("slidingWindow", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FmdMetaParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fmd_grace_period_blocks",
            "fmdGracePeriodBlocks",
            "fixed_precision_bits",
            "fixedPrecisionBits",
            "sliding_window",
            "slidingWindow",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FmdGracePeriodBlocks,
            FixedPrecisionBits,
            SlidingWindow,
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
                            "fmdGracePeriodBlocks" | "fmd_grace_period_blocks" => Ok(GeneratedField::FmdGracePeriodBlocks),
                            "fixedPrecisionBits" | "fixed_precision_bits" => Ok(GeneratedField::FixedPrecisionBits),
                            "slidingWindow" | "sliding_window" => Ok(GeneratedField::SlidingWindow),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FmdMetaParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.FmdMetaParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FmdMetaParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fmd_grace_period_blocks__ = None;
                let mut algorithm__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FmdGracePeriodBlocks => {
                            if fmd_grace_period_blocks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fmdGracePeriodBlocks"));
                            }
                            fmd_grace_period_blocks__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::FixedPrecisionBits => {
                            if algorithm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fixedPrecisionBits"));
                            }
                            algorithm__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| fmd_meta_parameters::Algorithm::FixedPrecisionBits(x.0));
                        }
                        GeneratedField::SlidingWindow => {
                            if algorithm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slidingWindow"));
                            }
                            algorithm__ = map_.next_value::<::std::option::Option<_>>()?.map(fmd_meta_parameters::Algorithm::SlidingWindow)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FmdMetaParameters {
                    fmd_grace_period_blocks: fmd_grace_period_blocks__.unwrap_or_default(),
                    algorithm: algorithm__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.FmdMetaParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for fmd_meta_parameters::AlgorithmSlidingWindow {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.window_update_periods != 0 {
            len += 1;
        }
        if self.targeted_detections_per_window != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.FmdMetaParameters.AlgorithmSlidingWindow", len)?;
        if self.window_update_periods != 0 {
            struct_ser.serialize_field("windowUpdatePeriods", &self.window_update_periods)?;
        }
        if self.targeted_detections_per_window != 0 {
            struct_ser.serialize_field("targetedDetectionsPerWindow", &self.targeted_detections_per_window)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for fmd_meta_parameters::AlgorithmSlidingWindow {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "window_update_periods",
            "windowUpdatePeriods",
            "targeted_detections_per_window",
            "targetedDetectionsPerWindow",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WindowUpdatePeriods,
            TargetedDetectionsPerWindow,
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
                            "windowUpdatePeriods" | "window_update_periods" => Ok(GeneratedField::WindowUpdatePeriods),
                            "targetedDetectionsPerWindow" | "targeted_detections_per_window" => Ok(GeneratedField::TargetedDetectionsPerWindow),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = fmd_meta_parameters::AlgorithmSlidingWindow;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.FmdMetaParameters.AlgorithmSlidingWindow")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<fmd_meta_parameters::AlgorithmSlidingWindow, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut window_update_periods__ = None;
                let mut targeted_detections_per_window__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WindowUpdatePeriods => {
                            if window_update_periods__.is_some() {
                                return Err(serde::de::Error::duplicate_field("windowUpdatePeriods"));
                            }
                            window_update_periods__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TargetedDetectionsPerWindow => {
                            if targeted_detections_per_window__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targetedDetectionsPerWindow"));
                            }
                            targeted_detections_per_window__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(fmd_meta_parameters::AlgorithmSlidingWindow {
                    window_update_periods: window_update_periods__.unwrap_or_default(),
                    targeted_detections_per_window: targeted_detections_per_window__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.FmdMetaParameters.AlgorithmSlidingWindow", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FmdMetaParametersAlgorithmState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.FmdMetaParametersAlgorithmState", len)?;
        if let Some(v) = self.state.as_ref() {
            match v {
                fmd_meta_parameters_algorithm_state::State::Fixed(v) => {
                    struct_ser.serialize_field("fixed", v)?;
                }
                fmd_meta_parameters_algorithm_state::State::SlidingWindow(v) => {
                    struct_ser.serialize_field("slidingWindow", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FmdMetaParametersAlgorithmState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fixed",
            "sliding_window",
            "slidingWindow",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fixed,
            SlidingWindow,
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
                            "fixed" => Ok(GeneratedField::Fixed),
                            "slidingWindow" | "sliding_window" => Ok(GeneratedField::SlidingWindow),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FmdMetaParametersAlgorithmState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.FmdMetaParametersAlgorithmState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FmdMetaParametersAlgorithmState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Fixed => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fixed"));
                            }
                            state__ = map_.next_value::<::std::option::Option<_>>()?.map(fmd_meta_parameters_algorithm_state::State::Fixed)
;
                        }
                        GeneratedField::SlidingWindow => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slidingWindow"));
                            }
                            state__ = map_.next_value::<::std::option::Option<_>>()?.map(fmd_meta_parameters_algorithm_state::State::SlidingWindow)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FmdMetaParametersAlgorithmState {
                    state: state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.FmdMetaParametersAlgorithmState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for fmd_meta_parameters_algorithm_state::FixedState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.FmdMetaParametersAlgorithmState.FixedState", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for fmd_meta_parameters_algorithm_state::FixedState {
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
            type Value = fmd_meta_parameters_algorithm_state::FixedState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.FmdMetaParametersAlgorithmState.FixedState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<fmd_meta_parameters_algorithm_state::FixedState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(fmd_meta_parameters_algorithm_state::FixedState {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.FmdMetaParametersAlgorithmState.FixedState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for fmd_meta_parameters_algorithm_state::SlidingWindowState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.approximate_clue_count != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.FmdMetaParametersAlgorithmState.SlidingWindowState", len)?;
        if self.approximate_clue_count != 0 {
            struct_ser.serialize_field("approximateClueCount", &self.approximate_clue_count)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for fmd_meta_parameters_algorithm_state::SlidingWindowState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "approximate_clue_count",
            "approximateClueCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ApproximateClueCount,
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
                            "approximateClueCount" | "approximate_clue_count" => Ok(GeneratedField::ApproximateClueCount),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = fmd_meta_parameters_algorithm_state::SlidingWindowState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.FmdMetaParametersAlgorithmState.SlidingWindowState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<fmd_meta_parameters_algorithm_state::SlidingWindowState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut approximate_clue_count__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ApproximateClueCount => {
                            if approximate_clue_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("approximateClueCount"));
                            }
                            approximate_clue_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(fmd_meta_parameters_algorithm_state::SlidingWindowState {
                    approximate_clue_count: approximate_clue_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.FmdMetaParametersAlgorithmState.SlidingWindowState", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.FmdParameters", len)?;
        if self.precision_bits != 0 {
            struct_ser.serialize_field("precisionBits", &self.precision_bits)?;
        }
        if self.as_of_block_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
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
                            "precisionBits" | "precision_bits" => Ok(GeneratedField::PrecisionBits),
                            "asOfBlockHeight" | "as_of_block_height" => Ok(GeneratedField::AsOfBlockHeight),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.FmdParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FmdParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut precision_bits__ = None;
                let mut as_of_block_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrecisionBits => {
                            if precision_bits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("precisionBits"));
                            }
                            precision_bits__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AsOfBlockHeight => {
                            if as_of_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asOfBlockHeight"));
                            }
                            as_of_block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FmdParameters {
                    precision_bits: precision_bits__.unwrap_or_default(),
                    as_of_block_height: as_of_block_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.FmdParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FungibleTokenTransferPacketMetadata {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.channel.is_empty() {
            len += 1;
        }
        if self.sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.FungibleTokenTransferPacketMetadata", len)?;
        if !self.channel.is_empty() {
            struct_ser.serialize_field("channel", &self.channel)?;
        }
        if self.sequence != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FungibleTokenTransferPacketMetadata {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "channel",
            "sequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Channel,
            Sequence,
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
                            "channel" => Ok(GeneratedField::Channel),
                            "sequence" => Ok(GeneratedField::Sequence),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FungibleTokenTransferPacketMetadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.FungibleTokenTransferPacketMetadata")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FungibleTokenTransferPacketMetadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut channel__ = None;
                let mut sequence__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Channel => {
                            if channel__.is_some() {
                                return Err(serde::de::Error::duplicate_field("channel"));
                            }
                            channel__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Sequence => {
                            if sequence__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sequence"));
                            }
                            sequence__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FungibleTokenTransferPacketMetadata {
                    channel: channel__.unwrap_or_default(),
                    sequence: sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.FungibleTokenTransferPacketMetadata", FIELDS, GeneratedVisitor)
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
        if self.shielded_pool_params.is_some() {
            len += 1;
        }
        if !self.allocations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.GenesisContent", len)?;
        if let Some(v) = self.shielded_pool_params.as_ref() {
            struct_ser.serialize_field("shieldedPoolParams", v)?;
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
            "shielded_pool_params",
            "shieldedPoolParams",
            "allocations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ShieldedPoolParams,
            Allocations,
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
                            "shieldedPoolParams" | "shielded_pool_params" => Ok(GeneratedField::ShieldedPoolParams),
                            "allocations" => Ok(GeneratedField::Allocations),
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
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut shielded_pool_params__ = None;
                let mut allocations__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ShieldedPoolParams => {
                            if shielded_pool_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shieldedPoolParams"));
                            }
                            shielded_pool_params__ = map_.next_value()?;
                        }
                        GeneratedField::Allocations => {
                            if allocations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allocations"));
                            }
                            allocations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    shielded_pool_params: shielded_pool_params__,
                    allocations: allocations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.GenesisContent", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.GenesisContent.Allocation", len)?;
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
                            "amount" => Ok(GeneratedField::Amount),
                            "denom" => Ok(GeneratedField::Denom),
                            "address" => Ok(GeneratedField::Address),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.GenesisContent.Allocation")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<genesis_content::Allocation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut denom__ = None;
                let mut address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map_.next_value()?;
                        }
                        GeneratedField::Denom => {
                            if denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denom"));
                            }
                            denom__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
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
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.GenesisContent.Allocation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Note {
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
        if !self.rseed.is_empty() {
            len += 1;
        }
        if self.address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.Note", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if !self.rseed.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("rseed", pbjson::private::base64::encode(&self.rseed).as_str())?;
        }
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Note {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "rseed",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Rseed,
            Address,
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
                            "value" => Ok(GeneratedField::Value),
                            "rseed" => Ok(GeneratedField::Rseed),
                            "address" => Ok(GeneratedField::Address),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Note;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.Note")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Note, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut rseed__ = None;
                let mut address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::Rseed => {
                            if rseed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rseed"));
                            }
                            rseed__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Note {
                    value: value__,
                    rseed: rseed__.unwrap_or_default(),
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.Note", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NoteCiphertext {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.NoteCiphertext", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NoteCiphertext {
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
                            "inner" => Ok(GeneratedField::Inner),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NoteCiphertext;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.NoteCiphertext")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NoteCiphertext, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NoteCiphertext {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.NoteCiphertext", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NotePayload {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_commitment.is_some() {
            len += 1;
        }
        if !self.ephemeral_key.is_empty() {
            len += 1;
        }
        if self.encrypted_note.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.NotePayload", len)?;
        if let Some(v) = self.note_commitment.as_ref() {
            struct_ser.serialize_field("noteCommitment", v)?;
        }
        if !self.ephemeral_key.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("ephemeralKey", pbjson::private::base64::encode(&self.ephemeral_key).as_str())?;
        }
        if let Some(v) = self.encrypted_note.as_ref() {
            struct_ser.serialize_field("encryptedNote", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NotePayload {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_commitment",
            "noteCommitment",
            "ephemeral_key",
            "ephemeralKey",
            "encrypted_note",
            "encryptedNote",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteCommitment,
            EphemeralKey,
            EncryptedNote,
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
                            "noteCommitment" | "note_commitment" => Ok(GeneratedField::NoteCommitment),
                            "ephemeralKey" | "ephemeral_key" => Ok(GeneratedField::EphemeralKey),
                            "encryptedNote" | "encrypted_note" => Ok(GeneratedField::EncryptedNote),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NotePayload;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.NotePayload")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NotePayload, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_commitment__ = None;
                let mut ephemeral_key__ = None;
                let mut encrypted_note__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NoteCommitment => {
                            if note_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitment"));
                            }
                            note_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::EphemeralKey => {
                            if ephemeral_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ephemeralKey"));
                            }
                            ephemeral_key__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EncryptedNote => {
                            if encrypted_note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encryptedNote"));
                            }
                            encrypted_note__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NotePayload {
                    note_commitment: note_commitment__,
                    ephemeral_key: ephemeral_key__.unwrap_or_default(),
                    encrypted_note: encrypted_note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.NotePayload", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NoteView {
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
        if !self.rseed.is_empty() {
            len += 1;
        }
        if self.address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.NoteView", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if !self.rseed.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("rseed", pbjson::private::base64::encode(&self.rseed).as_str())?;
        }
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NoteView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "rseed",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Rseed,
            Address,
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
                            "value" => Ok(GeneratedField::Value),
                            "rseed" => Ok(GeneratedField::Rseed),
                            "address" => Ok(GeneratedField::Address),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NoteView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.NoteView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NoteView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut rseed__ = None;
                let mut address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::Rseed => {
                            if rseed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rseed"));
                            }
                            rseed__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NoteView {
                    value: value__,
                    rseed: rseed__.unwrap_or_default(),
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.NoteView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Output {
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
        if self.proof.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.Output", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.proof.as_ref() {
            struct_ser.serialize_field("proof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Output {
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
                            "body" => Ok(GeneratedField::Body),
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Output;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.Output")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Output, V::Error>
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
                            proof__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Output {
                    body: body__,
                    proof: proof__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.Output", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_payload.is_some() {
            len += 1;
        }
        if self.balance_commitment.is_some() {
            len += 1;
        }
        if !self.wrapped_memo_key.is_empty() {
            len += 1;
        }
        if !self.ovk_wrapped_key.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.OutputBody", len)?;
        if let Some(v) = self.note_payload.as_ref() {
            struct_ser.serialize_field("notePayload", v)?;
        }
        if let Some(v) = self.balance_commitment.as_ref() {
            struct_ser.serialize_field("balanceCommitment", v)?;
        }
        if !self.wrapped_memo_key.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("wrappedMemoKey", pbjson::private::base64::encode(&self.wrapped_memo_key).as_str())?;
        }
        if !self.ovk_wrapped_key.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("ovkWrappedKey", pbjson::private::base64::encode(&self.ovk_wrapped_key).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_payload",
            "notePayload",
            "balance_commitment",
            "balanceCommitment",
            "wrapped_memo_key",
            "wrappedMemoKey",
            "ovk_wrapped_key",
            "ovkWrappedKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NotePayload,
            BalanceCommitment,
            WrappedMemoKey,
            OvkWrappedKey,
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
                            "notePayload" | "note_payload" => Ok(GeneratedField::NotePayload),
                            "balanceCommitment" | "balance_commitment" => Ok(GeneratedField::BalanceCommitment),
                            "wrappedMemoKey" | "wrapped_memo_key" => Ok(GeneratedField::WrappedMemoKey),
                            "ovkWrappedKey" | "ovk_wrapped_key" => Ok(GeneratedField::OvkWrappedKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.OutputBody")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OutputBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_payload__ = None;
                let mut balance_commitment__ = None;
                let mut wrapped_memo_key__ = None;
                let mut ovk_wrapped_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NotePayload => {
                            if note_payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("notePayload"));
                            }
                            note_payload__ = map_.next_value()?;
                        }
                        GeneratedField::BalanceCommitment => {
                            if balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceCommitment"));
                            }
                            balance_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::WrappedMemoKey => {
                            if wrapped_memo_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("wrappedMemoKey"));
                            }
                            wrapped_memo_key__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::OvkWrappedKey => {
                            if ovk_wrapped_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ovkWrappedKey"));
                            }
                            ovk_wrapped_key__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OutputBody {
                    note_payload: note_payload__,
                    balance_commitment: balance_commitment__,
                    wrapped_memo_key: wrapped_memo_key__.unwrap_or_default(),
                    ovk_wrapped_key: ovk_wrapped_key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.OutputBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputPlan {
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
        if self.dest_address.is_some() {
            len += 1;
        }
        if !self.rseed.is_empty() {
            len += 1;
        }
        if !self.value_blinding.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_r.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_s.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.OutputPlan", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.dest_address.as_ref() {
            struct_ser.serialize_field("destAddress", v)?;
        }
        if !self.rseed.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("rseed", pbjson::private::base64::encode(&self.rseed).as_str())?;
        }
        if !self.value_blinding.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("valueBlinding", pbjson::private::base64::encode(&self.value_blinding).as_str())?;
        }
        if !self.proof_blinding_r.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proofBlindingR", pbjson::private::base64::encode(&self.proof_blinding_r).as_str())?;
        }
        if !self.proof_blinding_s.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proofBlindingS", pbjson::private::base64::encode(&self.proof_blinding_s).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "dest_address",
            "destAddress",
            "rseed",
            "value_blinding",
            "valueBlinding",
            "proof_blinding_r",
            "proofBlindingR",
            "proof_blinding_s",
            "proofBlindingS",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            DestAddress,
            Rseed,
            ValueBlinding,
            ProofBlindingR,
            ProofBlindingS,
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
                            "value" => Ok(GeneratedField::Value),
                            "destAddress" | "dest_address" => Ok(GeneratedField::DestAddress),
                            "rseed" => Ok(GeneratedField::Rseed),
                            "valueBlinding" | "value_blinding" => Ok(GeneratedField::ValueBlinding),
                            "proofBlindingR" | "proof_blinding_r" => Ok(GeneratedField::ProofBlindingR),
                            "proofBlindingS" | "proof_blinding_s" => Ok(GeneratedField::ProofBlindingS),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.OutputPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OutputPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut dest_address__ = None;
                let mut rseed__ = None;
                let mut value_blinding__ = None;
                let mut proof_blinding_r__ = None;
                let mut proof_blinding_s__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::DestAddress => {
                            if dest_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destAddress"));
                            }
                            dest_address__ = map_.next_value()?;
                        }
                        GeneratedField::Rseed => {
                            if rseed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rseed"));
                            }
                            rseed__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValueBlinding => {
                            if value_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valueBlinding"));
                            }
                            value_blinding__ = 
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OutputPlan {
                    value: value__,
                    dest_address: dest_address__,
                    rseed: rseed__.unwrap_or_default(),
                    value_blinding: value_blinding__.unwrap_or_default(),
                    proof_blinding_r: proof_blinding_r__.unwrap_or_default(),
                    proof_blinding_s: proof_blinding_s__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.OutputPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.output_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.OutputView", len)?;
        if let Some(v) = self.output_view.as_ref() {
            match v {
                output_view::OutputView::Visible(v) => {
                    struct_ser.serialize_field("visible", v)?;
                }
                output_view::OutputView::Opaque(v) => {
                    struct_ser.serialize_field("opaque", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "visible",
            "opaque",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Visible,
            Opaque,
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
                            "visible" => Ok(GeneratedField::Visible),
                            "opaque" => Ok(GeneratedField::Opaque),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.OutputView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OutputView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut output_view__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if output_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            output_view__ = map_.next_value::<::std::option::Option<_>>()?.map(output_view::OutputView::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if output_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            output_view__ = map_.next_value::<::std::option::Option<_>>()?.map(output_view::OutputView::Opaque)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OutputView {
                    output_view: output_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.OutputView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for output_view::Opaque {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.output.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.OutputView.Opaque", len)?;
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for output_view::Opaque {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "output",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Output,
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
                            "output" => Ok(GeneratedField::Output),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = output_view::Opaque;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.OutputView.Opaque")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<output_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut output__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(output_view::Opaque {
                    output: output__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.OutputView.Opaque", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for output_view::Visible {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.output.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        if self.payload_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.OutputView.Visible", len)?;
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if let Some(v) = self.payload_key.as_ref() {
            struct_ser.serialize_field("payloadKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for output_view::Visible {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "output",
            "note",
            "payload_key",
            "payloadKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Output,
            Note,
            PayloadKey,
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
                            "output" => Ok(GeneratedField::Output),
                            "note" => Ok(GeneratedField::Note),
                            "payloadKey" | "payload_key" => Ok(GeneratedField::PayloadKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = output_view::Visible;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.OutputView.Visible")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<output_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut output__ = None;
                let mut note__ = None;
                let mut payload_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = map_.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map_.next_value()?;
                        }
                        GeneratedField::PayloadKey => {
                            if payload_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payloadKey"));
                            }
                            payload_key__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(output_view::Visible {
                    output: output__,
                    note: note__,
                    payload_key: payload_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.OutputView.Visible", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ShieldedPoolParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fixed_fmd_params.is_some() {
            len += 1;
        }
        if self.fmd_meta_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.ShieldedPoolParameters", len)?;
        if let Some(v) = self.fixed_fmd_params.as_ref() {
            struct_ser.serialize_field("fixedFmdParams", v)?;
        }
        if let Some(v) = self.fmd_meta_params.as_ref() {
            struct_ser.serialize_field("fmdMetaParams", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ShieldedPoolParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fixed_fmd_params",
            "fixedFmdParams",
            "fmd_meta_params",
            "fmdMetaParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FixedFmdParams,
            FmdMetaParams,
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
                            "fixedFmdParams" | "fixed_fmd_params" => Ok(GeneratedField::FixedFmdParams),
                            "fmdMetaParams" | "fmd_meta_params" => Ok(GeneratedField::FmdMetaParams),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ShieldedPoolParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.ShieldedPoolParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ShieldedPoolParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fixed_fmd_params__ = None;
                let mut fmd_meta_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FixedFmdParams => {
                            if fixed_fmd_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fixedFmdParams"));
                            }
                            fixed_fmd_params__ = map_.next_value()?;
                        }
                        GeneratedField::FmdMetaParams => {
                            if fmd_meta_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fmdMetaParams"));
                            }
                            fmd_meta_params__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ShieldedPoolParameters {
                    fixed_fmd_params: fixed_fmd_params__,
                    fmd_meta_params: fmd_meta_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.ShieldedPoolParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Spend {
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
        if self.auth_sig.is_some() {
            len += 1;
        }
        if self.proof.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.Spend", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.auth_sig.as_ref() {
            struct_ser.serialize_field("authSig", v)?;
        }
        if let Some(v) = self.proof.as_ref() {
            struct_ser.serialize_field("proof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Spend {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body",
            "auth_sig",
            "authSig",
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Body,
            AuthSig,
            Proof,
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
                            "body" => Ok(GeneratedField::Body),
                            "authSig" | "auth_sig" => Ok(GeneratedField::AuthSig),
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Spend;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.Spend")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Spend, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut auth_sig__ = None;
                let mut proof__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map_.next_value()?;
                        }
                        GeneratedField::AuthSig => {
                            if auth_sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authSig"));
                            }
                            auth_sig__ = map_.next_value()?;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Spend {
                    body: body__,
                    auth_sig: auth_sig__,
                    proof: proof__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.Spend", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.balance_commitment.is_some() {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.rk.is_some() {
            len += 1;
        }
        if !self.encrypted_backref.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.SpendBody", len)?;
        if let Some(v) = self.balance_commitment.as_ref() {
            struct_ser.serialize_field("balanceCommitment", v)?;
        }
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if let Some(v) = self.rk.as_ref() {
            struct_ser.serialize_field("rk", v)?;
        }
        if !self.encrypted_backref.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("encryptedBackref", pbjson::private::base64::encode(&self.encrypted_backref).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "balance_commitment",
            "balanceCommitment",
            "nullifier",
            "rk",
            "encrypted_backref",
            "encryptedBackref",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BalanceCommitment,
            Nullifier,
            Rk,
            EncryptedBackref,
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
                            "balanceCommitment" | "balance_commitment" => Ok(GeneratedField::BalanceCommitment),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "rk" => Ok(GeneratedField::Rk),
                            "encryptedBackref" | "encrypted_backref" => Ok(GeneratedField::EncryptedBackref),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.SpendBody")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SpendBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut balance_commitment__ = None;
                let mut nullifier__ = None;
                let mut rk__ = None;
                let mut encrypted_backref__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BalanceCommitment => {
                            if balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceCommitment"));
                            }
                            balance_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map_.next_value()?;
                        }
                        GeneratedField::Rk => {
                            if rk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rk"));
                            }
                            rk__ = map_.next_value()?;
                        }
                        GeneratedField::EncryptedBackref => {
                            if encrypted_backref__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encryptedBackref"));
                            }
                            encrypted_backref__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SpendBody {
                    balance_commitment: balance_commitment__,
                    nullifier: nullifier__,
                    rk: rk__,
                    encrypted_backref: encrypted_backref__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.SpendBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note.is_some() {
            len += 1;
        }
        if self.position != 0 {
            len += 1;
        }
        if !self.randomizer.is_empty() {
            len += 1;
        }
        if !self.value_blinding.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_r.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_s.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.SpendPlan", len)?;
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if self.position != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("position", ToString::to_string(&self.position).as_str())?;
        }
        if !self.randomizer.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("randomizer", pbjson::private::base64::encode(&self.randomizer).as_str())?;
        }
        if !self.value_blinding.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("valueBlinding", pbjson::private::base64::encode(&self.value_blinding).as_str())?;
        }
        if !self.proof_blinding_r.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proofBlindingR", pbjson::private::base64::encode(&self.proof_blinding_r).as_str())?;
        }
        if !self.proof_blinding_s.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proofBlindingS", pbjson::private::base64::encode(&self.proof_blinding_s).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note",
            "position",
            "randomizer",
            "value_blinding",
            "valueBlinding",
            "proof_blinding_r",
            "proofBlindingR",
            "proof_blinding_s",
            "proofBlindingS",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Note,
            Position,
            Randomizer,
            ValueBlinding,
            ProofBlindingR,
            ProofBlindingS,
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
                            "note" => Ok(GeneratedField::Note),
                            "position" => Ok(GeneratedField::Position),
                            "randomizer" => Ok(GeneratedField::Randomizer),
                            "valueBlinding" | "value_blinding" => Ok(GeneratedField::ValueBlinding),
                            "proofBlindingR" | "proof_blinding_r" => Ok(GeneratedField::ProofBlindingR),
                            "proofBlindingS" | "proof_blinding_s" => Ok(GeneratedField::ProofBlindingS),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.SpendPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SpendPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note__ = None;
                let mut position__ = None;
                let mut randomizer__ = None;
                let mut value_blinding__ = None;
                let mut proof_blinding_r__ = None;
                let mut proof_blinding_s__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map_.next_value()?;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Randomizer => {
                            if randomizer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("randomizer"));
                            }
                            randomizer__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValueBlinding => {
                            if value_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valueBlinding"));
                            }
                            value_blinding__ = 
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SpendPlan {
                    note: note__,
                    position: position__.unwrap_or_default(),
                    randomizer: randomizer__.unwrap_or_default(),
                    value_blinding: value_blinding__.unwrap_or_default(),
                    proof_blinding_r: proof_blinding_r__.unwrap_or_default(),
                    proof_blinding_s: proof_blinding_s__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.SpendPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.spend_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.SpendView", len)?;
        if let Some(v) = self.spend_view.as_ref() {
            match v {
                spend_view::SpendView::Visible(v) => {
                    struct_ser.serialize_field("visible", v)?;
                }
                spend_view::SpendView::Opaque(v) => {
                    struct_ser.serialize_field("opaque", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "visible",
            "opaque",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Visible,
            Opaque,
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
                            "visible" => Ok(GeneratedField::Visible),
                            "opaque" => Ok(GeneratedField::Opaque),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.SpendView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SpendView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spend_view__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if spend_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            spend_view__ = map_.next_value::<::std::option::Option<_>>()?.map(spend_view::SpendView::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if spend_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            spend_view__ = map_.next_value::<::std::option::Option<_>>()?.map(spend_view::SpendView::Opaque)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SpendView {
                    spend_view: spend_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.SpendView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for spend_view::Opaque {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.spend.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.SpendView.Opaque", len)?;
        if let Some(v) = self.spend.as_ref() {
            struct_ser.serialize_field("spend", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for spend_view::Opaque {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
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
                            "spend" => Ok(GeneratedField::Spend),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = spend_view::Opaque;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.SpendView.Opaque")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<spend_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spend__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            spend__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(spend_view::Opaque {
                    spend: spend__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.SpendView.Opaque", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for spend_view::Visible {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.spend.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.SpendView.Visible", len)?;
        if let Some(v) = self.spend.as_ref() {
            struct_ser.serialize_field("spend", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for spend_view::Visible {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend",
            "note",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
            Note,
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
                            "spend" => Ok(GeneratedField::Spend),
                            "note" => Ok(GeneratedField::Note),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = spend_view::Visible;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.SpendView.Visible")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<spend_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spend__ = None;
                let mut note__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            spend__ = map_.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(spend_view::Visible {
                    spend: spend__,
                    note: note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.SpendView.Visible", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZkNullifierDerivationProof {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.ZKNullifierDerivationProof", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZkNullifierDerivationProof {
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
                            "inner" => Ok(GeneratedField::Inner),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZkNullifierDerivationProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.ZKNullifierDerivationProof")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZkNullifierDerivationProof, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ZkNullifierDerivationProof {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.ZKNullifierDerivationProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZkOutputProof {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.ZKOutputProof", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZkOutputProof {
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
                            "inner" => Ok(GeneratedField::Inner),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZkOutputProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.ZKOutputProof")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZkOutputProof, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ZkOutputProof {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.ZKOutputProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZkSpendProof {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.shielded_pool.v1.ZKSpendProof", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZkSpendProof {
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
                            "inner" => Ok(GeneratedField::Inner),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ZkSpendProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.shielded_pool.v1.ZKSpendProof")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZkSpendProof, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ZkSpendProof {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.shielded_pool.v1.ZKSpendProof", FIELDS, GeneratedVisitor)
    }
}
