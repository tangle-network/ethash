use super::*;
use rlp::{RlpStream};
use ethereum_types::{Bloom, H160, H256, H64, U256};

#[derive(Debug, Clone)]
pub struct BlockHeader {
	pub parent_hash: H256,
	pub uncles_hash: H256,
	pub author: H160,
	pub state_root: H256,
	pub transactions_root: H256,
	pub receipts_root: H256,
	pub log_bloom: Bloom,
	pub difficulty: U256,
	pub number: u64,
	pub gas_limit: U256,
	pub gas_used: U256,
	pub timestamp: u64,
	pub extra_data: Vec<u8>,
	pub mix_hash: H256,
	pub nonce: H64,

	pub hash: Option<H256>,
	pub partial_hash: Option<H256>,
}

impl BlockHeader {
	pub fn extra_data(&self) -> H256 {
		let mut data = [0u8; 32];
		data.copy_from_slice(self.extra_data.as_slice());
		H256(data.into())
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
