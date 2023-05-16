use penumbra_crypto::dex::lp::position::Position;
use penumbra_crypto::dex::lp::Reserves;
use penumbra_crypto::dex::DirectedUnitPair;
use rand_core::OsRng;
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
    pub fee: u128,
    pub p: u128,
    pub q: u128,
    pub k: u128,
    pub r1: u128,
    pub r2: u128,
}

impl From<Position> for PayoffPosition {
    fn from(value: Position) -> Self {
        let p = value.phi.component.p.value();
        let q = value.phi.component.q.value();
        let r1 = value.reserves.r1.value();
        let r2 = value.reserves.r2.value();
        let k = p * r1 + q * r2;
        let fee = value.phi.component.fee as u128;
        Self {
            fee,
            p,
            q,
            k,
            r1,
            r2,
        }
    }
}

impl From<PayoffPositionEntry> for Position {
    fn from(entry: PayoffPositionEntry) -> Self {
        Position::new(
            OsRng,
            entry.pair.into_directed_trading_pair(),
            entry.payoff.fee as u32,
            entry.payoff.p.into(),
            entry.payoff.q.into(),
            Reserves {
                r1: entry.payoff.r1.into(),
                r2: entry.payoff.r2.into(),
            },
        )
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
