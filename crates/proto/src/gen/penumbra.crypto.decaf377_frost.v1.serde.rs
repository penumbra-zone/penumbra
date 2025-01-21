impl serde::Serialize for DkgRound1Package {
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
        if !self.proof_of_knowledge.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.decaf377_frost.v1.DKGRound1Package", len)?;
        if let Some(v) = self.commitment.as_ref() {
            struct_ser.serialize_field("commitment", v)?;
        }
        if !self.proof_of_knowledge.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proofOfKnowledge", pbjson::private::base64::encode(&self.proof_of_knowledge).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DkgRound1Package {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commitment",
            "proof_of_knowledge",
            "proofOfKnowledge",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commitment,
            ProofOfKnowledge,
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
                            "proofOfKnowledge" | "proof_of_knowledge" => Ok(GeneratedField::ProofOfKnowledge),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DkgRound1Package;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.decaf377_frost.v1.DKGRound1Package")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DkgRound1Package, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commitment__ = None;
                let mut proof_of_knowledge__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = map_.next_value()?;
                        }
                        GeneratedField::ProofOfKnowledge => {
                            if proof_of_knowledge__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofOfKnowledge"));
                            }
                            proof_of_knowledge__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DkgRound1Package {
                    commitment: commitment__,
                    proof_of_knowledge: proof_of_knowledge__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.decaf377_frost.v1.DKGRound1Package", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DkgRound2Package {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.signing_share.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.decaf377_frost.v1.DKGRound2Package", len)?;
        if let Some(v) = self.signing_share.as_ref() {
            struct_ser.serialize_field("signingShare", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DkgRound2Package {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "signing_share",
            "signingShare",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SigningShare,
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
                            "signingShare" | "signing_share" => Ok(GeneratedField::SigningShare),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DkgRound2Package;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.decaf377_frost.v1.DKGRound2Package")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DkgRound2Package, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut signing_share__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SigningShare => {
                            if signing_share__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signingShare"));
                            }
                            signing_share__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DkgRound2Package {
                    signing_share: signing_share__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.decaf377_frost.v1.DKGRound2Package", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NonceCommitment {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.element.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.decaf377_frost.v1.NonceCommitment", len)?;
        if !self.element.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("element", pbjson::private::base64::encode(&self.element).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NonceCommitment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "element",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Element,
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
                            "element" => Ok(GeneratedField::Element),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NonceCommitment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.decaf377_frost.v1.NonceCommitment")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NonceCommitment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut element__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Element => {
                            if element__.is_some() {
                                return Err(serde::de::Error::duplicate_field("element"));
                            }
                            element__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NonceCommitment {
                    element: element__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.decaf377_frost.v1.NonceCommitment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignatureShare {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.scalar.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.decaf377_frost.v1.SignatureShare", len)?;
        if !self.scalar.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("scalar", pbjson::private::base64::encode(&self.scalar).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignatureShare {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "scalar",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Scalar,
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
                            "scalar" => Ok(GeneratedField::Scalar),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignatureShare;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.decaf377_frost.v1.SignatureShare")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SignatureShare, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut scalar__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Scalar => {
                            if scalar__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scalar"));
                            }
                            scalar__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SignatureShare {
                    scalar: scalar__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.decaf377_frost.v1.SignatureShare", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SigningCommitments {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.hiding.is_some() {
            len += 1;
        }
        if self.binding.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.decaf377_frost.v1.SigningCommitments", len)?;
        if let Some(v) = self.hiding.as_ref() {
            struct_ser.serialize_field("hiding", v)?;
        }
        if let Some(v) = self.binding.as_ref() {
            struct_ser.serialize_field("binding", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SigningCommitments {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "hiding",
            "binding",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Hiding,
            Binding,
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
                            "hiding" => Ok(GeneratedField::Hiding),
                            "binding" => Ok(GeneratedField::Binding),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SigningCommitments;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.decaf377_frost.v1.SigningCommitments")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SigningCommitments, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut hiding__ = None;
                let mut binding__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Hiding => {
                            if hiding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hiding"));
                            }
                            hiding__ = map_.next_value()?;
                        }
                        GeneratedField::Binding => {
                            if binding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("binding"));
                            }
                            binding__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SigningCommitments {
                    hiding: hiding__,
                    binding: binding__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.decaf377_frost.v1.SigningCommitments", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SigningShare {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.scalar.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.decaf377_frost.v1.SigningShare", len)?;
        if !self.scalar.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("scalar", pbjson::private::base64::encode(&self.scalar).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SigningShare {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "scalar",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Scalar,
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
                            "scalar" => Ok(GeneratedField::Scalar),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SigningShare;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.decaf377_frost.v1.SigningShare")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SigningShare, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut scalar__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Scalar => {
                            if scalar__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scalar"));
                            }
                            scalar__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SigningShare {
                    scalar: scalar__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.decaf377_frost.v1.SigningShare", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VerifiableSecretSharingCommitment {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.elements.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.decaf377_frost.v1.VerifiableSecretSharingCommitment", len)?;
        if !self.elements.is_empty() {
            struct_ser.serialize_field("elements", &self.elements.iter().map(pbjson::private::base64::encode).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VerifiableSecretSharingCommitment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "elements",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Elements,
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
                            "elements" => Ok(GeneratedField::Elements),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VerifiableSecretSharingCommitment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.decaf377_frost.v1.VerifiableSecretSharingCommitment")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<VerifiableSecretSharingCommitment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut elements__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Elements => {
                            if elements__.is_some() {
                                return Err(serde::de::Error::duplicate_field("elements"));
                            }
                            elements__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(VerifiableSecretSharingCommitment {
                    elements: elements__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.decaf377_frost.v1.VerifiableSecretSharingCommitment", FIELDS, GeneratedVisitor)
    }
}
