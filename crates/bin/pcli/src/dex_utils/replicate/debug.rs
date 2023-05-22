use penumbra_crypto::fixpoint::U128x128;
use penumbra_dex::lp::position::Position;
use penumbra_dex::DirectedUnitPair;
use serde::Serialize;

#[derive(Serialize)]
pub struct PayoffPositionEntry {
    #[serde(
        serialize_with = "serialize_directed_unit_pair_to_canon",
        rename = "canonical_pair"
    )]
    pub pair: DirectedUnitPair,
    pub payoff: PayoffPosition,
    pub current_price: f64,
    pub index: usize,
    pub alpha: f64,
    pub total_k: f64,
}

/// For debugging purposes. We want to be able to serialize a position
/// to JSON so that we can pipe it into a Julia notebook. The reason why
/// this is a separate structure from [`position::Position`] is that we
/// might want to do extra processing, rounding, etc. and we'd rather note
/// clutter it with serializiation methods that are useful for narrow purposes.
#[derive(Serialize)]
pub struct PayoffPosition {
    pub p: f64,
    pub q: f64,
    pub k: f64,
    pub r1: f64,
    pub r2: f64,
    pub fee: f64,
}

impl PayoffPosition {
    pub fn from_position(pair: DirectedUnitPair, position: Position) -> PayoffPosition {
        let oriented_phi = position.phi.orient_end(pair.end.id()).unwrap();
        let p = U128x128::ratio(oriented_phi.p.value(), pair.end.unit_amount().value())
            .unwrap()
            .into();
        let q = U128x128::ratio(oriented_phi.q.value(), pair.start.unit_amount().value())
            .unwrap()
            .into();

        let r1 = position.reserves_for(pair.start.id()).unwrap();
        let r2 = position.reserves_for(pair.end.id()).unwrap();
        let r1 = U128x128::ratio(r1.value(), pair.start.unit_amount().value())
            .unwrap()
            .into();
        let r2 = U128x128::ratio(r2.value(), pair.end.unit_amount().value())
            .unwrap()
            .into();
        let k = p * r1 + q * r2;
        let k = k / p;
        let fee = position.phi.component.fee as f64;
        PayoffPosition {
            fee,
            p,
            q,
            k,
            r1,
            r2,
        }
    }
}

fn serialize_directed_unit_pair_to_canon<S>(
    pair: &DirectedUnitPair,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&pair.to_canonical_string())
}
