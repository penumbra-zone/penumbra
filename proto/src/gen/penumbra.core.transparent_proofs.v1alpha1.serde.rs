impl serde::Serialize for OutputProof {
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
        if !self.v_blinding.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transparent_proofs.v1alpha1.OutputProof", len)?;
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if !self.v_blinding.is_empty() {
            struct_ser.serialize_field("vBlinding", pbjson::private::base64::encode(&self.v_blinding).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputProof {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note",
            "v_blinding",
            "vBlinding",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Note,
            VBlinding,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "vBlinding" | "v_blinding" => Ok(GeneratedField::VBlinding),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transparent_proofs.v1alpha1.OutputProof")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<OutputProof, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note__ = None;
                let mut v_blinding__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map.next_value()?;
                        }
                        GeneratedField::VBlinding => {
                            if v_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vBlinding"));
                            }
                            v_blinding__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(OutputProof {
                    note: note__,
                    v_blinding: v_blinding__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transparent_proofs.v1alpha1.OutputProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendProof {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_commitment_proof.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        if !self.v_blinding.is_empty() {
            len += 1;
        }
        if !self.spend_auth_randomizer.is_empty() {
            len += 1;
        }
        if !self.ak.is_empty() {
            len += 1;
        }
        if !self.nk.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transparent_proofs.v1alpha1.SpendProof", len)?;
        if let Some(v) = self.note_commitment_proof.as_ref() {
            struct_ser.serialize_field("noteCommitmentProof", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if !self.v_blinding.is_empty() {
            struct_ser.serialize_field("vBlinding", pbjson::private::base64::encode(&self.v_blinding).as_str())?;
        }
        if !self.spend_auth_randomizer.is_empty() {
            struct_ser.serialize_field("spendAuthRandomizer", pbjson::private::base64::encode(&self.spend_auth_randomizer).as_str())?;
        }
        if !self.ak.is_empty() {
            struct_ser.serialize_field("ak", pbjson::private::base64::encode(&self.ak).as_str())?;
        }
        if !self.nk.is_empty() {
            struct_ser.serialize_field("nk", pbjson::private::base64::encode(&self.nk).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendProof {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_commitment_proof",
            "noteCommitmentProof",
            "note",
            "v_blinding",
            "vBlinding",
            "spend_auth_randomizer",
            "spendAuthRandomizer",
            "ak",
            "nk",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteCommitmentProof,
            Note,
            VBlinding,
            SpendAuthRandomizer,
            Ak,
            Nk,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "noteCommitmentProof" | "note_commitment_proof" => Ok(GeneratedField::NoteCommitmentProof),
                            "note" => Ok(GeneratedField::Note),
                            "vBlinding" | "v_blinding" => Ok(GeneratedField::VBlinding),
                            "spendAuthRandomizer" | "spend_auth_randomizer" => Ok(GeneratedField::SpendAuthRandomizer),
                            "ak" => Ok(GeneratedField::Ak),
                            "nk" => Ok(GeneratedField::Nk),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transparent_proofs.v1alpha1.SpendProof")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SpendProof, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_commitment_proof__ = None;
                let mut note__ = None;
                let mut v_blinding__ = None;
                let mut spend_auth_randomizer__ = None;
                let mut ak__ = None;
                let mut nk__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NoteCommitmentProof => {
                            if note_commitment_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitmentProof"));
                            }
                            note_commitment_proof__ = map.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map.next_value()?;
                        }
                        GeneratedField::VBlinding => {
                            if v_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vBlinding"));
                            }
                            v_blinding__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SpendAuthRandomizer => {
                            if spend_auth_randomizer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendAuthRandomizer"));
                            }
                            spend_auth_randomizer__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Ak => {
                            if ak__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ak"));
                            }
                            ak__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Nk => {
                            if nk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nk"));
                            }
                            nk__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SpendProof {
                    note_commitment_proof: note_commitment_proof__,
                    note: note__,
                    v_blinding: v_blinding__.unwrap_or_default(),
                    spend_auth_randomizer: spend_auth_randomizer__.unwrap_or_default(),
                    ak: ak__.unwrap_or_default(),
                    nk: nk__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transparent_proofs.v1alpha1.SpendProof", FIELDS, GeneratedVisitor)
    }
}
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
        if self.lambda_1_i != 0 {
            len += 1;
        }
        if self.lambda_2_i != 0 {
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
        if self.lambda_1_i != 0 {
            struct_ser.serialize_field("lambda1I", ToString::to_string(&self.lambda_1_i).as_str())?;
        }
        if self.lambda_2_i != 0 {
            struct_ser.serialize_field("lambda2I", ToString::to_string(&self.lambda_2_i).as_str())?;
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
                            lambda_1_i__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Lambda2I => {
                            if lambda_2_i__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lambda2I"));
                            }
                            lambda_2_i__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SwapClaimProof {
                    swap_plaintext: swap_plaintext__,
                    swap_commitment_proof: swap_commitment_proof__,
                    nk: nk__.unwrap_or_default(),
                    lambda_1_i: lambda_1_i__.unwrap_or_default(),
                    lambda_2_i: lambda_2_i__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transparent_proofs.v1alpha1.SwapClaimProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapProof {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transparent_proofs.v1alpha1.SwapProof", len)?;
        if let Some(v) = self.swap_plaintext.as_ref() {
            struct_ser.serialize_field("swapPlaintext", v)?;
        }
        if !self.fee_blinding.is_empty() {
            struct_ser.serialize_field("feeBlinding", pbjson::private::base64::encode(&self.fee_blinding).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapProof {
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
            type Value = SwapProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transparent_proofs.v1alpha1.SwapProof")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapProof, V::Error>
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
                Ok(SwapProof {
                    swap_plaintext: swap_plaintext__,
                    fee_blinding: fee_blinding__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transparent_proofs.v1alpha1.SwapProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UndelegateClaimProof {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.unbonding_amount.is_some() {
            len += 1;
        }
        if !self.balance_blinding.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transparent_proofs.v1alpha1.UndelegateClaimProof", len)?;
        if let Some(v) = self.unbonding_amount.as_ref() {
            struct_ser.serialize_field("unbondingAmount", v)?;
        }
        if !self.balance_blinding.is_empty() {
            struct_ser.serialize_field("balanceBlinding", pbjson::private::base64::encode(&self.balance_blinding).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UndelegateClaimProof {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "unbonding_amount",
            "unbondingAmount",
            "balance_blinding",
            "balanceBlinding",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UnbondingAmount,
            BalanceBlinding,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "unbondingAmount" | "unbonding_amount" => Ok(GeneratedField::UnbondingAmount),
                            "balanceBlinding" | "balance_blinding" => Ok(GeneratedField::BalanceBlinding),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UndelegateClaimProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transparent_proofs.v1alpha1.UndelegateClaimProof")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<UndelegateClaimProof, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut unbonding_amount__ = None;
                let mut balance_blinding__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::UnbondingAmount => {
                            if unbonding_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingAmount"));
                            }
                            unbonding_amount__ = map.next_value()?;
                        }
                        GeneratedField::BalanceBlinding => {
                            if balance_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceBlinding"));
                            }
                            balance_blinding__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(UndelegateClaimProof {
                    unbonding_amount: unbonding_amount__,
                    balance_blinding: balance_blinding__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transparent_proofs.v1alpha1.UndelegateClaimProof", FIELDS, GeneratedVisitor)
    }
}
