impl serde::Serialize for CeremonyCrs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.spend.is_empty() {
            len += 1;
        }
        if !self.output.is_empty() {
            len += 1;
        }
        if !self.delegator_vote.is_empty() {
            len += 1;
        }
        if !self.undelegate_claim.is_empty() {
            len += 1;
        }
        if !self.swap.is_empty() {
            len += 1;
        }
        if !self.swap_claim.is_empty() {
            len += 1;
        }
        if !self.nullifer_derivation_crs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.tools.summoning.v1.CeremonyCrs", len)?;
        if !self.spend.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("spend", pbjson::private::base64::encode(&self.spend).as_str())?;
        }
        if !self.output.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("output", pbjson::private::base64::encode(&self.output).as_str())?;
        }
        if !self.delegator_vote.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("delegatorVote", pbjson::private::base64::encode(&self.delegator_vote).as_str())?;
        }
        if !self.undelegate_claim.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("undelegateClaim", pbjson::private::base64::encode(&self.undelegate_claim).as_str())?;
        }
        if !self.swap.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("swap", pbjson::private::base64::encode(&self.swap).as_str())?;
        }
        if !self.swap_claim.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("swapClaim", pbjson::private::base64::encode(&self.swap_claim).as_str())?;
        }
        if !self.nullifer_derivation_crs.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("nulliferDerivationCrs", pbjson::private::base64::encode(&self.nullifer_derivation_crs).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CeremonyCrs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend",
            "output",
            "delegator_vote",
            "delegatorVote",
            "undelegate_claim",
            "undelegateClaim",
            "swap",
            "swap_claim",
            "swapClaim",
            "nullifer_derivation_crs",
            "nulliferDerivationCrs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
            Output,
            DelegatorVote,
            UndelegateClaim,
            Swap,
            SwapClaim,
            NulliferDerivationCrs,
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
                            "output" => Ok(GeneratedField::Output),
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
                            "undelegateClaim" | "undelegate_claim" => Ok(GeneratedField::UndelegateClaim),
                            "swap" => Ok(GeneratedField::Swap),
                            "swapClaim" | "swap_claim" => Ok(GeneratedField::SwapClaim),
                            "nulliferDerivationCrs" | "nullifer_derivation_crs" => Ok(GeneratedField::NulliferDerivationCrs),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CeremonyCrs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.tools.summoning.v1.CeremonyCrs")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CeremonyCrs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spend__ = None;
                let mut output__ = None;
                let mut delegator_vote__ = None;
                let mut undelegate_claim__ = None;
                let mut swap__ = None;
                let mut swap_claim__ = None;
                let mut nullifer_derivation_crs__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            spend__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DelegatorVote => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            delegator_vote__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UndelegateClaim => {
                            if undelegate_claim__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegateClaim"));
                            }
                            undelegate_claim__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SwapClaim => {
                            if swap_claim__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            swap_claim__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::NulliferDerivationCrs => {
                            if nullifer_derivation_crs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nulliferDerivationCrs"));
                            }
                            nullifer_derivation_crs__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CeremonyCrs {
                    spend: spend__.unwrap_or_default(),
                    output: output__.unwrap_or_default(),
                    delegator_vote: delegator_vote__.unwrap_or_default(),
                    undelegate_claim: undelegate_claim__.unwrap_or_default(),
                    swap: swap__.unwrap_or_default(),
                    swap_claim: swap_claim__.unwrap_or_default(),
                    nullifer_derivation_crs: nullifer_derivation_crs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.tools.summoning.v1.CeremonyCrs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CeremonyLinkingProof {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.spend.is_empty() {
            len += 1;
        }
        if !self.output.is_empty() {
            len += 1;
        }
        if !self.delegator_vote.is_empty() {
            len += 1;
        }
        if !self.undelegate_claim.is_empty() {
            len += 1;
        }
        if !self.swap.is_empty() {
            len += 1;
        }
        if !self.swap_claim.is_empty() {
            len += 1;
        }
        if !self.nullifer_derivation_crs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.tools.summoning.v1.CeremonyLinkingProof", len)?;
        if !self.spend.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("spend", pbjson::private::base64::encode(&self.spend).as_str())?;
        }
        if !self.output.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("output", pbjson::private::base64::encode(&self.output).as_str())?;
        }
        if !self.delegator_vote.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("delegatorVote", pbjson::private::base64::encode(&self.delegator_vote).as_str())?;
        }
        if !self.undelegate_claim.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("undelegateClaim", pbjson::private::base64::encode(&self.undelegate_claim).as_str())?;
        }
        if !self.swap.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("swap", pbjson::private::base64::encode(&self.swap).as_str())?;
        }
        if !self.swap_claim.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("swapClaim", pbjson::private::base64::encode(&self.swap_claim).as_str())?;
        }
        if !self.nullifer_derivation_crs.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("nulliferDerivationCrs", pbjson::private::base64::encode(&self.nullifer_derivation_crs).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CeremonyLinkingProof {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend",
            "output",
            "delegator_vote",
            "delegatorVote",
            "undelegate_claim",
            "undelegateClaim",
            "swap",
            "swap_claim",
            "swapClaim",
            "nullifer_derivation_crs",
            "nulliferDerivationCrs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
            Output,
            DelegatorVote,
            UndelegateClaim,
            Swap,
            SwapClaim,
            NulliferDerivationCrs,
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
                            "output" => Ok(GeneratedField::Output),
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
                            "undelegateClaim" | "undelegate_claim" => Ok(GeneratedField::UndelegateClaim),
                            "swap" => Ok(GeneratedField::Swap),
                            "swapClaim" | "swap_claim" => Ok(GeneratedField::SwapClaim),
                            "nulliferDerivationCrs" | "nullifer_derivation_crs" => Ok(GeneratedField::NulliferDerivationCrs),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CeremonyLinkingProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.tools.summoning.v1.CeremonyLinkingProof")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CeremonyLinkingProof, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spend__ = None;
                let mut output__ = None;
                let mut delegator_vote__ = None;
                let mut undelegate_claim__ = None;
                let mut swap__ = None;
                let mut swap_claim__ = None;
                let mut nullifer_derivation_crs__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            spend__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DelegatorVote => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            delegator_vote__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UndelegateClaim => {
                            if undelegate_claim__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegateClaim"));
                            }
                            undelegate_claim__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SwapClaim => {
                            if swap_claim__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            swap_claim__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::NulliferDerivationCrs => {
                            if nullifer_derivation_crs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nulliferDerivationCrs"));
                            }
                            nullifer_derivation_crs__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CeremonyLinkingProof {
                    spend: spend__.unwrap_or_default(),
                    output: output__.unwrap_or_default(),
                    delegator_vote: delegator_vote__.unwrap_or_default(),
                    undelegate_claim: undelegate_claim__.unwrap_or_default(),
                    swap: swap__.unwrap_or_default(),
                    swap_claim: swap_claim__.unwrap_or_default(),
                    nullifer_derivation_crs: nullifer_derivation_crs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.tools.summoning.v1.CeremonyLinkingProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CeremonyParentHashes {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.spend.is_empty() {
            len += 1;
        }
        if !self.output.is_empty() {
            len += 1;
        }
        if !self.delegator_vote.is_empty() {
            len += 1;
        }
        if !self.undelegate_claim.is_empty() {
            len += 1;
        }
        if !self.swap.is_empty() {
            len += 1;
        }
        if !self.swap_claim.is_empty() {
            len += 1;
        }
        if !self.nullifer_derivation_crs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.tools.summoning.v1.CeremonyParentHashes", len)?;
        if !self.spend.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("spend", pbjson::private::base64::encode(&self.spend).as_str())?;
        }
        if !self.output.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("output", pbjson::private::base64::encode(&self.output).as_str())?;
        }
        if !self.delegator_vote.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("delegatorVote", pbjson::private::base64::encode(&self.delegator_vote).as_str())?;
        }
        if !self.undelegate_claim.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("undelegateClaim", pbjson::private::base64::encode(&self.undelegate_claim).as_str())?;
        }
        if !self.swap.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("swap", pbjson::private::base64::encode(&self.swap).as_str())?;
        }
        if !self.swap_claim.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("swapClaim", pbjson::private::base64::encode(&self.swap_claim).as_str())?;
        }
        if !self.nullifer_derivation_crs.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("nulliferDerivationCrs", pbjson::private::base64::encode(&self.nullifer_derivation_crs).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CeremonyParentHashes {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend",
            "output",
            "delegator_vote",
            "delegatorVote",
            "undelegate_claim",
            "undelegateClaim",
            "swap",
            "swap_claim",
            "swapClaim",
            "nullifer_derivation_crs",
            "nulliferDerivationCrs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
            Output,
            DelegatorVote,
            UndelegateClaim,
            Swap,
            SwapClaim,
            NulliferDerivationCrs,
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
                            "output" => Ok(GeneratedField::Output),
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
                            "undelegateClaim" | "undelegate_claim" => Ok(GeneratedField::UndelegateClaim),
                            "swap" => Ok(GeneratedField::Swap),
                            "swapClaim" | "swap_claim" => Ok(GeneratedField::SwapClaim),
                            "nulliferDerivationCrs" | "nullifer_derivation_crs" => Ok(GeneratedField::NulliferDerivationCrs),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CeremonyParentHashes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.tools.summoning.v1.CeremonyParentHashes")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CeremonyParentHashes, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spend__ = None;
                let mut output__ = None;
                let mut delegator_vote__ = None;
                let mut undelegate_claim__ = None;
                let mut swap__ = None;
                let mut swap_claim__ = None;
                let mut nullifer_derivation_crs__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            spend__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DelegatorVote => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            delegator_vote__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UndelegateClaim => {
                            if undelegate_claim__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegateClaim"));
                            }
                            undelegate_claim__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SwapClaim => {
                            if swap_claim__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            swap_claim__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::NulliferDerivationCrs => {
                            if nullifer_derivation_crs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nulliferDerivationCrs"));
                            }
                            nullifer_derivation_crs__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CeremonyParentHashes {
                    spend: spend__.unwrap_or_default(),
                    output: output__.unwrap_or_default(),
                    delegator_vote: delegator_vote__.unwrap_or_default(),
                    undelegate_claim: undelegate_claim__.unwrap_or_default(),
                    swap: swap__.unwrap_or_default(),
                    swap_claim: swap_claim__.unwrap_or_default(),
                    nullifer_derivation_crs: nullifer_derivation_crs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.tools.summoning.v1.CeremonyParentHashes", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ParticipateRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.msg.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.tools.summoning.v1.ParticipateRequest", len)?;
        if let Some(v) = self.msg.as_ref() {
            match v {
                participate_request::Msg::Identify(v) => {
                    struct_ser.serialize_field("identify", v)?;
                }
                participate_request::Msg::Contribution(v) => {
                    struct_ser.serialize_field("contribution", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ParticipateRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "identify",
            "contribution",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Identify,
            Contribution,
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
                            "identify" => Ok(GeneratedField::Identify),
                            "contribution" => Ok(GeneratedField::Contribution),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ParticipateRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.tools.summoning.v1.ParticipateRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ParticipateRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut msg__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Identify => {
                            if msg__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identify"));
                            }
                            msg__ = map_.next_value::<::std::option::Option<_>>()?.map(participate_request::Msg::Identify)
;
                        }
                        GeneratedField::Contribution => {
                            if msg__.is_some() {
                                return Err(serde::de::Error::duplicate_field("contribution"));
                            }
                            msg__ = map_.next_value::<::std::option::Option<_>>()?.map(participate_request::Msg::Contribution)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ParticipateRequest {
                    msg: msg__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.tools.summoning.v1.ParticipateRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for participate_request::Contribution {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.updated.is_some() {
            len += 1;
        }
        if self.update_proofs.is_some() {
            len += 1;
        }
        if self.parent_hashes.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.tools.summoning.v1.ParticipateRequest.Contribution", len)?;
        if let Some(v) = self.updated.as_ref() {
            struct_ser.serialize_field("updated", v)?;
        }
        if let Some(v) = self.update_proofs.as_ref() {
            struct_ser.serialize_field("updateProofs", v)?;
        }
        if let Some(v) = self.parent_hashes.as_ref() {
            struct_ser.serialize_field("parentHashes", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for participate_request::Contribution {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "updated",
            "update_proofs",
            "updateProofs",
            "parent_hashes",
            "parentHashes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Updated,
            UpdateProofs,
            ParentHashes,
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
                            "updated" => Ok(GeneratedField::Updated),
                            "updateProofs" | "update_proofs" => Ok(GeneratedField::UpdateProofs),
                            "parentHashes" | "parent_hashes" => Ok(GeneratedField::ParentHashes),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = participate_request::Contribution;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.tools.summoning.v1.ParticipateRequest.Contribution")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<participate_request::Contribution, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut updated__ = None;
                let mut update_proofs__ = None;
                let mut parent_hashes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Updated => {
                            if updated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updated"));
                            }
                            updated__ = map_.next_value()?;
                        }
                        GeneratedField::UpdateProofs => {
                            if update_proofs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateProofs"));
                            }
                            update_proofs__ = map_.next_value()?;
                        }
                        GeneratedField::ParentHashes => {
                            if parent_hashes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parentHashes"));
                            }
                            parent_hashes__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(participate_request::Contribution {
                    updated: updated__,
                    update_proofs: update_proofs__,
                    parent_hashes: parent_hashes__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.tools.summoning.v1.ParticipateRequest.Contribution", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for participate_request::Identify {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.tools.summoning.v1.ParticipateRequest.Identify", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for participate_request::Identify {
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
            type Value = participate_request::Identify;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.tools.summoning.v1.ParticipateRequest.Identify")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<participate_request::Identify, V::Error>
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
                Ok(participate_request::Identify {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.tools.summoning.v1.ParticipateRequest.Identify", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ParticipateResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.msg.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.tools.summoning.v1.ParticipateResponse", len)?;
        if let Some(v) = self.msg.as_ref() {
            match v {
                participate_response::Msg::Position(v) => {
                    struct_ser.serialize_field("position", v)?;
                }
                participate_response::Msg::ContributeNow(v) => {
                    struct_ser.serialize_field("contributeNow", v)?;
                }
                participate_response::Msg::Confirm(v) => {
                    struct_ser.serialize_field("confirm", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ParticipateResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position",
            "contribute_now",
            "contributeNow",
            "confirm",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Position,
            ContributeNow,
            Confirm,
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
                            "contributeNow" | "contribute_now" => Ok(GeneratedField::ContributeNow),
                            "confirm" => Ok(GeneratedField::Confirm),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ParticipateResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.tools.summoning.v1.ParticipateResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ParticipateResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut msg__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Position => {
                            if msg__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            msg__ = map_.next_value::<::std::option::Option<_>>()?.map(participate_response::Msg::Position)
;
                        }
                        GeneratedField::ContributeNow => {
                            if msg__.is_some() {
                                return Err(serde::de::Error::duplicate_field("contributeNow"));
                            }
                            msg__ = map_.next_value::<::std::option::Option<_>>()?.map(participate_response::Msg::ContributeNow)
;
                        }
                        GeneratedField::Confirm => {
                            if msg__.is_some() {
                                return Err(serde::de::Error::duplicate_field("confirm"));
                            }
                            msg__ = map_.next_value::<::std::option::Option<_>>()?.map(participate_response::Msg::Confirm)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ParticipateResponse {
                    msg: msg__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.tools.summoning.v1.ParticipateResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for participate_response::Confirm {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.slot != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.tools.summoning.v1.ParticipateResponse.Confirm", len)?;
        if self.slot != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("slot", ToString::to_string(&self.slot).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for participate_response::Confirm {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "slot",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Slot,
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
                            "slot" => Ok(GeneratedField::Slot),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = participate_response::Confirm;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.tools.summoning.v1.ParticipateResponse.Confirm")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<participate_response::Confirm, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut slot__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Slot => {
                            if slot__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slot"));
                            }
                            slot__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(participate_response::Confirm {
                    slot: slot__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.tools.summoning.v1.ParticipateResponse.Confirm", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for participate_response::ContributeNow {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.parent.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.tools.summoning.v1.ParticipateResponse.ContributeNow", len)?;
        if let Some(v) = self.parent.as_ref() {
            struct_ser.serialize_field("parent", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for participate_response::ContributeNow {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parent",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Parent,
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
                            "parent" => Ok(GeneratedField::Parent),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = participate_response::ContributeNow;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.tools.summoning.v1.ParticipateResponse.ContributeNow")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<participate_response::ContributeNow, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut parent__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Parent => {
                            if parent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parent"));
                            }
                            parent__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(participate_response::ContributeNow {
                    parent: parent__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.tools.summoning.v1.ParticipateResponse.ContributeNow", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for participate_response::Position {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.position != 0 {
            len += 1;
        }
        if self.connected_participants != 0 {
            len += 1;
        }
        if self.last_slot_bid.is_some() {
            len += 1;
        }
        if self.your_bid.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.tools.summoning.v1.ParticipateResponse.Position", len)?;
        if self.position != 0 {
            struct_ser.serialize_field("position", &self.position)?;
        }
        if self.connected_participants != 0 {
            struct_ser.serialize_field("connectedParticipants", &self.connected_participants)?;
        }
        if let Some(v) = self.last_slot_bid.as_ref() {
            struct_ser.serialize_field("lastSlotBid", v)?;
        }
        if let Some(v) = self.your_bid.as_ref() {
            struct_ser.serialize_field("yourBid", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for participate_response::Position {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "position",
            "connected_participants",
            "connectedParticipants",
            "last_slot_bid",
            "lastSlotBid",
            "your_bid",
            "yourBid",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Position,
            ConnectedParticipants,
            LastSlotBid,
            YourBid,
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
                            "connectedParticipants" | "connected_participants" => Ok(GeneratedField::ConnectedParticipants),
                            "lastSlotBid" | "last_slot_bid" => Ok(GeneratedField::LastSlotBid),
                            "yourBid" | "your_bid" => Ok(GeneratedField::YourBid),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = participate_response::Position;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.tools.summoning.v1.ParticipateResponse.Position")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<participate_response::Position, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut position__ = None;
                let mut connected_participants__ = None;
                let mut last_slot_bid__ = None;
                let mut your_bid__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ConnectedParticipants => {
                            if connected_participants__.is_some() {
                                return Err(serde::de::Error::duplicate_field("connectedParticipants"));
                            }
                            connected_participants__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LastSlotBid => {
                            if last_slot_bid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastSlotBid"));
                            }
                            last_slot_bid__ = map_.next_value()?;
                        }
                        GeneratedField::YourBid => {
                            if your_bid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("yourBid"));
                            }
                            your_bid__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(participate_response::Position {
                    position: position__.unwrap_or_default(),
                    connected_participants: connected_participants__.unwrap_or_default(),
                    last_slot_bid: last_slot_bid__,
                    your_bid: your_bid__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.tools.summoning.v1.ParticipateResponse.Position", FIELDS, GeneratedVisitor)
    }
}
