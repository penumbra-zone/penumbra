use anyhow;
use std::collections::HashMap;
use tendermint::abci;
use crate::{Name, Message};


pub trait ProtoEvent: Message + Name + Sized {

    fn into_event(&self) -> abci::Event {

        let kind  = Self::NAME;

        // NOTE: Clarify if the serde derivation is pbjson/prost aware. Or, more specifically, if Penumbra generated serialization code for protobuf defs are more coherent than the 
        //       3-4 different ways the cosmo-sdk chooses to serializes and deserialize its protobuf data in the reference code.
        // TODO: Would it be preferred to return an empty(?) or error coded abci::Event?
        let event_json = serde_json::to_vec(&self.encode_to_vec()).expect("ProtoEvent constrained values should be JSON serializeable.");

        let attribute_kv : HashMap<String, String> = serde_json::from_slice(&event_json).expect("A serde derived Vec<u8> should be deserializeable.");

        let mut attributes = Vec::<abci::EventAttribute>::with_capacity(attribute_kv.len());
        let mut keys = Vec::from_iter(attribute_kv.clone().into_keys());

        keys.sort();

        for key in keys {
            attributes.push(abci::EventAttribute {
                value: attribute_kv
                        .get(&key)
                        .expect(&format!("EventAttribute map should have a value for key {}", key))
                        .clone(),
                index: true,
                key
            })
        }

        return abci::Event::new(kind, attributes)
    }

    fn from_event(event: &abci::Event) -> anyhow::Result<Self> {
        // This doesn't seem right but I'm not sure what the corallary is for runtime reflection is for inspecting whether a given Event is valid.
        let event_type = &event.kind;

        if Self::NAME != event_type {
            return Err(anyhow::anyhow!(format!("ABCI Event {} not expected for {}", event_type, Self::NAME)))
        }

        let mut attributes_kv = HashMap::<String, Vec<u8>>::new();

        for attr in &event.attributes {
            let _ = attributes_kv.insert(
                attr.key.clone(),
                serde_json::to_vec(&attr.value).expect("EventAttribute values should be serializeable.")
            );
        }

        // NOTE: Same thing here as into_event(), check what the equivalent to the jsonpb Unmarshaler is.
        let json_bytes = serde_json::to_vec(&attributes_kv).expect("Hashmap of EventAttribute Keys and values should be serializeable.");
        // let decoded_event = Self::decode(serde_json::from_slice(&json_bytes).expect("Serde derived serialized bytes should be deserializeable."));
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