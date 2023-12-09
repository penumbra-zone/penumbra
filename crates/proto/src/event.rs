use anyhow;
use std::collections::HashMap;
use tendermint::abci::{self, EventAttribute};
use crate::{Name, Message};
use serde::{Serialize, Deserialize};

pub trait ProtoEvent<'a>: Message + Name + Serialize + Deserialize<'a> + Sized {
    fn into_event(&self) -> abci::Event {

        let kind  = Self::NAME;

        let event_json = serde_json::to_value(&self).expect("ProtoEvent constrained values should be JSON serializeable.");

        // WARNING: Assuming that Rust value will always serialize into a valid JSON Object value. This falls apart the moment that isn't true, so we fail hard if that turns out to be the case.
        let mut attributes : Vec<EventAttribute> = event_json
            .as_object()
            .expect("serde_json Serialized ProtoEvent should not be empty.")
            .into_iter()
            .map(|(key, v)| abci::EventAttribute {
                value: serde_json::from_value(v.clone()).expect(&format!("serde_json Value for EventAttribute should have a value for key {}", key)),
                key: key.to_string(),
                index: true
            })
            .collect();


        // NOTE: cosmo-sdk sorts the attribute list so that it's deterministic every time.[0] I don't know if that is actually conformant but continuing that pattern here for now.
        // [0]: https://github.com/cosmos/cosmos-sdk/blob/8fb62054c59e580c0ae0c898751f8dc46044499a/types/events.go#L102-L104
        attributes.sort_by(|a, b| (&a.key).cmp(&b.key));

        return abci::Event::new(kind, attributes)
    }

    fn from_event(event: &abci::Event) -> anyhow::Result<Self> {
        // This doesn't seem right but I'm not sure what the corallary is for runtime reflection is for inspecting whether a given Event is valid.
        let event_type = &event.kind;

        if Self::NAME != event_type {
            return Err(anyhow::anyhow!(format!("ABCI Event {} not expected for {}", event_type, Self::NAME)))
        }

        // NOTE: Is there any condition where there would be duplicate EventAttributes and problems that fall out of that?
        let attributes = event
            .clone()
            .attributes
            .into_iter()
            .map(|x| (x.key, serde_json::to_value(x.value).expect("EventAttribute Value should be Serializeable.")))
            .collect::<HashMap<String, serde_json::Value>>();

        let json = serde_json::to_value(attributes).expect("HashMap of String, serde_json::Value should be serializeable.");

        // Hitting a lifetime error that requires further restriction on Self: Deserialize that conflicts with the current supertraits defined above.
        // return Ok(serde_json::from_value(json)?)
        unimplemented!()
    }
}

#[cfg(test)]
mod proto_event_tests {
    #[test]
    fn test_into_event() {
        use super::*;
        use crate::core::component::shielded_pool::v1alpha1::EventSpend;
        let event = EventSpend {
            nullifier: vec![148 ,190 ,149 ,23 ,86 ,113 ,152 ,145 ,104 ,242 ,142 ,162 ,233 ,239 ,137 ,141 ,140 ,164 ,180 ,98 ,154 ,55 ,168 ,255 ,163 ,228 ,179 ,176 ,26 ,25 ,219 ,211]
        };
    }
}