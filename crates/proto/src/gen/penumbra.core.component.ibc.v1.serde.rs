impl serde::Serialize for ClientConnections {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.connections.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.ClientConnections", len)?;
        if !self.connections.is_empty() {
            struct_ser.serialize_field("connections", &self.connections)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ClientConnections {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "connections",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Connections,
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
                            "connections" => Ok(GeneratedField::Connections),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ClientConnections;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.ibc.v1.ClientConnections")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ClientConnections, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connections__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Connections => {
                            if connections__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connections"));
                            }
                            connections__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ClientConnections {
                    connections: connections__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.ClientConnections", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ClientCounter {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.counter != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.ClientCounter", len)?;
        if self.counter != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("counter", ToString::to_string(&self.counter).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ClientCounter {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "counter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Counter,
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
                            "counter" => Ok(GeneratedField::Counter),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ClientCounter;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.ibc.v1.ClientCounter")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ClientCounter, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut counter__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Counter => {
                            if counter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counter"));
                            }
                            counter__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ClientCounter {
                    counter: counter__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.ClientCounter", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ClientData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.client_id.is_empty() {
            len += 1;
        }
        if self.client_state.is_some() {
            len += 1;
        }
        if !self.processed_time.is_empty() {
            len += 1;
        }
        if self.processed_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.ClientData", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if let Some(v) = self.client_state.as_ref() {
            struct_ser.serialize_field("clientState", v)?;
        }
        if !self.processed_time.is_empty() {
            struct_ser.serialize_field("processedTime", &self.processed_time)?;
        }
        if self.processed_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("processedHeight", ToString::to_string(&self.processed_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ClientData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
            "client_state",
            "clientState",
            "processed_time",
            "processedTime",
            "processed_height",
            "processedHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
            ClientState,
            ProcessedTime,
            ProcessedHeight,
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
                            "clientId" | "client_id" => Ok(GeneratedField::ClientId),
                            "clientState" | "client_state" => Ok(GeneratedField::ClientState),
                            "processedTime" | "processed_time" => Ok(GeneratedField::ProcessedTime),
                            "processedHeight" | "processed_height" => Ok(GeneratedField::ProcessedHeight),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ClientData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.ibc.v1.ClientData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ClientData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                let mut client_state__ = None;
                let mut processed_time__ = None;
                let mut processed_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ClientState => {
                            if client_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientState"));
                            }
                            client_state__ = map_.next_value()?;
                        }
                        GeneratedField::ProcessedTime => {
                            if processed_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("processedTime"));
                            }
                            processed_time__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProcessedHeight => {
                            if processed_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("processedHeight"));
                            }
                            processed_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ClientData {
                    client_id: client_id__.unwrap_or_default(),
                    client_state: client_state__,
                    processed_time: processed_time__.unwrap_or_default(),
                    processed_height: processed_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.ClientData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConnectionCounter {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.counter != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.ConnectionCounter", len)?;
        if self.counter != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("counter", ToString::to_string(&self.counter).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConnectionCounter {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "counter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Counter,
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
                            "counter" => Ok(GeneratedField::Counter),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConnectionCounter;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.ibc.v1.ConnectionCounter")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ConnectionCounter, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut counter__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Counter => {
                            if counter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counter"));
                            }
                            counter__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ConnectionCounter {
                    counter: counter__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.ConnectionCounter", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConsensusState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.consensus_state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.ConsensusState", len)?;
        if let Some(v) = self.consensus_state.as_ref() {
            struct_ser.serialize_field("consensusState", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConsensusState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "consensus_state",
            "consensusState",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ConsensusState,
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
                            "consensusState" | "consensus_state" => Ok(GeneratedField::ConsensusState),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConsensusState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.ibc.v1.ConsensusState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ConsensusState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut consensus_state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ConsensusState => {
                            if consensus_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusState"));
                            }
                            consensus_state__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ConsensusState {
                    consensus_state: consensus_state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.ConsensusState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FungibleTokenPacketData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.denom.is_empty() {
            len += 1;
        }
        if !self.amount.is_empty() {
            len += 1;
        }
        if !self.sender.is_empty() {
            len += 1;
        }
        if !self.receiver.is_empty() {
            len += 1;
        }
        if !self.memo.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.FungibleTokenPacketData", len)?;
        if !self.denom.is_empty() {
            struct_ser.serialize_field("denom", &self.denom)?;
        }
        if !self.amount.is_empty() {
            struct_ser.serialize_field("amount", &self.amount)?;
        }
        if !self.sender.is_empty() {
            struct_ser.serialize_field("sender", &self.sender)?;
        }
        if !self.receiver.is_empty() {
            struct_ser.serialize_field("receiver", &self.receiver)?;
        }
        if !self.memo.is_empty() {
            struct_ser.serialize_field("memo", &self.memo)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FungibleTokenPacketData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "denom",
            "amount",
            "sender",
            "receiver",
            "memo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Denom,
            Amount,
            Sender,
            Receiver,
            Memo,
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
                            "denom" => Ok(GeneratedField::Denom),
                            "amount" => Ok(GeneratedField::Amount),
                            "sender" => Ok(GeneratedField::Sender),
                            "receiver" => Ok(GeneratedField::Receiver),
                            "memo" => Ok(GeneratedField::Memo),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FungibleTokenPacketData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.ibc.v1.FungibleTokenPacketData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FungibleTokenPacketData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut denom__ = None;
                let mut amount__ = None;
                let mut sender__ = None;
                let mut receiver__ = None;
                let mut memo__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Denom => {
                            if denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denom"));
                            }
                            denom__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map_.next_value()?);
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
                            receiver__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Memo => {
                            if memo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("memo"));
                            }
                            memo__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FungibleTokenPacketData {
                    denom: denom__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                    sender: sender__.unwrap_or_default(),
                    receiver: receiver__.unwrap_or_default(),
                    memo: memo__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.FungibleTokenPacketData", FIELDS, GeneratedVisitor)
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
        if self.ibc_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.GenesisContent", len)?;
        if let Some(v) = self.ibc_params.as_ref() {
            struct_ser.serialize_field("ibcParams", v)?;
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
            "ibc_params",
            "ibcParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IbcParams,
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
                            "ibcParams" | "ibc_params" => Ok(GeneratedField::IbcParams),
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
                formatter.write_str("struct penumbra.core.component.ibc.v1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ibc_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IbcParams => {
                            if ibc_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcParams"));
                            }
                            ibc_params__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    ibc_params: ibc_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IbcParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.ibc_enabled {
            len += 1;
        }
        if self.inbound_ics20_transfers_enabled {
            len += 1;
        }
        if self.outbound_ics20_transfers_enabled {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.IbcParameters", len)?;
        if self.ibc_enabled {
            struct_ser.serialize_field("ibcEnabled", &self.ibc_enabled)?;
        }
        if self.inbound_ics20_transfers_enabled {
            struct_ser.serialize_field("inboundIcs20TransfersEnabled", &self.inbound_ics20_transfers_enabled)?;
        }
        if self.outbound_ics20_transfers_enabled {
            struct_ser.serialize_field("outboundIcs20TransfersEnabled", &self.outbound_ics20_transfers_enabled)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IbcParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ibc_enabled",
            "ibcEnabled",
            "inbound_ics20_transfers_enabled",
            "inboundIcs20TransfersEnabled",
            "outbound_ics20_transfers_enabled",
            "outboundIcs20TransfersEnabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IbcEnabled,
            InboundIcs20TransfersEnabled,
            OutboundIcs20TransfersEnabled,
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
                            "ibcEnabled" | "ibc_enabled" => Ok(GeneratedField::IbcEnabled),
                            "inboundIcs20TransfersEnabled" | "inbound_ics20_transfers_enabled" => Ok(GeneratedField::InboundIcs20TransfersEnabled),
                            "outboundIcs20TransfersEnabled" | "outbound_ics20_transfers_enabled" => Ok(GeneratedField::OutboundIcs20TransfersEnabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = IbcParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.ibc.v1.IbcParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<IbcParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ibc_enabled__ = None;
                let mut inbound_ics20_transfers_enabled__ = None;
                let mut outbound_ics20_transfers_enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IbcEnabled => {
                            if ibc_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcEnabled"));
                            }
                            ibc_enabled__ = Some(map_.next_value()?);
                        }
                        GeneratedField::InboundIcs20TransfersEnabled => {
                            if inbound_ics20_transfers_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inboundIcs20TransfersEnabled"));
                            }
                            inbound_ics20_transfers_enabled__ = Some(map_.next_value()?);
                        }
                        GeneratedField::OutboundIcs20TransfersEnabled => {
                            if outbound_ics20_transfers_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outboundIcs20TransfersEnabled"));
                            }
                            outbound_ics20_transfers_enabled__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(IbcParameters {
                    ibc_enabled: ibc_enabled__.unwrap_or_default(),
                    inbound_ics20_transfers_enabled: inbound_ics20_transfers_enabled__.unwrap_or_default(),
                    outbound_ics20_transfers_enabled: outbound_ics20_transfers_enabled__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.IbcParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IbcRelay {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.raw_action.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.IbcRelay", len)?;
        if let Some(v) = self.raw_action.as_ref() {
            struct_ser.serialize_field("rawAction", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IbcRelay {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "raw_action",
            "rawAction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RawAction,
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
                            "rawAction" | "raw_action" => Ok(GeneratedField::RawAction),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = IbcRelay;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.ibc.v1.IbcRelay")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<IbcRelay, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut raw_action__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RawAction => {
                            if raw_action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rawAction"));
                            }
                            raw_action__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(IbcRelay {
                    raw_action: raw_action__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.IbcRelay", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Ics20Withdrawal {
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
        if self.denom.is_some() {
            len += 1;
        }
        if !self.destination_chain_address.is_empty() {
            len += 1;
        }
        if self.return_address.is_some() {
            len += 1;
        }
        if self.timeout_height.is_some() {
            len += 1;
        }
        if self.timeout_time != 0 {
            len += 1;
        }
        if !self.source_channel.is_empty() {
            len += 1;
        }
        if self.use_compat_address {
            len += 1;
        }
        if !self.ics20_memo.is_empty() {
            len += 1;
        }
        if self.use_transparent_address {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.Ics20Withdrawal", len)?;
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if let Some(v) = self.denom.as_ref() {
            struct_ser.serialize_field("denom", v)?;
        }
        if !self.destination_chain_address.is_empty() {
            struct_ser.serialize_field("destinationChainAddress", &self.destination_chain_address)?;
        }
        if let Some(v) = self.return_address.as_ref() {
            struct_ser.serialize_field("returnAddress", v)?;
        }
        if let Some(v) = self.timeout_height.as_ref() {
            struct_ser.serialize_field("timeoutHeight", v)?;
        }
        if self.timeout_time != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timeoutTime", ToString::to_string(&self.timeout_time).as_str())?;
        }
        if !self.source_channel.is_empty() {
            struct_ser.serialize_field("sourceChannel", &self.source_channel)?;
        }
        if self.use_compat_address {
            struct_ser.serialize_field("useCompatAddress", &self.use_compat_address)?;
        }
        if !self.ics20_memo.is_empty() {
            struct_ser.serialize_field("ics20Memo", &self.ics20_memo)?;
        }
        if self.use_transparent_address {
            struct_ser.serialize_field("useTransparentAddress", &self.use_transparent_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Ics20Withdrawal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
            "denom",
            "destination_chain_address",
            "destinationChainAddress",
            "return_address",
            "returnAddress",
            "timeout_height",
            "timeoutHeight",
            "timeout_time",
            "timeoutTime",
            "source_channel",
            "sourceChannel",
            "use_compat_address",
            "useCompatAddress",
            "ics20_memo",
            "ics20Memo",
            "use_transparent_address",
            "useTransparentAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Amount,
            Denom,
            DestinationChainAddress,
            ReturnAddress,
            TimeoutHeight,
            TimeoutTime,
            SourceChannel,
            UseCompatAddress,
            Ics20Memo,
            UseTransparentAddress,
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
                            "destinationChainAddress" | "destination_chain_address" => Ok(GeneratedField::DestinationChainAddress),
                            "returnAddress" | "return_address" => Ok(GeneratedField::ReturnAddress),
                            "timeoutHeight" | "timeout_height" => Ok(GeneratedField::TimeoutHeight),
                            "timeoutTime" | "timeout_time" => Ok(GeneratedField::TimeoutTime),
                            "sourceChannel" | "source_channel" => Ok(GeneratedField::SourceChannel),
                            "useCompatAddress" | "use_compat_address" => Ok(GeneratedField::UseCompatAddress),
                            "ics20Memo" | "ics20_memo" => Ok(GeneratedField::Ics20Memo),
                            "useTransparentAddress" | "use_transparent_address" => Ok(GeneratedField::UseTransparentAddress),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Ics20Withdrawal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.ibc.v1.Ics20Withdrawal")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Ics20Withdrawal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut denom__ = None;
                let mut destination_chain_address__ = None;
                let mut return_address__ = None;
                let mut timeout_height__ = None;
                let mut timeout_time__ = None;
                let mut source_channel__ = None;
                let mut use_compat_address__ = None;
                let mut ics20_memo__ = None;
                let mut use_transparent_address__ = None;
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
                            denom__ = map_.next_value()?;
                        }
                        GeneratedField::DestinationChainAddress => {
                            if destination_chain_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destinationChainAddress"));
                            }
                            destination_chain_address__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ReturnAddress => {
                            if return_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("returnAddress"));
                            }
                            return_address__ = map_.next_value()?;
                        }
                        GeneratedField::TimeoutHeight => {
                            if timeout_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeoutHeight"));
                            }
                            timeout_height__ = map_.next_value()?;
                        }
                        GeneratedField::TimeoutTime => {
                            if timeout_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeoutTime"));
                            }
                            timeout_time__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SourceChannel => {
                            if source_channel__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceChannel"));
                            }
                            source_channel__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UseCompatAddress => {
                            if use_compat_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("useCompatAddress"));
                            }
                            use_compat_address__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Ics20Memo => {
                            if ics20_memo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ics20Memo"));
                            }
                            ics20_memo__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UseTransparentAddress => {
                            if use_transparent_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("useTransparentAddress"));
                            }
                            use_transparent_address__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Ics20Withdrawal {
                    amount: amount__,
                    denom: denom__,
                    destination_chain_address: destination_chain_address__.unwrap_or_default(),
                    return_address: return_address__,
                    timeout_height: timeout_height__,
                    timeout_time: timeout_time__.unwrap_or_default(),
                    source_channel: source_channel__.unwrap_or_default(),
                    use_compat_address: use_compat_address__.unwrap_or_default(),
                    ics20_memo: ics20_memo__.unwrap_or_default(),
                    use_transparent_address: use_transparent_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.Ics20Withdrawal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VerifiedHeights {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.heights.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.ibc.v1.VerifiedHeights", len)?;
        if !self.heights.is_empty() {
            struct_ser.serialize_field("heights", &self.heights)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VerifiedHeights {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "heights",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Heights,
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
                            "heights" => Ok(GeneratedField::Heights),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VerifiedHeights;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.ibc.v1.VerifiedHeights")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<VerifiedHeights, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut heights__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Heights => {
                            if heights__.is_some() {
                                return Err(serde::de::Error::duplicate_field("heights"));
                            }
                            heights__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(VerifiedHeights {
                    heights: heights__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.ibc.v1.VerifiedHeights", FIELDS, GeneratedVisitor)
    }
}
