impl serde::Serialize for AllTalliedDelegatorVotesForProposalRequest {
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
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.AllTalliedDelegatorVotesForProposalRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.proposal_id != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AllTalliedDelegatorVotesForProposalRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            ProposalId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AllTalliedDelegatorVotesForProposalRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.AllTalliedDelegatorVotesForProposalRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AllTalliedDelegatorVotesForProposalRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut proposal_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(AllTalliedDelegatorVotesForProposalRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.AllTalliedDelegatorVotesForProposalRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AllTalliedDelegatorVotesForProposalResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tally.is_some() {
            len += 1;
        }
        if self.identity_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.AllTalliedDelegatorVotesForProposalResponse", len)?;
        if let Some(v) = self.tally.as_ref() {
            struct_ser.serialize_field("tally", v)?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AllTalliedDelegatorVotesForProposalResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tally",
            "identity_key",
            "identityKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tally,
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
                            "tally" => Ok(GeneratedField::Tally),
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
            type Value = AllTalliedDelegatorVotesForProposalResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.AllTalliedDelegatorVotesForProposalResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AllTalliedDelegatorVotesForProposalResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tally__ = None;
                let mut identity_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Tally => {
                            if tally__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tally"));
                            }
                            tally__ = map_.next_value()?;
                        }
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AllTalliedDelegatorVotesForProposalResponse {
                    tally: tally__,
                    identity_key: identity_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.AllTalliedDelegatorVotesForProposalResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ChangedAppParameters {
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
        if self.community_pool_params.is_some() {
            len += 1;
        }
        if self.governance_params.is_some() {
            len += 1;
        }
        if self.ibc_params.is_some() {
            len += 1;
        }
        if self.stake_params.is_some() {
            len += 1;
        }
        if self.fee_params.is_some() {
            len += 1;
        }
        if self.distributions_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ChangedAppParameters", len)?;
        if let Some(v) = self.chain_params.as_ref() {
            struct_ser.serialize_field("chainParams", v)?;
        }
        if let Some(v) = self.community_pool_params.as_ref() {
            struct_ser.serialize_field("communityPoolParams", v)?;
        }
        if let Some(v) = self.governance_params.as_ref() {
            struct_ser.serialize_field("governanceParams", v)?;
        }
        if let Some(v) = self.ibc_params.as_ref() {
            struct_ser.serialize_field("ibcParams", v)?;
        }
        if let Some(v) = self.stake_params.as_ref() {
            struct_ser.serialize_field("stakeParams", v)?;
        }
        if let Some(v) = self.fee_params.as_ref() {
            struct_ser.serialize_field("feeParams", v)?;
        }
        if let Some(v) = self.distributions_params.as_ref() {
            struct_ser.serialize_field("distributionsParams", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChangedAppParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_params",
            "chainParams",
            "community_pool_params",
            "communityPoolParams",
            "governance_params",
            "governanceParams",
            "ibc_params",
            "ibcParams",
            "stake_params",
            "stakeParams",
            "fee_params",
            "feeParams",
            "distributions_params",
            "distributionsParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainParams,
            CommunityPoolParams,
            GovernanceParams,
            IbcParams,
            StakeParams,
            FeeParams,
            DistributionsParams,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "communityPoolParams" | "community_pool_params" => Ok(GeneratedField::CommunityPoolParams),
                            "governanceParams" | "governance_params" => Ok(GeneratedField::GovernanceParams),
                            "ibcParams" | "ibc_params" => Ok(GeneratedField::IbcParams),
                            "stakeParams" | "stake_params" => Ok(GeneratedField::StakeParams),
                            "feeParams" | "fee_params" => Ok(GeneratedField::FeeParams),
                            "distributionsParams" | "distributions_params" => Ok(GeneratedField::DistributionsParams),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ChangedAppParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ChangedAppParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ChangedAppParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_params__ = None;
                let mut community_pool_params__ = None;
                let mut governance_params__ = None;
                let mut ibc_params__ = None;
                let mut stake_params__ = None;
                let mut fee_params__ = None;
                let mut distributions_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainParams => {
                            if chain_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainParams"));
                            }
                            chain_params__ = map_.next_value()?;
                        }
                        GeneratedField::CommunityPoolParams => {
                            if community_pool_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolParams"));
                            }
                            community_pool_params__ = map_.next_value()?;
                        }
                        GeneratedField::GovernanceParams => {
                            if governance_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("governanceParams"));
                            }
                            governance_params__ = map_.next_value()?;
                        }
                        GeneratedField::IbcParams => {
                            if ibc_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcParams"));
                            }
                            ibc_params__ = map_.next_value()?;
                        }
                        GeneratedField::StakeParams => {
                            if stake_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakeParams"));
                            }
                            stake_params__ = map_.next_value()?;
                        }
                        GeneratedField::FeeParams => {
                            if fee_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeParams"));
                            }
                            fee_params__ = map_.next_value()?;
                        }
                        GeneratedField::DistributionsParams => {
                            if distributions_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("distributionsParams"));
                            }
                            distributions_params__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ChangedAppParameters {
                    chain_params: chain_params__,
                    community_pool_params: community_pool_params__,
                    governance_params: governance_params__,
                    ibc_params: ibc_params__,
                    stake_params: stake_params__,
                    fee_params: fee_params__,
                    distributions_params: distributions_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ChangedAppParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ChangedAppParametersSet {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.old.is_some() {
            len += 1;
        }
        if self.new.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ChangedAppParametersSet", len)?;
        if let Some(v) = self.old.as_ref() {
            struct_ser.serialize_field("old", v)?;
        }
        if let Some(v) = self.new.as_ref() {
            struct_ser.serialize_field("new", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChangedAppParametersSet {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "old",
            "new",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Old,
            New,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "old" => Ok(GeneratedField::Old),
                            "new" => Ok(GeneratedField::New),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ChangedAppParametersSet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ChangedAppParametersSet")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ChangedAppParametersSet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut old__ = None;
                let mut new__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Old => {
                            if old__.is_some() {
                                return Err(serde::de::Error::duplicate_field("old"));
                            }
                            old__ = map_.next_value()?;
                        }
                        GeneratedField::New => {
                            if new__.is_some() {
                                return Err(serde::de::Error::duplicate_field("new"));
                            }
                            new__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ChangedAppParametersSet {
                    old: old__,
                    new: new__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ChangedAppParametersSet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CommunityPoolDeposit {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.CommunityPoolDeposit", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CommunityPoolDeposit {
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
            type Value = CommunityPoolDeposit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.CommunityPoolDeposit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CommunityPoolDeposit, V::Error>
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
                            value__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CommunityPoolDeposit {
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.CommunityPoolDeposit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CommunityPoolOutput {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.CommunityPoolOutput", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CommunityPoolOutput {
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
            type Value = CommunityPoolOutput;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.CommunityPoolOutput")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CommunityPoolOutput, V::Error>
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
                Ok(CommunityPoolOutput {
                    value: value__,
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.CommunityPoolOutput", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CommunityPoolSpend {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.CommunityPoolSpend", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CommunityPoolSpend {
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
            type Value = CommunityPoolSpend;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.CommunityPoolSpend")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CommunityPoolSpend, V::Error>
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
                            value__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CommunityPoolSpend {
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.CommunityPoolSpend", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorVote {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVote", len)?;
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
impl<'de> serde::Deserialize<'de> for DelegatorVote {
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.DelegatorVote")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DelegatorVote, V::Error>
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
                    }
                }
                Ok(DelegatorVote {
                    body: body__,
                    auth_sig: auth_sig__,
                    proof: proof__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorVoteBody {
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
        if self.start_position != 0 {
            len += 1;
        }
        if self.vote.is_some() {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        if self.unbonded_amount.is_some() {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.rk.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVoteBody", len)?;
        if self.proposal != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if self.start_position != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startPosition", ToString::to_string(&self.start_position).as_str())?;
        }
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.unbonded_amount.as_ref() {
            struct_ser.serialize_field("unbondedAmount", v)?;
        }
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if let Some(v) = self.rk.as_ref() {
            struct_ser.serialize_field("rk", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorVoteBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "start_position",
            "startPosition",
            "vote",
            "value",
            "unbonded_amount",
            "unbondedAmount",
            "nullifier",
            "rk",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            StartPosition,
            Vote,
            Value,
            UnbondedAmount,
            Nullifier,
            Rk,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "startPosition" | "start_position" => Ok(GeneratedField::StartPosition),
                            "vote" => Ok(GeneratedField::Vote),
                            "value" => Ok(GeneratedField::Value),
                            "unbondedAmount" | "unbonded_amount" => Ok(GeneratedField::UnbondedAmount),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "rk" => Ok(GeneratedField::Rk),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorVoteBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.DelegatorVoteBody")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DelegatorVoteBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut start_position__ = None;
                let mut vote__ = None;
                let mut value__ = None;
                let mut unbonded_amount__ = None;
                let mut nullifier__ = None;
                let mut rk__ = None;
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
                        GeneratedField::StartPosition => {
                            if start_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startPosition"));
                            }
                            start_position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map_.next_value()?;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::UnbondedAmount => {
                            if unbonded_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondedAmount"));
                            }
                            unbonded_amount__ = map_.next_value()?;
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
                    }
                }
                Ok(DelegatorVoteBody {
                    proposal: proposal__.unwrap_or_default(),
                    start_position: start_position__.unwrap_or_default(),
                    vote: vote__,
                    value: value__,
                    unbonded_amount: unbonded_amount__,
                    nullifier: nullifier__,
                    rk: rk__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVoteBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorVotePlan {
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
        if self.start_position != 0 {
            len += 1;
        }
        if self.vote.is_some() {
            len += 1;
        }
        if self.staked_note.is_some() {
            len += 1;
        }
        if self.staked_note_position != 0 {
            len += 1;
        }
        if self.unbonded_amount.is_some() {
            len += 1;
        }
        if !self.randomizer.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_r.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_s.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVotePlan", len)?;
        if self.proposal != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if self.start_position != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startPosition", ToString::to_string(&self.start_position).as_str())?;
        }
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        if let Some(v) = self.staked_note.as_ref() {
            struct_ser.serialize_field("stakedNote", v)?;
        }
        if self.staked_note_position != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("stakedNotePosition", ToString::to_string(&self.staked_note_position).as_str())?;
        }
        if let Some(v) = self.unbonded_amount.as_ref() {
            struct_ser.serialize_field("unbondedAmount", v)?;
        }
        if !self.randomizer.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("randomizer", pbjson::private::base64::encode(&self.randomizer).as_str())?;
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
impl<'de> serde::Deserialize<'de> for DelegatorVotePlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "start_position",
            "startPosition",
            "vote",
            "staked_note",
            "stakedNote",
            "staked_note_position",
            "stakedNotePosition",
            "unbonded_amount",
            "unbondedAmount",
            "randomizer",
            "proof_blinding_r",
            "proofBlindingR",
            "proof_blinding_s",
            "proofBlindingS",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            StartPosition,
            Vote,
            StakedNote,
            StakedNotePosition,
            UnbondedAmount,
            Randomizer,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "startPosition" | "start_position" => Ok(GeneratedField::StartPosition),
                            "vote" => Ok(GeneratedField::Vote),
                            "stakedNote" | "staked_note" => Ok(GeneratedField::StakedNote),
                            "stakedNotePosition" | "staked_note_position" => Ok(GeneratedField::StakedNotePosition),
                            "unbondedAmount" | "unbonded_amount" => Ok(GeneratedField::UnbondedAmount),
                            "randomizer" => Ok(GeneratedField::Randomizer),
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
            type Value = DelegatorVotePlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.DelegatorVotePlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DelegatorVotePlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut start_position__ = None;
                let mut vote__ = None;
                let mut staked_note__ = None;
                let mut staked_note_position__ = None;
                let mut unbonded_amount__ = None;
                let mut randomizer__ = None;
                let mut proof_blinding_r__ = None;
                let mut proof_blinding_s__ = None;
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
                        GeneratedField::StartPosition => {
                            if start_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startPosition"));
                            }
                            start_position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map_.next_value()?;
                        }
                        GeneratedField::StakedNote => {
                            if staked_note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakedNote"));
                            }
                            staked_note__ = map_.next_value()?;
                        }
                        GeneratedField::StakedNotePosition => {
                            if staked_note_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakedNotePosition"));
                            }
                            staked_note_position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UnbondedAmount => {
                            if unbonded_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondedAmount"));
                            }
                            unbonded_amount__ = map_.next_value()?;
                        }
                        GeneratedField::Randomizer => {
                            if randomizer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("randomizer"));
                            }
                            randomizer__ = 
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
                Ok(DelegatorVotePlan {
                    proposal: proposal__.unwrap_or_default(),
                    start_position: start_position__.unwrap_or_default(),
                    vote: vote__,
                    staked_note: staked_note__,
                    staked_note_position: staked_note_position__.unwrap_or_default(),
                    unbonded_amount: unbonded_amount__,
                    randomizer: randomizer__.unwrap_or_default(),
                    proof_blinding_r: proof_blinding_r__.unwrap_or_default(),
                    proof_blinding_s: proof_blinding_s__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVotePlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorVoteView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.delegator_vote.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVoteView", len)?;
        if let Some(v) = self.delegator_vote.as_ref() {
            match v {
                delegator_vote_view::DelegatorVote::Visible(v) => {
                    struct_ser.serialize_field("visible", v)?;
                }
                delegator_vote_view::DelegatorVote::Opaque(v) => {
                    struct_ser.serialize_field("opaque", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorVoteView {
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
            type Value = DelegatorVoteView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.DelegatorVoteView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DelegatorVoteView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_vote__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            delegator_vote__ = map_.next_value::<::std::option::Option<_>>()?.map(delegator_vote_view::DelegatorVote::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            delegator_vote__ = map_.next_value::<::std::option::Option<_>>()?.map(delegator_vote_view::DelegatorVote::Opaque)
;
                        }
                    }
                }
                Ok(DelegatorVoteView {
                    delegator_vote: delegator_vote__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVoteView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for delegator_vote_view::Opaque {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.delegator_vote.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVoteView.Opaque", len)?;
        if let Some(v) = self.delegator_vote.as_ref() {
            struct_ser.serialize_field("delegatorVote", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for delegator_vote_view::Opaque {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_vote",
            "delegatorVote",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorVote,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = delegator_vote_view::Opaque;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.DelegatorVoteView.Opaque")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<delegator_vote_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_vote__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DelegatorVote => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            delegator_vote__ = map_.next_value()?;
                        }
                    }
                }
                Ok(delegator_vote_view::Opaque {
                    delegator_vote: delegator_vote__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVoteView.Opaque", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for delegator_vote_view::Visible {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.delegator_vote.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVoteView.Visible", len)?;
        if let Some(v) = self.delegator_vote.as_ref() {
            struct_ser.serialize_field("delegatorVote", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for delegator_vote_view::Visible {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_vote",
            "delegatorVote",
            "note",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorVote,
            Note,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
                            "note" => Ok(GeneratedField::Note),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = delegator_vote_view::Visible;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.DelegatorVoteView.Visible")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<delegator_vote_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_vote__ = None;
                let mut note__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DelegatorVote => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            delegator_vote__ = map_.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map_.next_value()?;
                        }
                    }
                }
                Ok(delegator_vote_view::Visible {
                    delegator_vote: delegator_vote__,
                    note: note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.DelegatorVoteView.Visible", FIELDS, GeneratedVisitor)
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
        if self.governance_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.GenesisContent", len)?;
        if let Some(v) = self.governance_params.as_ref() {
            struct_ser.serialize_field("governanceParams", v)?;
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
            "governance_params",
            "governanceParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GovernanceParams,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "governanceParams" | "governance_params" => Ok(GeneratedField::GovernanceParams),
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
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut governance_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::GovernanceParams => {
                            if governance_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("governanceParams"));
                            }
                            governance_params__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    governance_params: governance_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GovernanceParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_voting_blocks != 0 {
            len += 1;
        }
        if self.proposal_deposit_amount.is_some() {
            len += 1;
        }
        if !self.proposal_valid_quorum.is_empty() {
            len += 1;
        }
        if !self.proposal_pass_threshold.is_empty() {
            len += 1;
        }
        if !self.proposal_slash_threshold.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.GovernanceParameters", len)?;
        if self.proposal_voting_blocks != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposalVotingBlocks", ToString::to_string(&self.proposal_voting_blocks).as_str())?;
        }
        if let Some(v) = self.proposal_deposit_amount.as_ref() {
            struct_ser.serialize_field("proposalDepositAmount", v)?;
        }
        if !self.proposal_valid_quorum.is_empty() {
            struct_ser.serialize_field("proposalValidQuorum", &self.proposal_valid_quorum)?;
        }
        if !self.proposal_pass_threshold.is_empty() {
            struct_ser.serialize_field("proposalPassThreshold", &self.proposal_pass_threshold)?;
        }
        if !self.proposal_slash_threshold.is_empty() {
            struct_ser.serialize_field("proposalSlashThreshold", &self.proposal_slash_threshold)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GovernanceParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_voting_blocks",
            "proposalVotingBlocks",
            "proposal_deposit_amount",
            "proposalDepositAmount",
            "proposal_valid_quorum",
            "proposalValidQuorum",
            "proposal_pass_threshold",
            "proposalPassThreshold",
            "proposal_slash_threshold",
            "proposalSlashThreshold",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalVotingBlocks,
            ProposalDepositAmount,
            ProposalValidQuorum,
            ProposalPassThreshold,
            ProposalSlashThreshold,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "proposalVotingBlocks" | "proposal_voting_blocks" => Ok(GeneratedField::ProposalVotingBlocks),
                            "proposalDepositAmount" | "proposal_deposit_amount" => Ok(GeneratedField::ProposalDepositAmount),
                            "proposalValidQuorum" | "proposal_valid_quorum" => Ok(GeneratedField::ProposalValidQuorum),
                            "proposalPassThreshold" | "proposal_pass_threshold" => Ok(GeneratedField::ProposalPassThreshold),
                            "proposalSlashThreshold" | "proposal_slash_threshold" => Ok(GeneratedField::ProposalSlashThreshold),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GovernanceParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.GovernanceParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GovernanceParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_voting_blocks__ = None;
                let mut proposal_deposit_amount__ = None;
                let mut proposal_valid_quorum__ = None;
                let mut proposal_pass_threshold__ = None;
                let mut proposal_slash_threshold__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ProposalVotingBlocks => {
                            if proposal_voting_blocks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalVotingBlocks"));
                            }
                            proposal_voting_blocks__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProposalDepositAmount => {
                            if proposal_deposit_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalDepositAmount"));
                            }
                            proposal_deposit_amount__ = map_.next_value()?;
                        }
                        GeneratedField::ProposalValidQuorum => {
                            if proposal_valid_quorum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalValidQuorum"));
                            }
                            proposal_valid_quorum__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProposalPassThreshold => {
                            if proposal_pass_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalPassThreshold"));
                            }
                            proposal_pass_threshold__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProposalSlashThreshold => {
                            if proposal_slash_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalSlashThreshold"));
                            }
                            proposal_slash_threshold__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(GovernanceParameters {
                    proposal_voting_blocks: proposal_voting_blocks__.unwrap_or_default(),
                    proposal_deposit_amount: proposal_deposit_amount__,
                    proposal_valid_quorum: proposal_valid_quorum__.unwrap_or_default(),
                    proposal_pass_threshold: proposal_pass_threshold__.unwrap_or_default(),
                    proposal_slash_threshold: proposal_slash_threshold__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.GovernanceParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NextProposalIdRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.NextProposalIdRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NextProposalIdRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NextProposalIdRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.NextProposalIdRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NextProposalIdRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(NextProposalIdRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.NextProposalIdRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NextProposalIdResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.next_proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.NextProposalIdResponse", len)?;
        if self.next_proposal_id != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("nextProposalId", ToString::to_string(&self.next_proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NextProposalIdResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "next_proposal_id",
            "nextProposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NextProposalId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "nextProposalId" | "next_proposal_id" => Ok(GeneratedField::NextProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NextProposalIdResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.NextProposalIdResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NextProposalIdResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut next_proposal_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::NextProposalId => {
                            if next_proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextProposalId"));
                            }
                            next_proposal_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(NextProposalIdResponse {
                    next_proposal_id: next_proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.NextProposalIdResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Proposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if !self.title.is_empty() {
            len += 1;
        }
        if !self.description.is_empty() {
            len += 1;
        }
        if self.signaling.is_some() {
            len += 1;
        }
        if self.emergency.is_some() {
            len += 1;
        }
        if self.parameter_change.is_some() {
            len += 1;
        }
        if self.community_pool_spend.is_some() {
            len += 1;
        }
        if self.upgrade_plan.is_some() {
            len += 1;
        }
        if self.freeze_ibc_client.is_some() {
            len += 1;
        }
        if self.unfreeze_ibc_client.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.Proposal", len)?;
        if self.id != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("id", ToString::to_string(&self.id).as_str())?;
        }
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if let Some(v) = self.signaling.as_ref() {
            struct_ser.serialize_field("signaling", v)?;
        }
        if let Some(v) = self.emergency.as_ref() {
            struct_ser.serialize_field("emergency", v)?;
        }
        if let Some(v) = self.parameter_change.as_ref() {
            struct_ser.serialize_field("parameterChange", v)?;
        }
        if let Some(v) = self.community_pool_spend.as_ref() {
            struct_ser.serialize_field("communityPoolSpend", v)?;
        }
        if let Some(v) = self.upgrade_plan.as_ref() {
            struct_ser.serialize_field("upgradePlan", v)?;
        }
        if let Some(v) = self.freeze_ibc_client.as_ref() {
            struct_ser.serialize_field("freezeIbcClient", v)?;
        }
        if let Some(v) = self.unfreeze_ibc_client.as_ref() {
            struct_ser.serialize_field("unfreezeIbcClient", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Proposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "title",
            "description",
            "signaling",
            "emergency",
            "parameter_change",
            "parameterChange",
            "community_pool_spend",
            "communityPoolSpend",
            "upgrade_plan",
            "upgradePlan",
            "freeze_ibc_client",
            "freezeIbcClient",
            "unfreeze_ibc_client",
            "unfreezeIbcClient",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Title,
            Description,
            Signaling,
            Emergency,
            ParameterChange,
            CommunityPoolSpend,
            UpgradePlan,
            FreezeIbcClient,
            UnfreezeIbcClient,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "title" => Ok(GeneratedField::Title),
                            "description" => Ok(GeneratedField::Description),
                            "signaling" => Ok(GeneratedField::Signaling),
                            "emergency" => Ok(GeneratedField::Emergency),
                            "parameterChange" | "parameter_change" => Ok(GeneratedField::ParameterChange),
                            "communityPoolSpend" | "community_pool_spend" => Ok(GeneratedField::CommunityPoolSpend),
                            "upgradePlan" | "upgrade_plan" => Ok(GeneratedField::UpgradePlan),
                            "freezeIbcClient" | "freeze_ibc_client" => Ok(GeneratedField::FreezeIbcClient),
                            "unfreezeIbcClient" | "unfreeze_ibc_client" => Ok(GeneratedField::UnfreezeIbcClient),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Proposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.Proposal")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Proposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut title__ = None;
                let mut description__ = None;
                let mut signaling__ = None;
                let mut emergency__ = None;
                let mut parameter_change__ = None;
                let mut community_pool_spend__ = None;
                let mut upgrade_plan__ = None;
                let mut freeze_ibc_client__ = None;
                let mut unfreeze_ibc_client__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Signaling => {
                            if signaling__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signaling"));
                            }
                            signaling__ = map_.next_value()?;
                        }
                        GeneratedField::Emergency => {
                            if emergency__.is_some() {
                                return Err(serde::de::Error::duplicate_field("emergency"));
                            }
                            emergency__ = map_.next_value()?;
                        }
                        GeneratedField::ParameterChange => {
                            if parameter_change__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameterChange"));
                            }
                            parameter_change__ = map_.next_value()?;
                        }
                        GeneratedField::CommunityPoolSpend => {
                            if community_pool_spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolSpend"));
                            }
                            community_pool_spend__ = map_.next_value()?;
                        }
                        GeneratedField::UpgradePlan => {
                            if upgrade_plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("upgradePlan"));
                            }
                            upgrade_plan__ = map_.next_value()?;
                        }
                        GeneratedField::FreezeIbcClient => {
                            if freeze_ibc_client__.is_some() {
                                return Err(serde::de::Error::duplicate_field("freezeIbcClient"));
                            }
                            freeze_ibc_client__ = map_.next_value()?;
                        }
                        GeneratedField::UnfreezeIbcClient => {
                            if unfreeze_ibc_client__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unfreezeIbcClient"));
                            }
                            unfreeze_ibc_client__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Proposal {
                    id: id__.unwrap_or_default(),
                    title: title__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    signaling: signaling__,
                    emergency: emergency__,
                    parameter_change: parameter_change__,
                    community_pool_spend: community_pool_spend__,
                    upgrade_plan: upgrade_plan__,
                    freeze_ibc_client: freeze_ibc_client__,
                    unfreeze_ibc_client: unfreeze_ibc_client__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.Proposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::CommunityPoolSpend {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.CommunityPoolSpend", len)?;
        if let Some(v) = self.transaction_plan.as_ref() {
            struct_ser.serialize_field("transactionPlan", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::CommunityPoolSpend {
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
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::CommunityPoolSpend;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.Proposal.CommunityPoolSpend")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal::CommunityPoolSpend, V::Error>
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
                    }
                }
                Ok(proposal::CommunityPoolSpend {
                    transaction_plan: transaction_plan__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.CommunityPoolSpend", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::Emergency {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.halt_chain {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.Emergency", len)?;
        if self.halt_chain {
            struct_ser.serialize_field("haltChain", &self.halt_chain)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::Emergency {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "halt_chain",
            "haltChain",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            HaltChain,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "haltChain" | "halt_chain" => Ok(GeneratedField::HaltChain),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::Emergency;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.Proposal.Emergency")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal::Emergency, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut halt_chain__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::HaltChain => {
                            if halt_chain__.is_some() {
                                return Err(serde::de::Error::duplicate_field("haltChain"));
                            }
                            halt_chain__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(proposal::Emergency {
                    halt_chain: halt_chain__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.Emergency", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::FreezeIbcClient {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.FreezeIbcClient", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::FreezeIbcClient {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::FreezeIbcClient;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.Proposal.FreezeIbcClient")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal::FreezeIbcClient, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(proposal::FreezeIbcClient {
                    client_id: client_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.FreezeIbcClient", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::ParameterChange {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.old_parameters.is_some() {
            len += 1;
        }
        if self.new_parameters.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.ParameterChange", len)?;
        if let Some(v) = self.old_parameters.as_ref() {
            struct_ser.serialize_field("oldParameters", v)?;
        }
        if let Some(v) = self.new_parameters.as_ref() {
            struct_ser.serialize_field("newParameters", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::ParameterChange {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "old_parameters",
            "oldParameters",
            "new_parameters",
            "newParameters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OldParameters,
            NewParameters,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "oldParameters" | "old_parameters" => Ok(GeneratedField::OldParameters),
                            "newParameters" | "new_parameters" => Ok(GeneratedField::NewParameters),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::ParameterChange;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.Proposal.ParameterChange")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal::ParameterChange, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut old_parameters__ = None;
                let mut new_parameters__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OldParameters => {
                            if old_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("oldParameters"));
                            }
                            old_parameters__ = map_.next_value()?;
                        }
                        GeneratedField::NewParameters => {
                            if new_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newParameters"));
                            }
                            new_parameters__ = map_.next_value()?;
                        }
                    }
                }
                Ok(proposal::ParameterChange {
                    old_parameters: old_parameters__,
                    new_parameters: new_parameters__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.ParameterChange", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::Signaling {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.commit.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.Signaling", len)?;
        if !self.commit.is_empty() {
            struct_ser.serialize_field("commit", &self.commit)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::Signaling {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commit,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "commit" => Ok(GeneratedField::Commit),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::Signaling;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.Proposal.Signaling")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal::Signaling, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commit__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Commit => {
                            if commit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commit"));
                            }
                            commit__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(proposal::Signaling {
                    commit: commit__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.Signaling", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::UnfreezeIbcClient {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.UnfreezeIbcClient", len)?;
        if !self.client_id.is_empty() {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::UnfreezeIbcClient {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::UnfreezeIbcClient;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.Proposal.UnfreezeIbcClient")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal::UnfreezeIbcClient, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(proposal::UnfreezeIbcClient {
                    client_id: client_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.UnfreezeIbcClient", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::UpgradePlan {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.UpgradePlan", len)?;
        if self.height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::UpgradePlan {
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
            type Value = proposal::UpgradePlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.Proposal.UpgradePlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal::UpgradePlan, V::Error>
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
                    }
                }
                Ok(proposal::UpgradePlan {
                    height: height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.Proposal.UpgradePlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalDataRequest {
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
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalDataRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.proposal_id != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalDataRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            ProposalId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalDataRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalDataRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalDataRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut proposal_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ProposalDataRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalDataRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalDataResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal.is_some() {
            len += 1;
        }
        if self.start_block_height != 0 {
            len += 1;
        }
        if self.end_block_height != 0 {
            len += 1;
        }
        if self.start_position != 0 {
            len += 1;
        }
        if self.state.is_some() {
            len += 1;
        }
        if self.proposal_deposit_amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalDataResponse", len)?;
        if let Some(v) = self.proposal.as_ref() {
            struct_ser.serialize_field("proposal", v)?;
        }
        if self.start_block_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startBlockHeight", ToString::to_string(&self.start_block_height).as_str())?;
        }
        if self.end_block_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("endBlockHeight", ToString::to_string(&self.end_block_height).as_str())?;
        }
        if self.start_position != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startPosition", ToString::to_string(&self.start_position).as_str())?;
        }
        if let Some(v) = self.state.as_ref() {
            struct_ser.serialize_field("state", v)?;
        }
        if let Some(v) = self.proposal_deposit_amount.as_ref() {
            struct_ser.serialize_field("proposalDepositAmount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalDataResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "start_block_height",
            "startBlockHeight",
            "end_block_height",
            "endBlockHeight",
            "start_position",
            "startPosition",
            "state",
            "proposal_deposit_amount",
            "proposalDepositAmount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            StartBlockHeight,
            EndBlockHeight,
            StartPosition,
            State,
            ProposalDepositAmount,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "startBlockHeight" | "start_block_height" => Ok(GeneratedField::StartBlockHeight),
                            "endBlockHeight" | "end_block_height" => Ok(GeneratedField::EndBlockHeight),
                            "startPosition" | "start_position" => Ok(GeneratedField::StartPosition),
                            "state" => Ok(GeneratedField::State),
                            "proposalDepositAmount" | "proposal_deposit_amount" => Ok(GeneratedField::ProposalDepositAmount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalDataResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalDataResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalDataResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut start_block_height__ = None;
                let mut end_block_height__ = None;
                let mut start_position__ = None;
                let mut state__ = None;
                let mut proposal_deposit_amount__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = map_.next_value()?;
                        }
                        GeneratedField::StartBlockHeight => {
                            if start_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startBlockHeight"));
                            }
                            start_block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndBlockHeight => {
                            if end_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endBlockHeight"));
                            }
                            end_block_height__ = 
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
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map_.next_value()?;
                        }
                        GeneratedField::ProposalDepositAmount => {
                            if proposal_deposit_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalDepositAmount"));
                            }
                            proposal_deposit_amount__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ProposalDataResponse {
                    proposal: proposal__,
                    start_block_height: start_block_height__.unwrap_or_default(),
                    end_block_height: end_block_height__.unwrap_or_default(),
                    start_position: start_position__.unwrap_or_default(),
                    state: state__,
                    proposal_deposit_amount: proposal_deposit_amount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalDataResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalDepositClaim {
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
        if self.deposit_amount.is_some() {
            len += 1;
        }
        if self.outcome.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalDepositClaim", len)?;
        if self.proposal != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if let Some(v) = self.deposit_amount.as_ref() {
            struct_ser.serialize_field("depositAmount", v)?;
        }
        if let Some(v) = self.outcome.as_ref() {
            struct_ser.serialize_field("outcome", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalDepositClaim {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "deposit_amount",
            "depositAmount",
            "outcome",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            DepositAmount,
            Outcome,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "depositAmount" | "deposit_amount" => Ok(GeneratedField::DepositAmount),
                            "outcome" => Ok(GeneratedField::Outcome),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalDepositClaim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalDepositClaim")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalDepositClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut deposit_amount__ = None;
                let mut outcome__ = None;
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
                        GeneratedField::DepositAmount => {
                            if deposit_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositAmount"));
                            }
                            deposit_amount__ = map_.next_value()?;
                        }
                        GeneratedField::Outcome => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outcome"));
                            }
                            outcome__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ProposalDepositClaim {
                    proposal: proposal__.unwrap_or_default(),
                    deposit_amount: deposit_amount__,
                    outcome: outcome__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalDepositClaim", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalInfoRequest {
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
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalInfoRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.proposal_id != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalInfoRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            ProposalId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalInfoRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalInfoRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalInfoRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut proposal_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ProposalInfoRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalInfoRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalInfoResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_block_height != 0 {
            len += 1;
        }
        if self.start_position != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalInfoResponse", len)?;
        if self.start_block_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startBlockHeight", ToString::to_string(&self.start_block_height).as_str())?;
        }
        if self.start_position != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startPosition", ToString::to_string(&self.start_position).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalInfoResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_block_height",
            "startBlockHeight",
            "start_position",
            "startPosition",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartBlockHeight,
            StartPosition,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "startBlockHeight" | "start_block_height" => Ok(GeneratedField::StartBlockHeight),
                            "startPosition" | "start_position" => Ok(GeneratedField::StartPosition),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalInfoResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalInfoResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalInfoResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_block_height__ = None;
                let mut start_position__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                    }
                }
                Ok(ProposalInfoResponse {
                    start_block_height: start_block_height__.unwrap_or_default(),
                    start_position: start_position__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalInfoResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalListRequest {
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
        if self.inactive {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalListRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.inactive {
            struct_ser.serialize_field("inactive", &self.inactive)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalListRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "inactive",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            Inactive,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "inactive" => Ok(GeneratedField::Inactive),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalListRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalListRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalListRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut inactive__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Inactive => {
                            if inactive__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inactive"));
                            }
                            inactive__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ProposalListRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    inactive: inactive__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalListRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalListResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal.is_some() {
            len += 1;
        }
        if self.start_block_height != 0 {
            len += 1;
        }
        if self.end_block_height != 0 {
            len += 1;
        }
        if self.start_position != 0 {
            len += 1;
        }
        if self.state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalListResponse", len)?;
        if let Some(v) = self.proposal.as_ref() {
            struct_ser.serialize_field("proposal", v)?;
        }
        if self.start_block_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startBlockHeight", ToString::to_string(&self.start_block_height).as_str())?;
        }
        if self.end_block_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("endBlockHeight", ToString::to_string(&self.end_block_height).as_str())?;
        }
        if self.start_position != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startPosition", ToString::to_string(&self.start_position).as_str())?;
        }
        if let Some(v) = self.state.as_ref() {
            struct_ser.serialize_field("state", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalListResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "start_block_height",
            "startBlockHeight",
            "end_block_height",
            "endBlockHeight",
            "start_position",
            "startPosition",
            "state",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            StartBlockHeight,
            EndBlockHeight,
            StartPosition,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "startBlockHeight" | "start_block_height" => Ok(GeneratedField::StartBlockHeight),
                            "endBlockHeight" | "end_block_height" => Ok(GeneratedField::EndBlockHeight),
                            "startPosition" | "start_position" => Ok(GeneratedField::StartPosition),
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
            type Value = ProposalListResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalListResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalListResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut start_block_height__ = None;
                let mut end_block_height__ = None;
                let mut start_position__ = None;
                let mut state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = map_.next_value()?;
                        }
                        GeneratedField::StartBlockHeight => {
                            if start_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startBlockHeight"));
                            }
                            start_block_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndBlockHeight => {
                            if end_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endBlockHeight"));
                            }
                            end_block_height__ = 
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
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ProposalListResponse {
                    proposal: proposal__,
                    start_block_height: start_block_height__.unwrap_or_default(),
                    end_block_height: end_block_height__.unwrap_or_default(),
                    start_position: start_position__.unwrap_or_default(),
                    state: state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalListResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalOutcome {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.outcome.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalOutcome", len)?;
        if let Some(v) = self.outcome.as_ref() {
            match v {
                proposal_outcome::Outcome::Passed(v) => {
                    struct_ser.serialize_field("passed", v)?;
                }
                proposal_outcome::Outcome::Failed(v) => {
                    struct_ser.serialize_field("failed", v)?;
                }
                proposal_outcome::Outcome::Slashed(v) => {
                    struct_ser.serialize_field("slashed", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalOutcome {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "passed",
            "failed",
            "slashed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Passed,
            Failed,
            Slashed,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "passed" => Ok(GeneratedField::Passed),
                            "failed" => Ok(GeneratedField::Failed),
                            "slashed" => Ok(GeneratedField::Slashed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalOutcome;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalOutcome")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalOutcome, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut outcome__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Passed => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("passed"));
                            }
                            outcome__ = map_.next_value::<::std::option::Option<_>>()?.map(proposal_outcome::Outcome::Passed)
;
                        }
                        GeneratedField::Failed => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("failed"));
                            }
                            outcome__ = map_.next_value::<::std::option::Option<_>>()?.map(proposal_outcome::Outcome::Failed)
;
                        }
                        GeneratedField::Slashed => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slashed"));
                            }
                            outcome__ = map_.next_value::<::std::option::Option<_>>()?.map(proposal_outcome::Outcome::Slashed)
;
                        }
                    }
                }
                Ok(ProposalOutcome {
                    outcome: outcome__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalOutcome", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_outcome::Failed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.withdrawn.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalOutcome.Failed", len)?;
        if let Some(v) = self.withdrawn.as_ref() {
            struct_ser.serialize_field("withdrawn", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_outcome::Failed {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "withdrawn",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Withdrawn,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "withdrawn" => Ok(GeneratedField::Withdrawn),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_outcome::Failed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalOutcome.Failed")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal_outcome::Failed, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut withdrawn__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Withdrawn => {
                            if withdrawn__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawn"));
                            }
                            withdrawn__ = map_.next_value()?;
                        }
                    }
                }
                Ok(proposal_outcome::Failed {
                    withdrawn: withdrawn__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalOutcome.Failed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_outcome::Passed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalOutcome.Passed", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_outcome::Passed {
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
            type Value = proposal_outcome::Passed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalOutcome.Passed")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal_outcome::Passed, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(proposal_outcome::Passed {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalOutcome.Passed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_outcome::Slashed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.withdrawn.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalOutcome.Slashed", len)?;
        if let Some(v) = self.withdrawn.as_ref() {
            struct_ser.serialize_field("withdrawn", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_outcome::Slashed {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "withdrawn",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Withdrawn,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "withdrawn" => Ok(GeneratedField::Withdrawn),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_outcome::Slashed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalOutcome.Slashed")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal_outcome::Slashed, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut withdrawn__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Withdrawn => {
                            if withdrawn__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawn"));
                            }
                            withdrawn__ = map_.next_value()?;
                        }
                    }
                }
                Ok(proposal_outcome::Slashed {
                    withdrawn: withdrawn__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalOutcome.Slashed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_outcome::Withdrawn {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.reason.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalOutcome.Withdrawn", len)?;
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_outcome::Withdrawn {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "reason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Reason,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "reason" => Ok(GeneratedField::Reason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_outcome::Withdrawn;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalOutcome.Withdrawn")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal_outcome::Withdrawn, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut reason__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(proposal_outcome::Withdrawn {
                    reason: reason__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalOutcome.Withdrawn", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalRateDataRequest {
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
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalRateDataRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.proposal_id != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalRateDataRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            ProposalId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalRateDataRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalRateDataRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalRateDataRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut proposal_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ProposalRateDataRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalRateDataRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalRateDataResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.rate_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalRateDataResponse", len)?;
        if let Some(v) = self.rate_data.as_ref() {
            struct_ser.serialize_field("rateData", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalRateDataResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rate_data",
            "rateData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = ProposalRateDataResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalRateDataResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalRateDataResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rate_data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RateData => {
                            if rate_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rateData"));
                            }
                            rate_data__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ProposalRateDataResponse {
                    rate_data: rate_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalRateDataResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalState {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalState", len)?;
        if let Some(v) = self.state.as_ref() {
            match v {
                proposal_state::State::Voting(v) => {
                    struct_ser.serialize_field("voting", v)?;
                }
                proposal_state::State::Withdrawn(v) => {
                    struct_ser.serialize_field("withdrawn", v)?;
                }
                proposal_state::State::Finished(v) => {
                    struct_ser.serialize_field("finished", v)?;
                }
                proposal_state::State::Claimed(v) => {
                    struct_ser.serialize_field("claimed", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "voting",
            "withdrawn",
            "finished",
            "claimed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Voting,
            Withdrawn,
            Finished,
            Claimed,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "voting" => Ok(GeneratedField::Voting),
                            "withdrawn" => Ok(GeneratedField::Withdrawn),
                            "finished" => Ok(GeneratedField::Finished),
                            "claimed" => Ok(GeneratedField::Claimed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Voting => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voting"));
                            }
                            state__ = map_.next_value::<::std::option::Option<_>>()?.map(proposal_state::State::Voting)
;
                        }
                        GeneratedField::Withdrawn => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawn"));
                            }
                            state__ = map_.next_value::<::std::option::Option<_>>()?.map(proposal_state::State::Withdrawn)
;
                        }
                        GeneratedField::Finished => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finished"));
                            }
                            state__ = map_.next_value::<::std::option::Option<_>>()?.map(proposal_state::State::Finished)
;
                        }
                        GeneratedField::Claimed => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("claimed"));
                            }
                            state__ = map_.next_value::<::std::option::Option<_>>()?.map(proposal_state::State::Claimed)
;
                        }
                    }
                }
                Ok(ProposalState {
                    state: state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_state::Claimed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.outcome.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalState.Claimed", len)?;
        if let Some(v) = self.outcome.as_ref() {
            struct_ser.serialize_field("outcome", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_state::Claimed {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "outcome",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Outcome,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "outcome" => Ok(GeneratedField::Outcome),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_state::Claimed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalState.Claimed")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal_state::Claimed, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut outcome__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Outcome => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outcome"));
                            }
                            outcome__ = map_.next_value()?;
                        }
                    }
                }
                Ok(proposal_state::Claimed {
                    outcome: outcome__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalState.Claimed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_state::Finished {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.outcome.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalState.Finished", len)?;
        if let Some(v) = self.outcome.as_ref() {
            struct_ser.serialize_field("outcome", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_state::Finished {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "outcome",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Outcome,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "outcome" => Ok(GeneratedField::Outcome),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_state::Finished;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalState.Finished")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal_state::Finished, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut outcome__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Outcome => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outcome"));
                            }
                            outcome__ = map_.next_value()?;
                        }
                    }
                }
                Ok(proposal_state::Finished {
                    outcome: outcome__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalState.Finished", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_state::Voting {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalState.Voting", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_state::Voting {
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
            type Value = proposal_state::Voting;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalState.Voting")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal_state::Voting, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(proposal_state::Voting {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalState.Voting", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_state::Withdrawn {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.reason.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalState.Withdrawn", len)?;
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_state::Withdrawn {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "reason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Reason,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "reason" => Ok(GeneratedField::Reason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_state::Withdrawn;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalState.Withdrawn")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<proposal_state::Withdrawn, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut reason__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(proposal_state::Withdrawn {
                    reason: reason__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalState.Withdrawn", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalSubmit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal.is_some() {
            len += 1;
        }
        if self.deposit_amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalSubmit", len)?;
        if let Some(v) = self.proposal.as_ref() {
            struct_ser.serialize_field("proposal", v)?;
        }
        if let Some(v) = self.deposit_amount.as_ref() {
            struct_ser.serialize_field("depositAmount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalSubmit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "deposit_amount",
            "depositAmount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            DepositAmount,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "depositAmount" | "deposit_amount" => Ok(GeneratedField::DepositAmount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalSubmit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalSubmit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalSubmit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut deposit_amount__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = map_.next_value()?;
                        }
                        GeneratedField::DepositAmount => {
                            if deposit_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositAmount"));
                            }
                            deposit_amount__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ProposalSubmit {
                    proposal: proposal__,
                    deposit_amount: deposit_amount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalSubmit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalWithdraw {
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
        if !self.reason.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ProposalWithdraw", len)?;
        if self.proposal != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalWithdraw {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "reason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            Reason,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "reason" => Ok(GeneratedField::Reason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalWithdraw;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ProposalWithdraw")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProposalWithdraw, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut reason__ = None;
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
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ProposalWithdraw {
                    proposal: proposal__.unwrap_or_default(),
                    reason: reason__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ProposalWithdraw", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Tally {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.yes != 0 {
            len += 1;
        }
        if self.no != 0 {
            len += 1;
        }
        if self.abstain != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.Tally", len)?;
        if self.yes != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("yes", ToString::to_string(&self.yes).as_str())?;
        }
        if self.no != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("no", ToString::to_string(&self.no).as_str())?;
        }
        if self.abstain != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("abstain", ToString::to_string(&self.abstain).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Tally {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "yes",
            "no",
            "abstain",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Yes,
            No,
            Abstain,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "yes" => Ok(GeneratedField::Yes),
                            "no" => Ok(GeneratedField::No),
                            "abstain" => Ok(GeneratedField::Abstain),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Tally;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.Tally")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Tally, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut yes__ = None;
                let mut no__ = None;
                let mut abstain__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Yes => {
                            if yes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("yes"));
                            }
                            yes__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::No => {
                            if no__.is_some() {
                                return Err(serde::de::Error::duplicate_field("no"));
                            }
                            no__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Abstain => {
                            if abstain__.is_some() {
                                return Err(serde::de::Error::duplicate_field("abstain"));
                            }
                            abstain__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Tally {
                    yes: yes__.unwrap_or_default(),
                    no: no__.unwrap_or_default(),
                    abstain: abstain__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.Tally", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorVote {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ValidatorVote", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.auth_sig.as_ref() {
            struct_ser.serialize_field("authSig", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorVote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body",
            "auth_sig",
            "authSig",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Body,
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
                            "body" => Ok(GeneratedField::Body),
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
            type Value = ValidatorVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ValidatorVote")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut auth_sig__ = None;
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
                    }
                }
                Ok(ValidatorVote {
                    body: body__,
                    auth_sig: auth_sig__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ValidatorVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorVoteBody {
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
        if self.identity_key.is_some() {
            len += 1;
        }
        if self.governance_key.is_some() {
            len += 1;
        }
        if self.reason.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ValidatorVoteBody", len)?;
        if self.proposal != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        if let Some(v) = self.governance_key.as_ref() {
            struct_ser.serialize_field("governanceKey", v)?;
        }
        if let Some(v) = self.reason.as_ref() {
            struct_ser.serialize_field("reason", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorVoteBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "vote",
            "identity_key",
            "identityKey",
            "governance_key",
            "governanceKey",
            "reason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            Vote,
            IdentityKey,
            GovernanceKey,
            Reason,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "identityKey" | "identity_key" => Ok(GeneratedField::IdentityKey),
                            "governanceKey" | "governance_key" => Ok(GeneratedField::GovernanceKey),
                            "reason" => Ok(GeneratedField::Reason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorVoteBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ValidatorVoteBody")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorVoteBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut vote__ = None;
                let mut identity_key__ = None;
                let mut governance_key__ = None;
                let mut reason__ = None;
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
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                        GeneratedField::GovernanceKey => {
                            if governance_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("governanceKey"));
                            }
                            governance_key__ = map_.next_value()?;
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ValidatorVoteBody {
                    proposal: proposal__.unwrap_or_default(),
                    vote: vote__,
                    identity_key: identity_key__,
                    governance_key: governance_key__,
                    reason: reason__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ValidatorVoteBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorVoteReason {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.reason.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ValidatorVoteReason", len)?;
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorVoteReason {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "reason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Reason,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "reason" => Ok(GeneratedField::Reason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorVoteReason;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ValidatorVoteReason")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorVoteReason, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut reason__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ValidatorVoteReason {
                    reason: reason__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ValidatorVoteReason", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorVotesRequest {
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
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ValidatorVotesRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.proposal_id != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorVotesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            ProposalId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorVotesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ValidatorVotesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorVotesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut proposal_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ValidatorVotesRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ValidatorVotesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorVotesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vote.is_some() {
            len += 1;
        }
        if self.identity_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ValidatorVotesResponse", len)?;
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorVotesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vote",
            "identity_key",
            "identityKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vote,
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
                            "vote" => Ok(GeneratedField::Vote),
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
            type Value = ValidatorVotesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ValidatorVotesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ValidatorVotesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vote__ = None;
                let mut identity_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map_.next_value()?;
                        }
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ValidatorVotesResponse {
                    vote: vote__,
                    identity_key: identity_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ValidatorVotesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Vote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vote != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.Vote", len)?;
        if self.vote != 0 {
            let v = vote::Vote::try_from(self.vote)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.vote)))?;
            struct_ser.serialize_field("vote", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Vote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vote",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vote,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vote" => Ok(GeneratedField::Vote),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Vote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.Vote")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Vote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vote__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = Some(map_.next_value::<vote::Vote>()? as i32);
                        }
                    }
                }
                Ok(Vote {
                    vote: vote__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.Vote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for vote::Vote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "VOTE_UNSPECIFIED",
            Self::Abstain => "VOTE_ABSTAIN",
            Self::Yes => "VOTE_YES",
            Self::No => "VOTE_NO",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for vote::Vote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "VOTE_UNSPECIFIED",
            "VOTE_ABSTAIN",
            "VOTE_YES",
            "VOTE_NO",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = vote::Vote;

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
                    "VOTE_UNSPECIFIED" => Ok(vote::Vote::Unspecified),
                    "VOTE_ABSTAIN" => Ok(vote::Vote::Abstain),
                    "VOTE_YES" => Ok(vote::Vote::Yes),
                    "VOTE_NO" => Ok(vote::Vote::No),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for VotingPowerAtProposalStartRequest {
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
        if self.proposal_id != 0 {
            len += 1;
        }
        if self.identity_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.VotingPowerAtProposalStartRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.proposal_id != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VotingPowerAtProposalStartRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "proposal_id",
            "proposalId",
            "identity_key",
            "identityKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
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
            type Value = VotingPowerAtProposalStartRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.VotingPowerAtProposalStartRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<VotingPowerAtProposalStartRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut proposal_id__ = None;
                let mut identity_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map_.next_value()?;
                        }
                    }
                }
                Ok(VotingPowerAtProposalStartRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                    proposal_id: proposal_id__.unwrap_or_default(),
                    identity_key: identity_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.VotingPowerAtProposalStartRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VotingPowerAtProposalStartResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.voting_power != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.VotingPowerAtProposalStartResponse", len)?;
        if self.voting_power != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("votingPower", ToString::to_string(&self.voting_power).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VotingPowerAtProposalStartResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "voting_power",
            "votingPower",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VotingPower,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "votingPower" | "voting_power" => Ok(GeneratedField::VotingPower),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VotingPowerAtProposalStartResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.VotingPowerAtProposalStartResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<VotingPowerAtProposalStartResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut voting_power__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::VotingPower => {
                            if voting_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingPower"));
                            }
                            voting_power__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(VotingPowerAtProposalStartResponse {
                    voting_power: voting_power__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.VotingPowerAtProposalStartResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZkDelegatorVoteProof {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.governance.v1alpha1.ZKDelegatorVoteProof", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZkDelegatorVoteProof {
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
            type Value = ZkDelegatorVoteProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.governance.v1alpha1.ZKDelegatorVoteProof")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZkDelegatorVoteProof, V::Error>
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
                Ok(ZkDelegatorVoteProof {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.governance.v1alpha1.ZKDelegatorVoteProof", FIELDS, GeneratedVisitor)
    }
}
