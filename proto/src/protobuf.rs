/// A marker trait that captures the relationships between a domain type (`Self`) and a protobuf type (`P`).
pub trait Protobuf<P>: Sized
where
    P: prost::Message + Default,
    P: std::convert::From<Self>,
    Self: std::convert::TryFrom<P> + Clone,
    <Self as std::convert::TryFrom<P>>::Error: Into<anyhow::Error>,
{
    /// Encode this domain type to a byte vector, via proto type `P`.
    fn encode_to_vec(&self) -> Vec<u8> {
        self.to_proto().encode_to_vec()
    }

    /// Convert this domain type to the associated proto type.
    ///
    /// This uses the `From` impl internally, so it works exactly
    /// like `.into()`, but does not require type inference.
    fn to_proto(&self) -> P {
        P::from(self.clone())
    }

    /// Decode this domain type from a byte buffer, via proto type `P`.
    fn decode<B: bytes::Buf>(buf: B) -> Result<Self, anyhow::Error> {
        <P as prost::Message>::decode(buf)?
            .try_into()
            .map_err(Into::into)
    }
}

// Implementations on foreign types.
//
// This should only be done here in cases where the domain type lives in a crate
// that shouldn't depend on the Penumbra proto framework.

use crate::core::crypto::v1alpha1::{BindingSignature, SpendAuthSignature, SpendVerificationKey};
use decaf377_rdsa::{Binding, Signature, SpendAuth, VerificationKey};
use ibc_rs::clients::ics07_tendermint;

impl Protobuf<SpendAuthSignature> for Signature<SpendAuth> {}
impl Protobuf<BindingSignature> for Signature<Binding> {}
impl Protobuf<SpendVerificationKey> for VerificationKey<SpendAuth> {}

impl From<Signature<SpendAuth>> for SpendAuthSignature {
    fn from(sig: Signature<SpendAuth>) -> Self {
        Self {
            inner: sig.to_bytes().to_vec(),
        }
    }
}

impl From<Signature<Binding>> for BindingSignature {
    fn from(sig: Signature<Binding>) -> Self {
        Self {
            inner: sig.to_bytes().to_vec(),
        }
    }
}

impl From<VerificationKey<SpendAuth>> for SpendVerificationKey {
    fn from(key: VerificationKey<SpendAuth>) -> Self {
        Self {
            inner: key.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<SpendAuthSignature> for Signature<SpendAuth> {
    type Error = anyhow::Error;
    fn try_from(value: SpendAuthSignature) -> Result<Self, Self::Error> {
        Ok(value.inner.as_slice().try_into()?)
    }
}

impl TryFrom<BindingSignature> for Signature<Binding> {
    type Error = anyhow::Error;
    fn try_from(value: BindingSignature) -> Result<Self, Self::Error> {
        Ok(value.inner.as_slice().try_into()?)
    }
}

impl TryFrom<SpendVerificationKey> for VerificationKey<SpendAuth> {
    type Error = anyhow::Error;
    fn try_from(value: SpendVerificationKey) -> Result<Self, Self::Error> {
        Ok(value.inner.as_slice().try_into()?)
    }
}

// Fuzzy Message Detection
use crate::core::crypto::v1alpha1::Clue as ProtoClue;
use decaf377_fmd::Clue;

impl Protobuf<ProtoClue> for Clue {}

impl From<Clue> for ProtoClue {
    fn from(msg: Clue) -> Self {
        ProtoClue {
            inner: bytes::Bytes::copy_from_slice(&msg.0).to_vec(),
        }
    }
}

impl TryFrom<ProtoClue> for Clue {
    type Error = anyhow::Error;

    fn try_from(proto: ProtoClue) -> anyhow::Result<Self, Self::Error> {
        let clue: [u8; 68] = proto.inner[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("clue malformed"))?;

        Ok(Clue(clue))
    }
}

// Consensus key
//
// The tendermint-rs PublicKey type already has a tendermint-proto type;
// this redefines its proto, because the encodings are consensus-critical
// and we don't vendor all of the tendermint protos.

impl Protobuf<crate::core::crypto::v1alpha1::ConsensusKey> for tendermint::PublicKey {}

impl From<tendermint::PublicKey> for crate::core::crypto::v1alpha1::ConsensusKey {
    fn from(v: tendermint::PublicKey) -> Self {
        Self {
            inner: v.to_bytes(),
        }
    }
}

impl TryFrom<crate::core::crypto::v1alpha1::ConsensusKey> for tendermint::PublicKey {
    type Error = anyhow::Error;
    fn try_from(value: crate::core::crypto::v1alpha1::ConsensusKey) -> Result<Self, Self::Error> {
        Self::from_raw_ed25519(value.inner.as_slice())
            .ok_or_else(|| anyhow::anyhow!("invalid ed25519 key"))
    }
}

impl Protobuf<crate::core::chain::v1alpha1::Ratio> for num_rational::Ratio<u64> {}

impl From<num_rational::Ratio<u64>> for crate::core::chain::v1alpha1::Ratio {
    fn from(v: num_rational::Ratio<u64>) -> Self {
        Self {
            numerator: *v.numer(),
            denominator: *v.denom(),
        }
    }
}

impl From<crate::core::chain::v1alpha1::Ratio> for num_rational::Ratio<u64> {
    fn from(value: crate::core::chain::v1alpha1::Ratio) -> Self {
        Self::new(value.numerator, value.denominator)
    }
}

// IBC-rs impls

extern crate ibc as ibc_rs;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::channel::v1::Channel as RawChannel;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;

use ibc_rs::core::ics03_connection::connection::ConnectionEnd;
use ibc_rs::core::ics04_channel::channel::ChannelEnd;
use ibc_rs::Height;

impl Protobuf<RawConnectionEnd> for ConnectionEnd {}
impl Protobuf<RawChannel> for ChannelEnd {}
impl Protobuf<RawHeight> for Height {}

// TODO(erwan): create ticket to switch to a trait object based approach
impl Protobuf<Any> for ics07_tendermint::client_state::ClientState {}
impl Protobuf<Any> for ics07_tendermint::consensus_state::ConsensusState {}
