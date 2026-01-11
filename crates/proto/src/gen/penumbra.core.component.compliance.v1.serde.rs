impl serde::Serialize for ComplianceAnchorsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.ComplianceAnchorsRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ComplianceAnchorsRequest {
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
            type Value = ComplianceAnchorsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.ComplianceAnchorsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ComplianceAnchorsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ComplianceAnchorsRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.ComplianceAnchorsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ComplianceAnchorsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.user_tree_root.is_empty() {
            len += 1;
        }
        if !self.asset_tree_root.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.ComplianceAnchorsResponse", len)?;
        if !self.user_tree_root.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("userTreeRoot", pbjson::private::base64::encode(&self.user_tree_root).as_str())?;
        }
        if !self.asset_tree_root.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("assetTreeRoot", pbjson::private::base64::encode(&self.asset_tree_root).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ComplianceAnchorsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_tree_root",
            "userTreeRoot",
            "asset_tree_root",
            "assetTreeRoot",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserTreeRoot,
            AssetTreeRoot,
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
                            "userTreeRoot" | "user_tree_root" => Ok(GeneratedField::UserTreeRoot),
                            "assetTreeRoot" | "asset_tree_root" => Ok(GeneratedField::AssetTreeRoot),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ComplianceAnchorsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.ComplianceAnchorsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ComplianceAnchorsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_tree_root__ = None;
                let mut asset_tree_root__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserTreeRoot => {
                            if user_tree_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTreeRoot"));
                            }
                            user_tree_root__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AssetTreeRoot => {
                            if asset_tree_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetTreeRoot"));
                            }
                            asset_tree_root__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ComplianceAnchorsResponse {
                    user_tree_root: user_tree_root__.unwrap_or_default(),
                    asset_tree_root: asset_tree_root__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.ComplianceAnchorsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ComplianceAssetStatusRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.asset_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.ComplianceAssetStatusRequest", len)?;
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ComplianceAssetStatusRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_id",
            "assetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssetId,
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
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ComplianceAssetStatusRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.ComplianceAssetStatusRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ComplianceAssetStatusRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ComplianceAssetStatusRequest {
                    asset_id: asset_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.ComplianceAssetStatusRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ComplianceAssetStatusResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.asset_id.is_some() {
            len += 1;
        }
        if self.is_registered {
            len += 1;
        }
        if self.is_regulated {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.ComplianceAssetStatusResponse", len)?;
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        if self.is_registered {
            struct_ser.serialize_field("isRegistered", &self.is_registered)?;
        }
        if self.is_regulated {
            struct_ser.serialize_field("isRegulated", &self.is_regulated)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ComplianceAssetStatusResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_id",
            "assetId",
            "is_registered",
            "isRegistered",
            "is_regulated",
            "isRegulated",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssetId,
            IsRegistered,
            IsRegulated,
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
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            "isRegistered" | "is_registered" => Ok(GeneratedField::IsRegistered),
                            "isRegulated" | "is_regulated" => Ok(GeneratedField::IsRegulated),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ComplianceAssetStatusResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.ComplianceAssetStatusResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ComplianceAssetStatusResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_id__ = None;
                let mut is_registered__ = None;
                let mut is_regulated__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::IsRegistered => {
                            if is_registered__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isRegistered"));
                            }
                            is_registered__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IsRegulated => {
                            if is_regulated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isRegulated"));
                            }
                            is_regulated__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ComplianceAssetStatusResponse {
                    asset_id: asset_id__,
                    is_registered: is_registered__.unwrap_or_default(),
                    is_regulated: is_regulated__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.ComplianceAssetStatusResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ComplianceLeaf {
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
        if self.key.is_some() {
            len += 1;
        }
        if self.asset_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.ComplianceLeaf", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        if let Some(v) = self.key.as_ref() {
            struct_ser.serialize_field("key", v)?;
        }
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ComplianceLeaf {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "key",
            "asset_id",
            "assetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            Key,
            AssetId,
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
                            "key" => Ok(GeneratedField::Key),
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ComplianceLeaf;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.ComplianceLeaf")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ComplianceLeaf, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut key__ = None;
                let mut asset_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = map_.next_value()?;
                        }
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ComplianceLeaf {
                    address: address__,
                    key: key__,
                    asset_id: asset_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.ComplianceLeaf", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ComplianceMerkleProofsRequest {
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
        if self.asset_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.ComplianceMerkleProofsRequest", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ComplianceMerkleProofsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "asset_id",
            "assetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            AssetId,
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
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ComplianceMerkleProofsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.ComplianceMerkleProofsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ComplianceMerkleProofsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut asset_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ComplianceMerkleProofsRequest {
                    address: address__,
                    asset_id: asset_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.ComplianceMerkleProofsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ComplianceMerkleProofsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.user_registered {
            len += 1;
        }
        if self.asset_registered {
            len += 1;
        }
        if self.is_regulated {
            len += 1;
        }
        if self.compliance_path.is_some() {
            len += 1;
        }
        if self.compliance_position != 0 {
            len += 1;
        }
        if self.asset_path.is_some() {
            len += 1;
        }
        if self.asset_position != 0 {
            len += 1;
        }
        if !self.compliance_anchor.is_empty() {
            len += 1;
        }
        if !self.asset_anchor.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.ComplianceMerkleProofsResponse", len)?;
        if self.user_registered {
            struct_ser.serialize_field("userRegistered", &self.user_registered)?;
        }
        if self.asset_registered {
            struct_ser.serialize_field("assetRegistered", &self.asset_registered)?;
        }
        if self.is_regulated {
            struct_ser.serialize_field("isRegulated", &self.is_regulated)?;
        }
        if let Some(v) = self.compliance_path.as_ref() {
            struct_ser.serialize_field("compliancePath", v)?;
        }
        if self.compliance_position != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("compliancePosition", ToString::to_string(&self.compliance_position).as_str())?;
        }
        if let Some(v) = self.asset_path.as_ref() {
            struct_ser.serialize_field("assetPath", v)?;
        }
        if self.asset_position != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("assetPosition", ToString::to_string(&self.asset_position).as_str())?;
        }
        if !self.compliance_anchor.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("complianceAnchor", pbjson::private::base64::encode(&self.compliance_anchor).as_str())?;
        }
        if !self.asset_anchor.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("assetAnchor", pbjson::private::base64::encode(&self.asset_anchor).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ComplianceMerkleProofsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_registered",
            "userRegistered",
            "asset_registered",
            "assetRegistered",
            "is_regulated",
            "isRegulated",
            "compliance_path",
            "compliancePath",
            "compliance_position",
            "compliancePosition",
            "asset_path",
            "assetPath",
            "asset_position",
            "assetPosition",
            "compliance_anchor",
            "complianceAnchor",
            "asset_anchor",
            "assetAnchor",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserRegistered,
            AssetRegistered,
            IsRegulated,
            CompliancePath,
            CompliancePosition,
            AssetPath,
            AssetPosition,
            ComplianceAnchor,
            AssetAnchor,
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
                            "userRegistered" | "user_registered" => Ok(GeneratedField::UserRegistered),
                            "assetRegistered" | "asset_registered" => Ok(GeneratedField::AssetRegistered),
                            "isRegulated" | "is_regulated" => Ok(GeneratedField::IsRegulated),
                            "compliancePath" | "compliance_path" => Ok(GeneratedField::CompliancePath),
                            "compliancePosition" | "compliance_position" => Ok(GeneratedField::CompliancePosition),
                            "assetPath" | "asset_path" => Ok(GeneratedField::AssetPath),
                            "assetPosition" | "asset_position" => Ok(GeneratedField::AssetPosition),
                            "complianceAnchor" | "compliance_anchor" => Ok(GeneratedField::ComplianceAnchor),
                            "assetAnchor" | "asset_anchor" => Ok(GeneratedField::AssetAnchor),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ComplianceMerkleProofsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.ComplianceMerkleProofsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ComplianceMerkleProofsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_registered__ = None;
                let mut asset_registered__ = None;
                let mut is_regulated__ = None;
                let mut compliance_path__ = None;
                let mut compliance_position__ = None;
                let mut asset_path__ = None;
                let mut asset_position__ = None;
                let mut compliance_anchor__ = None;
                let mut asset_anchor__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserRegistered => {
                            if user_registered__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userRegistered"));
                            }
                            user_registered__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AssetRegistered => {
                            if asset_registered__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetRegistered"));
                            }
                            asset_registered__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IsRegulated => {
                            if is_regulated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isRegulated"));
                            }
                            is_regulated__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CompliancePath => {
                            if compliance_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compliancePath"));
                            }
                            compliance_path__ = map_.next_value()?;
                        }
                        GeneratedField::CompliancePosition => {
                            if compliance_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compliancePosition"));
                            }
                            compliance_position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AssetPath => {
                            if asset_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetPath"));
                            }
                            asset_path__ = map_.next_value()?;
                        }
                        GeneratedField::AssetPosition => {
                            if asset_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetPosition"));
                            }
                            asset_position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ComplianceAnchor => {
                            if compliance_anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("complianceAnchor"));
                            }
                            compliance_anchor__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AssetAnchor => {
                            if asset_anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetAnchor"));
                            }
                            asset_anchor__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ComplianceMerkleProofsResponse {
                    user_registered: user_registered__.unwrap_or_default(),
                    asset_registered: asset_registered__.unwrap_or_default(),
                    is_regulated: is_regulated__.unwrap_or_default(),
                    compliance_path: compliance_path__,
                    compliance_position: compliance_position__.unwrap_or_default(),
                    asset_path: asset_path__,
                    asset_position: asset_position__.unwrap_or_default(),
                    compliance_anchor: compliance_anchor__.unwrap_or_default(),
                    asset_anchor: asset_anchor__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.ComplianceMerkleProofsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ComplianceUserLeafRequest {
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
        if self.asset_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.ComplianceUserLeafRequest", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ComplianceUserLeafRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "asset_id",
            "assetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            AssetId,
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
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ComplianceUserLeafRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.ComplianceUserLeafRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ComplianceUserLeafRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut asset_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ComplianceUserLeafRequest {
                    address: address__,
                    asset_id: asset_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.ComplianceUserLeafRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ComplianceUserLeafResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.is_registered {
            len += 1;
        }
        if self.leaf.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.ComplianceUserLeafResponse", len)?;
        if self.is_registered {
            struct_ser.serialize_field("isRegistered", &self.is_registered)?;
        }
        if let Some(v) = self.leaf.as_ref() {
            struct_ser.serialize_field("leaf", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ComplianceUserLeafResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "is_registered",
            "isRegistered",
            "leaf",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IsRegistered,
            Leaf,
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
                            "isRegistered" | "is_registered" => Ok(GeneratedField::IsRegistered),
                            "leaf" => Ok(GeneratedField::Leaf),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ComplianceUserLeafResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.ComplianceUserLeafResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ComplianceUserLeafResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut is_registered__ = None;
                let mut leaf__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IsRegistered => {
                            if is_registered__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isRegistered"));
                            }
                            is_registered__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Leaf => {
                            if leaf__.is_some() {
                                return Err(serde::de::Error::duplicate_field("leaf"));
                            }
                            leaf__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ComplianceUserLeafResponse {
                    is_registered: is_registered__.unwrap_or_default(),
                    leaf: leaf__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.ComplianceUserLeafResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ComplianceViewingKey {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.ComplianceViewingKey", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ComplianceViewingKey {
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
                            "inner" => Ok(GeneratedField::Inner),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ComplianceViewingKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.ComplianceViewingKey")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ComplianceViewingKey, V::Error>
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ComplianceViewingKey {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.ComplianceViewingKey", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MerklePath {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.layers.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.MerklePath", len)?;
        if !self.layers.is_empty() {
            struct_ser.serialize_field("layers", &self.layers)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MerklePath {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "layers",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Layers,
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
                            "layers" => Ok(GeneratedField::Layers),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MerklePath;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.MerklePath")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MerklePath, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut layers__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Layers => {
                            if layers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("layers"));
                            }
                            layers__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MerklePath {
                    layers: layers__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.MerklePath", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MerklePathLayer {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.siblings.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.MerklePathLayer", len)?;
        if !self.siblings.is_empty() {
            struct_ser.serialize_field("siblings", &self.siblings.iter().map(pbjson::private::base64::encode).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MerklePathLayer {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "siblings",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Siblings,
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
                            "siblings" => Ok(GeneratedField::Siblings),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MerklePathLayer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.MerklePathLayer")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MerklePathLayer, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut siblings__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Siblings => {
                            if siblings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("siblings"));
                            }
                            siblings__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MerklePathLayer {
                    siblings: siblings__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.MerklePathLayer", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgRegisterAsset {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.asset_id.is_some() {
            len += 1;
        }
        if self.is_regulated {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.MsgRegisterAsset", len)?;
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        if self.is_regulated {
            struct_ser.serialize_field("isRegulated", &self.is_regulated)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgRegisterAsset {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_id",
            "assetId",
            "is_regulated",
            "isRegulated",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssetId,
            IsRegulated,
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
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            "isRegulated" | "is_regulated" => Ok(GeneratedField::IsRegulated),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgRegisterAsset;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.MsgRegisterAsset")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MsgRegisterAsset, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_id__ = None;
                let mut is_regulated__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::IsRegulated => {
                            if is_regulated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isRegulated"));
                            }
                            is_regulated__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MsgRegisterAsset {
                    asset_id: asset_id__,
                    is_regulated: is_regulated__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.MsgRegisterAsset", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgRegisterUser {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.leaf.is_some() {
            len += 1;
        }
        if !self.signature.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.compliance.v1.MsgRegisterUser", len)?;
        if let Some(v) = self.leaf.as_ref() {
            struct_ser.serialize_field("leaf", v)?;
        }
        if !self.signature.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("signature", pbjson::private::base64::encode(&self.signature).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgRegisterUser {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "leaf",
            "signature",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Leaf,
            Signature,
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
                            "leaf" => Ok(GeneratedField::Leaf),
                            "signature" => Ok(GeneratedField::Signature),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgRegisterUser;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.compliance.v1.MsgRegisterUser")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MsgRegisterUser, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut leaf__ = None;
                let mut signature__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Leaf => {
                            if leaf__.is_some() {
                                return Err(serde::de::Error::duplicate_field("leaf"));
                            }
                            leaf__ = map_.next_value()?;
                        }
                        GeneratedField::Signature => {
                            if signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signature"));
                            }
                            signature__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MsgRegisterUser {
                    leaf: leaf__,
                    signature: signature__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.compliance.v1.MsgRegisterUser", FIELDS, GeneratedVisitor)
    }
}
