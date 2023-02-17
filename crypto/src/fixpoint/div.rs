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
    v[0] = v[0] << lg_d;

    // Normalize u in place by shifting, carrying bits across words.
    // We may need an extra word to hold extra bits, since d was chosen from v, not u.
    for i in (1..7).rev() {
        u[i] = (u[i] << lg_d) | (u[i - 1] >> (64 - lg_d));
    }
    u[0] = u[0] << lg_d;

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
            q_hat = q_hat - 1;
            r_hat = r_hat + divisor;
            if r_hat >= 1 << 64 {
                break 'correction;
            }
        }

        // D4. [Multiply and subtract.] Multiply v by q_hat, subtracting the result from u.

        for i in 0..=n {
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
