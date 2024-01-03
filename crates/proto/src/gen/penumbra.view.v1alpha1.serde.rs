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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AddressByIndexRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.AddressByIndexRequest")
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
                    }
                }
                Ok(AddressByIndexRequest {
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AddressByIndexRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AddressByIndexResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.AddressByIndexResponse")
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
                    }
                }
                Ok(AddressByIndexResponse {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AddressByIndexResponse", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AppParametersRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
            type Value = AppParametersRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.AppParametersRequest")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AppParametersRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AppParametersResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.AppParametersResponse")
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
                    }
                }
                Ok(AppParametersResponse {
                    parameters: parameters__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AppParametersResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AssetsRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.AssetsRequest")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AssetsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AssetsResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.AssetsResponse")
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
                    }
                }
                Ok(AssetsResponse {
                    denom_metadata: denom_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AssetsResponse", FIELDS, GeneratedVisitor)
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
        if self.authorization_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AuthorizeAndBuildRequest", len)?;
        if let Some(v) = self.transaction_plan.as_ref() {
            struct_ser.serialize_field("transactionPlan", v)?;
        }
        if let Some(v) = self.authorization_data.as_ref() {
            struct_ser.serialize_field("authorizationData", v)?;
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
            "authorization_data",
            "authorizationData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TransactionPlan,
            AuthorizationData,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.AuthorizeAndBuildRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizeAndBuildRequest, V::Error>
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
                    }
                }
                Ok(AuthorizeAndBuildRequest {
                    transaction_plan: transaction_plan__,
                    authorization_data: authorization_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AuthorizeAndBuildRequest", FIELDS, GeneratedVisitor)
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
        if self.transaction.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AuthorizeAndBuildResponse", len)?;
        if let Some(v) = self.transaction.as_ref() {
            struct_ser.serialize_field("transaction", v)?;
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
            "transaction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Transaction,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.AuthorizeAndBuildResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizeAndBuildResponse, V::Error>
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
                    }
                }
                Ok(AuthorizeAndBuildResponse {
                    transaction: transaction__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AuthorizeAndBuildResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.BalancesRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.BalancesRequest")
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
                    }
                }
                Ok(BalancesRequest {
                    account_filter: account_filter__,
                    asset_id_filter: asset_id_filter__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.BalancesRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.BalancesResponse", len)?;
        if let Some(v) = self.account.as_ref() {
            struct_ser.serialize_field("account", v)?;
        }
        if let Some(v) = self.balance.as_ref() {
            struct_ser.serialize_field("balance", v)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Account,
            Balance,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.BalancesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BalancesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut account__ = None;
                let mut balance__ = None;
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
                    }
                }
                Ok(BalancesResponse {
                    account: account__,
                    balance: balance__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.BalancesResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.BroadcastTransactionRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.BroadcastTransactionRequest")
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
                    }
                }
                Ok(BroadcastTransactionRequest {
                    transaction: transaction__,
                    await_detection: await_detection__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.BroadcastTransactionRequest", FIELDS, GeneratedVisitor)
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
        if self.id.is_some() {
            len += 1;
        }
        if self.detection_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.BroadcastTransactionResponse", len)?;
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if self.detection_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("detectionHeight", ToString::to_string(&self.detection_height).as_str())?;
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
            "id",
            "detection_height",
            "detectionHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            DetectionHeight,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.BroadcastTransactionResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<BroadcastTransactionResponse, V::Error>
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
                    }
                }
                Ok(BroadcastTransactionResponse {
                    id: id__,
                    detection_height: detection_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.BroadcastTransactionResponse", FIELDS, GeneratedVisitor)
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
        if self.display_confirm {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.EphemeralAddressRequest", len)?;
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        if self.display_confirm {
            struct_ser.serialize_field("displayConfirm", &self.display_confirm)?;
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
            "display_confirm",
            "displayConfirm",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AddressIndex,
            DisplayConfirm,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "displayConfirm" | "display_confirm" => Ok(GeneratedField::DisplayConfirm),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.EphemeralAddressRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EphemeralAddressRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address_index__ = None;
                let mut display_confirm__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map_.next_value()?;
                        }
                        GeneratedField::DisplayConfirm => {
                            if display_confirm__.is_some() {
                                return Err(serde::de::Error::duplicate_field("displayConfirm"));
                            }
                            display_confirm__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(EphemeralAddressRequest {
                    address_index: address_index__,
                    display_confirm: display_confirm__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.EphemeralAddressRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.EphemeralAddressResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.EphemeralAddressResponse")
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
                    }
                }
                Ok(EphemeralAddressResponse {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.EphemeralAddressResponse", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.FMDParametersRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
            type Value = FmdParametersRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.FMDParametersRequest")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.FMDParametersRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.FMDParametersResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.FMDParametersResponse")
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
                    }
                }
                Ok(FmdParametersResponse {
                    parameters: parameters__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.FMDParametersResponse", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.GasPricesRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
            type Value = GasPricesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.GasPricesRequest")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.GasPricesRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.GasPricesResponse", len)?;
        if let Some(v) = self.gas_prices.as_ref() {
            struct_ser.serialize_field("gasPrices", v)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GasPrices,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.GasPricesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GasPricesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut gas_prices__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::GasPrices => {
                            if gas_prices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gasPrices"));
                            }
                            gas_prices__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GasPricesResponse {
                    gas_prices: gas_prices__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.GasPricesResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.IndexByAddressRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.IndexByAddressRequest")
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
                    }
                }
                Ok(IndexByAddressRequest {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.IndexByAddressRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.IndexByAddressResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.IndexByAddressResponse")
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
                    }
                }
                Ok(IndexByAddressResponse {
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.IndexByAddressResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NoteByCommitmentRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.NoteByCommitmentRequest")
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
                    }
                }
                Ok(NoteByCommitmentRequest {
                    note_commitment: note_commitment__,
                    await_detection: await_detection__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NoteByCommitmentRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NoteByCommitmentResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.NoteByCommitmentResponse")
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
                    }
                }
                Ok(NoteByCommitmentResponse {
                    spendable_note: spendable_note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NoteByCommitmentResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NotesForVotingRequest", len)?;
        if self.votable_at_height != 0 {
            #[allow(clippy::needless_borrow)]
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.NotesForVotingRequest")
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
                    }
                }
                Ok(NotesForVotingRequest {
                    votable_at_height: votable_at_height__.unwrap_or_default(),
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NotesForVotingRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NotesForVotingResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.NotesForVotingResponse")
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
                    }
                }
                Ok(NotesForVotingResponse {
                    note_record: note_record__,
                    identity_key: identity_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NotesForVotingResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NotesRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.NotesRequest")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NotesRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NotesResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.NotesResponse")
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
                    }
                }
                Ok(NotesResponse {
                    note_record: note_record__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NotesResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NullifierStatusRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.NullifierStatusRequest")
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
                    }
                }
                Ok(NullifierStatusRequest {
                    nullifier: nullifier__,
                    await_detection: await_detection__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NullifierStatusRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NullifierStatusResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.NullifierStatusResponse")
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
                    }
                }
                Ok(NullifierStatusResponse {
                    spent: spent__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NullifierStatusResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.OwnedPositionIdsRequest", len)?;
        if let Some(v) = self.position_state.as_ref() {
            struct_ser.serialize_field("positionState", v)?;
        }
        if let Some(v) = self.trading_pair.as_ref() {
            struct_ser.serialize_field("tradingPair", v)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PositionState,
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
                            "positionState" | "position_state" => Ok(GeneratedField::PositionState),
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
            type Value = OwnedPositionIdsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.OwnedPositionIdsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OwnedPositionIdsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position_state__ = None;
                let mut trading_pair__ = None;
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
                    }
                }
                Ok(OwnedPositionIdsRequest {
                    position_state: position_state__,
                    trading_pair: trading_pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.OwnedPositionIdsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.OwnedPositionIdsResponse", len)?;
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
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
            type Value = OwnedPositionIdsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.OwnedPositionIdsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OwnedPositionIdsResponse, V::Error>
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
                    }
                }
                Ok(OwnedPositionIdsResponse {
                    position_id: position_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.OwnedPositionIdsResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.SpendableNoteRecord", len)?;
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
            struct_ser.serialize_field("heightCreated", ToString::to_string(&self.height_created).as_str())?;
        }
        if self.height_spent != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("heightSpent", ToString::to_string(&self.height_spent).as_str())?;
        }
        if self.position != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("position", ToString::to_string(&self.position).as_str())?;
        }
        if let Some(v) = self.source.as_ref() {
            struct_ser.serialize_field("source", v)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.SpendableNoteRecord")
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
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.SpendableNoteRecord", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.StatusRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
            type Value = StatusRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.StatusRequest")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.StatusRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.StatusResponse", len)?;
        if self.full_sync_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("fullSyncHeight", ToString::to_string(&self.full_sync_height).as_str())?;
        }
        if self.partial_sync_height != 0 {
            #[allow(clippy::needless_borrow)]
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.StatusResponse")
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
                    }
                }
                Ok(StatusResponse {
                    full_sync_height: full_sync_height__.unwrap_or_default(),
                    partial_sync_height: partial_sync_height__.unwrap_or_default(),
                    catching_up: catching_up__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.StatusResponse", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.StatusStreamRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
            type Value = StatusStreamRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.StatusStreamRequest")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.StatusStreamRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.StatusStreamResponse", len)?;
        if self.latest_known_block_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("latestKnownBlockHeight", ToString::to_string(&self.latest_known_block_height).as_str())?;
        }
        if self.full_sync_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("fullSyncHeight", ToString::to_string(&self.full_sync_height).as_str())?;
        }
        if self.partial_sync_height != 0 {
            #[allow(clippy::needless_borrow)]
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.StatusStreamResponse")
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
                    }
                }
                Ok(StatusStreamResponse {
                    latest_known_block_height: latest_known_block_height__.unwrap_or_default(),
                    full_sync_height: full_sync_height__.unwrap_or_default(),
                    partial_sync_height: partial_sync_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.StatusStreamResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.SwapByCommitmentRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.SwapByCommitmentRequest")
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
                    }
                }
                Ok(SwapByCommitmentRequest {
                    swap_commitment: swap_commitment__,
                    await_detection: await_detection__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.SwapByCommitmentRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.SwapByCommitmentResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
            type Value = SwapByCommitmentResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.SwapByCommitmentResponse")
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
                    }
                }
                Ok(SwapByCommitmentResponse {
                    swap: swap__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.SwapByCommitmentResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.SwapRecord", len)?;
        if let Some(v) = self.swap_commitment.as_ref() {
            struct_ser.serialize_field("swapCommitment", v)?;
        }
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        if self.position != 0 {
            #[allow(clippy::needless_borrow)]
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.SwapRecord")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.SwapRecord", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionInfo", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            Id,
            Transaction,
            Perspective,
            View,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionInfo")
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
                    }
                }
                Ok(TransactionInfo {
                    height: height__.unwrap_or_default(),
                    id: id__,
                    transaction: transaction__,
                    perspective: perspective__,
                    view: view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionInfo", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionInfoByHashRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionInfoByHashRequest")
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
                    }
                }
                Ok(TransactionInfoByHashRequest {
                    id: id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionInfoByHashRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionInfoByHashResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionInfoByHashResponse")
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
                    }
                }
                Ok(TransactionInfoByHashResponse {
                    tx_info: tx_info__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionInfoByHashResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionInfoRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionInfoRequest")
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
                    }
                }
                Ok(TransactionInfoRequest {
                    start_height: start_height__.unwrap_or_default(),
                    end_height: end_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionInfoRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionInfoResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionInfoResponse")
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
                    }
                }
                Ok(TransactionInfoResponse {
                    tx_info: tx_info__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionInfoResponse", FIELDS, GeneratedVisitor)
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
        if self.fee.is_some() {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest", len)?;
        if self.expiry_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("expiryHeight", ToString::to_string(&self.expiry_height).as_str())?;
        }
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
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
            "fee",
            "memo",
            "source",
            "outputs",
            "swaps",
            "swap_claims",
            "swapClaims",
            "delegations",
            "undelegations",
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExpiryHeight,
            Fee,
            Memo,
            Source,
            Outputs,
            Swaps,
            SwapClaims,
            Delegations,
            Undelegations,
            IbcRelayActions,
            Ics20Withdrawals,
            PositionOpens,
            PositionCloses,
            PositionWithdraws,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "fee" => Ok(GeneratedField::Fee),
                            "memo" => Ok(GeneratedField::Memo),
                            "source" => Ok(GeneratedField::Source),
                            "outputs" => Ok(GeneratedField::Outputs),
                            "swaps" => Ok(GeneratedField::Swaps),
                            "swapClaims" | "swap_claims" => Ok(GeneratedField::SwapClaims),
                            "delegations" => Ok(GeneratedField::Delegations),
                            "undelegations" => Ok(GeneratedField::Undelegations),
                            "ibcRelayActions" | "ibc_relay_actions" => Ok(GeneratedField::IbcRelayActions),
                            "ics20Withdrawals" | "ics20_withdrawals" => Ok(GeneratedField::Ics20Withdrawals),
                            "positionOpens" | "position_opens" => Ok(GeneratedField::PositionOpens),
                            "positionCloses" | "position_closes" => Ok(GeneratedField::PositionCloses),
                            "positionWithdraws" | "position_withdraws" => Ok(GeneratedField::PositionWithdraws),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionPlannerRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut expiry_height__ = None;
                let mut fee__ = None;
                let mut memo__ = None;
                let mut source__ = None;
                let mut outputs__ = None;
                let mut swaps__ = None;
                let mut swap_claims__ = None;
                let mut delegations__ = None;
                let mut undelegations__ = None;
                let mut ibc_relay_actions__ = None;
                let mut ics20_withdrawals__ = None;
                let mut position_opens__ = None;
                let mut position_closes__ = None;
                let mut position_withdraws__ = None;
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
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map_.next_value()?;
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
                    }
                }
                Ok(TransactionPlannerRequest {
                    expiry_height: expiry_height__.unwrap_or_default(),
                    fee: fee__,
                    memo: memo__,
                    source: source__,
                    outputs: outputs__.unwrap_or_default(),
                    swaps: swaps__.unwrap_or_default(),
                    swap_claims: swap_claims__.unwrap_or_default(),
                    delegations: delegations__.unwrap_or_default(),
                    undelegations: undelegations__.unwrap_or_default(),
                    ibc_relay_actions: ibc_relay_actions__.unwrap_or_default(),
                    ics20_withdrawals: ics20_withdrawals__.unwrap_or_default(),
                    position_opens: position_opens__.unwrap_or_default(),
                    position_closes: position_closes__.unwrap_or_default(),
                    position_withdraws: position_withdraws__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Delegate", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.Delegate")
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
                    }
                }
                Ok(transaction_planner_request::Delegate {
                    amount: amount__,
                    rate_data: rate_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Delegate", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Output", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.Output")
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
                    }
                }
                Ok(transaction_planner_request::Output {
                    value: value__,
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Output", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.PositionClose", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
            type Value = transaction_planner_request::PositionClose;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.PositionClose")
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
                    }
                }
                Ok(transaction_planner_request::PositionClose {
                    position_id: position_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.PositionClose", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.PositionOpen", len)?;
        if let Some(v) = self.position.as_ref() {
            struct_ser.serialize_field("position", v)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Position,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.PositionOpen")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_planner_request::PositionOpen, V::Error>
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
                    }
                }
                Ok(transaction_planner_request::PositionOpen {
                    position: position__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.PositionOpen", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.PositionWithdraw", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.PositionWithdraw")
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
                    }
                }
                Ok(transaction_planner_request::PositionWithdraw {
                    position_id: position_id__,
                    reserves: reserves__,
                    trading_pair: trading_pair__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.PositionWithdraw", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Swap", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.Swap")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Swap", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.SwapClaim", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.SwapClaim")
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
                    }
                }
                Ok(transaction_planner_request::SwapClaim {
                    swap_commitment: swap_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.SwapClaim", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Undelegate", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.Undelegate")
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
                    }
                }
                Ok(transaction_planner_request::Undelegate {
                    value: value__,
                    rate_data: rate_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Undelegate", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerResponse")
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
                    }
                }
                Ok(TransactionPlannerResponse {
                    plan: plan__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerResponse", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.UnclaimedSwapsRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
            type Value = UnclaimedSwapsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.UnclaimedSwapsRequest")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.UnclaimedSwapsRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.UnclaimedSwapsResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
            type Value = UnclaimedSwapsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.UnclaimedSwapsResponse")
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
                    }
                }
                Ok(UnclaimedSwapsResponse {
                    swap: swap__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.UnclaimedSwapsResponse", FIELDS, GeneratedVisitor)
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
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.WalletIdRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
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
            type Value = WalletIdRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.WalletIdRequest")
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
        deserializer.deserialize_struct("penumbra.view.v1alpha1.WalletIdRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.WalletIdResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.WalletIdResponse")
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
                    }
                }
                Ok(WalletIdResponse {
                    wallet_id: wallet_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.WalletIdResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.WitnessAndBuildRequest", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.WitnessAndBuildRequest")
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
                    }
                }
                Ok(WitnessAndBuildRequest {
                    transaction_plan: transaction_plan__,
                    authorization_data: authorization_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.WitnessAndBuildRequest", FIELDS, GeneratedVisitor)
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
        if self.transaction.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.WitnessAndBuildResponse", len)?;
        if let Some(v) = self.transaction.as_ref() {
            struct_ser.serialize_field("transaction", v)?;
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
            "transaction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Transaction,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.WitnessAndBuildResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WitnessAndBuildResponse, V::Error>
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
                    }
                }
                Ok(WitnessAndBuildResponse {
                    transaction: transaction__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.WitnessAndBuildResponse", FIELDS, GeneratedVisitor)
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
        if !self.note_commitments.is_empty() {
            len += 1;
        }
        if self.transaction_plan.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.WitnessRequest", len)?;
        if !self.note_commitments.is_empty() {
            struct_ser.serialize_field("noteCommitments", &self.note_commitments)?;
        }
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
            "note_commitments",
            "noteCommitments",
            "transaction_plan",
            "transactionPlan",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteCommitments,
            TransactionPlan,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "noteCommitments" | "note_commitments" => Ok(GeneratedField::NoteCommitments),
                            "transactionPlan" | "transaction_plan" => Ok(GeneratedField::TransactionPlan),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.WitnessRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WitnessRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_commitments__ = None;
                let mut transaction_plan__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NoteCommitments => {
                            if note_commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitments"));
                            }
                            note_commitments__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TransactionPlan => {
                            if transaction_plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionPlan"));
                            }
                            transaction_plan__ = map_.next_value()?;
                        }
                    }
                }
                Ok(WitnessRequest {
                    note_commitments: note_commitments__.unwrap_or_default(),
                    transaction_plan: transaction_plan__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.WitnessRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.WitnessResponse", len)?;
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.view.v1alpha1.WitnessResponse")
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
                    }
                }
                Ok(WitnessResponse {
                    witness_data: witness_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.WitnessResponse", FIELDS, GeneratedVisitor)
    }
}
