#[cfg(feature = "std")]
use std::io;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use byteorder::ByteOrder;
use byteorder::LittleEndian;
use ethereum_types::{H128, H256, H64};
use tiny_keccak::{Hasher, Keccak};

use crate::ACCESSES;
use crate::MIX_BYTES;

pub const CACHE_LEVEL: u64 = 15;
pub const HASH_LENGTH: usize = 16;
pub const WORD_LENGTH: usize = 128;
pub const BRANCH_ELEMENT_LENGTH: usize = 32;

mod merkle_proof;
pub mod mtree;
pub mod types;

pub fn keccak_512(data: &[u8]) -> [u8; 64] {
    let mut keccak = Keccak::v512();
    keccak.update(data);
    let mut output = [0u8; 64];
    keccak.finalize(&mut output);
    output
}

pub fn get_indices<F>(
    header_hash: H256,
    nonce: H64,
    full_size: usize,
    lookup: F,
) -> Vec<u32>
where
    F: Fn(usize) -> [u32; HASH_LENGTH],
{
    let mut result = Vec::new();
    let rows = (full_size / MIX_BYTES) as u32;
    let mut seed = [0u8; 40]; // 32 + 8
    seed[0..32].copy_from_slice(header_hash.as_bytes()); // 32
    seed[32..].copy_from_slice(nonce.as_bytes()); // 8
    seed[32..].reverse();
    let seed = keccak_512(&seed);
    let seed_head = LittleEndian::read_u32(&seed);

    const MIX_LEN: usize = MIX_BYTES / 4;
    let mut mix = [0u32; MIX_LEN];
    for (i, b) in mix.iter_mut().enumerate() {
        *b = LittleEndian::read_u32(&seed[(i % 16 * 4)..]);
    }
    let mut temp = [0u32; MIX_LEN];
    for i in 0..ACCESSES {
        let a = i as u32 ^ seed_head;
        let m = mix[i % MIX_LEN];
        let parent = crate::fnv(a, m) % rows;
        result.push(parent);
        for k in 0..MIX_BYTES / ACCESSES {
            let cache_index = 2 * parent + k as u32;
            let data = lookup(cache_index as _);
            let from = k * HASH_LENGTH;
            let to = from + HASH_LENGTH;
            temp[from..to].copy_from_slice(&data);
        }
        crate::fnv_mix_hash(&mut mix, temp);
    }
    result
}

/// A conventional way for calculating the Root hash of the merkle tree.
#[cfg(feature = "std")]
pub fn calc_dataset_merkle_root(epoch: usize, dataset: impl io::Read) -> H128 {
    let map = calc_dataset_merkle_proofs(epoch, dataset);
    let root = map.hash();
    H128::from_slice(&root.0)
}

#[cfg(feature = "std")]
pub fn calc_dataset_depth(epoch: usize) -> usize {
    let full_size = crate::get_full_size(epoch);
    let full_size_128_resolution = full_size / 128;
    format!("{:b}", full_size_128_resolution - 1).len()
}

/// Calculate the merkle tree and return a HashCache that can be used to
/// calculating proofs and can be used to cache them to filesystem.
#[cfg(feature = "std")]
pub fn calc_dataset_merkle_proofs(
    epoch: usize,
    mut dataset: impl io::Read,
) -> mtree::MerkleTree {
    let full_size = crate::get_full_size(epoch);
    let full_size_128_resolution = full_size / 128;
    let branch_depth = calc_dataset_depth(epoch);
    let mut buf = [0u8; 128];
    let mut i = 0;
    let mut leaves = Vec::with_capacity(full_size_128_resolution);
    while i < full_size_128_resolution {
        if let Ok(n) = dataset.read(&mut buf) {
            if n == 0 {
                break;
            }
            if n != 128 {
                panic!("Malformed dataset");
            }
        }
        let leaf = mtree::hash_element(&mtree::Word(buf));
        leaves.push(leaf);
        i += 1;
    }

    mtree::MerkleTree::create(&leaves, branch_depth)
}
