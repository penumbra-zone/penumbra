use anyhow::{anyhow, Context};
use penumbra_sdk_asset::{asset, Value};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{
    penumbra::core::component::dex::v1 as pb, serializers::bech32str, DomainType,
};
use rand_core::CryptoRngCore;
use serde::{Deserialize, Serialize};

use crate::{DirectedTradingPair, TradingPair};

use super::{trading_function::TradingFunction, Reserves};

/// Reserve amounts for positions must be at most 80 bits wide.
pub const MAX_RESERVE_AMOUNT: u128 = (1 << 80) - 1;

/// A trading function's fee (spread) must be at most 50% (5000 bps)
pub const MAX_FEE_BPS: u32 = 5000;

/// Encapsulates the immutable parts of the position (phi/nonce), along
/// with the mutable parts (state/reserves).
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::Position", into = "pb::Position")]
pub struct Position {
    pub state: State,
    pub reserves: Reserves,
    /// A trading function to a specific trading pair.
    pub phi: TradingFunction,
    /// A random value used to disambiguate different positions with the exact
    /// same trading function.  The position ID is a hash of the trading
    /// function and the nonce; the chain rejects transactions creating
    /// duplicate position [`Id`]s, so it can track position ownership with a
    /// sequence of stateful NFTs based on the [`Id`].
    pub nonce: [u8; 32],
    /// Set to `true` if a position is a limit-order, meaning that it will be closed after being
    /// filled against.
    pub close_on_fill: bool,
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Position")
            .field("state", &self.state)
            .field("reserves", &self.reserves)
            .field("phi", &self.phi)
            .field("nonce", &hex::encode(self.nonce))
            .finish()
    }
}

impl Position {
    /// Construct a new opened [Position] with a random nonce.
    ///
    /// The `p` value is the coefficient for the position's trading function that will be
    /// associated with the start asset, and the `q` value is the coefficient for the end asset.
    ///
    /// The reserves `r1` and `r2` also correspond to the start and end assets, respectively.
    pub fn new<R: CryptoRngCore>(
        mut rng: R,
        pair: DirectedTradingPair,
        fee: u32,
        p: Amount,
        q: Amount,
        reserves: Reserves,
    ) -> Position {
        // Internally mutable so we can swap the `p` and `q` values if necessary.
        let mut p = p;
        let mut q = q;
        let mut reserves = reserves;

        let mut nonce_bytes = [0u8; 32];
        rng.fill_bytes(&mut nonce_bytes);

        // The [`TradingFunction`] uses a canonical non-directed trading pair ([`TradingPair`]).
        // This means that the `p` and `q` values may need to be swapped, depending on the canonical
        // representation of the trading pair.
        let canonical_tp: TradingPair = pair.into();

        // The passed-in `p` value is associated with the start asset, as is `r1`.
        if pair.start != canonical_tp.asset_1() {
            // The canonical representation of the trading pair has the start asset as the second
            // asset, so we need to swap the `p` and `q` values.
            std::mem::swap(&mut p, &mut q);

            // The ordering of the reserves should also be swapped.
            reserves = Reserves {
                r1: reserves.r2,
                r2: reserves.r1,
            };
        }

        let phi = TradingFunction::new(canonical_tp, fee, p, q);
        Position {
            phi,
            nonce: nonce_bytes,
            state: State::Opened,
            reserves,
            close_on_fill: false,
        }
    }

    /// Construct a new opened [Position] with a supplied random nonce.
    pub fn new_with_nonce(
        nonce: [u8; 32],
        pair: DirectedTradingPair,
        fee: u32,
        p: Amount,
        q: Amount,
        reserves: Reserves,
    ) -> Position {
        // Internally mutable so we can swap the `p` and `q` values if necessary.
        let mut p = p;
        let mut q = q;
        let mut reserves = reserves;

        // The [`TradingFunction`] uses a canonical non-directed trading pair ([`TradingPair`]).
        // This means that the `p` and `q` values may need to be swapped, depending on the canonical
        // representation of the trading pair.
        let canonical_tp: TradingPair = pair.into();

        // The passed-in `p` value is associated with the start asset, as is `r1`.
        if pair.start != canonical_tp.asset_1() {
            // The canonical representation of the trading pair has the start asset as the second
            // asset, so we need to swap the `p` and `q` values.
            std::mem::swap(&mut p, &mut q);

            // The ordering of the reserves should also be swapped.
            reserves = Reserves {
                r1: reserves.r2,
                r2: reserves.r1,
            };
        }

        let phi = TradingFunction::new(canonical_tp, fee, p, q);
        Position {
            phi,
            nonce,
            state: State::Opened,
            reserves,
            close_on_fill: false,
        }
    }

