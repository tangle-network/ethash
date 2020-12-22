use super::*;
use ethereum_types::{Address, Bloom, H256, H64, U256};
use rlp::RlpStream;
use rlp_derive::{RlpDecodable, RlpEncodable};

#[derive(Debug, Clone, RlpEncodable, RlpDecodable)]
pub struct BlockHeader {
    pub parent_hash: H256,
    pub uncles_hash: H256,
    pub author: Address,
    pub state_root: H256,
    pub transactions_root: H256,
    pub receipts_root: H256,
    pub log_bloom: Bloom,
    pub difficulty: U256,
    pub number: U256,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub extra_data: String,
    pub mix_hash: H256,
    pub nonce: H64,
}

#[derive(Debug, Clone, RlpEncodable, RlpDecodable)]
pub struct BlockHeaderSeal {
    pub parent_hash: H256,
    pub uncles_hash: H256,
    pub author: Address,
    pub state_root: H256,
    pub transactions_root: H256,
    pub receipts_root: H256,
    pub log_bloom: Bloom,
    pub difficulty: U256,
    pub number: U256,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub extra_data: String,
}

impl From<BlockHeader> for BlockHeaderSeal {
    fn from(b: BlockHeader) -> Self {
        Self {
            parent_hash: b.parent_hash,
            uncles_hash: b.uncles_hash,
            author: b.author,
            state_root: b.state_root,
            transactions_root: b.transactions_root,
            receipts_root: b.receipts_root,
            log_bloom: b.log_bloom,
            difficulty: b.difficulty,
            number: b.number,
            gas_limit: b.gas_limit,
            gas_used: b.gas_used,
            timestamp: b.timestamp,
            extra_data: b.extra_data,
        }
    }
}

impl BlockHeader {
    pub fn extra_data(&self) -> H256 {
        let mut data = [0u8; 32];
        data.copy_from_slice(self.extra_data.as_bytes());
        H256(data)
    }

    fn stream_rlp(&self, stream: &mut RlpStream, partial: bool) {
        stream.begin_list(13 + if !partial { 2 } else { 0 });

        stream.append(&self.parent_hash);
        stream.append(&self.uncles_hash);
        stream.append(&self.author);
        stream.append(&self.state_root);
        stream.append(&self.transactions_root);
        stream.append(&self.receipts_root);
        stream.append(&self.log_bloom);
        stream.append(&self.difficulty);
        stream.append(&self.number);
        stream.append(&self.gas_limit);
        stream.append(&self.gas_used);
        stream.append(&self.timestamp);
        stream.append(&self.extra_data);

        if !partial {
            stream.append(&self.mix_hash);
            stream.append(&self.nonce);
        }
    }
}
