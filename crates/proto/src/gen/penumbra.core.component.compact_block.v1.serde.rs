impl serde::Serialize for CompactBlock {
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
        if !self.state_payloads.is_empty() {
            len += 1;
        }
        if !self.nullifiers.is_empty() {
            len += 1;
        }
        if self.block_root.is_some() {
            len += 1;
        }
        if self.epoch_root.is_some() {
            len += 1;
        }
        if self.proposal_started {
            len += 1;
        }
        if self.fmd_parameters.is_some() {
            len += 1;
        }
        if !self.swap_outputs.is_empty() {
            len += 1;
        }
        if self.app_parameters_updated {
            len += 1;
        }
        if self.gas_prices.is_some() {
            len += 1;
        }
        if !self.alt_gas_prices.is_empty() {
            len += 1;
        }
        if self.epoch_index != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compact_block.v1.CompactBlock", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if !self.state_payloads.is_empty() {
            struct_ser.serialize_field("statePayloads", &self.state_payloads)?;
        }
        if !self.nullifiers.is_empty() {
            struct_ser.serialize_field("nullifiers", &self.nullifiers)?;
        }
        if let Some(v) = self.block_root.as_ref() {
            struct_ser.serialize_field("blockRoot", v)?;
        }
        if let Some(v) = self.epoch_root.as_ref() {
            struct_ser.serialize_field("epochRoot", v)?;
        }
        if self.proposal_started {
            struct_ser.serialize_field("proposalStarted", &self.proposal_started)?;
        }
        if let Some(v) = self.fmd_parameters.as_ref() {
            struct_ser.serialize_field("fmdParameters", v)?;
        }
        if !self.swap_outputs.is_empty() {
            struct_ser.serialize_field("swapOutputs", &self.swap_outputs)?;
        }
        if self.app_parameters_updated {
            struct_ser.serialize_field("appParametersUpdated", &self.app_parameters_updated)?;
        }
        if let Some(v) = self.gas_prices.as_ref() {
            struct_ser.serialize_field("gasPrices", v)?;
        }
        if !self.alt_gas_prices.is_empty() {
            struct_ser.serialize_field("altGasPrices", &self.alt_gas_prices)?;
        }
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CompactBlock {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "state_payloads",
            "statePayloads",
            "nullifiers",
            "block_root",
            "blockRoot",
            "epoch_root",
            "epochRoot",
            "proposal_started",
            "proposalStarted",
            "fmd_parameters",
            "fmdParameters",
            "swap_outputs",
            "swapOutputs",
            "app_parameters_updated",
            "appParametersUpdated",
            "gas_prices",
            "gasPrices",
            "alt_gas_prices",
            "altGasPrices",
            "epoch_index",
            "epochIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            StatePayloads,
            Nullifiers,
            BlockRoot,
            EpochRoot,
            ProposalStarted,
            FmdParameters,
            SwapOutputs,
            AppParametersUpdated,
            GasPrices,
            AltGasPrices,
            EpochIndex,
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
                            "statePayloads" | "state_payloads" => Ok(GeneratedField::StatePayloads),
                            "nullifiers" => Ok(GeneratedField::Nullifiers),
                            "blockRoot" | "block_root" => Ok(GeneratedField::BlockRoot),
                            "epochRoot" | "epoch_root" => Ok(GeneratedField::EpochRoot),
                            "proposalStarted" | "proposal_started" => Ok(GeneratedField::ProposalStarted),
                            "fmdParameters" | "fmd_parameters" => Ok(GeneratedField::FmdParameters),
                            "swapOutputs" | "swap_outputs" => Ok(GeneratedField::SwapOutputs),
                            "appParametersUpdated" | "app_parameters_updated" => Ok(GeneratedField::AppParametersUpdated),
                            "gasPrices" | "gas_prices" => Ok(GeneratedField::GasPrices),
                            "altGasPrices" | "alt_gas_prices" => Ok(GeneratedField::AltGasPrices),
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CompactBlock;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compact_block.v1.CompactBlock")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CompactBlock, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut state_payloads__ = None;
                let mut nullifiers__ = None;
                let mut block_root__ = None;
                let mut epoch_root__ = None;
                let mut proposal_started__ = None;
                let mut fmd_parameters__ = None;
                let mut swap_outputs__ = None;
                let mut app_parameters_updated__ = None;
                let mut gas_prices__ = None;
                let mut alt_gas_prices__ = None;
                let mut epoch_index__ = None;
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
                        GeneratedField::StatePayloads => {
                            if state_payloads__.is_some() {
                                return Err(serde::de::Error::duplicate_field("statePayloads"));
                            }
                            state_payloads__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Nullifiers => {
                            if nullifiers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifiers"));
                            }
                            nullifiers__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BlockRoot => {
                            if block_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockRoot"));
                            }
                            block_root__ = map_.next_value()?;
                        }
                        GeneratedField::EpochRoot => {
                            if epoch_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochRoot"));
                            }
                            epoch_root__ = map_.next_value()?;
                        }
                        GeneratedField::ProposalStarted => {
                            if proposal_started__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalStarted"));
                            }
                            proposal_started__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FmdParameters => {
                            if fmd_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fmdParameters"));
                            }
                            fmd_parameters__ = map_.next_value()?;
                        }
                        GeneratedField::SwapOutputs => {
                            if swap_outputs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapOutputs"));
                            }
                            swap_outputs__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AppParametersUpdated => {
                            if app_parameters_updated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appParametersUpdated"));
                            }
                            app_parameters_updated__ = Some(map_.next_value()?);
                        }
                        GeneratedField::GasPrices => {
                            if gas_prices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gasPrices"));
                            }
                            gas_prices__ = map_.next_value()?;
                        }
                        GeneratedField::AltGasPrices => {
                            if alt_gas_prices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("altGasPrices"));
                            }
                            alt_gas_prices__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CompactBlock {
                    height: height__.unwrap_or_default(),
                    state_payloads: state_payloads__.unwrap_or_default(),
                    nullifiers: nullifiers__.unwrap_or_default(),
                    block_root: block_root__,
                    epoch_root: epoch_root__,
                    proposal_started: proposal_started__.unwrap_or_default(),
                    fmd_parameters: fmd_parameters__,
                    swap_outputs: swap_outputs__.unwrap_or_default(),
                    app_parameters_updated: app_parameters_updated__.unwrap_or_default(),
                    gas_prices: gas_prices__,
                    alt_gas_prices: alt_gas_prices__.unwrap_or_default(),
                    epoch_index: epoch_index__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compact_block.v1.CompactBlock", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CompactBlockRangeRequest {
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
        if self.keep_alive {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compact_block.v1.CompactBlockRangeRequest", len)?;
        if self.start_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startHeight", ToString::to_string(&self.start_height).as_str())?;
        }
        if self.end_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("endHeight", ToString::to_string(&self.end_height).as_str())?;
        }
        if self.keep_alive {
            struct_ser.serialize_field("keepAlive", &self.keep_alive)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CompactBlockRangeRequest {
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
            "keep_alive",
            "keepAlive",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartHeight,
            EndHeight,
            KeepAlive,
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
                            "keepAlive" | "keep_alive" => Ok(GeneratedField::KeepAlive),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CompactBlockRangeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compact_block.v1.CompactBlockRangeRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CompactBlockRangeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_height__ = None;
                let mut end_height__ = None;
                let mut keep_alive__ = None;
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
                        GeneratedField::KeepAlive => {
                            if keep_alive__.is_some() {
                                return Err(serde::de::Error::duplicate_field("keepAlive"));
                            }
                            keep_alive__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CompactBlockRangeRequest {
                    start_height: start_height__.unwrap_or_default(),
                    end_height: end_height__.unwrap_or_default(),
                    keep_alive: keep_alive__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compact_block.v1.CompactBlockRangeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CompactBlockRangeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.compact_block.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compact_block.v1.CompactBlockRangeResponse", len)?;
        if let Some(v) = self.compact_block.as_ref() {
            struct_ser.serialize_field("compactBlock", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CompactBlockRangeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "compact_block",
            "compactBlock",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CompactBlock,
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
                            "compactBlock" | "compact_block" => Ok(GeneratedField::CompactBlock),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CompactBlockRangeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compact_block.v1.CompactBlockRangeResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CompactBlockRangeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut compact_block__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CompactBlock => {
                            if compact_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compactBlock"));
                            }
                            compact_block__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CompactBlockRangeResponse {
                    compact_block: compact_block__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compact_block.v1.CompactBlockRangeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CompactBlockRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compact_block.v1.CompactBlockRequest", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CompactBlockRequest {
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
            type Value = CompactBlockRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compact_block.v1.CompactBlockRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CompactBlockRequest, V::Error>
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
                Ok(CompactBlockRequest {
                    height: height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compact_block.v1.CompactBlockRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CompactBlockResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.compact_block.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compact_block.v1.CompactBlockResponse", len)?;
        if let Some(v) = self.compact_block.as_ref() {
            struct_ser.serialize_field("compactBlock", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CompactBlockResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "compact_block",
            "compactBlock",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CompactBlock,
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
                            "compactBlock" | "compact_block" => Ok(GeneratedField::CompactBlock),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CompactBlockResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compact_block.v1.CompactBlockResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CompactBlockResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut compact_block__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CompactBlock => {
                            if compact_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compactBlock"));
                            }
                            compact_block__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CompactBlockResponse {
                    compact_block: compact_block__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compact_block.v1.CompactBlockResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StatePayload {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.source.is_some() {
            len += 1;
        }
        if self.state_payload.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compact_block.v1.StatePayload", len)?;
        if let Some(v) = self.source.as_ref() {
            struct_ser.serialize_field("source", v)?;
        }
        if let Some(v) = self.state_payload.as_ref() {
            match v {
                state_payload::StatePayload::RolledUp(v) => {
                    struct_ser.serialize_field("rolledUp", v)?;
                }
                state_payload::StatePayload::Note(v) => {
                    struct_ser.serialize_field("note", v)?;
                }
                state_payload::StatePayload::Swap(v) => {
                    struct_ser.serialize_field("swap", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StatePayload {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "source",
            "rolled_up",
            "rolledUp",
            "note",
            "swap",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Source,
            RolledUp,
            Note,
            Swap,
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
                            "source" => Ok(GeneratedField::Source),
                            "rolledUp" | "rolled_up" => Ok(GeneratedField::RolledUp),
                            "note" => Ok(GeneratedField::Note),
                            "swap" => Ok(GeneratedField::Swap),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StatePayload;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compact_block.v1.StatePayload")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<StatePayload, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut source__ = None;
                let mut state_payload__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Source => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source__ = map_.next_value()?;
                        }
                        GeneratedField::RolledUp => {
                            if state_payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rolledUp"));
                            }
                            state_payload__ = map_.next_value::<::std::option::Option<_>>()?.map(state_payload::StatePayload::RolledUp)
;
                        }
                        GeneratedField::Note => {
                            if state_payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            state_payload__ = map_.next_value::<::std::option::Option<_>>()?.map(state_payload::StatePayload::Note)
;
                        }
                        GeneratedField::Swap => {
                            if state_payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            state_payload__ = map_.next_value::<::std::option::Option<_>>()?.map(state_payload::StatePayload::Swap)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(StatePayload {
                    source: source__,
                    state_payload: state_payload__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compact_block.v1.StatePayload", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for state_payload::Note {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compact_block.v1.StatePayload.Note", len)?;
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for state_payload::Note {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = state_payload::Note;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compact_block.v1.StatePayload.Note")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<state_payload::Note, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                Ok(state_payload::Note {
                    note: note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compact_block.v1.StatePayload.Note", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for state_payload::RolledUp {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compact_block.v1.StatePayload.RolledUp", len)?;
        if let Some(v) = self.commitment.as_ref() {
            struct_ser.serialize_field("commitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for state_payload::RolledUp {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commitment,
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
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = state_payload::RolledUp;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compact_block.v1.StatePayload.RolledUp")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<state_payload::RolledUp, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(state_payload::RolledUp {
                    commitment: commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compact_block.v1.StatePayload.RolledUp", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for state_payload::Swap {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compact_block.v1.StatePayload.Swap", len)?;
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for state_payload::Swap {
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
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = state_payload::Swap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compact_block.v1.StatePayload.Swap")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<state_payload::Swap, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(state_payload::Swap {
                    swap: swap__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compact_block.v1.StatePayload.Swap", FIELDS, GeneratedVisitor)
    }
}
