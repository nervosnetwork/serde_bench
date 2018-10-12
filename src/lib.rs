extern crate bigint;
extern crate flatbuffers;
extern crate protobuf;
extern crate rand;

pub mod bench_flatbuffers;
pub mod bench_protobuf;

use bench_flatbuffers::{Header as FlatbuffersHeader, HeaderBuilder};
use bench_protobuf::Header as ProtobufHeader;
use bigint::{H256, U256};
use flatbuffers::{get_root, FlatBufferBuilder};
use protobuf::{parse_from_bytes, Message};
use rand::{thread_rng, Rng};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Header {
    pub version: u32,
    pub parent_hash: H256,
    pub timestamp: u64,
    pub number: u64,
    pub txs_commit: H256,
    pub txs_proposal: H256,
    pub difficulty: U256,
    pub cellbase_id: H256,
    pub uncles_hash: H256,
    pub seal: Seal,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Seal {
    pub nonce: u64,
    pub proof: Vec<u8>,
}

impl Header {
    pub fn random() -> Self {
        Header {
            version: thread_rng().gen_range(0, 100),
            parent_hash: H256::random(),
            timestamp: thread_rng().gen_range(1500000000, 1600000000),
            number: thread_rng().gen_range(0, 100000),
            txs_commit: H256::random(),
            txs_proposal: H256::random(),
            difficulty: H256::random().into(),
            cellbase_id: H256::random(),
            uncles_hash: H256::random(),
            seal: Seal {
                nonce: thread_rng().gen_range(0, 100000),
                proof: H256::random().to_vec(),
            },
        }
    }

    pub fn to_flatbuffers(&self) -> Vec<u8> {
        let mut fbb = FlatBufferBuilder::new();
        let parent_hash = fbb.create_vector(&self.parent_hash);
        let txs_commit = fbb.create_vector(&self.txs_commit);
        let txs_proposal = fbb.create_vector(&self.txs_proposal);
        let difficulty = fbb.create_vector(&<[u8; 32]>::from(self.difficulty));
        let proof = fbb.create_vector(&self.seal.proof);
        let cellbase_id = fbb.create_vector(&self.cellbase_id);
        let uncles_hash = fbb.create_vector(&self.uncles_hash);

        let message = {
            let mut builder = HeaderBuilder::new(&mut fbb);
            builder.add_version(self.version);
            builder.add_parent_hash(parent_hash);
            builder.add_timestamp(self.timestamp);
            builder.add_number(self.number);
            builder.add_txs_commit(txs_commit);
            builder.add_txs_proposal(txs_proposal);
            builder.add_difficulty(difficulty);
            builder.add_nonce(self.seal.nonce);
            builder.add_proof(proof);
            builder.add_cellbase_id(cellbase_id);
            builder.add_uncles_hash(uncles_hash);
            builder.finish()
        };
        fbb.finish(message, None);
        fbb.finished_data().to_vec()
    }

    pub fn from_flatbuffers(data: &[u8]) -> Self {
        let header = get_root::<FlatbuffersHeader>(data);
        Header {
            version: header.version(),
            parent_hash: H256::from_slice(header.parent_hash().unwrap()),
            timestamp: header.timestamp(),
            number: header.number(),
            txs_commit: H256::from_slice(header.txs_commit().unwrap()),
            txs_proposal: H256::from_slice(header.txs_proposal().unwrap()),
            difficulty: H256::from_slice(header.difficulty().unwrap()).into(),
            cellbase_id: H256::from_slice(header.cellbase_id().unwrap()),
            uncles_hash: H256::from_slice(header.uncles_hash().unwrap()),
            seal: Seal {
                nonce: header.nonce(),
                proof: header.proof().unwrap().to_vec(),
            },
        }
    }

    pub fn to_protobuf(&self) -> Vec<u8> {
        let mut header = ProtobufHeader::new();
        header.set_version(self.version);
        header.set_parent_hash(self.parent_hash.to_vec());
        header.set_timestamp(self.timestamp);
        header.set_number(self.number);
        header.set_txs_commit(self.txs_commit.to_vec());
        header.set_txs_proposal(self.txs_proposal.to_vec());
        header.set_difficulty(<[u8; 32]>::from(self.difficulty).to_vec());
        header.set_nonce(self.seal.nonce);
        header.set_proof(self.seal.proof.to_vec());
        header.set_cellbase_id(self.cellbase_id.to_vec());
        header.set_uncles_hash(self.uncles_hash.to_vec());
        header.write_to_bytes().unwrap()
    }

    pub fn from_protobuf(data: &[u8]) -> Self {
        let header = parse_from_bytes::<ProtobufHeader>(data).unwrap();
        Header {
            version: header.get_version(),
            parent_hash: H256::from_slice(header.get_parent_hash()),
            timestamp: header.get_timestamp(),
            number: header.get_number(),
            txs_commit: H256::from_slice(header.get_txs_commit()),
            txs_proposal: H256::from_slice(header.get_txs_proposal()),
            difficulty: H256::from_slice(header.get_difficulty()).into(),
            cellbase_id: H256::from_slice(header.get_cellbase_id()),
            uncles_hash: H256::from_slice(header.get_uncles_hash()),
            seal: Seal {
                nonce: header.get_nonce(),
                proof: header.get_proof().to_vec(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod flatbuffers {
        use super::*;

        #[test]
        fn ser_de() {
            let header = Header::random();
            let data = header.to_flatbuffers();
            assert_eq!(header, Header::from_flatbuffers(&data));
        }
    }

    mod protobuf {
        use super::*;

        #[test]
        fn ser_de() {
            let header = Header::random();
            let data = header.to_protobuf();
            assert_eq!(header, Header::from_protobuf(&data));
        }
    }
}
