impl serde::Serialize for ArbExecutionRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.ArbExecutionRequest", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ArbExecutionRequest {
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
                            "height" => Ok(GeneratedField::Height),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ArbExecutionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.ArbExecutionRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ArbExecutionRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ArbExecutionRequest {
                    height: height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.ArbExecutionRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ArbExecutionResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_execution.is_some() {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.ArbExecutionResponse", len)?;
        if let Some(v) = self.swap_execution.as_ref() {
            struct_ser.serialize_field("swapExecution", v)?;
        }
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ArbExecutionResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_execution",
            "swapExecution",
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapExecution,
            Height,
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
                            "swapExecution" | "swap_execution" => Ok(GeneratedField::SwapExecution),
                            "height" => Ok(GeneratedField::Height),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ArbExecutionResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.ArbExecutionResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ArbExecutionResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_execution__ = None;
                let mut height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapExecution => {
                            if swap_execution__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapExecution"));
                            }
                            swap_execution__ = map_.next_value()?;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ArbExecutionResponse {
                    swap_execution: swap_execution__,
                    height: height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.ArbExecutionResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ArbExecutionsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_height != 0 {
            len += 1;
        }
        if self.end_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.ArbExecutionsRequest", len)?;
        if self.start_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startHeight", ToString::to_string(&self.start_height).as_str())?;
        }
        if self.end_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("endHeight", ToString::to_string(&self.end_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ArbExecutionsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_height",
            "startHeight",
            "end_height",
            "endHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartHeight,
            EndHeight,
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
                            "startHeight" | "start_height" => Ok(GeneratedField::StartHeight),
                            "endHeight" | "end_height" => Ok(GeneratedField::EndHeight),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ArbExecutionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.ArbExecutionsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ArbExecutionsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_height__ = None;
                let mut end_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StartHeight => {
                            if start_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startHeight"));
                            }
                            start_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndHeight => {
                            if end_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endHeight"));
                            }
                            end_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ArbExecutionsRequest {
                    start_height: start_height__.unwrap_or_default(),
                    end_height: end_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.ArbExecutionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ArbExecutionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_execution.is_some() {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.ArbExecutionsResponse", len)?;
        if let Some(v) = self.swap_execution.as_ref() {
            struct_ser.serialize_field("swapExecution", v)?;
        }
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ArbExecutionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_execution",
            "swapExecution",
            "height",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapExecution,
            Height,
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
                            "swapExecution" | "swap_execution" => Ok(GeneratedField::SwapExecution),
                            "height" => Ok(GeneratedField::Height),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ArbExecutionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.ArbExecutionsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ArbExecutionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_execution__ = None;
                let mut height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapExecution => {
                            if swap_execution__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapExecution"));
                            }
                            swap_execution__ = map_.next_value()?;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ArbExecutionsResponse {
                    swap_execution: swap_execution__,
                    height: height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.ArbExecutionsResponse", FIELDS, GeneratedVisitor)
    }
}
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.BareTradingFunction", len)?;
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
                            "fee" => Ok(GeneratedField::Fee),
                            "p" => Ok(GeneratedField::P),
                            "q" => Ok(GeneratedField::Q),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.BareTradingFunction")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BareTradingFunction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee__ = None;
                let mut p__ = None;
                let mut q__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::P => {
                            if p__.is_some() {
                                return Err(serde::de::Error::duplicate_field("p"));
                            }
                            p__ = map_.next_value()?;
                        }
                        GeneratedField::Q => {
                            if q__.is_some() {
                                return Err(serde::de::Error::duplicate_field("q"));
                            }
                            q__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
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
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.BareTradingFunction", FIELDS, GeneratedVisitor)
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
        if self.delta_1.is_some() {
            len += 1;
        }
        if self.delta_2.is_some() {
            len += 1;
        }
        if self.lambda_1.is_some() {
            len += 1;
        }
        if self.lambda_2.is_some() {
            len += 1;
        }
        if self.unfilled_1.is_some() {
            len += 1;
        }
        if self.unfilled_2.is_some() {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.trading_pair.is_some() {
            len += 1;
        }
        if self.epoch_starting_height != 0 {
            len += 1;
        }
        if self.sct_position_prefix != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.BatchSwapOutputData", len)?;
        if let Some(v) = self.delta_1.as_ref() {
            struct_ser.serialize_field("delta1", v)?;
        }
        if let Some(v) = self.delta_2.as_ref() {
            struct_ser.serialize_field("delta2", v)?;
        }
        if let Some(v) = self.lambda_1.as_ref() {
            struct_ser.serialize_field("lambda1", v)?;
        }
        if let Some(v) = self.lambda_2.as_ref() {
            struct_ser.serialize_field("lambda2", v)?;
        }
        if let Some(v) = self.unfilled_1.as_ref() {
            struct_ser.serialize_field("unfilled1", v)?;
        }
        if let Some(v) = self.unfilled_2.as_ref() {
            struct_ser.serialize_field("unfilled2", v)?;
        }
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if self.epoch_starting_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("epochStartingHeight", ToString::to_string(&self.epoch_starting_height).as_str())?;
        }
        if self.sct_position_prefix != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("sctPositionPrefix", ToString::to_string(&self.sct_position_prefix).as_str())?;
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
            "unfilled_1",
            "unfilled1",
            "unfilled_2",
            "unfilled2",
            "height",
            "trading_pair",
            "tradingPair",
            "epoch_starting_height",
            "epochStartingHeight",
            "sct_position_prefix",
            "sctPositionPrefix",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Delta1,
            Delta2,
            Lambda1,
            Lambda2,
            Unfilled1,
            Unfilled2,
            Height,
            TradingPair,
            EpochStartingHeight,
            SctPositionPrefix,
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
                            "delta1" | "delta_1" => Ok(GeneratedField::Delta1),
                            "delta2" | "delta_2" => Ok(GeneratedField::Delta2),
                            "lambda1" | "lambda_1" => Ok(GeneratedField::Lambda1),
                            "lambda2" | "lambda_2" => Ok(GeneratedField::Lambda2),
                            "unfilled1" | "unfilled_1" => Ok(GeneratedField::Unfilled1),
                            "unfilled2" | "unfilled_2" => Ok(GeneratedField::Unfilled2),
                            "height" => Ok(GeneratedField::Height),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "epochStartingHeight" | "epoch_starting_height" => Ok(GeneratedField::EpochStartingHeight),
                            "sctPositionPrefix" | "sct_position_prefix" => Ok(GeneratedField::SctPositionPrefix),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.BatchSwapOutputData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BatchSwapOutputData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delta_1__ = None;
                let mut delta_2__ = None;
                let mut lambda_1__ = None;
                let mut lambda_2__ = None;
                let mut unfilled_1__ = None;
                let mut unfilled_2__ = None;
                let mut height__ = None;
                let mut trading_pair__ = None;
                let mut epoch_starting_height__ = None;
                let mut sct_position_prefix__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Delta1 => {
                            if delta_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta1"));
                            }
                            delta_1__ = map_.next_value()?;
                        }
                        GeneratedField::Delta2 => {
                            if delta_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta2"));
                            }
                            delta_2__ = map_.next_value()?;
                        }
                        GeneratedField::Lambda1 => {
                            if lambda_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lambda1"));
                            }
                            lambda_1__ = map_.next_value()?;
                        }
                        GeneratedField::Lambda2 => {
                            if lambda_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lambda2"));
                            }
                            lambda_2__ = map_.next_value()?;
                        }
                        GeneratedField::Unfilled1 => {
                            if unfilled_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unfilled1"));
                            }
                            unfilled_1__ = map_.next_value()?;
                        }
                        GeneratedField::Unfilled2 => {
                            if unfilled_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unfilled2"));
                            }
                            unfilled_2__ = map_.next_value()?;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::EpochStartingHeight => {
                            if epoch_starting_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochStartingHeight"));
                            }
                            epoch_starting_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SctPositionPrefix => {
                            if sct_position_prefix__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sctPositionPrefix"));
                            }
                            sct_position_prefix__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(BatchSwapOutputData {
                    delta_1: delta_1__,
                    delta_2: delta_2__,
                    lambda_1: lambda_1__,
                    lambda_2: lambda_2__,
                    unfilled_1: unfilled_1__,
                    unfilled_2: unfilled_2__,
                    height: height__.unwrap_or_default(),
                    trading_pair: trading_pair__,
                    epoch_starting_height: epoch_starting_height__.unwrap_or_default(),
                    sct_position_prefix: sct_position_prefix__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.BatchSwapOutputData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BatchSwapOutputDataRequest {
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
        if self.trading_pair.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.BatchSwapOutputDataRequest", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BatchSwapOutputDataRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "trading_pair",
            "tradingPair",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            TradingPair,
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
                            "height" => Ok(GeneratedField::Height),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BatchSwapOutputDataRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.BatchSwapOutputDataRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BatchSwapOutputDataRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut trading_pair__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(BatchSwapOutputDataRequest {
                    height: height__.unwrap_or_default(),
                    trading_pair: trading_pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.BatchSwapOutputDataRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BatchSwapOutputDataResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.BatchSwapOutputDataResponse", len)?;
        if let Some(v) = self.data.as_ref() {
            struct_ser.serialize_field("data", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BatchSwapOutputDataResponse {
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
                            "data" => Ok(GeneratedField::Data),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BatchSwapOutputDataResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.BatchSwapOutputDataResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BatchSwapOutputDataResponse, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(BatchSwapOutputDataResponse {
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.BatchSwapOutputDataResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CandlestickData {
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
        if self.open != 0. {
            len += 1;
        }
        if self.close != 0. {
            len += 1;
        }
        if self.high != 0. {
            len += 1;
        }
        if self.low != 0. {
            len += 1;
        }
        if self.direct_volume != 0. {
            len += 1;
        }
        if self.swap_volume != 0. {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.CandlestickData", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if self.open != 0. {
            struct_ser.serialize_field("open", &self.open)?;
        }
        if self.close != 0. {
            struct_ser.serialize_field("close", &self.close)?;
        }
        if self.high != 0. {
            struct_ser.serialize_field("high", &self.high)?;
        }
        if self.low != 0. {
            struct_ser.serialize_field("low", &self.low)?;
        }
        if self.direct_volume != 0. {
            struct_ser.serialize_field("directVolume", &self.direct_volume)?;
        }
        if self.swap_volume != 0. {
            struct_ser.serialize_field("swapVolume", &self.swap_volume)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CandlestickData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "open",
            "close",
            "high",
            "low",
            "direct_volume",
            "directVolume",
            "swap_volume",
            "swapVolume",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            Open,
            Close,
            High,
            Low,
            DirectVolume,
            SwapVolume,
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
                            "height" => Ok(GeneratedField::Height),
                            "open" => Ok(GeneratedField::Open),
                            "close" => Ok(GeneratedField::Close),
                            "high" => Ok(GeneratedField::High),
                            "low" => Ok(GeneratedField::Low),
                            "directVolume" | "direct_volume" => Ok(GeneratedField::DirectVolume),
                            "swapVolume" | "swap_volume" => Ok(GeneratedField::SwapVolume),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CandlestickData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.CandlestickData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CandlestickData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut open__ = None;
                let mut close__ = None;
                let mut high__ = None;
                let mut low__ = None;
                let mut direct_volume__ = None;
                let mut swap_volume__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Open => {
                            if open__.is_some() {
                                return Err(serde::de::Error::duplicate_field("open"));
                            }
                            open__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Close => {
                            if close__.is_some() {
                                return Err(serde::de::Error::duplicate_field("close"));
                            }
                            close__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::High => {
                            if high__.is_some() {
                                return Err(serde::de::Error::duplicate_field("high"));
                            }
                            high__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Low => {
                            if low__.is_some() {
                                return Err(serde::de::Error::duplicate_field("low"));
                            }
                            low__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DirectVolume => {
                            if direct_volume__.is_some() {
                                return Err(serde::de::Error::duplicate_field("directVolume"));
                            }
                            direct_volume__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SwapVolume => {
                            if swap_volume__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapVolume"));
                            }
                            swap_volume__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CandlestickData {
                    height: height__.unwrap_or_default(),
                    open: open__.unwrap_or_default(),
                    close: close__.unwrap_or_default(),
                    high: high__.unwrap_or_default(),
                    low: low__.unwrap_or_default(),
                    direct_volume: direct_volume__.unwrap_or_default(),
                    swap_volume: swap_volume__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.CandlestickData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CandlestickDataRequest {
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
        if self.limit != 0 {
            len += 1;
        }
        if self.start_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.CandlestickDataRequest", len)?;
        if let Some(v) = self.pair.as_ref() {
            struct_ser.serialize_field("pair", v)?;
        }
        if self.limit != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("limit", ToString::to_string(&self.limit).as_str())?;
        }
        if self.start_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startHeight", ToString::to_string(&self.start_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CandlestickDataRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pair",
            "limit",
            "start_height",
            "startHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pair,
            Limit,
            StartHeight,
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
                            "pair" => Ok(GeneratedField::Pair),
                            "limit" => Ok(GeneratedField::Limit),
                            "startHeight" | "start_height" => Ok(GeneratedField::StartHeight),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CandlestickDataRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.CandlestickDataRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CandlestickDataRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pair__ = None;
                let mut limit__ = None;
                let mut start_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Pair => {
                            if pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pair"));
                            }
                            pair__ = map_.next_value()?;
                        }
                        GeneratedField::Limit => {
                            if limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("limit"));
                            }
                            limit__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::StartHeight => {
                            if start_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startHeight"));
                            }
                            start_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CandlestickDataRequest {
                    pair: pair__,
                    limit: limit__.unwrap_or_default(),
                    start_height: start_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.CandlestickDataRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CandlestickDataResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.CandlestickDataResponse", len)?;
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", &self.data)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CandlestickDataResponse {
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
                            "data" => Ok(GeneratedField::Data),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CandlestickDataResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.CandlestickDataResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CandlestickDataResponse, V::Error>
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
                            data__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CandlestickDataResponse {
                    data: data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.CandlestickDataResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CandlestickDataStreamRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.CandlestickDataStreamRequest", len)?;
        if let Some(v) = self.pair.as_ref() {
            struct_ser.serialize_field("pair", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CandlestickDataStreamRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pair",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pair,
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
                            "pair" => Ok(GeneratedField::Pair),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CandlestickDataStreamRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.CandlestickDataStreamRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CandlestickDataStreamRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pair__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Pair => {
                            if pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pair"));
                            }
                            pair__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CandlestickDataStreamRequest {
                    pair: pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.CandlestickDataStreamRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CandlestickDataStreamResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.CandlestickDataStreamResponse", len)?;
        if let Some(v) = self.data.as_ref() {
            struct_ser.serialize_field("data", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CandlestickDataStreamResponse {
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
                            "data" => Ok(GeneratedField::Data),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CandlestickDataStreamResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.CandlestickDataStreamResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CandlestickDataStreamResponse, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CandlestickDataStreamResponse {
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.CandlestickDataStreamResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DexParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.is_enabled {
            len += 1;
        }
        if !self.fixed_candidates.is_empty() {
            len += 1;
        }
        if self.max_hops != 0 {
            len += 1;
        }
        if self.max_positions_per_pair != 0 {
            len += 1;
        }
        if self.max_execution_budget != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.DexParameters", len)?;
        if self.is_enabled {
            struct_ser.serialize_field("isEnabled", &self.is_enabled)?;
        }
        if !self.fixed_candidates.is_empty() {
            struct_ser.serialize_field("fixedCandidates", &self.fixed_candidates)?;
        }
        if self.max_hops != 0 {
            struct_ser.serialize_field("maxHops", &self.max_hops)?;
        }
        if self.max_positions_per_pair != 0 {
            struct_ser.serialize_field("maxPositionsPerPair", &self.max_positions_per_pair)?;
        }
        if self.max_execution_budget != 0 {
            struct_ser.serialize_field("maxExecutionBudget", &self.max_execution_budget)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DexParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "is_enabled",
            "isEnabled",
            "fixed_candidates",
            "fixedCandidates",
            "max_hops",
            "maxHops",
            "max_positions_per_pair",
            "maxPositionsPerPair",
            "max_execution_budget",
            "maxExecutionBudget",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IsEnabled,
            FixedCandidates,
            MaxHops,
            MaxPositionsPerPair,
            MaxExecutionBudget,
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
                            "isEnabled" | "is_enabled" => Ok(GeneratedField::IsEnabled),
                            "fixedCandidates" | "fixed_candidates" => Ok(GeneratedField::FixedCandidates),
                            "maxHops" | "max_hops" => Ok(GeneratedField::MaxHops),
                            "maxPositionsPerPair" | "max_positions_per_pair" => Ok(GeneratedField::MaxPositionsPerPair),
                            "maxExecutionBudget" | "max_execution_budget" => Ok(GeneratedField::MaxExecutionBudget),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DexParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.DexParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DexParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut is_enabled__ = None;
                let mut fixed_candidates__ = None;
                let mut max_hops__ = None;
                let mut max_positions_per_pair__ = None;
                let mut max_execution_budget__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IsEnabled => {
                            if is_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isEnabled"));
                            }
                            is_enabled__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FixedCandidates => {
                            if fixed_candidates__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fixedCandidates"));
                            }
                            fixed_candidates__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MaxHops => {
                            if max_hops__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxHops"));
                            }
                            max_hops__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxPositionsPerPair => {
                            if max_positions_per_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxPositionsPerPair"));
                            }
                            max_positions_per_pair__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxExecutionBudget => {
                            if max_execution_budget__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxExecutionBudget"));
                            }
                            max_execution_budget__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DexParameters {
                    is_enabled: is_enabled__.unwrap_or_default(),
                    fixed_candidates: fixed_candidates__.unwrap_or_default(),
                    max_hops: max_hops__.unwrap_or_default(),
                    max_positions_per_pair: max_positions_per_pair__.unwrap_or_default(),
                    max_execution_budget: max_execution_budget__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.DexParameters", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.DirectedTradingPair", len)?;
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
                            "start" => Ok(GeneratedField::Start),
                            "end" => Ok(GeneratedField::End),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.DirectedTradingPair")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DirectedTradingPair, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start__ = None;
                let mut end__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Start => {
                            if start__.is_some() {
                                return Err(serde::de::Error::duplicate_field("start"));
                            }
                            start__ = map_.next_value()?;
                        }
                        GeneratedField::End => {
                            if end__.is_some() {
                                return Err(serde::de::Error::duplicate_field("end"));
                            }
                            end__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DirectedTradingPair {
                    start: start__,
                    end: end__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.DirectedTradingPair", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventArbExecution {
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
        if self.swap_execution.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventArbExecution", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.swap_execution.as_ref() {
            struct_ser.serialize_field("swapExecution", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventArbExecution {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "swap_execution",
            "swapExecution",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            SwapExecution,
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
                            "height" => Ok(GeneratedField::Height),
                            "swapExecution" | "swap_execution" => Ok(GeneratedField::SwapExecution),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventArbExecution;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventArbExecution")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventArbExecution, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut swap_execution__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SwapExecution => {
                            if swap_execution__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapExecution"));
                            }
                            swap_execution__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventArbExecution {
                    height: height__.unwrap_or_default(),
                    swap_execution: swap_execution__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventArbExecution", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventBatchSwap {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.batch_swap_output_data.is_some() {
            len += 1;
        }
        if self.swap_execution_1_for_2.is_some() {
            len += 1;
        }
        if self.swap_execution_2_for_1.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventBatchSwap", len)?;
        if let Some(v) = self.batch_swap_output_data.as_ref() {
            struct_ser.serialize_field("batchSwapOutputData", v)?;
        }
        if let Some(v) = self.swap_execution_1_for_2.as_ref() {
            struct_ser.serialize_field("swapExecution1For2", v)?;
        }
        if let Some(v) = self.swap_execution_2_for_1.as_ref() {
            struct_ser.serialize_field("swapExecution2For1", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventBatchSwap {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "batch_swap_output_data",
            "batchSwapOutputData",
            "swap_execution_1_for_2",
            "swapExecution1For2",
            "swap_execution_2_for_1",
            "swapExecution2For1",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BatchSwapOutputData,
            SwapExecution1For2,
            SwapExecution2For1,
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
                            "batchSwapOutputData" | "batch_swap_output_data" => Ok(GeneratedField::BatchSwapOutputData),
                            "swapExecution1For2" | "swap_execution_1_for_2" => Ok(GeneratedField::SwapExecution1For2),
                            "swapExecution2For1" | "swap_execution_2_for_1" => Ok(GeneratedField::SwapExecution2For1),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventBatchSwap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventBatchSwap")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventBatchSwap, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut batch_swap_output_data__ = None;
                let mut swap_execution_1_for_2__ = None;
                let mut swap_execution_2_for_1__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BatchSwapOutputData => {
                            if batch_swap_output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("batchSwapOutputData"));
                            }
                            batch_swap_output_data__ = map_.next_value()?;
                        }
                        GeneratedField::SwapExecution1For2 => {
                            if swap_execution_1_for_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapExecution1For2"));
                            }
                            swap_execution_1_for_2__ = map_.next_value()?;
                        }
                        GeneratedField::SwapExecution2For1 => {
                            if swap_execution_2_for_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapExecution2For1"));
                            }
                            swap_execution_2_for_1__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventBatchSwap {
                    batch_swap_output_data: batch_swap_output_data__,
                    swap_execution_1_for_2: swap_execution_1_for_2__,
                    swap_execution_2_for_1: swap_execution_2_for_1__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventBatchSwap", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventCandlestickData {
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
        if self.stick.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventCandlestickData", len)?;
        if let Some(v) = self.pair.as_ref() {
            struct_ser.serialize_field("pair", v)?;
        }
        if let Some(v) = self.stick.as_ref() {
            struct_ser.serialize_field("stick", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventCandlestickData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pair",
            "stick",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pair,
            Stick,
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
                            "pair" => Ok(GeneratedField::Pair),
                            "stick" => Ok(GeneratedField::Stick),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventCandlestickData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventCandlestickData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventCandlestickData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pair__ = None;
                let mut stick__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Pair => {
                            if pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pair"));
                            }
                            pair__ = map_.next_value()?;
                        }
                        GeneratedField::Stick => {
                            if stick__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stick"));
                            }
                            stick__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventCandlestickData {
                    pair: pair__,
                    stick: stick__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventCandlestickData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventPositionClose {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventPositionClose", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventPositionClose {
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventPositionClose;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventPositionClose")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventPositionClose, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventPositionClose {
                    position_id: position_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventPositionClose", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventPositionExecution {
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
        if self.trading_pair.is_some() {
            len += 1;
        }
        if self.reserves_1.is_some() {
            len += 1;
        }
        if self.reserves_2.is_some() {
            len += 1;
        }
        if self.prev_reserves_1.is_some() {
            len += 1;
        }
        if self.prev_reserves_2.is_some() {
            len += 1;
        }
        if self.context.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventPositionExecution", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.reserves_1.as_ref() {
            struct_ser.serialize_field("reserves1", v)?;
        }
        if let Some(v) = self.reserves_2.as_ref() {
            struct_ser.serialize_field("reserves2", v)?;
        }
        if let Some(v) = self.prev_reserves_1.as_ref() {
            struct_ser.serialize_field("prevReserves1", v)?;
        }
        if let Some(v) = self.prev_reserves_2.as_ref() {
            struct_ser.serialize_field("prevReserves2", v)?;
        }
        if let Some(v) = self.context.as_ref() {
            struct_ser.serialize_field("context", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventPositionExecution {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position_id",
            "positionId",
            "trading_pair",
            "tradingPair",
            "reserves_1",
            "reserves1",
            "reserves_2",
            "reserves2",
            "prev_reserves_1",
            "prevReserves1",
            "prev_reserves_2",
            "prevReserves2",
            "context",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionId,
            TradingPair,
            Reserves1,
            Reserves2,
            PrevReserves1,
            PrevReserves2,
            Context,
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "reserves1" | "reserves_1" => Ok(GeneratedField::Reserves1),
                            "reserves2" | "reserves_2" => Ok(GeneratedField::Reserves2),
                            "prevReserves1" | "prev_reserves_1" => Ok(GeneratedField::PrevReserves1),
                            "prevReserves2" | "prev_reserves_2" => Ok(GeneratedField::PrevReserves2),
                            "context" => Ok(GeneratedField::Context),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventPositionExecution;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventPositionExecution")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventPositionExecution, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut trading_pair__ = None;
                let mut reserves_1__ = None;
                let mut reserves_2__ = None;
                let mut prev_reserves_1__ = None;
                let mut prev_reserves_2__ = None;
                let mut context__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Reserves1 => {
                            if reserves_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves1"));
                            }
                            reserves_1__ = map_.next_value()?;
                        }
                        GeneratedField::Reserves2 => {
                            if reserves_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves2"));
                            }
                            reserves_2__ = map_.next_value()?;
                        }
                        GeneratedField::PrevReserves1 => {
                            if prev_reserves_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prevReserves1"));
                            }
                            prev_reserves_1__ = map_.next_value()?;
                        }
                        GeneratedField::PrevReserves2 => {
                            if prev_reserves_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prevReserves2"));
                            }
                            prev_reserves_2__ = map_.next_value()?;
                        }
                        GeneratedField::Context => {
                            if context__.is_some() {
                                return Err(serde::de::Error::duplicate_field("context"));
                            }
                            context__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventPositionExecution {
                    position_id: position_id__,
                    trading_pair: trading_pair__,
                    reserves_1: reserves_1__,
                    reserves_2: reserves_2__,
                    prev_reserves_1: prev_reserves_1__,
                    prev_reserves_2: prev_reserves_2__,
                    context: context__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventPositionExecution", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventPositionOpen {
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
        if self.trading_pair.is_some() {
            len += 1;
        }
        if self.reserves_1.is_some() {
            len += 1;
        }
        if self.reserves_2.is_some() {
            len += 1;
        }
        if self.trading_fee != 0 {
            len += 1;
        }
        if self.position.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventPositionOpen", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.reserves_1.as_ref() {
            struct_ser.serialize_field("reserves1", v)?;
        }
        if let Some(v) = self.reserves_2.as_ref() {
            struct_ser.serialize_field("reserves2", v)?;
        }
        if self.trading_fee != 0 {
            struct_ser.serialize_field("tradingFee", &self.trading_fee)?;
        }
        if let Some(v) = self.position.as_ref() {
            struct_ser.serialize_field("position", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventPositionOpen {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position_id",
            "positionId",
            "trading_pair",
            "tradingPair",
            "reserves_1",
            "reserves1",
            "reserves_2",
            "reserves2",
            "trading_fee",
            "tradingFee",
            "position",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionId,
            TradingPair,
            Reserves1,
            Reserves2,
            TradingFee,
            Position,
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "reserves1" | "reserves_1" => Ok(GeneratedField::Reserves1),
                            "reserves2" | "reserves_2" => Ok(GeneratedField::Reserves2),
                            "tradingFee" | "trading_fee" => Ok(GeneratedField::TradingFee),
                            "position" => Ok(GeneratedField::Position),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventPositionOpen;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventPositionOpen")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventPositionOpen, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut trading_pair__ = None;
                let mut reserves_1__ = None;
                let mut reserves_2__ = None;
                let mut trading_fee__ = None;
                let mut position__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Reserves1 => {
                            if reserves_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves1"));
                            }
                            reserves_1__ = map_.next_value()?;
                        }
                        GeneratedField::Reserves2 => {
                            if reserves_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves2"));
                            }
                            reserves_2__ = map_.next_value()?;
                        }
                        GeneratedField::TradingFee => {
                            if trading_fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingFee"));
                            }
                            trading_fee__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventPositionOpen {
                    position_id: position_id__,
                    trading_pair: trading_pair__,
                    reserves_1: reserves_1__,
                    reserves_2: reserves_2__,
                    trading_fee: trading_fee__.unwrap_or_default(),
                    position: position__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventPositionOpen", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventPositionWithdraw {
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
        if self.trading_pair.is_some() {
            len += 1;
        }
        if self.reserves_1.is_some() {
            len += 1;
        }
        if self.reserves_2.is_some() {
            len += 1;
        }
        if self.sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventPositionWithdraw", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.reserves_1.as_ref() {
            struct_ser.serialize_field("reserves1", v)?;
        }
        if let Some(v) = self.reserves_2.as_ref() {
            struct_ser.serialize_field("reserves2", v)?;
        }
        if self.sequence != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventPositionWithdraw {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position_id",
            "positionId",
            "trading_pair",
            "tradingPair",
            "reserves_1",
            "reserves1",
            "reserves_2",
            "reserves2",
            "sequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionId,
            TradingPair,
            Reserves1,
            Reserves2,
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "reserves1" | "reserves_1" => Ok(GeneratedField::Reserves1),
                            "reserves2" | "reserves_2" => Ok(GeneratedField::Reserves2),
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
            type Value = EventPositionWithdraw;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventPositionWithdraw")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventPositionWithdraw, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut trading_pair__ = None;
                let mut reserves_1__ = None;
                let mut reserves_2__ = None;
                let mut sequence__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Reserves1 => {
                            if reserves_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves1"));
                            }
                            reserves_1__ = map_.next_value()?;
                        }
                        GeneratedField::Reserves2 => {
                            if reserves_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves2"));
                            }
                            reserves_2__ = map_.next_value()?;
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
                Ok(EventPositionWithdraw {
                    position_id: position_id__,
                    trading_pair: trading_pair__,
                    reserves_1: reserves_1__,
                    reserves_2: reserves_2__,
                    sequence: sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventPositionWithdraw", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventQueuePositionClose {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventQueuePositionClose", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventQueuePositionClose {
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventQueuePositionClose;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventQueuePositionClose")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventQueuePositionClose, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventQueuePositionClose {
                    position_id: position_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventQueuePositionClose", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventSwap {
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
        if self.swap_commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventSwap", len)?;
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.delta_1_i.as_ref() {
            struct_ser.serialize_field("delta1I", v)?;
        }
        if let Some(v) = self.delta_2_i.as_ref() {
            struct_ser.serialize_field("delta2I", v)?;
        }
        if let Some(v) = self.swap_commitment.as_ref() {
            struct_ser.serialize_field("swapCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventSwap {
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
            "swap_commitment",
            "swapCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TradingPair,
            Delta1I,
            Delta2I,
            SwapCommitment,
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
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "delta1I" | "delta_1_i" => Ok(GeneratedField::Delta1I),
                            "delta2I" | "delta_2_i" => Ok(GeneratedField::Delta2I),
                            "swapCommitment" | "swap_commitment" => Ok(GeneratedField::SwapCommitment),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventSwap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventSwap")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventSwap, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trading_pair__ = None;
                let mut delta_1_i__ = None;
                let mut delta_2_i__ = None;
                let mut swap_commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Delta1I => {
                            if delta_1_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta1I"));
                            }
                            delta_1_i__ = map_.next_value()?;
                        }
                        GeneratedField::Delta2I => {
                            if delta_2_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta2I"));
                            }
                            delta_2_i__ = map_.next_value()?;
                        }
                        GeneratedField::SwapCommitment => {
                            if swap_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapCommitment"));
                            }
                            swap_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventSwap {
                    trading_pair: trading_pair__,
                    delta_1_i: delta_1_i__,
                    delta_2_i: delta_2_i__,
                    swap_commitment: swap_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventSwap", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventSwapClaim {
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
        if self.output_1_commitment.is_some() {
            len += 1;
        }
        if self.output_2_commitment.is_some() {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventSwapClaim", len)?;
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.output_1_commitment.as_ref() {
            struct_ser.serialize_field("output1Commitment", v)?;
        }
        if let Some(v) = self.output_2_commitment.as_ref() {
            struct_ser.serialize_field("output2Commitment", v)?;
        }
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventSwapClaim {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trading_pair",
            "tradingPair",
            "output_1_commitment",
            "output1Commitment",
            "output_2_commitment",
            "output2Commitment",
            "nullifier",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TradingPair,
            Output1Commitment,
            Output2Commitment,
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
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "output1Commitment" | "output_1_commitment" => Ok(GeneratedField::Output1Commitment),
                            "output2Commitment" | "output_2_commitment" => Ok(GeneratedField::Output2Commitment),
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
            type Value = EventSwapClaim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventSwapClaim")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventSwapClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trading_pair__ = None;
                let mut output_1_commitment__ = None;
                let mut output_2_commitment__ = None;
                let mut nullifier__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Output1Commitment => {
                            if output_1_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output1Commitment"));
                            }
                            output_1_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::Output2Commitment => {
                            if output_2_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output2Commitment"));
                            }
                            output_2_commitment__ = map_.next_value()?;
                        }
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
                Ok(EventSwapClaim {
                    trading_pair: trading_pair__,
                    output_1_commitment: output_1_commitment__,
                    output_2_commitment: output_2_commitment__,
                    nullifier: nullifier__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventSwapClaim", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventValueCircuitBreakerCredit {
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
        if self.previous_balance.is_some() {
            len += 1;
        }
        if self.new_balance.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventValueCircuitBreakerCredit", len)?;
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        if let Some(v) = self.previous_balance.as_ref() {
            struct_ser.serialize_field("previousBalance", v)?;
        }
        if let Some(v) = self.new_balance.as_ref() {
            struct_ser.serialize_field("newBalance", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventValueCircuitBreakerCredit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_id",
            "assetId",
            "previous_balance",
            "previousBalance",
            "new_balance",
            "newBalance",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssetId,
            PreviousBalance,
            NewBalance,
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
                            "previousBalance" | "previous_balance" => Ok(GeneratedField::PreviousBalance),
                            "newBalance" | "new_balance" => Ok(GeneratedField::NewBalance),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventValueCircuitBreakerCredit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventValueCircuitBreakerCredit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventValueCircuitBreakerCredit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_id__ = None;
                let mut previous_balance__ = None;
                let mut new_balance__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::PreviousBalance => {
                            if previous_balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("previousBalance"));
                            }
                            previous_balance__ = map_.next_value()?;
                        }
                        GeneratedField::NewBalance => {
                            if new_balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newBalance"));
                            }
                            new_balance__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventValueCircuitBreakerCredit {
                    asset_id: asset_id__,
                    previous_balance: previous_balance__,
                    new_balance: new_balance__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventValueCircuitBreakerCredit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventValueCircuitBreakerDebit {
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
        if self.previous_balance.is_some() {
            len += 1;
        }
        if self.new_balance.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.EventValueCircuitBreakerDebit", len)?;
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        if let Some(v) = self.previous_balance.as_ref() {
            struct_ser.serialize_field("previousBalance", v)?;
        }
        if let Some(v) = self.new_balance.as_ref() {
            struct_ser.serialize_field("newBalance", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventValueCircuitBreakerDebit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_id",
            "assetId",
            "previous_balance",
            "previousBalance",
            "new_balance",
            "newBalance",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssetId,
            PreviousBalance,
            NewBalance,
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
                            "previousBalance" | "previous_balance" => Ok(GeneratedField::PreviousBalance),
                            "newBalance" | "new_balance" => Ok(GeneratedField::NewBalance),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventValueCircuitBreakerDebit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.EventValueCircuitBreakerDebit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventValueCircuitBreakerDebit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_id__ = None;
                let mut previous_balance__ = None;
                let mut new_balance__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::PreviousBalance => {
                            if previous_balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("previousBalance"));
                            }
                            previous_balance__ = map_.next_value()?;
                        }
                        GeneratedField::NewBalance => {
                            if new_balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newBalance"));
                            }
                            new_balance__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventValueCircuitBreakerDebit {
                    asset_id: asset_id__,
                    previous_balance: previous_balance__,
                    new_balance: new_balance__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.EventValueCircuitBreakerDebit", FIELDS, GeneratedVisitor)
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
        if self.dex_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.GenesisContent", len)?;
        if let Some(v) = self.dex_params.as_ref() {
            struct_ser.serialize_field("dexParams", v)?;
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
            "dex_params",
            "dexParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DexParams,
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
                            "dexParams" | "dex_params" => Ok(GeneratedField::DexParams),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut dex_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DexParams => {
                            if dex_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dexParams"));
                            }
                            dex_params__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    dex_params: dex_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LiquidityPositionByIdRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.LiquidityPositionByIdRequest", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LiquidityPositionByIdRequest {
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LiquidityPositionByIdRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.LiquidityPositionByIdRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LiquidityPositionByIdRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LiquidityPositionByIdRequest {
                    position_id: position_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.LiquidityPositionByIdRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LiquidityPositionByIdResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.LiquidityPositionByIdResponse", len)?;
        if let Some(v) = self.data.as_ref() {
            struct_ser.serialize_field("data", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LiquidityPositionByIdResponse {
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
                            "data" => Ok(GeneratedField::Data),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LiquidityPositionByIdResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.LiquidityPositionByIdResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LiquidityPositionByIdResponse, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LiquidityPositionByIdResponse {
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.LiquidityPositionByIdResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LiquidityPositionsByIdRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.position_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsByIdRequest", len)?;
        if !self.position_id.is_empty() {
            struct_ser.serialize_field("positionId", &self.position_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LiquidityPositionsByIdRequest {
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LiquidityPositionsByIdRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.LiquidityPositionsByIdRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LiquidityPositionsByIdRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LiquidityPositionsByIdRequest {
                    position_id: position_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsByIdRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LiquidityPositionsByIdResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsByIdResponse", len)?;
        if let Some(v) = self.data.as_ref() {
            struct_ser.serialize_field("data", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LiquidityPositionsByIdResponse {
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
                            "data" => Ok(GeneratedField::Data),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LiquidityPositionsByIdResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.LiquidityPositionsByIdResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LiquidityPositionsByIdResponse, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LiquidityPositionsByIdResponse {
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsByIdResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LiquidityPositionsByPriceRequest {
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
        if self.limit != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsByPriceRequest", len)?;
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if self.limit != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("limit", ToString::to_string(&self.limit).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LiquidityPositionsByPriceRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trading_pair",
            "tradingPair",
            "limit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TradingPair,
            Limit,
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
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "limit" => Ok(GeneratedField::Limit),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LiquidityPositionsByPriceRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.LiquidityPositionsByPriceRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LiquidityPositionsByPriceRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trading_pair__ = None;
                let mut limit__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Limit => {
                            if limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("limit"));
                            }
                            limit__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LiquidityPositionsByPriceRequest {
                    trading_pair: trading_pair__,
                    limit: limit__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsByPriceRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LiquidityPositionsByPriceResponse {
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
        if self.id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsByPriceResponse", len)?;
        if let Some(v) = self.data.as_ref() {
            struct_ser.serialize_field("data", v)?;
        }
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LiquidityPositionsByPriceResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data",
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Data,
            Id,
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
                            "data" => Ok(GeneratedField::Data),
                            "id" => Ok(GeneratedField::Id),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LiquidityPositionsByPriceResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.LiquidityPositionsByPriceResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LiquidityPositionsByPriceResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data__ = None;
                let mut id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = map_.next_value()?;
                        }
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LiquidityPositionsByPriceResponse {
                    data: data__,
                    id: id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsByPriceResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LiquidityPositionsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.include_closed {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsRequest", len)?;
        if self.include_closed {
            struct_ser.serialize_field("includeClosed", &self.include_closed)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LiquidityPositionsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "include_closed",
            "includeClosed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IncludeClosed,
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
                            "includeClosed" | "include_closed" => Ok(GeneratedField::IncludeClosed),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LiquidityPositionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.LiquidityPositionsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LiquidityPositionsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut include_closed__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IncludeClosed => {
                            if include_closed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeClosed"));
                            }
                            include_closed__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LiquidityPositionsRequest {
                    include_closed: include_closed__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LiquidityPositionsResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsResponse", len)?;
        if let Some(v) = self.data.as_ref() {
            struct_ser.serialize_field("data", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LiquidityPositionsResponse {
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
                            "data" => Ok(GeneratedField::Data),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LiquidityPositionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.LiquidityPositionsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LiquidityPositionsResponse, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LiquidityPositionsResponse {
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.LiquidityPositionsResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.LpNft", len)?;
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "state" => Ok(GeneratedField::State),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.LpNft")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LpNft, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LpNft {
                    position_id: position_id__,
                    state: state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.LpNft", FIELDS, GeneratedVisitor)
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
        if self.state.is_some() {
            len += 1;
        }
        if self.reserves.is_some() {
            len += 1;
        }
        if self.close_on_fill {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.Position", len)?;
        if let Some(v) = self.phi.as_ref() {
            struct_ser.serialize_field("phi", v)?;
        }
        if !self.nonce.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("nonce", pbjson::private::base64::encode(&self.nonce).as_str())?;
        }
        if let Some(v) = self.state.as_ref() {
            struct_ser.serialize_field("state", v)?;
        }
        if let Some(v) = self.reserves.as_ref() {
            struct_ser.serialize_field("reserves", v)?;
        }
        if self.close_on_fill {
            struct_ser.serialize_field("closeOnFill", &self.close_on_fill)?;
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
            "state",
            "reserves",
            "close_on_fill",
            "closeOnFill",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Phi,
            Nonce,
            State,
            Reserves,
            CloseOnFill,
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
                            "phi" => Ok(GeneratedField::Phi),
                            "nonce" => Ok(GeneratedField::Nonce),
                            "state" => Ok(GeneratedField::State),
                            "reserves" => Ok(GeneratedField::Reserves),
                            "closeOnFill" | "close_on_fill" => Ok(GeneratedField::CloseOnFill),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.Position")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Position, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut phi__ = None;
                let mut nonce__ = None;
                let mut state__ = None;
                let mut reserves__ = None;
                let mut close_on_fill__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Phi => {
                            if phi__.is_some() {
                                return Err(serde::de::Error::duplicate_field("phi"));
                            }
                            phi__ = map_.next_value()?;
                        }
                        GeneratedField::Nonce => {
                            if nonce__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nonce"));
                            }
                            nonce__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map_.next_value()?;
                        }
                        GeneratedField::Reserves => {
                            if reserves__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves"));
                            }
                            reserves__ = map_.next_value()?;
                        }
                        GeneratedField::CloseOnFill => {
                            if close_on_fill__.is_some() {
                                return Err(serde::de::Error::duplicate_field("closeOnFill"));
                            }
                            close_on_fill__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Position {
                    phi: phi__,
                    nonce: nonce__.unwrap_or_default(),
                    state: state__,
                    reserves: reserves__,
                    close_on_fill: close_on_fill__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.Position", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.PositionClose", len)?;
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.PositionClose")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PositionClose, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PositionClose {
                    position_id: position_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.PositionClose", FIELDS, GeneratedVisitor)
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
        if !self.alt_bech32m.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.PositionId", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        if !self.alt_bech32m.is_empty() {
            struct_ser.serialize_field("altBech32m", &self.alt_bech32m)?;
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
            "alt_bech32m",
            "altBech32m",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Inner,
            AltBech32m,
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
                            "altBech32m" | "alt_bech32m" => Ok(GeneratedField::AltBech32m),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.PositionId")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PositionId, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                let mut alt_bech32m__ = None;
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
                        GeneratedField::AltBech32m => {
                            if alt_bech32m__.is_some() {
                                return Err(serde::de::Error::duplicate_field("altBech32m"));
                            }
                            alt_bech32m__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PositionId {
                    inner: inner__.unwrap_or_default(),
                    alt_bech32m: alt_bech32m__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.PositionId", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.PositionOpen", len)?;
        if let Some(v) = self.position.as_ref() {
            struct_ser.serialize_field("position", v)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Position,
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
                            "position" => Ok(GeneratedField::Position),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.PositionOpen")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PositionOpen, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PositionOpen {
                    position: position__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.PositionOpen", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.PositionRewardClaim", len)?;
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "rewardsCommitment" | "rewards_commitment" => Ok(GeneratedField::RewardsCommitment),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.PositionRewardClaim")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PositionRewardClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut rewards_commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::RewardsCommitment => {
                            if rewards_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewardsCommitment"));
                            }
                            rewards_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PositionRewardClaim {
                    position_id: position_id__,
                    rewards_commitment: rewards_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.PositionRewardClaim", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionRewardClaimPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.reserves.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.PositionRewardClaimPlan", len)?;
        if let Some(v) = self.reserves.as_ref() {
            struct_ser.serialize_field("reserves", v)?;
        }
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
            "reserves",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Reserves,
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
                            "reserves" => Ok(GeneratedField::Reserves),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PositionRewardClaimPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.PositionRewardClaimPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PositionRewardClaimPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut reserves__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Reserves => {
                            if reserves__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves"));
                            }
                            reserves__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PositionRewardClaimPlan {
                    reserves: reserves__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.PositionRewardClaimPlan", FIELDS, GeneratedVisitor)
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
        if self.sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.PositionState", len)?;
        if self.state != 0 {
            let v = position_state::PositionStateEnum::try_from(self.state)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.state)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        if self.sequence != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
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
            "sequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            State,
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
                            "state" => Ok(GeneratedField::State),
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
            type Value = PositionState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.PositionState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PositionState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state__ = None;
                let mut sequence__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = Some(map_.next_value::<position_state::PositionStateEnum>()? as i32);
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
                Ok(PositionState {
                    state: state__.unwrap_or_default(),
                    sequence: sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.PositionState", FIELDS, GeneratedVisitor)
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
        if self.sequence != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.PositionWithdraw", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.reserves_commitment.as_ref() {
            struct_ser.serialize_field("reservesCommitment", v)?;
        }
        if self.sequence != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
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
            "sequence",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionId,
            ReservesCommitment,
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "reservesCommitment" | "reserves_commitment" => Ok(GeneratedField::ReservesCommitment),
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
            type Value = PositionWithdraw;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.PositionWithdraw")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PositionWithdraw, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut reserves_commitment__ = None;
                let mut sequence__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::ReservesCommitment => {
                            if reserves_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reservesCommitment"));
                            }
                            reserves_commitment__ = map_.next_value()?;
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
                Ok(PositionWithdraw {
                    position_id: position_id__,
                    reserves_commitment: reserves_commitment__,
                    sequence: sequence__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.PositionWithdraw", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionWithdrawPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.reserves.is_some() {
            len += 1;
        }
        if self.position_id.is_some() {
            len += 1;
        }
        if self.pair.is_some() {
            len += 1;
        }
        if self.sequence != 0 {
            len += 1;
        }
        if !self.rewards.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.PositionWithdrawPlan", len)?;
        if let Some(v) = self.reserves.as_ref() {
            struct_ser.serialize_field("reserves", v)?;
        }
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.pair.as_ref() {
            struct_ser.serialize_field("pair", v)?;
        }
        if self.sequence != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("sequence", ToString::to_string(&self.sequence).as_str())?;
        }
        if !self.rewards.is_empty() {
            struct_ser.serialize_field("rewards", &self.rewards)?;
        }
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
            "reserves",
            "position_id",
            "positionId",
            "pair",
            "sequence",
            "rewards",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Reserves,
            PositionId,
            Pair,
            Sequence,
            Rewards,
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
                            "reserves" => Ok(GeneratedField::Reserves),
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "pair" => Ok(GeneratedField::Pair),
                            "sequence" => Ok(GeneratedField::Sequence),
                            "rewards" => Ok(GeneratedField::Rewards),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PositionWithdrawPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.PositionWithdrawPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PositionWithdrawPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut reserves__ = None;
                let mut position_id__ = None;
                let mut pair__ = None;
                let mut sequence__ = None;
                let mut rewards__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Reserves => {
                            if reserves__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves"));
                            }
                            reserves__ = map_.next_value()?;
                        }
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::Pair => {
                            if pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pair"));
                            }
                            pair__ = map_.next_value()?;
                        }
                        GeneratedField::Sequence => {
                            if sequence__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sequence"));
                            }
                            sequence__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Rewards => {
                            if rewards__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewards"));
                            }
                            rewards__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PositionWithdrawPlan {
                    reserves: reserves__,
                    position_id: position_id__,
                    pair: pair__,
                    sequence: sequence__.unwrap_or_default(),
                    rewards: rewards__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.PositionWithdrawPlan", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.Reserves", len)?;
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
                            "r1" => Ok(GeneratedField::R1),
                            "r2" => Ok(GeneratedField::R2),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.Reserves")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Reserves, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r1__ = None;
                let mut r2__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::R1 => {
                            if r1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("r1"));
                            }
                            r1__ = map_.next_value()?;
                        }
                        GeneratedField::R2 => {
                            if r2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("r2"));
                            }
                            r2__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Reserves {
                    r1: r1__,
                    r2: r2__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.Reserves", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SimulateTradeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.input.is_some() {
            len += 1;
        }
        if self.output.is_some() {
            len += 1;
        }
        if self.routing.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SimulateTradeRequest", len)?;
        if let Some(v) = self.input.as_ref() {
            struct_ser.serialize_field("input", v)?;
        }
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        if let Some(v) = self.routing.as_ref() {
            struct_ser.serialize_field("routing", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SimulateTradeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "input",
            "output",
            "routing",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Input,
            Output,
            Routing,
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
                            "input" => Ok(GeneratedField::Input),
                            "output" => Ok(GeneratedField::Output),
                            "routing" => Ok(GeneratedField::Routing),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SimulateTradeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SimulateTradeRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SimulateTradeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut input__ = None;
                let mut output__ = None;
                let mut routing__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Input => {
                            if input__.is_some() {
                                return Err(serde::de::Error::duplicate_field("input"));
                            }
                            input__ = map_.next_value()?;
                        }
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = map_.next_value()?;
                        }
                        GeneratedField::Routing => {
                            if routing__.is_some() {
                                return Err(serde::de::Error::duplicate_field("routing"));
                            }
                            routing__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SimulateTradeRequest {
                    input: input__,
                    output: output__,
                    routing: routing__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SimulateTradeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for simulate_trade_request::Routing {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.setting.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SimulateTradeRequest.Routing", len)?;
        if let Some(v) = self.setting.as_ref() {
            match v {
                simulate_trade_request::routing::Setting::Default(v) => {
                    struct_ser.serialize_field("default", v)?;
                }
                simulate_trade_request::routing::Setting::SingleHop(v) => {
                    struct_ser.serialize_field("singleHop", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for simulate_trade_request::Routing {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "default",
            "single_hop",
            "singleHop",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Default,
            SingleHop,
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
                            "default" => Ok(GeneratedField::Default),
                            "singleHop" | "single_hop" => Ok(GeneratedField::SingleHop),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = simulate_trade_request::Routing;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SimulateTradeRequest.Routing")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<simulate_trade_request::Routing, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut setting__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Default => {
                            if setting__.is_some() {
                                return Err(serde::de::Error::duplicate_field("default"));
                            }
                            setting__ = map_.next_value::<::std::option::Option<_>>()?.map(simulate_trade_request::routing::Setting::Default)
;
                        }
                        GeneratedField::SingleHop => {
                            if setting__.is_some() {
                                return Err(serde::de::Error::duplicate_field("singleHop"));
                            }
                            setting__ = map_.next_value::<::std::option::Option<_>>()?.map(simulate_trade_request::routing::Setting::SingleHop)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(simulate_trade_request::Routing {
                    setting: setting__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SimulateTradeRequest.Routing", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for simulate_trade_request::routing::Default {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SimulateTradeRequest.Routing.Default", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for simulate_trade_request::routing::Default {
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
            type Value = simulate_trade_request::routing::Default;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SimulateTradeRequest.Routing.Default")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<simulate_trade_request::routing::Default, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(simulate_trade_request::routing::Default {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SimulateTradeRequest.Routing.Default", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for simulate_trade_request::routing::SingleHop {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SimulateTradeRequest.Routing.SingleHop", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for simulate_trade_request::routing::SingleHop {
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
            type Value = simulate_trade_request::routing::SingleHop;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SimulateTradeRequest.Routing.SingleHop")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<simulate_trade_request::routing::SingleHop, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(simulate_trade_request::routing::SingleHop {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SimulateTradeRequest.Routing.SingleHop", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SimulateTradeResponse {
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
        if self.unfilled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SimulateTradeResponse", len)?;
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        if let Some(v) = self.unfilled.as_ref() {
            struct_ser.serialize_field("unfilled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SimulateTradeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "output",
            "unfilled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Output,
            Unfilled,
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
                            "unfilled" => Ok(GeneratedField::Unfilled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SimulateTradeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SimulateTradeResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SimulateTradeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut output__ = None;
                let mut unfilled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = map_.next_value()?;
                        }
                        GeneratedField::Unfilled => {
                            if unfilled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unfilled"));
                            }
                            unfilled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SimulateTradeResponse {
                    output: output__,
                    unfilled: unfilled__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SimulateTradeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpreadRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SpreadRequest", len)?;
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpreadRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trading_pair",
            "tradingPair",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TradingPair,
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
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpreadRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SpreadRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SpreadRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trading_pair__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SpreadRequest {
                    trading_pair: trading_pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SpreadRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpreadResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.best_1_to_2_position.is_some() {
            len += 1;
        }
        if self.best_2_to_1_position.is_some() {
            len += 1;
        }
        if self.approx_effective_price_1_to_2 != 0. {
            len += 1;
        }
        if self.approx_effective_price_2_to_1 != 0. {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SpreadResponse", len)?;
        if let Some(v) = self.best_1_to_2_position.as_ref() {
            struct_ser.serialize_field("best1To2Position", v)?;
        }
        if let Some(v) = self.best_2_to_1_position.as_ref() {
            struct_ser.serialize_field("best2To1Position", v)?;
        }
        if self.approx_effective_price_1_to_2 != 0. {
            struct_ser.serialize_field("approxEffectivePrice1To2", &self.approx_effective_price_1_to_2)?;
        }
        if self.approx_effective_price_2_to_1 != 0. {
            struct_ser.serialize_field("approxEffectivePrice2To1", &self.approx_effective_price_2_to_1)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpreadResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "best_1_to_2_position",
            "best1To2Position",
            "best_2_to_1_position",
            "best2To1Position",
            "approx_effective_price_1_to_2",
            "approxEffectivePrice1To2",
            "approx_effective_price_2_to_1",
            "approxEffectivePrice2To1",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Best1To2Position,
            Best2To1Position,
            ApproxEffectivePrice1To2,
            ApproxEffectivePrice2To1,
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
                            "best1To2Position" | "best_1_to_2_position" => Ok(GeneratedField::Best1To2Position),
                            "best2To1Position" | "best_2_to_1_position" => Ok(GeneratedField::Best2To1Position),
                            "approxEffectivePrice1To2" | "approx_effective_price_1_to_2" => Ok(GeneratedField::ApproxEffectivePrice1To2),
                            "approxEffectivePrice2To1" | "approx_effective_price_2_to_1" => Ok(GeneratedField::ApproxEffectivePrice2To1),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpreadResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SpreadResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SpreadResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut best_1_to_2_position__ = None;
                let mut best_2_to_1_position__ = None;
                let mut approx_effective_price_1_to_2__ = None;
                let mut approx_effective_price_2_to_1__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Best1To2Position => {
                            if best_1_to_2_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("best1To2Position"));
                            }
                            best_1_to_2_position__ = map_.next_value()?;
                        }
                        GeneratedField::Best2To1Position => {
                            if best_2_to_1_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("best2To1Position"));
                            }
                            best_2_to_1_position__ = map_.next_value()?;
                        }
                        GeneratedField::ApproxEffectivePrice1To2 => {
                            if approx_effective_price_1_to_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("approxEffectivePrice1To2"));
                            }
                            approx_effective_price_1_to_2__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ApproxEffectivePrice2To1 => {
                            if approx_effective_price_2_to_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("approxEffectivePrice2To1"));
                            }
                            approx_effective_price_2_to_1__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SpreadResponse {
                    best_1_to_2_position: best_1_to_2_position__,
                    best_2_to_1_position: best_2_to_1_position__,
                    approx_effective_price_1_to_2: approx_effective_price_1_to_2__.unwrap_or_default(),
                    approx_effective_price_2_to_1: approx_effective_price_2_to_1__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SpreadResponse", FIELDS, GeneratedVisitor)
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
        if self.proof.is_some() {
            len += 1;
        }
        if self.body.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.Swap", len)?;
        if let Some(v) = self.proof.as_ref() {
            struct_ser.serialize_field("proof", v)?;
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
                            "proof" => Ok(GeneratedField::Proof),
                            "body" => Ok(GeneratedField::Body),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.Swap")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Swap, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proof__ = None;
                let mut body__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = map_.next_value()?;
                        }
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Swap {
                    proof: proof__,
                    body: body__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.Swap", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapBody", len)?;
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
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "delta1I" | "delta_1_i" => Ok(GeneratedField::Delta1I),
                            "delta2I" | "delta_2_i" => Ok(GeneratedField::Delta2I),
                            "feeCommitment" | "fee_commitment" => Ok(GeneratedField::FeeCommitment),
                            "payload" => Ok(GeneratedField::Payload),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapBody")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trading_pair__ = None;
                let mut delta_1_i__ = None;
                let mut delta_2_i__ = None;
                let mut fee_commitment__ = None;
                let mut payload__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Delta1I => {
                            if delta_1_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta1I"));
                            }
                            delta_1_i__ = map_.next_value()?;
                        }
                        GeneratedField::Delta2I => {
                            if delta_2_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta2I"));
                            }
                            delta_2_i__ = map_.next_value()?;
                        }
                        GeneratedField::FeeCommitment => {
                            if fee_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeCommitment"));
                            }
                            fee_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::Payload => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payload"));
                            }
                            payload__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
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
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapBody", FIELDS, GeneratedVisitor)
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
        if self.proof.is_some() {
            len += 1;
        }
        if self.body.is_some() {
            len += 1;
        }
        if self.epoch_duration != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapClaim", len)?;
        if let Some(v) = self.proof.as_ref() {
            struct_ser.serialize_field("proof", v)?;
        }
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if self.epoch_duration != 0 {
            #[allow(clippy::needless_borrow)]
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
                            "proof" => Ok(GeneratedField::Proof),
                            "body" => Ok(GeneratedField::Body),
                            "epochDuration" | "epoch_duration" => Ok(GeneratedField::EpochDuration),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapClaim")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proof__ = None;
                let mut body__ = None;
                let mut epoch_duration__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = map_.next_value()?;
                        }
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map_.next_value()?;
                        }
                        GeneratedField::EpochDuration => {
                            if epoch_duration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochDuration"));
                            }
                            epoch_duration__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapClaim {
                    proof: proof__,
                    body: body__,
                    epoch_duration: epoch_duration__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapClaim", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapClaimBody", len)?;
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
                            "fee" => Ok(GeneratedField::Fee),
                            "output1Commitment" | "output_1_commitment" => Ok(GeneratedField::Output1Commitment),
                            "output2Commitment" | "output_2_commitment" => Ok(GeneratedField::Output2Commitment),
                            "outputData" | "output_data" => Ok(GeneratedField::OutputData),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapClaimBody")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapClaimBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut nullifier__ = None;
                let mut fee__ = None;
                let mut output_1_commitment__ = None;
                let mut output_2_commitment__ = None;
                let mut output_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map_.next_value()?;
                        }
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map_.next_value()?;
                        }
                        GeneratedField::Output1Commitment => {
                            if output_1_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output1Commitment"));
                            }
                            output_1_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::Output2Commitment => {
                            if output_2_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output2Commitment"));
                            }
                            output_2_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::OutputData => {
                            if output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputData"));
                            }
                            output_data__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
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
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapClaimBody", FIELDS, GeneratedVisitor)
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
        if !self.proof_blinding_r.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_s.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapClaimPlan", len)?;
        if let Some(v) = self.swap_plaintext.as_ref() {
            struct_ser.serialize_field("swapPlaintext", v)?;
        }
        if self.position != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("position", ToString::to_string(&self.position).as_str())?;
        }
        if let Some(v) = self.output_data.as_ref() {
            struct_ser.serialize_field("outputData", v)?;
        }
        if self.epoch_duration != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("epochDuration", ToString::to_string(&self.epoch_duration).as_str())?;
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
            "proof_blinding_r",
            "proofBlindingR",
            "proof_blinding_s",
            "proofBlindingS",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapPlaintext,
            Position,
            OutputData,
            EpochDuration,
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
                            "swapPlaintext" | "swap_plaintext" => Ok(GeneratedField::SwapPlaintext),
                            "position" => Ok(GeneratedField::Position),
                            "outputData" | "output_data" => Ok(GeneratedField::OutputData),
                            "epochDuration" | "epoch_duration" => Ok(GeneratedField::EpochDuration),
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
            type Value = SwapClaimPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapClaimPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapClaimPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_plaintext__ = None;
                let mut position__ = None;
                let mut output_data__ = None;
                let mut epoch_duration__ = None;
                let mut proof_blinding_r__ = None;
                let mut proof_blinding_s__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapPlaintext => {
                            if swap_plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapPlaintext"));
                            }
                            swap_plaintext__ = map_.next_value()?;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::OutputData => {
                            if output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputData"));
                            }
                            output_data__ = map_.next_value()?;
                        }
                        GeneratedField::EpochDuration => {
                            if epoch_duration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochDuration"));
                            }
                            epoch_duration__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
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
                Ok(SwapClaimPlan {
                    swap_plaintext: swap_plaintext__,
                    position: position__.unwrap_or_default(),
                    output_data: output_data__,
                    epoch_duration: epoch_duration__.unwrap_or_default(),
                    proof_blinding_r: proof_blinding_r__.unwrap_or_default(),
                    proof_blinding_s: proof_blinding_s__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapClaimPlan", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapClaimView", len)?;
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
            type Value = SwapClaimView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapClaimView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapClaimView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_claim_view__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if swap_claim_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            swap_claim_view__ = map_.next_value::<::std::option::Option<_>>()?.map(swap_claim_view::SwapClaimView::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if swap_claim_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            swap_claim_view__ = map_.next_value::<::std::option::Option<_>>()?.map(swap_claim_view::SwapClaimView::Opaque)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapClaimView {
                    swap_claim_view: swap_claim_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapClaimView", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapClaimView.Opaque", len)?;
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
                            "swapClaim" | "swap_claim" => Ok(GeneratedField::SwapClaim),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapClaimView.Opaque")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<swap_claim_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_claim__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapClaim => {
                            if swap_claim__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            swap_claim__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(swap_claim_view::Opaque {
                    swap_claim: swap_claim__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapClaimView.Opaque", FIELDS, GeneratedVisitor)
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
        if self.swap_tx.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapClaimView.Visible", len)?;
        if let Some(v) = self.swap_claim.as_ref() {
            struct_ser.serialize_field("swapClaim", v)?;
        }
        if let Some(v) = self.output_1.as_ref() {
            struct_ser.serialize_field("output1", v)?;
        }
        if let Some(v) = self.output_2.as_ref() {
            struct_ser.serialize_field("output2", v)?;
        }
        if let Some(v) = self.swap_tx.as_ref() {
            struct_ser.serialize_field("swapTx", v)?;
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
            "swap_tx",
            "swapTx",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapClaim,
            Output1,
            Output2,
            SwapTx,
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
                            "swapClaim" | "swap_claim" => Ok(GeneratedField::SwapClaim),
                            "output1" | "output_1" => Ok(GeneratedField::Output1),
                            "output2" | "output_2" => Ok(GeneratedField::Output2),
                            "swapTx" | "swap_tx" => Ok(GeneratedField::SwapTx),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapClaimView.Visible")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<swap_claim_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_claim__ = None;
                let mut output_1__ = None;
                let mut output_2__ = None;
                let mut swap_tx__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapClaim => {
                            if swap_claim__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            swap_claim__ = map_.next_value()?;
                        }
                        GeneratedField::Output1 => {
                            if output_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output1"));
                            }
                            output_1__ = map_.next_value()?;
                        }
                        GeneratedField::Output2 => {
                            if output_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output2"));
                            }
                            output_2__ = map_.next_value()?;
                        }
                        GeneratedField::SwapTx => {
                            if swap_tx__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapTx"));
                            }
                            swap_tx__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(swap_claim_view::Visible {
                    swap_claim: swap_claim__,
                    output_1: output_1__,
                    output_2: output_2__,
                    swap_tx: swap_tx__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapClaimView.Visible", FIELDS, GeneratedVisitor)
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
        if !self.traces.is_empty() {
            len += 1;
        }
        if self.input.is_some() {
            len += 1;
        }
        if self.output.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapExecution", len)?;
        if !self.traces.is_empty() {
            struct_ser.serialize_field("traces", &self.traces)?;
        }
        if let Some(v) = self.input.as_ref() {
            struct_ser.serialize_field("input", v)?;
        }
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
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
            "traces",
            "input",
            "output",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Traces,
            Input,
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
                            "traces" => Ok(GeneratedField::Traces),
                            "input" => Ok(GeneratedField::Input),
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
            type Value = SwapExecution;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapExecution")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapExecution, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut traces__ = None;
                let mut input__ = None;
                let mut output__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Traces => {
                            if traces__.is_some() {
                                return Err(serde::de::Error::duplicate_field("traces"));
                            }
                            traces__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Input => {
                            if input__.is_some() {
                                return Err(serde::de::Error::duplicate_field("input"));
                            }
                            input__ = map_.next_value()?;
                        }
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
                Ok(SwapExecution {
                    traces: traces__.unwrap_or_default(),
                    input: input__,
                    output: output__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapExecution", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for swap_execution::Trace {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.value.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapExecution.Trace", len)?;
        if !self.value.is_empty() {
            struct_ser.serialize_field("value", &self.value)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for swap_execution::Trace {
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
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = swap_execution::Trace;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapExecution.Trace")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<swap_execution::Trace, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(swap_execution::Trace {
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapExecution.Trace", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapExecutionRequest {
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
        if self.trading_pair.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapExecutionRequest", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapExecutionRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "trading_pair",
            "tradingPair",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            TradingPair,
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
                            "height" => Ok(GeneratedField::Height),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapExecutionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapExecutionRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapExecutionRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut trading_pair__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapExecutionRequest {
                    height: height__.unwrap_or_default(),
                    trading_pair: trading_pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapExecutionRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapExecutionResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_execution.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapExecutionResponse", len)?;
        if let Some(v) = self.swap_execution.as_ref() {
            struct_ser.serialize_field("swapExecution", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapExecutionResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_execution",
            "swapExecution",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapExecution,
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
                            "swapExecution" | "swap_execution" => Ok(GeneratedField::SwapExecution),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapExecutionResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapExecutionResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapExecutionResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_execution__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapExecution => {
                            if swap_execution__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapExecution"));
                            }
                            swap_execution__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapExecutionResponse {
                    swap_execution: swap_execution__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapExecutionResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapExecutionsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_height != 0 {
            len += 1;
        }
        if self.end_height != 0 {
            len += 1;
        }
        if self.trading_pair.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapExecutionsRequest", len)?;
        if self.start_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startHeight", ToString::to_string(&self.start_height).as_str())?;
        }
        if self.end_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("endHeight", ToString::to_string(&self.end_height).as_str())?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapExecutionsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_height",
            "startHeight",
            "end_height",
            "endHeight",
            "trading_pair",
            "tradingPair",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartHeight,
            EndHeight,
            TradingPair,
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
                            "startHeight" | "start_height" => Ok(GeneratedField::StartHeight),
                            "endHeight" | "end_height" => Ok(GeneratedField::EndHeight),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapExecutionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapExecutionsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapExecutionsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_height__ = None;
                let mut end_height__ = None;
                let mut trading_pair__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StartHeight => {
                            if start_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startHeight"));
                            }
                            start_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndHeight => {
                            if end_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endHeight"));
                            }
                            end_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapExecutionsRequest {
                    start_height: start_height__.unwrap_or_default(),
                    end_height: end_height__.unwrap_or_default(),
                    trading_pair: trading_pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapExecutionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapExecutionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_execution.is_some() {
            len += 1;
        }
        if self.height != 0 {
            len += 1;
        }
        if self.trading_pair.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapExecutionsResponse", len)?;
        if let Some(v) = self.swap_execution.as_ref() {
            struct_ser.serialize_field("swapExecution", v)?;
        }
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapExecutionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_execution",
            "swapExecution",
            "height",
            "trading_pair",
            "tradingPair",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapExecution,
            Height,
            TradingPair,
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
                            "swapExecution" | "swap_execution" => Ok(GeneratedField::SwapExecution),
                            "height" => Ok(GeneratedField::Height),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapExecutionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapExecutionsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapExecutionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_execution__ = None;
                let mut height__ = None;
                let mut trading_pair__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapExecution => {
                            if swap_execution__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapExecution"));
                            }
                            swap_execution__ = map_.next_value()?;
                        }
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapExecutionsResponse {
                    swap_execution: swap_execution__,
                    height: height__.unwrap_or_default(),
                    trading_pair: trading_pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapExecutionsResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapPayload", len)?;
        if let Some(v) = self.commitment.as_ref() {
            struct_ser.serialize_field("commitment", v)?;
        }
        if !self.encrypted_swap.is_empty() {
            #[allow(clippy::needless_borrow)]
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
                            "commitment" => Ok(GeneratedField::Commitment),
                            "encryptedSwap" | "encrypted_swap" => Ok(GeneratedField::EncryptedSwap),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapPayload")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapPayload, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commitment__ = None;
                let mut encrypted_swap__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = map_.next_value()?;
                        }
                        GeneratedField::EncryptedSwap => {
                            if encrypted_swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encryptedSwap"));
                            }
                            encrypted_swap__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapPayload {
                    commitment: commitment__,
                    encrypted_swap: encrypted_swap__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapPayload", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapPlaintext", len)?;
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
            #[allow(clippy::needless_borrow)]
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
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "delta1I" | "delta_1_i" => Ok(GeneratedField::Delta1I),
                            "delta2I" | "delta_2_i" => Ok(GeneratedField::Delta2I),
                            "claimFee" | "claim_fee" => Ok(GeneratedField::ClaimFee),
                            "claimAddress" | "claim_address" => Ok(GeneratedField::ClaimAddress),
                            "rseed" => Ok(GeneratedField::Rseed),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapPlaintext")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapPlaintext, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trading_pair__ = None;
                let mut delta_1_i__ = None;
                let mut delta_2_i__ = None;
                let mut claim_fee__ = None;
                let mut claim_address__ = None;
                let mut rseed__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Delta1I => {
                            if delta_1_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta1I"));
                            }
                            delta_1_i__ = map_.next_value()?;
                        }
                        GeneratedField::Delta2I => {
                            if delta_2_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delta2I"));
                            }
                            delta_2_i__ = map_.next_value()?;
                        }
                        GeneratedField::ClaimFee => {
                            if claim_fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("claimFee"));
                            }
                            claim_fee__ = map_.next_value()?;
                        }
                        GeneratedField::ClaimAddress => {
                            if claim_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("claimAddress"));
                            }
                            claim_address__ = map_.next_value()?;
                        }
                        GeneratedField::Rseed => {
                            if rseed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rseed"));
                            }
                            rseed__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
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
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapPlaintext", FIELDS, GeneratedVisitor)
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
        if !self.proof_blinding_r.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_s.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapPlan", len)?;
        if let Some(v) = self.swap_plaintext.as_ref() {
            struct_ser.serialize_field("swapPlaintext", v)?;
        }
        if !self.fee_blinding.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("feeBlinding", pbjson::private::base64::encode(&self.fee_blinding).as_str())?;
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
            "proof_blinding_r",
            "proofBlindingR",
            "proof_blinding_s",
            "proofBlindingS",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapPlaintext,
            FeeBlinding,
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
                            "swapPlaintext" | "swap_plaintext" => Ok(GeneratedField::SwapPlaintext),
                            "feeBlinding" | "fee_blinding" => Ok(GeneratedField::FeeBlinding),
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
            type Value = SwapPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_plaintext__ = None;
                let mut fee_blinding__ = None;
                let mut proof_blinding_r__ = None;
                let mut proof_blinding_s__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapPlaintext => {
                            if swap_plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapPlaintext"));
                            }
                            swap_plaintext__ = map_.next_value()?;
                        }
                        GeneratedField::FeeBlinding => {
                            if fee_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeBlinding"));
                            }
                            fee_blinding__ = 
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
                Ok(SwapPlan {
                    swap_plaintext: swap_plaintext__,
                    fee_blinding: fee_blinding__.unwrap_or_default(),
                    proof_blinding_r: proof_blinding_r__.unwrap_or_default(),
                    proof_blinding_s: proof_blinding_s__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapPlan", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapView", len)?;
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
            type Value = SwapView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_view__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if swap_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            swap_view__ = map_.next_value::<::std::option::Option<_>>()?.map(swap_view::SwapView::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if swap_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            swap_view__ = map_.next_value::<::std::option::Option<_>>()?.map(swap_view::SwapView::Opaque)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapView {
                    swap_view: swap_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapView", FIELDS, GeneratedVisitor)
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
        if self.batch_swap_output_data.is_some() {
            len += 1;
        }
        if self.output_1_value.is_some() {
            len += 1;
        }
        if self.output_2_value.is_some() {
            len += 1;
        }
        if self.asset_1_metadata.is_some() {
            len += 1;
        }
        if self.asset_2_metadata.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapView.Opaque", len)?;
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        if let Some(v) = self.batch_swap_output_data.as_ref() {
            struct_ser.serialize_field("batchSwapOutputData", v)?;
        }
        if let Some(v) = self.output_1_value.as_ref() {
            struct_ser.serialize_field("output1Value", v)?;
        }
        if let Some(v) = self.output_2_value.as_ref() {
            struct_ser.serialize_field("output2Value", v)?;
        }
        if let Some(v) = self.asset_1_metadata.as_ref() {
            struct_ser.serialize_field("asset1Metadata", v)?;
        }
        if let Some(v) = self.asset_2_metadata.as_ref() {
            struct_ser.serialize_field("asset2Metadata", v)?;
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
            "batch_swap_output_data",
            "batchSwapOutputData",
            "output_1_value",
            "output1Value",
            "output_2_value",
            "output2Value",
            "asset_1_metadata",
            "asset1Metadata",
            "asset_2_metadata",
            "asset2Metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Swap,
            BatchSwapOutputData,
            Output1Value,
            Output2Value,
            Asset1Metadata,
            Asset2Metadata,
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
                            "swap" => Ok(GeneratedField::Swap),
                            "batchSwapOutputData" | "batch_swap_output_data" => Ok(GeneratedField::BatchSwapOutputData),
                            "output1Value" | "output_1_value" => Ok(GeneratedField::Output1Value),
                            "output2Value" | "output_2_value" => Ok(GeneratedField::Output2Value),
                            "asset1Metadata" | "asset_1_metadata" => Ok(GeneratedField::Asset1Metadata),
                            "asset2Metadata" | "asset_2_metadata" => Ok(GeneratedField::Asset2Metadata),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapView.Opaque")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<swap_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap__ = None;
                let mut batch_swap_output_data__ = None;
                let mut output_1_value__ = None;
                let mut output_2_value__ = None;
                let mut asset_1_metadata__ = None;
                let mut asset_2_metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = map_.next_value()?;
                        }
                        GeneratedField::BatchSwapOutputData => {
                            if batch_swap_output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("batchSwapOutputData"));
                            }
                            batch_swap_output_data__ = map_.next_value()?;
                        }
                        GeneratedField::Output1Value => {
                            if output_1_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output1Value"));
                            }
                            output_1_value__ = map_.next_value()?;
                        }
                        GeneratedField::Output2Value => {
                            if output_2_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output2Value"));
                            }
                            output_2_value__ = map_.next_value()?;
                        }
                        GeneratedField::Asset1Metadata => {
                            if asset_1_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asset1Metadata"));
                            }
                            asset_1_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::Asset2Metadata => {
                            if asset_2_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asset2Metadata"));
                            }
                            asset_2_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(swap_view::Opaque {
                    swap: swap__,
                    batch_swap_output_data: batch_swap_output_data__,
                    output_1_value: output_1_value__,
                    output_2_value: output_2_value__,
                    asset_1_metadata: asset_1_metadata__,
                    asset_2_metadata: asset_2_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapView.Opaque", FIELDS, GeneratedVisitor)
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
        if self.claim_tx.is_some() {
            len += 1;
        }
        if self.batch_swap_output_data.is_some() {
            len += 1;
        }
        if self.output_1.is_some() {
            len += 1;
        }
        if self.output_2.is_some() {
            len += 1;
        }
        if self.asset_1_metadata.is_some() {
            len += 1;
        }
        if self.asset_2_metadata.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.SwapView.Visible", len)?;
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        if let Some(v) = self.swap_plaintext.as_ref() {
            struct_ser.serialize_field("swapPlaintext", v)?;
        }
        if let Some(v) = self.claim_tx.as_ref() {
            struct_ser.serialize_field("claimTx", v)?;
        }
        if let Some(v) = self.batch_swap_output_data.as_ref() {
            struct_ser.serialize_field("batchSwapOutputData", v)?;
        }
        if let Some(v) = self.output_1.as_ref() {
            struct_ser.serialize_field("output1", v)?;
        }
        if let Some(v) = self.output_2.as_ref() {
            struct_ser.serialize_field("output2", v)?;
        }
        if let Some(v) = self.asset_1_metadata.as_ref() {
            struct_ser.serialize_field("asset1Metadata", v)?;
        }
        if let Some(v) = self.asset_2_metadata.as_ref() {
            struct_ser.serialize_field("asset2Metadata", v)?;
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
            "claim_tx",
            "claimTx",
            "batch_swap_output_data",
            "batchSwapOutputData",
            "output_1",
            "output1",
            "output_2",
            "output2",
            "asset_1_metadata",
            "asset1Metadata",
            "asset_2_metadata",
            "asset2Metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Swap,
            SwapPlaintext,
            ClaimTx,
            BatchSwapOutputData,
            Output1,
            Output2,
            Asset1Metadata,
            Asset2Metadata,
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
                            "swap" => Ok(GeneratedField::Swap),
                            "swapPlaintext" | "swap_plaintext" => Ok(GeneratedField::SwapPlaintext),
                            "claimTx" | "claim_tx" => Ok(GeneratedField::ClaimTx),
                            "batchSwapOutputData" | "batch_swap_output_data" => Ok(GeneratedField::BatchSwapOutputData),
                            "output1" | "output_1" => Ok(GeneratedField::Output1),
                            "output2" | "output_2" => Ok(GeneratedField::Output2),
                            "asset1Metadata" | "asset_1_metadata" => Ok(GeneratedField::Asset1Metadata),
                            "asset2Metadata" | "asset_2_metadata" => Ok(GeneratedField::Asset2Metadata),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.SwapView.Visible")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<swap_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap__ = None;
                let mut swap_plaintext__ = None;
                let mut claim_tx__ = None;
                let mut batch_swap_output_data__ = None;
                let mut output_1__ = None;
                let mut output_2__ = None;
                let mut asset_1_metadata__ = None;
                let mut asset_2_metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = map_.next_value()?;
                        }
                        GeneratedField::SwapPlaintext => {
                            if swap_plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapPlaintext"));
                            }
                            swap_plaintext__ = map_.next_value()?;
                        }
                        GeneratedField::ClaimTx => {
                            if claim_tx__.is_some() {
                                return Err(serde::de::Error::duplicate_field("claimTx"));
                            }
                            claim_tx__ = map_.next_value()?;
                        }
                        GeneratedField::BatchSwapOutputData => {
                            if batch_swap_output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("batchSwapOutputData"));
                            }
                            batch_swap_output_data__ = map_.next_value()?;
                        }
                        GeneratedField::Output1 => {
                            if output_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output1"));
                            }
                            output_1__ = map_.next_value()?;
                        }
                        GeneratedField::Output2 => {
                            if output_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output2"));
                            }
                            output_2__ = map_.next_value()?;
                        }
                        GeneratedField::Asset1Metadata => {
                            if asset_1_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asset1Metadata"));
                            }
                            asset_1_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::Asset2Metadata => {
                            if asset_2_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asset2Metadata"));
                            }
                            asset_2_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(swap_view::Visible {
                    swap: swap__,
                    swap_plaintext: swap_plaintext__,
                    claim_tx: claim_tx__,
                    batch_swap_output_data: batch_swap_output_data__,
                    output_1: output_1__,
                    output_2: output_2__,
                    asset_1_metadata: asset_1_metadata__,
                    asset_2_metadata: asset_2_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.SwapView.Visible", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.TradingFunction", len)?;
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
                            "component" => Ok(GeneratedField::Component),
                            "pair" => Ok(GeneratedField::Pair),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.TradingFunction")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TradingFunction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut component__ = None;
                let mut pair__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Component => {
                            if component__.is_some() {
                                return Err(serde::de::Error::duplicate_field("component"));
                            }
                            component__ = map_.next_value()?;
                        }
                        GeneratedField::Pair => {
                            if pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pair"));
                            }
                            pair__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TradingFunction {
                    component: component__,
                    pair: pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.TradingFunction", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.TradingPair", len)?;
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
                            "asset1" | "asset_1" => Ok(GeneratedField::Asset1),
                            "asset2" | "asset_2" => Ok(GeneratedField::Asset2),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.core.component.dex.v1.TradingPair")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TradingPair, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_1__ = None;
                let mut asset_2__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Asset1 => {
                            if asset_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asset1"));
                            }
                            asset_1__ = map_.next_value()?;
                        }
                        GeneratedField::Asset2 => {
                            if asset_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asset2"));
                            }
                            asset_2__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TradingPair {
                    asset_1: asset_1__,
                    asset_2: asset_2__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.TradingPair", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZkSwapClaimProof {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.ZKSwapClaimProof", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZkSwapClaimProof {
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
            type Value = ZkSwapClaimProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.ZKSwapClaimProof")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZkSwapClaimProof, V::Error>
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
                Ok(ZkSwapClaimProof {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.ZKSwapClaimProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZkSwapProof {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.dex.v1.ZKSwapProof", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZkSwapProof {
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
            type Value = ZkSwapProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.dex.v1.ZKSwapProof")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZkSwapProof, V::Error>
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
                Ok(ZkSwapProof {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.dex.v1.ZKSwapProof", FIELDS, GeneratedVisitor)
    }
}
