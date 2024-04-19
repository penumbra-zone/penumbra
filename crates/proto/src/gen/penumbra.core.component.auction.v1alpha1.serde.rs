impl serde::Serialize for ActionDutchAuctionEnd {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.auction_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionEnd", len)?;
        if let Some(v) = self.auction_id.as_ref() {
            struct_ser.serialize_field("auctionId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionDutchAuctionEnd {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "auction_id",
            "auctionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuctionId,
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
                            "auctionId" | "auction_id" => Ok(GeneratedField::AuctionId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionDutchAuctionEnd;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.ActionDutchAuctionEnd")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionDutchAuctionEnd, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut auction_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AuctionId => {
                            if auction_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("auctionId"));
                            }
                            auction_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionDutchAuctionEnd {
                    auction_id: auction_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionEnd", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionDutchAuctionSchedule {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionSchedule", len)?;
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionDutchAuctionSchedule {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "description",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Description,
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
                            "description" => Ok(GeneratedField::Description),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionDutchAuctionSchedule;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.ActionDutchAuctionSchedule")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionDutchAuctionSchedule, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut description__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionDutchAuctionSchedule {
                    description: description__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionSchedule", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionDutchAuctionScheduleView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.action.is_some() {
            len += 1;
        }
        if self.auction_id.is_some() {
            len += 1;
        }
        if self.input_metadata.is_some() {
            len += 1;
        }
        if self.output_metadata.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionScheduleView", len)?;
        if let Some(v) = self.action.as_ref() {
            struct_ser.serialize_field("action", v)?;
        }
        if let Some(v) = self.auction_id.as_ref() {
            struct_ser.serialize_field("auctionId", v)?;
        }
        if let Some(v) = self.input_metadata.as_ref() {
            struct_ser.serialize_field("inputMetadata", v)?;
        }
        if let Some(v) = self.output_metadata.as_ref() {
            struct_ser.serialize_field("outputMetadata", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionDutchAuctionScheduleView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "action",
            "auction_id",
            "auctionId",
            "input_metadata",
            "inputMetadata",
            "output_metadata",
            "outputMetadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Action,
            AuctionId,
            InputMetadata,
            OutputMetadata,
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
                            "action" => Ok(GeneratedField::Action),
                            "auctionId" | "auction_id" => Ok(GeneratedField::AuctionId),
                            "inputMetadata" | "input_metadata" => Ok(GeneratedField::InputMetadata),
                            "outputMetadata" | "output_metadata" => Ok(GeneratedField::OutputMetadata),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionDutchAuctionScheduleView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.ActionDutchAuctionScheduleView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionDutchAuctionScheduleView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut action__ = None;
                let mut auction_id__ = None;
                let mut input_metadata__ = None;
                let mut output_metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Action => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("action"));
                            }
                            action__ = map_.next_value()?;
                        }
                        GeneratedField::AuctionId => {
                            if auction_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("auctionId"));
                            }
                            auction_id__ = map_.next_value()?;
                        }
                        GeneratedField::InputMetadata => {
                            if input_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inputMetadata"));
                            }
                            input_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::OutputMetadata => {
                            if output_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputMetadata"));
                            }
                            output_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionDutchAuctionScheduleView {
                    action: action__,
                    auction_id: auction_id__,
                    input_metadata: input_metadata__,
                    output_metadata: output_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionScheduleView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionDutchAuctionWithdraw {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.auction_id.is_some() {
            len += 1;
        }
        if self.seq != 0 {
            len += 1;
        }
        if self.reserves_commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionWithdraw", len)?;
        if let Some(v) = self.auction_id.as_ref() {
            struct_ser.serialize_field("auctionId", v)?;
        }
        if self.seq != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("seq", ToString::to_string(&self.seq).as_str())?;
        }
        if let Some(v) = self.reserves_commitment.as_ref() {
            struct_ser.serialize_field("reservesCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionDutchAuctionWithdraw {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "auction_id",
            "auctionId",
            "seq",
            "reserves_commitment",
            "reservesCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuctionId,
            Seq,
            ReservesCommitment,
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
                            "auctionId" | "auction_id" => Ok(GeneratedField::AuctionId),
                            "seq" => Ok(GeneratedField::Seq),
                            "reservesCommitment" | "reserves_commitment" => Ok(GeneratedField::ReservesCommitment),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionDutchAuctionWithdraw;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.ActionDutchAuctionWithdraw")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionDutchAuctionWithdraw, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut auction_id__ = None;
                let mut seq__ = None;
                let mut reserves_commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AuctionId => {
                            if auction_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("auctionId"));
                            }
                            auction_id__ = map_.next_value()?;
                        }
                        GeneratedField::Seq => {
                            if seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("seq"));
                            }
                            seq__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ReservesCommitment => {
                            if reserves_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reservesCommitment"));
                            }
                            reserves_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionDutchAuctionWithdraw {
                    auction_id: auction_id__,
                    seq: seq__.unwrap_or_default(),
                    reserves_commitment: reserves_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionWithdraw", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionDutchAuctionWithdrawPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.auction_id.is_some() {
            len += 1;
        }
        if self.seq != 0 {
            len += 1;
        }
        if self.reserves_input.is_some() {
            len += 1;
        }
        if self.reserves_output.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionWithdrawPlan", len)?;
        if let Some(v) = self.auction_id.as_ref() {
            struct_ser.serialize_field("auctionId", v)?;
        }
        if self.seq != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("seq", ToString::to_string(&self.seq).as_str())?;
        }
        if let Some(v) = self.reserves_input.as_ref() {
            struct_ser.serialize_field("reservesInput", v)?;
        }
        if let Some(v) = self.reserves_output.as_ref() {
            struct_ser.serialize_field("reservesOutput", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionDutchAuctionWithdrawPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "auction_id",
            "auctionId",
            "seq",
            "reserves_input",
            "reservesInput",
            "reserves_output",
            "reservesOutput",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuctionId,
            Seq,
            ReservesInput,
            ReservesOutput,
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
                            "auctionId" | "auction_id" => Ok(GeneratedField::AuctionId),
                            "seq" => Ok(GeneratedField::Seq),
                            "reservesInput" | "reserves_input" => Ok(GeneratedField::ReservesInput),
                            "reservesOutput" | "reserves_output" => Ok(GeneratedField::ReservesOutput),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionDutchAuctionWithdrawPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.ActionDutchAuctionWithdrawPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionDutchAuctionWithdrawPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut auction_id__ = None;
                let mut seq__ = None;
                let mut reserves_input__ = None;
                let mut reserves_output__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AuctionId => {
                            if auction_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("auctionId"));
                            }
                            auction_id__ = map_.next_value()?;
                        }
                        GeneratedField::Seq => {
                            if seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("seq"));
                            }
                            seq__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ReservesInput => {
                            if reserves_input__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reservesInput"));
                            }
                            reserves_input__ = map_.next_value()?;
                        }
                        GeneratedField::ReservesOutput => {
                            if reserves_output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reservesOutput"));
                            }
                            reserves_output__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionDutchAuctionWithdrawPlan {
                    auction_id: auction_id__,
                    seq: seq__.unwrap_or_default(),
                    reserves_input: reserves_input__,
                    reserves_output: reserves_output__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionWithdrawPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionDutchAuctionWithdrawView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.action.is_some() {
            len += 1;
        }
        if !self.reserves.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionWithdrawView", len)?;
        if let Some(v) = self.action.as_ref() {
            struct_ser.serialize_field("action", v)?;
        }
        if !self.reserves.is_empty() {
            struct_ser.serialize_field("reserves", &self.reserves)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionDutchAuctionWithdrawView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "action",
            "reserves",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Action,
            Reserves,
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
                            "action" => Ok(GeneratedField::Action),
                            "reserves" => Ok(GeneratedField::Reserves),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionDutchAuctionWithdrawView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.ActionDutchAuctionWithdrawView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionDutchAuctionWithdrawView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut action__ = None;
                let mut reserves__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Action => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("action"));
                            }
                            action__ = map_.next_value()?;
                        }
                        GeneratedField::Reserves => {
                            if reserves__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reserves"));
                            }
                            reserves__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionDutchAuctionWithdrawView {
                    action: action__,
                    reserves: reserves__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.ActionDutchAuctionWithdrawView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuctionId {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.AuctionId", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuctionId {
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
            type Value = AuctionId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.AuctionId")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuctionId, V::Error>
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
                Ok(AuctionId {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.AuctionId", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuctionNft {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id.is_some() {
            len += 1;
        }
        if self.seq != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.AuctionNft", len)?;
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if self.seq != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("seq", ToString::to_string(&self.seq).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuctionNft {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "seq",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Seq,
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
                            "id" => Ok(GeneratedField::Id),
                            "seq" => Ok(GeneratedField::Seq),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuctionNft;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.AuctionNft")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuctionNft, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut seq__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = map_.next_value()?;
                        }
                        GeneratedField::Seq => {
                            if seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("seq"));
                            }
                            seq__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuctionNft {
                    id: id__,
                    seq: seq__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.AuctionNft", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuctionParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.AuctionParameters", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuctionParameters {
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
            type Value = AuctionParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.AuctionParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuctionParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(AuctionParameters {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.AuctionParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DutchAuction {
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
        if self.state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.DutchAuction", len)?;
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if let Some(v) = self.state.as_ref() {
            struct_ser.serialize_field("state", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DutchAuction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "description",
            "state",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Description,
            State,
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
                            "description" => Ok(GeneratedField::Description),
                            "state" => Ok(GeneratedField::State),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DutchAuction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.DutchAuction")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DutchAuction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut description__ = None;
                let mut state__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DutchAuction {
                    description: description__,
                    state: state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.DutchAuction", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DutchAuctionDescription {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.input.is_some() {
            len += 1;
        }
        if self.output_id.is_some() {
            len += 1;
        }
        if self.max_output.is_some() {
            len += 1;
        }
        if self.min_output.is_some() {
            len += 1;
        }
        if self.start_height != 0 {
            len += 1;
        }
        if self.end_height != 0 {
            len += 1;
        }
        if self.step_count != 0 {
            len += 1;
        }
        if !self.nonce.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.DutchAuctionDescription", len)?;
        if let Some(v) = self.input.as_ref() {
            struct_ser.serialize_field("input", v)?;
        }
        if let Some(v) = self.output_id.as_ref() {
            struct_ser.serialize_field("outputId", v)?;
        }
        if let Some(v) = self.max_output.as_ref() {
            struct_ser.serialize_field("maxOutput", v)?;
        }
        if let Some(v) = self.min_output.as_ref() {
            struct_ser.serialize_field("minOutput", v)?;
        }
        if self.start_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("startHeight", ToString::to_string(&self.start_height).as_str())?;
        }
        if self.end_height != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("endHeight", ToString::to_string(&self.end_height).as_str())?;
        }
        if self.step_count != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("stepCount", ToString::to_string(&self.step_count).as_str())?;
        }
        if !self.nonce.is_empty() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("nonce", pbjson::private::base64::encode(&self.nonce).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DutchAuctionDescription {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "input",
            "output_id",
            "outputId",
            "max_output",
            "maxOutput",
            "min_output",
            "minOutput",
            "start_height",
            "startHeight",
            "end_height",
            "endHeight",
            "step_count",
            "stepCount",
            "nonce",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Input,
            OutputId,
            MaxOutput,
            MinOutput,
            StartHeight,
            EndHeight,
            StepCount,
            Nonce,
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
                            "input" => Ok(GeneratedField::Input),
                            "outputId" | "output_id" => Ok(GeneratedField::OutputId),
                            "maxOutput" | "max_output" => Ok(GeneratedField::MaxOutput),
                            "minOutput" | "min_output" => Ok(GeneratedField::MinOutput),
                            "startHeight" | "start_height" => Ok(GeneratedField::StartHeight),
                            "endHeight" | "end_height" => Ok(GeneratedField::EndHeight),
                            "stepCount" | "step_count" => Ok(GeneratedField::StepCount),
                            "nonce" => Ok(GeneratedField::Nonce),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DutchAuctionDescription;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.DutchAuctionDescription")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DutchAuctionDescription, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut input__ = None;
                let mut output_id__ = None;
                let mut max_output__ = None;
                let mut min_output__ = None;
                let mut start_height__ = None;
                let mut end_height__ = None;
                let mut step_count__ = None;
                let mut nonce__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Input => {
                            if input__.is_some() {
                                return Err(serde::de::Error::duplicate_field("input"));
                            }
                            input__ = map_.next_value()?;
                        }
                        GeneratedField::OutputId => {
                            if output_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputId"));
                            }
                            output_id__ = map_.next_value()?;
                        }
                        GeneratedField::MaxOutput => {
                            if max_output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxOutput"));
                            }
                            max_output__ = map_.next_value()?;
                        }
                        GeneratedField::MinOutput => {
                            if min_output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minOutput"));
                            }
                            min_output__ = map_.next_value()?;
                        }
                        GeneratedField::StartHeight => {
                            if start_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startHeight"));
                            }
                            start_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndHeight => {
                            if end_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endHeight"));
                            }
                            end_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::StepCount => {
                            if step_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stepCount"));
                            }
                            step_count__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Nonce => {
                            if nonce__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nonce"));
                            }
                            nonce__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DutchAuctionDescription {
                    input: input__,
                    output_id: output_id__,
                    max_output: max_output__,
                    min_output: min_output__,
                    start_height: start_height__.unwrap_or_default(),
                    end_height: end_height__.unwrap_or_default(),
                    step_count: step_count__.unwrap_or_default(),
                    nonce: nonce__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.DutchAuctionDescription", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DutchAuctionState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.seq != 0 {
            len += 1;
        }
        if self.current_position.is_some() {
            len += 1;
        }
        if self.next_trigger != 0 {
            len += 1;
        }
        if self.input_reserves.is_some() {
            len += 1;
        }
        if self.output_reserves.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.DutchAuctionState", len)?;
        if self.seq != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("seq", ToString::to_string(&self.seq).as_str())?;
        }
        if let Some(v) = self.current_position.as_ref() {
            struct_ser.serialize_field("currentPosition", v)?;
        }
        if self.next_trigger != 0 {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("nextTrigger", ToString::to_string(&self.next_trigger).as_str())?;
        }
        if let Some(v) = self.input_reserves.as_ref() {
            struct_ser.serialize_field("inputReserves", v)?;
        }
        if let Some(v) = self.output_reserves.as_ref() {
            struct_ser.serialize_field("outputReserves", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DutchAuctionState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "seq",
            "current_position",
            "currentPosition",
            "next_trigger",
            "nextTrigger",
            "input_reserves",
            "inputReserves",
            "output_reserves",
            "outputReserves",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Seq,
            CurrentPosition,
            NextTrigger,
            InputReserves,
            OutputReserves,
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
                            "seq" => Ok(GeneratedField::Seq),
                            "currentPosition" | "current_position" => Ok(GeneratedField::CurrentPosition),
                            "nextTrigger" | "next_trigger" => Ok(GeneratedField::NextTrigger),
                            "inputReserves" | "input_reserves" => Ok(GeneratedField::InputReserves),
                            "outputReserves" | "output_reserves" => Ok(GeneratedField::OutputReserves),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DutchAuctionState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.DutchAuctionState")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DutchAuctionState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut seq__ = None;
                let mut current_position__ = None;
                let mut next_trigger__ = None;
                let mut input_reserves__ = None;
                let mut output_reserves__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Seq => {
                            if seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("seq"));
                            }
                            seq__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CurrentPosition => {
                            if current_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("currentPosition"));
                            }
                            current_position__ = map_.next_value()?;
                        }
                        GeneratedField::NextTrigger => {
                            if next_trigger__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextTrigger"));
                            }
                            next_trigger__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::InputReserves => {
                            if input_reserves__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inputReserves"));
                            }
                            input_reserves__ = map_.next_value()?;
                        }
                        GeneratedField::OutputReserves => {
                            if output_reserves__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputReserves"));
                            }
                            output_reserves__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DutchAuctionState {
                    seq: seq__.unwrap_or_default(),
                    current_position: current_position__,
                    next_trigger: next_trigger__.unwrap_or_default(),
                    input_reserves: input_reserves__,
                    output_reserves: output_reserves__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.DutchAuctionState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenesisContent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.auction.v1alpha1.GenesisContent", len)?;
        if let Some(v) = self.params.as_ref() {
            struct_ser.serialize_field("params", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenesisContent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "params",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Params,
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
                            "params" => Ok(GeneratedField::Params),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenesisContent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.auction.v1alpha1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Params => {
                            if params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("params"));
                            }
                            params__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    params: params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.auction.v1alpha1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
