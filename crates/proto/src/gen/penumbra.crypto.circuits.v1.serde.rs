impl serde::Serialize for DelegatorVoteCircuit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.public.is_some() {
            len += 1;
        }
        if self.private.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.DelegatorVoteCircuit", len)?;
        if let Some(v) = self.public.as_ref() {
            struct_ser.serialize_field("public", v)?;
        }
        if let Some(v) = self.private.as_ref() {
            struct_ser.serialize_field("private", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorVoteCircuit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "public",
            "private",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Public,
            Private,
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
                            "public" => Ok(GeneratedField::Public),
                            "private" => Ok(GeneratedField::Private),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorVoteCircuit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.DelegatorVoteCircuit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DelegatorVoteCircuit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut public__ = None;
                let mut private__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Public => {
                            if public__.is_some() {
                                return Err(serde::de::Error::duplicate_field("public"));
                            }
                            public__ = map_.next_value()?;
                        }
                        GeneratedField::Private => {
                            if private__.is_some() {
                                return Err(serde::de::Error::duplicate_field("private"));
                            }
                            private__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DelegatorVoteCircuit {
                    public: public__,
                    private: private__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.DelegatorVoteCircuit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorVoteProofPrivate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.state_commitment_proof.is_some() {
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
        if self.ak.is_some() {
            len += 1;
        }
        if self.nk.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.DelegatorVoteProofPrivate", len)?;
        if let Some(v) = self.state_commitment_proof.as_ref() {
            struct_ser.serialize_field("stateCommitmentProof", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if !self.v_blinding.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("vBlinding", pbjson::private::base64::encode(&self.v_blinding).as_str())?;
        }
        if !self.spend_auth_randomizer.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("spendAuthRandomizer", pbjson::private::base64::encode(&self.spend_auth_randomizer).as_str())?;
        }
        if let Some(v) = self.ak.as_ref() {
            struct_ser.serialize_field("ak", v)?;
        }
        if let Some(v) = self.nk.as_ref() {
            struct_ser.serialize_field("nk", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorVoteProofPrivate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "state_commitment_proof",
            "stateCommitmentProof",
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
            StateCommitmentProof,
            Note,
            VBlinding,
            SpendAuthRandomizer,
            Ak,
            Nk,
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
                            "stateCommitmentProof" | "state_commitment_proof" => Ok(GeneratedField::StateCommitmentProof),
                            "note" => Ok(GeneratedField::Note),
                            "vBlinding" | "v_blinding" => Ok(GeneratedField::VBlinding),
                            "spendAuthRandomizer" | "spend_auth_randomizer" => Ok(GeneratedField::SpendAuthRandomizer),
                            "ak" => Ok(GeneratedField::Ak),
                            "nk" => Ok(GeneratedField::Nk),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorVoteProofPrivate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.DelegatorVoteProofPrivate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DelegatorVoteProofPrivate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state_commitment_proof__ = None;
                let mut note__ = None;
                let mut v_blinding__ = None;
                let mut spend_auth_randomizer__ = None;
                let mut ak__ = None;
                let mut nk__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StateCommitmentProof => {
                            if state_commitment_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stateCommitmentProof"));
                            }
                            state_commitment_proof__ = map_.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map_.next_value()?;
                        }
                        GeneratedField::VBlinding => {
                            if v_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vBlinding"));
                            }
                            v_blinding__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SpendAuthRandomizer => {
                            if spend_auth_randomizer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendAuthRandomizer"));
                            }
                            spend_auth_randomizer__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Ak => {
                            if ak__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ak"));
                            }
                            ak__ = map_.next_value()?;
                        }
                        GeneratedField::Nk => {
                            if nk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nk"));
                            }
                            nk__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DelegatorVoteProofPrivate {
                    state_commitment_proof: state_commitment_proof__,
                    note: note__,
                    v_blinding: v_blinding__.unwrap_or_default(),
                    spend_auth_randomizer: spend_auth_randomizer__.unwrap_or_default(),
                    ak: ak__,
                    nk: nk__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.DelegatorVoteProofPrivate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorVoteProofPublic {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.anchor.is_some() {
            len += 1;
        }
        if self.balance_commitment.is_some() {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.rk.is_some() {
            len += 1;
        }
        if self.start_position != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.DelegatorVoteProofPublic", len)?;
        if let Some(v) = self.anchor.as_ref() {
            struct_ser.serialize_field("anchor", v)?;
        }
        if let Some(v) = self.balance_commitment.as_ref() {
            struct_ser.serialize_field("balanceCommitment", v)?;
        }
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if let Some(v) = self.rk.as_ref() {
            struct_ser.serialize_field("rk", v)?;
        }
        if self.start_position != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startPosition", ToString::to_string(&self.start_position).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorVoteProofPublic {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "anchor",
            "balance_commitment",
            "balanceCommitment",
            "nullifier",
            "rk",
            "start_position",
            "startPosition",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Anchor,
            BalanceCommitment,
            Nullifier,
            Rk,
            StartPosition,
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
                            "anchor" => Ok(GeneratedField::Anchor),
                            "balanceCommitment" | "balance_commitment" => Ok(GeneratedField::BalanceCommitment),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "rk" => Ok(GeneratedField::Rk),
                            "startPosition" | "start_position" => Ok(GeneratedField::StartPosition),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorVoteProofPublic;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.DelegatorVoteProofPublic")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DelegatorVoteProofPublic, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut anchor__ = None;
                let mut balance_commitment__ = None;
                let mut nullifier__ = None;
                let mut rk__ = None;
                let mut start_position__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Anchor => {
                            if anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("anchor"));
                            }
                            anchor__ = map_.next_value()?;
                        }
                        GeneratedField::BalanceCommitment => {
                            if balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceCommitment"));
                            }
                            balance_commitment__ = map_.next_value()?;
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
                        GeneratedField::StartPosition => {
                            if start_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startPosition"));
                            }
                            start_position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DelegatorVoteProofPublic {
                    anchor: anchor__,
                    balance_commitment: balance_commitment__,
                    nullifier: nullifier__,
                    rk: rk__,
                    start_position: start_position__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.DelegatorVoteProofPublic", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputCircuit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.public.is_some() {
            len += 1;
        }
        if self.private.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.OutputCircuit", len)?;
        if let Some(v) = self.public.as_ref() {
            struct_ser.serialize_field("public", v)?;
        }
        if let Some(v) = self.private.as_ref() {
            struct_ser.serialize_field("private", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputCircuit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "public",
            "private",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Public,
            Private,
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
                            "public" => Ok(GeneratedField::Public),
                            "private" => Ok(GeneratedField::Private),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputCircuit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.OutputCircuit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OutputCircuit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut public__ = None;
                let mut private__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Public => {
                            if public__.is_some() {
                                return Err(serde::de::Error::duplicate_field("public"));
                            }
                            public__ = map_.next_value()?;
                        }
                        GeneratedField::Private => {
                            if private__.is_some() {
                                return Err(serde::de::Error::duplicate_field("private"));
                            }
                            private__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OutputCircuit {
                    public: public__,
                    private: private__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.OutputCircuit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputProofPrivate {
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
        if !self.balance_blinding.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.OutputProofPrivate", len)?;
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if !self.balance_blinding.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("balanceBlinding", pbjson::private::base64::encode(&self.balance_blinding).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputProofPrivate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note",
            "balance_blinding",
            "balanceBlinding",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Note,
            BalanceBlinding,
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
                            "balanceBlinding" | "balance_blinding" => Ok(GeneratedField::BalanceBlinding),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputProofPrivate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.OutputProofPrivate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OutputProofPrivate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note__ = None;
                let mut balance_blinding__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map_.next_value()?;
                        }
                        GeneratedField::BalanceBlinding => {
                            if balance_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceBlinding"));
                            }
                            balance_blinding__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OutputProofPrivate {
                    note: note__,
                    balance_blinding: balance_blinding__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.OutputProofPrivate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputProofPublic {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.balance_commitment.is_some() {
            len += 1;
        }
        if self.note_commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.OutputProofPublic", len)?;
        if let Some(v) = self.balance_commitment.as_ref() {
            struct_ser.serialize_field("balanceCommitment", v)?;
        }
        if let Some(v) = self.note_commitment.as_ref() {
            struct_ser.serialize_field("noteCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputProofPublic {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "balance_commitment",
            "balanceCommitment",
            "note_commitment",
            "noteCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BalanceCommitment,
            NoteCommitment,
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
                            "balanceCommitment" | "balance_commitment" => Ok(GeneratedField::BalanceCommitment),
                            "noteCommitment" | "note_commitment" => Ok(GeneratedField::NoteCommitment),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputProofPublic;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.OutputProofPublic")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OutputProofPublic, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut balance_commitment__ = None;
                let mut note_commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BalanceCommitment => {
                            if balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceCommitment"));
                            }
                            balance_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::NoteCommitment => {
                            if note_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitment"));
                            }
                            note_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OutputProofPublic {
                    balance_commitment: balance_commitment__,
                    note_commitment: note_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.OutputProofPublic", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendCircuit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.public.is_some() {
            len += 1;
        }
        if self.private.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.SpendCircuit", len)?;
        if let Some(v) = self.public.as_ref() {
            struct_ser.serialize_field("public", v)?;
        }
        if let Some(v) = self.private.as_ref() {
            struct_ser.serialize_field("private", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendCircuit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "public",
            "private",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Public,
            Private,
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
                            "public" => Ok(GeneratedField::Public),
                            "private" => Ok(GeneratedField::Private),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendCircuit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.SpendCircuit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SpendCircuit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut public__ = None;
                let mut private__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Public => {
                            if public__.is_some() {
                                return Err(serde::de::Error::duplicate_field("public"));
                            }
                            public__ = map_.next_value()?;
                        }
                        GeneratedField::Private => {
                            if private__.is_some() {
                                return Err(serde::de::Error::duplicate_field("private"));
                            }
                            private__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SpendCircuit {
                    public: public__,
                    private: private__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.SpendCircuit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendProofPrivate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.state_commitment_proof.is_some() {
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
        if self.ak.is_some() {
            len += 1;
        }
        if self.nk.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.SpendProofPrivate", len)?;
        if let Some(v) = self.state_commitment_proof.as_ref() {
            struct_ser.serialize_field("stateCommitmentProof", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if !self.v_blinding.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("vBlinding", pbjson::private::base64::encode(&self.v_blinding).as_str())?;
        }
        if !self.spend_auth_randomizer.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("spendAuthRandomizer", pbjson::private::base64::encode(&self.spend_auth_randomizer).as_str())?;
        }
        if let Some(v) = self.ak.as_ref() {
            struct_ser.serialize_field("ak", v)?;
        }
        if let Some(v) = self.nk.as_ref() {
            struct_ser.serialize_field("nk", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendProofPrivate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "state_commitment_proof",
            "stateCommitmentProof",
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
            StateCommitmentProof,
            Note,
            VBlinding,
            SpendAuthRandomizer,
            Ak,
            Nk,
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
                            "stateCommitmentProof" | "state_commitment_proof" => Ok(GeneratedField::StateCommitmentProof),
                            "note" => Ok(GeneratedField::Note),
                            "vBlinding" | "v_blinding" => Ok(GeneratedField::VBlinding),
                            "spendAuthRandomizer" | "spend_auth_randomizer" => Ok(GeneratedField::SpendAuthRandomizer),
                            "ak" => Ok(GeneratedField::Ak),
                            "nk" => Ok(GeneratedField::Nk),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendProofPrivate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.SpendProofPrivate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SpendProofPrivate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state_commitment_proof__ = None;
                let mut note__ = None;
                let mut v_blinding__ = None;
                let mut spend_auth_randomizer__ = None;
                let mut ak__ = None;
                let mut nk__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StateCommitmentProof => {
                            if state_commitment_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stateCommitmentProof"));
                            }
                            state_commitment_proof__ = map_.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map_.next_value()?;
                        }
                        GeneratedField::VBlinding => {
                            if v_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vBlinding"));
                            }
                            v_blinding__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SpendAuthRandomizer => {
                            if spend_auth_randomizer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendAuthRandomizer"));
                            }
                            spend_auth_randomizer__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Ak => {
                            if ak__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ak"));
                            }
                            ak__ = map_.next_value()?;
                        }
                        GeneratedField::Nk => {
                            if nk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nk"));
                            }
                            nk__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SpendProofPrivate {
                    state_commitment_proof: state_commitment_proof__,
                    note: note__,
                    v_blinding: v_blinding__.unwrap_or_default(),
                    spend_auth_randomizer: spend_auth_randomizer__.unwrap_or_default(),
                    ak: ak__,
                    nk: nk__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.SpendProofPrivate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendProofPublic {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.anchor.is_some() {
            len += 1;
        }
        if self.balance_commitment.is_some() {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.rk.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.SpendProofPublic", len)?;
        if let Some(v) = self.anchor.as_ref() {
            struct_ser.serialize_field("anchor", v)?;
        }
        if let Some(v) = self.balance_commitment.as_ref() {
            struct_ser.serialize_field("balanceCommitment", v)?;
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
impl<'de> serde::Deserialize<'de> for SpendProofPublic {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "anchor",
            "balance_commitment",
            "balanceCommitment",
            "nullifier",
            "rk",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Anchor,
            BalanceCommitment,
            Nullifier,
            Rk,
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
                            "anchor" => Ok(GeneratedField::Anchor),
                            "balanceCommitment" | "balance_commitment" => Ok(GeneratedField::BalanceCommitment),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "rk" => Ok(GeneratedField::Rk),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendProofPublic;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.SpendProofPublic")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SpendProofPublic, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut anchor__ = None;
                let mut balance_commitment__ = None;
                let mut nullifier__ = None;
                let mut rk__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Anchor => {
                            if anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("anchor"));
                            }
                            anchor__ = map_.next_value()?;
                        }
                        GeneratedField::BalanceCommitment => {
                            if balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceCommitment"));
                            }
                            balance_commitment__ = map_.next_value()?;
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SpendProofPublic {
                    anchor: anchor__,
                    balance_commitment: balance_commitment__,
                    nullifier: nullifier__,
                    rk: rk__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.SpendProofPublic", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapCircuit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.public.is_some() {
            len += 1;
        }
        if self.private.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.SwapCircuit", len)?;
        if let Some(v) = self.public.as_ref() {
            struct_ser.serialize_field("public", v)?;
        }
        if let Some(v) = self.private.as_ref() {
            struct_ser.serialize_field("private", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapCircuit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "public",
            "private",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Public,
            Private,
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
                            "public" => Ok(GeneratedField::Public),
                            "private" => Ok(GeneratedField::Private),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapCircuit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.SwapCircuit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapCircuit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut public__ = None;
                let mut private__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Public => {
                            if public__.is_some() {
                                return Err(serde::de::Error::duplicate_field("public"));
                            }
                            public__ = map_.next_value()?;
                        }
                        GeneratedField::Private => {
                            if private__.is_some() {
                                return Err(serde::de::Error::duplicate_field("private"));
                            }
                            private__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapCircuit {
                    public: public__,
                    private: private__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.SwapCircuit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapClaimCircuit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.public.is_some() {
            len += 1;
        }
        if self.private.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.SwapClaimCircuit", len)?;
        if let Some(v) = self.public.as_ref() {
            struct_ser.serialize_field("public", v)?;
        }
        if let Some(v) = self.private.as_ref() {
            struct_ser.serialize_field("private", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapClaimCircuit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "public",
            "private",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Public,
            Private,
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
                            "public" => Ok(GeneratedField::Public),
                            "private" => Ok(GeneratedField::Private),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapClaimCircuit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.SwapClaimCircuit")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapClaimCircuit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut public__ = None;
                let mut private__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Public => {
                            if public__.is_some() {
                                return Err(serde::de::Error::duplicate_field("public"));
                            }
                            public__ = map_.next_value()?;
                        }
                        GeneratedField::Private => {
                            if private__.is_some() {
                                return Err(serde::de::Error::duplicate_field("private"));
                            }
                            private__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapClaimCircuit {
                    public: public__,
                    private: private__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.SwapClaimCircuit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapClaimProofPrivate {
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
        if self.state_commitment_proof.is_some() {
            len += 1;
        }
        if self.ak.is_some() {
            len += 1;
        }
        if self.nk.is_some() {
            len += 1;
        }
        if self.lambda_1.is_some() {
            len += 1;
        }
        if self.lambda_2.is_some() {
            len += 1;
        }
        if !self.note_blinding_1.is_empty() {
            len += 1;
        }
        if !self.note_blinding_2.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.SwapClaimProofPrivate", len)?;
        if let Some(v) = self.swap_plaintext.as_ref() {
            struct_ser.serialize_field("swapPlaintext", v)?;
        }
        if let Some(v) = self.state_commitment_proof.as_ref() {
            struct_ser.serialize_field("stateCommitmentProof", v)?;
        }
        if let Some(v) = self.ak.as_ref() {
            struct_ser.serialize_field("ak", v)?;
        }
        if let Some(v) = self.nk.as_ref() {
            struct_ser.serialize_field("nk", v)?;
        }
        if let Some(v) = self.lambda_1.as_ref() {
            struct_ser.serialize_field("lambda1", v)?;
        }
        if let Some(v) = self.lambda_2.as_ref() {
            struct_ser.serialize_field("lambda2", v)?;
        }
        if !self.note_blinding_1.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("noteBlinding1", pbjson::private::base64::encode(&self.note_blinding_1).as_str())?;
        }
        if !self.note_blinding_2.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("noteBlinding2", pbjson::private::base64::encode(&self.note_blinding_2).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapClaimProofPrivate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_plaintext",
            "swapPlaintext",
            "state_commitment_proof",
            "stateCommitmentProof",
            "ak",
            "nk",
            "lambda_1",
            "lambda1",
            "lambda_2",
            "lambda2",
            "note_blinding_1",
            "noteBlinding1",
            "note_blinding_2",
            "noteBlinding2",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapPlaintext,
            StateCommitmentProof,
            Ak,
            Nk,
            Lambda1,
            Lambda2,
            NoteBlinding1,
            NoteBlinding2,
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
                            "stateCommitmentProof" | "state_commitment_proof" => Ok(GeneratedField::StateCommitmentProof),
                            "ak" => Ok(GeneratedField::Ak),
                            "nk" => Ok(GeneratedField::Nk),
                            "lambda1" | "lambda_1" => Ok(GeneratedField::Lambda1),
                            "lambda2" | "lambda_2" => Ok(GeneratedField::Lambda2),
                            "noteBlinding1" | "note_blinding_1" => Ok(GeneratedField::NoteBlinding1),
                            "noteBlinding2" | "note_blinding_2" => Ok(GeneratedField::NoteBlinding2),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapClaimProofPrivate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.SwapClaimProofPrivate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapClaimProofPrivate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_plaintext__ = None;
                let mut state_commitment_proof__ = None;
                let mut ak__ = None;
                let mut nk__ = None;
                let mut lambda_1__ = None;
                let mut lambda_2__ = None;
                let mut note_blinding_1__ = None;
                let mut note_blinding_2__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SwapPlaintext => {
                            if swap_plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapPlaintext"));
                            }
                            swap_plaintext__ = map_.next_value()?;
                        }
                        GeneratedField::StateCommitmentProof => {
                            if state_commitment_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stateCommitmentProof"));
                            }
                            state_commitment_proof__ = map_.next_value()?;
                        }
                        GeneratedField::Ak => {
                            if ak__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ak"));
                            }
                            ak__ = map_.next_value()?;
                        }
                        GeneratedField::Nk => {
                            if nk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nk"));
                            }
                            nk__ = map_.next_value()?;
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
                        GeneratedField::NoteBlinding1 => {
                            if note_blinding_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteBlinding1"));
                            }
                            note_blinding_1__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::NoteBlinding2 => {
                            if note_blinding_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteBlinding2"));
                            }
                            note_blinding_2__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapClaimProofPrivate {
                    swap_plaintext: swap_plaintext__,
                    state_commitment_proof: state_commitment_proof__,
                    ak: ak__,
                    nk: nk__,
                    lambda_1: lambda_1__,
                    lambda_2: lambda_2__,
                    note_blinding_1: note_blinding_1__.unwrap_or_default(),
                    note_blinding_2: note_blinding_2__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.SwapClaimProofPrivate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapClaimProofPublic {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.anchor.is_some() {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.claim_fee.is_some() {
            len += 1;
        }
        if self.output_data.is_some() {
            len += 1;
        }
        if self.note_commitment_1.is_some() {
            len += 1;
        }
        if self.note_commitment_2.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.SwapClaimProofPublic", len)?;
        if let Some(v) = self.anchor.as_ref() {
            struct_ser.serialize_field("anchor", v)?;
        }
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if let Some(v) = self.claim_fee.as_ref() {
            struct_ser.serialize_field("claimFee", v)?;
        }
        if let Some(v) = self.output_data.as_ref() {
            struct_ser.serialize_field("outputData", v)?;
        }
        if let Some(v) = self.note_commitment_1.as_ref() {
            struct_ser.serialize_field("noteCommitment1", v)?;
        }
        if let Some(v) = self.note_commitment_2.as_ref() {
            struct_ser.serialize_field("noteCommitment2", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapClaimProofPublic {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "anchor",
            "nullifier",
            "claim_fee",
            "claimFee",
            "output_data",
            "outputData",
            "note_commitment_1",
            "noteCommitment1",
            "note_commitment_2",
            "noteCommitment2",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Anchor,
            Nullifier,
            ClaimFee,
            OutputData,
            NoteCommitment1,
            NoteCommitment2,
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
                            "anchor" => Ok(GeneratedField::Anchor),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "claimFee" | "claim_fee" => Ok(GeneratedField::ClaimFee),
                            "outputData" | "output_data" => Ok(GeneratedField::OutputData),
                            "noteCommitment1" | "note_commitment_1" => Ok(GeneratedField::NoteCommitment1),
                            "noteCommitment2" | "note_commitment_2" => Ok(GeneratedField::NoteCommitment2),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapClaimProofPublic;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.SwapClaimProofPublic")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapClaimProofPublic, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut anchor__ = None;
                let mut nullifier__ = None;
                let mut claim_fee__ = None;
                let mut output_data__ = None;
                let mut note_commitment_1__ = None;
                let mut note_commitment_2__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Anchor => {
                            if anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("anchor"));
                            }
                            anchor__ = map_.next_value()?;
                        }
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map_.next_value()?;
                        }
                        GeneratedField::ClaimFee => {
                            if claim_fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("claimFee"));
                            }
                            claim_fee__ = map_.next_value()?;
                        }
                        GeneratedField::OutputData => {
                            if output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputData"));
                            }
                            output_data__ = map_.next_value()?;
                        }
                        GeneratedField::NoteCommitment1 => {
                            if note_commitment_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitment1"));
                            }
                            note_commitment_1__ = map_.next_value()?;
                        }
                        GeneratedField::NoteCommitment2 => {
                            if note_commitment_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitment2"));
                            }
                            note_commitment_2__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapClaimProofPublic {
                    anchor: anchor__,
                    nullifier: nullifier__,
                    claim_fee: claim_fee__,
                    output_data: output_data__,
                    note_commitment_1: note_commitment_1__,
                    note_commitment_2: note_commitment_2__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.SwapClaimProofPublic", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapProofPrivate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fee_blinding.is_empty() {
            len += 1;
        }
        if self.swap_plaintext.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.SwapProofPrivate", len)?;
        if !self.fee_blinding.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("feeBlinding", pbjson::private::base64::encode(&self.fee_blinding).as_str())?;
        }
        if let Some(v) = self.swap_plaintext.as_ref() {
            struct_ser.serialize_field("swapPlaintext", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapProofPrivate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fee_blinding",
            "feeBlinding",
            "swap_plaintext",
            "swapPlaintext",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FeeBlinding,
            SwapPlaintext,
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
                            "feeBlinding" | "fee_blinding" => Ok(GeneratedField::FeeBlinding),
                            "swapPlaintext" | "swap_plaintext" => Ok(GeneratedField::SwapPlaintext),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapProofPrivate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.SwapProofPrivate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapProofPrivate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fee_blinding__ = None;
                let mut swap_plaintext__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FeeBlinding => {
                            if fee_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeBlinding"));
                            }
                            fee_blinding__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SwapPlaintext => {
                            if swap_plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapPlaintext"));
                            }
                            swap_plaintext__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapProofPrivate {
                    fee_blinding: fee_blinding__.unwrap_or_default(),
                    swap_plaintext: swap_plaintext__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.SwapProofPrivate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapProofPublic {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.balance_commitment.is_some() {
            len += 1;
        }
        if self.swap_commitment.is_some() {
            len += 1;
        }
        if self.fee_commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.crypto.circuits.v1.SwapProofPublic", len)?;
        if let Some(v) = self.balance_commitment.as_ref() {
            struct_ser.serialize_field("balanceCommitment", v)?;
        }
        if let Some(v) = self.swap_commitment.as_ref() {
            struct_ser.serialize_field("swapCommitment", v)?;
        }
        if let Some(v) = self.fee_commitment.as_ref() {
            struct_ser.serialize_field("feeCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapProofPublic {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "balance_commitment",
            "balanceCommitment",
            "swap_commitment",
            "swapCommitment",
            "fee_commitment",
            "feeCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BalanceCommitment,
            SwapCommitment,
            FeeCommitment,
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
                            "balanceCommitment" | "balance_commitment" => Ok(GeneratedField::BalanceCommitment),
                            "swapCommitment" | "swap_commitment" => Ok(GeneratedField::SwapCommitment),
                            "feeCommitment" | "fee_commitment" => Ok(GeneratedField::FeeCommitment),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapProofPublic;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.crypto.circuits.v1.SwapProofPublic")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SwapProofPublic, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut balance_commitment__ = None;
                let mut swap_commitment__ = None;
                let mut fee_commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BalanceCommitment => {
                            if balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceCommitment"));
                            }
                            balance_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::SwapCommitment => {
                            if swap_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapCommitment"));
                            }
                            swap_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::FeeCommitment => {
                            if fee_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("feeCommitment"));
                            }
                            fee_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SwapProofPublic {
                    balance_commitment: balance_commitment__,
                    swap_commitment: swap_commitment__,
                    fee_commitment: fee_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.crypto.circuits.v1.SwapProofPublic", FIELDS, GeneratedVisitor)
    }
}
