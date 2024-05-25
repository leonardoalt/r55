use core::marker::PhantomData;
use core::default::Default;

use crate::*;

use alloy_core::primitives::Address;
use tiny_keccak::{Hasher, Keccak};

extern crate alloc;
use alloc::vec::Vec;

/// Implements a Solidity-like Mapping type.
#[derive(Default)]
pub struct Mapping<K, V> {
    id: u64,
    pd: PhantomData<(K, V)>
}

impl<K: ToBytes, V: Into<u64> + From<u64>> Mapping<K, V> {
    pub fn encode_key(&self, key: K) -> u64 {
        let key_bytes = key.to_bytes();
        let id_bytes = self.id.to_le_bytes();

        // Concatenate the key bytes and id bytes
        let mut concatenated = Vec::with_capacity(key_bytes.len() + id_bytes.len());
        concatenated.extend_from_slice(&key_bytes);
        concatenated.extend_from_slice(&id_bytes);

        let mut output = [0u8; 32];
        let mut hasher = Keccak::v256();
        hasher.update(&concatenated);
        hasher.finalize(&mut output);

        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&output[..8]);
        u64::from_le_bytes(bytes)
    }

    pub fn read(&self, key: K) -> V {
        sload(self.encode_key(key)).into()
    }

    pub fn write(&self, key: K, value: V) {
        sstore(self.encode_key(key), value.into());
    }
}

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for Address {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}
