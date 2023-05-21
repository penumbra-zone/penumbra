use ethnum::U256;
use ibig::UBig;

use super::Error;

/// Computes (2^128 * x) / y and its remainder.
/// TEMP HACK: need to implement this properly
pub(super) fn stub_div_rem_u384_by_u256(x: U256, y: U256) -> Result<(U256, U256), Error> {
    if y == U256::ZERO {
        return Err(Error::DivisionByZero);
    }

    let x_big = ibig::UBig::from_le_bytes(&x.to_le_bytes());
    let y_big = ibig::UBig::from_le_bytes(&y.to_le_bytes());
    // this is what we actually want to compute: 384-bit / 256-bit division.
    let x_big_128 = x_big << 128;
    let q_big = &x_big_128 / &y_big;
    let rem_big = x_big_128 - (&y_big * &q_big);

    let Some(q) = ubig_to_u256(&q_big) else {
        return Err(Error::Overflow);
    };
    let rem = ubig_to_u256(&rem_big).expect("rem < q, so we already returned on overflow");

    Ok((q, rem))
}

#[allow(dead_code)]
fn u256_to_ubig(x: U256) -> UBig {
    let mut bytes = [0; 32];
    bytes.copy_from_slice(&x.to_le_bytes());
    UBig::from_le_bytes(&bytes)
}

fn ubig_to_u256(x: &UBig) -> Option<U256> {
    let bytes = x.to_le_bytes();
    if bytes.len() <= 32 {
        let mut u256_bytes = [0; 32];
        u256_bytes[..bytes.len()].copy_from_slice(&bytes);
        Some(U256::from_le_bytes(u256_bytes))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn u256_strategy() -> BoxedStrategy<U256> {
        any::<[u8; 32]>().prop_map(U256::from_le_bytes).boxed()
    }

    proptest! {
        #[test]
        fn stub_div_rem_works(
            x in u256_strategy(),
            y in u256_strategy()
        ) {
            let Ok((q, rem)) = stub_div_rem_u384_by_u256(x, y) else {
                return Ok(());
            };

            let q_big = u256_to_ubig(q);
            let rem_big = u256_to_ubig(rem);
            let x_big = u256_to_ubig(x);
            let y_big = u256_to_ubig(y);

            let rhs = x_big << 128;
            let lhs = &q_big * &y_big + &rem_big;
            assert_eq!(rhs, lhs);
        }
    }
}

#[allow(dead_code)]
fn div_rem_u384_by_u256(u: [u64; 6], mut v: [u64; 4]) -> ([u64; 6], [u64; 4]) {
    // Uses Algorithm D from Knuth, vol 2, 4.3.1, p 272.

    // Make a new buffer for u that will have an extra word.
    let mut u = [u[0], u[1], u[2], u[3], u[4], u[5], 0];

    // Find the most significant non-zero word of v.
    let n = v
        .iter()
        .rposition(|&x| x != 0)
        .expect("v has at least one nonzero word")
        + 1;
    assert!(
        n >= 2,
        "single-word division should use a different algorithm"
    );
    // 6 = m + n => m = 6 - n
    let m = 6 - n;

    // D1. [Normalize.] Multiply by d, a power of 2, so that the most significant bit of v[n-1] is set.
    let lg_d = v[n - 1].leading_zeros();

    // Normalize v in place by shifting, carrying bits across words.
    // Working from the top down lets us avoid an explicit carry.
    for i in (1..n).rev() {
        v[i] = (v[i] << lg_d) | (v[i - 1] >> (64 - lg_d));
    }
    v[0] <<= lg_d;

    // Normalize u in place by shifting, carrying bits across words.
    // We may need an extra word to hold extra bits, since d was chosen from v, not u.
    for i in (1..7).rev() {
        u[i] = (u[i] << lg_d) | (u[i - 1] >> (64 - lg_d));
    }
    u[0] <<= lg_d;

    // D2. [Initialize j.] Set j to m.
    let mut j = m;

    // This is really while j >= 0, but that's awkward without signed indexes.
    loop {
        // D3. [Calculate q_hat.]

        // Set q_hat = (u[j+n]*2^64 + u[j+n-1]) / v[n-1].
        let dividend = u128::from(u[j + n]) << 64 | u128::from(u[j + n - 1]);
        let divisor = u128::from(v[n - 1]);
        let mut q_hat = dividend / divisor;
        let mut r_hat = dividend % divisor;

        // Check whether we need to correct the estimated q_hat.
        'correction: while q_hat >= 1 << 64
            || q_hat * u128::from(v[n - 2]) > ((r_hat << 64) | u128::from(u[j + n - 2]))
        {
            q_hat -= 1;
            r_hat += divisor;
            if r_hat >= 1 << 64 {
                break 'correction;
            }
        }

        // D4. [Multiply and subtract.] Multiply v by q_hat, subtracting the result from u.

        for _i in 0..=n {
            todo!()
        }

        if j == 0 {
            break;
        } else {
            j -= 1;
        }
    }

    todo!()
}
