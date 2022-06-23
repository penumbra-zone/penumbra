//! Lookup tables for decryption.
//!
//! Currently, this module just has a naive, in-memory implementation.  In the
//! future, it should have a trait that allows users to plug their own storage
//! system to back the lookup table.

use std::{collections::BTreeMap, future::Future, pin::Pin};

use futures::FutureExt;
use parking_lot::Mutex;

pub trait DecryptionTable {
    fn lookup(&self, key: [u8; 32]) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<u32>>>>>;
    // idea: a rocksdb backing store could store the data in the form
    // (nbits || key, value)
    // where nbits is the bitsize of value
    // then do a linear scan through bitsizes to cut the working set size
    // might not be better than naive caching?
    fn store(&self, key: [u8; 32], value: u32)
        -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>>;
}

/// A naive, in-memory decryption table
pub struct MockDecryptionTable {
    inner: Mutex<BTreeMap<[u8; 32], u32>>,
}

impl MockDecryptionTable {
    #[allow(non_snake_case)]
    pub fn build(bitsize: usize) -> Self {
        let mut inner = BTreeMap::default();
        let (mut x, mut xB) = (0u32, decaf377::Element::default());
        let B = decaf377::basepoint();

        let bound = 1 << bitsize;
        while x < bound {
            inner.insert(xB.compress().0, x);
            x += 1;
            xB += B;
        }

        Self {
            inner: Mutex::new(inner),
        }
    }
}

impl DecryptionTable for MockDecryptionTable {
    fn lookup(&self, key: [u8; 32]) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<u32>>>>> {
        futures::future::ready(Ok(self.inner.lock().get(&key).cloned())).boxed()
    }

    fn store(
        &self,
        key: [u8; 32],
        value: u32,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>> {
        self.inner.lock().insert(key, value);
        futures::future::ready(Ok(())).boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_bitsize(bitsize: usize) {
        use std::time::Instant;
        let now = Instant::now();

        let _table = MockDecryptionTable::build(bitsize);

        let elapsed = now.elapsed();

        println!(
            "Table of bitsize {} took {}s",
            bitsize,
            elapsed.as_secs_f64()
        );
    }

    #[test]
    fn build_16() {
        build_bitsize(16);
    }

    #[test]
    fn build_21() {
        build_bitsize(21);
    }
}
