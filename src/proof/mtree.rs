use core::convert::TryInto;
use core::ops::Deref;

use ethereum_types::H256;
use sha2::Digest;

const HASH_LENGTH: usize = 16; // bytes.
const WORD_LENGTH: usize = 128; // bytes.
const BRANCH_ELEMENT_LENGTH: usize = 32; // bytes.

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Hash(pub [u8; HASH_LENGTH]);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct Word(pub [u8; WORD_LENGTH]);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct BranchElement(pub [u8; BRANCH_ELEMENT_LENGTH]);

impl Word {
    pub fn into_h256_array(mut self) -> [H256; 4] {
        self.0
            .chunks_exact_mut(32)
            .map(|s| {
                s.reverse();
                H256::from_slice(s)
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("hash to H256 should never fails")
    }

    /// #### Conventional encoding
    ///
    /// To make it easier for ethereum smartcontract to follow the hash
    /// calculation, we use a convention to encode DAG dataset element to use in
    /// hash function. The encoding is defined as the following pseudo code:
    ///
    /// - 1 assume the element is `abcd` where a, b, c, d are 32 bytes word
    /// - 2 `first = concat(reverse(a), reverse(b))` where `reverse` reverses
    ///   the bytes.
    /// - 3 `second = concat(reverse(c), reverse(d))`
    /// 4. conventional encoding of `abcd` is `concat(first, second)`
    pub fn conventional(&self) -> ([u8; 64], [u8; 64]) {
        let mut first = [0u8; 64];
        let mut second = [0u8; 64];
        self.0
            .clone()
            .chunks_exact_mut(32)
            .map(|c| {
                c.reverse();
                c
            })
            .enumerate()
            .for_each(|(i, chunk)| match i {
                0 => first[0..32].copy_from_slice(chunk),
                1 => first[32..64].copy_from_slice(chunk),
                2 => second[0..32].copy_from_slice(chunk),
                3 => second[32..64].copy_from_slice(chunk),
                _ => unreachable!("only 4 chunks"),
            });
        (first, second)
    }
}

impl From<[u8; HASH_LENGTH]> for Hash {
    fn from(b: [u8; HASH_LENGTH]) -> Self { Self(b) }
}

impl From<[u8; WORD_LENGTH]> for Word {
    fn from(b: [u8; WORD_LENGTH]) -> Self { Self(b) }
}

impl From<[u8; BRANCH_ELEMENT_LENGTH]> for BranchElement {
    fn from(b: [u8; BRANCH_ELEMENT_LENGTH]) -> Self { Self(b) }
}

impl From<[Hash; 2]> for BranchElement {
    fn from([h1, h2]: [Hash; 2]) -> Self {
        let mut b: [u8; BRANCH_ELEMENT_LENGTH] = [0; BRANCH_ELEMENT_LENGTH];
        b[0..16].copy_from_slice(h1.deref());
        b[16..32].copy_from_slice(h2.deref());
        Self(b)
    }
}

impl Into<[H256; 4]> for Word {
    fn into(self) -> [H256; 4] { self.into_h256_array() }
}

impl Deref for Hash {
    type Target = [u8; HASH_LENGTH];

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Deref for Word {
    type Target = [u8; WORD_LENGTH];

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl Deref for BranchElement {
    type Target = [u8; BRANCH_ELEMENT_LENGTH];

    fn deref(&self) -> &Self::Target { &self.0 }
}

pub(super) fn hash(a: &Hash, b: &Hash) -> Hash {
    let hasher = sha2::Sha256::default();
    let hash = hasher
        .chain([0u8; 16])
        .chain(a.0)
        .chain([0u8; 16])
        .chain(b.0)
        .result();
    let mut data = [0u8; HASH_LENGTH];
    data.copy_from_slice(&hash[HASH_LENGTH..]);
    Hash(data)
}

pub(super) fn hash_element(word: &Word) -> Hash {
    let (first, second) = word.conventional();
    let hasher = sha2::Sha256::default();
    let hash = hasher.chain(first).chain(second).result();
    let mut data = [0u8; HASH_LENGTH];
    data.copy_from_slice(&hash[HASH_LENGTH..]);
    Hash(data)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_to_h256_array() {
        let word: [u8; WORD_LENGTH] = [
            1, 2, 3, 4, 5, 6, 7, 8, // 0
            1, 2, 3, 4, 5, 6, 7, 8, // 1
            1, 2, 3, 4, 5, 6, 7, 8, // 2
            1, 2, 3, 4, 5, 6, 7, 8, // 3
            1, 2, 3, 4, 5, 6, 7, 8, // 4
            1, 2, 3, 4, 5, 6, 7, 8, // 5
            1, 2, 3, 4, 5, 6, 7, 8, // 6
            1, 2, 3, 4, 5, 6, 7, 8, // 7
            1, 2, 3, 4, 5, 6, 7, 8, // 8
            1, 2, 3, 4, 5, 6, 7, 8, // 9
            1, 2, 3, 4, 5, 6, 7, 8, // a
            1, 2, 3, 4, 5, 6, 7, 8, // b
            1, 2, 3, 4, 5, 6, 7, 8, // c
            1, 2, 3, 4, 5, 6, 7, 8, // d
            1, 2, 3, 4, 5, 6, 7, 8, // e
            1, 2, 3, 4, 5, 6, 7, 8, // f
        ];
        let hashes = Word::from(word).into_h256_array();
        println!("{:#?}", hashes);
    }
}
