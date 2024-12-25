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
        if !self.pre_authorizations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.AuthorizeRequest", len)?;
        if let Some(v) = self.plan.as_ref() {
            struct_ser.serialize_field("plan", v)?;
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
            "pre_authorizations",
            "preAuthorizations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Plan,
            PreAuthorizations,
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
                            "plan" => Ok(GeneratedField::Plan),
                            "preAuthorizations" | "pre_authorizations" => Ok(GeneratedField::PreAuthorizations),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.custody.v1.AuthorizeRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut plan__ = None;
                let mut pre_authorizations__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Plan => {
                            if plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("plan"));
                            }
                            plan__ = map_.next_value()?;
                        }
                        GeneratedField::PreAuthorizations => {
                            if pre_authorizations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("preAuthorizations"));
                            }
                            pre_authorizations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuthorizeRequest {
                    plan: plan__,
                    pre_authorizations: pre_authorizations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.AuthorizeRequest", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.AuthorizeResponse", len)?;
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
                            "data" => Ok(GeneratedField::Data),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.custody.v1.AuthorizeResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuthorizeResponse {
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.AuthorizeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthorizeValidatorDefinitionRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator_definition.is_some() {
            len += 1;
        }
        if !self.pre_authorizations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.AuthorizeValidatorDefinitionRequest", len)?;
        if let Some(v) = self.validator_definition.as_ref() {
            struct_ser.serialize_field("validatorDefinition", v)?;
        }
        if !self.pre_authorizations.is_empty() {
            struct_ser.serialize_field("preAuthorizations", &self.pre_authorizations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizeValidatorDefinitionRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_definition",
            "validatorDefinition",
            "pre_authorizations",
            "preAuthorizations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorDefinition,
            PreAuthorizations,
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
                            "validatorDefinition" | "validator_definition" => Ok(GeneratedField::ValidatorDefinition),
                            "preAuthorizations" | "pre_authorizations" => Ok(GeneratedField::PreAuthorizations),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizeValidatorDefinitionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1.AuthorizeValidatorDefinitionRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizeValidatorDefinitionRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_definition__ = None;
                let mut pre_authorizations__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidatorDefinition => {
                            if validator_definition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDefinition"));
                            }
                            validator_definition__ = map_.next_value()?;
                        }
                        GeneratedField::PreAuthorizations => {
                            if pre_authorizations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("preAuthorizations"));
                            }
                            pre_authorizations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuthorizeValidatorDefinitionRequest {
                    validator_definition: validator_definition__,
                    pre_authorizations: pre_authorizations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.AuthorizeValidatorDefinitionRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthorizeValidatorDefinitionResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator_definition_auth.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.AuthorizeValidatorDefinitionResponse", len)?;
        if let Some(v) = self.validator_definition_auth.as_ref() {
            struct_ser.serialize_field("validatorDefinitionAuth", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizeValidatorDefinitionResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_definition_auth",
            "validatorDefinitionAuth",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorDefinitionAuth,
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
                            "validatorDefinitionAuth" | "validator_definition_auth" => Ok(GeneratedField::ValidatorDefinitionAuth),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizeValidatorDefinitionResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1.AuthorizeValidatorDefinitionResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizeValidatorDefinitionResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_definition_auth__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidatorDefinitionAuth => {
                            if validator_definition_auth__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDefinitionAuth"));
                            }
                            validator_definition_auth__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuthorizeValidatorDefinitionResponse {
                    validator_definition_auth: validator_definition_auth__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.AuthorizeValidatorDefinitionResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthorizeValidatorVoteRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator_vote.is_some() {
            len += 1;
        }
        if !self.pre_authorizations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.AuthorizeValidatorVoteRequest", len)?;
        if let Some(v) = self.validator_vote.as_ref() {
            struct_ser.serialize_field("validatorVote", v)?;
        }
        if !self.pre_authorizations.is_empty() {
            struct_ser.serialize_field("preAuthorizations", &self.pre_authorizations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizeValidatorVoteRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_vote",
            "validatorVote",
            "pre_authorizations",
            "preAuthorizations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorVote,
            PreAuthorizations,
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
                            "validatorVote" | "validator_vote" => Ok(GeneratedField::ValidatorVote),
                            "preAuthorizations" | "pre_authorizations" => Ok(GeneratedField::PreAuthorizations),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizeValidatorVoteRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1.AuthorizeValidatorVoteRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizeValidatorVoteRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_vote__ = None;
                let mut pre_authorizations__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidatorVote => {
                            if validator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorVote"));
                            }
                            validator_vote__ = map_.next_value()?;
                        }
                        GeneratedField::PreAuthorizations => {
                            if pre_authorizations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("preAuthorizations"));
                            }
                            pre_authorizations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuthorizeValidatorVoteRequest {
                    validator_vote: validator_vote__,
                    pre_authorizations: pre_authorizations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.AuthorizeValidatorVoteRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthorizeValidatorVoteResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.validator_vote_auth.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.AuthorizeValidatorVoteResponse", len)?;
        if let Some(v) = self.validator_vote_auth.as_ref() {
            struct_ser.serialize_field("validatorVoteAuth", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizeValidatorVoteResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validator_vote_auth",
            "validatorVoteAuth",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidatorVoteAuth,
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
                            "validatorVoteAuth" | "validator_vote_auth" => Ok(GeneratedField::ValidatorVoteAuth),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizeValidatorVoteResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1.AuthorizeValidatorVoteResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizeValidatorVoteResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validator_vote_auth__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidatorVoteAuth => {
                            if validator_vote_auth__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorVoteAuth"));
                            }
                            validator_vote_auth__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuthorizeValidatorVoteResponse {
                    validator_vote_auth: validator_vote_auth__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.AuthorizeValidatorVoteResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConfirmAddressRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.ConfirmAddressRequest", len)?;
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConfirmAddressRequest {
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
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConfirmAddressRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1.ConfirmAddressRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ConfirmAddressRequest, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ConfirmAddressRequest {
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.ConfirmAddressRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConfirmAddressResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.ConfirmAddressResponse", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConfirmAddressResponse {
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
            type Value = ConfirmAddressResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1.ConfirmAddressResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ConfirmAddressResponse, V::Error>
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
                Ok(ConfirmAddressResponse {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.ConfirmAddressResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportFullViewingKeyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.custody.v1.ExportFullViewingKeyRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportFullViewingKeyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Ok(GeneratedField::__SkipField__)
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportFullViewingKeyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1.ExportFullViewingKeyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportFullViewingKeyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ExportFullViewingKeyRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.ExportFullViewingKeyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportFullViewingKeyResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.ExportFullViewingKeyResponse", len)?;
        if let Some(v) = self.full_viewing_key.as_ref() {
            struct_ser.serialize_field("fullViewingKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportFullViewingKeyResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "full_viewing_key",
            "fullViewingKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FullViewingKey,
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
                            "fullViewingKey" | "full_viewing_key" => Ok(GeneratedField::FullViewingKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportFullViewingKeyResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.custody.v1.ExportFullViewingKeyResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportFullViewingKeyResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut full_viewing_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FullViewingKey => {
                            if full_viewing_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullViewingKey"));
                            }
                            full_viewing_key__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ExportFullViewingKeyResponse {
                    full_viewing_key: full_viewing_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.ExportFullViewingKeyResponse", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.PreAuthorization", len)?;
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
                            "ed25519" => Ok(GeneratedField::Ed25519),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.custody.v1.PreAuthorization")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PreAuthorization, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pre_authorization__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Ed25519 => {
                            if pre_authorization__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ed25519"));
                            }
                            pre_authorization__ = map_.next_value::<::std::option::Option<_>>()?.map(pre_authorization::PreAuthorization::Ed25519)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PreAuthorization {
                    pre_authorization: pre_authorization__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.PreAuthorization", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.custody.v1.PreAuthorization.Ed25519", len)?;
        if !self.vk.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("vk", pbjson::private::base64::encode(&self.vk).as_str())?;
        }
        if !self.sig.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
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
                            "vk" => Ok(GeneratedField::Vk),
                            "sig" => Ok(GeneratedField::Sig),
                            _ => Ok(GeneratedField::__SkipField__),
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
                formatter.write_str("struct penumbra.custody.v1.PreAuthorization.Ed25519")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<pre_authorization::Ed25519, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vk__ = None;
                let mut sig__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Vk => {
                            if vk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vk"));
                            }
                            vk__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Sig => {
                            if sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sig"));
                            }
                            sig__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(pre_authorization::Ed25519 {
                    vk: vk__.unwrap_or_default(),
                    sig: sig__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.custody.v1.PreAuthorization.Ed25519", FIELDS, GeneratedVisitor)
    }
}
