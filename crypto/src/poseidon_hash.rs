// XXX move into poseidon377 crate?

use crate::Fq;

use poseidon377::ark_sponge::{
    poseidon::PoseidonSponge, CryptographicSponge, FieldBasedCryptographicSponge,
};

pub fn hash_1(domain_separator: &Fq, value: Fq) -> Fq {
    // we want to set the capacity to domain_separator and the rate to value,
    // then run the sponge and extract the rate. it's a bit hard to do this
    // using the ark-sponge api, which is trying to do a higher-level duplex
    // construction and doesn't allow access to the underlying sponge

    let mut sponge = PoseidonSponge::new(&poseidon377::params::rate_1());

    // arkworks sponge api doesn't let us call permute
    //
    // best we can do now is to look in the source to see how the rate and
    // capacity are arranged and try to plumb the functionality we want through
    // the higher-level API
    //
    // arkworks uses (rate || capacity) instead of (capacity || rate)
    //
    // this also gives incompatible outputs, but let's deal with that later

    // set the capacity
    assert_eq!(sponge.state.len(), 2);
    sponge.state[1] = *domain_separator;

    // now use absorb to set the rate (hopefully)
    sponge.absorb(&value);
    // and squeeze an element
    let out_vec = sponge.squeeze_native_field_elements(1);

    out_vec.into_iter().next().unwrap()
}

pub fn hash_4(domain_separator: &Fq, value: (Fq, Fq, Fq, Fq)) -> Fq {
    let mut sponge = PoseidonSponge::new(&poseidon377::params::rate_4());
    assert_eq!(sponge.state.len(), 5);
    sponge.state[1] = *domain_separator;

    // now use absorb to set the rate (hopefully)
    sponge.absorb(&value.0);
    sponge.absorb(&value.1);
    sponge.absorb(&value.2);
    sponge.absorb(&value.3);

    // and squeeze an element
    let out_vec = sponge.squeeze_native_field_elements(1);

    out_vec.into_iter().next().unwrap()
}

pub fn hash_5(domain_separator: &Fq, value: (Fq, Fq, Fq, Fq, Fq)) -> Fq {
    let mut sponge = PoseidonSponge::new(&poseidon377::params::rate_5());
    assert_eq!(sponge.state.len(), 6);
    sponge.state[1] = *domain_separator;

    // now use absorb to set the rate (hopefully)
    sponge.absorb(&value.0);
    sponge.absorb(&value.1);
    sponge.absorb(&value.2);
    sponge.absorb(&value.3);
    sponge.absorb(&value.4);

    // and squeeze an element
    let out_vec = sponge.squeeze_native_field_elements(1);

    out_vec.into_iter().next().unwrap()
}
