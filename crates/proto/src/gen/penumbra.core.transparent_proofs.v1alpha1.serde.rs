// @generated
impl serde::Serialize for SwapClaimProof {
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
        if self.swap_commitment_proof.is_some() {
            len += 1;
        }
        if !self.nk.is_empty() {
            len += 1;
        }
        if self.lambda_1_i.is_some() {
            len += 1;
        }
        if self.lambda_2_i.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transparent_proofs.v1alpha1.SwapClaimProof", len)?;
        if let Some(v) = self.swap_plaintext.as_ref() {
            struct_ser.serialize_field("swapPlaintext", v)?;
        }
        if let Some(v) = self.swap_commitment_proof.as_ref() {
            struct_ser.serialize_field("swapCommitmentProof", v)?;
        }
        if !self.nk.is_empty() {
            struct_ser.serialize_field("nk", pbjson::private::base64::encode(&self.nk).as_str())?;
        }
        if let Some(v) = self.lambda_1_i.as_ref() {
            struct_ser.serialize_field("lambda1I", v)?;
        }
        if let Some(v) = self.lambda_2_i.as_ref() {
            struct_ser.serialize_field("lambda2I", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapClaimProof {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_plaintext",
            "swapPlaintext",
            "swap_commitment_proof",
            "swapCommitmentProof",
            "nk",
            "lambda_1_i",
            "lambda1I",
            "lambda_2_i",
            "lambda2I",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapPlaintext,
            SwapCommitmentProof,
            Nk,
            Lambda1I,
            Lambda2I,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "swapCommitmentProof" | "swap_commitment_proof" => Ok(GeneratedField::SwapCommitmentProof),
                            "nk" => Ok(GeneratedField::Nk),
                            "lambda1I" | "lambda_1_i" => Ok(GeneratedField::Lambda1I),
                            "lambda2I" | "lambda_2_i" => Ok(GeneratedField::Lambda2I),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapClaimProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transparent_proofs.v1alpha1.SwapClaimProof")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapClaimProof, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_plaintext__ = None;
                let mut swap_commitment_proof__ = None;
                let mut nk__ = None;
                let mut lambda_1_i__ = None;
                let mut lambda_2_i__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SwapPlaintext => {
                            if swap_plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapPlaintext"));
                            }
                            swap_plaintext__ = map.next_value()?;
                        }
                        GeneratedField::SwapCommitmentProof => {
                            if swap_commitment_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapCommitmentProof"));
                            }
                            swap_commitment_proof__ = map.next_value()?;
                        }
                        GeneratedField::Nk => {
                            if nk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nk"));
                            }
                            nk__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Lambda1I => {
                            if lambda_1_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lambda1I"));
                            }
                            lambda_1_i__ = map.next_value()?;
                        }
                        GeneratedField::Lambda2I => {
                            if lambda_2_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lambda2I"));
                            }
                            lambda_2_i__ = map.next_value()?;
                        }
                    }
                }
                Ok(SwapClaimProof {
                    swap_plaintext: swap_plaintext__,
                    swap_commitment_proof: swap_commitment_proof__,
                    nk: nk__.unwrap_or_default(),
                    lambda_1_i: lambda_1_i__,
                    lambda_2_i: lambda_2_i__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transparent_proofs.v1alpha1.SwapClaimProof", FIELDS, GeneratedVisitor)
    }
}
