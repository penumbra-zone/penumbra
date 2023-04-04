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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.ibc.v1alpha1.ClientConnections", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.ibc.v1alpha1.ClientConnections")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ClientConnections, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut connections__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Connections => {
                            if connections__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connections"));
                            }
                            connections__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ClientConnections {
                    connections: connections__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.ibc.v1alpha1.ClientConnections", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.ibc.v1alpha1.ClientCounter", len)?;
        if self.counter != 0 {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.ibc.v1alpha1.ClientCounter")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ClientCounter, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut counter__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Counter => {
                            if counter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counter"));
                            }
                            counter__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ClientCounter {
                    counter: counter__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.ibc.v1alpha1.ClientCounter", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.ibc.v1alpha1.ClientData", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.ibc.v1alpha1.ClientData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ClientData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                let mut client_state__ = None;
                let mut processed_time__ = None;
                let mut processed_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::ClientState => {
                            if client_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientState"));
                            }
                            client_state__ = map.next_value()?;
                        }
                        GeneratedField::ProcessedTime => {
                            if processed_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("processedTime"));
                            }
                            processed_time__ = Some(map.next_value()?);
                        }
                        GeneratedField::ProcessedHeight => {
                            if processed_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("processedHeight"));
                            }
                            processed_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
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
        deserializer.deserialize_struct("penumbra.core.ibc.v1alpha1.ClientData", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.ibc.v1alpha1.ConnectionCounter", len)?;
        if self.counter != 0 {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.ibc.v1alpha1.ConnectionCounter")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConnectionCounter, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut counter__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Counter => {
                            if counter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("counter"));
                            }
                            counter__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ConnectionCounter {
                    counter: counter__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.ibc.v1alpha1.ConnectionCounter", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.ibc.v1alpha1.ConsensusState", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.ibc.v1alpha1.ConsensusState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConsensusState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut consensus_state__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ConsensusState => {
                            if consensus_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusState"));
                            }
                            consensus_state__ = map.next_value()?;
                        }
                    }
                }
                Ok(ConsensusState {
                    consensus_state: consensus_state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.ibc.v1alpha1.ConsensusState", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.ibc.v1alpha1.FungibleTokenPacketData", len)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Denom,
            Amount,
            Sender,
            Receiver,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.ibc.v1alpha1.FungibleTokenPacketData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FungibleTokenPacketData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut denom__ = None;
                let mut amount__ = None;
                let mut sender__ = None;
                let mut receiver__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Denom => {
                            if denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denom"));
                            }
                            denom__ = Some(map.next_value()?);
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = Some(map.next_value()?);
                        }
                        GeneratedField::Sender => {
                            if sender__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sender"));
                            }
                            sender__ = Some(map.next_value()?);
                        }
                        GeneratedField::Receiver => {
                            if receiver__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiver"));
                            }
                            receiver__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(FungibleTokenPacketData {
                    denom: denom__.unwrap_or_default(),
                    amount: amount__.unwrap_or_default(),
                    sender: sender__.unwrap_or_default(),
                    receiver: receiver__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.ibc.v1alpha1.FungibleTokenPacketData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IbcAction {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.ibc.v1alpha1.IbcAction", len)?;
        if let Some(v) = self.raw_action.as_ref() {
            struct_ser.serialize_field("rawAction", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IbcAction {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = IbcAction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.ibc.v1alpha1.IbcAction")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<IbcAction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut raw_action__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RawAction => {
                            if raw_action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rawAction"));
                            }
                            raw_action__ = map.next_value()?;
                        }
                    }
                }
                Ok(IbcAction {
                    raw_action: raw_action__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.ibc.v1alpha1.IbcAction", FIELDS, GeneratedVisitor)
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
        if !self.destination_chain_id.is_empty() {
            len += 1;
        }
        if self.denom.is_some() {
            len += 1;
        }
        if self.amount.is_some() {
            len += 1;
        }
        if !self.destination_chain_address.is_empty() {
            len += 1;
        }
        if self.return_address.is_some() {
            len += 1;
        }
        if self.timeout_height != 0 {
            len += 1;
        }
        if self.timeout_time != 0 {
            len += 1;
        }
        if !self.source_port.is_empty() {
            len += 1;
        }
        if !self.source_channel.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.ibc.v1alpha1.Ics20Withdrawal", len)?;
        if !self.destination_chain_id.is_empty() {
            struct_ser.serialize_field("destinationChainId", &self.destination_chain_id)?;
        }
        if let Some(v) = self.denom.as_ref() {
            struct_ser.serialize_field("denom", v)?;
        }
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if !self.destination_chain_address.is_empty() {
            struct_ser.serialize_field("destinationChainAddress", &self.destination_chain_address)?;
        }
        if let Some(v) = self.return_address.as_ref() {
            struct_ser.serialize_field("returnAddress", v)?;
        }
        if self.timeout_height != 0 {
            struct_ser.serialize_field("timeoutHeight", ToString::to_string(&self.timeout_height).as_str())?;
        }
        if self.timeout_time != 0 {
            struct_ser.serialize_field("timeoutTime", ToString::to_string(&self.timeout_time).as_str())?;
        }
        if !self.source_port.is_empty() {
            struct_ser.serialize_field("sourcePort", &self.source_port)?;
        }
        if !self.source_channel.is_empty() {
            struct_ser.serialize_field("sourceChannel", &self.source_channel)?;
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
            "destination_chain_id",
            "destinationChainId",
            "denom",
            "amount",
            "destination_chain_address",
            "destinationChainAddress",
            "return_address",
            "returnAddress",
            "timeout_height",
            "timeoutHeight",
            "timeout_time",
            "timeoutTime",
            "source_port",
            "sourcePort",
            "source_channel",
            "sourceChannel",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DestinationChainId,
            Denom,
            Amount,
            DestinationChainAddress,
            ReturnAddress,
            TimeoutHeight,
            TimeoutTime,
            SourcePort,
            SourceChannel,
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
                            "destinationChainId" | "destination_chain_id" => Ok(GeneratedField::DestinationChainId),
                            "denom" => Ok(GeneratedField::Denom),
                            "amount" => Ok(GeneratedField::Amount),
                            "destinationChainAddress" | "destination_chain_address" => Ok(GeneratedField::DestinationChainAddress),
                            "returnAddress" | "return_address" => Ok(GeneratedField::ReturnAddress),
                            "timeoutHeight" | "timeout_height" => Ok(GeneratedField::TimeoutHeight),
                            "timeoutTime" | "timeout_time" => Ok(GeneratedField::TimeoutTime),
                            "sourcePort" | "source_port" => Ok(GeneratedField::SourcePort),
                            "sourceChannel" | "source_channel" => Ok(GeneratedField::SourceChannel),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.ibc.v1alpha1.Ics20Withdrawal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Ics20Withdrawal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut destination_chain_id__ = None;
                let mut denom__ = None;
                let mut amount__ = None;
                let mut destination_chain_address__ = None;
                let mut return_address__ = None;
                let mut timeout_height__ = None;
                let mut timeout_time__ = None;
                let mut source_port__ = None;
                let mut source_channel__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DestinationChainId => {
                            if destination_chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destinationChainId"));
                            }
                            destination_chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Denom => {
                            if denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denom"));
                            }
                            denom__ = map.next_value()?;
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map.next_value()?;
                        }
                        GeneratedField::DestinationChainAddress => {
                            if destination_chain_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destinationChainAddress"));
                            }
                            destination_chain_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::ReturnAddress => {
                            if return_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("returnAddress"));
                            }
                            return_address__ = map.next_value()?;
                        }
                        GeneratedField::TimeoutHeight => {
                            if timeout_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeoutHeight"));
                            }
                            timeout_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TimeoutTime => {
                            if timeout_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeoutTime"));
                            }
                            timeout_time__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SourcePort => {
                            if source_port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourcePort"));
                            }
                            source_port__ = Some(map.next_value()?);
                        }
                        GeneratedField::SourceChannel => {
                            if source_channel__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceChannel"));
                            }
                            source_channel__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Ics20Withdrawal {
                    destination_chain_id: destination_chain_id__.unwrap_or_default(),
                    denom: denom__,
                    amount: amount__,
                    destination_chain_address: destination_chain_address__.unwrap_or_default(),
                    return_address: return_address__,
                    timeout_height: timeout_height__.unwrap_or_default(),
                    timeout_time: timeout_time__.unwrap_or_default(),
                    source_port: source_port__.unwrap_or_default(),
                    source_channel: source_channel__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.ibc.v1alpha1.Ics20Withdrawal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Ics20WithdrawalPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.destination_chain_id.is_empty() {
            len += 1;
        }
        if !self.destination_chain_address.is_empty() {
            len += 1;
        }
        if self.asset_id.is_some() {
            len += 1;
        }
        if self.amount.is_some() {
            len += 1;
        }
        if self.return_address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.ibc.v1alpha1.Ics20WithdrawalPlan", len)?;
        if !self.destination_chain_id.is_empty() {
            struct_ser.serialize_field("destinationChainId", &self.destination_chain_id)?;
        }
        if !self.destination_chain_address.is_empty() {
            struct_ser.serialize_field("destinationChainAddress", &self.destination_chain_address)?;
        }
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if let Some(v) = self.return_address.as_ref() {
            struct_ser.serialize_field("returnAddress", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Ics20WithdrawalPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "destination_chain_id",
            "destinationChainId",
            "destination_chain_address",
            "destinationChainAddress",
            "asset_id",
            "assetId",
            "amount",
            "return_address",
            "returnAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DestinationChainId,
            DestinationChainAddress,
            AssetId,
            Amount,
            ReturnAddress,
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
                            "destinationChainId" | "destination_chain_id" => Ok(GeneratedField::DestinationChainId),
                            "destinationChainAddress" | "destination_chain_address" => Ok(GeneratedField::DestinationChainAddress),
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            "amount" => Ok(GeneratedField::Amount),
                            "returnAddress" | "return_address" => Ok(GeneratedField::ReturnAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Ics20WithdrawalPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.ibc.v1alpha1.Ics20WithdrawalPlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Ics20WithdrawalPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut destination_chain_id__ = None;
                let mut destination_chain_address__ = None;
                let mut asset_id__ = None;
                let mut amount__ = None;
                let mut return_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DestinationChainId => {
                            if destination_chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destinationChainId"));
                            }
                            destination_chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::DestinationChainAddress => {
                            if destination_chain_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destinationChainAddress"));
                            }
                            destination_chain_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map.next_value()?;
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map.next_value()?;
                        }
                        GeneratedField::ReturnAddress => {
                            if return_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("returnAddress"));
                            }
                            return_address__ = map.next_value()?;
                        }
                    }
                }
                Ok(Ics20WithdrawalPlan {
                    destination_chain_id: destination_chain_id__.unwrap_or_default(),
                    destination_chain_address: destination_chain_address__.unwrap_or_default(),
                    asset_id: asset_id__,
                    amount: amount__,
                    return_address: return_address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.ibc.v1alpha1.Ics20WithdrawalPlan", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.ibc.v1alpha1.VerifiedHeights", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.ibc.v1alpha1.VerifiedHeights")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<VerifiedHeights, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut heights__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Heights => {
                            if heights__.is_some() {
                                return Err(serde::de::Error::duplicate_field("heights"));
                            }
                            heights__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(VerifiedHeights {
                    heights: heights__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.ibc.v1alpha1.VerifiedHeights", FIELDS, GeneratedVisitor)
    }
}