    /// Get the ID of this position.
    pub fn id(&self) -> Id {
        let mut state = blake2b_simd::Params::default()
            .personal(b"penumbra_lp_id")
            .to_state();

        state.update(&self.nonce);
        state.update(&self.phi.pair.asset_1().to_bytes());
        state.update(&self.phi.pair.asset_2().to_bytes());
        state.update(&self.phi.component.fee.to_le_bytes());
        state.update(&self.phi.component.p.to_le_bytes());
        state.update(&self.phi.component.q.to_le_bytes());

        let hash = state.finalize();
        let mut bytes = [0; 32];
        bytes[0..32].copy_from_slice(&hash.as_bytes()[0..32]);
        Id(bytes)
    }

    pub fn check_stateless(&self) -> anyhow::Result<()> {
        if self.reserves.r1.value() > MAX_RESERVE_AMOUNT
            || self.reserves.r2.value() > MAX_RESERVE_AMOUNT
        {
            Err(anyhow::anyhow!(format!(
                "Reserve amounts are out-of-bounds (limit: {MAX_RESERVE_AMOUNT})"
            )))
        } else if self.reserves.r1.value() == 0 && self.reserves.r2.value() == 0 {
            Err(anyhow::anyhow!(
                "initial reserves must provision some amount of either asset",
            ))
        } else if self.phi.component.p == 0u64.into() || self.phi.component.q == 0u64.into() {
            Err(anyhow::anyhow!(
                "trading function coefficients must be nonzero"
            ))
        } else if self.phi.component.p.value() > MAX_RESERVE_AMOUNT
            || self.phi.component.q.value() > MAX_RESERVE_AMOUNT
        {
            Err(anyhow!("trading function coefficients are too large"))
        } else if self.phi.pair.asset_1() == self.phi.pair.asset_2() {
            Err(anyhow!("cyclical pairs aren't allowed"))
        } else if self.phi.component.fee > MAX_FEE_BPS {
            Err(anyhow!("fee cannot be greater than 50% (5000bps)"))
        } else {
            Ok(())
        }
    }

    /// Returns the amount of the given asset that is currently in the position's reserves.
    pub fn reserves_for(&self, asset: asset::Id) -> Option<Amount> {
        if asset == self.phi.pair.asset_1() {
            Some(self.reserves.r1)
        } else if asset == self.phi.pair.asset_2() {
            Some(self.reserves.r2)
        } else {
            None
        }
    }

    /// Returns the amount of reserves for asset 1.
    pub fn reserves_1(&self) -> Value {
        Value {
            amount: self.reserves.r1,
            asset_id: self.phi.pair.asset_1(),
        }
    }

    /// Returns the amount of reserves for asset 2.
    pub fn reserves_2(&self) -> Value {
        Value {
            amount: self.reserves.r2,
            asset_id: self.phi.pair.asset_2(),
        }
    }

