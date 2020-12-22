use std::ops::BitXor;

use byteorder::ByteOrder;
use byteorder::LittleEndian;
use ethereum_types::H512;
use ethereum_types::{H128, H256, H64};
use tiny_keccak::{Hasher, Keccak};

use crate::ACCESSES;
use crate::HASH_BYTES;
use crate::MIX_BYTES;
use crate::WORD_BYTES;

pub const CACHE_LEVEL: u64 = 15;
pub const HASH_LENGTH: usize = 16;
pub const WORD_LENGTH: usize = 128;
pub const BRANCH_ELEMENT_LENGTH: usize = 32;

// not needed for now
mod linked_list;
pub mod mtree;
pub mod types;

pub fn keccak_512(data: &[u8]) -> [u8; 64] {
    let mut keccak = Keccak::v512();
    keccak.update(data);
    let mut output = [0u8; 64];
    keccak.finalize(&mut output);
    output
}

#[derive(Debug)]
pub struct Hex(pub Vec<u8>);

pub struct BlockWithProofs {
    pub proof_length: u64,
    pub header_rlp: Hex,
    pub merkle_root: H128,
    pub elements: Vec<H256>,
    pub merkle_proofs: Vec<H128>,
}

impl BlockWithProofs {
    pub fn append_to_elements(&mut self, elts: Vec<H256>) {
        for i in 0..elts.len() {
            self.elements.push(elts[i]);
        }
    }

    pub fn append_merkle_proofs(&mut self, _elts: Vec<H256>) {}

    pub fn create_proof(&mut self, cache: Vec<u8>, header: types::BlockHeader) {
        let epoch = (header.number.as_u128() / 30000) as usize;
        let full_size = crate::get_full_size(epoch);
        let indices = vec![];

        for index in indices {
            if let Some((element, proof)) =
                Self::calculate_proof(index, header.clone(), cache.clone())
            {
                let es = Self::to_h256_array(element);
                self.append_to_elements(es);

                let mut all_proofs: Vec<H256> = vec![];
                let br_arr = Self::hashes_to_branches_array(proof);
                for i in 0..br_arr.len() {
                    all_proofs.push(H256::from(br_arr[i]));
                }

                self.append_merkle_proofs(all_proofs);
            }
        }
    }

    fn calculate_proof(
        _index: u32,
        _header: types::BlockHeader,
        _cache: Vec<u8>,
    ) -> Option<([u8; WORD_LENGTH], [u8; HASH_LENGTH])> {
        None
    }

    fn hashes_to_branches_array(_proof: [u8; HASH_LENGTH]) -> Vec<[u8; BRANCH_ELEMENT_LENGTH]> {
        vec![]
    }

    fn to_h256_array(elt: [u8; 128]) -> Vec<H256> {
        let mut result = vec![];
        for i in 0..(WORD_LENGTH / 32) {
            let elt_temp = &elt[(i * 32)..((i + 1) * 32)];
            let mut temp: [u8; 32] = [0; 32];
            for j in 0..temp.len() {
                temp[j] = elt_temp[j];
            }
            // ensure the endianness is preserved
            let val = H256::from(temp);
            result.push(val);
        }
        return result;
    }
}

pub fn get_indices<F>(header_hash: H256, nonce: H64, full_size: usize, lookup: F) -> Vec<u32>
where
    F: Fn(usize) -> [u32; HASH_LENGTH],
{
    let mut result = vec![];
    let rows = (full_size / MIX_BYTES) as u32;
    let mut seed = [0u8; 40]; // 32 + 8
    seed[0..32].copy_from_slice(header_hash.as_bytes());
    seed[32..].copy_from_slice(nonce.as_bytes());
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
