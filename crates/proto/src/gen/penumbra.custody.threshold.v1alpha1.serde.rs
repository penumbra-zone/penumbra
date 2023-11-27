impl serde::Serialize for CoordinatorRound1 {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.threshold.v1alpha1.CoordinatorRound1", len)?;
        if let Some(v) = self.plan.as_ref() {
            struct_ser.serialize_field("plan", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CoordinatorRound1 {
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
            type Value = CoordinatorRound1;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.threshold.v1alpha1.CoordinatorRound1")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CoordinatorRound1, V::Error>
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
                Ok(CoordinatorRound1 {
                    plan: plan__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.threshold.v1alpha1.CoordinatorRound1", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CoordinatorRound2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.signing_packages.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.threshold.v1alpha1.CoordinatorRound2", len)?;
        if !self.signing_packages.is_empty() {
            struct_ser.serialize_field("signingPackages", &self.signing_packages)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CoordinatorRound2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "signing_packages",
            "signingPackages",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SigningPackages,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "signingPackages" | "signing_packages" => Ok(GeneratedField::SigningPackages),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CoordinatorRound2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.threshold.v1alpha1.CoordinatorRound2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CoordinatorRound2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut signing_packages__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SigningPackages => {
                            if signing_packages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signingPackages"));
                            }
                            signing_packages__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CoordinatorRound2 {
                    signing_packages: signing_packages__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.threshold.v1alpha1.CoordinatorRound2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for coordinator_round2::IdentifiedCommitments {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.identifier.is_empty() {
            len += 1;
        }
        if self.commitments.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.threshold.v1alpha1.CoordinatorRound2.IdentifiedCommitments", len)?;
        if !self.identifier.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("identifier", pbjson::private::base64::encode(&self.identifier).as_str())?;
        }
        if let Some(v) = self.commitments.as_ref() {
            struct_ser.serialize_field("commitments", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for coordinator_round2::IdentifiedCommitments {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "identifier",
            "commitments",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Identifier,
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
                            "identifier" => Ok(GeneratedField::Identifier),
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
            type Value = coordinator_round2::IdentifiedCommitments;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.threshold.v1alpha1.CoordinatorRound2.IdentifiedCommitments")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<coordinator_round2::IdentifiedCommitments, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut identifier__ = None;
                let mut commitments__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Identifier => {
                            if identifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identifier"));
                            }
                            identifier__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Commitments => {
                            if commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitments"));
                            }
                            commitments__ = map_.next_value()?;
                        }
                    }
                }
                Ok(coordinator_round2::IdentifiedCommitments {
                    identifier: identifier__.unwrap_or_default(),
                    commitments: commitments__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.threshold.v1alpha1.CoordinatorRound2.IdentifiedCommitments", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for coordinator_round2::PartialSigningPackage {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.all_commitments.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.threshold.v1alpha1.CoordinatorRound2.PartialSigningPackage", len)?;
        if !self.all_commitments.is_empty() {
            struct_ser.serialize_field("allCommitments", &self.all_commitments)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for coordinator_round2::PartialSigningPackage {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "all_commitments",
            "allCommitments",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AllCommitments,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "allCommitments" | "all_commitments" => Ok(GeneratedField::AllCommitments),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = coordinator_round2::PartialSigningPackage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.threshold.v1alpha1.CoordinatorRound2.PartialSigningPackage")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<coordinator_round2::PartialSigningPackage, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut all_commitments__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AllCommitments => {
                            if all_commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allCommitments"));
                            }
                            all_commitments__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(coordinator_round2::PartialSigningPackage {
                    all_commitments: all_commitments__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.threshold.v1alpha1.CoordinatorRound2.PartialSigningPackage", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FollowerRound1 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.inner.is_some() {
            len += 1;
        }
        if self.pk.is_some() {
            len += 1;
        }
        if self.sig.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.threshold.v1alpha1.FollowerRound1", len)?;
        if let Some(v) = self.inner.as_ref() {
            struct_ser.serialize_field("inner", v)?;
        }
        if let Some(v) = self.pk.as_ref() {
            struct_ser.serialize_field("pk", v)?;
        }
        if let Some(v) = self.sig.as_ref() {
            struct_ser.serialize_field("sig", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FollowerRound1 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "inner",
            "pk",
            "sig",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Inner,
            Pk,
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
                            "inner" => Ok(GeneratedField::Inner),
                            "pk" => Ok(GeneratedField::Pk),
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
            type Value = FollowerRound1;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.threshold.v1alpha1.FollowerRound1")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FollowerRound1, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                let mut pk__ = None;
                let mut sig__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = map_.next_value()?;
                        }
                        GeneratedField::Pk => {
                            if pk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pk"));
                            }
                            pk__ = map_.next_value()?;
                        }
                        GeneratedField::Sig => {
                            if sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sig"));
                            }
                            sig__ = map_.next_value()?;
                        }
                    }
                }
                Ok(FollowerRound1 {
                    inner: inner__,
                    pk: pk__,
                    sig: sig__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.threshold.v1alpha1.FollowerRound1", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for follower_round1::Inner {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.commitments.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.threshold.v1alpha1.FollowerRound1.Inner", len)?;
        if !self.commitments.is_empty() {
            struct_ser.serialize_field("commitments", &self.commitments)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for follower_round1::Inner {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commitments",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = follower_round1::Inner;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.threshold.v1alpha1.FollowerRound1.Inner")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<follower_round1::Inner, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commitments__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Commitments => {
                            if commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitments"));
                            }
                            commitments__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(follower_round1::Inner {
                    commitments: commitments__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.threshold.v1alpha1.FollowerRound1.Inner", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FollowerRound2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.inner.is_some() {
            len += 1;
        }
        if self.pk.is_some() {
            len += 1;
        }
        if self.sig.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.threshold.v1alpha1.FollowerRound2", len)?;
        if let Some(v) = self.inner.as_ref() {
            struct_ser.serialize_field("inner", v)?;
        }
        if let Some(v) = self.pk.as_ref() {
            struct_ser.serialize_field("pk", v)?;
        }
        if let Some(v) = self.sig.as_ref() {
            struct_ser.serialize_field("sig", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FollowerRound2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "inner",
            "pk",
            "sig",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Inner,
            Pk,
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
                            "inner" => Ok(GeneratedField::Inner),
                            "pk" => Ok(GeneratedField::Pk),
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
            type Value = FollowerRound2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.threshold.v1alpha1.FollowerRound2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FollowerRound2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                let mut pk__ = None;
                let mut sig__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = map_.next_value()?;
                        }
                        GeneratedField::Pk => {
                            if pk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pk"));
                            }
                            pk__ = map_.next_value()?;
                        }
                        GeneratedField::Sig => {
                            if sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sig"));
                            }
                            sig__ = map_.next_value()?;
                        }
                    }
                }
                Ok(FollowerRound2 {
                    inner: inner__,
                    pk: pk__,
                    sig: sig__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.threshold.v1alpha1.FollowerRound2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for follower_round2::Inner {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.shares.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.threshold.v1alpha1.FollowerRound2.Inner", len)?;
        if !self.shares.is_empty() {
            struct_ser.serialize_field("shares", &self.shares)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for follower_round2::Inner {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "shares",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = follower_round2::Inner;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.threshold.v1alpha1.FollowerRound2.Inner")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<follower_round2::Inner, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut shares__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Shares => {
                            if shares__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shares"));
                            }
                            shares__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(follower_round2::Inner {
                    shares: shares__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.threshold.v1alpha1.FollowerRound2.Inner", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Signature {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.threshold.v1alpha1.Signature", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Signature {
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
            type Value = Signature;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.threshold.v1alpha1.Signature")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Signature, V::Error>
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
                Ok(Signature {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.threshold.v1alpha1.Signature", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VerificationKey {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.threshold.v1alpha1.VerificationKey", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VerificationKey {
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
            type Value = VerificationKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.threshold.v1alpha1.VerificationKey")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<VerificationKey, V::Error>
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
                Ok(VerificationKey {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.threshold.v1alpha1.VerificationKey", FIELDS, GeneratedVisitor)
    }
}
