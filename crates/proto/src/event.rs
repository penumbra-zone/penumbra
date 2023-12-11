use crate::{Message, Name};
use anyhow;
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
                value: serde_json::from_value(v.clone()).expect(&format!(
                    "serde_json Value for EventAttribute should have a value for key {}",
                    key
                )),
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
        let attributes = event
            .clone()
            .attributes
            .into_iter()
            .map(|x| {
                (
                    x.key,
                    serde_json::to_value(x.value)
                        .expect("EventAttribute Value should be Serializeable."),
                )
            })
            .collect::<HashMap<String, serde_json::Value>>();

        let json = serde_json::to_value(attributes)
            .expect("HashMap of String, serde_json::Value should be serializeable.");

        return Ok(serde_json::from_value(json)?);
    }
}

impl<E: Message + Name + Serialize + DeserializeOwned + Sized> ProtoEvent for E {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_into_event() {
        use super::*;
        use crate::core::component::shielded_pool::v1alpha1::EventSpend;
        let proto_event = EventSpend {
            nullifier: vec![
                148, 190, 149, 23, 86, 113, 152, 145, 104, 242, 142, 162, 233, 239, 137, 141, 140,
                164, 180, 98, 154, 55, 168, 255, 163, 228, 179, 176, 26, 25, 219, 211,
            ],
        };

        let abci_event = proto_event.into_event();

        let expected = abci::Event::new(
            "penumbra.core.component.shielded_pool.v1alpha1.EventSpend",
            vec![abci::EventAttribute {
                key: "nullifier".to_string(),
                value: "lL6VF1ZxmJFo8o6i6e+JjYyktGKaN6j/o+SzsBoZ29M=".to_string(),
                index: true,
            }],
        );
        assert_eq!(abci_event, expected);

        let proto_event2 = EventSpend::from_event(&abci_event).unwrap();

        assert_eq!(proto_event, proto_event2);
    }
}
