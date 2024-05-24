use core::marker::PhantomData;
use core::default::Default;

use crate::*;

/// Implements a Solidity-like Mapping type.
#[derive(Default)]
pub struct Mapping<K, V> {
    id: u64,
    pd: PhantomData<(K, V)>
}

impl<K: Into<u64>, V: Into<u64> + From<u64>> Mapping<K, V> {
    pub fn encode_key(&self, key: K) -> u64 {
        // TODO This is of course unsafe, should use Keccak eventually.
        key.into() << 32 | self.id
    }

    pub fn read(&self, key: K) -> V {
        sload(self.encode_key(key)).into()
    }

    pub fn write(&self, key: K, value: V) {
        sstore(self.encode_key(key), value.into());
    }
}
