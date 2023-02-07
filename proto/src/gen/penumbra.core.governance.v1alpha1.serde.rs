impl serde::Serialize for DelegatorVote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.body.is_some() {
            len += 1;
        }
        if self.auth_sig.is_some() {
            len += 1;
        }
        if !self.proof.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.DelegatorVote", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.auth_sig.as_ref() {
            struct_ser.serialize_field("authSig", v)?;
        }
        if !self.proof.is_empty() {
            struct_ser.serialize_field("proof", pbjson::private::base64::encode(&self.proof).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorVote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body",
            "auth_sig",
            "authSig",
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Body,
            AuthSig,
            Proof,
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
                            "body" => Ok(GeneratedField::Body),
                            "authSig" | "auth_sig" => Ok(GeneratedField::AuthSig),
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.DelegatorVote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DelegatorVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut auth_sig__ = None;
                let mut proof__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map.next_value()?;
                        }
                        GeneratedField::AuthSig => {
                            if auth_sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authSig"));
                            }
                            auth_sig__ = map.next_value()?;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(DelegatorVote {
                    body: body__,
                    auth_sig: auth_sig__,
                    proof: proof__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.DelegatorVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorVoteBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal != 0 {
            len += 1;
        }
        if !self.nullifier.is_empty() {
            len += 1;
        }
        if !self.rk.is_empty() {
            len += 1;
        }
        if self.yes_balance_commitment.is_some() {
            len += 1;
        }
        if self.no_balance_commitment.is_some() {
            len += 1;
        }
        if self.abstain_balance_commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.DelegatorVoteBody", len)?;
        if self.proposal != 0 {
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if !self.nullifier.is_empty() {
            struct_ser.serialize_field("nullifier", pbjson::private::base64::encode(&self.nullifier).as_str())?;
        }
        if !self.rk.is_empty() {
            struct_ser.serialize_field("rk", pbjson::private::base64::encode(&self.rk).as_str())?;
        }
        if let Some(v) = self.yes_balance_commitment.as_ref() {
            struct_ser.serialize_field("yesBalanceCommitment", v)?;
        }
        if let Some(v) = self.no_balance_commitment.as_ref() {
            struct_ser.serialize_field("noBalanceCommitment", v)?;
        }
        if let Some(v) = self.abstain_balance_commitment.as_ref() {
            struct_ser.serialize_field("abstainBalanceCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorVoteBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "nullifier",
            "rk",
            "yes_balance_commitment",
            "yesBalanceCommitment",
            "no_balance_commitment",
            "noBalanceCommitment",
            "abstain_balance_commitment",
            "abstainBalanceCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            Nullifier,
            Rk,
            YesBalanceCommitment,
            NoBalanceCommitment,
            AbstainBalanceCommitment,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "rk" => Ok(GeneratedField::Rk),
                            "yesBalanceCommitment" | "yes_balance_commitment" => Ok(GeneratedField::YesBalanceCommitment),
                            "noBalanceCommitment" | "no_balance_commitment" => Ok(GeneratedField::NoBalanceCommitment),
                            "abstainBalanceCommitment" | "abstain_balance_commitment" => Ok(GeneratedField::AbstainBalanceCommitment),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorVoteBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.DelegatorVoteBody")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DelegatorVoteBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut nullifier__ = None;
                let mut rk__ = None;
                let mut yes_balance_commitment__ = None;
                let mut no_balance_commitment__ = None;
                let mut abstain_balance_commitment__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Rk => {
                            if rk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rk"));
                            }
                            rk__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::YesBalanceCommitment => {
                            if yes_balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("yesBalanceCommitment"));
                            }
                            yes_balance_commitment__ = map.next_value()?;
                        }
                        GeneratedField::NoBalanceCommitment => {
                            if no_balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noBalanceCommitment"));
                            }
                            no_balance_commitment__ = map.next_value()?;
                        }
                        GeneratedField::AbstainBalanceCommitment => {
                            if abstain_balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("abstainBalanceCommitment"));
                            }
                            abstain_balance_commitment__ = map.next_value()?;
                        }
                    }
                }
                Ok(DelegatorVoteBody {
                    proposal: proposal__.unwrap_or_default(),
                    nullifier: nullifier__.unwrap_or_default(),
                    rk: rk__.unwrap_or_default(),
                    yes_balance_commitment: yes_balance_commitment__,
                    no_balance_commitment: no_balance_commitment__,
                    abstain_balance_commitment: abstain_balance_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.DelegatorVoteBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorVotePlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal != 0 {
            len += 1;
        }
        if self.vote.is_some() {
            len += 1;
        }
        if self.staked_note.is_some() {
            len += 1;
        }
        if self.position != 0 {
            len += 1;
        }
        if !self.randomizer.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.DelegatorVotePlan", len)?;
        if self.proposal != 0 {
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        if let Some(v) = self.staked_note.as_ref() {
            struct_ser.serialize_field("stakedNote", v)?;
        }
        if self.position != 0 {
            struct_ser.serialize_field("position", ToString::to_string(&self.position).as_str())?;
        }
        if !self.randomizer.is_empty() {
            struct_ser.serialize_field("randomizer", pbjson::private::base64::encode(&self.randomizer).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorVotePlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "vote",
            "staked_note",
            "stakedNote",
            "position",
            "randomizer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            Vote,
            StakedNote,
            Position,
            Randomizer,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "vote" => Ok(GeneratedField::Vote),
                            "stakedNote" | "staked_note" => Ok(GeneratedField::StakedNote),
                            "position" => Ok(GeneratedField::Position),
                            "randomizer" => Ok(GeneratedField::Randomizer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorVotePlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.DelegatorVotePlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DelegatorVotePlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut vote__ = None;
                let mut staked_note__ = None;
                let mut position__ = None;
                let mut randomizer__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map.next_value()?;
                        }
                        GeneratedField::StakedNote => {
                            if staked_note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakedNote"));
                            }
                            staked_note__ = map.next_value()?;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Randomizer => {
                            if randomizer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("randomizer"));
                            }
                            randomizer__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(DelegatorVotePlan {
                    proposal: proposal__.unwrap_or_default(),
                    vote: vote__,
                    staked_note: staked_note__,
                    position: position__.unwrap_or_default(),
                    randomizer: randomizer__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.DelegatorVotePlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MutableChainParameter {
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
        if !self.description.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.MutableChainParameter", len)?;
        if !self.identifier.is_empty() {
            struct_ser.serialize_field("identifier", &self.identifier)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MutableChainParameter {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "identifier",
            "description",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Identifier,
            Description,
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
                            "description" => Ok(GeneratedField::Description),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MutableChainParameter;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.MutableChainParameter")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MutableChainParameter, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut identifier__ = None;
                let mut description__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Identifier => {
                            if identifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identifier"));
                            }
                            identifier__ = Some(map.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MutableChainParameter {
                    identifier: identifier__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.MutableChainParameter", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Proposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if !self.title.is_empty() {
            len += 1;
        }
        if !self.description.is_empty() {
            len += 1;
        }
        if self.signaling.is_some() {
            len += 1;
        }
        if self.emergency.is_some() {
            len += 1;
        }
        if self.parameter_change.is_some() {
            len += 1;
        }
        if self.dao_spend.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.Proposal", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", ToString::to_string(&self.id).as_str())?;
        }
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if let Some(v) = self.signaling.as_ref() {
            struct_ser.serialize_field("signaling", v)?;
        }
        if let Some(v) = self.emergency.as_ref() {
            struct_ser.serialize_field("emergency", v)?;
        }
        if let Some(v) = self.parameter_change.as_ref() {
            struct_ser.serialize_field("parameterChange", v)?;
        }
        if let Some(v) = self.dao_spend.as_ref() {
            struct_ser.serialize_field("daoSpend", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Proposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "title",
            "description",
            "signaling",
            "emergency",
            "parameter_change",
            "parameterChange",
            "dao_spend",
            "daoSpend",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Title,
            Description,
            Signaling,
            Emergency,
            ParameterChange,
            DaoSpend,
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
                            "title" => Ok(GeneratedField::Title),
                            "description" => Ok(GeneratedField::Description),
                            "signaling" => Ok(GeneratedField::Signaling),
                            "emergency" => Ok(GeneratedField::Emergency),
                            "parameterChange" | "parameter_change" => Ok(GeneratedField::ParameterChange),
                            "daoSpend" | "dao_spend" => Ok(GeneratedField::DaoSpend),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Proposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.Proposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Proposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut title__ = None;
                let mut description__ = None;
                let mut signaling__ = None;
                let mut emergency__ = None;
                let mut parameter_change__ = None;
                let mut dao_spend__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map.next_value()?);
                        }
                        GeneratedField::Signaling => {
                            if signaling__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signaling"));
                            }
                            signaling__ = map.next_value()?;
                        }
                        GeneratedField::Emergency => {
                            if emergency__.is_some() {
                                return Err(serde::de::Error::duplicate_field("emergency"));
                            }
                            emergency__ = map.next_value()?;
                        }
                        GeneratedField::ParameterChange => {
                            if parameter_change__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameterChange"));
                            }
                            parameter_change__ = map.next_value()?;
                        }
                        GeneratedField::DaoSpend => {
                            if dao_spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoSpend"));
                            }
                            dao_spend__ = map.next_value()?;
                        }
                    }
                }
                Ok(Proposal {
                    id: id__.unwrap_or_default(),
                    title: title__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    signaling: signaling__,
                    emergency: emergency__,
                    parameter_change: parameter_change__,
                    dao_spend: dao_spend__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.Proposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::DaoSpend {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.schedule_transactions.is_empty() {
            len += 1;
        }
        if !self.cancel_transactions.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.Proposal.DaoSpend", len)?;
        if !self.schedule_transactions.is_empty() {
            struct_ser.serialize_field("scheduleTransactions", &self.schedule_transactions)?;
        }
        if !self.cancel_transactions.is_empty() {
            struct_ser.serialize_field("cancelTransactions", &self.cancel_transactions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::DaoSpend {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "schedule_transactions",
            "scheduleTransactions",
            "cancel_transactions",
            "cancelTransactions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ScheduleTransactions,
            CancelTransactions,
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
                            "scheduleTransactions" | "schedule_transactions" => Ok(GeneratedField::ScheduleTransactions),
                            "cancelTransactions" | "cancel_transactions" => Ok(GeneratedField::CancelTransactions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::DaoSpend;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.Proposal.DaoSpend")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal::DaoSpend, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut schedule_transactions__ = None;
                let mut cancel_transactions__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ScheduleTransactions => {
                            if schedule_transactions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scheduleTransactions"));
                            }
                            schedule_transactions__ = Some(map.next_value()?);
                        }
                        GeneratedField::CancelTransactions => {
                            if cancel_transactions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cancelTransactions"));
                            }
                            cancel_transactions__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(proposal::DaoSpend {
                    schedule_transactions: schedule_transactions__.unwrap_or_default(),
                    cancel_transactions: cancel_transactions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.Proposal.DaoSpend", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::dao_spend::CancelTransaction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.scheduled_at_height != 0 {
            len += 1;
        }
        if self.effect_hash.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.Proposal.DaoSpend.CancelTransaction", len)?;
        if self.scheduled_at_height != 0 {
            struct_ser.serialize_field("scheduledAtHeight", ToString::to_string(&self.scheduled_at_height).as_str())?;
        }
        if let Some(v) = self.effect_hash.as_ref() {
            struct_ser.serialize_field("effectHash", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::dao_spend::CancelTransaction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "scheduled_at_height",
            "scheduledAtHeight",
            "effect_hash",
            "effectHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ScheduledAtHeight,
            EffectHash,
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
                            "scheduledAtHeight" | "scheduled_at_height" => Ok(GeneratedField::ScheduledAtHeight),
                            "effectHash" | "effect_hash" => Ok(GeneratedField::EffectHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::dao_spend::CancelTransaction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.Proposal.DaoSpend.CancelTransaction")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal::dao_spend::CancelTransaction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut scheduled_at_height__ = None;
                let mut effect_hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ScheduledAtHeight => {
                            if scheduled_at_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scheduledAtHeight"));
                            }
                            scheduled_at_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EffectHash => {
                            if effect_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effectHash"));
                            }
                            effect_hash__ = map.next_value()?;
                        }
                    }
                }
                Ok(proposal::dao_spend::CancelTransaction {
                    scheduled_at_height: scheduled_at_height__.unwrap_or_default(),
                    effect_hash: effect_hash__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.Proposal.DaoSpend.CancelTransaction", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::dao_spend::ScheduleTransaction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.execute_at_height != 0 {
            len += 1;
        }
        if self.transaction.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.Proposal.DaoSpend.ScheduleTransaction", len)?;
        if self.execute_at_height != 0 {
            struct_ser.serialize_field("executeAtHeight", ToString::to_string(&self.execute_at_height).as_str())?;
        }
        if let Some(v) = self.transaction.as_ref() {
            struct_ser.serialize_field("transaction", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::dao_spend::ScheduleTransaction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "execute_at_height",
            "executeAtHeight",
            "transaction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExecuteAtHeight,
            Transaction,
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
                            "executeAtHeight" | "execute_at_height" => Ok(GeneratedField::ExecuteAtHeight),
                            "transaction" => Ok(GeneratedField::Transaction),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::dao_spend::ScheduleTransaction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.Proposal.DaoSpend.ScheduleTransaction")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal::dao_spend::ScheduleTransaction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut execute_at_height__ = None;
                let mut transaction__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ExecuteAtHeight => {
                            if execute_at_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("executeAtHeight"));
                            }
                            execute_at_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Transaction => {
                            if transaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transaction"));
                            }
                            transaction__ = map.next_value()?;
                        }
                    }
                }
                Ok(proposal::dao_spend::ScheduleTransaction {
                    execute_at_height: execute_at_height__.unwrap_or_default(),
                    transaction: transaction__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.Proposal.DaoSpend.ScheduleTransaction", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::Emergency {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.halt_chain {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.Proposal.Emergency", len)?;
        if self.halt_chain {
            struct_ser.serialize_field("haltChain", &self.halt_chain)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::Emergency {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "halt_chain",
            "haltChain",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            HaltChain,
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
                            "haltChain" | "halt_chain" => Ok(GeneratedField::HaltChain),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::Emergency;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.Proposal.Emergency")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal::Emergency, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut halt_chain__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::HaltChain => {
                            if halt_chain__.is_some() {
                                return Err(serde::de::Error::duplicate_field("haltChain"));
                            }
                            halt_chain__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(proposal::Emergency {
                    halt_chain: halt_chain__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.Proposal.Emergency", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::ParameterChange {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.effective_height != 0 {
            len += 1;
        }
        if !self.new_parameters.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.Proposal.ParameterChange", len)?;
        if self.effective_height != 0 {
            struct_ser.serialize_field("effectiveHeight", ToString::to_string(&self.effective_height).as_str())?;
        }
        if !self.new_parameters.is_empty() {
            struct_ser.serialize_field("newParameters", &self.new_parameters)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::ParameterChange {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "effective_height",
            "effectiveHeight",
            "new_parameters",
            "newParameters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EffectiveHeight,
            NewParameters,
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
                            "effectiveHeight" | "effective_height" => Ok(GeneratedField::EffectiveHeight),
                            "newParameters" | "new_parameters" => Ok(GeneratedField::NewParameters),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::ParameterChange;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.Proposal.ParameterChange")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal::ParameterChange, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut effective_height__ = None;
                let mut new_parameters__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::EffectiveHeight => {
                            if effective_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effectiveHeight"));
                            }
                            effective_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::NewParameters => {
                            if new_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newParameters"));
                            }
                            new_parameters__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(proposal::ParameterChange {
                    effective_height: effective_height__.unwrap_or_default(),
                    new_parameters: new_parameters__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.Proposal.ParameterChange", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::parameter_change::SetParameter {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.parameter.is_empty() {
            len += 1;
        }
        if !self.value.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.Proposal.ParameterChange.SetParameter", len)?;
        if !self.parameter.is_empty() {
            struct_ser.serialize_field("parameter", &self.parameter)?;
        }
        if !self.value.is_empty() {
            struct_ser.serialize_field("value", &self.value)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::parameter_change::SetParameter {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parameter",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Parameter,
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
                            "parameter" => Ok(GeneratedField::Parameter),
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
            type Value = proposal::parameter_change::SetParameter;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.Proposal.ParameterChange.SetParameter")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal::parameter_change::SetParameter, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut parameter__ = None;
                let mut value__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Parameter => {
                            if parameter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameter"));
                            }
                            parameter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(proposal::parameter_change::SetParameter {
                    parameter: parameter__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.Proposal.ParameterChange.SetParameter", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal::Signaling {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.commit.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.Proposal.Signaling", len)?;
        if let Some(v) = self.commit.as_ref() {
            struct_ser.serialize_field("commit", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal::Signaling {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commit,
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
                            "commit" => Ok(GeneratedField::Commit),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal::Signaling;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.Proposal.Signaling")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal::Signaling, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commit__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Commit => {
                            if commit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commit"));
                            }
                            commit__ = map.next_value()?;
                        }
                    }
                }
                Ok(proposal::Signaling {
                    commit: commit__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.Proposal.Signaling", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalDepositClaim {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal != 0 {
            len += 1;
        }
        if self.deposit_amount.is_some() {
            len += 1;
        }
        if self.outcome.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalDepositClaim", len)?;
        if self.proposal != 0 {
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if let Some(v) = self.deposit_amount.as_ref() {
            struct_ser.serialize_field("depositAmount", v)?;
        }
        if let Some(v) = self.outcome.as_ref() {
            struct_ser.serialize_field("outcome", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalDepositClaim {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "deposit_amount",
            "depositAmount",
            "outcome",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            DepositAmount,
            Outcome,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "depositAmount" | "deposit_amount" => Ok(GeneratedField::DepositAmount),
                            "outcome" => Ok(GeneratedField::Outcome),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalDepositClaim;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalDepositClaim")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ProposalDepositClaim, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut deposit_amount__ = None;
                let mut outcome__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DepositAmount => {
                            if deposit_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositAmount"));
                            }
                            deposit_amount__ = map.next_value()?;
                        }
                        GeneratedField::Outcome => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outcome"));
                            }
                            outcome__ = map.next_value()?;
                        }
                    }
                }
                Ok(ProposalDepositClaim {
                    proposal: proposal__.unwrap_or_default(),
                    deposit_amount: deposit_amount__,
                    outcome: outcome__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalDepositClaim", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalList {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.proposals.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalList", len)?;
        if !self.proposals.is_empty() {
            struct_ser.serialize_field("proposals", &self.proposals.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalList {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposals",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposals,
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
                            "proposals" => Ok(GeneratedField::Proposals),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalList;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalList")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ProposalList, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposals__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposals => {
                            if proposals__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposals"));
                            }
                            proposals__ = 
                                Some(map.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(ProposalList {
                    proposals: proposals__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalList", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalOutcome {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.outcome.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalOutcome", len)?;
        if let Some(v) = self.outcome.as_ref() {
            match v {
                proposal_outcome::Outcome::Passed(v) => {
                    struct_ser.serialize_field("passed", v)?;
                }
                proposal_outcome::Outcome::Failed(v) => {
                    struct_ser.serialize_field("failed", v)?;
                }
                proposal_outcome::Outcome::Vetoed(v) => {
                    struct_ser.serialize_field("vetoed", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalOutcome {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "passed",
            "failed",
            "vetoed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Passed,
            Failed,
            Vetoed,
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
                            "passed" => Ok(GeneratedField::Passed),
                            "failed" => Ok(GeneratedField::Failed),
                            "vetoed" => Ok(GeneratedField::Vetoed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalOutcome;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalOutcome")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ProposalOutcome, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut outcome__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Passed => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("passed"));
                            }
                            outcome__ = map.next_value::<::std::option::Option<_>>()?.map(proposal_outcome::Outcome::Passed)
;
                        }
                        GeneratedField::Failed => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("failed"));
                            }
                            outcome__ = map.next_value::<::std::option::Option<_>>()?.map(proposal_outcome::Outcome::Failed)
;
                        }
                        GeneratedField::Vetoed => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vetoed"));
                            }
                            outcome__ = map.next_value::<::std::option::Option<_>>()?.map(proposal_outcome::Outcome::Vetoed)
;
                        }
                    }
                }
                Ok(ProposalOutcome {
                    outcome: outcome__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalOutcome", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_outcome::Failed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.withdrawn_with_reason.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalOutcome.Failed", len)?;
        if let Some(v) = self.withdrawn_with_reason.as_ref() {
            struct_ser.serialize_field("withdrawnWithReason", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_outcome::Failed {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "withdrawn_with_reason",
            "withdrawnWithReason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WithdrawnWithReason,
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
                            "withdrawnWithReason" | "withdrawn_with_reason" => Ok(GeneratedField::WithdrawnWithReason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_outcome::Failed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalOutcome.Failed")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal_outcome::Failed, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut withdrawn_with_reason__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::WithdrawnWithReason => {
                            if withdrawn_with_reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawnWithReason"));
                            }
                            withdrawn_with_reason__ = map.next_value()?;
                        }
                    }
                }
                Ok(proposal_outcome::Failed {
                    withdrawn_with_reason: withdrawn_with_reason__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalOutcome.Failed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_outcome::Passed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalOutcome.Passed", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_outcome::Passed {
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
            type Value = proposal_outcome::Passed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalOutcome.Passed")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal_outcome::Passed, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(proposal_outcome::Passed {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalOutcome.Passed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_outcome::Vetoed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.withdrawn_with_reason.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalOutcome.Vetoed", len)?;
        if let Some(v) = self.withdrawn_with_reason.as_ref() {
            struct_ser.serialize_field("withdrawnWithReason", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_outcome::Vetoed {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "withdrawn_with_reason",
            "withdrawnWithReason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WithdrawnWithReason,
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
                            "withdrawnWithReason" | "withdrawn_with_reason" => Ok(GeneratedField::WithdrawnWithReason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_outcome::Vetoed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalOutcome.Vetoed")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal_outcome::Vetoed, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut withdrawn_with_reason__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::WithdrawnWithReason => {
                            if withdrawn_with_reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawnWithReason"));
                            }
                            withdrawn_with_reason__ = map.next_value()?;
                        }
                    }
                }
                Ok(proposal_outcome::Vetoed {
                    withdrawn_with_reason: withdrawn_with_reason__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalOutcome.Vetoed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.state.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalState", len)?;
        if let Some(v) = self.state.as_ref() {
            match v {
                proposal_state::State::Voting(v) => {
                    struct_ser.serialize_field("voting", v)?;
                }
                proposal_state::State::Withdrawn(v) => {
                    struct_ser.serialize_field("withdrawn", v)?;
                }
                proposal_state::State::Finished(v) => {
                    struct_ser.serialize_field("finished", v)?;
                }
                proposal_state::State::Claimed(v) => {
                    struct_ser.serialize_field("claimed", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "voting",
            "withdrawn",
            "finished",
            "claimed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Voting,
            Withdrawn,
            Finished,
            Claimed,
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
                            "voting" => Ok(GeneratedField::Voting),
                            "withdrawn" => Ok(GeneratedField::Withdrawn),
                            "finished" => Ok(GeneratedField::Finished),
                            "claimed" => Ok(GeneratedField::Claimed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ProposalState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Voting => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voting"));
                            }
                            state__ = map.next_value::<::std::option::Option<_>>()?.map(proposal_state::State::Voting)
;
                        }
                        GeneratedField::Withdrawn => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawn"));
                            }
                            state__ = map.next_value::<::std::option::Option<_>>()?.map(proposal_state::State::Withdrawn)
;
                        }
                        GeneratedField::Finished => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finished"));
                            }
                            state__ = map.next_value::<::std::option::Option<_>>()?.map(proposal_state::State::Finished)
;
                        }
                        GeneratedField::Claimed => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("claimed"));
                            }
                            state__ = map.next_value::<::std::option::Option<_>>()?.map(proposal_state::State::Claimed)
;
                        }
                    }
                }
                Ok(ProposalState {
                    state: state__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_state::Claimed {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.outcome.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalState.Claimed", len)?;
        if let Some(v) = self.outcome.as_ref() {
            struct_ser.serialize_field("outcome", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_state::Claimed {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "outcome",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Outcome,
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
                            "outcome" => Ok(GeneratedField::Outcome),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_state::Claimed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalState.Claimed")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal_state::Claimed, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut outcome__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Outcome => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outcome"));
                            }
                            outcome__ = map.next_value()?;
                        }
                    }
                }
                Ok(proposal_state::Claimed {
                    outcome: outcome__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalState.Claimed", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_state::Finished {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.outcome.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalState.Finished", len)?;
        if let Some(v) = self.outcome.as_ref() {
            struct_ser.serialize_field("outcome", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_state::Finished {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "outcome",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Outcome,
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
                            "outcome" => Ok(GeneratedField::Outcome),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_state::Finished;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalState.Finished")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal_state::Finished, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut outcome__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Outcome => {
                            if outcome__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outcome"));
                            }
                            outcome__ = map.next_value()?;
                        }
                    }
                }
                Ok(proposal_state::Finished {
                    outcome: outcome__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalState.Finished", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_state::Voting {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalState.Voting", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_state::Voting {
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
            type Value = proposal_state::Voting;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalState.Voting")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal_state::Voting, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(proposal_state::Voting {
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalState.Voting", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for proposal_state::Withdrawn {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.reason.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalState.Withdrawn", len)?;
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for proposal_state::Withdrawn {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "reason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Reason,
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
                            "reason" => Ok(GeneratedField::Reason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = proposal_state::Withdrawn;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalState.Withdrawn")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<proposal_state::Withdrawn, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut reason__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(proposal_state::Withdrawn {
                    reason: reason__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalState.Withdrawn", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalSubmit {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal.is_some() {
            len += 1;
        }
        if self.deposit_amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalSubmit", len)?;
        if let Some(v) = self.proposal.as_ref() {
            struct_ser.serialize_field("proposal", v)?;
        }
        if let Some(v) = self.deposit_amount.as_ref() {
            struct_ser.serialize_field("depositAmount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalSubmit {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "deposit_amount",
            "depositAmount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            DepositAmount,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "depositAmount" | "deposit_amount" => Ok(GeneratedField::DepositAmount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalSubmit;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalSubmit")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ProposalSubmit, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut deposit_amount__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = map.next_value()?;
                        }
                        GeneratedField::DepositAmount => {
                            if deposit_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("depositAmount"));
                            }
                            deposit_amount__ = map.next_value()?;
                        }
                    }
                }
                Ok(ProposalSubmit {
                    proposal: proposal__,
                    deposit_amount: deposit_amount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalSubmit", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalWithdraw {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal != 0 {
            len += 1;
        }
        if !self.reason.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ProposalWithdraw", len)?;
        if self.proposal != 0 {
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProposalWithdraw {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "reason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            Reason,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "reason" => Ok(GeneratedField::Reason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalWithdraw;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ProposalWithdraw")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ProposalWithdraw, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut reason__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ProposalWithdraw {
                    proposal: proposal__.unwrap_or_default(),
                    reason: reason__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ProposalWithdraw", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorVote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.body.is_some() {
            len += 1;
        }
        if self.auth_sig.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ValidatorVote", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.auth_sig.as_ref() {
            struct_ser.serialize_field("authSig", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorVote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body",
            "auth_sig",
            "authSig",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Body,
            AuthSig,
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
                            "body" => Ok(GeneratedField::Body),
                            "authSig" | "auth_sig" => Ok(GeneratedField::AuthSig),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ValidatorVote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut auth_sig__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map.next_value()?;
                        }
                        GeneratedField::AuthSig => {
                            if auth_sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authSig"));
                            }
                            auth_sig__ = map.next_value()?;
                        }
                    }
                }
                Ok(ValidatorVote {
                    body: body__,
                    auth_sig: auth_sig__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ValidatorVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValidatorVoteBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal != 0 {
            len += 1;
        }
        if self.vote.is_some() {
            len += 1;
        }
        if self.identity_key.is_some() {
            len += 1;
        }
        if self.governance_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.ValidatorVoteBody", len)?;
        if self.proposal != 0 {
            struct_ser.serialize_field("proposal", ToString::to_string(&self.proposal).as_str())?;
        }
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        if let Some(v) = self.identity_key.as_ref() {
            struct_ser.serialize_field("identityKey", v)?;
        }
        if let Some(v) = self.governance_key.as_ref() {
            struct_ser.serialize_field("governanceKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValidatorVoteBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "vote",
            "identity_key",
            "identityKey",
            "governance_key",
            "governanceKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            Vote,
            IdentityKey,
            GovernanceKey,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "vote" => Ok(GeneratedField::Vote),
                            "identityKey" | "identity_key" => Ok(GeneratedField::IdentityKey),
                            "governanceKey" | "governance_key" => Ok(GeneratedField::GovernanceKey),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValidatorVoteBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.ValidatorVoteBody")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValidatorVoteBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut vote__ = None;
                let mut identity_key__ = None;
                let mut governance_key__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map.next_value()?;
                        }
                        GeneratedField::IdentityKey => {
                            if identity_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("identityKey"));
                            }
                            identity_key__ = map.next_value()?;
                        }
                        GeneratedField::GovernanceKey => {
                            if governance_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("governanceKey"));
                            }
                            governance_key__ = map.next_value()?;
                        }
                    }
                }
                Ok(ValidatorVoteBody {
                    proposal: proposal__.unwrap_or_default(),
                    vote: vote__,
                    identity_key: identity_key__,
                    governance_key: governance_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.ValidatorVoteBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Vote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vote != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.governance.v1alpha1.Vote", len)?;
        if self.vote != 0 {
            let v = vote::Vote::from_i32(self.vote)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.vote)))?;
            struct_ser.serialize_field("vote", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Vote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vote",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vote,
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
                            "vote" => Ok(GeneratedField::Vote),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Vote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.governance.v1alpha1.Vote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Vote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vote__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = Some(map.next_value::<vote::Vote>()? as i32);
                        }
                    }
                }
                Ok(Vote {
                    vote: vote__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.governance.v1alpha1.Vote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for vote::Vote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "VOTE_UNSPECIFIED",
            Self::Abstain => "VOTE_ABSTAIN",
            Self::Yes => "VOTE_YES",
            Self::No => "VOTE_NO",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for vote::Vote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "VOTE_UNSPECIFIED",
            "VOTE_ABSTAIN",
            "VOTE_YES",
            "VOTE_NO",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = vote::Vote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(vote::Vote::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(vote::Vote::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "VOTE_UNSPECIFIED" => Ok(vote::Vote::Unspecified),
                    "VOTE_ABSTAIN" => Ok(vote::Vote::Abstain),
                    "VOTE_YES" => Ok(vote::Vote::Yes),
                    "VOTE_NO" => Ok(vote::Vote::No),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
