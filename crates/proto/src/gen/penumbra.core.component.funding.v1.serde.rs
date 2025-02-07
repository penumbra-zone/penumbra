impl serde::Serialize for ActionLiquidityTournamentVote {
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
        if self.proof.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.ActionLiquidityTournamentVote", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.auth_sig.as_ref() {
            struct_ser.serialize_field("authSig", v)?;
        }
        if let Some(v) = self.proof.as_ref() {
            struct_ser.serialize_field("proof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionLiquidityTournamentVote {
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
                            "body" => Ok(GeneratedField::Body),
                            "authSig" | "auth_sig" => Ok(GeneratedField::AuthSig),
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionLiquidityTournamentVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.ActionLiquidityTournamentVote")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionLiquidityTournamentVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut auth_sig__ = None;
                let mut proof__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map_.next_value()?;
                        }
                        GeneratedField::AuthSig => {
                            if auth_sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authSig"));
                            }
                            auth_sig__ = map_.next_value()?;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionLiquidityTournamentVote {
                    body: body__,
                    auth_sig: auth_sig__,
                    proof: proof__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.ActionLiquidityTournamentVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionLiquidityTournamentVotePlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.incentivized.is_some() {
            len += 1;
        }
        if self.rewards_recipient.is_some() {
            len += 1;
        }
        if self.staked_note.is_some() {
            len += 1;
        }
        if self.staked_note_position != 0 {
            len += 1;
        }
        if self.start_position != 0 {
            len += 1;
        }
        if !self.randomizer.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_r.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_s.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.ActionLiquidityTournamentVotePlan", len)?;
        if let Some(v) = self.incentivized.as_ref() {
            struct_ser.serialize_field("incentivized", v)?;
        }
        if let Some(v) = self.rewards_recipient.as_ref() {
            struct_ser.serialize_field("rewardsRecipient", v)?;
        }
        if let Some(v) = self.staked_note.as_ref() {
            struct_ser.serialize_field("stakedNote", v)?;
        }
        if self.staked_note_position != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("stakedNotePosition", ToString::to_string(&self.staked_note_position).as_str())?;
        }
        if self.start_position != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startPosition", ToString::to_string(&self.start_position).as_str())?;
        }
        if !self.randomizer.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("randomizer", pbjson::private::base64::encode(&self.randomizer).as_str())?;
        }
        if !self.proof_blinding_r.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proofBlindingR", pbjson::private::base64::encode(&self.proof_blinding_r).as_str())?;
        }
        if !self.proof_blinding_s.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("proofBlindingS", pbjson::private::base64::encode(&self.proof_blinding_s).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionLiquidityTournamentVotePlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "incentivized",
            "rewards_recipient",
            "rewardsRecipient",
            "staked_note",
            "stakedNote",
            "staked_note_position",
            "stakedNotePosition",
            "start_position",
            "startPosition",
            "randomizer",
            "proof_blinding_r",
            "proofBlindingR",
            "proof_blinding_s",
            "proofBlindingS",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Incentivized,
            RewardsRecipient,
            StakedNote,
            StakedNotePosition,
            StartPosition,
            Randomizer,
            ProofBlindingR,
            ProofBlindingS,
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
                            "incentivized" => Ok(GeneratedField::Incentivized),
                            "rewardsRecipient" | "rewards_recipient" => Ok(GeneratedField::RewardsRecipient),
                            "stakedNote" | "staked_note" => Ok(GeneratedField::StakedNote),
                            "stakedNotePosition" | "staked_note_position" => Ok(GeneratedField::StakedNotePosition),
                            "startPosition" | "start_position" => Ok(GeneratedField::StartPosition),
                            "randomizer" => Ok(GeneratedField::Randomizer),
                            "proofBlindingR" | "proof_blinding_r" => Ok(GeneratedField::ProofBlindingR),
                            "proofBlindingS" | "proof_blinding_s" => Ok(GeneratedField::ProofBlindingS),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionLiquidityTournamentVotePlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.ActionLiquidityTournamentVotePlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionLiquidityTournamentVotePlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut incentivized__ = None;
                let mut rewards_recipient__ = None;
                let mut staked_note__ = None;
                let mut staked_note_position__ = None;
                let mut start_position__ = None;
                let mut randomizer__ = None;
                let mut proof_blinding_r__ = None;
                let mut proof_blinding_s__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Incentivized => {
                            if incentivized__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivized"));
                            }
                            incentivized__ = map_.next_value()?;
                        }
                        GeneratedField::RewardsRecipient => {
                            if rewards_recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewardsRecipient"));
                            }
                            rewards_recipient__ = map_.next_value()?;
                        }
                        GeneratedField::StakedNote => {
                            if staked_note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakedNote"));
                            }
                            staked_note__ = map_.next_value()?;
                        }
                        GeneratedField::StakedNotePosition => {
                            if staked_note_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stakedNotePosition"));
                            }
                            staked_note_position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::StartPosition => {
                            if start_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startPosition"));
                            }
                            start_position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Randomizer => {
                            if randomizer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("randomizer"));
                            }
                            randomizer__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofBlindingR => {
                            if proof_blinding_r__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofBlindingR"));
                            }
                            proof_blinding_r__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofBlindingS => {
                            if proof_blinding_s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofBlindingS"));
                            }
                            proof_blinding_s__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionLiquidityTournamentVotePlan {
                    incentivized: incentivized__,
                    rewards_recipient: rewards_recipient__,
                    staked_note: staked_note__,
                    staked_note_position: staked_note_position__.unwrap_or_default(),
                    start_position: start_position__.unwrap_or_default(),
                    randomizer: randomizer__.unwrap_or_default(),
                    proof_blinding_r: proof_blinding_r__.unwrap_or_default(),
                    proof_blinding_s: proof_blinding_s__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.ActionLiquidityTournamentVotePlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionLiquidityTournamentVoteView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.liquidity_tournament_vote.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.ActionLiquidityTournamentVoteView", len)?;
        if let Some(v) = self.liquidity_tournament_vote.as_ref() {
            match v {
                action_liquidity_tournament_vote_view::LiquidityTournamentVote::Visible(v) => {
                    struct_ser.serialize_field("visible", v)?;
                }
                action_liquidity_tournament_vote_view::LiquidityTournamentVote::Opaque(v) => {
                    struct_ser.serialize_field("opaque", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionLiquidityTournamentVoteView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "visible",
            "opaque",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Visible,
            Opaque,
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
                            "visible" => Ok(GeneratedField::Visible),
                            "opaque" => Ok(GeneratedField::Opaque),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionLiquidityTournamentVoteView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.ActionLiquidityTournamentVoteView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionLiquidityTournamentVoteView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut liquidity_tournament_vote__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if liquidity_tournament_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            liquidity_tournament_vote__ = map_.next_value::<::std::option::Option<_>>()?.map(action_liquidity_tournament_vote_view::LiquidityTournamentVote::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if liquidity_tournament_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            liquidity_tournament_vote__ = map_.next_value::<::std::option::Option<_>>()?.map(action_liquidity_tournament_vote_view::LiquidityTournamentVote::Opaque)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionLiquidityTournamentVoteView {
                    liquidity_tournament_vote: liquidity_tournament_vote__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.ActionLiquidityTournamentVoteView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for action_liquidity_tournament_vote_view::Opaque {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vote.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.ActionLiquidityTournamentVoteView.Opaque", len)?;
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for action_liquidity_tournament_vote_view::Opaque {
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
                            "vote" => Ok(GeneratedField::Vote),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = action_liquidity_tournament_vote_view::Opaque;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.ActionLiquidityTournamentVoteView.Opaque")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<action_liquidity_tournament_vote_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vote__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(action_liquidity_tournament_vote_view::Opaque {
                    vote: vote__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.ActionLiquidityTournamentVoteView.Opaque", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for action_liquidity_tournament_vote_view::Visible {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vote.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.ActionLiquidityTournamentVoteView.Visible", len)?;
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for action_liquidity_tournament_vote_view::Visible {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vote",
            "note",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vote,
            Note,
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
                            "vote" => Ok(GeneratedField::Vote),
                            "note" => Ok(GeneratedField::Note),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = action_liquidity_tournament_vote_view::Visible;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.ActionLiquidityTournamentVoteView.Visible")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<action_liquidity_tournament_vote_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vote__ = None;
                let mut note__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map_.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(action_liquidity_tournament_vote_view::Visible {
                    vote: vote__,
                    note: note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.ActionLiquidityTournamentVoteView.Visible", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventFundingStreamReward {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.recipient.is_empty() {
            len += 1;
        }
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.reward_amount.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.EventFundingStreamReward", len)?;
        if !self.recipient.is_empty() {
            struct_ser.serialize_field("recipient", &self.recipient)?;
        }
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.reward_amount.as_ref() {
            struct_ser.serialize_field("rewardAmount", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventFundingStreamReward {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "recipient",
            "epoch_index",
            "epochIndex",
            "reward_amount",
            "rewardAmount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Recipient,
            EpochIndex,
            RewardAmount,
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
                            "recipient" => Ok(GeneratedField::Recipient),
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "rewardAmount" | "reward_amount" => Ok(GeneratedField::RewardAmount),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventFundingStreamReward;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.EventFundingStreamReward")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventFundingStreamReward, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut recipient__ = None;
                let mut epoch_index__ = None;
                let mut reward_amount__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Recipient => {
                            if recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recipient"));
                            }
                            recipient__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RewardAmount => {
                            if reward_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewardAmount"));
                            }
                            reward_amount__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventFundingStreamReward {
                    recipient: recipient__.unwrap_or_default(),
                    epoch_index: epoch_index__.unwrap_or_default(),
                    reward_amount: reward_amount__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.EventFundingStreamReward", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventLqtDelegatorReward {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.reward_amount.is_some() {
            len += 1;
        }
        if self.delegation_tokens.is_some() {
            len += 1;
        }
        if self.address.is_some() {
            len += 1;
        }
        if self.incentivized_asset_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.EventLqtDelegatorReward", len)?;
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.reward_amount.as_ref() {
            struct_ser.serialize_field("rewardAmount", v)?;
        }
        if let Some(v) = self.delegation_tokens.as_ref() {
            struct_ser.serialize_field("delegationTokens", v)?;
        }
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        if let Some(v) = self.incentivized_asset_id.as_ref() {
            struct_ser.serialize_field("incentivizedAssetId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventLqtDelegatorReward {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch_index",
            "epochIndex",
            "reward_amount",
            "rewardAmount",
            "delegation_tokens",
            "delegationTokens",
            "address",
            "incentivized_asset_id",
            "incentivizedAssetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EpochIndex,
            RewardAmount,
            DelegationTokens,
            Address,
            IncentivizedAssetId,
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
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "rewardAmount" | "reward_amount" => Ok(GeneratedField::RewardAmount),
                            "delegationTokens" | "delegation_tokens" => Ok(GeneratedField::DelegationTokens),
                            "address" => Ok(GeneratedField::Address),
                            "incentivizedAssetId" | "incentivized_asset_id" => Ok(GeneratedField::IncentivizedAssetId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventLqtDelegatorReward;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.EventLqtDelegatorReward")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventLqtDelegatorReward, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch_index__ = None;
                let mut reward_amount__ = None;
                let mut delegation_tokens__ = None;
                let mut address__ = None;
                let mut incentivized_asset_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RewardAmount => {
                            if reward_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewardAmount"));
                            }
                            reward_amount__ = map_.next_value()?;
                        }
                        GeneratedField::DelegationTokens => {
                            if delegation_tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegationTokens"));
                            }
                            delegation_tokens__ = map_.next_value()?;
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::IncentivizedAssetId => {
                            if incentivized_asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivizedAssetId"));
                            }
                            incentivized_asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventLqtDelegatorReward {
                    epoch_index: epoch_index__.unwrap_or_default(),
                    reward_amount: reward_amount__,
                    delegation_tokens: delegation_tokens__,
                    address: address__,
                    incentivized_asset_id: incentivized_asset_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.EventLqtDelegatorReward", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventLqtPositionReward {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.reward_amount.is_some() {
            len += 1;
        }
        if self.position_id.is_some() {
            len += 1;
        }
        if self.incentivized_asset_id.is_some() {
            len += 1;
        }
        if self.tournament_volume.is_some() {
            len += 1;
        }
        if self.position_volume.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.EventLqtPositionReward", len)?;
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.reward_amount.as_ref() {
            struct_ser.serialize_field("rewardAmount", v)?;
        }
        if let Some(v) = self.position_id.as_ref() {
            struct_ser.serialize_field("positionId", v)?;
        }
        if let Some(v) = self.incentivized_asset_id.as_ref() {
            struct_ser.serialize_field("incentivizedAssetId", v)?;
        }
        if let Some(v) = self.tournament_volume.as_ref() {
            struct_ser.serialize_field("tournamentVolume", v)?;
        }
        if let Some(v) = self.position_volume.as_ref() {
            struct_ser.serialize_field("positionVolume", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventLqtPositionReward {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch_index",
            "epochIndex",
            "reward_amount",
            "rewardAmount",
            "position_id",
            "positionId",
            "incentivized_asset_id",
            "incentivizedAssetId",
            "tournament_volume",
            "tournamentVolume",
            "position_volume",
            "positionVolume",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EpochIndex,
            RewardAmount,
            PositionId,
            IncentivizedAssetId,
            TournamentVolume,
            PositionVolume,
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
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "rewardAmount" | "reward_amount" => Ok(GeneratedField::RewardAmount),
                            "positionId" | "position_id" => Ok(GeneratedField::PositionId),
                            "incentivizedAssetId" | "incentivized_asset_id" => Ok(GeneratedField::IncentivizedAssetId),
                            "tournamentVolume" | "tournament_volume" => Ok(GeneratedField::TournamentVolume),
                            "positionVolume" | "position_volume" => Ok(GeneratedField::PositionVolume),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventLqtPositionReward;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.EventLqtPositionReward")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventLqtPositionReward, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch_index__ = None;
                let mut reward_amount__ = None;
                let mut position_id__ = None;
                let mut incentivized_asset_id__ = None;
                let mut tournament_volume__ = None;
                let mut position_volume__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RewardAmount => {
                            if reward_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewardAmount"));
                            }
                            reward_amount__ = map_.next_value()?;
                        }
                        GeneratedField::PositionId => {
                            if position_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionId"));
                            }
                            position_id__ = map_.next_value()?;
                        }
                        GeneratedField::IncentivizedAssetId => {
                            if incentivized_asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivizedAssetId"));
                            }
                            incentivized_asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::TournamentVolume => {
                            if tournament_volume__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tournamentVolume"));
                            }
                            tournament_volume__ = map_.next_value()?;
                        }
                        GeneratedField::PositionVolume => {
                            if position_volume__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionVolume"));
                            }
                            position_volume__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventLqtPositionReward {
                    epoch_index: epoch_index__.unwrap_or_default(),
                    reward_amount: reward_amount__,
                    position_id: position_id__,
                    incentivized_asset_id: incentivized_asset_id__,
                    tournament_volume: tournament_volume__,
                    position_volume: position_volume__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.EventLqtPositionReward", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventLqtVote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.voting_power.is_some() {
            len += 1;
        }
        if self.incentivized_asset_id.is_some() {
            len += 1;
        }
        if self.incentivized.is_some() {
            len += 1;
        }
        if self.rewards_recipient.is_some() {
            len += 1;
        }
        if self.tx_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.EventLqtVote", len)?;
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.voting_power.as_ref() {
            struct_ser.serialize_field("votingPower", v)?;
        }
        if let Some(v) = self.incentivized_asset_id.as_ref() {
            struct_ser.serialize_field("incentivizedAssetId", v)?;
        }
        if let Some(v) = self.incentivized.as_ref() {
            struct_ser.serialize_field("incentivized", v)?;
        }
        if let Some(v) = self.rewards_recipient.as_ref() {
            struct_ser.serialize_field("rewardsRecipient", v)?;
        }
        if let Some(v) = self.tx_id.as_ref() {
            struct_ser.serialize_field("txId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventLqtVote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch_index",
            "epochIndex",
            "voting_power",
            "votingPower",
            "incentivized_asset_id",
            "incentivizedAssetId",
            "incentivized",
            "rewards_recipient",
            "rewardsRecipient",
            "tx_id",
            "txId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EpochIndex,
            VotingPower,
            IncentivizedAssetId,
            Incentivized,
            RewardsRecipient,
            TxId,
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
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "votingPower" | "voting_power" => Ok(GeneratedField::VotingPower),
                            "incentivizedAssetId" | "incentivized_asset_id" => Ok(GeneratedField::IncentivizedAssetId),
                            "incentivized" => Ok(GeneratedField::Incentivized),
                            "rewardsRecipient" | "rewards_recipient" => Ok(GeneratedField::RewardsRecipient),
                            "txId" | "tx_id" => Ok(GeneratedField::TxId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventLqtVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.EventLqtVote")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EventLqtVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch_index__ = None;
                let mut voting_power__ = None;
                let mut incentivized_asset_id__ = None;
                let mut incentivized__ = None;
                let mut rewards_recipient__ = None;
                let mut tx_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VotingPower => {
                            if voting_power__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingPower"));
                            }
                            voting_power__ = map_.next_value()?;
                        }
                        GeneratedField::IncentivizedAssetId => {
                            if incentivized_asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivizedAssetId"));
                            }
                            incentivized_asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::Incentivized => {
                            if incentivized__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivized"));
                            }
                            incentivized__ = map_.next_value()?;
                        }
                        GeneratedField::RewardsRecipient => {
                            if rewards_recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewardsRecipient"));
                            }
                            rewards_recipient__ = map_.next_value()?;
                        }
                        GeneratedField::TxId => {
                            if tx_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("txId"));
                            }
                            tx_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EventLqtVote {
                    epoch_index: epoch_index__.unwrap_or_default(),
                    voting_power: voting_power__,
                    incentivized_asset_id: incentivized_asset_id__,
                    incentivized: incentivized__,
                    rewards_recipient: rewards_recipient__,
                    tx_id: tx_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.EventLqtVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FundingParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.liquidity_tournament.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.FundingParameters", len)?;
        if let Some(v) = self.liquidity_tournament.as_ref() {
            struct_ser.serialize_field("liquidityTournament", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FundingParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "liquidity_tournament",
            "liquidityTournament",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LiquidityTournament,
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
                            "liquidityTournament" | "liquidity_tournament" => Ok(GeneratedField::LiquidityTournament),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FundingParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.FundingParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FundingParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut liquidity_tournament__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::LiquidityTournament => {
                            if liquidity_tournament__.is_some() {
                                return Err(serde::de::Error::duplicate_field("liquidityTournament"));
                            }
                            liquidity_tournament__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FundingParameters {
                    liquidity_tournament: liquidity_tournament__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.FundingParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for funding_parameters::LiquidityTournament {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.gauge_threshold_percent != 0 {
            len += 1;
        }
        if self.max_positions != 0 {
            len += 1;
        }
        if self.max_delegators != 0 {
            len += 1;
        }
        if self.delegator_share_percent != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.FundingParameters.LiquidityTournament", len)?;
        if self.gauge_threshold_percent != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("gaugeThresholdPercent", ToString::to_string(&self.gauge_threshold_percent).as_str())?;
        }
        if self.max_positions != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("maxPositions", ToString::to_string(&self.max_positions).as_str())?;
        }
        if self.max_delegators != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("maxDelegators", ToString::to_string(&self.max_delegators).as_str())?;
        }
        if self.delegator_share_percent != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("delegatorSharePercent", ToString::to_string(&self.delegator_share_percent).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for funding_parameters::LiquidityTournament {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "gauge_threshold_percent",
            "gaugeThresholdPercent",
            "max_positions",
            "maxPositions",
            "max_delegators",
            "maxDelegators",
            "delegator_share_percent",
            "delegatorSharePercent",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GaugeThresholdPercent,
            MaxPositions,
            MaxDelegators,
            DelegatorSharePercent,
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
                            "gaugeThresholdPercent" | "gauge_threshold_percent" => Ok(GeneratedField::GaugeThresholdPercent),
                            "maxPositions" | "max_positions" => Ok(GeneratedField::MaxPositions),
                            "maxDelegators" | "max_delegators" => Ok(GeneratedField::MaxDelegators),
                            "delegatorSharePercent" | "delegator_share_percent" => Ok(GeneratedField::DelegatorSharePercent),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = funding_parameters::LiquidityTournament;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.FundingParameters.LiquidityTournament")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<funding_parameters::LiquidityTournament, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut gauge_threshold_percent__ = None;
                let mut max_positions__ = None;
                let mut max_delegators__ = None;
                let mut delegator_share_percent__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::GaugeThresholdPercent => {
                            if gauge_threshold_percent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gaugeThresholdPercent"));
                            }
                            gauge_threshold_percent__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxPositions => {
                            if max_positions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxPositions"));
                            }
                            max_positions__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MaxDelegators => {
                            if max_delegators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxDelegators"));
                            }
                            max_delegators__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DelegatorSharePercent => {
                            if delegator_share_percent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorSharePercent"));
                            }
                            delegator_share_percent__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(funding_parameters::LiquidityTournament {
                    gauge_threshold_percent: gauge_threshold_percent__.unwrap_or_default(),
                    max_positions: max_positions__.unwrap_or_default(),
                    max_delegators: max_delegators__.unwrap_or_default(),
                    delegator_share_percent: delegator_share_percent__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.FundingParameters.LiquidityTournament", FIELDS, GeneratedVisitor)
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
        if self.funding_params.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.GenesisContent", len)?;
        if let Some(v) = self.funding_params.as_ref() {
            struct_ser.serialize_field("fundingParams", v)?;
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
            "funding_params",
            "fundingParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FundingParams,
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
                            "fundingParams" | "funding_params" => Ok(GeneratedField::FundingParams),
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
                formatter.write_str("struct penumbra.core.component.funding.v1.GenesisContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenesisContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut funding_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FundingParams => {
                            if funding_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fundingParams"));
                            }
                            funding_params__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenesisContent {
                    funding_params: funding_params__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.GenesisContent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LiquidityTournamentVoteBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.incentivized.is_some() {
            len += 1;
        }
        if self.rewards_recipient.is_some() {
            len += 1;
        }
        if self.start_position != 0 {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.rk.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.LiquidityTournamentVoteBody", len)?;
        if let Some(v) = self.incentivized.as_ref() {
            struct_ser.serialize_field("incentivized", v)?;
        }
        if let Some(v) = self.rewards_recipient.as_ref() {
            struct_ser.serialize_field("rewardsRecipient", v)?;
        }
        if self.start_position != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("startPosition", ToString::to_string(&self.start_position).as_str())?;
        }
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
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
impl<'de> serde::Deserialize<'de> for LiquidityTournamentVoteBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "incentivized",
            "rewards_recipient",
            "rewardsRecipient",
            "start_position",
            "startPosition",
            "value",
            "nullifier",
            "rk",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Incentivized,
            RewardsRecipient,
            StartPosition,
            Value,
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
                            "incentivized" => Ok(GeneratedField::Incentivized),
                            "rewardsRecipient" | "rewards_recipient" => Ok(GeneratedField::RewardsRecipient),
                            "startPosition" | "start_position" => Ok(GeneratedField::StartPosition),
                            "value" => Ok(GeneratedField::Value),
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
            type Value = LiquidityTournamentVoteBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.LiquidityTournamentVoteBody")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LiquidityTournamentVoteBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut incentivized__ = None;
                let mut rewards_recipient__ = None;
                let mut start_position__ = None;
                let mut value__ = None;
                let mut nullifier__ = None;
                let mut rk__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Incentivized => {
                            if incentivized__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incentivized"));
                            }
                            incentivized__ = map_.next_value()?;
                        }
                        GeneratedField::RewardsRecipient => {
                            if rewards_recipient__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rewardsRecipient"));
                            }
                            rewards_recipient__ = map_.next_value()?;
                        }
                        GeneratedField::StartPosition => {
                            if start_position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startPosition"));
                            }
                            start_position__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
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
                Ok(LiquidityTournamentVoteBody {
                    incentivized: incentivized__,
                    rewards_recipient: rewards_recipient__,
                    start_position: start_position__.unwrap_or_default(),
                    value: value__,
                    nullifier: nullifier__,
                    rk: rk__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.LiquidityTournamentVoteBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LqtCheckNullifierRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.epoch_index != 0 {
            len += 1;
        }
        if self.nullifier.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.LqtCheckNullifierRequest", len)?;
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LqtCheckNullifierRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "epoch_index",
            "epochIndex",
            "nullifier",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EpochIndex,
            Nullifier,
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
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LqtCheckNullifierRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.LqtCheckNullifierRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LqtCheckNullifierRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut epoch_index__ = None;
                let mut nullifier__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LqtCheckNullifierRequest {
                    epoch_index: epoch_index__.unwrap_or_default(),
                    nullifier: nullifier__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.LqtCheckNullifierRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LqtCheckNullifierResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.transaction.is_some() {
            len += 1;
        }
        if self.already_voted {
            len += 1;
        }
        if self.epoch_index != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.LqtCheckNullifierResponse", len)?;
        if let Some(v) = self.transaction.as_ref() {
            struct_ser.serialize_field("transaction", v)?;
        }
        if self.already_voted {
            struct_ser.serialize_field("alreadyVoted", &self.already_voted)?;
        }
        if self.epoch_index != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("epochIndex", ToString::to_string(&self.epoch_index).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for LqtCheckNullifierResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "transaction",
            "already_voted",
            "alreadyVoted",
            "epoch_index",
            "epochIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Transaction,
            AlreadyVoted,
            EpochIndex,
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
                            "transaction" => Ok(GeneratedField::Transaction),
                            "alreadyVoted" | "already_voted" => Ok(GeneratedField::AlreadyVoted),
                            "epochIndex" | "epoch_index" => Ok(GeneratedField::EpochIndex),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LqtCheckNullifierResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.LqtCheckNullifierResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<LqtCheckNullifierResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut transaction__ = None;
                let mut already_voted__ = None;
                let mut epoch_index__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Transaction => {
                            if transaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transaction"));
                            }
                            transaction__ = map_.next_value()?;
                        }
                        GeneratedField::AlreadyVoted => {
                            if already_voted__.is_some() {
                                return Err(serde::de::Error::duplicate_field("alreadyVoted"));
                            }
                            already_voted__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EpochIndex => {
                            if epoch_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochIndex"));
                            }
                            epoch_index__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(LqtCheckNullifierResponse {
                    transaction: transaction__,
                    already_voted: already_voted__.unwrap_or_default(),
                    epoch_index: epoch_index__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.LqtCheckNullifierResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ZkLiquidityTournamentVoteProof {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.component.funding.v1.ZKLiquidityTournamentVoteProof", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ZkLiquidityTournamentVoteProof {
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
            type Value = ZkLiquidityTournamentVoteProof;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.component.funding.v1.ZKLiquidityTournamentVoteProof")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ZkLiquidityTournamentVoteProof, V::Error>
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
                Ok(ZkLiquidityTournamentVoteProof {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.component.funding.v1.ZKLiquidityTournamentVoteProof", FIELDS, GeneratedVisitor)
    }
}
