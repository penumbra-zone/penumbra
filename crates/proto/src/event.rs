use crate::{Message, Name};
use anyhow::{self, Context};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use tendermint::abci::{self, EventAttribute};

pub trait ProtoEvent: Message + Name + Serialize + DeserializeOwned + Sized {
    fn into_event(&self) -> abci::Event {
        let kind = Self::full_name();

        let event_json = serde_json::to_value(&self)
            .expect("ProtoEvent constrained values should be JSON serializeable.");

        // WARNING: Assuming that Rust value will always serialize into a valid JSON Object value. This falls apart the moment that isn't true, so we fail hard if that turns out to be the case.
        let mut attributes: Vec<EventAttribute> = event_json
            .as_object()
            .expect("serde_json Serialized ProtoEvent should not be empty.")
            .into_iter()
            .map(|(key, v)| abci::EventAttribute {
                value: serde_json::to_string(v).expect("must be able to serialize value as JSON"),
                key: key.to_string(),
                index: true,
            })
            .collect();

        // NOTE: cosmo-sdk sorts the attribute list so that it's deterministic every time.[0] I don't know if that is actually conformant but continuing that pattern here for now.
        // [0]: https://github.com/cosmos/cosmos-sdk/blob/8fb62054c59e580c0ae0c898751f8dc46044499a/types/events.go#L102-L104
        attributes.sort_by(|a, b| (&a.key).cmp(&b.key));

        return abci::Event::new(kind, attributes);
    }

    fn from_event(event: &abci::Event) -> anyhow::Result<Self> {
        // Check that we're dealing with the right type of event.
        if Self::full_name() != event.kind {
            return Err(anyhow::anyhow!(format!(
                "ABCI Event {} not expected for {}",
                event.kind,
                Self::full_name()
            )));
        }

        // NOTE: Is there any condition where there would be duplicate EventAttributes and problems that fall out of that?
        let mut attributes = HashMap::<String, serde_json::Value>::new();
        for attr in &event.attributes {
            let value = serde_json::from_str(&attr.value)
                .with_context(|| format!("could not parse JSON for attribute {:?}", attr))?;
            attributes.insert(attr.key.clone(), value);
        }

        let json = serde_json::to_value(attributes)
            .expect("HashMap of String, serde_json::Value should be serializeable.");

        return Ok(
            serde_json::from_value(json).context("could not deserialise ProtoJSON into event")?
        );
    }
}

impl<E: Message + Name + Serialize + DeserializeOwned + Sized> ProtoEvent for E {}

#[cfg(test)]
mod tests {
    #[test]
    fn event_round_trip() {
        use super::*;
        use crate::core::component::sct::v1alpha1::Nullifier;
        use crate::core::component::shielded_pool::v1alpha1::{EventOutput, EventSpend};
        use crate::crypto::tct::v1alpha1::StateCommitment;

        let proto_spend = EventSpend {
            nullifier: Some(Nullifier {
                inner: vec![
                    148, 190, 149, 23, 86, 113, 152, 145, 104, 242, 142, 162, 233, 239, 137, 141,
                    140, 164, 180, 98, 154, 55, 168, 255, 163, 228, 179, 176, 26, 25, 219, 211,
                ],
            }),
        };

        let abci_spend = proto_spend.into_event();

        let expected_abci_spend = abci::Event::new(
            "penumbra.core.component.shielded_pool.v1alpha1.EventSpend",
            vec![abci::EventAttribute {
                key: "nullifier".to_string(),
                value: "{\"inner\":\"lL6VF1ZxmJFo8o6i6e+JjYyktGKaN6j/o+SzsBoZ29M=\"}".to_string(),
                index: true,
            }],
        );
        assert_eq!(abci_spend, expected_abci_spend);

        let proto_spend2 = EventSpend::from_event(&abci_spend).unwrap();

        assert_eq!(proto_spend, proto_spend2);

        let proto_output = EventOutput {
            // This is the same bytes as the nullifier above, we just care about the data format, not the value.
            note_commitment: Some(StateCommitment {
                inner: vec![
                    148, 190, 149, 23, 86, 113, 152, 145, 104, 242, 142, 162, 233, 239, 137, 141,
                    140, 164, 180, 98, 154, 55, 168, 255, 163, 228, 179, 176, 26, 25, 219, 211,
                ],
            }),
        };

        let abci_output = proto_output.into_event();

        let expected_abci_output = abci::Event::new(
            "penumbra.core.component.shielded_pool.v1alpha1.EventOutput",
            vec![abci::EventAttribute {
                // note: attribute keys become camelCase because ProtoJSON...
                key: "noteCommitment".to_string(),
                // note: attribute values are JSON objects, potentially nested as here
                value: "{\"inner\":\"lL6VF1ZxmJFo8o6i6e+JjYyktGKaN6j/o+SzsBoZ29M=\"}".to_string(),
                index: true,
            }],
        );
        assert_eq!(abci_output, expected_abci_output);

        let proto_output2 = EventOutput::from_event(&abci_output).unwrap();
        assert_eq!(proto_output, proto_output2);
    }
}