    /// Compute the flows compared to a previous position.
    ///
    /// This takes in the desired first asset for the flows.
    ///
    /// This will fail, returning None, if this desired first asset doesn't match
    /// the position, or if the previous position's pair doesn't match either.
    pub fn flows(&self, prev: &Self) -> Flows {
        Flows::from_phi_and_reserves(&self.phi, &self.reserves, &prev.reserves)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Flows {
    pair: DirectedTradingPair,
    delta_1: Amount,
    delta_2: Amount,
    lambda_1: Amount,
    lambda_2: Amount,
    fee_1: Amount,
    fee_2: Amount,
}

impl Flows {
    pub fn pair(&self) -> DirectedTradingPair {
        self.pair
    }

    pub fn delta_1(&self) -> Amount {
        self.delta_1
    }

    pub fn delta_2(&self) -> Amount {
        self.delta_2
    }

    pub fn lambda_1(&self) -> Amount {
        self.lambda_1
    }

    pub fn lambda_2(&self) -> Amount {
        self.lambda_2
    }

    pub fn fee_1(&self) -> Amount {
        self.fee_1
    }

    pub fn fee_2(&self) -> Amount {
        self.fee_2
    }

    pub fn from_phi_and_reserves(
        phi: &TradingFunction,
        current: &Reserves,
        prev: &Reserves,
    ) -> Self {
        // Determine trade direction and compute deltas
        let (delta_1, delta_2, lambda_1, lambda_2) = if current.r1 > prev.r1 {
            // Asset 1 was input
            let delta_1 = current.r1 - prev.r1;
            let lambda_2 = prev.r2 - current.r2;
            (delta_1, Amount::zero(), Amount::zero(), lambda_2)
        } else {
            // Asset 2 was input
            let delta_2 = current.r2 - prev.r2;
            let lambda_1 = prev.r1 - current.r1;
            (Amount::zero(), delta_2, lambda_1, Amount::zero())
        };
        // Compute fees directly from input amounts using u128 arithmetic
        let fee_bps = u128::from(phi.component.fee);
        let fee_1 = Amount::from((delta_1.value() * fee_bps) / 10_000u128);
        let fee_2 = Amount::from((delta_2.value() * fee_bps) / 10_000u128);
        Self {
            pair: DirectedTradingPair {
                start: phi.pair.asset_1,
                end: phi.pair.asset_2,
            },
            delta_1,
            delta_2,
            lambda_1,
            lambda_2,
            fee_1,
            fee_2,
        }
    }

    /// Try to reorient these flows around a new trading pair.
    ///
    /// This will fail if the assets in the pair don't match these flows.
    pub fn redirect(self, pair: DirectedTradingPair) -> Option<Self> {
        if self.pair == pair {
            return Some(self);
        }
        let flip = pair.flip();
        if self.pair == flip {
            return Some(Self {
                pair: flip,
                delta_1: self.delta_2,
                delta_2: self.delta_1,
                lambda_1: self.lambda_2,
                lambda_2: self.lambda_1,
                fee_1: self.fee_2,
                fee_2: self.fee_1,
            });
        }
        None
    }
}

/// A hash of a [`Position`].
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::PositionId", into = "pb::PositionId")]
pub struct Id(pub [u8; 32]);

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&bech32str::encode(
            &self.0,
            bech32str::lp_id::BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&bech32str::encode(
            &self.0,
            bech32str::lp_id::BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

impl std::str::FromStr for Id {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = bech32str::decode(s, bech32str::lp_id::BECH32_PREFIX, bech32str::Bech32m)?;
        pb::PositionId {
            inner,
            alt_bech32m: String::new(),
        }
        .try_into()
    }
}

/// The state of a position.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::PositionState", into = "pb::PositionState")]
pub enum State {
    /// The position has been opened, is active, has reserves and accumulated
    /// fees, and can be traded against.
    Opened,
    /// The position has been closed, is inactive and can no longer be traded
    /// against, but still has reserves and accumulated fees.
    Closed,
    /// The final reserves and accumulated fees have been withdrawn, leaving an
    /// empty, inactive position awaiting (possible) retroactive rewards.
    Withdrawn {
        /// The sequence number, incrementing with each withdrawal.
        sequence: u64,
    },
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Opened => write!(f, "opened"),
            State::Closed => write!(f, "closed"),
            State::Withdrawn { sequence } => write!(f, "withdrawn_{}", sequence),
        }
    }
}

impl std::str::FromStr for State {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "opened" => Ok(State::Opened),
            "closed" => Ok(State::Closed),
            _ => {
                let mut parts = s.splitn(2, '_');
                if parts.next() == Some("withdrawn") {
                    let sequence = parts
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("missing sequence number"))?
                        .parse()?;
                    Ok(State::Withdrawn { sequence })
                } else {
                    Err(anyhow::anyhow!("unknown position state"))
                }
            }
        }
    }
}

// ==== Protobuf impls

impl DomainType for Position {
    type Proto = pb::Position;
}

impl DomainType for Id {
    type Proto = pb::PositionId;
}

impl TryFrom<pb::PositionId> for Id {
    type Error = anyhow::Error;

