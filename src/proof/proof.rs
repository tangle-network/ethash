use super::*;

use ethereum_types::{H128, H256, H64};
use tiny_keccak::{Keccak, Hasher};
pub const CACHE_LEVEL: u64 = 15;
pub const HASH_LENGTH: usize = 16;
pub const WORD_LENGTH: usize = 128;
pub const BRANCH_ELEMENT_LENGTH: usize = 32;

pub fn keccak_512(data: &[u8]) -> [u8; 64] {
	let mut keccak = Keccak::v512();
	keccak.update(data);
	let mut output = [0u8; 64];
	keccak.finalize(&mut output);
	output
}

#[derive(Debug)]
struct Hex(pub Vec<u8>);

struct BlockWithProofs {
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
		let epoch = (header.number / 30000) as usize;
		let full_size = crate::get_full_size(epoch);
		let indices = Self::get_indices(
			header.partial_hash.unwrap(),
			header.nonce,
			full_size,
			|i| calc_dataset_item(&cache, i),
		);

		for index in indices {
			if let Some((element, proof)) = Self::calculate_proof(index, header.clone(), cache.clone()) {
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

	fn get_indices<F: Fn(usize) -> H512>(
		header_hash: H256, nonce: H64, full_size: usize, lookup: F
	) -> Vec<u32> {
		let mut result: Vec<u32> = vec![];
		let n = full_size / HASH_BYTES;
		let w = MIX_BYTES / WORD_BYTES;
		const MIXHASHES: usize = MIX_BYTES / HASH_BYTES;
		let s = {
			let mut data = [0u8; 40];
			data[..32].copy_from_slice(&header_hash.0);
			data[32..].copy_from_slice(&nonce.0);
			data[32..].reverse();
			keccak_512(&data)
		};
		let mut mix = [0u8; MIX_BYTES];
		for i in 0..MIXHASHES {
			for j in 0..64 {
				mix[i * HASH_BYTES + j] = s[j];
			}
		}

		for i in 0..ACCESSES {
			let p = (fnv((i as u32).bitxor(LittleEndian::read_u32(s.as_ref())),
						 LittleEndian::read_u32(&mix[(i % w * 4)..]))
					 as usize) % (n / MIXHASHES) * MIXHASHES;
			result.push(p as u32);
			let mut newdata = [0u8; MIX_BYTES];
			for j in 0..MIXHASHES {
				let v = lookup(p + j);
				for k in 0..64 {
					newdata[j * 64 + k] = v[k];
				}
			}
			mix = fnv128(mix, newdata);
		}
		let mut cmix = [0u8; MIX_BYTES / 4];
		for i in 0..(MIX_BYTES / 4 / 4) {
			let j = i * 4;
			let a = fnv(LittleEndian::read_u32(&mix[(j * 4)..]),
						LittleEndian::read_u32(&mix[((j + 1) * 4)..]));
			let b = fnv(a, LittleEndian::read_u32(&mix[((j + 2) * 4)..]));
			let c = fnv(b, LittleEndian::read_u32(&mix[((j + 3) * 4)..]));

			LittleEndian::write_u32(&mut cmix[j..], c);
		}

		result
	}

	fn calculate_proof(_index: u32, _header: types::BlockHeader, _cache: Vec<u8>) -> Option<([u8; WORD_LENGTH], [u8; HASH_LENGTH])> {
		None
	}

	fn hashes_to_branches_array(_proof: [u8; HASH_LENGTH]) -> Vec<[u8; BRANCH_ELEMENT_LENGTH]> {
		vec![]
	}

	fn to_h256_array(elt: [u8; 128]) -> Vec<H256> {
		let mut result = vec![];
		for i in 0..(WORD_LENGTH / 32) {
			let elt_temp = &elt[(i * 32)..((i+1) * 32)];
			let mut temp: [u8; 32] = [0; 32];
			for j in 0..temp.len() {
				temp[j] = elt_temp[j];
			}
			// ensure the endianness is preserved
			let val = H256::from(temp);
			result.push(val);
		}
		return result
	}
}
