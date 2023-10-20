pub fn transform<A, B, const N: usize>(data: [A; N], f: impl Fn(A) -> B) -> [B; N] {
    match data.into_iter().map(f).collect::<Vec<B>>().try_into() {
        Ok(x) => x,
        _ => panic!("The size of the iterator should not have changed"),
    }
}

#[cfg(not(feature = "parallel"))]
pub fn transform_parallel<A, B, const N: usize>(data: [A; N], f: impl Fn(A) -> B) -> [B; N] {
    transform(data, f)
}

#[cfg(feature = "parallel")]
pub fn transform_parallel<A: Send, B: Sync + Send, const N: usize>(
    data: [A; N],
    f: impl Fn(A) -> B + Sync + Send,
) -> [B; N] {
    use rayon::prelude::*;
    let out: Vec<B> = data.into_par_iter().map(f).collect();
    // Note: we do it this way because we don't have a Debug implementation for B
    match out.try_into() {
        Ok(x) => x,
        _ => panic!("The size of the iterator should not have changed"),
    }
}

pub fn flatten_results<A, E, const N: usize>(data: [Result<A, E>; N]) -> Result<[A; N], E> {
    let mut buf = Vec::with_capacity(N);
    for x in data {
        buf.push(x?);
    }
    match buf.try_into() {
        Ok(x) => Ok(x),
        _ => panic!("The size of the iterator should not have changed"),
    }
}
