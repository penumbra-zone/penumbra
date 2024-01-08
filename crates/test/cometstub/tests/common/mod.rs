//! Common facilities for the `penumbra-cometstub` test suite.

use tendermint::Time;
use tendermint_light_client_verifier::{
    options::Options,
    types::{TrustedBlockState, UntrustedBlockState},
    Verdict, Verifier,
};

// XXX(kate): is this actually useful? do we need this?
pub struct TestVerifier;

impl Verifier for TestVerifier {
    fn verify_update_header(
        &self,
        _untrusted: UntrustedBlockState<'_>,
        _trusted: TrustedBlockState<'_>,
        _options: &Options,
        _now: Time,
    ) -> Verdict {
        todo!("<TestVerifier as Verifier>::verify_update_header")
    }
    fn verify_misbehaviour_header(
        &self,
        _untrusted: UntrustedBlockState<'_>,
        _trusted: TrustedBlockState<'_>,
        _options: &Options,
        _now: Time,
    ) -> Verdict {
        todo!("<TestVerifier as Verifier>::verify_misbehaviour_header")
    }
}
