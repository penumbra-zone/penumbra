//! See the [state machine][state-machine] diagram for a visual depiction of an IBC handshake.
//!
//! The handhshake proceeds like so:
//! - ConnectionOpenInit
//! - ConnectionOpenTry
//! - ConnectionOpenAck
//! - ConnectionOpenConfirm
//!
//! [state-machine]: https://media.githubusercontent.com/media/cosmos/ibc/main/spec/core/ics-003-connection-semantics/state.png

#![allow(unused)] // XXX(kate)

mod common;

use {
    anyhow::Error,
    std::time::Duration,
    penumbra_cometstub::Node,
    tendermint::block::{signed_header::SignedHeader, Block, Header},
    tendermint_light_client_verifier::{
        options::Options,
        types::{TrustedBlockState, UntrustedBlockState, TrustThreshold},
        Verdict, Verifier, ProdVerifier,
    },
};

fn builder() {
    let _ = penumbra_cometstub::engine::Engine::builder();
}

#[ignore] // XXX(kate): this test doesn't pass yet.
#[tokio::test]
async fn ibc_handshake_can_be_generated() -> Result<(), Error> {
    let engine = Node::new();
    let block = engine.next_block();

    validate(&engine).await?;
    todo!("XXX(kate): fallthrough! nice work. 💕");
    Ok(())
}

async fn validate(engine: &Node) -> Result<(), Error> {
    let validators = engine.validators();
    let Block {
        header:
            ref header @ Header {
                time: header_time,
                height,
                ..
            },
        last_commit: Some(ref commit),
        ..
    } = engine.block
    else {
        anyhow::bail!("TODO(kate): next, generate blocks with valid commit info");
    };

    let signed_header = SignedHeader::new(header.clone(), commit.clone())?;
    let untrusted_state = UntrustedBlockState {
        signed_header: &signed_header,
        validators: &validators, // &untrusted_header.validator_set,
        next_validators: None,
    };

    let trusted_state = TrustedBlockState {
        chain_id: engine.chain_id(),
        header_time,
        height,
        next_validators: &validators,
        next_validators_hash: validators.hash(),
    };
    let options = Options {
        // XXX(kate): figure out a better way to store some defaults, or generate options from
        // the pd config? how should we configure the light client.
        trust_threshold: TrustThreshold::ONE_THIRD,
        trusting_period: Duration::from_secs(1),
        clock_drift: Duration::from_millis(100),
    };

    let verdict = ProdVerifier::default().verify_update_header(
        untrusted_state,
        trusted_state,
        &options,
        engine.block.header.time,
    );

    match verdict {
        Verdict::Success => Ok(()),
        Verdict::NotEnoughTrust(voting_power_tally) => Err(anyhow::anyhow!(
            "not enough trust, voting power tally: {:?}",
            voting_power_tally
        )),
        Verdict::Invalid(detail) => Err(anyhow::anyhow!(
            "could not verify tendermint header: invalid: {:?}",
            detail
        )),
    }
}
