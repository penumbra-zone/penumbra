impl serde::Serialize for AccountGroupInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.full_viewing_key.is_some() {
            len += 1;
        }
        if !self.participants.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.AccountGroupInfo", len)?;
        if let Some(v) = self.full_viewing_key.as_ref() {
            struct_ser.serialize_field("fullViewingKey", v)?;
        }
        if !self.participants.is_empty() {
            struct_ser.serialize_field("participants", &self.participants)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AccountGroupInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "full_viewing_key",
            "fullViewingKey",
            "participants",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FullViewingKey,
            Participants,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "fullViewingKey" | "full_viewing_key" => Ok(GeneratedField::FullViewingKey),
                            "participants" => Ok(GeneratedField::Participants),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AccountGroupInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.AccountGroupInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AccountGroupInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut full_viewing_key__ = None;
                let mut participants__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FullViewingKey => {
                            if full_viewing_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullViewingKey"));
                            }
                            full_viewing_key__ = map.next_value()?;
                        }
                        GeneratedField::Participants => {
                            if participants__.is_some() {
                                return Err(serde::de::Error::duplicate_field("participants"));
                            }
                            participants__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AccountGroupInfo {
                    full_viewing_key: full_viewing_key__,
                    participants: participants__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.AccountGroupInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthorizeCommitment {
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
        if self.signer.is_some() {
            len += 1;
        }
        if self.signature.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.AuthorizeCommitment", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.signer.as_ref() {
            struct_ser.serialize_field("signer", v)?;
        }
        if let Some(v) = self.signature.as_ref() {
            struct_ser.serialize_field("signature", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizeCommitment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body",
            "signer",
            "signature",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Body,
            Signer,
            Signature,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "signer" => Ok(GeneratedField::Signer),
                            "signature" => Ok(GeneratedField::Signature),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizeCommitment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.AuthorizeCommitment")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AuthorizeCommitment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut signer__ = None;
                let mut signature__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = map.next_value()?;
                        }
                        GeneratedField::Signature => {
                            if signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signature"));
                            }
                            signature__ = map.next_value()?;
                        }
                    }
                }
                Ok(AuthorizeCommitment {
                    body: body__,
                    signer: signer__,
                    signature: signature__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.AuthorizeCommitment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for authorize_commitment::Body {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.ceremony_index.is_some() {
            len += 1;
        }
        if !self.commitments.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.AuthorizeCommitment.Body", len)?;
        if let Some(v) = self.ceremony_index.as_ref() {
            struct_ser.serialize_field("ceremonyIndex", v)?;
        }
        if !self.commitments.is_empty() {
            struct_ser.serialize_field("commitments", &self.commitments)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for authorize_commitment::Body {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ceremony_index",
            "ceremonyIndex",
            "commitments",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CeremonyIndex,
            Commitments,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "ceremonyIndex" | "ceremony_index" => Ok(GeneratedField::CeremonyIndex),
                            "commitments" => Ok(GeneratedField::Commitments),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = authorize_commitment::Body;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.AuthorizeCommitment.Body")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<authorize_commitment::Body, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ceremony_index__ = None;
                let mut commitments__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CeremonyIndex => {
                            if ceremony_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ceremonyIndex"));
                            }
                            ceremony_index__ = map.next_value()?;
                        }
                        GeneratedField::Commitments => {
                            if commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitments"));
                            }
                            commitments__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(authorize_commitment::Body {
                    ceremony_index: ceremony_index__,
                    commitments: commitments__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.AuthorizeCommitment.Body", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthorizeShare {
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
        if self.signer.is_some() {
            len += 1;
        }
        if self.signature.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.AuthorizeShare", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.signer.as_ref() {
            struct_ser.serialize_field("signer", v)?;
        }
        if let Some(v) = self.signature.as_ref() {
            struct_ser.serialize_field("signature", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizeShare {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body",
            "signer",
            "signature",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Body,
            Signer,
            Signature,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "signer" => Ok(GeneratedField::Signer),
                            "signature" => Ok(GeneratedField::Signature),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizeShare;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.AuthorizeShare")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AuthorizeShare, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut signer__ = None;
                let mut signature__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map.next_value()?;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = map.next_value()?;
                        }
                        GeneratedField::Signature => {
                            if signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signature"));
                            }
                            signature__ = map.next_value()?;
                        }
                    }
                }
                Ok(AuthorizeShare {
                    body: body__,
                    signer: signer__,
                    signature: signature__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.AuthorizeShare", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for authorize_share::Body {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.ceremony_index.is_some() {
            len += 1;
        }
        if !self.commitments.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.AuthorizeShare.Body", len)?;
        if let Some(v) = self.ceremony_index.as_ref() {
            struct_ser.serialize_field("ceremonyIndex", v)?;
        }
        if !self.commitments.is_empty() {
            struct_ser.serialize_field("commitments", &self.commitments)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for authorize_share::Body {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ceremony_index",
            "ceremonyIndex",
            "commitments",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CeremonyIndex,
            Commitments,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "ceremonyIndex" | "ceremony_index" => Ok(GeneratedField::CeremonyIndex),
                            "commitments" => Ok(GeneratedField::Commitments),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = authorize_share::Body;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.AuthorizeShare.Body")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<authorize_share::Body, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ceremony_index__ = None;
                let mut commitments__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::CeremonyIndex => {
                            if ceremony_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ceremonyIndex"));
                            }
                            ceremony_index__ = map.next_value()?;
                        }
                        GeneratedField::Commitments => {
                            if commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitments"));
                            }
                            commitments__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(authorize_share::Body {
                    ceremony_index: ceremony_index__,
                    commitments: commitments__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.AuthorizeShare.Body", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CeremonyFailure {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.failure.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyFailure", len)?;
        if let Some(v) = self.failure.as_ref() {
            match v {
                ceremony_failure::Failure::Timeout(v) => {
                    struct_ser.serialize_field("timeout", v)?;
                }
                ceremony_failure::Failure::BadCommitment(v) => {
                    struct_ser.serialize_field("badCommitment", v)?;
                }
                ceremony_failure::Failure::BadShare(v) => {
                    struct_ser.serialize_field("badShare", v)?;
                }
                ceremony_failure::Failure::Canceled(v) => {
                    struct_ser.serialize_field("canceled", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CeremonyFailure {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timeout",
            "bad_commitment",
            "badCommitment",
            "bad_share",
            "badShare",
            "canceled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Timeout,
            BadCommitment,
            BadShare,
            Canceled,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "timeout" => Ok(GeneratedField::Timeout),
                            "badCommitment" | "bad_commitment" => Ok(GeneratedField::BadCommitment),
                            "badShare" | "bad_share" => Ok(GeneratedField::BadShare),
                            "canceled" => Ok(GeneratedField::Canceled),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CeremonyFailure;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyFailure")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CeremonyFailure, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut failure__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Timeout => {
                            if failure__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeout"));
                            }
                            failure__ = map.next_value::<::std::option::Option<_>>()?.map(ceremony_failure::Failure::Timeout)
;
                        }
                        GeneratedField::BadCommitment => {
                            if failure__.is_some() {
                                return Err(serde::de::Error::duplicate_field("badCommitment"));
                            }
                            failure__ = map.next_value::<::std::option::Option<_>>()?.map(ceremony_failure::Failure::BadCommitment)
;
                        }
                        GeneratedField::BadShare => {
                            if failure__.is_some() {
                                return Err(serde::de::Error::duplicate_field("badShare"));
                            }
                            failure__ = map.next_value::<::std::option::Option<_>>()?.map(ceremony_failure::Failure::BadShare)
;
                        }
                        GeneratedField::Canceled => {
                            if failure__.is_some() {
                                return Err(serde::de::Error::duplicate_field("canceled"));
                            }
                            failure__ = map.next_value::<::std::option::Option<_>>()?.map(ceremony_failure::Failure::Canceled)
;
                        }
                    }
                }
                Ok(CeremonyFailure {
                    failure: failure__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyFailure", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ceremony_failure::BadCommitment {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyFailure.BadCommitment", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ceremony_failure::BadCommitment {
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
            type Value = ceremony_failure::BadCommitment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyFailure.BadCommitment")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ceremony_failure::BadCommitment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ceremony_failure::BadCommitment {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyFailure.BadCommitment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ceremony_failure::BadShare {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyFailure.BadShare", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ceremony_failure::BadShare {
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
            type Value = ceremony_failure::BadShare;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyFailure.BadShare")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ceremony_failure::BadShare, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ceremony_failure::BadShare {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyFailure.BadShare", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ceremony_failure::Canceled {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyFailure.Canceled", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ceremony_failure::Canceled {
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
            type Value = ceremony_failure::Canceled;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyFailure.Canceled")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ceremony_failure::Canceled, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ceremony_failure::Canceled {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyFailure.Canceled", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ceremony_failure::Timeout {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyFailure.Timeout", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ceremony_failure::Timeout {
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
            type Value = ceremony_failure::Timeout;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyFailure.Timeout")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ceremony_failure::Timeout, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ceremony_failure::Timeout {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyFailure.Timeout", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CeremonyIndex {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.request_index.is_some() {
            len += 1;
        }
        if self.ceremony_index != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyIndex", len)?;
        if let Some(v) = self.request_index.as_ref() {
            struct_ser.serialize_field("requestIndex", v)?;
        }
        if self.ceremony_index != 0 {
            struct_ser.serialize_field("ceremonyIndex", ToString::to_string(&self.ceremony_index).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CeremonyIndex {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "request_index",
            "requestIndex",
            "ceremony_index",
            "ceremonyIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RequestIndex,
            CeremonyIndex,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "requestIndex" | "request_index" => Ok(GeneratedField::RequestIndex),
                            "ceremonyIndex" | "ceremony_index" => Ok(GeneratedField::CeremonyIndex),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CeremonyIndex;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyIndex")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CeremonyIndex, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut request_index__ = None;
                let mut ceremony_index__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RequestIndex => {
                            if request_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requestIndex"));
                            }
                            request_index__ = map.next_value()?;
                        }
                        GeneratedField::CeremonyIndex => {
                            if ceremony_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ceremonyIndex"));
                            }
                            ceremony_index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(CeremonyIndex {
                    request_index: request_index__,
                    ceremony_index: ceremony_index__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyIndex", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CeremonyState {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState", len)?;
        if let Some(v) = self.state.as_ref() {
            match v {
                ceremony_state::State::Pending(v) => {
                    struct_ser.serialize_field("pending", v)?;
                }
                ceremony_state::State::StartedRound1(v) => {
                    struct_ser.serialize_field("startedRound1", v)?;
                }
                ceremony_state::State::StartedRound2(v) => {
                    struct_ser.serialize_field("startedRound2", v)?;
                }
                ceremony_state::State::Finished(v) => {
                    struct_ser.serialize_field("finished", v)?;
                }
                ceremony_state::State::Failed(v) => {
                    struct_ser.serialize_field("failed", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CeremonyState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pending",
            "started_round_1",
            "startedRound1",
            "started_round_2",
            "startedRound2",
            "finished",
            "failed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Pending,
            StartedRound1,
            StartedRound2,
            Finished,
            Failed,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "pending" => Ok(GeneratedField::Pending),
                            "startedRound1" | "started_round_1" => Ok(GeneratedField::StartedRound1),
                            "startedRound2" | "started_round_2" => Ok(GeneratedField::StartedRound2),
                            "finished" => Ok(GeneratedField::Finished),
                            "failed" => Ok(GeneratedField::Failed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CeremonyState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CeremonyState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Pending => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pending"));
                            }
                            state__ = map.next_value::<::std::option::Option<_>>()?.map(ceremony_state::State::Pending)
;
                        }
                        GeneratedField::StartedRound1 => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startedRound1"));
                            }
                            state__ = map.next_value::<::std::option::Option<_>>()?.map(ceremony_state::State::StartedRound1)
;
                        }
                        GeneratedField::StartedRound2 => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startedRound2"));
                            }
                            state__ = map.next_value::<::std::option::Option<_>>()?.map(ceremony_state::State::StartedRound2)
;
                        }
                        GeneratedField::Finished => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finished"));
                            }
                            state__ = map.next_value::<::std::option::Option<_>>()?.map(ceremony_state::State::Finished)
;
                        }
                        GeneratedField::Failed => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("failed"));
                            }
                            state__ = map.next_value::<::std::option::Option<_>>()?.map(ceremony_state::State::Failed)
;
                        }
                    }
                }
                Ok(CeremonyState {
                    state: state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ceremony_state::Failed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.committee.is_some() {
            len += 1;
        }
        if !self.commitments.is_empty() {
            len += 1;
        }
        if !self.shares.is_empty() {
            len += 1;
        }
        if self.failure.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState.Failed", len)?;
        if let Some(v) = self.committee.as_ref() {
            struct_ser.serialize_field("committee", v)?;
        }
        if !self.commitments.is_empty() {
            struct_ser.serialize_field("commitments", &self.commitments)?;
        }
        if !self.shares.is_empty() {
            struct_ser.serialize_field("shares", &self.shares)?;
        }
        if let Some(v) = self.failure.as_ref() {
            struct_ser.serialize_field("failure", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ceremony_state::Failed {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "committee",
            "commitments",
            "shares",
            "failure",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Committee,
            Commitments,
            Shares,
            Failure,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "committee" => Ok(GeneratedField::Committee),
                            "commitments" => Ok(GeneratedField::Commitments),
                            "shares" => Ok(GeneratedField::Shares),
                            "failure" => Ok(GeneratedField::Failure),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ceremony_state::Failed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyState.Failed")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ceremony_state::Failed, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut committee__ = None;
                let mut commitments__ = None;
                let mut shares__ = None;
                let mut failure__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Committee => {
                            if committee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("committee"));
                            }
                            committee__ = map.next_value()?;
                        }
                        GeneratedField::Commitments => {
                            if commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitments"));
                            }
                            commitments__ = Some(map.next_value()?);
                        }
                        GeneratedField::Shares => {
                            if shares__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shares"));
                            }
                            shares__ = Some(map.next_value()?);
                        }
                        GeneratedField::Failure => {
                            if failure__.is_some() {
                                return Err(serde::de::Error::duplicate_field("failure"));
                            }
                            failure__ = map.next_value()?;
                        }
                    }
                }
                Ok(ceremony_state::Failed {
                    committee: committee__,
                    commitments: commitments__.unwrap_or_default(),
                    shares: shares__.unwrap_or_default(),
                    failure: failure__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState.Failed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ceremony_state::Finished {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.committee.is_some() {
            len += 1;
        }
        if !self.commitments.is_empty() {
            len += 1;
        }
        if !self.shares.is_empty() {
            len += 1;
        }
        if self.auth_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState.Finished", len)?;
        if let Some(v) = self.committee.as_ref() {
            struct_ser.serialize_field("committee", v)?;
        }
        if !self.commitments.is_empty() {
            struct_ser.serialize_field("commitments", &self.commitments)?;
        }
        if !self.shares.is_empty() {
            struct_ser.serialize_field("shares", &self.shares)?;
        }
        if let Some(v) = self.auth_data.as_ref() {
            struct_ser.serialize_field("authData", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ceremony_state::Finished {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "committee",
            "commitments",
            "shares",
            "auth_data",
            "authData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Committee,
            Commitments,
            Shares,
            AuthData,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "committee" => Ok(GeneratedField::Committee),
                            "commitments" => Ok(GeneratedField::Commitments),
                            "shares" => Ok(GeneratedField::Shares),
                            "authData" | "auth_data" => Ok(GeneratedField::AuthData),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ceremony_state::Finished;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyState.Finished")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ceremony_state::Finished, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut committee__ = None;
                let mut commitments__ = None;
                let mut shares__ = None;
                let mut auth_data__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Committee => {
                            if committee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("committee"));
                            }
                            committee__ = map.next_value()?;
                        }
                        GeneratedField::Commitments => {
                            if commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitments"));
                            }
                            commitments__ = Some(map.next_value()?);
                        }
                        GeneratedField::Shares => {
                            if shares__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shares"));
                            }
                            shares__ = Some(map.next_value()?);
                        }
                        GeneratedField::AuthData => {
                            if auth_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authData"));
                            }
                            auth_data__ = map.next_value()?;
                        }
                    }
                }
                Ok(ceremony_state::Finished {
                    committee: committee__,
                    commitments: commitments__.unwrap_or_default(),
                    shares: shares__.unwrap_or_default(),
                    auth_data: auth_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState.Finished", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ceremony_state::Pending {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState.Pending", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ceremony_state::Pending {
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
            type Value = ceremony_state::Pending;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyState.Pending")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ceremony_state::Pending, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ceremony_state::Pending {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState.Pending", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ceremony_state::StartedRound1 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.committee.is_some() {
            len += 1;
        }
        if !self.commitments.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState.StartedRound1", len)?;
        if let Some(v) = self.committee.as_ref() {
            struct_ser.serialize_field("committee", v)?;
        }
        if !self.commitments.is_empty() {
            struct_ser.serialize_field("commitments", &self.commitments)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ceremony_state::StartedRound1 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "committee",
            "commitments",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Committee,
            Commitments,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "committee" => Ok(GeneratedField::Committee),
                            "commitments" => Ok(GeneratedField::Commitments),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ceremony_state::StartedRound1;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyState.StartedRound1")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ceremony_state::StartedRound1, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut committee__ = None;
                let mut commitments__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Committee => {
                            if committee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("committee"));
                            }
                            committee__ = map.next_value()?;
                        }
                        GeneratedField::Commitments => {
                            if commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitments"));
                            }
                            commitments__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ceremony_state::StartedRound1 {
                    committee: committee__,
                    commitments: commitments__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState.StartedRound1", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ceremony_state::StartedRound2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.committee.is_some() {
            len += 1;
        }
        if !self.commitments.is_empty() {
            len += 1;
        }
        if !self.shares.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState.StartedRound2", len)?;
        if let Some(v) = self.committee.as_ref() {
            struct_ser.serialize_field("committee", v)?;
        }
        if !self.commitments.is_empty() {
            struct_ser.serialize_field("commitments", &self.commitments)?;
        }
        if !self.shares.is_empty() {
            struct_ser.serialize_field("shares", &self.shares)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ceremony_state::StartedRound2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "committee",
            "commitments",
            "shares",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Committee,
            Commitments,
            Shares,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "committee" => Ok(GeneratedField::Committee),
                            "commitments" => Ok(GeneratedField::Commitments),
                            "shares" => Ok(GeneratedField::Shares),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ceremony_state::StartedRound2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.CeremonyState.StartedRound2")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ceremony_state::StartedRound2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut committee__ = None;
                let mut commitments__ = None;
                let mut shares__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Committee => {
                            if committee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("committee"));
                            }
                            committee__ = map.next_value()?;
                        }
                        GeneratedField::Commitments => {
                            if commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitments"));
                            }
                            commitments__ = Some(map.next_value()?);
                        }
                        GeneratedField::Shares => {
                            if shares__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shares"));
                            }
                            shares__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ceremony_state::StartedRound2 {
                    committee: committee__,
                    commitments: commitments__.unwrap_or_default(),
                    shares: shares__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.CeremonyState.StartedRound2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Committee {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.ceremony.is_some() {
            len += 1;
        }
        if !self.participants.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.Committee", len)?;
        if let Some(v) = self.ceremony.as_ref() {
            struct_ser.serialize_field("ceremony", v)?;
        }
        if !self.participants.is_empty() {
            struct_ser.serialize_field("participants", &self.participants)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Committee {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ceremony",
            "participants",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Ceremony,
            Participants,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "ceremony" => Ok(GeneratedField::Ceremony),
                            "participants" => Ok(GeneratedField::Participants),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Committee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.Committee")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Committee, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ceremony__ = None;
                let mut participants__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Ceremony => {
                            if ceremony__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ceremony"));
                            }
                            ceremony__ = map.next_value()?;
                        }
                        GeneratedField::Participants => {
                            if participants__.is_some() {
                                return Err(serde::de::Error::duplicate_field("participants"));
                            }
                            participants__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Committee {
                    ceremony: ceremony__,
                    participants: participants__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.Committee", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConsensusKey {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.ConsensusKey", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConsensusKey {
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
            type Value = ConsensusKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.ConsensusKey")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConsensusKey, V::Error>
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
                Ok(ConsensusKey {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.ConsensusKey", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DkgRound1 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.payload.is_empty() {
            len += 1;
        }
        if self.signer.is_some() {
            len += 1;
        }
        if self.signature.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.DkgRound1", len)?;
        if !self.payload.is_empty() {
            struct_ser.serialize_field("payload", pbjson::private::base64::encode(&self.payload).as_str())?;
        }
        if let Some(v) = self.signer.as_ref() {
            struct_ser.serialize_field("signer", v)?;
        }
        if let Some(v) = self.signature.as_ref() {
            struct_ser.serialize_field("signature", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DkgRound1 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payload",
            "signer",
            "signature",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Payload,
            Signer,
            Signature,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "payload" => Ok(GeneratedField::Payload),
                            "signer" => Ok(GeneratedField::Signer),
                            "signature" => Ok(GeneratedField::Signature),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DkgRound1;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.DkgRound1")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DkgRound1, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payload__ = None;
                let mut signer__ = None;
                let mut signature__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Payload => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payload"));
                            }
                            payload__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = map.next_value()?;
                        }
                        GeneratedField::Signature => {
                            if signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signature"));
                            }
                            signature__ = map.next_value()?;
                        }
                    }
                }
                Ok(DkgRound1 {
                    payload: payload__.unwrap_or_default(),
                    signer: signer__,
                    signature: signature__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.DkgRound1", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DkgRound2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.payload.is_empty() {
            len += 1;
        }
        if self.signer.is_some() {
            len += 1;
        }
        if self.signature.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.DkgRound2", len)?;
        if !self.payload.is_empty() {
            struct_ser.serialize_field("payload", pbjson::private::base64::encode(&self.payload).as_str())?;
        }
        if let Some(v) = self.signer.as_ref() {
            struct_ser.serialize_field("signer", v)?;
        }
        if let Some(v) = self.signature.as_ref() {
            struct_ser.serialize_field("signature", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DkgRound2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payload",
            "signer",
            "signature",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Payload,
            Signer,
            Signature,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "payload" => Ok(GeneratedField::Payload),
                            "signer" => Ok(GeneratedField::Signer),
                            "signature" => Ok(GeneratedField::Signature),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DkgRound2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.DkgRound2")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DkgRound2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payload__ = None;
                let mut signer__ = None;
                let mut signature__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Payload => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payload"));
                            }
                            payload__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Signer => {
                            if signer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signer"));
                            }
                            signer__ = map.next_value()?;
                        }
                        GeneratedField::Signature => {
                            if signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signature"));
                            }
                            signature__ = map.next_value()?;
                        }
                    }
                }
                Ok(DkgRound2 {
                    payload: payload__.unwrap_or_default(),
                    signer: signer__,
                    signature: signature__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.DkgRound2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DkgState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.DkgState", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DkgState {
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
            type Value = DkgState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.DkgState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DkgState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(DkgState {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.DkgState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for dkg_state::Finished {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.round_1_messages.is_empty() {
            len += 1;
        }
        if !self.round_2_messages.is_empty() {
            len += 1;
        }
        if self.account_group_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.DkgState.Finished", len)?;
        if !self.round_1_messages.is_empty() {
            struct_ser.serialize_field("round1Messages", &self.round_1_messages)?;
        }
        if !self.round_2_messages.is_empty() {
            struct_ser.serialize_field("round2Messages", &self.round_2_messages)?;
        }
        if let Some(v) = self.account_group_info.as_ref() {
            struct_ser.serialize_field("accountGroupInfo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for dkg_state::Finished {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "round_1_messages",
            "round1Messages",
            "round_2_messages",
            "round2Messages",
            "account_group_info",
            "accountGroupInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Round1Messages,
            Round2Messages,
            AccountGroupInfo,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "round1Messages" | "round_1_messages" => Ok(GeneratedField::Round1Messages),
                            "round2Messages" | "round_2_messages" => Ok(GeneratedField::Round2Messages),
                            "accountGroupInfo" | "account_group_info" => Ok(GeneratedField::AccountGroupInfo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = dkg_state::Finished;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.DkgState.Finished")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<dkg_state::Finished, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut round_1_messages__ = None;
                let mut round_2_messages__ = None;
                let mut account_group_info__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Round1Messages => {
                            if round_1_messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round1Messages"));
                            }
                            round_1_messages__ = Some(map.next_value()?);
                        }
                        GeneratedField::Round2Messages => {
                            if round_2_messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round2Messages"));
                            }
                            round_2_messages__ = Some(map.next_value()?);
                        }
                        GeneratedField::AccountGroupInfo => {
                            if account_group_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountGroupInfo"));
                            }
                            account_group_info__ = map.next_value()?;
                        }
                    }
                }
                Ok(dkg_state::Finished {
                    round_1_messages: round_1_messages__.unwrap_or_default(),
                    round_2_messages: round_2_messages__.unwrap_or_default(),
                    account_group_info: account_group_info__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.DkgState.Finished", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for dkg_state::StartedRound1 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.round_1_messages.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.DkgState.StartedRound1", len)?;
        if !self.round_1_messages.is_empty() {
            struct_ser.serialize_field("round1Messages", &self.round_1_messages)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for dkg_state::StartedRound1 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "round_1_messages",
            "round1Messages",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Round1Messages,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "round1Messages" | "round_1_messages" => Ok(GeneratedField::Round1Messages),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = dkg_state::StartedRound1;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.DkgState.StartedRound1")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<dkg_state::StartedRound1, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut round_1_messages__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Round1Messages => {
                            if round_1_messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round1Messages"));
                            }
                            round_1_messages__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(dkg_state::StartedRound1 {
                    round_1_messages: round_1_messages__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.DkgState.StartedRound1", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for dkg_state::StartedRound2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.round_1_messages.is_empty() {
            len += 1;
        }
        if !self.round_2_messages.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.DkgState.StartedRound2", len)?;
        if !self.round_1_messages.is_empty() {
            struct_ser.serialize_field("round1Messages", &self.round_1_messages)?;
        }
        if !self.round_2_messages.is_empty() {
            struct_ser.serialize_field("round2Messages", &self.round_2_messages)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for dkg_state::StartedRound2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "round_1_messages",
            "round1Messages",
            "round_2_messages",
            "round2Messages",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Round1Messages,
            Round2Messages,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "round1Messages" | "round_1_messages" => Ok(GeneratedField::Round1Messages),
                            "round2Messages" | "round_2_messages" => Ok(GeneratedField::Round2Messages),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = dkg_state::StartedRound2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.DkgState.StartedRound2")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<dkg_state::StartedRound2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut round_1_messages__ = None;
                let mut round_2_messages__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Round1Messages => {
                            if round_1_messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round1Messages"));
                            }
                            round_1_messages__ = Some(map.next_value()?);
                        }
                        GeneratedField::Round2Messages => {
                            if round_2_messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("round2Messages"));
                            }
                            round_2_messages__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(dkg_state::StartedRound2 {
                    round_1_messages: round_1_messages__.unwrap_or_default(),
                    round_2_messages: round_2_messages__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.DkgState.StartedRound2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FrostCommitment {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.payload.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.FrostCommitment", len)?;
        if !self.payload.is_empty() {
            struct_ser.serialize_field("payload", pbjson::private::base64::encode(&self.payload).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FrostCommitment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payload",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = FrostCommitment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.FrostCommitment")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FrostCommitment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payload__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Payload => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payload"));
                            }
                            payload__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(FrostCommitment {
                    payload: payload__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.FrostCommitment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FrostSignatureShare {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.payload.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.FrostSignatureShare", len)?;
        if !self.payload.is_empty() {
            struct_ser.serialize_field("payload", pbjson::private::base64::encode(&self.payload).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FrostSignatureShare {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payload",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = FrostSignatureShare;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.FrostSignatureShare")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FrostSignatureShare, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payload__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Payload => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payload"));
                            }
                            payload__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(FrostSignatureShare {
                    payload: payload__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.FrostSignatureShare", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenesisData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.operators.is_empty() {
            len += 1;
        }
        if self.threshold != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.GenesisData", len)?;
        if !self.operators.is_empty() {
            struct_ser.serialize_field("operators", &self.operators)?;
        }
        if self.threshold != 0 {
            struct_ser.serialize_field("threshold", &self.threshold)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenesisData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "operators",
            "threshold",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Operators,
            Threshold,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "operators" => Ok(GeneratedField::Operators),
                            "threshold" => Ok(GeneratedField::Threshold),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenesisData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.GenesisData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut operators__ = None;
                let mut threshold__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Operators => {
                            if operators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("operators"));
                            }
                            operators__ = Some(map.next_value()?);
                        }
                        GeneratedField::Threshold => {
                            if threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("threshold"));
                            }
                            threshold__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(GenesisData {
                    operators: operators__.unwrap_or_default(),
                    threshold: threshold__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.GenesisData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InfoRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.version.is_empty() {
            len += 1;
        }
        if self.block_version != 0 {
            len += 1;
        }
        if self.p2p_version != 0 {
            len += 1;
        }
        if !self.abci_version.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.InfoRequest", len)?;
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if self.block_version != 0 {
            struct_ser.serialize_field("blockVersion", ToString::to_string(&self.block_version).as_str())?;
        }
        if self.p2p_version != 0 {
            struct_ser.serialize_field("p2pVersion", ToString::to_string(&self.p2p_version).as_str())?;
        }
        if !self.abci_version.is_empty() {
            struct_ser.serialize_field("abciVersion", &self.abci_version)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InfoRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "version",
            "block_version",
            "blockVersion",
            "p2p_version",
            "p2pVersion",
            "abci_version",
            "abciVersion",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Version,
            BlockVersion,
            P2pVersion,
            AbciVersion,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "version" => Ok(GeneratedField::Version),
                            "blockVersion" | "block_version" => Ok(GeneratedField::BlockVersion),
                            "p2pVersion" | "p2p_version" => Ok(GeneratedField::P2pVersion),
                            "abciVersion" | "abci_version" => Ok(GeneratedField::AbciVersion),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InfoRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.InfoRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<InfoRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut version__ = None;
                let mut block_version__ = None;
                let mut p2p_version__ = None;
                let mut abci_version__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
                        GeneratedField::BlockVersion => {
                            if block_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockVersion"));
                            }
                            block_version__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::P2pVersion => {
                            if p2p_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("p2pVersion"));
                            }
                            p2p_version__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AbciVersion => {
                            if abci_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("abciVersion"));
                            }
                            abci_version__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(InfoRequest {
                    version: version__.unwrap_or_default(),
                    block_version: block_version__.unwrap_or_default(),
                    p2p_version: p2p_version__.unwrap_or_default(),
                    abci_version: abci_version__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.InfoRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InfoResponse {
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
        if !self.version.is_empty() {
            len += 1;
        }
        if self.app_version != 0 {
            len += 1;
        }
        if self.last_block_height != 0 {
            len += 1;
        }
        if !self.last_block_app_hash.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.InfoResponse", len)?;
        if !self.data.is_empty() {
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&self.data).as_str())?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if self.app_version != 0 {
            struct_ser.serialize_field("appVersion", ToString::to_string(&self.app_version).as_str())?;
        }
        if self.last_block_height != 0 {
            struct_ser.serialize_field("lastBlockHeight", ToString::to_string(&self.last_block_height).as_str())?;
        }
        if !self.last_block_app_hash.is_empty() {
            struct_ser.serialize_field("lastBlockAppHash", pbjson::private::base64::encode(&self.last_block_app_hash).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InfoResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data",
            "version",
            "app_version",
            "appVersion",
            "last_block_height",
            "lastBlockHeight",
            "last_block_app_hash",
            "lastBlockAppHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Data,
            Version,
            AppVersion,
            LastBlockHeight,
            LastBlockAppHash,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "version" => Ok(GeneratedField::Version),
                            "appVersion" | "app_version" => Ok(GeneratedField::AppVersion),
                            "lastBlockHeight" | "last_block_height" => Ok(GeneratedField::LastBlockHeight),
                            "lastBlockAppHash" | "last_block_app_hash" => Ok(GeneratedField::LastBlockAppHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InfoResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.InfoResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<InfoResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data__ = None;
                let mut version__ = None;
                let mut app_version__ = None;
                let mut last_block_height__ = None;
                let mut last_block_app_hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppVersion => {
                            if app_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appVersion"));
                            }
                            app_version__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LastBlockHeight => {
                            if last_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastBlockHeight"));
                            }
                            last_block_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LastBlockAppHash => {
                            if last_block_app_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastBlockAppHash"));
                            }
                            last_block_app_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(InfoResponse {
                    data: data__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                    app_version: app_version__.unwrap_or_default(),
                    last_block_height: last_block_height__.unwrap_or_default(),
                    last_block_app_hash: last_block_app_hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.InfoResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NarsilPacket {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.packet.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.NarsilPacket", len)?;
        if let Some(v) = self.packet.as_ref() {
            match v {
                narsil_packet::Packet::AuthorizeRequest(v) => {
                    struct_ser.serialize_field("authorizeRequest", v)?;
                }
                narsil_packet::Packet::AuthorizeCommitment(v) => {
                    struct_ser.serialize_field("authorizeCommitment", v)?;
                }
                narsil_packet::Packet::AuthorizeShare(v) => {
                    struct_ser.serialize_field("authorizeShare", v)?;
                }
                narsil_packet::Packet::DkgRound1(v) => {
                    struct_ser.serialize_field("dkgRound1", v)?;
                }
                narsil_packet::Packet::DkgRound2(v) => {
                    struct_ser.serialize_field("dkgRound2", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NarsilPacket {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authorize_request",
            "authorizeRequest",
            "authorize_commitment",
            "authorizeCommitment",
            "authorize_share",
            "authorizeShare",
            "dkg_round_1",
            "dkgRound1",
            "dkg_round_2",
            "dkgRound2",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuthorizeRequest,
            AuthorizeCommitment,
            AuthorizeShare,
            DkgRound1,
            DkgRound2,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "authorizeRequest" | "authorize_request" => Ok(GeneratedField::AuthorizeRequest),
                            "authorizeCommitment" | "authorize_commitment" => Ok(GeneratedField::AuthorizeCommitment),
                            "authorizeShare" | "authorize_share" => Ok(GeneratedField::AuthorizeShare),
                            "dkgRound1" | "dkg_round_1" => Ok(GeneratedField::DkgRound1),
                            "dkgRound2" | "dkg_round_2" => Ok(GeneratedField::DkgRound2),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NarsilPacket;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.NarsilPacket")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NarsilPacket, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut packet__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AuthorizeRequest => {
                            if packet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorizeRequest"));
                            }
                            packet__ = map.next_value::<::std::option::Option<_>>()?.map(narsil_packet::Packet::AuthorizeRequest)
;
                        }
                        GeneratedField::AuthorizeCommitment => {
                            if packet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorizeCommitment"));
                            }
                            packet__ = map.next_value::<::std::option::Option<_>>()?.map(narsil_packet::Packet::AuthorizeCommitment)
;
                        }
                        GeneratedField::AuthorizeShare => {
                            if packet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorizeShare"));
                            }
                            packet__ = map.next_value::<::std::option::Option<_>>()?.map(narsil_packet::Packet::AuthorizeShare)
;
                        }
                        GeneratedField::DkgRound1 => {
                            if packet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dkgRound1"));
                            }
                            packet__ = map.next_value::<::std::option::Option<_>>()?.map(narsil_packet::Packet::DkgRound1)
;
                        }
                        GeneratedField::DkgRound2 => {
                            if packet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dkgRound2"));
                            }
                            packet__ = map.next_value::<::std::option::Option<_>>()?.map(narsil_packet::Packet::DkgRound2)
;
                        }
                    }
                }
                Ok(NarsilPacket {
                    packet: packet__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.NarsilPacket", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RequestIndex {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.effect_hash.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.RequestIndex", len)?;
        if let Some(v) = self.effect_hash.as_ref() {
            struct_ser.serialize_field("effectHash", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RequestIndex {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "effect_hash",
            "effectHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EffectHash,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "effectHash" | "effect_hash" => Ok(GeneratedField::EffectHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RequestIndex;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.RequestIndex")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RequestIndex, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut effect_hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::EffectHash => {
                            if effect_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effectHash"));
                            }
                            effect_hash__ = map.next_value()?;
                        }
                    }
                }
                Ok(RequestIndex {
                    effect_hash: effect_hash__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.RequestIndex", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ShardDescription {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.identity_key.is_some() {
            len += 1;
        }
        if self.message_key.is_some() {
            len += 1;
        }
        if self.consensus_key.is_some() {
            len += 1;
        }
        if !self.label.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.ShardDescription", len)?;
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        if let Some(v) = self.message_key.as_ref() {
            struct_ser.serialize_field("messageKey", v)?;
        }
        if let Some(v) = self.consensus_key.as_ref() {
            struct_ser.serialize_field("consensusKey", v)?;
        }
        if !self.label.is_empty() {
            struct_ser.serialize_field("label", &self.label)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ShardDescription {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "identity_key",
            "identityKey",
            "message_key",
            "messageKey",
            "consensus_key",
            "consensusKey",
            "label",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IdentityKey,
            MessageKey,
            ConsensusKey,
            Label,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "identityKey" | "identity_key" => Ok(GeneratedField::IdentityKey),
                            "messageKey" | "message_key" => Ok(GeneratedField::MessageKey),
                            "consensusKey" | "consensus_key" => Ok(GeneratedField::ConsensusKey),
                            "label" => Ok(GeneratedField::Label),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ShardDescription;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.ShardDescription")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ShardDescription, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut identity_key__ = None;
                let mut message_key__ = None;
                let mut consensus_key__ = None;
                let mut label__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map.next_value()?;
                        }
                        GeneratedField::MessageKey => {
                            if message_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("messageKey"));
                            }
                            message_key__ = map.next_value()?;
                        }
                        GeneratedField::ConsensusKey => {
                            if consensus_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensusKey"));
                            }
                            consensus_key__ = map.next_value()?;
                        }
                        GeneratedField::Label => {
                            if label__.is_some() {
                                return Err(serde::de::Error::duplicate_field("label"));
                            }
                            label__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ShardDescription {
                    identity_key: identity_key__,
                    message_key: message_key__,
                    consensus_key: consensus_key__,
                    label: label__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.ShardDescription", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ShardIdentityKey {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.ShardIdentityKey", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ShardIdentityKey {
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
            type Value = ShardIdentityKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.ShardIdentityKey")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ShardIdentityKey, V::Error>
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
                Ok(ShardIdentityKey {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.ShardIdentityKey", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ShardInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.index != 0 {
            len += 1;
        }
        if self.shard_verification_key.is_some() {
            len += 1;
        }
        if self.identity_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.ShardInfo", len)?;
        if self.index != 0 {
            struct_ser.serialize_field("index", &self.index)?;
        }
        if let Some(v) = self.shard_verification_key.as_ref() {
            struct_ser.serialize_field("shardVerificationKey", v)?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ShardInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "index",
            "shard_verification_key",
            "shardVerificationKey",
            "identity_key",
            "identityKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Index,
            ShardVerificationKey,
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
                            "index" => Ok(GeneratedField::Index),
                            "shardVerificationKey" | "shard_verification_key" => Ok(GeneratedField::ShardVerificationKey),
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
            type Value = ShardInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.ShardInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ShardInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut index__ = None;
                let mut shard_verification_key__ = None;
                let mut identity_key__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ShardVerificationKey => {
                            if shard_verification_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shardVerificationKey"));
                            }
                            shard_verification_key__ = map.next_value()?;
                        }
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map.next_value()?;
                        }
                    }
                }
                Ok(ShardInfo {
                    index: index__.unwrap_or_default(),
                    shard_verification_key: shard_verification_key__,
                    identity_key: identity_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.ShardInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ShardKey {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.ShardKey", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ShardKey {
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
            type Value = ShardKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.ShardKey")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ShardKey, V::Error>
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
                Ok(ShardKey {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.ShardKey", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ShardMessageKey {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.ShardMessageKey", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ShardMessageKey {
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
            type Value = ShardMessageKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.ShardMessageKey")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ShardMessageKey, V::Error>
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
                Ok(ShardMessageKey {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.ShardMessageKey", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ShardMessageSignature {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.ShardMessageSignature", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ShardMessageSignature {
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
            type Value = ShardMessageSignature;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.ShardMessageSignature")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ShardMessageSignature, V::Error>
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
                Ok(ShardMessageSignature {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.ShardMessageSignature", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ShardOperator {
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
        if !self.sig.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.narsil.ledger.v1alpha1.ShardOperator", len)?;
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if !self.sig.is_empty() {
            struct_ser.serialize_field("sig", pbjson::private::base64::encode(&self.sig).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ShardOperator {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "description",
            "sig",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Description,
            Sig,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "sig" => Ok(GeneratedField::Sig),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ShardOperator;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.narsil.ledger.v1alpha1.ShardOperator")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ShardOperator, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut description__ = None;
                let mut sig__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map.next_value()?;
                        }
                        GeneratedField::Sig => {
                            if sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sig"));
                            }
                            sig__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ShardOperator {
                    description: description__,
                    sig: sig__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.narsil.ledger.v1alpha1.ShardOperator", FIELDS, GeneratedVisitor)
    }
}
