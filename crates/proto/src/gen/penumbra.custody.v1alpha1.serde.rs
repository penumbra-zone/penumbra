// @generated
impl serde::Serialize for AuthorizeRequest {
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
        if self.account_group_id.is_some() {
            len += 1;
        }
        if !self.pre_authorizations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1alpha1.AuthorizeRequest", len)?;
        if let Some(v) = self.plan.as_ref() {
            struct_ser.serialize_field("plan", v)?;
        }
        if let Some(v) = self.account_group_id.as_ref() {
            struct_ser.serialize_field("accountGroupId", v)?;
        }
        if !self.pre_authorizations.is_empty() {
            struct_ser.serialize_field("preAuthorizations", &self.pre_authorizations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "plan",
            "account_group_id",
            "accountGroupId",
            "pre_authorizations",
            "preAuthorizations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Plan,
            AccountGroupId,
            PreAuthorizations,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            "accountGroupId" | "account_group_id" => Ok(GeneratedField::AccountGroupId),
                            "preAuthorizations" | "pre_authorizations" => Ok(GeneratedField::PreAuthorizations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1alpha1.AuthorizeRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AuthorizeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut plan__ = None;
                let mut account_group_id__ = None;
                let mut pre_authorizations__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Plan => {
                            if plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("plan"));
                            }
                            plan__ = map.next_value()?;
                        }
                        GeneratedField::AccountGroupId => {
                            if account_group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountGroupId"));
                            }
                            account_group_id__ = map.next_value()?;
                        }
                        GeneratedField::PreAuthorizations => {
                            if pre_authorizations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("preAuthorizations"));
                            }
                            pre_authorizations__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AuthorizeRequest {
                    plan: plan__,
                    account_group_id: account_group_id__,
                    pre_authorizations: pre_authorizations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1alpha1.AuthorizeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthorizeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1alpha1.AuthorizeResponse", len)?;
        if let Some(v) = self.data.as_ref() {
            struct_ser.serialize_field("data", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Data,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1alpha1.AuthorizeResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AuthorizeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = map.next_value()?;
                        }
                    }
                }
                Ok(AuthorizeResponse {
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1alpha1.AuthorizeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PreAuthorization {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.pre_authorization.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1alpha1.PreAuthorization", len)?;
        if let Some(v) = self.pre_authorization.as_ref() {
            match v {
                pre_authorization::PreAuthorization::Ed25519(v) => {
                    struct_ser.serialize_field("ed25519", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PreAuthorization {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ed25519",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Ed25519,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "ed25519" => Ok(GeneratedField::Ed25519),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PreAuthorization;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1alpha1.PreAuthorization")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PreAuthorization, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pre_authorization__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Ed25519 => {
                            if pre_authorization__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ed25519"));
                            }
                            pre_authorization__ = map.next_value::<::std::option::Option<_>>()?.map(pre_authorization::PreAuthorization::Ed25519)
;
                        }
                    }
                }
                Ok(PreAuthorization {
                    pre_authorization: pre_authorization__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1alpha1.PreAuthorization", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for pre_authorization::Ed25519 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.vk.is_empty() {
            len += 1;
        }
        if !self.sig.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1alpha1.PreAuthorization.Ed25519", len)?;
        if !self.vk.is_empty() {
            struct_ser.serialize_field("vk", pbjson::private::base64::encode(&self.vk).as_str())?;
        }
        if !self.sig.is_empty() {
            struct_ser.serialize_field("sig", pbjson::private::base64::encode(&self.sig).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for pre_authorization::Ed25519 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vk",
            "sig",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vk,
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
                            "vk" => Ok(GeneratedField::Vk),
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
            type Value = pre_authorization::Ed25519;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1alpha1.PreAuthorization.Ed25519")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<pre_authorization::Ed25519, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vk__ = None;
                let mut sig__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Vk => {
                            if vk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vk"));
                            }
                            vk__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
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
                Ok(pre_authorization::Ed25519 {
                    vk: vk__.unwrap_or_default(),
                    sig: sig__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1alpha1.PreAuthorization.Ed25519", FIELDS, GeneratedVisitor)
    }
}