    fn try_from(value: pb::PositionId) -> Result<Self, Self::Error> {
        match (value.inner.is_empty(), value.alt_bech32m.is_empty()) {
            (false, true) => Ok(Id(value
                .inner
                .as_slice()
                .try_into()
                .context("expected 32-byte id")?)),
            (true, false) => value.alt_bech32m.parse(),
            (false, false) => Err(anyhow::anyhow!(
                "AssetId proto has both inner and alt_bech32m fields set"
            )),
            (true, true) => Err(anyhow::anyhow!(
                "AssetId proto has neither inner nor alt_bech32m fields set"
            )),
        }
    }
}

impl From<Id> for pb::PositionId {
    fn from(value: Id) -> Self {
        Self {
            inner: value.0.to_vec(),
            // Never produce a proto encoding with the alt field set.
            alt_bech32m: String::new(),
        }
    }
}

impl DomainType for State {
    type Proto = pb::PositionState;
}

impl From<State> for pb::PositionState {
    fn from(v: State) -> Self {
        pb::PositionState {
            state: match v {
                State::Opened => pb::position_state::PositionStateEnum::Opened,
                State::Closed => pb::position_state::PositionStateEnum::Closed,
                State::Withdrawn { .. } => pb::position_state::PositionStateEnum::Withdrawn,
            } as i32,
            sequence: match v {
                State::Withdrawn { sequence } => sequence,
                // This will be omitted from serialization.
                _ => 0,
            },
        }
    }
}

impl TryFrom<pb::PositionState> for State {
    type Error = anyhow::Error;
    fn try_from(v: pb::PositionState) -> Result<Self, Self::Error> {
        let position_state =
            pb::position_state::PositionStateEnum::try_from(v.state).map_err(|e| {
                anyhow::anyhow!(
                    "invalid position state enum value: {}, error: {}",
                    v.state,
                    e
                )
            })?;

        match position_state {
            pb::position_state::PositionStateEnum::Opened => Ok(State::Opened),
            pb::position_state::PositionStateEnum::Closed => Ok(State::Closed),
            pb::position_state::PositionStateEnum::Withdrawn => Ok(State::Withdrawn {
                sequence: v.sequence,
            }),
            _ => Err(anyhow!("unknown position state")),
        }
    }
}

impl From<Position> for pb::Position {
    fn from(p: Position) -> Self {
        Self {
            state: Some(p.state.into()),
            reserves: Some(p.reserves.into()),
            phi: Some(p.phi.into()),
            nonce: p.nonce.to_vec(),
            close_on_fill: p.close_on_fill,
        }
    }
}

