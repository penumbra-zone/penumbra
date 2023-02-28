impl serde::Serialize for BareTradingFunction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fee != 0 {
            len += 1;
        }
        if self.p.is_some() {
            len += 1;
        }
        if self.q.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.BareTradingFunction", len)?;
        if self.fee != 0 {
            struct_ser.serialize_field("fee", &self.fee)?;
        }
        if let Some(v) = self.p.as_ref() {
            struct_ser.serialize_field("p", v)?;
        }
        if let Some(v) = self.q.as_ref() {
            struct_ser.serialize_field("q", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BareTradingFunction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fee",
            "p",
            "q",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fee,
            P,
            Q,
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
                            "fee" => Ok(GeneratedField::Fee),
                            "p" => Ok(GeneratedField::P),
                            "q" => Ok(GeneratedField::Q),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BareTradingFunction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.BareTradingFunction")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BareTradingFunction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee__ = None;
                let mut p__ = None;
                let mut q__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::P => {
                            if p__.is_some() {
                                return Err(serde::de::Error::duplicate_field("p"));
                            }
                            p__ = map.next_value()?;
                        }
                        GeneratedField::Q => {
                            if q__.is_some() {
                                return Err(serde::de::Error::duplicate_field("q"));
                            }
                            q__ = map.next_value()?;
                        }
                    }
                }
                Ok(BareTradingFunction {
                    fee: fee__.unwrap_or_default(),
                    p: p__,
                    q: q__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.BareTradingFunction", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BatchSwapOutputData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.delta_1 != 0 {
            len += 1;
        }
        if self.delta_2 != 0 {
            len += 1;
        }
        if self.lambda_1 != 0 {
            len += 1;
        }
        if self.lambda_2 != 0 {
            len += 1;
        }
        if self.success {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.trading_pair.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.BatchSwapOutputData", len)?;
        if self.delta_1 != 0 {
            struct_ser.serialize_field("delta1", ToString::to_string(&self.delta_1).as_str())?;
        }
        if self.delta_2 != 0 {
            struct_ser.serialize_field("delta2", ToString::to_string(&self.delta_2).as_str())?;
        }
        if self.lambda_1 != 0 {
            struct_ser.serialize_field("lambda1", ToString::to_string(&self.lambda_1).as_str())?;
        }
        if self.lambda_2 != 0 {
            struct_ser.serialize_field("lambda2", ToString::to_string(&self.lambda_2).as_str())?;
        }
        if self.success {
            struct_ser.serialize_field("success", &self.success)?;
        }
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BatchSwapOutputData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delta_1",
            "delta1",
            "delta_2",
            "delta2",
            "lambda_1",
            "lambda1",
            "lambda_2",
            "lambda2",
            "success",
            "height",
            "trading_pair",
            "tradingPair",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Delta1,
            Delta2,
            Lambda1,
            Lambda2,
            Success,
            Height,
            TradingPair,
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
                            "delta1" | "delta_1" => Ok(GeneratedField::Delta1),
                            "delta2" | "delta_2" => Ok(GeneratedField::Delta2),
                            "lambda1" | "lambda_1" => Ok(GeneratedField::Lambda1),
                            "lambda2" | "lambda_2" => Ok(GeneratedField::Lambda2),
                            "success" => Ok(GeneratedField::Success),
                            "height" => Ok(GeneratedField::Height),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BatchSwapOutputData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.BatchSwapOutputData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BatchSwapOutputData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delta_1__ = None;
                let mut delta_2__ = None;
                let mut lambda_1__ = None;
                let mut lambda_2__ = None;
                let mut success__ = None;
                let mut height__ = None;
                let mut trading_pair__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Delta1 => {
                            if delta_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta1"));
                            }
                            delta_1__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Delta2 => {
                            if delta_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta2"));
                            }
                            delta_2__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Lambda1 => {
                            if lambda_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lambda1"));
                            }
                            lambda_1__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Lambda2 => {
                            if lambda_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lambda2"));
                            }
                            lambda_2__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Success => {
                            if success__.is_some() {
                                return Err(serde::de::Error::duplicate_field("success"));
                            }
                            success__ = Some(map.next_value()?);
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map.next_value()?;
                        }
                    }
                }
                Ok(BatchSwapOutputData {
                    delta_1: delta_1__.unwrap_or_default(),
                    delta_2: delta_2__.unwrap_or_default(),
                    lambda_1: lambda_1__.unwrap_or_default(),
                    lambda_2: lambda_2__.unwrap_or_default(),
                    success: success__.unwrap_or_default(),
                    height: height__.unwrap_or_default(),
                    trading_pair: trading_pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.BatchSwapOutputData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DirectedTradingPair {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start.is_some() {
            len += 1;
        }
        if self.end.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.DirectedTradingPair", len)?;
        if let Some(v) = self.start.as_ref() {
            struct_ser.serialize_field("start", v)?;
        }
        if let Some(v) = self.end.as_ref() {
            struct_ser.serialize_field("end", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DirectedTradingPair {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start",
            "end",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Start,
            End,
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
                            "start" => Ok(GeneratedField::Start),
                            "end" => Ok(GeneratedField::End),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DirectedTradingPair;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.DirectedTradingPair")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DirectedTradingPair, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start__ = None;
                let mut end__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Start => {
                            if start__.is_some() {
                                return Err(serde::de::Error::duplicate_field("start"));
                            }
                            start__ = map.next_value()?;
                        }
                        GeneratedField::End => {
                            if end__.is_some() {
                                return Err(serde::de::Error::duplicate_field("end"));
                            }
                            end__ = map.next_value()?;
                        }
                    }
                }
                Ok(DirectedTradingPair {
                    start: start__,
                    end: end__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.DirectedTradingPair", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LpNft {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.position_id.is_some() {
            len += 1;
        }
        if self.state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.LpNft", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.state.as_ref() {
            struct_ser.serialize_field("state", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LpNft {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position_id",
            "positionId",
            "state",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionId,
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
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
            type Value = LpNft;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.LpNft")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<LpNft, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut state__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map.next_value()?;
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map.next_value()?;
                        }
                    }
                }
                Ok(LpNft {
                    position_id: position_id__,
                    state: state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.LpNft", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MockFlowCiphertext {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.MockFlowCiphertext", len)?;
        if self.value != 0 {
            struct_ser.serialize_field("value", ToString::to_string(&self.value).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MockFlowCiphertext {
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
            type Value = MockFlowCiphertext;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.MockFlowCiphertext")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MockFlowCiphertext, V::Error>
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
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MockFlowCiphertext {
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.MockFlowCiphertext", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Path {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.pair.is_some() {
            len += 1;
        }
        if !self.route.is_empty() {
            len += 1;
        }
        if self.phi.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.Path", len)?;
        if let Some(v) = self.pair.as_ref() {
            struct_ser.serialize_field("pair", v)?;
        }
        if !self.route.is_empty() {
            struct_ser.serialize_field("route", &self.route)?;
        }
        if let Some(v) = self.phi.as_ref() {
            struct_ser.serialize_field("phi", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Path {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pair",
            "route",
            "phi",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pair,
            Route,
            Phi,
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
                            "pair" => Ok(GeneratedField::Pair),
                            "route" => Ok(GeneratedField::Route),
                            "phi" => Ok(GeneratedField::Phi),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Path;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.Path")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Path, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pair__ = None;
                let mut route__ = None;
                let mut phi__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Pair => {
                            if pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pair"));
                            }
                            pair__ = map.next_value()?;
                        }
                        GeneratedField::Route => {
                            if route__.is_some() {
                                return Err(serde::de::Error::duplicate_field("route"));
                            }
                            route__ = Some(map.next_value()?);
                        }
                        GeneratedField::Phi => {
                            if phi__.is_some() {
                                return Err(serde::de::Error::duplicate_field("phi"));
                            }
                            phi__ = map.next_value()?;
                        }
                    }
                }
                Ok(Path {
                    pair: pair__,
                    route: route__.unwrap_or_default(),
                    phi: phi__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.Path", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Position {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.phi.is_some() {
            len += 1;
        }
        if !self.nonce.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.Position", len)?;
        if let Some(v) = self.phi.as_ref() {
            struct_ser.serialize_field("phi", v)?;
        }
        if !self.nonce.is_empty() {
            struct_ser.serialize_field("nonce", pbjson::private::base64::encode(&self.nonce).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Position {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "phi",
            "nonce",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Phi,
            Nonce,
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
                            "phi" => Ok(GeneratedField::Phi),
                            "nonce" => Ok(GeneratedField::Nonce),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Position;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.Position")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Position, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut phi__ = None;
                let mut nonce__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Phi => {
                            if phi__.is_some() {
                                return Err(serde::de::Error::duplicate_field("phi"));
                            }
                            phi__ = map.next_value()?;
                        }
                        GeneratedField::Nonce => {
                            if nonce__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nonce"));
                            }
                            nonce__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Position {
                    phi: phi__,
                    nonce: nonce__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.Position", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionClose {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.position_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.PositionClose", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PositionClose {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position_id",
            "positionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionId,
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PositionClose;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.PositionClose")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PositionClose, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map.next_value()?;
                        }
                    }
                }
                Ok(PositionClose {
                    position_id: position_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.PositionClose", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionId {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.PositionId", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PositionId {
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
            type Value = PositionId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.PositionId")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PositionId, V::Error>
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
                Ok(PositionId {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.PositionId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionMetadata {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.position.is_some() {
            len += 1;
        }
        if self.state.is_some() {
            len += 1;
        }
        if self.reserves.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.PositionMetadata", len)?;
        if let Some(v) = self.position.as_ref() {
            struct_ser.serialize_field("position", v)?;
        }
        if let Some(v) = self.state.as_ref() {
            struct_ser.serialize_field("state", v)?;
        }
        if let Some(v) = self.reserves.as_ref() {
            struct_ser.serialize_field("reserves", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PositionMetadata {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position",
            "state",
            "reserves",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Position,
            State,
            Reserves,
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
                            "position" => Ok(GeneratedField::Position),
                            "state" => Ok(GeneratedField::State),
                            "reserves" => Ok(GeneratedField::Reserves),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PositionMetadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.PositionMetadata")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PositionMetadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position__ = None;
                let mut state__ = None;
                let mut reserves__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = map.next_value()?;
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map.next_value()?;
                        }
                        GeneratedField::Reserves => {
                            if reserves__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves"));
                            }
                            reserves__ = map.next_value()?;
                        }
                    }
                }
                Ok(PositionMetadata {
                    position: position__,
                    state: state__,
                    reserves: reserves__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.PositionMetadata", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionOpen {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.position.is_some() {
            len += 1;
        }
        if self.initial_reserves.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.PositionOpen", len)?;
        if let Some(v) = self.position.as_ref() {
            struct_ser.serialize_field("position", v)?;
        }
        if let Some(v) = self.initial_reserves.as_ref() {
            struct_ser.serialize_field("initialReserves", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PositionOpen {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position",
            "initial_reserves",
            "initialReserves",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Position,
            InitialReserves,
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
                            "position" => Ok(GeneratedField::Position),
                            "initialReserves" | "initial_reserves" => Ok(GeneratedField::InitialReserves),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PositionOpen;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.PositionOpen")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PositionOpen, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position__ = None;
                let mut initial_reserves__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = map.next_value()?;
                        }
                        GeneratedField::InitialReserves => {
                            if initial_reserves__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initialReserves"));
                            }
                            initial_reserves__ = map.next_value()?;
                        }
                    }
                }
                Ok(PositionOpen {
                    position: position__,
                    initial_reserves: initial_reserves__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.PositionOpen", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionRewardClaim {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.position_id.is_some() {
            len += 1;
        }
        if self.rewards_commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.PositionRewardClaim", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.rewards_commitment.as_ref() {
            struct_ser.serialize_field("rewardsCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PositionRewardClaim {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position_id",
            "positionId",
            "rewards_commitment",
            "rewardsCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionId,
            RewardsCommitment,
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "rewardsCommitment" | "rewards_commitment" => Ok(GeneratedField::RewardsCommitment),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PositionRewardClaim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.PositionRewardClaim")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PositionRewardClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut rewards_commitment__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map.next_value()?;
                        }
                        GeneratedField::RewardsCommitment => {
                            if rewards_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewardsCommitment"));
                            }
                            rewards_commitment__ = map.next_value()?;
                        }
                    }
                }
                Ok(PositionRewardClaim {
                    position_id: position_id__,
                    rewards_commitment: rewards_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.PositionRewardClaim", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionRewardClaimPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.PositionRewardClaimPlan", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PositionRewardClaimPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PositionRewardClaimPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.PositionRewardClaimPlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PositionRewardClaimPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(PositionRewardClaimPlan {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.PositionRewardClaimPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionState {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.PositionState", len)?;
        if self.state != 0 {
            let v = position_state::PositionStateEnum::from_i32(self.state)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.state)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PositionState {
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
            type Value = PositionState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.PositionState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PositionState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = Some(map.next_value::<position_state::PositionStateEnum>()? as i32);
                        }
                    }
                }
                Ok(PositionState {
                    state: state__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.PositionState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for position_state::PositionStateEnum {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "POSITION_STATE_ENUM_UNSPECIFIED",
            Self::Opened => "POSITION_STATE_ENUM_OPENED",
            Self::Closed => "POSITION_STATE_ENUM_CLOSED",
            Self::Withdrawn => "POSITION_STATE_ENUM_WITHDRAWN",
            Self::Claimed => "POSITION_STATE_ENUM_CLAIMED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for position_state::PositionStateEnum {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "POSITION_STATE_ENUM_UNSPECIFIED",
            "POSITION_STATE_ENUM_OPENED",
            "POSITION_STATE_ENUM_CLOSED",
            "POSITION_STATE_ENUM_WITHDRAWN",
            "POSITION_STATE_ENUM_CLAIMED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = position_state::PositionStateEnum;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(position_state::PositionStateEnum::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(position_state::PositionStateEnum::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "POSITION_STATE_ENUM_UNSPECIFIED" => Ok(position_state::PositionStateEnum::Unspecified),
                    "POSITION_STATE_ENUM_OPENED" => Ok(position_state::PositionStateEnum::Opened),
                    "POSITION_STATE_ENUM_CLOSED" => Ok(position_state::PositionStateEnum::Closed),
                    "POSITION_STATE_ENUM_WITHDRAWN" => Ok(position_state::PositionStateEnum::Withdrawn),
                    "POSITION_STATE_ENUM_CLAIMED" => Ok(position_state::PositionStateEnum::Claimed),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for PositionWithdraw {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.position_id.is_some() {
            len += 1;
        }
        if self.reserves_commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.PositionWithdraw", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.reserves_commitment.as_ref() {
            struct_ser.serialize_field("reservesCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PositionWithdraw {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position_id",
            "positionId",
            "reserves_commitment",
            "reservesCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionId,
            ReservesCommitment,
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "reservesCommitment" | "reserves_commitment" => Ok(GeneratedField::ReservesCommitment),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PositionWithdraw;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.PositionWithdraw")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PositionWithdraw, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut reserves_commitment__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map.next_value()?;
                        }
                        GeneratedField::ReservesCommitment => {
                            if reserves_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reservesCommitment"));
                            }
                            reserves_commitment__ = map.next_value()?;
                        }
                    }
                }
                Ok(PositionWithdraw {
                    position_id: position_id__,
                    reserves_commitment: reserves_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.PositionWithdraw", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionWithdrawPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.PositionWithdrawPlan", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PositionWithdrawPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PositionWithdrawPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.PositionWithdrawPlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PositionWithdrawPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(PositionWithdrawPlan {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.PositionWithdrawPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Reserves {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.r1.is_some() {
            len += 1;
        }
        if self.r2.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.Reserves", len)?;
        if let Some(v) = self.r1.as_ref() {
            struct_ser.serialize_field("r1", v)?;
        }
        if let Some(v) = self.r2.as_ref() {
            struct_ser.serialize_field("r2", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Reserves {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "r1",
            "r2",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            R1,
            R2,
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
                            "r1" => Ok(GeneratedField::R1),
                            "r2" => Ok(GeneratedField::R2),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Reserves;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.Reserves")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Reserves, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r1__ = None;
                let mut r2__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::R1 => {
                            if r1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("r1"));
                            }
                            r1__ = map.next_value()?;
                        }
                        GeneratedField::R2 => {
                            if r2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("r2"));
                            }
                            r2__ = map.next_value()?;
                        }
                    }
                }
                Ok(Reserves {
                    r1: r1__,
                    r2: r2__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.Reserves", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Swap {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.body.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.Swap", len)?;
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Swap {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proof",
            "body",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proof,
            Body,
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
                            "proof" => Ok(GeneratedField::Proof),
                            "body" => Ok(GeneratedField::Body),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Swap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.Swap")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Swap, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proof__ = None;
                let mut body__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map.next_value()?;
                        }
                    }
                }
                Ok(Swap {
                    proof: proof__.unwrap_or_default(),
                    body: body__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.Swap", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.trading_pair.is_some() {
            len += 1;
        }
        if self.delta_1_i.is_some() {
            len += 1;
        }
        if self.delta_2_i.is_some() {
            len += 1;
        }
        if self.fee_commitment.is_some() {
            len += 1;
        }
        if self.payload.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapBody", len)?;
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.delta_1_i.as_ref() {
            struct_ser.serialize_field("delta1I", v)?;
        }
        if let Some(v) = self.delta_2_i.as_ref() {
            struct_ser.serialize_field("delta2I", v)?;
        }
        if let Some(v) = self.fee_commitment.as_ref() {
            struct_ser.serialize_field("feeCommitment", v)?;
        }
        if let Some(v) = self.payload.as_ref() {
            struct_ser.serialize_field("payload", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trading_pair",
            "tradingPair",
            "delta_1_i",
            "delta1I",
            "delta_2_i",
            "delta2I",
            "fee_commitment",
            "feeCommitment",
            "payload",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TradingPair,
            Delta1I,
            Delta2I,
            FeeCommitment,
            Payload,
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
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "delta1I" | "delta_1_i" => Ok(GeneratedField::Delta1I),
                            "delta2I" | "delta_2_i" => Ok(GeneratedField::Delta2I),
                            "feeCommitment" | "fee_commitment" => Ok(GeneratedField::FeeCommitment),
                            "payload" => Ok(GeneratedField::Payload),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapBody")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trading_pair__ = None;
                let mut delta_1_i__ = None;
                let mut delta_2_i__ = None;
                let mut fee_commitment__ = None;
                let mut payload__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map.next_value()?;
                        }
                        GeneratedField::Delta1I => {
                            if delta_1_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta1I"));
                            }
                            delta_1_i__ = map.next_value()?;
                        }
                        GeneratedField::Delta2I => {
                            if delta_2_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta2I"));
                            }
                            delta_2_i__ = map.next_value()?;
                        }
                        GeneratedField::FeeCommitment => {
                            if fee_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeCommitment"));
                            }
                            fee_commitment__ = map.next_value()?;
                        }
                        GeneratedField::Payload => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payload"));
                            }
                            payload__ = map.next_value()?;
                        }
                    }
                }
                Ok(SwapBody {
                    trading_pair: trading_pair__,
                    delta_1_i: delta_1_i__,
                    delta_2_i: delta_2_i__,
                    fee_commitment: fee_commitment__,
                    payload: payload__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapClaim {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.proof.is_empty() {
            len += 1;
        }
        if self.body.is_some() {
            len += 1;
        }
        if self.epoch_duration != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapClaim", len)?;
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if self.epoch_duration != 0 {
            struct_ser.serialize_field("epochDuration", ToString::to_string(&self.epoch_duration).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapClaim {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proof",
            "body",
            "epoch_duration",
            "epochDuration",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proof,
            Body,
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
                            "proof" => Ok(GeneratedField::Proof),
                            "body" => Ok(GeneratedField::Body),
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
            type Value = SwapClaim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapClaim")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proof__ = None;
                let mut body__ = None;
                let mut epoch_duration__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map.next_value()?;
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
                Ok(SwapClaim {
                    proof: proof__.unwrap_or_default(),
                    body: body__,
                    epoch_duration: epoch_duration__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapClaim", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapClaimBody {
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
        if self.fee.is_some() {
            len += 1;
        }
        if self.output_1_commitment.is_some() {
            len += 1;
        }
        if self.output_2_commitment.is_some() {
            len += 1;
        }
        if self.output_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapClaimBody", len)?;
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        if let Some(v) = self.output_1_commitment.as_ref() {
            struct_ser.serialize_field("output1Commitment", v)?;
        }
        if let Some(v) = self.output_2_commitment.as_ref() {
            struct_ser.serialize_field("output2Commitment", v)?;
        }
        if let Some(v) = self.output_data.as_ref() {
            struct_ser.serialize_field("outputData", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapClaimBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "nullifier",
            "fee",
            "output_1_commitment",
            "output1Commitment",
            "output_2_commitment",
            "output2Commitment",
            "output_data",
            "outputData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Nullifier,
            Fee,
            Output1Commitment,
            Output2Commitment,
            OutputData,
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
                            "fee" => Ok(GeneratedField::Fee),
                            "output1Commitment" | "output_1_commitment" => Ok(GeneratedField::Output1Commitment),
                            "output2Commitment" | "output_2_commitment" => Ok(GeneratedField::Output2Commitment),
                            "outputData" | "output_data" => Ok(GeneratedField::OutputData),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapClaimBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapClaimBody")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapClaimBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut nullifier__ = None;
                let mut fee__ = None;
                let mut output_1_commitment__ = None;
                let mut output_2_commitment__ = None;
                let mut output_data__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map.next_value()?;
                        }
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map.next_value()?;
                        }
                        GeneratedField::Output1Commitment => {
                            if output_1_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output1Commitment"));
                            }
                            output_1_commitment__ = map.next_value()?;
                        }
                        GeneratedField::Output2Commitment => {
                            if output_2_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output2Commitment"));
                            }
                            output_2_commitment__ = map.next_value()?;
                        }
                        GeneratedField::OutputData => {
                            if output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputData"));
                            }
                            output_data__ = map.next_value()?;
                        }
                    }
                }
                Ok(SwapClaimBody {
                    nullifier: nullifier__,
                    fee: fee__,
                    output_1_commitment: output_1_commitment__,
                    output_2_commitment: output_2_commitment__,
                    output_data: output_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapClaimBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapClaimPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_plaintext.is_some() {
            len += 1;
        }
        if self.position != 0 {
            len += 1;
        }
        if self.output_data.is_some() {
            len += 1;
        }
        if self.epoch_duration != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapClaimPlan", len)?;
        if let Some(v) = self.swap_plaintext.as_ref() {
            struct_ser.serialize_field("swapPlaintext", v)?;
        }
        if self.position != 0 {
            struct_ser.serialize_field("position", ToString::to_string(&self.position).as_str())?;
        }
        if let Some(v) = self.output_data.as_ref() {
            struct_ser.serialize_field("outputData", v)?;
        }
        if self.epoch_duration != 0 {
            struct_ser.serialize_field("epochDuration", ToString::to_string(&self.epoch_duration).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapClaimPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_plaintext",
            "swapPlaintext",
            "position",
            "output_data",
            "outputData",
            "epoch_duration",
            "epochDuration",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapPlaintext,
            Position,
            OutputData,
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
                            "swapPlaintext" | "swap_plaintext" => Ok(GeneratedField::SwapPlaintext),
                            "position" => Ok(GeneratedField::Position),
                            "outputData" | "output_data" => Ok(GeneratedField::OutputData),
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
            type Value = SwapClaimPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapClaimPlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapClaimPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_plaintext__ = None;
                let mut position__ = None;
                let mut output_data__ = None;
                let mut epoch_duration__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SwapPlaintext => {
                            if swap_plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapPlaintext"));
                            }
                            swap_plaintext__ = map.next_value()?;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::OutputData => {
                            if output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputData"));
                            }
                            output_data__ = map.next_value()?;
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
                Ok(SwapClaimPlan {
                    swap_plaintext: swap_plaintext__,
                    position: position__.unwrap_or_default(),
                    output_data: output_data__,
                    epoch_duration: epoch_duration__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapClaimPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapClaimView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_claim_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapClaimView", len)?;
        if let Some(v) = self.swap_claim_view.as_ref() {
            match v {
                swap_claim_view::SwapClaimView::Visible(v) => {
                    struct_ser.serialize_field("visible", v)?;
                }
                swap_claim_view::SwapClaimView::Opaque(v) => {
                    struct_ser.serialize_field("opaque", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapClaimView {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapClaimView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapClaimView")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapClaimView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_claim_view__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if swap_claim_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            swap_claim_view__ = map.next_value::<::std::option::Option<_>>()?.map(swap_claim_view::SwapClaimView::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if swap_claim_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            swap_claim_view__ = map.next_value::<::std::option::Option<_>>()?.map(swap_claim_view::SwapClaimView::Opaque)
;
                        }
                    }
                }
                Ok(SwapClaimView {
                    swap_claim_view: swap_claim_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapClaimView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for swap_claim_view::Opaque {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_claim.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapClaimView.Opaque", len)?;
        if let Some(v) = self.swap_claim.as_ref() {
            struct_ser.serialize_field("swapClaim", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for swap_claim_view::Opaque {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_claim",
            "swapClaim",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapClaim,
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
                            "swapClaim" | "swap_claim" => Ok(GeneratedField::SwapClaim),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = swap_claim_view::Opaque;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapClaimView.Opaque")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<swap_claim_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_claim__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SwapClaim => {
                            if swap_claim__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            swap_claim__ = map.next_value()?;
                        }
                    }
                }
                Ok(swap_claim_view::Opaque {
                    swap_claim: swap_claim__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapClaimView.Opaque", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for swap_claim_view::Visible {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_claim.is_some() {
            len += 1;
        }
        if self.output_1.is_some() {
            len += 1;
        }
        if self.output_2.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapClaimView.Visible", len)?;
        if let Some(v) = self.swap_claim.as_ref() {
            struct_ser.serialize_field("swapClaim", v)?;
        }
        if let Some(v) = self.output_1.as_ref() {
            struct_ser.serialize_field("output1", v)?;
        }
        if let Some(v) = self.output_2.as_ref() {
            struct_ser.serialize_field("output2", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for swap_claim_view::Visible {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_claim",
            "swapClaim",
            "output_1",
            "output1",
            "output_2",
            "output2",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapClaim,
            Output1,
            Output2,
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
                            "swapClaim" | "swap_claim" => Ok(GeneratedField::SwapClaim),
                            "output1" | "output_1" => Ok(GeneratedField::Output1),
                            "output2" | "output_2" => Ok(GeneratedField::Output2),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = swap_claim_view::Visible;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapClaimView.Visible")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<swap_claim_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_claim__ = None;
                let mut output_1__ = None;
                let mut output_2__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SwapClaim => {
                            if swap_claim__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            swap_claim__ = map.next_value()?;
                        }
                        GeneratedField::Output1 => {
                            if output_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output1"));
                            }
                            output_1__ = map.next_value()?;
                        }
                        GeneratedField::Output2 => {
                            if output_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output2"));
                            }
                            output_2__ = map.next_value()?;
                        }
                    }
                }
                Ok(swap_claim_view::Visible {
                    swap_claim: swap_claim__,
                    output_1: output_1__,
                    output_2: output_2__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapClaimView.Visible", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapExecution {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.trades.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapExecution", len)?;
        if !self.trades.is_empty() {
            struct_ser.serialize_field("trades", &self.trades)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapExecution {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trades",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Trades,
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
                            "trades" => Ok(GeneratedField::Trades),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapExecution;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapExecution")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapExecution, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trades__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Trades => {
                            if trades__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trades"));
                            }
                            trades__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SwapExecution {
                    trades: trades__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapExecution", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapPayload {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.commitment.is_some() {
            len += 1;
        }
        if !self.encrypted_swap.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapPayload", len)?;
        if let Some(v) = self.commitment.as_ref() {
            struct_ser.serialize_field("commitment", v)?;
        }
        if !self.encrypted_swap.is_empty() {
            struct_ser.serialize_field("encryptedSwap", pbjson::private::base64::encode(&self.encrypted_swap).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapPayload {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commitment",
            "encrypted_swap",
            "encryptedSwap",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commitment,
            EncryptedSwap,
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
                            "commitment" => Ok(GeneratedField::Commitment),
                            "encryptedSwap" | "encrypted_swap" => Ok(GeneratedField::EncryptedSwap),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapPayload;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapPayload")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapPayload, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commitment__ = None;
                let mut encrypted_swap__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = map.next_value()?;
                        }
                        GeneratedField::EncryptedSwap => {
                            if encrypted_swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encryptedSwap"));
                            }
                            encrypted_swap__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SwapPayload {
                    commitment: commitment__,
                    encrypted_swap: encrypted_swap__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapPayload", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapPlaintext {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.trading_pair.is_some() {
            len += 1;
        }
        if self.delta_1_i.is_some() {
            len += 1;
        }
        if self.delta_2_i.is_some() {
            len += 1;
        }
        if self.claim_fee.is_some() {
            len += 1;
        }
        if self.claim_address.is_some() {
            len += 1;
        }
        if !self.rseed.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapPlaintext", len)?;
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.delta_1_i.as_ref() {
            struct_ser.serialize_field("delta1I", v)?;
        }
        if let Some(v) = self.delta_2_i.as_ref() {
            struct_ser.serialize_field("delta2I", v)?;
        }
        if let Some(v) = self.claim_fee.as_ref() {
            struct_ser.serialize_field("claimFee", v)?;
        }
        if let Some(v) = self.claim_address.as_ref() {
            struct_ser.serialize_field("claimAddress", v)?;
        }
        if !self.rseed.is_empty() {
            struct_ser.serialize_field("rseed", pbjson::private::base64::encode(&self.rseed).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapPlaintext {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trading_pair",
            "tradingPair",
            "delta_1_i",
            "delta1I",
            "delta_2_i",
            "delta2I",
            "claim_fee",
            "claimFee",
            "claim_address",
            "claimAddress",
            "rseed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TradingPair,
            Delta1I,
            Delta2I,
            ClaimFee,
            ClaimAddress,
            Rseed,
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
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "delta1I" | "delta_1_i" => Ok(GeneratedField::Delta1I),
                            "delta2I" | "delta_2_i" => Ok(GeneratedField::Delta2I),
                            "claimFee" | "claim_fee" => Ok(GeneratedField::ClaimFee),
                            "claimAddress" | "claim_address" => Ok(GeneratedField::ClaimAddress),
                            "rseed" => Ok(GeneratedField::Rseed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapPlaintext;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapPlaintext")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapPlaintext, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trading_pair__ = None;
                let mut delta_1_i__ = None;
                let mut delta_2_i__ = None;
                let mut claim_fee__ = None;
                let mut claim_address__ = None;
                let mut rseed__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map.next_value()?;
                        }
                        GeneratedField::Delta1I => {
                            if delta_1_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta1I"));
                            }
                            delta_1_i__ = map.next_value()?;
                        }
                        GeneratedField::Delta2I => {
                            if delta_2_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta2I"));
                            }
                            delta_2_i__ = map.next_value()?;
                        }
                        GeneratedField::ClaimFee => {
                            if claim_fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("claimFee"));
                            }
                            claim_fee__ = map.next_value()?;
                        }
                        GeneratedField::ClaimAddress => {
                            if claim_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("claimAddress"));
                            }
                            claim_address__ = map.next_value()?;
                        }
                        GeneratedField::Rseed => {
                            if rseed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rseed"));
                            }
                            rseed__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SwapPlaintext {
                    trading_pair: trading_pair__,
                    delta_1_i: delta_1_i__,
                    delta_2_i: delta_2_i__,
                    claim_fee: claim_fee__,
                    claim_address: claim_address__,
                    rseed: rseed__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapPlaintext", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_plaintext.is_some() {
            len += 1;
        }
        if !self.fee_blinding.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapPlan", len)?;
        if let Some(v) = self.swap_plaintext.as_ref() {
            struct_ser.serialize_field("swapPlaintext", v)?;
        }
        if !self.fee_blinding.is_empty() {
            struct_ser.serialize_field("feeBlinding", pbjson::private::base64::encode(&self.fee_blinding).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_plaintext",
            "swapPlaintext",
            "fee_blinding",
            "feeBlinding",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapPlaintext,
            FeeBlinding,
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
                            "swapPlaintext" | "swap_plaintext" => Ok(GeneratedField::SwapPlaintext),
                            "feeBlinding" | "fee_blinding" => Ok(GeneratedField::FeeBlinding),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapPlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_plaintext__ = None;
                let mut fee_blinding__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SwapPlaintext => {
                            if swap_plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapPlaintext"));
                            }
                            swap_plaintext__ = map.next_value()?;
                        }
                        GeneratedField::FeeBlinding => {
                            if fee_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeBlinding"));
                            }
                            fee_blinding__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SwapPlan {
                    swap_plaintext: swap_plaintext__,
                    fee_blinding: fee_blinding__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapView", len)?;
        if let Some(v) = self.swap_view.as_ref() {
            match v {
                swap_view::SwapView::Visible(v) => {
                    struct_ser.serialize_field("visible", v)?;
                }
                swap_view::SwapView::Opaque(v) => {
                    struct_ser.serialize_field("opaque", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapView {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapView")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_view__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if swap_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            swap_view__ = map.next_value::<::std::option::Option<_>>()?.map(swap_view::SwapView::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if swap_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            swap_view__ = map.next_value::<::std::option::Option<_>>()?.map(swap_view::SwapView::Opaque)
;
                        }
                    }
                }
                Ok(SwapView {
                    swap_view: swap_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for swap_view::Opaque {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapView.Opaque", len)?;
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for swap_view::Opaque {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Swap,
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
                            "swap" => Ok(GeneratedField::Swap),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = swap_view::Opaque;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapView.Opaque")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<swap_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = map.next_value()?;
                        }
                    }
                }
                Ok(swap_view::Opaque {
                    swap: swap__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapView.Opaque", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for swap_view::Visible {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap.is_some() {
            len += 1;
        }
        if self.swap_plaintext.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.SwapView.Visible", len)?;
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        if let Some(v) = self.swap_plaintext.as_ref() {
            struct_ser.serialize_field("swapPlaintext", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for swap_view::Visible {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap",
            "swap_plaintext",
            "swapPlaintext",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Swap,
            SwapPlaintext,
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
                            "swap" => Ok(GeneratedField::Swap),
                            "swapPlaintext" | "swap_plaintext" => Ok(GeneratedField::SwapPlaintext),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = swap_view::Visible;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.SwapView.Visible")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<swap_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap__ = None;
                let mut swap_plaintext__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = map.next_value()?;
                        }
                        GeneratedField::SwapPlaintext => {
                            if swap_plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapPlaintext"));
                            }
                            swap_plaintext__ = map.next_value()?;
                        }
                    }
                }
                Ok(swap_view::Visible {
                    swap: swap__,
                    swap_plaintext: swap_plaintext__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.SwapView.Visible", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Trade {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.path.is_some() {
            len += 1;
        }
        if self.start_amount.is_some() {
            len += 1;
        }
        if self.end_amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.Trade", len)?;
        if let Some(v) = self.path.as_ref() {
            struct_ser.serialize_field("path", v)?;
        }
        if let Some(v) = self.start_amount.as_ref() {
            struct_ser.serialize_field("startAmount", v)?;
        }
        if let Some(v) = self.end_amount.as_ref() {
            struct_ser.serialize_field("endAmount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Trade {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path",
            "start_amount",
            "startAmount",
            "end_amount",
            "endAmount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Path,
            StartAmount,
            EndAmount,
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
                            "path" => Ok(GeneratedField::Path),
                            "startAmount" | "start_amount" => Ok(GeneratedField::StartAmount),
                            "endAmount" | "end_amount" => Ok(GeneratedField::EndAmount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Trade;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.Trade")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Trade, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path__ = None;
                let mut start_amount__ = None;
                let mut end_amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = map.next_value()?;
                        }
                        GeneratedField::StartAmount => {
                            if start_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startAmount"));
                            }
                            start_amount__ = map.next_value()?;
                        }
                        GeneratedField::EndAmount => {
                            if end_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endAmount"));
                            }
                            end_amount__ = map.next_value()?;
                        }
                    }
                }
                Ok(Trade {
                    path: path__,
                    start_amount: start_amount__,
                    end_amount: end_amount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.Trade", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TradingFunction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.component.is_some() {
            len += 1;
        }
        if self.pair.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.TradingFunction", len)?;
        if let Some(v) = self.component.as_ref() {
            struct_ser.serialize_field("component", v)?;
        }
        if let Some(v) = self.pair.as_ref() {
            struct_ser.serialize_field("pair", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TradingFunction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "component",
            "pair",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Component,
            Pair,
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
                            "component" => Ok(GeneratedField::Component),
                            "pair" => Ok(GeneratedField::Pair),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TradingFunction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.TradingFunction")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TradingFunction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut component__ = None;
                let mut pair__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Component => {
                            if component__.is_some() {
                                return Err(serde::de::Error::duplicate_field("component"));
                            }
                            component__ = map.next_value()?;
                        }
                        GeneratedField::Pair => {
                            if pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pair"));
                            }
                            pair__ = map.next_value()?;
                        }
                    }
                }
                Ok(TradingFunction {
                    component: component__,
                    pair: pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.TradingFunction", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TradingPair {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.asset_1.is_some() {
            len += 1;
        }
        if self.asset_2.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.dex.v1alpha1.TradingPair", len)?;
        if let Some(v) = self.asset_1.as_ref() {
            struct_ser.serialize_field("asset1", v)?;
        }
        if let Some(v) = self.asset_2.as_ref() {
            struct_ser.serialize_field("asset2", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TradingPair {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_1",
            "asset1",
            "asset_2",
            "asset2",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Asset1,
            Asset2,
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
                            "asset1" | "asset_1" => Ok(GeneratedField::Asset1),
                            "asset2" | "asset_2" => Ok(GeneratedField::Asset2),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TradingPair;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.dex.v1alpha1.TradingPair")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TradingPair, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_1__ = None;
                let mut asset_2__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Asset1 => {
                            if asset_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asset1"));
                            }
                            asset_1__ = map.next_value()?;
                        }
                        GeneratedField::Asset2 => {
                            if asset_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asset2"));
                            }
                            asset_2__ = map.next_value()?;
                        }
                    }
                }
                Ok(TradingPair {
                    asset_1: asset_1__,
                    asset_2: asset_2__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.dex.v1alpha1.TradingPair", FIELDS, GeneratedVisitor)
    }
}
