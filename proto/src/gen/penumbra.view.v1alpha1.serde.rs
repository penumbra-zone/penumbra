impl serde::Serialize for AddressByIndexRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AddressByIndexRequest", len)?;
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AddressByIndexRequest {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AddressByIndexRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.AddressByIndexRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AddressByIndexRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address_index__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map.next_value()?;
                        }
                    }
                }
                Ok(AddressByIndexRequest {
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AddressByIndexRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AddressByIndexResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AddressByIndexResponse", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AddressByIndexResponse {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AddressByIndexResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.AddressByIndexResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AddressByIndexResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map.next_value()?;
                        }
                    }
                }
                Ok(AddressByIndexResponse {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AddressByIndexResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AssetsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AssetsRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetsRequest {
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
            type Value = AssetsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.AssetsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AssetsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(AssetsRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AssetsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AssetsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.asset.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.AssetsResponse", len)?;
        if let Some(v) = self.asset.as_ref() {
            struct_ser.serialize_field("asset", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Asset,
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
                            "asset" => Ok(GeneratedField::Asset),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AssetsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.AssetsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AssetsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Asset => {
                            if asset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asset"));
                            }
                            asset__ = map.next_value()?;
                        }
                    }
                }
                Ok(AssetsResponse {
                    asset: asset__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.AssetsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BalanceByAddressRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.BalanceByAddressRequest", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BalanceByAddressRequest {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BalanceByAddressRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.BalanceByAddressRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BalanceByAddressRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map.next_value()?;
                        }
                    }
                }
                Ok(BalanceByAddressRequest {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.BalanceByAddressRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for BalanceByAddressResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.asset.is_some() {
            len += 1;
        }
        if self.amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.BalanceByAddressResponse", len)?;
        if let Some(v) = self.asset.as_ref() {
            struct_ser.serialize_field("asset", v)?;
        }
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for BalanceByAddressResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset",
            "amount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Asset,
            Amount,
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
                            "asset" => Ok(GeneratedField::Asset),
                            "amount" => Ok(GeneratedField::Amount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = BalanceByAddressResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.BalanceByAddressResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<BalanceByAddressResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset__ = None;
                let mut amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Asset => {
                            if asset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asset"));
                            }
                            asset__ = map.next_value()?;
                        }
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map.next_value()?;
                        }
                    }
                }
                Ok(BalanceByAddressResponse {
                    asset: asset__,
                    amount: amount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.BalanceByAddressResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ChainParametersRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.ChainParametersRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChainParametersRequest {
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
            type Value = ChainParametersRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.ChainParametersRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ChainParametersRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ChainParametersRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.ChainParametersRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ChainParametersResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.parameters.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.ChainParametersResponse", len)?;
        if let Some(v) = self.parameters.as_ref() {
            struct_ser.serialize_field("parameters", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChainParametersResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parameters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Parameters,
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
                            "parameters" => Ok(GeneratedField::Parameters),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ChainParametersResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.ChainParametersResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ChainParametersResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut parameters__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Parameters => {
                            if parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameters"));
                            }
                            parameters__ = map.next_value()?;
                        }
                    }
                }
                Ok(ChainParametersResponse {
                    parameters: parameters__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.ChainParametersResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EphemeralAddressRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.EphemeralAddressRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EphemeralAddressRequest {
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
            type Value = EphemeralAddressRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.EphemeralAddressRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EphemeralAddressRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(EphemeralAddressRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.EphemeralAddressRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EphemeralAddressResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.EphemeralAddressResponse", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EphemeralAddressResponse {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EphemeralAddressResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.EphemeralAddressResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EphemeralAddressResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map.next_value()?;
                        }
                    }
                }
                Ok(EphemeralAddressResponse {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.EphemeralAddressResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FmdParametersRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.FMDParametersRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FmdParametersRequest {
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
            type Value = FmdParametersRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.FMDParametersRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FmdParametersRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(FmdParametersRequest {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.FMDParametersRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FmdParametersResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.parameters.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.FMDParametersResponse", len)?;
        if let Some(v) = self.parameters.as_ref() {
            struct_ser.serialize_field("parameters", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FmdParametersResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parameters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Parameters,
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
                            "parameters" => Ok(GeneratedField::Parameters),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FmdParametersResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.FMDParametersResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FmdParametersResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut parameters__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Parameters => {
                            if parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameters"));
                            }
                            parameters__ = map.next_value()?;
                        }
                    }
                }
                Ok(FmdParametersResponse {
                    parameters: parameters__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.FMDParametersResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IndexByAddressRequest {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.IndexByAddressRequest", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IndexByAddressRequest {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = IndexByAddressRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.IndexByAddressRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<IndexByAddressRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map.next_value()?;
                        }
                    }
                }
                Ok(IndexByAddressRequest {
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.IndexByAddressRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for IndexByAddressResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.IndexByAddressResponse", len)?;
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for IndexByAddressResponse {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = IndexByAddressResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.IndexByAddressResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<IndexByAddressResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address_index__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map.next_value()?;
                        }
                    }
                }
                Ok(IndexByAddressResponse {
                    address_index: address_index__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.IndexByAddressResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NoteByCommitmentRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_commitment.is_some() {
            len += 1;
        }
        if self.await_detection {
            len += 1;
        }
        if self.account_id.is_some() {
            len += 1;
        }
        if self.token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NoteByCommitmentRequest", len)?;
        if let Some(v) = self.note_commitment.as_ref() {
            struct_ser.serialize_field("noteCommitment", v)?;
        }
        if self.await_detection {
            struct_ser.serialize_field("awaitDetection", &self.await_detection)?;
        }
        if let Some(v) = self.account_id.as_ref() {
            struct_ser.serialize_field("accountId", v)?;
        }
        if let Some(v) = self.token.as_ref() {
            struct_ser.serialize_field("token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NoteByCommitmentRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_commitment",
            "noteCommitment",
            "await_detection",
            "awaitDetection",
            "account_id",
            "accountId",
            "token",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteCommitment,
            AwaitDetection,
            AccountId,
            Token,
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
                            "noteCommitment" | "note_commitment" => Ok(GeneratedField::NoteCommitment),
                            "awaitDetection" | "await_detection" => Ok(GeneratedField::AwaitDetection),
                            "accountId" | "account_id" => Ok(GeneratedField::AccountId),
                            "token" => Ok(GeneratedField::Token),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NoteByCommitmentRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.NoteByCommitmentRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NoteByCommitmentRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_commitment__ = None;
                let mut await_detection__ = None;
                let mut account_id__ = None;
                let mut token__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NoteCommitment => {
                            if note_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitment"));
                            }
                            note_commitment__ = map.next_value()?;
                        }
                        GeneratedField::AwaitDetection => {
                            if await_detection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awaitDetection"));
                            }
                            await_detection__ = Some(map.next_value()?);
                        }
                        GeneratedField::AccountId => {
                            if account_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountId"));
                            }
                            account_id__ = map.next_value()?;
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = map.next_value()?;
                        }
                    }
                }
                Ok(NoteByCommitmentRequest {
                    note_commitment: note_commitment__,
                    await_detection: await_detection__.unwrap_or_default(),
                    account_id: account_id__,
                    token: token__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NoteByCommitmentRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NoteByCommitmentResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.spendable_note.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NoteByCommitmentResponse", len)?;
        if let Some(v) = self.spendable_note.as_ref() {
            struct_ser.serialize_field("spendableNote", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NoteByCommitmentResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spendable_note",
            "spendableNote",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SpendableNote,
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
                            "spendableNote" | "spendable_note" => Ok(GeneratedField::SpendableNote),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NoteByCommitmentResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.NoteByCommitmentResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NoteByCommitmentResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spendable_note__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SpendableNote => {
                            if spendable_note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendableNote"));
                            }
                            spendable_note__ = map.next_value()?;
                        }
                    }
                }
                Ok(NoteByCommitmentResponse {
                    spendable_note: spendable_note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NoteByCommitmentResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NotesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.include_spent {
            len += 1;
        }
        if self.asset_id.is_some() {
            len += 1;
        }
        if self.address_index.is_some() {
            len += 1;
        }
        if self.amount_to_spend != 0 {
            len += 1;
        }
        if self.account_id.is_some() {
            len += 1;
        }
        if self.token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NotesRequest", len)?;
        if self.include_spent {
            struct_ser.serialize_field("includeSpent", &self.include_spent)?;
        }
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        if self.amount_to_spend != 0 {
            struct_ser.serialize_field("amountToSpend", ToString::to_string(&self.amount_to_spend).as_str())?;
        }
        if let Some(v) = self.account_id.as_ref() {
            struct_ser.serialize_field("accountId", v)?;
        }
        if let Some(v) = self.token.as_ref() {
            struct_ser.serialize_field("token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NotesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "include_spent",
            "includeSpent",
            "asset_id",
            "assetId",
            "address_index",
            "addressIndex",
            "amount_to_spend",
            "amountToSpend",
            "account_id",
            "accountId",
            "token",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IncludeSpent,
            AssetId,
            AddressIndex,
            AmountToSpend,
            AccountId,
            Token,
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
                            "includeSpent" | "include_spent" => Ok(GeneratedField::IncludeSpent),
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            "amountToSpend" | "amount_to_spend" => Ok(GeneratedField::AmountToSpend),
                            "accountId" | "account_id" => Ok(GeneratedField::AccountId),
                            "token" => Ok(GeneratedField::Token),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NotesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.NotesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NotesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut include_spent__ = None;
                let mut asset_id__ = None;
                let mut address_index__ = None;
                let mut amount_to_spend__ = None;
                let mut account_id__ = None;
                let mut token__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::IncludeSpent => {
                            if include_spent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeSpent"));
                            }
                            include_spent__ = Some(map.next_value()?);
                        }
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map.next_value()?;
                        }
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map.next_value()?;
                        }
                        GeneratedField::AmountToSpend => {
                            if amount_to_spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amountToSpend"));
                            }
                            amount_to_spend__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AccountId => {
                            if account_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountId"));
                            }
                            account_id__ = map.next_value()?;
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = map.next_value()?;
                        }
                    }
                }
                Ok(NotesRequest {
                    include_spent: include_spent__.unwrap_or_default(),
                    asset_id: asset_id__,
                    address_index: address_index__,
                    amount_to_spend: amount_to_spend__.unwrap_or_default(),
                    account_id: account_id__,
                    token: token__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NotesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NotesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_record.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NotesResponse", len)?;
        if let Some(v) = self.note_record.as_ref() {
            struct_ser.serialize_field("noteRecord", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NotesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_record",
            "noteRecord",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteRecord,
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
                            "noteRecord" | "note_record" => Ok(GeneratedField::NoteRecord),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NotesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.NotesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NotesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_record__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NoteRecord => {
                            if note_record__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteRecord"));
                            }
                            note_record__ = map.next_value()?;
                        }
                    }
                }
                Ok(NotesResponse {
                    note_record: note_record__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NotesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NullifierStatusRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.await_detection {
            len += 1;
        }
        if self.account_id.is_some() {
            len += 1;
        }
        if self.token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NullifierStatusRequest", len)?;
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if self.await_detection {
            struct_ser.serialize_field("awaitDetection", &self.await_detection)?;
        }
        if let Some(v) = self.account_id.as_ref() {
            struct_ser.serialize_field("accountId", v)?;
        }
        if let Some(v) = self.token.as_ref() {
            struct_ser.serialize_field("token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NullifierStatusRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "nullifier",
            "await_detection",
            "awaitDetection",
            "account_id",
            "accountId",
            "token",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Nullifier,
            AwaitDetection,
            AccountId,
            Token,
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
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "awaitDetection" | "await_detection" => Ok(GeneratedField::AwaitDetection),
                            "accountId" | "account_id" => Ok(GeneratedField::AccountId),
                            "token" => Ok(GeneratedField::Token),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NullifierStatusRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.NullifierStatusRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NullifierStatusRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut nullifier__ = None;
                let mut await_detection__ = None;
                let mut account_id__ = None;
                let mut token__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map.next_value()?;
                        }
                        GeneratedField::AwaitDetection => {
                            if await_detection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awaitDetection"));
                            }
                            await_detection__ = Some(map.next_value()?);
                        }
                        GeneratedField::AccountId => {
                            if account_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountId"));
                            }
                            account_id__ = map.next_value()?;
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = map.next_value()?;
                        }
                    }
                }
                Ok(NullifierStatusRequest {
                    nullifier: nullifier__,
                    await_detection: await_detection__.unwrap_or_default(),
                    account_id: account_id__,
                    token: token__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NullifierStatusRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NullifierStatusResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.spent {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.NullifierStatusResponse", len)?;
        if self.spent {
            struct_ser.serialize_field("spent", &self.spent)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NullifierStatusResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spent",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spent,
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
                            "spent" => Ok(GeneratedField::Spent),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NullifierStatusResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.NullifierStatusResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NullifierStatusResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spent__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Spent => {
                            if spent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spent"));
                            }
                            spent__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(NullifierStatusResponse {
                    spent: spent__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.NullifierStatusResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendableNoteRecord {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_commitment.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        if self.address_index.is_some() {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.height_created != 0 {
            len += 1;
        }
        if self.height_spent.is_some() {
            len += 1;
        }
        if self.position != 0 {
            len += 1;
        }
        if self.source.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.SpendableNoteRecord", len)?;
        if let Some(v) = self.note_commitment.as_ref() {
            struct_ser.serialize_field("noteCommitment", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if let Some(v) = self.address_index.as_ref() {
            struct_ser.serialize_field("addressIndex", v)?;
        }
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if self.height_created != 0 {
            struct_ser.serialize_field("heightCreated", ToString::to_string(&self.height_created).as_str())?;
        }
        if let Some(v) = self.height_spent.as_ref() {
            struct_ser.serialize_field("heightSpent", ToString::to_string(&v).as_str())?;
        }
        if self.position != 0 {
            struct_ser.serialize_field("position", ToString::to_string(&self.position).as_str())?;
        }
        if let Some(v) = self.source.as_ref() {
            struct_ser.serialize_field("source", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendableNoteRecord {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_commitment",
            "noteCommitment",
            "note",
            "address_index",
            "addressIndex",
            "nullifier",
            "height_created",
            "heightCreated",
            "height_spent",
            "heightSpent",
            "position",
            "source",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteCommitment,
            Note,
            AddressIndex,
            Nullifier,
            HeightCreated,
            HeightSpent,
            Position,
            Source,
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
                            "noteCommitment" | "note_commitment" => Ok(GeneratedField::NoteCommitment),
                            "note" => Ok(GeneratedField::Note),
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "heightCreated" | "height_created" => Ok(GeneratedField::HeightCreated),
                            "heightSpent" | "height_spent" => Ok(GeneratedField::HeightSpent),
                            "position" => Ok(GeneratedField::Position),
                            "source" => Ok(GeneratedField::Source),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendableNoteRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.SpendableNoteRecord")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SpendableNoteRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_commitment__ = None;
                let mut note__ = None;
                let mut address_index__ = None;
                let mut nullifier__ = None;
                let mut height_created__ = None;
                let mut height_spent__ = None;
                let mut position__ = None;
                let mut source__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NoteCommitment => {
                            if note_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitment"));
                            }
                            note_commitment__ = map.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map.next_value()?;
                        }
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = map.next_value()?;
                        }
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map.next_value()?;
                        }
                        GeneratedField::HeightCreated => {
                            if height_created__.is_some() {
                                return Err(serde::de::Error::duplicate_field("heightCreated"));
                            }
                            height_created__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::HeightSpent => {
                            if height_spent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("heightSpent"));
                            }
                            height_spent__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Source => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source__ = map.next_value()?;
                        }
                    }
                }
                Ok(SpendableNoteRecord {
                    note_commitment: note_commitment__,
                    note: note__,
                    address_index: address_index__,
                    nullifier: nullifier__,
                    height_created: height_created__.unwrap_or_default(),
                    height_spent: height_spent__,
                    position: position__.unwrap_or_default(),
                    source: source__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.SpendableNoteRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StatusRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.account_id.is_some() {
            len += 1;
        }
        if self.token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.StatusRequest", len)?;
        if let Some(v) = self.account_id.as_ref() {
            struct_ser.serialize_field("accountId", v)?;
        }
        if let Some(v) = self.token.as_ref() {
            struct_ser.serialize_field("token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StatusRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "account_id",
            "accountId",
            "token",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AccountId,
            Token,
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
                            "accountId" | "account_id" => Ok(GeneratedField::AccountId),
                            "token" => Ok(GeneratedField::Token),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StatusRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.StatusRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StatusRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut account_id__ = None;
                let mut token__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AccountId => {
                            if account_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountId"));
                            }
                            account_id__ = map.next_value()?;
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = map.next_value()?;
                        }
                    }
                }
                Ok(StatusRequest {
                    account_id: account_id__,
                    token: token__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.StatusRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StatusResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.sync_height != 0 {
            len += 1;
        }
        if self.catching_up {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.StatusResponse", len)?;
        if self.sync_height != 0 {
            struct_ser.serialize_field("syncHeight", ToString::to_string(&self.sync_height).as_str())?;
        }
        if self.catching_up {
            struct_ser.serialize_field("catchingUp", &self.catching_up)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StatusResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sync_height",
            "syncHeight",
            "catching_up",
            "catchingUp",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SyncHeight,
            CatchingUp,
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
                            "syncHeight" | "sync_height" => Ok(GeneratedField::SyncHeight),
                            "catchingUp" | "catching_up" => Ok(GeneratedField::CatchingUp),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StatusResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.StatusResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StatusResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sync_height__ = None;
                let mut catching_up__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SyncHeight => {
                            if sync_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("syncHeight"));
                            }
                            sync_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CatchingUp => {
                            if catching_up__.is_some() {
                                return Err(serde::de::Error::duplicate_field("catchingUp"));
                            }
                            catching_up__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(StatusResponse {
                    sync_height: sync_height__.unwrap_or_default(),
                    catching_up: catching_up__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.StatusResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StatusStreamRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.account_id.is_some() {
            len += 1;
        }
        if self.token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.StatusStreamRequest", len)?;
        if let Some(v) = self.account_id.as_ref() {
            struct_ser.serialize_field("accountId", v)?;
        }
        if let Some(v) = self.token.as_ref() {
            struct_ser.serialize_field("token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StatusStreamRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "account_id",
            "accountId",
            "token",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AccountId,
            Token,
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
                            "accountId" | "account_id" => Ok(GeneratedField::AccountId),
                            "token" => Ok(GeneratedField::Token),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StatusStreamRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.StatusStreamRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StatusStreamRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut account_id__ = None;
                let mut token__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AccountId => {
                            if account_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountId"));
                            }
                            account_id__ = map.next_value()?;
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = map.next_value()?;
                        }
                    }
                }
                Ok(StatusStreamRequest {
                    account_id: account_id__,
                    token: token__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.StatusStreamRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StatusStreamResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.latest_known_block_height != 0 {
            len += 1;
        }
        if self.sync_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.StatusStreamResponse", len)?;
        if self.latest_known_block_height != 0 {
            struct_ser.serialize_field("latestKnownBlockHeight", ToString::to_string(&self.latest_known_block_height).as_str())?;
        }
        if self.sync_height != 0 {
            struct_ser.serialize_field("syncHeight", ToString::to_string(&self.sync_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StatusStreamResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "latest_known_block_height",
            "latestKnownBlockHeight",
            "sync_height",
            "syncHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LatestKnownBlockHeight,
            SyncHeight,
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
                            "latestKnownBlockHeight" | "latest_known_block_height" => Ok(GeneratedField::LatestKnownBlockHeight),
                            "syncHeight" | "sync_height" => Ok(GeneratedField::SyncHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StatusStreamResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.StatusStreamResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StatusStreamResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut latest_known_block_height__ = None;
                let mut sync_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::LatestKnownBlockHeight => {
                            if latest_known_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("latestKnownBlockHeight"));
                            }
                            latest_known_block_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SyncHeight => {
                            if sync_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("syncHeight"));
                            }
                            sync_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(StatusStreamResponse {
                    latest_known_block_height: latest_known_block_height__.unwrap_or_default(),
                    sync_height: sync_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.StatusStreamResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapByCommitmentRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_commitment.is_some() {
            len += 1;
        }
        if self.await_detection {
            len += 1;
        }
        if self.account_id.is_some() {
            len += 1;
        }
        if self.token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.SwapByCommitmentRequest", len)?;
        if let Some(v) = self.swap_commitment.as_ref() {
            struct_ser.serialize_field("swapCommitment", v)?;
        }
        if self.await_detection {
            struct_ser.serialize_field("awaitDetection", &self.await_detection)?;
        }
        if let Some(v) = self.account_id.as_ref() {
            struct_ser.serialize_field("accountId", v)?;
        }
        if let Some(v) = self.token.as_ref() {
            struct_ser.serialize_field("token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapByCommitmentRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_commitment",
            "swapCommitment",
            "await_detection",
            "awaitDetection",
            "account_id",
            "accountId",
            "token",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapCommitment,
            AwaitDetection,
            AccountId,
            Token,
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
                            "swapCommitment" | "swap_commitment" => Ok(GeneratedField::SwapCommitment),
                            "awaitDetection" | "await_detection" => Ok(GeneratedField::AwaitDetection),
                            "accountId" | "account_id" => Ok(GeneratedField::AccountId),
                            "token" => Ok(GeneratedField::Token),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapByCommitmentRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.SwapByCommitmentRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapByCommitmentRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_commitment__ = None;
                let mut await_detection__ = None;
                let mut account_id__ = None;
                let mut token__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SwapCommitment => {
                            if swap_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapCommitment"));
                            }
                            swap_commitment__ = map.next_value()?;
                        }
                        GeneratedField::AwaitDetection => {
                            if await_detection__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awaitDetection"));
                            }
                            await_detection__ = Some(map.next_value()?);
                        }
                        GeneratedField::AccountId => {
                            if account_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountId"));
                            }
                            account_id__ = map.next_value()?;
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = map.next_value()?;
                        }
                    }
                }
                Ok(SwapByCommitmentRequest {
                    swap_commitment: swap_commitment__,
                    await_detection: await_detection__.unwrap_or_default(),
                    account_id: account_id__,
                    token: token__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.SwapByCommitmentRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapByCommitmentResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.SwapByCommitmentResponse", len)?;
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapByCommitmentResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Swap,
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
                            "swap" => Ok(GeneratedField::Swap),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapByCommitmentResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.SwapByCommitmentResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapByCommitmentResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = map.next_value()?;
                        }
                    }
                }
                Ok(SwapByCommitmentResponse {
                    swap: swap__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.SwapByCommitmentResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SwapRecord {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.swap_commitment.is_some() {
            len += 1;
        }
        if self.swap.is_some() {
            len += 1;
        }
        if self.position != 0 {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.output_data.is_some() {
            len += 1;
        }
        if self.height_claimed.is_some() {
            len += 1;
        }
        if self.source.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.SwapRecord", len)?;
        if let Some(v) = self.swap_commitment.as_ref() {
            struct_ser.serialize_field("swapCommitment", v)?;
        }
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        if self.position != 0 {
            struct_ser.serialize_field("position", ToString::to_string(&self.position).as_str())?;
        }
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if let Some(v) = self.output_data.as_ref() {
            struct_ser.serialize_field("outputData", v)?;
        }
        if let Some(v) = self.height_claimed.as_ref() {
            struct_ser.serialize_field("heightClaimed", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.source.as_ref() {
            struct_ser.serialize_field("source", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SwapRecord {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swap_commitment",
            "swapCommitment",
            "swap",
            "position",
            "nullifier",
            "output_data",
            "outputData",
            "height_claimed",
            "heightClaimed",
            "source",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SwapCommitment,
            Swap,
            Position,
            Nullifier,
            OutputData,
            HeightClaimed,
            Source,
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
                            "swapCommitment" | "swap_commitment" => Ok(GeneratedField::SwapCommitment),
                            "swap" => Ok(GeneratedField::Swap),
                            "position" => Ok(GeneratedField::Position),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "outputData" | "output_data" => Ok(GeneratedField::OutputData),
                            "heightClaimed" | "height_claimed" => Ok(GeneratedField::HeightClaimed),
                            "source" => Ok(GeneratedField::Source),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SwapRecord;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.SwapRecord")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SwapRecord, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swap_commitment__ = None;
                let mut swap__ = None;
                let mut position__ = None;
                let mut nullifier__ = None;
                let mut output_data__ = None;
                let mut height_claimed__ = None;
                let mut source__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SwapCommitment => {
                            if swap_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapCommitment"));
                            }
                            swap_commitment__ = map.next_value()?;
                        }
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = map.next_value()?;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map.next_value()?;
                        }
                        GeneratedField::OutputData => {
                            if output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputData"));
                            }
                            output_data__ = map.next_value()?;
                        }
                        GeneratedField::HeightClaimed => {
                            if height_claimed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("heightClaimed"));
                            }
                            height_claimed__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Source => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source__ = map.next_value()?;
                        }
                    }
                }
                Ok(SwapRecord {
                    swap_commitment: swap_commitment__,
                    swap: swap__,
                    position: position__.unwrap_or_default(),
                    nullifier: nullifier__,
                    output_data: output_data__,
                    height_claimed: height_claimed__,
                    source: source__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.SwapRecord", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionByHashRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tx_hash.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionByHashRequest", len)?;
        if !self.tx_hash.is_empty() {
            struct_ser.serialize_field("txHash", pbjson::private::base64::encode(&self.tx_hash).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionByHashRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tx_hash",
            "txHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TxHash,
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
                            "txHash" | "tx_hash" => Ok(GeneratedField::TxHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionByHashRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionByHashRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionByHashRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tx_hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TxHash => {
                            if tx_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txHash"));
                            }
                            tx_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(TransactionByHashRequest {
                    tx_hash: tx_hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionByHashRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionByHashResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tx.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionByHashResponse", len)?;
        if let Some(v) = self.tx.as_ref() {
            struct_ser.serialize_field("tx", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionByHashResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tx",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tx,
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
                            "tx" => Ok(GeneratedField::Tx),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionByHashResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionByHashResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionByHashResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tx__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Tx => {
                            if tx__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tx"));
                            }
                            tx__ = map.next_value()?;
                        }
                    }
                }
                Ok(TransactionByHashResponse {
                    tx: tx__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionByHashResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionHashesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_height.is_some() {
            len += 1;
        }
        if self.end_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionHashesRequest", len)?;
        if let Some(v) = self.start_height.as_ref() {
            struct_ser.serialize_field("startHeight", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.end_height.as_ref() {
            struct_ser.serialize_field("endHeight", ToString::to_string(&v).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionHashesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_height",
            "startHeight",
            "end_height",
            "endHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartHeight,
            EndHeight,
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
                            "startHeight" | "start_height" => Ok(GeneratedField::StartHeight),
                            "endHeight" | "end_height" => Ok(GeneratedField::EndHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionHashesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionHashesRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionHashesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_height__ = None;
                let mut end_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::StartHeight => {
                            if start_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startHeight"));
                            }
                            start_height__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::EndHeight => {
                            if end_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endHeight"));
                            }
                            end_height__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(TransactionHashesRequest {
                    start_height: start_height__,
                    end_height: end_height__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionHashesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionHashesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block_height != 0 {
            len += 1;
        }
        if !self.tx_hash.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionHashesResponse", len)?;
        if self.block_height != 0 {
            struct_ser.serialize_field("blockHeight", ToString::to_string(&self.block_height).as_str())?;
        }
        if !self.tx_hash.is_empty() {
            struct_ser.serialize_field("txHash", pbjson::private::base64::encode(&self.tx_hash).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionHashesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_height",
            "blockHeight",
            "tx_hash",
            "txHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockHeight,
            TxHash,
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
                            "blockHeight" | "block_height" => Ok(GeneratedField::BlockHeight),
                            "txHash" | "tx_hash" => Ok(GeneratedField::TxHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionHashesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionHashesResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionHashesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_height__ = None;
                let mut tx_hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BlockHeight => {
                            if block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHeight"));
                            }
                            block_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TxHash => {
                            if tx_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txHash"));
                            }
                            tx_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(TransactionHashesResponse {
                    block_height: block_height__.unwrap_or_default(),
                    tx_hash: tx_hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionHashesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionPerspectiveRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tx_hash.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPerspectiveRequest", len)?;
        if !self.tx_hash.is_empty() {
            struct_ser.serialize_field("txHash", pbjson::private::base64::encode(&self.tx_hash).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionPerspectiveRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tx_hash",
            "txHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TxHash,
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
                            "txHash" | "tx_hash" => Ok(GeneratedField::TxHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionPerspectiveRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPerspectiveRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionPerspectiveRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tx_hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TxHash => {
                            if tx_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txHash"));
                            }
                            tx_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(TransactionPerspectiveRequest {
                    tx_hash: tx_hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPerspectiveRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionPerspectiveResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.txp.is_some() {
            len += 1;
        }
        if self.tx.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPerspectiveResponse", len)?;
        if let Some(v) = self.txp.as_ref() {
            struct_ser.serialize_field("txp", v)?;
        }
        if let Some(v) = self.tx.as_ref() {
            struct_ser.serialize_field("tx", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionPerspectiveResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "txp",
            "tx",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Txp,
            Tx,
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
                            "txp" => Ok(GeneratedField::Txp),
                            "tx" => Ok(GeneratedField::Tx),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionPerspectiveResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPerspectiveResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionPerspectiveResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut txp__ = None;
                let mut tx__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Txp => {
                            if txp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txp"));
                            }
                            txp__ = map.next_value()?;
                        }
                        GeneratedField::Tx => {
                            if tx__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tx"));
                            }
                            tx__ = map.next_value()?;
                        }
                    }
                }
                Ok(TransactionPerspectiveResponse {
                    txp: txp__,
                    tx: tx__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPerspectiveResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionPlannerRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.expiry_height != 0 {
            len += 1;
        }
        if self.fee.is_some() {
            len += 1;
        }
        if !self.memo.is_empty() {
            len += 1;
        }
        if self.account_id.is_some() {
            len += 1;
        }
        if self.token.is_some() {
            len += 1;
        }
        if !self.outputs.is_empty() {
            len += 1;
        }
        if !self.swaps.is_empty() {
            len += 1;
        }
        if !self.delegations.is_empty() {
            len += 1;
        }
        if !self.undelegations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest", len)?;
        if self.expiry_height != 0 {
            struct_ser.serialize_field("expiryHeight", ToString::to_string(&self.expiry_height).as_str())?;
        }
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        if !self.memo.is_empty() {
            struct_ser.serialize_field("memo", &self.memo)?;
        }
        if let Some(v) = self.account_id.as_ref() {
            struct_ser.serialize_field("accountId", v)?;
        }
        if let Some(v) = self.token.as_ref() {
            struct_ser.serialize_field("token", v)?;
        }
        if !self.outputs.is_empty() {
            struct_ser.serialize_field("outputs", &self.outputs)?;
        }
        if !self.swaps.is_empty() {
            struct_ser.serialize_field("swaps", &self.swaps)?;
        }
        if !self.delegations.is_empty() {
            struct_ser.serialize_field("delegations", &self.delegations)?;
        }
        if !self.undelegations.is_empty() {
            struct_ser.serialize_field("undelegations", &self.undelegations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionPlannerRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "expiry_height",
            "expiryHeight",
            "fee",
            "memo",
            "account_id",
            "accountId",
            "token",
            "outputs",
            "swaps",
            "delegations",
            "undelegations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExpiryHeight,
            Fee,
            Memo,
            AccountId,
            Token,
            Outputs,
            Swaps,
            Delegations,
            Undelegations,
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
                            "expiryHeight" | "expiry_height" => Ok(GeneratedField::ExpiryHeight),
                            "fee" => Ok(GeneratedField::Fee),
                            "memo" => Ok(GeneratedField::Memo),
                            "accountId" | "account_id" => Ok(GeneratedField::AccountId),
                            "token" => Ok(GeneratedField::Token),
                            "outputs" => Ok(GeneratedField::Outputs),
                            "swaps" => Ok(GeneratedField::Swaps),
                            "delegations" => Ok(GeneratedField::Delegations),
                            "undelegations" => Ok(GeneratedField::Undelegations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionPlannerRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionPlannerRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut expiry_height__ = None;
                let mut fee__ = None;
                let mut memo__ = None;
                let mut account_id__ = None;
                let mut token__ = None;
                let mut outputs__ = None;
                let mut swaps__ = None;
                let mut delegations__ = None;
                let mut undelegations__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ExpiryHeight => {
                            if expiry_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expiryHeight"));
                            }
                            expiry_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map.next_value()?;
                        }
                        GeneratedField::Memo => {
                            if memo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("memo"));
                            }
                            memo__ = Some(map.next_value()?);
                        }
                        GeneratedField::AccountId => {
                            if account_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountId"));
                            }
                            account_id__ = map.next_value()?;
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = map.next_value()?;
                        }
                        GeneratedField::Outputs => {
                            if outputs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputs"));
                            }
                            outputs__ = Some(map.next_value()?);
                        }
                        GeneratedField::Swaps => {
                            if swaps__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swaps"));
                            }
                            swaps__ = Some(map.next_value()?);
                        }
                        GeneratedField::Delegations => {
                            if delegations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegations"));
                            }
                            delegations__ = Some(map.next_value()?);
                        }
                        GeneratedField::Undelegations => {
                            if undelegations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegations"));
                            }
                            undelegations__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TransactionPlannerRequest {
                    expiry_height: expiry_height__.unwrap_or_default(),
                    fee: fee__,
                    memo: memo__.unwrap_or_default(),
                    account_id: account_id__,
                    token: token__,
                    outputs: outputs__.unwrap_or_default(),
                    swaps: swaps__.unwrap_or_default(),
                    delegations: delegations__.unwrap_or_default(),
                    undelegations: undelegations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::Delegate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.amount.is_some() {
            len += 1;
        }
        if self.identity_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Delegate", len)?;
        if let Some(v) = self.amount.as_ref() {
            struct_ser.serialize_field("amount", v)?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::Delegate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
            "identity_key",
            "identityKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Amount,
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
                            "amount" => Ok(GeneratedField::Amount),
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
            type Value = transaction_planner_request::Delegate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.Delegate")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<transaction_planner_request::Delegate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut identity_key__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = map.next_value()?;
                        }
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map.next_value()?;
                        }
                    }
                }
                Ok(transaction_planner_request::Delegate {
                    amount: amount__,
                    identity_key: identity_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Delegate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::Output {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        if self.address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Output", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::Output {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            Address,
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
                            "value" => Ok(GeneratedField::Value),
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::Output;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.Output")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<transaction_planner_request::Output, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map.next_value()?;
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map.next_value()?;
                        }
                    }
                }
                Ok(transaction_planner_request::Output {
                    value: value__,
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Output", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::Swap {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        if self.target_asset.is_some() {
            len += 1;
        }
        if self.fee.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Swap", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.target_asset.as_ref() {
            struct_ser.serialize_field("targetAsset", v)?;
        }
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::Swap {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "target_asset",
            "targetAsset",
            "fee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            TargetAsset,
            Fee,
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
                            "value" => Ok(GeneratedField::Value),
                            "targetAsset" | "target_asset" => Ok(GeneratedField::TargetAsset),
                            "fee" => Ok(GeneratedField::Fee),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::Swap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.Swap")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<transaction_planner_request::Swap, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut target_asset__ = None;
                let mut fee__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map.next_value()?;
                        }
                        GeneratedField::TargetAsset => {
                            if target_asset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targetAsset"));
                            }
                            target_asset__ = map.next_value()?;
                        }
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map.next_value()?;
                        }
                    }
                }
                Ok(transaction_planner_request::Swap {
                    value: value__,
                    target_asset: target_asset__,
                    fee: fee__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Swap", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_planner_request::Undelegate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Undelegate", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_planner_request::Undelegate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
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
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_planner_request::Undelegate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerRequest.Undelegate")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<transaction_planner_request::Undelegate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map.next_value()?;
                        }
                    }
                }
                Ok(transaction_planner_request::Undelegate {
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerRequest.Undelegate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionPlannerResponse {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionPlannerResponse", len)?;
        if let Some(v) = self.plan.as_ref() {
            struct_ser.serialize_field("plan", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionPlannerResponse {
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
            type Value = TransactionPlannerResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionPlannerResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionPlannerResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut plan__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Plan => {
                            if plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("plan"));
                            }
                            plan__ = map.next_value()?;
                        }
                    }
                }
                Ok(TransactionPlannerResponse {
                    plan: plan__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionPlannerResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_height.is_some() {
            len += 1;
        }
        if self.end_height.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionsRequest", len)?;
        if let Some(v) = self.start_height.as_ref() {
            struct_ser.serialize_field("startHeight", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.end_height.as_ref() {
            struct_ser.serialize_field("endHeight", ToString::to_string(&v).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_height",
            "startHeight",
            "end_height",
            "endHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartHeight,
            EndHeight,
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
                            "startHeight" | "start_height" => Ok(GeneratedField::StartHeight),
                            "endHeight" | "end_height" => Ok(GeneratedField::EndHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_height__ = None;
                let mut end_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::StartHeight => {
                            if start_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startHeight"));
                            }
                            start_height__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::EndHeight => {
                            if end_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endHeight"));
                            }
                            end_height__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(TransactionsRequest {
                    start_height: start_height__,
                    end_height: end_height__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.block_height != 0 {
            len += 1;
        }
        if !self.tx_hash.is_empty() {
            len += 1;
        }
        if self.tx.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.TransactionsResponse", len)?;
        if self.block_height != 0 {
            struct_ser.serialize_field("blockHeight", ToString::to_string(&self.block_height).as_str())?;
        }
        if !self.tx_hash.is_empty() {
            struct_ser.serialize_field("txHash", pbjson::private::base64::encode(&self.tx_hash).as_str())?;
        }
        if let Some(v) = self.tx.as_ref() {
            struct_ser.serialize_field("tx", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "block_height",
            "blockHeight",
            "tx_hash",
            "txHash",
            "tx",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BlockHeight,
            TxHash,
            Tx,
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
                            "blockHeight" | "block_height" => Ok(GeneratedField::BlockHeight),
                            "txHash" | "tx_hash" => Ok(GeneratedField::TxHash),
                            "tx" => Ok(GeneratedField::Tx),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.TransactionsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut block_height__ = None;
                let mut tx_hash__ = None;
                let mut tx__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BlockHeight => {
                            if block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHeight"));
                            }
                            block_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TxHash => {
                            if tx_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txHash"));
                            }
                            tx_hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Tx => {
                            if tx__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tx"));
                            }
                            tx__ = map.next_value()?;
                        }
                    }
                }
                Ok(TransactionsResponse {
                    block_height: block_height__.unwrap_or_default(),
                    tx_hash: tx_hash__.unwrap_or_default(),
                    tx: tx__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.TransactionsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ViewAuthRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.fvk.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.ViewAuthRequest", len)?;
        if let Some(v) = self.fvk.as_ref() {
            struct_ser.serialize_field("fvk", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ViewAuthRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fvk",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fvk,
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
                            "fvk" => Ok(GeneratedField::Fvk),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ViewAuthRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.ViewAuthRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ViewAuthRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fvk__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fvk => {
                            if fvk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fvk"));
                            }
                            fvk__ = map.next_value()?;
                        }
                    }
                }
                Ok(ViewAuthRequest {
                    fvk: fvk__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.ViewAuthRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ViewAuthResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.ViewAuthResponse", len)?;
        if let Some(v) = self.token.as_ref() {
            struct_ser.serialize_field("token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ViewAuthResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "token",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Token,
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
                            "token" => Ok(GeneratedField::Token),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ViewAuthResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.ViewAuthResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ViewAuthResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut token__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = map.next_value()?;
                        }
                    }
                }
                Ok(ViewAuthResponse {
                    token: token__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.ViewAuthResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ViewAuthToken {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.ViewAuthToken", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ViewAuthToken {
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
            type Value = ViewAuthToken;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.ViewAuthToken")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ViewAuthToken, V::Error>
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
                Ok(ViewAuthToken {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.ViewAuthToken", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WitnessRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.note_commitments.is_empty() {
            len += 1;
        }
        if self.transaction_plan.is_some() {
            len += 1;
        }
        if self.account_id.is_some() {
            len += 1;
        }
        if self.token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.WitnessRequest", len)?;
        if !self.note_commitments.is_empty() {
            struct_ser.serialize_field("noteCommitments", &self.note_commitments)?;
        }
        if let Some(v) = self.transaction_plan.as_ref() {
            struct_ser.serialize_field("transactionPlan", v)?;
        }
        if let Some(v) = self.account_id.as_ref() {
            struct_ser.serialize_field("accountId", v)?;
        }
        if let Some(v) = self.token.as_ref() {
            struct_ser.serialize_field("token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WitnessRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_commitments",
            "noteCommitments",
            "transaction_plan",
            "transactionPlan",
            "account_id",
            "accountId",
            "token",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteCommitments,
            TransactionPlan,
            AccountId,
            Token,
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
                            "noteCommitments" | "note_commitments" => Ok(GeneratedField::NoteCommitments),
                            "transactionPlan" | "transaction_plan" => Ok(GeneratedField::TransactionPlan),
                            "accountId" | "account_id" => Ok(GeneratedField::AccountId),
                            "token" => Ok(GeneratedField::Token),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WitnessRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.WitnessRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<WitnessRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_commitments__ = None;
                let mut transaction_plan__ = None;
                let mut account_id__ = None;
                let mut token__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NoteCommitments => {
                            if note_commitments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteCommitments"));
                            }
                            note_commitments__ = Some(map.next_value()?);
                        }
                        GeneratedField::TransactionPlan => {
                            if transaction_plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionPlan"));
                            }
                            transaction_plan__ = map.next_value()?;
                        }
                        GeneratedField::AccountId => {
                            if account_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountId"));
                            }
                            account_id__ = map.next_value()?;
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = map.next_value()?;
                        }
                    }
                }
                Ok(WitnessRequest {
                    note_commitments: note_commitments__.unwrap_or_default(),
                    transaction_plan: transaction_plan__,
                    account_id: account_id__,
                    token: token__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.WitnessRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WitnessResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.witness_data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.view.v1alpha1.WitnessResponse", len)?;
        if let Some(v) = self.witness_data.as_ref() {
            struct_ser.serialize_field("witnessData", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WitnessResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "witness_data",
            "witnessData",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WitnessData,
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
                            "witnessData" | "witness_data" => Ok(GeneratedField::WitnessData),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WitnessResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.view.v1alpha1.WitnessResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<WitnessResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut witness_data__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::WitnessData => {
                            if witness_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("witnessData"));
                            }
                            witness_data__ = map.next_value()?;
                        }
                    }
                }
                Ok(WitnessResponse {
                    witness_data: witness_data__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.view.v1alpha1.WitnessResponse", FIELDS, GeneratedVisitor)
    }
}