impl TryFrom<pb::Position> for Position {
    type Error = anyhow::Error;
    fn try_from(p: pb::Position) -> Result<Self, Self::Error> {
        Ok(Self {
            state: p
                .state
                .ok_or_else(|| anyhow::anyhow!("missing state in Position message"))?
                .try_into()?,
            reserves: p
                .reserves
                .ok_or_else(|| anyhow::anyhow!("missing reserves in Position message"))?
                .try_into()?,
            phi: p
                .phi
                .ok_or_else(|| anyhow::anyhow!("missing trading function"))?
                .try_into()?,
            nonce: p
                .nonce
                .as_slice()
                .try_into()
                .context("expected 32-byte nonce")?,
            close_on_fill: p.close_on_fill,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_ff::Zero;
    use decaf377::Fq;
    use penumbra_sdk_asset::asset;
    use rand_core::OsRng;

    #[test]
    fn position_state_fromstr() {
        assert_eq!("opened".parse::<State>().unwrap(), State::Opened);
        assert_eq!("closed".parse::<State>().unwrap(), State::Closed);
        assert_eq!(
            "withdrawn_0".parse::<State>().unwrap(),
            State::Withdrawn { sequence: 0 }
        );
        assert_eq!(
            "withdrawn_1".parse::<State>().unwrap(),
            State::Withdrawn { sequence: 1 }
        );
        assert!("withdrawn".parse::<State>().is_err());
        assert!("withdrawn_".parse::<State>().is_err());
        assert!("withdrawn_1_".parse::<State>().is_err());
        assert!("withdrawn_1_2".parse::<State>().is_err());
        assert!("withdrawn_1_2_3".parse::<State>().is_err());
    }

    fn assert_position_similar(p1: Position, p2: Position) {
        assert_eq!(p1.reserves.r1, p2.reserves.r1);
        assert_eq!(p1.reserves.r2, p2.reserves.r2);
        assert_eq!(p1.phi.component.p, p2.phi.component.p);
        assert_eq!(p1.phi.component.q, p2.phi.component.q);
    }

    fn assert_position_not_similar(p1: Position, p2: Position) {
        let different_reserves = p1.reserves.r1 != p2.reserves.r1;
        let different_reserves = different_reserves || p1.reserves.r2 != p2.reserves.r2;
        let different_prices = p1.phi.component.p != p2.phi.component.p;
        let different_prices = different_prices || p1.phi.component.q != p2.phi.component.q;
        assert!(different_prices || different_reserves);
    }
    #[test]
    fn test_position() {
        let small_id = asset::Id(Fq::zero());
        let big_id = asset::Id(Fq::from(1u64));

        let pair_1 = DirectedTradingPair::new(small_id, big_id);
        let pair_2 = DirectedTradingPair::new(big_id, small_id);

        let price100i: (Amount, Amount) = (1u64.into(), 100u64.into());

        /*
           We create four positions per pair, where id(A) < id(B):
               + Case 1: for pair 1 (A -> B):
                   * position 1: provisions 150 units of asset 1 (A) at a price of 1/100.
                   * position 2: provisions 150 units of asset 2 (B) at a price of 100.
                   * position 3: provisions 150 units of asset 1 (A) at a price of 100.
                   * position 4: provisions 150 units of asset 2 (B) at a price of 1/100.
               + Case 2: for pair 2 (B -> A):
                   * position 1: provisions 150 units of asset 1 (B) at a price of 1/100.
                   * position 2: provisions 150 units fo asset 2 (A) at a price of 100.
                   * position 3: provisions 150 units of asset 1 (B) at a price of 100.
                   * position 4: provisions 150 units of asset 2 (A) at a price of 1/100.

           We want to check that:
               1. Case_1.p1 != Case_2.p2
               2. Case_1.p2 != Case_2.p1
               3. Case_1.p3 == Case_2.p2
               4. Case_1.p4 == Case_2.p1
               5. Case_2.p3 == Case_1.p2
               6. Case_2.p4 == Case_1.p1
        */

        let reserves_1 = Reserves {
            r1: 150u64.into(),
            r2: 0u64.into(),
        };

        let reserves_2 = reserves_1.flip();

        let a_position_1 = Position::new(
            OsRng,
            pair_1,
            0u32,
            price100i.0,
            price100i.1,
            reserves_1.clone(),
        );
        let a_position_2 = Position::new(
            OsRng,
            pair_1,
            0u32,
            price100i.0,
            price100i.1,
            reserves_2.clone(),
        );

        let a_position_3 = Position::new(
            OsRng,
            pair_1,
            0u32,
            price100i.1,
            price100i.0,
            reserves_1.clone(),
        );
        let a_position_4 = Position::new(
            OsRng,
            pair_1,
            0u32,
            price100i.1,
            price100i.0,
            reserves_2.clone(),
        );

        let b_position_1 = Position::new(
            OsRng,
            pair_2,
            0u32,
            price100i.0,
            price100i.1,
            reserves_1.clone(),
        );
        let b_position_2 = Position::new(
            OsRng,
            pair_2,
            0u32,
            price100i.0,
            price100i.1,
            reserves_2.clone(),
        );

        let b_position_3 = Position::new(
            OsRng,
            pair_2,
            0u32,
            price100i.1,
            price100i.0,
            reserves_1.clone(),
        );
        let b_position_4 = Position::new(
            OsRng,
            pair_2,
            0u32,
            price100i.1,
            price100i.0,
            reserves_2.clone(),
        );

        /*
                We want to check that:
                1. Case_1.p1 != Case_2.p2
                2. Case_1.p2 != Case_2.p1
                3. Case_1.p3 == Case_2.p2
                4. Case_1.p4 == Case_2.p1
                5. Case_2.p3 == Case_1.p2
                6. Case_2.p4 == Case_1.p1
        */
        assert_position_not_similar(a_position_1.clone(), b_position_2.clone());
        assert_position_not_similar(a_position_2.clone(), b_position_1.clone());
        assert_position_similar(a_position_3, b_position_2);
        assert_position_similar(a_position_4, b_position_1);
        assert_position_similar(b_position_3, a_position_2);
        assert_position_similar(b_position_4, a_position_1);
    }
}
