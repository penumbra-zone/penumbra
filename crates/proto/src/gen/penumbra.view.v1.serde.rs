impl serde::Serialize for AddressByIndexRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address_index.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AddressByIndexRequest", len)?;
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AddressByIndexRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address_index",
            "addressIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AddressIndex,
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
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AddressByIndexRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AddressByIndexRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AddressByIndexRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AddressByIndexRequest {
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AddressByIndexRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AddressByIndexResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AddressByIndexResponse", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AddressByIndexResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = AddressByIndexResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AddressByIndexResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AddressByIndexResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                Ok(AddressByIndexResponse {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AddressByIndexResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AppParametersRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1.AppParametersRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AppParametersRequest {
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
            type Value = AppParametersRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AppParametersRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AppParametersRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(AppParametersRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AppParametersRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AppParametersResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.parameters.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AppParametersResponse", len)?;
        if let Some(v) = self.parameters.as_ref() {
            struct_ser.serialize_field("parameters", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AppParametersResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parameters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Parameters,
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
                            "parameters" => Ok(GeneratedField::Parameters),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AppParametersResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AppParametersResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AppParametersResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut parameters__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Parameters => {
                            if parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameters"));
                            }
                            parameters__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AppParametersResponse {
                    parameters: parameters__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AppParametersResponse", FIELDS, GeneratedVisitor)
    }
}
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AssetMetadataByIdRequest", len)?;
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
                formatter.write_str("struct penumbra.view.v1.AssetMetadataByIdRequest")
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
        deserializer.deserialize_struct("penumbra.view.v1.AssetMetadataByIdRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AssetMetadataByIdResponse", len)?;
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
                formatter.write_str("struct penumbra.view.v1.AssetMetadataByIdResponse")
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
        deserializer.deserialize_struct("penumbra.view.v1.AssetMetadataByIdResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AssetsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.filtered {
            len += 1;
        }
        if !self.include_specific_denominations.is_empty() {
            len += 1;
        }
        if self.include_delegation_tokens {
            len += 1;
        }
        if self.include_unbonding_tokens {
            len += 1;
        }
        if self.include_lp_nfts {
            len += 1;
        }
        if self.include_proposal_nfts {
            len += 1;
        }
        if self.include_voting_receipt_tokens {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AssetsRequest", len)?;
        if self.filtered {
            struct_ser.serialize_field("filtered", &self.filtered)?;
        }
        if !self.include_specific_denominations.is_empty() {
            struct_ser.serialize_field("includeSpecificDenominations", &self.include_specific_denominations)?;
        }
        if self.include_delegation_tokens {
            struct_ser.serialize_field("includeDelegationTokens", &self.include_delegation_tokens)?;
        }
        if self.include_unbonding_tokens {
            struct_ser.serialize_field("includeUnbondingTokens", &self.include_unbonding_tokens)?;
        }
        if self.include_lp_nfts {
            struct_ser.serialize_field("includeLpNfts", &self.include_lp_nfts)?;
        }
        if self.include_proposal_nfts {
            struct_ser.serialize_field("includeProposalNfts", &self.include_proposal_nfts)?;
        }
        if self.include_voting_receipt_tokens {
            struct_ser.serialize_field("includeVotingReceiptTokens", &self.include_voting_receipt_tokens)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "filtered",
            "include_specific_denominations",
            "includeSpecificDenominations",
            "include_delegation_tokens",
            "includeDelegationTokens",
            "include_unbonding_tokens",
            "includeUnbondingTokens",
            "include_lp_nfts",
            "includeLpNfts",
            "include_proposal_nfts",
            "includeProposalNfts",
            "include_voting_receipt_tokens",
            "includeVotingReceiptTokens",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Filtered,
            IncludeSpecificDenominations,
            IncludeDelegationTokens,
            IncludeUnbondingTokens,
            IncludeLpNfts,
            IncludeProposalNfts,
            IncludeVotingReceiptTokens,
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
                            "filtered" => Ok(GeneratedField::Filtered),
                            "includeSpecificDenominations" | "include_specific_denominations" => Ok(GeneratedField::IncludeSpecificDenominations),
                            "includeDelegationTokens" | "include_delegation_tokens" => Ok(GeneratedField::IncludeDelegationTokens),
                            "includeUnbondingTokens" | "include_unbonding_tokens" => Ok(GeneratedField::IncludeUnbondingTokens),
                            "includeLpNfts" | "include_lp_nfts" => Ok(GeneratedField::IncludeLpNfts),
                            "includeProposalNfts" | "include_proposal_nfts" => Ok(GeneratedField::IncludeProposalNfts),
                            "includeVotingReceiptTokens" | "include_voting_receipt_tokens" => Ok(GeneratedField::IncludeVotingReceiptTokens),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AssetsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AssetsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AssetsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut filtered__ = None;
                let mut include_specific_denominations__ = None;
                let mut include_delegation_tokens__ = None;
                let mut include_unbonding_tokens__ = None;
                let mut include_lp_nfts__ = None;
                let mut include_proposal_nfts__ = None;
                let mut include_voting_receipt_tokens__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Filtered => {
                            if filtered__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filtered"));
                            }
                            filtered__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IncludeSpecificDenominations => {
                            if include_specific_denominations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeSpecificDenominations"));
                            }
                            include_specific_denominations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IncludeDelegationTokens => {
                            if include_delegation_tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeDelegationTokens"));
                            }
                            include_delegation_tokens__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IncludeUnbondingTokens => {
                            if include_unbonding_tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeUnbondingTokens"));
                            }
                            include_unbonding_tokens__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IncludeLpNfts => {
                            if include_lp_nfts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeLpNfts"));
                            }
                            include_lp_nfts__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IncludeProposalNfts => {
                            if include_proposal_nfts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeProposalNfts"));
                            }
                            include_proposal_nfts__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IncludeVotingReceiptTokens => {
                            if include_voting_receipt_tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeVotingReceiptTokens"));
                            }
                            include_voting_receipt_tokens__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AssetsRequest {
                    filtered: filtered__.unwrap_or_default(),
                    include_specific_denominations: include_specific_denominations__.unwrap_or_default(),
                    include_delegation_tokens: include_delegation_tokens__.unwrap_or_default(),
                    include_unbonding_tokens: include_unbonding_tokens__.unwrap_or_default(),
                    include_lp_nfts: include_lp_nfts__.unwrap_or_default(),
                    include_proposal_nfts: include_proposal_nfts__.unwrap_or_default(),
                    include_voting_receipt_tokens: include_voting_receipt_tokens__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AssetsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AssetsResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AssetsResponse", len)?;
        if let Some(v) = self.denom_metadata.as_ref() {
            struct_ser.serialize_field("denomMetadata", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetsResponse {
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
            type Value = AssetsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AssetsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AssetsResponse, V::Error>
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
                Ok(AssetsResponse {
                    denom_metadata: denom_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AssetsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuctionsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.account_filter.is_some() {
            len += 1;
        }
        if self.include_inactive {
            len += 1;
        }
        if self.query_latest_state {
            len += 1;
        }
        if !self.auction_ids_filter.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AuctionsRequest", len)?;
        if let Some(v) = self.account_filter.as_ref() {
            struct_ser.serialize_field("accountFilter", v)?;
        }
        if self.include_inactive {
            struct_ser.serialize_field("includeInactive", &self.include_inactive)?;
        }
        if self.query_latest_state {
            struct_ser.serialize_field("queryLatestState", &self.query_latest_state)?;
        }
        if !self.auction_ids_filter.is_empty() {
            struct_ser.serialize_field("auctionIdsFilter", &self.auction_ids_filter)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuctionsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "account_filter",
            "accountFilter",
            "include_inactive",
            "includeInactive",
            "query_latest_state",
            "queryLatestState",
            "auction_ids_filter",
            "auctionIdsFilter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AccountFilter,
            IncludeInactive,
            QueryLatestState,
            AuctionIdsFilter,
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
                            "accountFilter" | "account_filter" => Ok(GeneratedField::AccountFilter),
                            "includeInactive" | "include_inactive" => Ok(GeneratedField::IncludeInactive),
                            "queryLatestState" | "query_latest_state" => Ok(GeneratedField::QueryLatestState),
                            "auctionIdsFilter" | "auction_ids_filter" => Ok(GeneratedField::AuctionIdsFilter),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuctionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AuctionsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuctionsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut account_filter__ = None;
                let mut include_inactive__ = None;
                let mut query_latest_state__ = None;
                let mut auction_ids_filter__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AccountFilter => {
                            if account_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountFilter"));
                            }
                            account_filter__ = map_.next_value()?;
                        }
                        GeneratedField::IncludeInactive => {
                            if include_inactive__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeInactive"));
                            }
                            include_inactive__ = Some(map_.next_value()?);
                        }
                        GeneratedField::QueryLatestState => {
                            if query_latest_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("queryLatestState"));
                            }
                            query_latest_state__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuctionIdsFilter => {
                            if auction_ids_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("auctionIdsFilter"));
                            }
                            auction_ids_filter__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuctionsRequest {
                    account_filter: account_filter__,
                    include_inactive: include_inactive__.unwrap_or_default(),
                    query_latest_state: query_latest_state__.unwrap_or_default(),
                    auction_ids_filter: auction_ids_filter__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AuctionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuctionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id.is_some() {
            len += 1;
        }
        if self.auction.is_some() {
            len += 1;
        }
        if !self.positions.is_empty() {
            len += 1;
        }
        if self.note_record.is_some() {
            len += 1;
        }
        if self.local_seq != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AuctionsResponse", len)?;
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if let Some(v) = self.auction.as_ref() {
            struct_ser.serialize_field("auction", v)?;
        }
        if !self.positions.is_empty() {
            struct_ser.serialize_field("positions", &self.positions)?;
        }
        if let Some(v) = self.note_record.as_ref() {
            struct_ser.serialize_field("noteRecord", v)?;
        }
        if self.local_seq != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("localSeq", ToString::to_string(&self.local_seq).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuctionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "auction",
            "positions",
            "note_record",
            "noteRecord",
            "local_seq",
            "localSeq",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Auction,
            Positions,
            NoteRecord,
            LocalSeq,
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
                            "id" => Ok(GeneratedField::Id),
                            "auction" => Ok(GeneratedField::Auction),
                            "positions" => Ok(GeneratedField::Positions),
                            "noteRecord" | "note_record" => Ok(GeneratedField::NoteRecord),
                            "localSeq" | "local_seq" => Ok(GeneratedField::LocalSeq),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuctionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AuctionsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuctionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut auction__ = None;
                let mut positions__ = None;
                let mut note_record__ = None;
                let mut local_seq__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = map_.next_value()?;
                        }
                        GeneratedField::Auction => {
                            if auction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("auction"));
                            }
                            auction__ = map_.next_value()?;
                        }
                        GeneratedField::Positions => {
                            if positions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positions"));
                            }
                            positions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NoteRecord => {
                            if note_record__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteRecord"));
                            }
                            note_record__ = map_.next_value()?;
                        }
                        GeneratedField::LocalSeq => {
                            if local_seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("localSeq"));
                            }
                            local_seq__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuctionsResponse {
                    id: id__,
                    auction: auction__,
                    positions: positions__.unwrap_or_default(),
                    note_record: note_record__,
                    local_seq: local_seq__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AuctionsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthorizeAndBuildRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.transaction_plan.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AuthorizeAndBuildRequest", len)?;
        if let Some(v) = self.transaction_plan.as_ref() {
            struct_ser.serialize_field("transactionPlan", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizeAndBuildRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "transaction_plan",
            "transactionPlan",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TransactionPlan,
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
                            "transactionPlan" | "transaction_plan" => Ok(GeneratedField::TransactionPlan),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizeAndBuildRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AuthorizeAndBuildRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizeAndBuildRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut transaction_plan__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TransactionPlan => {
                            if transaction_plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionPlan"));
                            }
                            transaction_plan__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuthorizeAndBuildRequest {
                    transaction_plan: transaction_plan__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AuthorizeAndBuildRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthorizeAndBuildResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AuthorizeAndBuildResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            match v {
                authorize_and_build_response::Status::BuildProgress(v) => {
                    struct_ser.serialize_field("buildProgress", v)?;
                }
                authorize_and_build_response::Status::Complete(v) => {
                    struct_ser.serialize_field("complete", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizeAndBuildResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "build_progress",
            "buildProgress",
            "complete",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BuildProgress,
            Complete,
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
                            "buildProgress" | "build_progress" => Ok(GeneratedField::BuildProgress),
                            "complete" => Ok(GeneratedField::Complete),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizeAndBuildResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AuthorizeAndBuildResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizeAndBuildResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BuildProgress => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildProgress"));
                            }
                            status__ = map_.next_value::<::std::option::Option<_>>()?.map(authorize_and_build_response::Status::BuildProgress)
;
                        }
                        GeneratedField::Complete => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("complete"));
                            }
                            status__ = map_.next_value::<::std::option::Option<_>>()?.map(authorize_and_build_response::Status::Complete)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuthorizeAndBuildResponse {
                    status: status__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AuthorizeAndBuildResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for authorize_and_build_response::BuildProgress {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.progress != 0. {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AuthorizeAndBuildResponse.BuildProgress", len)?;
        if self.progress != 0. {
            struct_ser.serialize_field("progress", &self.progress)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for authorize_and_build_response::BuildProgress {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "progress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Progress,
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
                            "progress" => Ok(GeneratedField::Progress),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = authorize_and_build_response::BuildProgress;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AuthorizeAndBuildResponse.BuildProgress")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<authorize_and_build_response::BuildProgress, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut progress__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Progress => {
                            if progress__.is_some() {
                                return Err(serde::de::Error::duplicate_field("progress"));
                            }
                            progress__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(authorize_and_build_response::BuildProgress {
                    progress: progress__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AuthorizeAndBuildResponse.BuildProgress", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for authorize_and_build_response::Complete {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.transaction.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.AuthorizeAndBuildResponse.Complete", len)?;
        if let Some(v) = self.transaction.as_ref() {
            struct_ser.serialize_field("transaction", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for authorize_and_build_response::Complete {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "transaction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Transaction,
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
                            "transaction" => Ok(GeneratedField::Transaction),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = authorize_and_build_response::Complete;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.AuthorizeAndBuildResponse.Complete")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<authorize_and_build_response::Complete, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut transaction__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Transaction => {
                            if transaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transaction"));
                            }
                            transaction__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(authorize_and_build_response::Complete {
                    transaction: transaction__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.AuthorizeAndBuildResponse.Complete", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BalancesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.account_filter.is_some() {
            len += 1;
        }
        if self.asset_id_filter.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.BalancesRequest", len)?;
        if let Some(v) = self.account_filter.as_ref() {
            struct_ser.serialize_field("accountFilter", v)?;
        }
        if let Some(v) = self.asset_id_filter.as_ref() {
            struct_ser.serialize_field("assetIdFilter", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BalancesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "account_filter",
            "accountFilter",
            "asset_id_filter",
            "assetIdFilter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AccountFilter,
            AssetIdFilter,
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
                            "accountFilter" | "account_filter" => Ok(GeneratedField::AccountFilter),
                            "assetIdFilter" | "asset_id_filter" => Ok(GeneratedField::AssetIdFilter),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BalancesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.BalancesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BalancesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut account_filter__ = None;
                let mut asset_id_filter__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AccountFilter => {
                            if account_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountFilter"));
                            }
                            account_filter__ = map_.next_value()?;
                        }
                        GeneratedField::AssetIdFilter => {
                            if asset_id_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetIdFilter"));
                            }
                            asset_id_filter__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(BalancesRequest {
                    account_filter: account_filter__,
                    asset_id_filter: asset_id_filter__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.BalancesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BalancesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.account.is_some() {
            len += 1;
        }
        if self.balance.is_some() {
            len += 1;
        }
        if self.account_address.is_some() {
            len += 1;
        }
        if self.balance_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.BalancesResponse", len)?;
        if let Some(v) = self.account.as_ref() {
            struct_ser.serialize_field("account", v)?;
        }
        if let Some(v) = self.balance.as_ref() {
            struct_ser.serialize_field("balance", v)?;
        }
        if let Some(v) = self.account_address.as_ref() {
            struct_ser.serialize_field("accountAddress", v)?;
        }
        if let Some(v) = self.balance_view.as_ref() {
            struct_ser.serialize_field("balanceView", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BalancesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "account",
            "balance",
            "account_address",
            "accountAddress",
            "balance_view",
            "balanceView",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Account,
            Balance,
            AccountAddress,
            BalanceView,
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
                            "account" => Ok(GeneratedField::Account),
                            "balance" => Ok(GeneratedField::Balance),
                            "accountAddress" | "account_address" => Ok(GeneratedField::AccountAddress),
                            "balanceView" | "balance_view" => Ok(GeneratedField::BalanceView),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BalancesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.BalancesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BalancesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut account__ = None;
                let mut balance__ = None;
                let mut account_address__ = None;
                let mut balance_view__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Account => {
                            if account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("account"));
                            }
                            account__ = map_.next_value()?;
                        }
                        GeneratedField::Balance => {
                            if balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balance"));
                            }
                            balance__ = map_.next_value()?;
                        }
                        GeneratedField::AccountAddress => {
                            if account_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountAddress"));
                            }
                            account_address__ = map_.next_value()?;
                        }
                        GeneratedField::BalanceView => {
                            if balance_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceView"));
                            }
                            balance_view__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(BalancesResponse {
                    account: account__,
                    balance: balance__,
                    account_address: account_address__,
                    balance_view: balance_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.BalancesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BroadcastTransactionRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.transaction.is_some() {
            len += 1;
        }
        if self.await_detection {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.BroadcastTransactionRequest", len)?;
        if let Some(v) = self.transaction.as_ref() {
            struct_ser.serialize_field("transaction", v)?;
        }
        if self.await_detection {
            struct_ser.serialize_field("awaitDetection", &self.await_detection)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BroadcastTransactionRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "transaction",
            "await_detection",
            "awaitDetection",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Transaction,
            AwaitDetection,
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
                            "transaction" => Ok(GeneratedField::Transaction),
                            "awaitDetection" | "await_detection" => Ok(GeneratedField::AwaitDetection),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BroadcastTransactionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.BroadcastTransactionRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BroadcastTransactionRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut transaction__ = None;
                let mut await_detection__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Transaction => {
                            if transaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transaction"));
                            }
                            transaction__ = map_.next_value()?;
                        }
                        GeneratedField::AwaitDetection => {
                            if await_detection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awaitDetection"));
                            }
                            await_detection__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(BroadcastTransactionRequest {
                    transaction: transaction__,
                    await_detection: await_detection__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.BroadcastTransactionRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BroadcastTransactionResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.BroadcastTransactionResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            match v {
                broadcast_transaction_response::Status::BroadcastSuccess(v) => {
                    struct_ser.serialize_field("broadcastSuccess", v)?;
                }
                broadcast_transaction_response::Status::Confirmed(v) => {
                    struct_ser.serialize_field("confirmed", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BroadcastTransactionResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "broadcast_success",
            "broadcastSuccess",
            "confirmed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BroadcastSuccess,
            Confirmed,
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
                            "broadcastSuccess" | "broadcast_success" => Ok(GeneratedField::BroadcastSuccess),
                            "confirmed" => Ok(GeneratedField::Confirmed),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BroadcastTransactionResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.BroadcastTransactionResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BroadcastTransactionResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BroadcastSuccess => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("broadcastSuccess"));
                            }
                            status__ = map_.next_value::<::std::option::Option<_>>()?.map(broadcast_transaction_response::Status::BroadcastSuccess)
;
                        }
                        GeneratedField::Confirmed => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("confirmed"));
                            }
                            status__ = map_.next_value::<::std::option::Option<_>>()?.map(broadcast_transaction_response::Status::Confirmed)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(BroadcastTransactionResponse {
                    status: status__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.BroadcastTransactionResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for broadcast_transaction_response::BroadcastSuccess {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.BroadcastTransactionResponse.BroadcastSuccess", len)?;
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for broadcast_transaction_response::BroadcastSuccess {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = broadcast_transaction_response::BroadcastSuccess;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.BroadcastTransactionResponse.BroadcastSuccess")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<broadcast_transaction_response::BroadcastSuccess, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                Ok(broadcast_transaction_response::BroadcastSuccess {
                    id: id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.BroadcastTransactionResponse.BroadcastSuccess", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for broadcast_transaction_response::Confirmed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id.is_some() {
            len += 1;
        }
        if self.detection_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.BroadcastTransactionResponse.Confirmed", len)?;
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if self.detection_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("detectionHeight", ToString::to_string(&self.detection_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for broadcast_transaction_response::Confirmed {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "detection_height",
            "detectionHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            DetectionHeight,
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
                            "id" => Ok(GeneratedField::Id),
                            "detectionHeight" | "detection_height" => Ok(GeneratedField::DetectionHeight),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = broadcast_transaction_response::Confirmed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.BroadcastTransactionResponse.Confirmed")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<broadcast_transaction_response::Confirmed, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut detection_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = map_.next_value()?;
                        }
                        GeneratedField::DetectionHeight => {
                            if detection_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("detectionHeight"));
                            }
                            detection_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(broadcast_transaction_response::Confirmed {
                    id: id__,
                    detection_height: detection_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.BroadcastTransactionResponse.Confirmed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegationsByAddressIndexRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address_index.is_some() {
            len += 1;
        }
        if self.filter != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.DelegationsByAddressIndexRequest", len)?;
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        if self.filter != 0 {
            let v = delegations_by_address_index_request::Filter::try_from(self.filter)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.filter)))?;
            struct_ser.serialize_field("filter", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegationsByAddressIndexRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address_index",
            "addressIndex",
            "filter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AddressIndex,
            Filter,
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
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            "filter" => Ok(GeneratedField::Filter),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegationsByAddressIndexRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.DelegationsByAddressIndexRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DelegationsByAddressIndexRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address_index__ = None;
                let mut filter__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map_.next_value()?;
                        }
                        GeneratedField::Filter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filter"));
                            }
                            filter__ = Some(map_.next_value::<delegations_by_address_index_request::Filter>()? as i32);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DelegationsByAddressIndexRequest {
                    address_index: address_index__,
                    filter: filter__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.DelegationsByAddressIndexRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for delegations_by_address_index_request::Filter {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "FILTER_UNSPECIFIED",
            Self::AllActiveWithNonzeroBalances => "FILTER_ALL_ACTIVE_WITH_NONZERO_BALANCES",
            Self::All => "FILTER_ALL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for delegations_by_address_index_request::Filter {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "FILTER_UNSPECIFIED",
            "FILTER_ALL_ACTIVE_WITH_NONZERO_BALANCES",
            "FILTER_ALL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = delegations_by_address_index_request::Filter;

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
                    "FILTER_UNSPECIFIED" => Ok(delegations_by_address_index_request::Filter::Unspecified),
                    "FILTER_ALL_ACTIVE_WITH_NONZERO_BALANCES" => Ok(delegations_by_address_index_request::Filter::AllActiveWithNonzeroBalances),
                    "FILTER_ALL" => Ok(delegations_by_address_index_request::Filter::All),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for DelegationsByAddressIndexResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.DelegationsByAddressIndexResponse", len)?;
        if let Some(v) = self.value_view.as_ref() {
            struct_ser.serialize_field("valueView", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegationsByAddressIndexResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value_view",
            "valueView",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValueView,
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
                            "valueView" | "value_view" => Ok(GeneratedField::ValueView),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegationsByAddressIndexResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.DelegationsByAddressIndexResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DelegationsByAddressIndexResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value_view__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValueView => {
                            if value_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valueView"));
                            }
                            value_view__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DelegationsByAddressIndexResponse {
                    value_view: value_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.DelegationsByAddressIndexResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EphemeralAddressRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address_index.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.EphemeralAddressRequest", len)?;
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EphemeralAddressRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address_index",
            "addressIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AddressIndex,
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
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EphemeralAddressRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.EphemeralAddressRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EphemeralAddressRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EphemeralAddressRequest {
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.EphemeralAddressRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EphemeralAddressResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.EphemeralAddressResponse", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EphemeralAddressResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = EphemeralAddressResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.EphemeralAddressResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EphemeralAddressResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                Ok(EphemeralAddressResponse {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.EphemeralAddressResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FmdParametersRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1.FMDParametersRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FmdParametersRequest {
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
            type Value = FmdParametersRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.FMDParametersRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FmdParametersRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(FmdParametersRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.FMDParametersRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FmdParametersResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.parameters.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.FMDParametersResponse", len)?;
        if let Some(v) = self.parameters.as_ref() {
            struct_ser.serialize_field("parameters", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FmdParametersResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parameters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Parameters,
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
                            "parameters" => Ok(GeneratedField::Parameters),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FmdParametersResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.FMDParametersResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FmdParametersResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut parameters__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Parameters => {
                            if parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameters"));
                            }
                            parameters__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FmdParametersResponse {
                    parameters: parameters__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.FMDParametersResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GasPricesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1.GasPricesRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GasPricesRequest {
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
            type Value = GasPricesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.GasPricesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GasPricesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GasPricesRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.GasPricesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GasPricesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.gas_prices.is_some() {
            len += 1;
        }
        if !self.alt_gas_prices.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.GasPricesResponse", len)?;
        if let Some(v) = self.gas_prices.as_ref() {
            struct_ser.serialize_field("gasPrices", v)?;
        }
        if !self.alt_gas_prices.is_empty() {
            struct_ser.serialize_field("altGasPrices", &self.alt_gas_prices)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GasPricesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "gas_prices",
            "gasPrices",
            "alt_gas_prices",
            "altGasPrices",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GasPrices,
            AltGasPrices,
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
                            "gasPrices" | "gas_prices" => Ok(GeneratedField::GasPrices),
                            "altGasPrices" | "alt_gas_prices" => Ok(GeneratedField::AltGasPrices),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GasPricesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.GasPricesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GasPricesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut gas_prices__ = None;
                let mut alt_gas_prices__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GasPricesResponse {
                    gas_prices: gas_prices__,
                    alt_gas_prices: alt_gas_prices__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.GasPricesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IndexByAddressRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.IndexByAddressRequest", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IndexByAddressRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = IndexByAddressRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.IndexByAddressRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<IndexByAddressRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                Ok(IndexByAddressRequest {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.IndexByAddressRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IndexByAddressResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address_index.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.IndexByAddressResponse", len)?;
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IndexByAddressResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address_index",
            "addressIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AddressIndex,
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
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = IndexByAddressResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.IndexByAddressResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<IndexByAddressResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(IndexByAddressResponse {
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.IndexByAddressResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LatestSwapsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.account_filter.is_some() {
            len += 1;
        }
        if self.pair.is_some() {
            len += 1;
        }
        if self.after_height != 0 {
            len += 1;
        }
        if self.response_limit != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.LatestSwapsRequest", len)?;
        if let Some(v) = self.account_filter.as_ref() {
            struct_ser.serialize_field("accountFilter", v)?;
        }
        if let Some(v) = self.pair.as_ref() {
            struct_ser.serialize_field("pair", v)?;
        }
        if self.after_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("afterHeight", ToString::to_string(&self.after_height).as_str())?;
        }
        if self.response_limit != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("responseLimit", ToString::to_string(&self.response_limit).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LatestSwapsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "account_filter",
            "accountFilter",
            "pair",
            "after_height",
            "afterHeight",
            "response_limit",
            "responseLimit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AccountFilter,
            Pair,
            AfterHeight,
            ResponseLimit,
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
                            "accountFilter" | "account_filter" => Ok(GeneratedField::AccountFilter),
                            "pair" => Ok(GeneratedField::Pair),
                            "afterHeight" | "after_height" => Ok(GeneratedField::AfterHeight),
                            "responseLimit" | "response_limit" => Ok(GeneratedField::ResponseLimit),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LatestSwapsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.LatestSwapsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LatestSwapsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut account_filter__ = None;
                let mut pair__ = None;
                let mut after_height__ = None;
                let mut response_limit__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AccountFilter => {
                            if account_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountFilter"));
                            }
                            account_filter__ = map_.next_value()?;
                        }
                        GeneratedField::Pair => {
                            if pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pair"));
                            }
                            pair__ = map_.next_value()?;
                        }
                        GeneratedField::AfterHeight => {
                            if after_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("afterHeight"));
                            }
                            after_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ResponseLimit => {
                            if response_limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("responseLimit"));
                            }
                            response_limit__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LatestSwapsRequest {
                    account_filter: account_filter__,
                    pair: pair__,
                    after_height: after_height__.unwrap_or_default(),
                    response_limit: response_limit__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.LatestSwapsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LatestSwapsResponse {
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
        if self.input.is_some() {
            len += 1;
        }
        if self.output.is_some() {
            len += 1;
        }
        if self.block_height != 0 {
            len += 1;
        }
        if self.id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.LatestSwapsResponse", len)?;
        if let Some(v) = self.pair.as_ref() {
            struct_ser.serialize_field("pair", v)?;
        }
        if let Some(v) = self.input.as_ref() {
            struct_ser.serialize_field("input", v)?;
        }
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        if self.block_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("blockHeight", ToString::to_string(&self.block_height).as_str())?;
        }
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LatestSwapsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pair",
            "input",
            "output",
            "block_height",
            "blockHeight",
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pair,
            Input,
            Output,
            BlockHeight,
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
                            "pair" => Ok(GeneratedField::Pair),
                            "input" => Ok(GeneratedField::Input),
                            "output" => Ok(GeneratedField::Output),
                            "blockHeight" | "block_height" => Ok(GeneratedField::BlockHeight),
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
            type Value = LatestSwapsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.LatestSwapsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LatestSwapsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pair__ = None;
                let mut input__ = None;
                let mut output__ = None;
                let mut block_height__ = None;
                let mut id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Pair => {
                            if pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pair"));
                            }
                            pair__ = map_.next_value()?;
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
                        GeneratedField::BlockHeight => {
                            if block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHeight"));
                            }
                            block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
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
                Ok(LatestSwapsResponse {
                    pair: pair__,
                    input: input__,
                    output: output__,
                    block_height: block_height__.unwrap_or_default(),
                    id: id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.LatestSwapsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LpPositionBundleRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.subaccount.is_some() {
            len += 1;
        }
        if self.trading_pair.is_some() {
            len += 1;
        }
        if self.position_metadata.is_some() {
            len += 1;
        }
        if self.position_state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.LpPositionBundleRequest", len)?;
        if let Some(v) = self.subaccount.as_ref() {
            struct_ser.serialize_field("subaccount", v)?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.position_metadata.as_ref() {
            struct_ser.serialize_field("positionMetadata", v)?;
        }
        if let Some(v) = self.position_state.as_ref() {
            struct_ser.serialize_field("positionState", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LpPositionBundleRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "subaccount",
            "trading_pair",
            "tradingPair",
            "position_metadata",
            "positionMetadata",
            "position_state",
            "positionState",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Subaccount,
            TradingPair,
            PositionMetadata,
            PositionState,
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
                            "subaccount" => Ok(GeneratedField::Subaccount),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "positionMetadata" | "position_metadata" => Ok(GeneratedField::PositionMetadata),
                            "positionState" | "position_state" => Ok(GeneratedField::PositionState),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LpPositionBundleRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.LpPositionBundleRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LpPositionBundleRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut subaccount__ = None;
                let mut trading_pair__ = None;
                let mut position_metadata__ = None;
                let mut position_state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Subaccount => {
                            if subaccount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subaccount"));
                            }
                            subaccount__ = map_.next_value()?;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::PositionMetadata => {
                            if position_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionMetadata"));
                            }
                            position_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::PositionState => {
                            if position_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionState"));
                            }
                            position_state__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LpPositionBundleRequest {
                    subaccount: subaccount__,
                    trading_pair: trading_pair__,
                    position_metadata: position_metadata__,
                    position_state: position_state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.LpPositionBundleRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LpPositionBundleResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.entries.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.LpPositionBundleResponse", len)?;
        if !self.entries.is_empty() {
            struct_ser.serialize_field("entries", &self.entries)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LpPositionBundleResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "entries",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Entries,
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
                            "entries" => Ok(GeneratedField::Entries),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LpPositionBundleResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.LpPositionBundleResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LpPositionBundleResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut entries__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Entries => {
                            if entries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            entries__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LpPositionBundleResponse {
                    entries: entries__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.LpPositionBundleResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for lp_position_bundle_response::Entry {
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
        if self.subaccount.is_some() {
            len += 1;
        }
        if self.position_metadata.is_some() {
            len += 1;
        }
        if self.position_state.is_some() {
            len += 1;
        }
        if !self.position_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.LpPositionBundleResponse.Entry", len)?;
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.subaccount.as_ref() {
            struct_ser.serialize_field("subaccount", v)?;
        }
        if let Some(v) = self.position_metadata.as_ref() {
            struct_ser.serialize_field("positionMetadata", v)?;
        }
        if let Some(v) = self.position_state.as_ref() {
            struct_ser.serialize_field("positionState", v)?;
        }
        if !self.position_id.is_empty() {
            struct_ser.serialize_field("positionId", &self.position_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for lp_position_bundle_response::Entry {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trading_pair",
            "tradingPair",
            "subaccount",
            "position_metadata",
            "positionMetadata",
            "position_state",
            "positionState",
            "position_id",
            "positionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TradingPair,
            Subaccount,
            PositionMetadata,
            PositionState,
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
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "subaccount" => Ok(GeneratedField::Subaccount),
                            "positionMetadata" | "position_metadata" => Ok(GeneratedField::PositionMetadata),
                            "positionState" | "position_state" => Ok(GeneratedField::PositionState),
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
            type Value = lp_position_bundle_response::Entry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.LpPositionBundleResponse.Entry")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<lp_position_bundle_response::Entry, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trading_pair__ = None;
                let mut subaccount__ = None;
                let mut position_metadata__ = None;
                let mut position_state__ = None;
                let mut position_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Subaccount => {
                            if subaccount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subaccount"));
                            }
                            subaccount__ = map_.next_value()?;
                        }
                        GeneratedField::PositionMetadata => {
                            if position_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionMetadata"));
                            }
                            position_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::PositionState => {
                            if position_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionState"));
                            }
                            position_state__ = map_.next_value()?;
                        }
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
                Ok(lp_position_bundle_response::Entry {
                    trading_pair: trading_pair__,
                    subaccount: subaccount__,
                    position_metadata: position_metadata__,
                    position_state: position_state__,
                    position_id: position_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.LpPositionBundleResponse.Entry", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LpStrategyCatalogRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.subaccount.is_some() {
            len += 1;
        }
        if self.trading_pair.is_some() {
            len += 1;
        }
        if self.position_metadata.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.LpStrategyCatalogRequest", len)?;
        if let Some(v) = self.subaccount.as_ref() {
            struct_ser.serialize_field("subaccount", v)?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.position_metadata.as_ref() {
            struct_ser.serialize_field("positionMetadata", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LpStrategyCatalogRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "subaccount",
            "trading_pair",
            "tradingPair",
            "position_metadata",
            "positionMetadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Subaccount,
            TradingPair,
            PositionMetadata,
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
                            "subaccount" => Ok(GeneratedField::Subaccount),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "positionMetadata" | "position_metadata" => Ok(GeneratedField::PositionMetadata),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LpStrategyCatalogRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.LpStrategyCatalogRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LpStrategyCatalogRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut subaccount__ = None;
                let mut trading_pair__ = None;
                let mut position_metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Subaccount => {
                            if subaccount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subaccount"));
                            }
                            subaccount__ = map_.next_value()?;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::PositionMetadata => {
                            if position_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionMetadata"));
                            }
                            position_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LpStrategyCatalogRequest {
                    subaccount: subaccount__,
                    trading_pair: trading_pair__,
                    position_metadata: position_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.LpStrategyCatalogRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LpStrategyCatalogResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.strategies.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.LpStrategyCatalogResponse", len)?;
        if !self.strategies.is_empty() {
            struct_ser.serialize_field("strategies", &self.strategies)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LpStrategyCatalogResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "strategies",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Strategies,
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
                            "strategies" => Ok(GeneratedField::Strategies),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LpStrategyCatalogResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.LpStrategyCatalogResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LpStrategyCatalogResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut strategies__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Strategies => {
                            if strategies__.is_some() {
                                return Err(serde::de::Error::duplicate_field("strategies"));
                            }
                            strategies__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LpStrategyCatalogResponse {
                    strategies: strategies__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.LpStrategyCatalogResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for lp_strategy_catalog_response::StrategyEntry {
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
        if self.subaccount.is_some() {
            len += 1;
        }
        if self.position_metadata.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.LpStrategyCatalogResponse.StrategyEntry", len)?;
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.subaccount.as_ref() {
            struct_ser.serialize_field("subaccount", v)?;
        }
        if let Some(v) = self.position_metadata.as_ref() {
            struct_ser.serialize_field("positionMetadata", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for lp_strategy_catalog_response::StrategyEntry {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trading_pair",
            "tradingPair",
            "subaccount",
            "position_metadata",
            "positionMetadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TradingPair,
            Subaccount,
            PositionMetadata,
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
                            "subaccount" => Ok(GeneratedField::Subaccount),
                            "positionMetadata" | "position_metadata" => Ok(GeneratedField::PositionMetadata),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = lp_strategy_catalog_response::StrategyEntry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.LpStrategyCatalogResponse.StrategyEntry")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<lp_strategy_catalog_response::StrategyEntry, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trading_pair__ = None;
                let mut subaccount__ = None;
                let mut position_metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Subaccount => {
                            if subaccount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subaccount"));
                            }
                            subaccount__ = map_.next_value()?;
                        }
                        GeneratedField::PositionMetadata => {
                            if position_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionMetadata"));
                            }
                            position_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(lp_strategy_catalog_response::StrategyEntry {
                    trading_pair: trading_pair__,
                    subaccount: subaccount__,
                    position_metadata: position_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.LpStrategyCatalogResponse.StrategyEntry", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LqtVotingNotesRequest {
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
        if self.account_filter.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.LqtVotingNotesRequest", len)?;
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.account_filter.as_ref() {
            struct_ser.serialize_field("accountFilter", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LqtVotingNotesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch_index",
            "epochIndex",
            "account_filter",
            "accountFilter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EpochIndex,
            AccountFilter,
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
                            "accountFilter" | "account_filter" => Ok(GeneratedField::AccountFilter),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LqtVotingNotesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.LqtVotingNotesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LqtVotingNotesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch_index__ = None;
                let mut account_filter__ = None;
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
                        GeneratedField::AccountFilter => {
                            if account_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountFilter"));
                            }
                            account_filter__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LqtVotingNotesRequest {
                    epoch_index: epoch_index__.unwrap_or_default(),
                    account_filter: account_filter__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.LqtVotingNotesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LqtVotingNotesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_record.is_some() {
            len += 1;
        }
        if self.already_voted {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.LqtVotingNotesResponse", len)?;
        if let Some(v) = self.note_record.as_ref() {
            struct_ser.serialize_field("noteRecord", v)?;
        }
        if self.already_voted {
            struct_ser.serialize_field("alreadyVoted", &self.already_voted)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LqtVotingNotesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_record",
            "noteRecord",
            "already_voted",
            "alreadyVoted",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteRecord,
            AlreadyVoted,
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
                            "noteRecord" | "note_record" => Ok(GeneratedField::NoteRecord),
                            "alreadyVoted" | "already_voted" => Ok(GeneratedField::AlreadyVoted),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LqtVotingNotesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.LqtVotingNotesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LqtVotingNotesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_record__ = None;
                let mut already_voted__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NoteRecord => {
                            if note_record__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteRecord"));
                            }
                            note_record__ = map_.next_value()?;
                        }
                        GeneratedField::AlreadyVoted => {
                            if already_voted__.is_some() {
                                return Err(serde::de::Error::duplicate_field("alreadyVoted"));
                            }
                            already_voted__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LqtVotingNotesResponse {
                    note_record: note_record__,
                    already_voted: already_voted__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.LqtVotingNotesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NoteByCommitmentRequest {
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
        if self.await_detection {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.NoteByCommitmentRequest", len)?;
        if let Some(v) = self.note_commitment.as_ref() {
            struct_ser.serialize_field("noteCommitment", v)?;
        }
        if self.await_detection {
            struct_ser.serialize_field("awaitDetection", &self.await_detection)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NoteByCommitmentRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_commitment",
            "noteCommitment",
            "await_detection",
            "awaitDetection",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteCommitment,
            AwaitDetection,
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
                            "awaitDetection" | "await_detection" => Ok(GeneratedField::AwaitDetection),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NoteByCommitmentRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.NoteByCommitmentRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NoteByCommitmentRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_commitment__ = None;
                let mut await_detection__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NoteCommitment => {
                            if note_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitment"));
                            }
                            note_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::AwaitDetection => {
                            if await_detection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awaitDetection"));
                            }
                            await_detection__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NoteByCommitmentRequest {
                    note_commitment: note_commitment__,
                    await_detection: await_detection__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.NoteByCommitmentRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NoteByCommitmentResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.spendable_note.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.NoteByCommitmentResponse", len)?;
        if let Some(v) = self.spendable_note.as_ref() {
            struct_ser.serialize_field("spendableNote", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NoteByCommitmentResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spendable_note",
            "spendableNote",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SpendableNote,
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
                            "spendableNote" | "spendable_note" => Ok(GeneratedField::SpendableNote),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NoteByCommitmentResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.NoteByCommitmentResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NoteByCommitmentResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spendable_note__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SpendableNote => {
                            if spendable_note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendableNote"));
                            }
                            spendable_note__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NoteByCommitmentResponse {
                    spendable_note: spendable_note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.NoteByCommitmentResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NotesForVotingRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.votable_at_height != 0 {
            len += 1;
        }
        if self.address_index.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.NotesForVotingRequest", len)?;
        if self.votable_at_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("votableAtHeight", ToString::to_string(&self.votable_at_height).as_str())?;
        }
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NotesForVotingRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "votable_at_height",
            "votableAtHeight",
            "address_index",
            "addressIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VotableAtHeight,
            AddressIndex,
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
                            "votableAtHeight" | "votable_at_height" => Ok(GeneratedField::VotableAtHeight),
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NotesForVotingRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.NotesForVotingRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NotesForVotingRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut votable_at_height__ = None;
                let mut address_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::VotableAtHeight => {
                            if votable_at_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votableAtHeight"));
                            }
                            votable_at_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NotesForVotingRequest {
                    votable_at_height: votable_at_height__.unwrap_or_default(),
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.NotesForVotingRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NotesForVotingResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_record.is_some() {
            len += 1;
        }
        if self.identity_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.NotesForVotingResponse", len)?;
        if let Some(v) = self.note_record.as_ref() {
            struct_ser.serialize_field("noteRecord", v)?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NotesForVotingResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_record",
            "noteRecord",
            "identity_key",
            "identityKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteRecord,
            IdentityKey,
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
                            "noteRecord" | "note_record" => Ok(GeneratedField::NoteRecord),
                            "identityKey" | "identity_key" => Ok(GeneratedField::IdentityKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NotesForVotingResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.NotesForVotingResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NotesForVotingResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_record__ = None;
                let mut identity_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NoteRecord => {
                            if note_record__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteRecord"));
                            }
                            note_record__ = map_.next_value()?;
                        }
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NotesForVotingResponse {
                    note_record: note_record__,
                    identity_key: identity_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.NotesForVotingResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NotesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.include_spent {
            len += 1;
        }
        if self.asset_id.is_some() {
            len += 1;
        }
        if self.address_index.is_some() {
            len += 1;
        }
        if self.amount_to_spend.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.NotesRequest", len)?;
        if self.include_spent {
            struct_ser.serialize_field("includeSpent", &self.include_spent)?;
        }
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        if let Some(v) = self.amount_to_spend.as_ref() {
            struct_ser.serialize_field("amountToSpend", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NotesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "include_spent",
            "includeSpent",
            "asset_id",
            "assetId",
            "address_index",
            "addressIndex",
            "amount_to_spend",
            "amountToSpend",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IncludeSpent,
            AssetId,
            AddressIndex,
            AmountToSpend,
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
                            "includeSpent" | "include_spent" => Ok(GeneratedField::IncludeSpent),
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            "amountToSpend" | "amount_to_spend" => Ok(GeneratedField::AmountToSpend),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NotesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.NotesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NotesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut include_spent__ = None;
                let mut asset_id__ = None;
                let mut address_index__ = None;
                let mut amount_to_spend__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IncludeSpent => {
                            if include_spent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeSpent"));
                            }
                            include_spent__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map_.next_value()?;
                        }
                        GeneratedField::AmountToSpend => {
                            if amount_to_spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amountToSpend"));
                            }
                            amount_to_spend__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NotesRequest {
                    include_spent: include_spent__.unwrap_or_default(),
                    asset_id: asset_id__,
                    address_index: address_index__,
                    amount_to_spend: amount_to_spend__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.NotesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NotesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_record.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.NotesResponse", len)?;
        if let Some(v) = self.note_record.as_ref() {
            struct_ser.serialize_field("noteRecord", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NotesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_record",
            "noteRecord",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteRecord,
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
                            "noteRecord" | "note_record" => Ok(GeneratedField::NoteRecord),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NotesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.NotesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NotesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_record__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NoteRecord => {
                            if note_record__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteRecord"));
                            }
                            note_record__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NotesResponse {
                    note_record: note_record__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.NotesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NullifierStatusRequest {
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
        if self.await_detection {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.NullifierStatusRequest", len)?;
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if self.await_detection {
            struct_ser.serialize_field("awaitDetection", &self.await_detection)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NullifierStatusRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "nullifier",
            "await_detection",
            "awaitDetection",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Nullifier,
            AwaitDetection,
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
                            "awaitDetection" | "await_detection" => Ok(GeneratedField::AwaitDetection),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NullifierStatusRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.NullifierStatusRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NullifierStatusRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut nullifier__ = None;
                let mut await_detection__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map_.next_value()?;
                        }
                        GeneratedField::AwaitDetection => {
                            if await_detection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awaitDetection"));
                            }
                            await_detection__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NullifierStatusRequest {
                    nullifier: nullifier__,
                    await_detection: await_detection__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.NullifierStatusRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NullifierStatusResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.spent {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.NullifierStatusResponse", len)?;
        if self.spent {
            struct_ser.serialize_field("spent", &self.spent)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NullifierStatusResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spent",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spent,
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
                            "spent" => Ok(GeneratedField::Spent),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NullifierStatusResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.NullifierStatusResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NullifierStatusResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spent__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Spent => {
                            if spent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spent"));
                            }
                            spent__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NullifierStatusResponse {
                    spent: spent__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.NullifierStatusResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OwnedPositionIdsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.position_state.is_some() {
            len += 1;
        }
        if self.trading_pair.is_some() {
            len += 1;
        }
        if self.subaccount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.OwnedPositionIdsRequest", len)?;
        if let Some(v) = self.position_state.as_ref() {
            struct_ser.serialize_field("positionState", v)?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        if let Some(v) = self.subaccount.as_ref() {
            struct_ser.serialize_field("subaccount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OwnedPositionIdsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position_state",
            "positionState",
            "trading_pair",
            "tradingPair",
            "subaccount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionState,
            TradingPair,
            Subaccount,
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
                            "positionState" | "position_state" => Ok(GeneratedField::PositionState),
                            "tradingPair" | "trading_pair" => Ok(GeneratedField::TradingPair),
                            "subaccount" => Ok(GeneratedField::Subaccount),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OwnedPositionIdsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.OwnedPositionIdsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OwnedPositionIdsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_state__ = None;
                let mut trading_pair__ = None;
                let mut subaccount__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionState => {
                            if position_state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionState"));
                            }
                            position_state__ = map_.next_value()?;
                        }
                        GeneratedField::TradingPair => {
                            if trading_pair__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tradingPair"));
                            }
                            trading_pair__ = map_.next_value()?;
                        }
                        GeneratedField::Subaccount => {
                            if subaccount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subaccount"));
                            }
                            subaccount__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OwnedPositionIdsRequest {
                    position_state: position_state__,
                    trading_pair: trading_pair__,
                    subaccount: subaccount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.OwnedPositionIdsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OwnedPositionIdsResponse {
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
        if self.subaccount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.OwnedPositionIdsResponse", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.subaccount.as_ref() {
            struct_ser.serialize_field("subaccount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OwnedPositionIdsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position_id",
            "positionId",
            "subaccount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionId,
            Subaccount,
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
                            "subaccount" => Ok(GeneratedField::Subaccount),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OwnedPositionIdsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.OwnedPositionIdsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OwnedPositionIdsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut subaccount__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::Subaccount => {
                            if subaccount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subaccount"));
                            }
                            subaccount__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OwnedPositionIdsResponse {
                    position_id: position_id__,
                    subaccount: subaccount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.OwnedPositionIdsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendableNoteRecord {
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
        if self.note.is_some() {
            len += 1;
        }
        if self.address_index.is_some() {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.height_created != 0 {
            len += 1;
        }
        if self.height_spent != 0 {
            len += 1;
        }
        if self.position != 0 {
            len += 1;
        }
        if self.source.is_some() {
            len += 1;
        }
        if self.return_address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.SpendableNoteRecord", len)?;
        if let Some(v) = self.note_commitment.as_ref() {
            struct_ser.serialize_field("noteCommitment", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if self.height_created != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("heightCreated", ToString::to_string(&self.height_created).as_str())?;
        }
        if self.height_spent != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("heightSpent", ToString::to_string(&self.height_spent).as_str())?;
        }
        if self.position != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("position", ToString::to_string(&self.position).as_str())?;
        }
        if let Some(v) = self.source.as_ref() {
            struct_ser.serialize_field("source", v)?;
        }
        if let Some(v) = self.return_address.as_ref() {
            struct_ser.serialize_field("returnAddress", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendableNoteRecord {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_commitment",
            "noteCommitment",
            "note",
            "address_index",
            "addressIndex",
            "nullifier",
            "height_created",
            "heightCreated",
            "height_spent",
            "heightSpent",
            "position",
            "source",
            "return_address",
            "returnAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteCommitment,
            Note,
            AddressIndex,
            Nullifier,
            HeightCreated,
            HeightSpent,
            Position,
            Source,
            ReturnAddress,
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
                            "note" => Ok(GeneratedField::Note),
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "heightCreated" | "height_created" => Ok(GeneratedField::HeightCreated),
                            "heightSpent" | "height_spent" => Ok(GeneratedField::HeightSpent),
                            "position" => Ok(GeneratedField::Position),
                            "source" => Ok(GeneratedField::Source),
                            "returnAddress" | "return_address" => Ok(GeneratedField::ReturnAddress),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendableNoteRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.SpendableNoteRecord")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SpendableNoteRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_commitment__ = None;
                let mut note__ = None;
                let mut address_index__ = None;
                let mut nullifier__ = None;
                let mut height_created__ = None;
                let mut height_spent__ = None;
                let mut position__ = None;
                let mut source__ = None;
                let mut return_address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NoteCommitment => {
                            if note_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitment"));
                            }
                            note_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map_.next_value()?;
                        }
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map_.next_value()?;
                        }
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map_.next_value()?;
                        }
                        GeneratedField::HeightCreated => {
                            if height_created__.is_some() {
                                return Err(serde::de::Error::duplicate_field("heightCreated"));
                            }
                            height_created__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::HeightSpent => {
                            if height_spent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("heightSpent"));
                            }
                            height_spent__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Source => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source__ = map_.next_value()?;
                        }
                        GeneratedField::ReturnAddress => {
                            if return_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("returnAddress"));
                            }
                            return_address__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SpendableNoteRecord {
                    note_commitment: note_commitment__,
                    note: note__,
                    address_index: address_index__,
                    nullifier: nullifier__,
                    height_created: height_created__.unwrap_or_default(),
                    height_spent: height_spent__.unwrap_or_default(),
                    position: position__.unwrap_or_default(),
                    source: source__,
                    return_address: return_address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.SpendableNoteRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StatusRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1.StatusRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StatusRequest {
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
            type Value = StatusRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.StatusRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<StatusRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(StatusRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.StatusRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StatusResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.full_sync_height != 0 {
            len += 1;
        }
        if self.partial_sync_height != 0 {
            len += 1;
        }
        if self.catching_up {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.StatusResponse", len)?;
        if self.full_sync_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("fullSyncHeight", ToString::to_string(&self.full_sync_height).as_str())?;
        }
        if self.partial_sync_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("partialSyncHeight", ToString::to_string(&self.partial_sync_height).as_str())?;
        }
        if self.catching_up {
            struct_ser.serialize_field("catchingUp", &self.catching_up)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StatusResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "full_sync_height",
            "fullSyncHeight",
            "partial_sync_height",
            "partialSyncHeight",
            "catching_up",
            "catchingUp",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FullSyncHeight,
            PartialSyncHeight,
            CatchingUp,
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
                            "fullSyncHeight" | "full_sync_height" => Ok(GeneratedField::FullSyncHeight),
                            "partialSyncHeight" | "partial_sync_height" => Ok(GeneratedField::PartialSyncHeight),
                            "catchingUp" | "catching_up" => Ok(GeneratedField::CatchingUp),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StatusResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.StatusResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<StatusResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut full_sync_height__ = None;
                let mut partial_sync_height__ = None;
                let mut catching_up__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FullSyncHeight => {
                            if full_sync_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullSyncHeight"));
                            }
                            full_sync_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PartialSyncHeight => {
                            if partial_sync_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("partialSyncHeight"));
                            }
                            partial_sync_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CatchingUp => {
                            if catching_up__.is_some() {
                                return Err(serde::de::Error::duplicate_field("catchingUp"));
                            }
                            catching_up__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(StatusResponse {
                    full_sync_height: full_sync_height__.unwrap_or_default(),
                    partial_sync_height: partial_sync_height__.unwrap_or_default(),
                    catching_up: catching_up__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.StatusResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StatusStreamRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1.StatusStreamRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StatusStreamRequest {
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
            type Value = StatusStreamRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.StatusStreamRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<StatusStreamRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(StatusStreamRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.StatusStreamRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StatusStreamResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.latest_known_block_height != 0 {
            len += 1;
        }
        if self.full_sync_height != 0 {
            len += 1;
        }
        if self.partial_sync_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.StatusStreamResponse", len)?;
        if self.latest_known_block_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("latestKnownBlockHeight", ToString::to_string(&self.latest_known_block_height).as_str())?;
        }
        if self.full_sync_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("fullSyncHeight", ToString::to_string(&self.full_sync_height).as_str())?;
        }
        if self.partial_sync_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("partialSyncHeight", ToString::to_string(&self.partial_sync_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StatusStreamResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "latest_known_block_height",
            "latestKnownBlockHeight",
            "full_sync_height",
            "fullSyncHeight",
            "partial_sync_height",
            "partialSyncHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LatestKnownBlockHeight,
            FullSyncHeight,
            PartialSyncHeight,
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
                            "latestKnownBlockHeight" | "latest_known_block_height" => Ok(GeneratedField::LatestKnownBlockHeight),
                            "fullSyncHeight" | "full_sync_height" => Ok(GeneratedField::FullSyncHeight),
                            "partialSyncHeight" | "partial_sync_height" => Ok(GeneratedField::PartialSyncHeight),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StatusStreamResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.StatusStreamResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<StatusStreamResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut latest_known_block_height__ = None;
                let mut full_sync_height__ = None;
                let mut partial_sync_height__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::LatestKnownBlockHeight => {
                            if latest_known_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("latestKnownBlockHeight"));
                            }
                            latest_known_block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::FullSyncHeight => {
                            if full_sync_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullSyncHeight"));
                            }
                            full_sync_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PartialSyncHeight => {
                            if partial_sync_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("partialSyncHeight"));
                            }
                            partial_sync_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(StatusStreamResponse {
                    latest_known_block_height: latest_known_block_height__.unwrap_or_default(),
                    full_sync_height: full_sync_height__.unwrap_or_default(),
                    partial_sync_height: partial_sync_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.StatusStreamResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapByCommitmentRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_commitment.is_some() {
            len += 1;
        }
        if self.await_detection {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.SwapByCommitmentRequest", len)?;
        if let Some(v) = self.swap_commitment.as_ref() {
            struct_ser.serialize_field("swapCommitment", v)?;
        }
        if self.await_detection {
            struct_ser.serialize_field("awaitDetection", &self.await_detection)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapByCommitmentRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_commitment",
            "swapCommitment",
            "await_detection",
            "awaitDetection",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapCommitment,
            AwaitDetection,
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
                            "swapCommitment" | "swap_commitment" => Ok(GeneratedField::SwapCommitment),
                            "awaitDetection" | "await_detection" => Ok(GeneratedField::AwaitDetection),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapByCommitmentRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.SwapByCommitmentRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapByCommitmentRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_commitment__ = None;
                let mut await_detection__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapCommitment => {
                            if swap_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapCommitment"));
                            }
                            swap_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::AwaitDetection => {
                            if await_detection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awaitDetection"));
                            }
                            await_detection__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapByCommitmentRequest {
                    swap_commitment: swap_commitment__,
                    await_detection: await_detection__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.SwapByCommitmentRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapByCommitmentResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.SwapByCommitmentResponse", len)?;
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapByCommitmentResponse {
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
            type Value = SwapByCommitmentResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.SwapByCommitmentResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapByCommitmentResponse, V::Error>
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
                Ok(SwapByCommitmentResponse {
                    swap: swap__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.SwapByCommitmentResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapRecord {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_commitment.is_some() {
            len += 1;
        }
        if self.swap.is_some() {
            len += 1;
        }
        if self.position != 0 {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.output_data.is_some() {
            len += 1;
        }
        if self.height_claimed != 0 {
            len += 1;
        }
        if self.source.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.SwapRecord", len)?;
        if let Some(v) = self.swap_commitment.as_ref() {
            struct_ser.serialize_field("swapCommitment", v)?;
        }
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        if self.position != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("position", ToString::to_string(&self.position).as_str())?;
        }
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if let Some(v) = self.output_data.as_ref() {
            struct_ser.serialize_field("outputData", v)?;
        }
        if self.height_claimed != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("heightClaimed", ToString::to_string(&self.height_claimed).as_str())?;
        }
        if let Some(v) = self.source.as_ref() {
            struct_ser.serialize_field("source", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapRecord {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_commitment",
            "swapCommitment",
            "swap",
            "position",
            "nullifier",
            "output_data",
            "outputData",
            "height_claimed",
            "heightClaimed",
            "source",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapCommitment,
            Swap,
            Position,
            Nullifier,
            OutputData,
            HeightClaimed,
            Source,
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
                            "swapCommitment" | "swap_commitment" => Ok(GeneratedField::SwapCommitment),
                            "swap" => Ok(GeneratedField::Swap),
                            "position" => Ok(GeneratedField::Position),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "outputData" | "output_data" => Ok(GeneratedField::OutputData),
                            "heightClaimed" | "height_claimed" => Ok(GeneratedField::HeightClaimed),
                            "source" => Ok(GeneratedField::Source),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.SwapRecord")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_commitment__ = None;
                let mut swap__ = None;
                let mut position__ = None;
                let mut nullifier__ = None;
                let mut output_data__ = None;
                let mut height_claimed__ = None;
                let mut source__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapCommitment => {
                            if swap_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapCommitment"));
                            }
                            swap_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = map_.next_value()?;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map_.next_value()?;
                        }
                        GeneratedField::OutputData => {
                            if output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputData"));
                            }
                            output_data__ = map_.next_value()?;
                        }
                        GeneratedField::HeightClaimed => {
                            if height_claimed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("heightClaimed"));
                            }
                            height_claimed__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Source => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapRecord {
                    swap_commitment: swap_commitment__,
                    swap: swap__,
                    position: position__.unwrap_or_default(),
                    nullifier: nullifier__,
                    output_data: output_data__,
                    height_claimed: height_claimed__.unwrap_or_default(),
                    source: source__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.SwapRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TournamentVotesRequest {
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
        if self.block_height != 0 {
            len += 1;
        }
        if self.account_filter.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TournamentVotesRequest", len)?;
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if self.block_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("blockHeight", ToString::to_string(&self.block_height).as_str())?;
        }
        if let Some(v) = self.account_filter.as_ref() {
            struct_ser.serialize_field("accountFilter", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TournamentVotesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch_index",
            "epochIndex",
            "block_height",
            "blockHeight",
            "account_filter",
            "accountFilter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EpochIndex,
            BlockHeight,
            AccountFilter,
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
                            "blockHeight" | "block_height" => Ok(GeneratedField::BlockHeight),
                            "accountFilter" | "account_filter" => Ok(GeneratedField::AccountFilter),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TournamentVotesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TournamentVotesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TournamentVotesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch_index__ = None;
                let mut block_height__ = None;
                let mut account_filter__ = None;
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
                        GeneratedField::BlockHeight => {
                            if block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHeight"));
                            }
                            block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AccountFilter => {
                            if account_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountFilter"));
                            }
                            account_filter__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TournamentVotesRequest {
                    epoch_index: epoch_index__.unwrap_or_default(),
                    block_height: block_height__.unwrap_or_default(),
                    account_filter: account_filter__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TournamentVotesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TournamentVotesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.votes.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TournamentVotesResponse", len)?;
        if !self.votes.is_empty() {
            struct_ser.serialize_field("votes", &self.votes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TournamentVotesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "votes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Votes,
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
                            "votes" => Ok(GeneratedField::Votes),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TournamentVotesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TournamentVotesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TournamentVotesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut votes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Votes => {
                            if votes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votes"));
                            }
                            votes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TournamentVotesResponse {
                    votes: votes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TournamentVotesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for tournament_votes_response::Vote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.incentivized_asset.is_some() {
            len += 1;
        }
        if self.vote_power.is_some() {
            len += 1;
        }
        if self.reward.is_some() {
            len += 1;
        }
        if self.transaction.is_some() {
            len += 1;
        }
        if self.epoch_index != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TournamentVotesResponse.Vote", len)?;
        if let Some(v) = self.incentivized_asset.as_ref() {
            struct_ser.serialize_field("incentivizedAsset", v)?;
        }
        if let Some(v) = self.vote_power.as_ref() {
            struct_ser.serialize_field("votePower", v)?;
        }
        if let Some(v) = self.reward.as_ref() {
            struct_ser.serialize_field("reward", v)?;
        }
        if let Some(v) = self.transaction.as_ref() {
            struct_ser.serialize_field("transaction", v)?;
        }
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for tournament_votes_response::Vote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "incentivized_asset",
            "incentivizedAsset",
            "vote_power",
            "votePower",
            "reward",
            "transaction",
            "epoch_index",
            "epochIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IncentivizedAsset,
            VotePower,
            Reward,
            Transaction,
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
                            "incentivizedAsset" | "incentivized_asset" => Ok(GeneratedField::IncentivizedAsset),
                            "votePower" | "vote_power" => Ok(GeneratedField::VotePower),
                            "reward" => Ok(GeneratedField::Reward),
                            "transaction" => Ok(GeneratedField::Transaction),
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
            type Value = tournament_votes_response::Vote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TournamentVotesResponse.Vote")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<tournament_votes_response::Vote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut incentivized_asset__ = None;
                let mut vote_power__ = None;
                let mut reward__ = None;
                let mut transaction__ = None;
                let mut epoch_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IncentivizedAsset => {
                            if incentivized_asset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivizedAsset"));
                            }
                            incentivized_asset__ = map_.next_value()?;
                        }
                        GeneratedField::VotePower => {
                            if vote_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votePower"));
                            }
                            vote_power__ = map_.next_value()?;
                        }
                        GeneratedField::Reward => {
                            if reward__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reward"));
                            }
                            reward__ = map_.next_value()?;
                        }
                        GeneratedField::Transaction => {
                            if transaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transaction"));
                            }
                            transaction__ = map_.next_value()?;
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
                Ok(tournament_votes_response::Vote {
                    incentivized_asset: incentivized_asset__,
                    vote_power: vote_power__,
                    reward: reward__,
                    transaction: transaction__,
                    epoch_index: epoch_index__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TournamentVotesResponse.Vote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionInfo {
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
        if self.id.is_some() {
            len += 1;
        }
        if self.transaction.is_some() {
            len += 1;
        }
        if self.perspective.is_some() {
            len += 1;
        }
        if self.view.is_some() {
            len += 1;
        }
        if self.summary.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionInfo", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if let Some(v) = self.transaction.as_ref() {
            struct_ser.serialize_field("transaction", v)?;
        }
        if let Some(v) = self.perspective.as_ref() {
            struct_ser.serialize_field("perspective", v)?;
        }
        if let Some(v) = self.view.as_ref() {
            struct_ser.serialize_field("view", v)?;
        }
        if let Some(v) = self.summary.as_ref() {
            struct_ser.serialize_field("summary", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "id",
            "transaction",
            "perspective",
            "view",
            "summary",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            Id,
            Transaction,
            Perspective,
            View,
            Summary,
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
                            "id" => Ok(GeneratedField::Id),
                            "transaction" => Ok(GeneratedField::Transaction),
                            "perspective" => Ok(GeneratedField::Perspective),
                            "view" => Ok(GeneratedField::View),
                            "summary" => Ok(GeneratedField::Summary),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut id__ = None;
                let mut transaction__ = None;
                let mut perspective__ = None;
                let mut view__ = None;
                let mut summary__ = None;
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
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = map_.next_value()?;
                        }
                        GeneratedField::Transaction => {
                            if transaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transaction"));
                            }
                            transaction__ = map_.next_value()?;
                        }
                        GeneratedField::Perspective => {
                            if perspective__.is_some() {
                                return Err(serde::de::Error::duplicate_field("perspective"));
                            }
                            perspective__ = map_.next_value()?;
                        }
                        GeneratedField::View => {
                            if view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("view"));
                            }
                            view__ = map_.next_value()?;
                        }
                        GeneratedField::Summary => {
                            if summary__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            summary__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionInfo {
                    height: height__.unwrap_or_default(),
                    id: id__,
                    transaction: transaction__,
                    perspective: perspective__,
                    view: view__,
                    summary: summary__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionInfoByHashRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionInfoByHashRequest", len)?;
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionInfoByHashRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = TransactionInfoByHashRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionInfoByHashRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionInfoByHashRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                Ok(TransactionInfoByHashRequest {
                    id: id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionInfoByHashRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionInfoByHashResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tx_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionInfoByHashResponse", len)?;
        if let Some(v) = self.tx_info.as_ref() {
            struct_ser.serialize_field("txInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionInfoByHashResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tx_info",
            "txInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TxInfo,
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
                            "txInfo" | "tx_info" => Ok(GeneratedField::TxInfo),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionInfoByHashResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionInfoByHashResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionInfoByHashResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tx_info__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TxInfo => {
                            if tx_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txInfo"));
                            }
                            tx_info__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionInfoByHashResponse {
                    tx_info: tx_info__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionInfoByHashResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionInfoRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionInfoRequest", len)?;
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
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionInfoRequest {
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
            type Value = TransactionInfoRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionInfoRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionInfoRequest, V::Error>
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
                Ok(TransactionInfoRequest {
                    start_height: start_height__.unwrap_or_default(),
                    end_height: end_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionInfoRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionInfoResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tx_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionInfoResponse", len)?;
        if let Some(v) = self.tx_info.as_ref() {
            struct_ser.serialize_field("txInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionInfoResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tx_info",
            "txInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TxInfo,
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
                            "txInfo" | "tx_info" => Ok(GeneratedField::TxInfo),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionInfoResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionInfoResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionInfoResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tx_info__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TxInfo => {
                            if tx_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txInfo"));
                            }
                            tx_info__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionInfoResponse {
                    tx_info: tx_info__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionInfoResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionPlannerRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.expiry_height != 0 {
            len += 1;
        }
        if self.memo.is_some() {
            len += 1;
        }
        if self.source.is_some() {
            len += 1;
        }
        if !self.outputs.is_empty() {
            len += 1;
        }
        if !self.spends.is_empty() {
            len += 1;
        }
        if !self.swaps.is_empty() {
            len += 1;
        }
        if !self.swap_claims.is_empty() {
            len += 1;
        }
        if !self.delegations.is_empty() {
            len += 1;
        }
        if !self.undelegations.is_empty() {
            len += 1;
        }
        if !self.undelegation_claims.is_empty() {
            len += 1;
        }
        if !self.ibc_relay_actions.is_empty() {
            len += 1;
        }
        if !self.ics20_withdrawals.is_empty() {
            len += 1;
        }
        if !self.position_opens.is_empty() {
            len += 1;
        }
        if !self.position_closes.is_empty() {
            len += 1;
        }
        if !self.position_withdraws.is_empty() {
            len += 1;
        }
        if !self.dutch_auction_schedule_actions.is_empty() {
            len += 1;
        }
        if !self.dutch_auction_end_actions.is_empty() {
            len += 1;
        }
        if !self.dutch_auction_withdraw_actions.is_empty() {
            len += 1;
        }
        if !self.delegator_votes.is_empty() {
            len += 1;
        }
        if !self.action_liquidity_tournament_vote.is_empty() {
            len += 1;
        }
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.epoch.is_some() {
            len += 1;
        }
        if self.fee_mode.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest", len)?;
        if self.expiry_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("expiryHeight", ToString::to_string(&self.expiry_height).as_str())?;
        }
        if let Some(v) = self.memo.as_ref() {
            struct_ser.serialize_field("memo", v)?;
        }
        if let Some(v) = self.source.as_ref() {
            struct_ser.serialize_field("source", v)?;
        }
        if !self.outputs.is_empty() {
            struct_ser.serialize_field("outputs", &self.outputs)?;
        }
        if !self.spends.is_empty() {
            struct_ser.serialize_field("spends", &self.spends)?;
        }
        if !self.swaps.is_empty() {
            struct_ser.serialize_field("swaps", &self.swaps)?;
        }
        if !self.swap_claims.is_empty() {
            struct_ser.serialize_field("swapClaims", &self.swap_claims)?;
        }
        if !self.delegations.is_empty() {
            struct_ser.serialize_field("delegations", &self.delegations)?;
        }
        if !self.undelegations.is_empty() {
            struct_ser.serialize_field("undelegations", &self.undelegations)?;
        }
        if !self.undelegation_claims.is_empty() {
            struct_ser.serialize_field("undelegationClaims", &self.undelegation_claims)?;
        }
        if !self.ibc_relay_actions.is_empty() {
            struct_ser.serialize_field("ibcRelayActions", &self.ibc_relay_actions)?;
        }
        if !self.ics20_withdrawals.is_empty() {
            struct_ser.serialize_field("ics20Withdrawals", &self.ics20_withdrawals)?;
        }
        if !self.position_opens.is_empty() {
            struct_ser.serialize_field("positionOpens", &self.position_opens)?;
        }
        if !self.position_closes.is_empty() {
            struct_ser.serialize_field("positionCloses", &self.position_closes)?;
        }
        if !self.position_withdraws.is_empty() {
            struct_ser.serialize_field("positionWithdraws", &self.position_withdraws)?;
        }
        if !self.dutch_auction_schedule_actions.is_empty() {
            struct_ser.serialize_field("dutchAuctionScheduleActions", &self.dutch_auction_schedule_actions)?;
        }
        if !self.dutch_auction_end_actions.is_empty() {
            struct_ser.serialize_field("dutchAuctionEndActions", &self.dutch_auction_end_actions)?;
        }
        if !self.dutch_auction_withdraw_actions.is_empty() {
            struct_ser.serialize_field("dutchAuctionWithdrawActions", &self.dutch_auction_withdraw_actions)?;
        }
        if !self.delegator_votes.is_empty() {
            struct_ser.serialize_field("delegatorVotes", &self.delegator_votes)?;
        }
        if !self.action_liquidity_tournament_vote.is_empty() {
            struct_ser.serialize_field("actionLiquidityTournamentVote", &self.action_liquidity_tournament_vote)?;
        }
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.epoch.as_ref() {
            struct_ser.serialize_field("epoch", v)?;
        }
        if let Some(v) = self.fee_mode.as_ref() {
            match v {
                transaction_planner_request::FeeMode::AutoFee(v) => {
                    struct_ser.serialize_field("autoFee", v)?;
                }
                transaction_planner_request::FeeMode::ManualFee(v) => {
                    struct_ser.serialize_field("manualFee", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionPlannerRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "expiry_height",
            "expiryHeight",
            "memo",
            "source",
            "outputs",
            "spends",
            "swaps",
            "swap_claims",
            "swapClaims",
            "delegations",
            "undelegations",
            "undelegation_claims",
            "undelegationClaims",
            "ibc_relay_actions",
            "ibcRelayActions",
            "ics20_withdrawals",
            "ics20Withdrawals",
            "position_opens",
            "positionOpens",
            "position_closes",
            "positionCloses",
            "position_withdraws",
            "positionWithdraws",
            "dutch_auction_schedule_actions",
            "dutchAuctionScheduleActions",
            "dutch_auction_end_actions",
            "dutchAuctionEndActions",
            "dutch_auction_withdraw_actions",
            "dutchAuctionWithdrawActions",
            "delegator_votes",
            "delegatorVotes",
            "action_liquidity_tournament_vote",
            "actionLiquidityTournamentVote",
            "epoch_index",
            "epochIndex",
            "epoch",
            "auto_fee",
            "autoFee",
            "manual_fee",
            "manualFee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExpiryHeight,
            Memo,
            Source,
            Outputs,
            Spends,
            Swaps,
            SwapClaims,
            Delegations,
            Undelegations,
            UndelegationClaims,
            IbcRelayActions,
            Ics20Withdrawals,
            PositionOpens,
            PositionCloses,
            PositionWithdraws,
            DutchAuctionScheduleActions,
            DutchAuctionEndActions,
            DutchAuctionWithdrawActions,
            DelegatorVotes,
            ActionLiquidityTournamentVote,
            EpochIndex,
            Epoch,
            AutoFee,
            ManualFee,
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
                            "expiryHeight" | "expiry_height" => Ok(GeneratedField::ExpiryHeight),
                            "memo" => Ok(GeneratedField::Memo),
                            "source" => Ok(GeneratedField::Source),
                            "outputs" => Ok(GeneratedField::Outputs),
                            "spends" => Ok(GeneratedField::Spends),
                            "swaps" => Ok(GeneratedField::Swaps),
                            "swapClaims" | "swap_claims" => Ok(GeneratedField::SwapClaims),
                            "delegations" => Ok(GeneratedField::Delegations),
                            "undelegations" => Ok(GeneratedField::Undelegations),
                            "undelegationClaims" | "undelegation_claims" => Ok(GeneratedField::UndelegationClaims),
                            "ibcRelayActions" | "ibc_relay_actions" => Ok(GeneratedField::IbcRelayActions),
                            "ics20Withdrawals" | "ics20_withdrawals" => Ok(GeneratedField::Ics20Withdrawals),
                            "positionOpens" | "position_opens" => Ok(GeneratedField::PositionOpens),
                            "positionCloses" | "position_closes" => Ok(GeneratedField::PositionCloses),
                            "positionWithdraws" | "position_withdraws" => Ok(GeneratedField::PositionWithdraws),
                            "dutchAuctionScheduleActions" | "dutch_auction_schedule_actions" => Ok(GeneratedField::DutchAuctionScheduleActions),
                            "dutchAuctionEndActions" | "dutch_auction_end_actions" => Ok(GeneratedField::DutchAuctionEndActions),
                            "dutchAuctionWithdrawActions" | "dutch_auction_withdraw_actions" => Ok(GeneratedField::DutchAuctionWithdrawActions),
                            "delegatorVotes" | "delegator_votes" => Ok(GeneratedField::DelegatorVotes),
                            "actionLiquidityTournamentVote" | "action_liquidity_tournament_vote" => Ok(GeneratedField::ActionLiquidityTournamentVote),
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "epoch" => Ok(GeneratedField::Epoch),
                            "autoFee" | "auto_fee" => Ok(GeneratedField::AutoFee),
                            "manualFee" | "manual_fee" => Ok(GeneratedField::ManualFee),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionPlannerRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionPlannerRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut expiry_height__ = None;
                let mut memo__ = None;
                let mut source__ = None;
                let mut outputs__ = None;
                let mut spends__ = None;
                let mut swaps__ = None;
                let mut swap_claims__ = None;
                let mut delegations__ = None;
                let mut undelegations__ = None;
                let mut undelegation_claims__ = None;
                let mut ibc_relay_actions__ = None;
                let mut ics20_withdrawals__ = None;
                let mut position_opens__ = None;
                let mut position_closes__ = None;
                let mut position_withdraws__ = None;
                let mut dutch_auction_schedule_actions__ = None;
                let mut dutch_auction_end_actions__ = None;
                let mut dutch_auction_withdraw_actions__ = None;
                let mut delegator_votes__ = None;
                let mut action_liquidity_tournament_vote__ = None;
                let mut epoch_index__ = None;
                let mut epoch__ = None;
                let mut fee_mode__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ExpiryHeight => {
                            if expiry_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expiryHeight"));
                            }
                            expiry_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Memo => {
                            if memo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("memo"));
                            }
                            memo__ = map_.next_value()?;
                        }
                        GeneratedField::Source => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source__ = map_.next_value()?;
                        }
                        GeneratedField::Outputs => {
                            if outputs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputs"));
                            }
                            outputs__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Spends => {
                            if spends__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spends"));
                            }
                            spends__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Swaps => {
                            if swaps__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swaps"));
                            }
                            swaps__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SwapClaims => {
                            if swap_claims__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaims"));
                            }
                            swap_claims__ = Some(map_.next_value()?);
                        }
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
                        GeneratedField::UndelegationClaims => {
                            if undelegation_claims__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegationClaims"));
                            }
                            undelegation_claims__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IbcRelayActions => {
                            if ibc_relay_actions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcRelayActions"));
                            }
                            ibc_relay_actions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Ics20Withdrawals => {
                            if ics20_withdrawals__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ics20Withdrawals"));
                            }
                            ics20_withdrawals__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PositionOpens => {
                            if position_opens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionOpens"));
                            }
                            position_opens__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PositionCloses => {
                            if position_closes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionCloses"));
                            }
                            position_closes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PositionWithdraws => {
                            if position_withdraws__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionWithdraws"));
                            }
                            position_withdraws__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DutchAuctionScheduleActions => {
                            if dutch_auction_schedule_actions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dutchAuctionScheduleActions"));
                            }
                            dutch_auction_schedule_actions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DutchAuctionEndActions => {
                            if dutch_auction_end_actions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dutchAuctionEndActions"));
                            }
                            dutch_auction_end_actions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DutchAuctionWithdrawActions => {
                            if dutch_auction_withdraw_actions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dutchAuctionWithdrawActions"));
                            }
                            dutch_auction_withdraw_actions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DelegatorVotes => {
                            if delegator_votes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVotes"));
                            }
                            delegator_votes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ActionLiquidityTournamentVote => {
                            if action_liquidity_tournament_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionLiquidityTournamentVote"));
                            }
                            action_liquidity_tournament_vote__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Epoch => {
                            if epoch__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epoch"));
                            }
                            epoch__ = map_.next_value()?;
                        }
                        GeneratedField::AutoFee => {
                            if fee_mode__.is_some() {
                                return Err(serde::de::Error::duplicate_field("autoFee"));
                            }
                            fee_mode__ = map_.next_value::<::std::option::Option<_>>()?.map(transaction_planner_request::FeeMode::AutoFee)
;
                        }
                        GeneratedField::ManualFee => {
                            if fee_mode__.is_some() {
                                return Err(serde::de::Error::duplicate_field("manualFee"));
                            }
                            fee_mode__ = map_.next_value::<::std::option::Option<_>>()?.map(transaction_planner_request::FeeMode::ManualFee)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionPlannerRequest {
                    expiry_height: expiry_height__.unwrap_or_default(),
                    memo: memo__,
                    source: source__,
                    outputs: outputs__.unwrap_or_default(),
                    spends: spends__.unwrap_or_default(),
                    swaps: swaps__.unwrap_or_default(),
                    swap_claims: swap_claims__.unwrap_or_default(),
                    delegations: delegations__.unwrap_or_default(),
                    undelegations: undelegations__.unwrap_or_default(),
                    undelegation_claims: undelegation_claims__.unwrap_or_default(),
                    ibc_relay_actions: ibc_relay_actions__.unwrap_or_default(),
                    ics20_withdrawals: ics20_withdrawals__.unwrap_or_default(),
                    position_opens: position_opens__.unwrap_or_default(),
                    position_closes: position_closes__.unwrap_or_default(),
                    position_withdraws: position_withdraws__.unwrap_or_default(),
                    dutch_auction_schedule_actions: dutch_auction_schedule_actions__.unwrap_or_default(),
                    dutch_auction_end_actions: dutch_auction_end_actions__.unwrap_or_default(),
                    dutch_auction_withdraw_actions: dutch_auction_withdraw_actions__.unwrap_or_default(),
                    delegator_votes: delegator_votes__.unwrap_or_default(),
                    action_liquidity_tournament_vote: action_liquidity_tournament_vote__.unwrap_or_default(),
                    epoch_index: epoch_index__.unwrap_or_default(),
                    epoch: epoch__,
                    fee_mode: fee_mode__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::ActionDutchAuctionEnd {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.auction_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.ActionDutchAuctionEnd", len)?;
        if let Some(v) = self.auction_id.as_ref() {
            struct_ser.serialize_field("auctionId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::ActionDutchAuctionEnd {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "auction_id",
            "auctionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuctionId,
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
                            "auctionId" | "auction_id" => Ok(GeneratedField::AuctionId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::ActionDutchAuctionEnd;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.ActionDutchAuctionEnd")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::ActionDutchAuctionEnd, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut auction_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AuctionId => {
                            if auction_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("auctionId"));
                            }
                            auction_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_planner_request::ActionDutchAuctionEnd {
                    auction_id: auction_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.ActionDutchAuctionEnd", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::ActionDutchAuctionSchedule {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.description.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.ActionDutchAuctionSchedule", len)?;
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::ActionDutchAuctionSchedule {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "description",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Description,
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
                            "description" => Ok(GeneratedField::Description),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::ActionDutchAuctionSchedule;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.ActionDutchAuctionSchedule")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::ActionDutchAuctionSchedule, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut description__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_planner_request::ActionDutchAuctionSchedule {
                    description: description__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.ActionDutchAuctionSchedule", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::ActionDutchAuctionWithdraw {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.auction_id.is_some() {
            len += 1;
        }
        if self.seq != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.ActionDutchAuctionWithdraw", len)?;
        if let Some(v) = self.auction_id.as_ref() {
            struct_ser.serialize_field("auctionId", v)?;
        }
        if self.seq != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("seq", ToString::to_string(&self.seq).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::ActionDutchAuctionWithdraw {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "auction_id",
            "auctionId",
            "seq",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuctionId,
            Seq,
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
                            "auctionId" | "auction_id" => Ok(GeneratedField::AuctionId),
                            "seq" => Ok(GeneratedField::Seq),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::ActionDutchAuctionWithdraw;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.ActionDutchAuctionWithdraw")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::ActionDutchAuctionWithdraw, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut auction_id__ = None;
                let mut seq__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AuctionId => {
                            if auction_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("auctionId"));
                            }
                            auction_id__ = map_.next_value()?;
                        }
                        GeneratedField::Seq => {
                            if seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("seq"));
                            }
                            seq__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_planner_request::ActionDutchAuctionWithdraw {
                    auction_id: auction_id__,
                    seq: seq__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.ActionDutchAuctionWithdraw", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::ActionLiquidityTournamentVote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.incentivized.is_some() {
            len += 1;
        }
        if self.rewards_recipient.is_some() {
            len += 1;
        }
        if !self.staked_notes.is_empty() {
            len += 1;
        }
        if self.epoch_index != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.ActionLiquidityTournamentVote", len)?;
        if let Some(v) = self.incentivized.as_ref() {
            struct_ser.serialize_field("incentivized", v)?;
        }
        if let Some(v) = self.rewards_recipient.as_ref() {
            struct_ser.serialize_field("rewardsRecipient", v)?;
        }
        if !self.staked_notes.is_empty() {
            struct_ser.serialize_field("stakedNotes", &self.staked_notes)?;
        }
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::ActionLiquidityTournamentVote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "incentivized",
            "rewards_recipient",
            "rewardsRecipient",
            "staked_notes",
            "stakedNotes",
            "epoch_index",
            "epochIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Incentivized,
            RewardsRecipient,
            StakedNotes,
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
                            "incentivized" => Ok(GeneratedField::Incentivized),
                            "rewardsRecipient" | "rewards_recipient" => Ok(GeneratedField::RewardsRecipient),
                            "stakedNotes" | "staked_notes" => Ok(GeneratedField::StakedNotes),
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
            type Value = transaction_planner_request::ActionLiquidityTournamentVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.ActionLiquidityTournamentVote")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::ActionLiquidityTournamentVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut incentivized__ = None;
                let mut rewards_recipient__ = None;
                let mut staked_notes__ = None;
                let mut epoch_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Incentivized => {
                            if incentivized__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivized"));
                            }
                            incentivized__ = map_.next_value()?;
                        }
                        GeneratedField::RewardsRecipient => {
                            if rewards_recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewardsRecipient"));
                            }
                            rewards_recipient__ = map_.next_value()?;
                        }
                        GeneratedField::StakedNotes => {
                            if staked_notes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakedNotes"));
                            }
                            staked_notes__ = Some(map_.next_value()?);
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
                Ok(transaction_planner_request::ActionLiquidityTournamentVote {
                    incentivized: incentivized__,
                    rewards_recipient: rewards_recipient__,
                    staked_notes: staked_notes__.unwrap_or_default(),
                    epoch_index: epoch_index__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.ActionLiquidityTournamentVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::Delegate {
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
        if self.rate_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.Delegate", len)?;
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if let Some(v) = self.rate_data.as_ref() {
            struct_ser.serialize_field("rateData", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::Delegate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
            "rate_data",
            "rateData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Amount,
            RateData,
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
                            "rateData" | "rate_data" => Ok(GeneratedField::RateData),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::Delegate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.Delegate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::Delegate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut rate_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map_.next_value()?;
                        }
                        GeneratedField::RateData => {
                            if rate_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rateData"));
                            }
                            rate_data__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_planner_request::Delegate {
                    amount: amount__,
                    rate_data: rate_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.Delegate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::DelegatorVote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal != 0 {
            len += 1;
        }
        if self.vote.is_some() {
            len += 1;
        }
        if self.start_block_height != 0 {
            len += 1;
        }
        if self.start_position != 0 {
            len += 1;
        }
        if !self.rate_data.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.DelegatorVote", len)?;
        if self.proposal != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        if self.start_block_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startBlockHeight", ToString::to_string(&self.start_block_height).as_str())?;
        }
        if self.start_position != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startPosition", ToString::to_string(&self.start_position).as_str())?;
        }
        if !self.rate_data.is_empty() {
            struct_ser.serialize_field("rateData", &self.rate_data)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::DelegatorVote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "vote",
            "start_block_height",
            "startBlockHeight",
            "start_position",
            "startPosition",
            "rate_data",
            "rateData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            Vote,
            StartBlockHeight,
            StartPosition,
            RateData,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "vote" => Ok(GeneratedField::Vote),
                            "startBlockHeight" | "start_block_height" => Ok(GeneratedField::StartBlockHeight),
                            "startPosition" | "start_position" => Ok(GeneratedField::StartPosition),
                            "rateData" | "rate_data" => Ok(GeneratedField::RateData),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::DelegatorVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.DelegatorVote")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::DelegatorVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut vote__ = None;
                let mut start_block_height__ = None;
                let mut start_position__ = None;
                let mut rate_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map_.next_value()?;
                        }
                        GeneratedField::StartBlockHeight => {
                            if start_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startBlockHeight"));
                            }
                            start_block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::StartPosition => {
                            if start_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startPosition"));
                            }
                            start_position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RateData => {
                            if rate_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rateData"));
                            }
                            rate_data__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_planner_request::DelegatorVote {
                    proposal: proposal__.unwrap_or_default(),
                    vote: vote__,
                    start_block_height: start_block_height__.unwrap_or_default(),
                    start_position: start_position__.unwrap_or_default(),
                    rate_data: rate_data__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.DelegatorVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::Output {
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
        if self.address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.Output", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::Output {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
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
            type Value = transaction_planner_request::Output;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.Output")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::Output, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
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
                Ok(transaction_planner_request::Output {
                    value: value__,
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.Output", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::PositionClose {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.PositionClose", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::PositionClose {
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
            type Value = transaction_planner_request::PositionClose;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.PositionClose")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::PositionClose, V::Error>
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
                Ok(transaction_planner_request::PositionClose {
                    position_id: position_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.PositionClose", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::PositionOpen {
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
        if self.position_meta.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.PositionOpen", len)?;
        if let Some(v) = self.position.as_ref() {
            struct_ser.serialize_field("position", v)?;
        }
        if let Some(v) = self.position_meta.as_ref() {
            struct_ser.serialize_field("positionMeta", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::PositionOpen {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position",
            "position_meta",
            "positionMeta",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Position,
            PositionMeta,
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
                            "positionMeta" | "position_meta" => Ok(GeneratedField::PositionMeta),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::PositionOpen;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.PositionOpen")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::PositionOpen, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position__ = None;
                let mut position_meta__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = map_.next_value()?;
                        }
                        GeneratedField::PositionMeta => {
                            if position_meta__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionMeta"));
                            }
                            position_meta__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_planner_request::PositionOpen {
                    position: position__,
                    position_meta: position_meta__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.PositionOpen", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::PositionWithdraw {
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
        if self.reserves.is_some() {
            len += 1;
        }
        if self.trading_pair.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.PositionWithdraw", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.reserves.as_ref() {
            struct_ser.serialize_field("reserves", v)?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::PositionWithdraw {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position_id",
            "positionId",
            "reserves",
            "trading_pair",
            "tradingPair",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionId,
            Reserves,
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
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "reserves" => Ok(GeneratedField::Reserves),
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
            type Value = transaction_planner_request::PositionWithdraw;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.PositionWithdraw")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::PositionWithdraw, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_id__ = None;
                let mut reserves__ = None;
                let mut trading_pair__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::Reserves => {
                            if reserves__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves"));
                            }
                            reserves__ = map_.next_value()?;
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
                Ok(transaction_planner_request::PositionWithdraw {
                    position_id: position_id__,
                    reserves: reserves__,
                    trading_pair: trading_pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.PositionWithdraw", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::Spend {
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
        if self.address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.Spend", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::Spend {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
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
            type Value = transaction_planner_request::Spend;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.Spend")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::Spend, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
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
                Ok(transaction_planner_request::Spend {
                    value: value__,
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.Spend", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::Swap {
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
        if self.target_asset.is_some() {
            len += 1;
        }
        if self.fee.is_some() {
            len += 1;
        }
        if self.claim_address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.Swap", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.target_asset.as_ref() {
            struct_ser.serialize_field("targetAsset", v)?;
        }
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        if let Some(v) = self.claim_address.as_ref() {
            struct_ser.serialize_field("claimAddress", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::Swap {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "target_asset",
            "targetAsset",
            "fee",
            "claim_address",
            "claimAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            TargetAsset,
            Fee,
            ClaimAddress,
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
                            "targetAsset" | "target_asset" => Ok(GeneratedField::TargetAsset),
                            "fee" => Ok(GeneratedField::Fee),
                            "claimAddress" | "claim_address" => Ok(GeneratedField::ClaimAddress),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::Swap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.Swap")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::Swap, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut target_asset__ = None;
                let mut fee__ = None;
                let mut claim_address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::TargetAsset => {
                            if target_asset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targetAsset"));
                            }
                            target_asset__ = map_.next_value()?;
                        }
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map_.next_value()?;
                        }
                        GeneratedField::ClaimAddress => {
                            if claim_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("claimAddress"));
                            }
                            claim_address__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_planner_request::Swap {
                    value: value__,
                    target_asset: target_asset__,
                    fee: fee__,
                    claim_address: claim_address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.Swap", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::SwapClaim {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.SwapClaim", len)?;
        if let Some(v) = self.swap_commitment.as_ref() {
            struct_ser.serialize_field("swapCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::SwapClaim {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_commitment",
            "swapCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = transaction_planner_request::SwapClaim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.SwapClaim")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::SwapClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                Ok(transaction_planner_request::SwapClaim {
                    swap_commitment: swap_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.SwapClaim", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::Undelegate {
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
        if self.rate_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.Undelegate", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.rate_data.as_ref() {
            struct_ser.serialize_field("rateData", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::Undelegate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "rate_data",
            "rateData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            RateData,
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
                            "rateData" | "rate_data" => Ok(GeneratedField::RateData),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::Undelegate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.Undelegate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::Undelegate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut rate_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::RateData => {
                            if rate_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rateData"));
                            }
                            rate_data__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_planner_request::Undelegate {
                    value: value__,
                    rate_data: rate_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.Undelegate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::UndelegateClaim {
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
        if self.unbonding_start_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerRequest.UndelegateClaim", len)?;
        if let Some(v) = self.validator_identity.as_ref() {
            struct_ser.serialize_field("validatorIdentity", v)?;
        }
        if self.start_epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startEpochIndex", ToString::to_string(&self.start_epoch_index).as_str())?;
        }
        if let Some(v) = self.penalty.as_ref() {
            struct_ser.serialize_field("penalty", v)?;
        }
        if let Some(v) = self.unbonding_amount.as_ref() {
            struct_ser.serialize_field("unbondingAmount", v)?;
        }
        if self.unbonding_start_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("unbondingStartHeight", ToString::to_string(&self.unbonding_start_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::UndelegateClaim {
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
            "unbonding_start_height",
            "unbondingStartHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorIdentity,
            StartEpochIndex,
            Penalty,
            UnbondingAmount,
            UnbondingStartHeight,
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
                            "validatorIdentity" | "validator_identity" => Ok(GeneratedField::ValidatorIdentity),
                            "startEpochIndex" | "start_epoch_index" => Ok(GeneratedField::StartEpochIndex),
                            "penalty" => Ok(GeneratedField::Penalty),
                            "unbondingAmount" | "unbonding_amount" => Ok(GeneratedField::UnbondingAmount),
                            "unbondingStartHeight" | "unbonding_start_height" => Ok(GeneratedField::UnbondingStartHeight),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::UndelegateClaim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerRequest.UndelegateClaim")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::UndelegateClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_identity__ = None;
                let mut start_epoch_index__ = None;
                let mut penalty__ = None;
                let mut unbonding_amount__ = None;
                let mut unbonding_start_height__ = None;
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
                        GeneratedField::UnbondingStartHeight => {
                            if unbonding_start_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingStartHeight"));
                            }
                            unbonding_start_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_planner_request::UndelegateClaim {
                    validator_identity: validator_identity__,
                    start_epoch_index: start_epoch_index__.unwrap_or_default(),
                    penalty: penalty__,
                    unbonding_amount: unbonding_amount__,
                    unbonding_start_height: unbonding_start_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerRequest.UndelegateClaim", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionPlannerResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.plan.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransactionPlannerResponse", len)?;
        if let Some(v) = self.plan.as_ref() {
            struct_ser.serialize_field("plan", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionPlannerResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "plan",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Plan,
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
                            "plan" => Ok(GeneratedField::Plan),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionPlannerResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransactionPlannerResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionPlannerResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut plan__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Plan => {
                            if plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("plan"));
                            }
                            plan__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionPlannerResponse {
                    plan: plan__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransactionPlannerResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransparentAddressRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1.TransparentAddressRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransparentAddressRequest {
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
            type Value = TransparentAddressRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransparentAddressRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransparentAddressRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(TransparentAddressRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransparentAddressRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransparentAddressResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address.is_some() {
            len += 1;
        }
        if !self.encoding.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.TransparentAddressResponse", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        if !self.encoding.is_empty() {
            struct_ser.serialize_field("encoding", &self.encoding)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransparentAddressResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "encoding",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            Encoding,
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
                            "address" => Ok(GeneratedField::Address),
                            "encoding" => Ok(GeneratedField::Encoding),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransparentAddressResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.TransparentAddressResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransparentAddressResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut encoding__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::Encoding => {
                            if encoding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encoding"));
                            }
                            encoding__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransparentAddressResponse {
                    address: address__,
                    encoding: encoding__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.TransparentAddressResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UnbondingTokensByAddressIndexRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.filter != 0 {
            len += 1;
        }
        if self.address_index.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.UnbondingTokensByAddressIndexRequest", len)?;
        if self.filter != 0 {
            let v = unbonding_tokens_by_address_index_request::Filter::try_from(self.filter)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.filter)))?;
            struct_ser.serialize_field("filter", &v)?;
        }
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UnbondingTokensByAddressIndexRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "filter",
            "address_index",
            "addressIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Filter,
            AddressIndex,
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
                            "filter" => Ok(GeneratedField::Filter),
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UnbondingTokensByAddressIndexRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.UnbondingTokensByAddressIndexRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UnbondingTokensByAddressIndexRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut filter__ = None;
                let mut address_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Filter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filter"));
                            }
                            filter__ = Some(map_.next_value::<unbonding_tokens_by_address_index_request::Filter>()? as i32);
                        }
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UnbondingTokensByAddressIndexRequest {
                    filter: filter__.unwrap_or_default(),
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.UnbondingTokensByAddressIndexRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for unbonding_tokens_by_address_index_request::Filter {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "FILTER_UNSPECIFIED",
            Self::Claimable => "FILTER_CLAIMABLE",
            Self::NotYetClaimable => "FILTER_NOT_YET_CLAIMABLE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for unbonding_tokens_by_address_index_request::Filter {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "FILTER_UNSPECIFIED",
            "FILTER_CLAIMABLE",
            "FILTER_NOT_YET_CLAIMABLE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = unbonding_tokens_by_address_index_request::Filter;

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
                    "FILTER_UNSPECIFIED" => Ok(unbonding_tokens_by_address_index_request::Filter::Unspecified),
                    "FILTER_CLAIMABLE" => Ok(unbonding_tokens_by_address_index_request::Filter::Claimable),
                    "FILTER_NOT_YET_CLAIMABLE" => Ok(unbonding_tokens_by_address_index_request::Filter::NotYetClaimable),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for UnbondingTokensByAddressIndexResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value_view.is_some() {
            len += 1;
        }
        if self.claimable {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.UnbondingTokensByAddressIndexResponse", len)?;
        if let Some(v) = self.value_view.as_ref() {
            struct_ser.serialize_field("valueView", v)?;
        }
        if self.claimable {
            struct_ser.serialize_field("claimable", &self.claimable)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UnbondingTokensByAddressIndexResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value_view",
            "valueView",
            "claimable",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValueView,
            Claimable,
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
                            "valueView" | "value_view" => Ok(GeneratedField::ValueView),
                            "claimable" => Ok(GeneratedField::Claimable),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UnbondingTokensByAddressIndexResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.UnbondingTokensByAddressIndexResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UnbondingTokensByAddressIndexResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value_view__ = None;
                let mut claimable__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValueView => {
                            if value_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valueView"));
                            }
                            value_view__ = map_.next_value()?;
                        }
                        GeneratedField::Claimable => {
                            if claimable__.is_some() {
                                return Err(serde::de::Error::duplicate_field("claimable"));
                            }
                            claimable__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UnbondingTokensByAddressIndexResponse {
                    value_view: value_view__,
                    claimable: claimable__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.UnbondingTokensByAddressIndexResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UnclaimedSwapsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1.UnclaimedSwapsRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UnclaimedSwapsRequest {
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
            type Value = UnclaimedSwapsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.UnclaimedSwapsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UnclaimedSwapsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(UnclaimedSwapsRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.UnclaimedSwapsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UnclaimedSwapsResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.UnclaimedSwapsResponse", len)?;
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UnclaimedSwapsResponse {
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
            type Value = UnclaimedSwapsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.UnclaimedSwapsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UnclaimedSwapsResponse, V::Error>
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
                Ok(UnclaimedSwapsResponse {
                    swap: swap__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.UnclaimedSwapsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WalletIdRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1.WalletIdRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WalletIdRequest {
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
            type Value = WalletIdRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.WalletIdRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WalletIdRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(WalletIdRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.WalletIdRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WalletIdResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.wallet_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.WalletIdResponse", len)?;
        if let Some(v) = self.wallet_id.as_ref() {
            struct_ser.serialize_field("walletId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WalletIdResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "wallet_id",
            "walletId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WalletId,
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
                            "walletId" | "wallet_id" => Ok(GeneratedField::WalletId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WalletIdResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.WalletIdResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WalletIdResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut wallet_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WalletId => {
                            if wallet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletId"));
                            }
                            wallet_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(WalletIdResponse {
                    wallet_id: wallet_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.WalletIdResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WitnessAndBuildRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.transaction_plan.is_some() {
            len += 1;
        }
        if self.authorization_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.WitnessAndBuildRequest", len)?;
        if let Some(v) = self.transaction_plan.as_ref() {
            struct_ser.serialize_field("transactionPlan", v)?;
        }
        if let Some(v) = self.authorization_data.as_ref() {
            struct_ser.serialize_field("authorizationData", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WitnessAndBuildRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "transaction_plan",
            "transactionPlan",
            "authorization_data",
            "authorizationData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TransactionPlan,
            AuthorizationData,
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
                            "transactionPlan" | "transaction_plan" => Ok(GeneratedField::TransactionPlan),
                            "authorizationData" | "authorization_data" => Ok(GeneratedField::AuthorizationData),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WitnessAndBuildRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.WitnessAndBuildRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WitnessAndBuildRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut transaction_plan__ = None;
                let mut authorization_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TransactionPlan => {
                            if transaction_plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionPlan"));
                            }
                            transaction_plan__ = map_.next_value()?;
                        }
                        GeneratedField::AuthorizationData => {
                            if authorization_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorizationData"));
                            }
                            authorization_data__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(WitnessAndBuildRequest {
                    transaction_plan: transaction_plan__,
                    authorization_data: authorization_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.WitnessAndBuildRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WitnessAndBuildResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.WitnessAndBuildResponse", len)?;
        if let Some(v) = self.status.as_ref() {
            match v {
                witness_and_build_response::Status::BuildProgress(v) => {
                    struct_ser.serialize_field("buildProgress", v)?;
                }
                witness_and_build_response::Status::Complete(v) => {
                    struct_ser.serialize_field("complete", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WitnessAndBuildResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "build_progress",
            "buildProgress",
            "complete",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BuildProgress,
            Complete,
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
                            "buildProgress" | "build_progress" => Ok(GeneratedField::BuildProgress),
                            "complete" => Ok(GeneratedField::Complete),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WitnessAndBuildResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.WitnessAndBuildResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WitnessAndBuildResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BuildProgress => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("buildProgress"));
                            }
                            status__ = map_.next_value::<::std::option::Option<_>>()?.map(witness_and_build_response::Status::BuildProgress)
;
                        }
                        GeneratedField::Complete => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("complete"));
                            }
                            status__ = map_.next_value::<::std::option::Option<_>>()?.map(witness_and_build_response::Status::Complete)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(WitnessAndBuildResponse {
                    status: status__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.WitnessAndBuildResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for witness_and_build_response::BuildProgress {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.progress != 0. {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.WitnessAndBuildResponse.BuildProgress", len)?;
        if self.progress != 0. {
            struct_ser.serialize_field("progress", &self.progress)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for witness_and_build_response::BuildProgress {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "progress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Progress,
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
                            "progress" => Ok(GeneratedField::Progress),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = witness_and_build_response::BuildProgress;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.WitnessAndBuildResponse.BuildProgress")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<witness_and_build_response::BuildProgress, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut progress__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Progress => {
                            if progress__.is_some() {
                                return Err(serde::de::Error::duplicate_field("progress"));
                            }
                            progress__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(witness_and_build_response::BuildProgress {
                    progress: progress__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.WitnessAndBuildResponse.BuildProgress", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for witness_and_build_response::Complete {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.transaction.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.WitnessAndBuildResponse.Complete", len)?;
        if let Some(v) = self.transaction.as_ref() {
            struct_ser.serialize_field("transaction", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for witness_and_build_response::Complete {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "transaction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Transaction,
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
                            "transaction" => Ok(GeneratedField::Transaction),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = witness_and_build_response::Complete;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.WitnessAndBuildResponse.Complete")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<witness_and_build_response::Complete, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut transaction__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Transaction => {
                            if transaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transaction"));
                            }
                            transaction__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(witness_and_build_response::Complete {
                    transaction: transaction__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.WitnessAndBuildResponse.Complete", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WitnessRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.transaction_plan.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.WitnessRequest", len)?;
        if let Some(v) = self.transaction_plan.as_ref() {
            struct_ser.serialize_field("transactionPlan", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WitnessRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "transaction_plan",
            "transactionPlan",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TransactionPlan,
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
                            "transactionPlan" | "transaction_plan" => Ok(GeneratedField::TransactionPlan),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WitnessRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.WitnessRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WitnessRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut transaction_plan__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TransactionPlan => {
                            if transaction_plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionPlan"));
                            }
                            transaction_plan__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(WitnessRequest {
                    transaction_plan: transaction_plan__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.WitnessRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WitnessResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.witness_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1.WitnessResponse", len)?;
        if let Some(v) = self.witness_data.as_ref() {
            struct_ser.serialize_field("witnessData", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WitnessResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "witness_data",
            "witnessData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WitnessData,
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
                            "witnessData" | "witness_data" => Ok(GeneratedField::WitnessData),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WitnessResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1.WitnessResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WitnessResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut witness_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WitnessData => {
                            if witness_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("witnessData"));
                            }
                            witness_data__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(WitnessResponse {
                    witness_data: witness_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1.WitnessResponse", FIELDS, GeneratedVisitor)
    }
}
