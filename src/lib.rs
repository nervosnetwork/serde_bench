extern crate bigint;
extern crate flatbuffers;
extern crate protobuf;
extern crate rand;
extern crate ssz;

pub mod bench_flatbuffers;
pub mod bench_protobuf;

use bench_flatbuffers::{
    Block as FbsBlock, BlockBuilder, CellInput as FbsCellInput, CellInputBuilder,
    CellOutput as FbsCellOutput, CellOutputBuilder, Header as FbsHeader, HeaderBuilder,
    OutPoint as FbsOutPoint, OutPointBuilder, Transaction as FbsTransaction, TransactionBuilder,
};
use bench_protobuf::{
    Block as ProtobufBlock, CellInput as ProtobufCellInput, CellOutput as ProtobufCellOutput,
    Header as ProtobufHeader, OutPoint as ProtobufOutPoint, Transaction as ProtobufTransaction,
};
use bigint::{H256, U256};
use flatbuffers::{get_root, FlatBufferBuilder};
use protobuf::{parse_from_bytes, Message};
use rand::distributions::Standard;
use rand::{thread_rng, Rng};
use ssz::{decode_ssz, decode_ssz_list, Decodable, DecodeError, Encodable, SszStream};
use std::borrow::Borrow;

pub struct FlatbuffersVectorIterator<'a, T: flatbuffers::Follow<'a> + 'a> {
    vector: flatbuffers::Vector<'a, T>,
    counter: usize,
}

impl<'a, T: flatbuffers::Follow<'a> + 'a> FlatbuffersVectorIterator<'a, T> {
    pub fn new(vector: flatbuffers::Vector<'a, T>) -> Self {
        Self { vector, counter: 0 }
    }
}

impl<'a, T: flatbuffers::Follow<'a> + 'a> Iterator for FlatbuffersVectorIterator<'a, T> {
    type Item = T::Inner;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter < self.vector.len() {
            let result = self.vector.get(self.counter);
            self.counter += 1;
            Some(result)
        } else {
            None
        }
    }
}

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

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub version: u32,
    pub deps: Vec<OutPoint>,
    pub inputs: Vec<CellInput>,
    pub outputs: Vec<CellOutput>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct OutPoint {
    pub hash: H256,
    pub index: u32,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct CellInput {
    pub previous_output: OutPoint,
    pub unlock: Vec<u8>,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct CellOutput {
    pub capacity: u64,
    pub data: Vec<u8>,
    pub lock: H256,
}

impl<'a> From<&'a FbsHeader<'a>> for Header {
    fn from(header: &FbsHeader<'a>) -> Self {
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
}

impl<'a> From<&'a ProtobufHeader> for Header {
    fn from(header: &ProtobufHeader) -> Self {
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

impl<'a> From<&'a Header> for ProtobufHeader {
    fn from(header: &Header) -> Self {
        let mut result = ProtobufHeader::new();
        result.set_version(header.version);
        result.set_parent_hash(header.parent_hash.to_vec());
        result.set_timestamp(header.timestamp);
        result.set_number(header.number);
        result.set_txs_commit(header.txs_commit.to_vec());
        result.set_txs_proposal(header.txs_proposal.to_vec());
        result.set_difficulty(<[u8; 32]>::from(header.difficulty).to_vec());
        result.set_nonce(header.seal.nonce);
        result.set_proof(header.seal.proof.to_vec());
        result.set_cellbase_id(header.cellbase_id.to_vec());
        result.set_uncles_hash(header.uncles_hash.to_vec());
        result
    }
}

impl Encodable for Header {
    fn ssz_append(&self, s: &mut SszStream) {
        s.append(&self.version);
        s.append_encoded_raw(&self.parent_hash.to_vec());
        s.append(&self.timestamp);
        s.append(&self.number);
        s.append_encoded_raw(&self.txs_commit.to_vec());
        s.append_encoded_raw(&self.txs_proposal.to_vec());
        s.append_encoded_raw(&<[u8; 32]>::from(self.difficulty).to_vec());
        s.append(&self.seal.nonce);
        s.append_encoded_raw(&self.seal.proof.to_vec());
        s.append_encoded_raw(&self.cellbase_id.to_vec());
        s.append_encoded_raw(&self.uncles_hash.to_vec());
    }
}

impl Decodable for Header {
    fn ssz_decode(bytes: &[u8], index: usize) -> Result<(Self, usize), DecodeError> {
        let (version, index) = decode_ssz(bytes, index)?;
        let (parent_hash, index) = decode_ssz_h256(bytes, index)?;
        let (timestamp, index) = decode_ssz(bytes, index)?;
        let (number, index) = decode_ssz(bytes, index)?;
        let (txs_commit, index) = decode_ssz_h256(bytes, index)?;
        let (txs_proposal, index) = decode_ssz_h256(bytes, index)?;
        let (difficulty, index) = decode_ssz_h256(bytes, index)?;
        let (nonce, index) = decode_ssz(bytes, index)?;
        let (proof, index) = decode_ssz_h256(bytes, index)?;
        let (cellbase_id, index) = decode_ssz_h256(bytes, index)?;
        let (uncles_hash, index) = decode_ssz_h256(bytes, index)?;

        Ok((
            Header {
                version,
                parent_hash,
                timestamp,
                number,
                txs_commit,
                txs_proposal,
                difficulty: difficulty.into(),
                cellbase_id,
                uncles_hash,
                seal: Seal {
                    nonce,
                    proof: proof.to_vec(),
                },
            },
            index,
        ))
    }
}

impl Header {
    pub fn random() -> Self {
        Header {
            version: thread_rng().gen_range(1, 10),
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
        let fbb = &mut FlatBufferBuilder::new();
        let parent_hash = fbb.create_vector(&self.parent_hash);
        let txs_commit = fbb.create_vector(&self.txs_commit);
        let txs_proposal = fbb.create_vector(&self.txs_proposal);
        let difficulty = fbb.create_vector(&<[u8; 32]>::from(self.difficulty));
        let proof = fbb.create_vector(&self.seal.proof);
        let cellbase_id = fbb.create_vector(&self.cellbase_id);
        let uncles_hash = fbb.create_vector(&self.uncles_hash);

        let message = {
            let mut builder = HeaderBuilder::new(fbb);
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
        get_root::<FbsHeader>(data).borrow().into()
    }

    pub fn to_protobuf(&self) -> Vec<u8> {
        let header: ProtobufHeader = self.into();
        header.write_to_bytes().unwrap()
    }

    pub fn from_protobuf(data: &[u8]) -> Self {
        let header = parse_from_bytes::<ProtobufHeader>(data).unwrap();
        header.borrow().into()
    }

    pub fn to_ssz(&self) -> Vec<u8> {
        let mut ssz = SszStream::new();
        ssz.append(self);
        ssz.drain().to_vec()
    }

    pub fn from_ssz(data: &[u8]) -> Self {
        decode_ssz(data, 0).unwrap().0
    }
}

impl<'a> From<&'a FbsBlock<'a>> for Block {
    fn from(block: &FbsBlock<'a>) -> Self {
        Block {
            header: block.header().unwrap().borrow().into(),
            transactions: FlatbuffersVectorIterator::new(block.transactions().unwrap())
                .map(Into::into)
                .collect(),
        }
    }
}

impl<'a> From<&'a Block> for ProtobufBlock {
    fn from(block: &Block) -> Self {
        let mut result = ProtobufBlock::new();
        result.set_header(block.header.borrow().into());
        result.set_transactions(block.transactions.iter().map(Into::into).collect());
        result
    }
}

impl<'a> From<&'a ProtobufBlock> for Block {
    fn from(block: &ProtobufBlock) -> Self {
        Block {
            header: block.get_header().borrow().into(),
            transactions: block.get_transactions().iter().map(Into::into).collect(),
        }
    }
}

impl Encodable for Block {
    fn ssz_append(&self, s: &mut SszStream) {
        s.append(&self.header);
        s.append_vec(&self.transactions);
    }
}

impl Decodable for Block {
    fn ssz_decode(bytes: &[u8], index: usize) -> Result<(Self, usize), DecodeError> {
        let (header, index) = decode_ssz(bytes, index)?;
        let (transactions, index) = decode_ssz_list(bytes, index)?;
        Ok((
            Block {
                header,
                transactions,
            },
            index,
        ))
    }
}

impl Block {
    pub fn random(transactions_size: usize, io_size: usize) -> Self {
        Block {
            header: Header::random(),
            transactions: (0..transactions_size)
                .map(|_| Transaction::random(io_size))
                .collect(),
        }
    }

    pub fn to_flatbuffers(&self) -> Vec<u8> {
        let fbb = &mut FlatBufferBuilder::new();

        let header = {
            let parent_hash = fbb.create_vector(&self.header.parent_hash);
            let txs_commit = fbb.create_vector(&self.header.txs_commit);
            let txs_proposal = fbb.create_vector(&self.header.txs_proposal);
            let difficulty = fbb.create_vector(&<[u8; 32]>::from(self.header.difficulty));
            let proof = fbb.create_vector(&self.header.seal.proof);
            let cellbase_id = fbb.create_vector(&self.header.cellbase_id);
            let uncles_hash = fbb.create_vector(&self.header.uncles_hash);

            let mut builder = HeaderBuilder::new(fbb);
            builder.add_version(self.header.version);
            builder.add_parent_hash(parent_hash);
            builder.add_timestamp(self.header.timestamp);
            builder.add_number(self.header.number);
            builder.add_txs_commit(txs_commit);
            builder.add_txs_proposal(txs_proposal);
            builder.add_difficulty(difficulty);
            builder.add_nonce(self.header.seal.nonce);
            builder.add_proof(proof);
            builder.add_cellbase_id(cellbase_id);
            builder.add_uncles_hash(uncles_hash);
            builder.finish()
        };

        let vec = self
            .transactions
            .iter()
            .map(|transaction| {
                let vec = transaction
                    .deps
                    .iter()
                    .map(|out_point| {
                        let hash = fbb.create_vector(&out_point.hash);
                        let mut builder = OutPointBuilder::new(fbb);
                        builder.add_hash(hash);
                        builder.add_index(out_point.index);
                        builder.finish()
                    }).collect::<Vec<_>>();
                let deps = fbb.create_vector(&vec);

                let vec = transaction
                    .inputs
                    .iter()
                    .map(|input| {
                        let hash = fbb.create_vector(&input.previous_output.hash);
                        let unlock = fbb.create_vector(&input.unlock);
                        let mut builder = CellInputBuilder::new(fbb);
                        builder.add_hash(hash);
                        builder.add_index(input.previous_output.index);
                        builder.add_unlock(unlock);
                        builder.finish()
                    }).collect::<Vec<_>>();
                let inputs = fbb.create_vector(&vec);

                let vec = transaction
                    .outputs
                    .iter()
                    .map(|output| {
                        let data = fbb.create_vector(&output.data);
                        let lock = fbb.create_vector(&output.lock);
                        let mut builder = CellOutputBuilder::new(fbb);
                        builder.add_capacity(output.capacity);
                        builder.add_data(data);
                        builder.add_lock(lock);
                        builder.finish()
                    }).collect::<Vec<_>>();
                let outputs = fbb.create_vector(&vec);

                let mut builder = TransactionBuilder::new(fbb);
                builder.add_version(transaction.version);
                builder.add_deps(deps);
                builder.add_inputs(inputs);
                builder.add_outputs(outputs);
                builder.finish()
            }).collect::<Vec<_>>();

        let transactions = fbb.create_vector(&vec);

        let message = {
            let mut builder = BlockBuilder::new(fbb);
            builder.add_header(header);
            builder.add_transactions(transactions);
            builder.finish()
        };
        fbb.finish(message, None);
        fbb.finished_data().to_vec()
    }

    pub fn from_flatbuffers(data: &[u8]) -> Self {
        get_root::<FbsBlock>(data).borrow().into()
    }

    pub fn to_protobuf(&self) -> Vec<u8> {
        let block: ProtobufBlock = self.into();
        block.write_to_bytes().unwrap()
    }

    pub fn from_protobuf(data: &[u8]) -> Self {
        let block = parse_from_bytes::<ProtobufBlock>(data).unwrap();
        block.borrow().into()
    }

    pub fn to_ssz(&self) -> Vec<u8> {
        let mut ssz = SszStream::new();
        ssz.append(self);
        ssz.drain().to_vec()
    }

    pub fn from_ssz(data: &[u8]) -> Self {
        decode_ssz(data, 0).unwrap().0
    }
}

impl Transaction {
    pub fn random(io_size: usize) -> Self {
        Transaction {
            version: thread_rng().gen_range(1, 10),
            deps: (0..io_size).map(|_| OutPoint::random()).collect(),
            inputs: (0..io_size).map(|_| CellInput::random()).collect(),
            outputs: (0..io_size).map(|_| CellOutput::random()).collect(),
        }
    }
}

impl<'a> From<FbsTransaction<'a>> for Transaction {
    fn from(transaction: FbsTransaction<'a>) -> Self {
        let deps = FlatbuffersVectorIterator::new(transaction.deps().unwrap())
            .map(Into::into)
            .collect();

        let inputs = FlatbuffersVectorIterator::new(transaction.inputs().unwrap())
            .map(Into::into)
            .collect();

        let outputs = FlatbuffersVectorIterator::new(transaction.outputs().unwrap())
            .map(Into::into)
            .collect();

        Transaction {
            version: transaction.version(),
            deps,
            inputs,
            outputs,
        }
    }
}

impl<'a> From<&'a ProtobufTransaction> for Transaction {
    fn from(transaction: &ProtobufTransaction) -> Self {
        Transaction {
            version: transaction.get_version(),
            deps: transaction.get_deps().iter().map(Into::into).collect(),
            inputs: transaction.get_inputs().iter().map(Into::into).collect(),
            outputs: transaction.get_outputs().iter().map(Into::into).collect(),
        }
    }
}

impl<'a> From<&'a Transaction> for ProtobufTransaction {
    fn from(transaction: &Transaction) -> Self {
        let mut result = ProtobufTransaction::new();
        result.set_version(transaction.version);
        result.set_deps(transaction.deps.iter().map(Into::into).collect());
        result.set_inputs(transaction.inputs.iter().map(Into::into).collect());
        result.set_outputs(transaction.outputs.iter().map(Into::into).collect());
        result
    }
}

impl Encodable for Transaction {
    fn ssz_append(&self, s: &mut SszStream) {
        s.append(&self.version);
        s.append_vec(&self.deps);
        s.append_vec(&self.inputs);
        s.append_vec(&self.outputs);
    }
}

impl Decodable for Transaction {
    fn ssz_decode(bytes: &[u8], index: usize) -> Result<(Self, usize), DecodeError> {
        let (version, index) = decode_ssz(bytes, index)?;
        let (deps, index) = decode_ssz_list(bytes, index)?;
        let (inputs, index) = decode_ssz_list(bytes, index)?;
        let (outputs, index) = decode_ssz_list(bytes, index)?;

        Ok((
            Transaction {
                version,
                deps,
                inputs,
                outputs,
            },
            index,
        ))
    }
}

impl OutPoint {
    pub fn random() -> Self {
        OutPoint {
            hash: H256::random(),
            index: thread_rng().gen_range(1, 10),
        }
    }
}

impl<'a> From<FbsOutPoint<'a>> for OutPoint {
    fn from(out_point: FbsOutPoint<'a>) -> Self {
        OutPoint {
            hash: H256::from_slice(out_point.hash().unwrap()),
            index: out_point.index(),
        }
    }
}

impl<'a> From<&'a ProtobufOutPoint> for OutPoint {
    fn from(out_point: &ProtobufOutPoint) -> Self {
        OutPoint {
            hash: H256::from_slice(out_point.get_hash()),
            index: out_point.get_index(),
        }
    }
}

impl<'a> From<&'a OutPoint> for ProtobufOutPoint {
    fn from(out_point: &OutPoint) -> Self {
        let mut result = ProtobufOutPoint::new();
        result.set_hash(out_point.hash.to_vec());
        result.set_index(out_point.index);
        result
    }
}

impl Encodable for OutPoint {
    fn ssz_append(&self, s: &mut SszStream) {
        s.append_encoded_raw(&self.hash.to_vec());
        s.append(&self.index);
    }
}

impl Decodable for OutPoint {
    fn ssz_decode(bytes: &[u8], index: usize) -> Result<(Self, usize), DecodeError> {
        let (hash, index) = decode_ssz_h256(bytes, index)?;
        let (oindex, index) = decode_ssz(bytes, index)?;
        Ok((
            OutPoint {
                hash,
                index: oindex,
            },
            index,
        ))
    }
}

impl CellInput {
    pub fn random() -> Self {
        CellInput {
            previous_output: OutPoint::random(),
            unlock: thread_rng().sample_iter(&Standard).take(100).collect(),
        }
    }
}

impl<'a> From<FbsCellInput<'a>> for CellInput {
    fn from(cell_input: FbsCellInput<'a>) -> Self {
        CellInput {
            previous_output: OutPoint {
                hash: H256::from_slice(cell_input.hash().unwrap()),
                index: cell_input.index(),
            },
            unlock: cell_input.unlock().unwrap().to_vec(),
        }
    }
}

impl<'a> From<&'a ProtobufCellInput> for CellInput {
    fn from(cell_input: &ProtobufCellInput) -> Self {
        CellInput {
            previous_output: OutPoint {
                hash: H256::from_slice(cell_input.get_hash()),
                index: cell_input.get_index(),
            },
            unlock: cell_input.get_unlock().to_vec(),
        }
    }
}

impl<'a> From<&'a CellInput> for ProtobufCellInput {
    fn from(cell_input: &CellInput) -> Self {
        let mut result = ProtobufCellInput::new();
        result.set_hash(cell_input.previous_output.hash.to_vec());
        result.set_index(cell_input.previous_output.index);
        result.set_unlock(cell_input.unlock.to_vec());
        result
    }
}

impl Encodable for CellInput {
    fn ssz_append(&self, s: &mut SszStream) {
        s.append_encoded_raw(&self.previous_output.hash.to_vec());
        s.append(&self.previous_output.index);
        s.append_vec(&self.unlock.to_vec());
    }
}

impl Decodable for CellInput {
    fn ssz_decode(bytes: &[u8], index: usize) -> Result<(Self, usize), DecodeError> {
        let (hash, index) = decode_ssz_h256(bytes, index)?;
        let (oindex, index) = decode_ssz(bytes, index)?;
        let (unlock, index) = decode_ssz_list(bytes, index)?;
        Ok((
            CellInput {
                previous_output: OutPoint {
                    hash,
                    index: oindex,
                },
                unlock,
            },
            index,
        ))
    }
}

impl CellOutput {
    pub fn random() -> Self {
        CellOutput {
            capacity: thread_rng().gen_range(600, 1000),
            data: thread_rng().sample_iter(&Standard).take(600).collect(),
            lock: H256::random(),
        }
    }
}

impl<'a> From<FbsCellOutput<'a>> for CellOutput {
    fn from(cell_output: FbsCellOutput<'a>) -> Self {
        CellOutput {
            capacity: cell_output.capacity(),
            data: cell_output.data().unwrap().to_vec(),
            lock: H256::from_slice(cell_output.lock().unwrap()),
        }
    }
}

impl<'a> From<&'a ProtobufCellOutput> for CellOutput {
    fn from(cell_output: &ProtobufCellOutput) -> Self {
        CellOutput {
            capacity: cell_output.get_capacity(),
            data: cell_output.get_data().to_vec(),
            lock: H256::from_slice(cell_output.get_lock()),
        }
    }
}

impl<'a> From<&'a CellOutput> for ProtobufCellOutput {
    fn from(cell_output: &CellOutput) -> Self {
        let mut result = ProtobufCellOutput::new();
        result.set_capacity(cell_output.capacity);
        result.set_data(cell_output.data.to_vec());
        result.set_lock(cell_output.lock.to_vec());
        result
    }
}

impl Encodable for CellOutput {
    fn ssz_append(&self, s: &mut SszStream) {
        s.append(&self.capacity);
        s.append_vec(&self.data.to_vec());
        s.append_encoded_raw(&self.lock.to_vec());
    }
}

impl Decodable for CellOutput {
    fn ssz_decode(bytes: &[u8], index: usize) -> Result<(Self, usize), DecodeError> {
        let (capacity, index) = decode_ssz(bytes, index)?;
        let (data, index) = decode_ssz_list(bytes, index)?;
        let (lock, index) = decode_ssz_h256(bytes, index)?;
        Ok((
            CellOutput {
                capacity,
                data,
                lock,
            },
            index,
        ))
    }
}

pub fn decode_ssz_h256(bytes: &[u8], index: usize) -> Result<(H256, usize), DecodeError> {
    if bytes.len() < 32 || bytes.len() - 32 < index {
        Err(DecodeError::TooShort)
    } else {
        Ok((H256::from(&bytes[index..(index + 32)]), index + 32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod flatbuffers {
        use super::*;

        #[test]
        fn ser_de_header() {
            let header = Header::random();
            let data = header.to_flatbuffers();
            assert_eq!(header, Header::from_flatbuffers(&data));
        }

        #[test]
        fn ser_de_block() {
            let block = Block::random(100, 3);
            let data = block.to_flatbuffers();
            assert_eq!(block, Block::from_flatbuffers(&data));
        }

        #[test]
        fn data_size() {
            let size: usize = (0..100)
                .map(|_| Header::random().to_flatbuffers().len())
                .sum();
            println!("flatbuffers header size: {}", size);

            let size: usize = (0..100)
                .map(|_| Block::random(100, 3).to_flatbuffers().len())
                .sum();
            println!("flatbuffers block size: {}", size);
        }
    }

    mod protobuf {
        use super::*;

        #[test]
        fn ser_de_header() {
            let header = Header::random();
            let data = header.to_protobuf();
            assert_eq!(header, Header::from_protobuf(&data));
        }

        #[test]
        fn ser_de_block() {
            let block = Block::random(100, 3);
            let data = block.to_protobuf();
            assert_eq!(block, Block::from_protobuf(&data));
        }

        #[test]
        fn data_size() {
            let size: usize = (0..100).map(|_| Header::random().to_protobuf().len()).sum();
            println!("protobuf size: {}", size);

            let size: usize = (0..100).map(|_| Block::random(100, 3).to_protobuf().len()).sum();
            println!("protobuf block size: {}", size);
        }
    }

    mod ssz {
        use super::*;

        #[test]
        fn ser_de_header() {
            let header = Header::random();
            let data = header.to_ssz();
            assert_eq!(header, Header::from_ssz(&data));
        }

        #[test]
        fn ser_de_block() {
            let block = Block::random(100, 3);
            let data = block.to_ssz();
            assert_eq!(block, Block::from_ssz(&data));
        }

        #[test]
        fn data_size() {
            let size: usize = (0..100).map(|_| Header::random().to_ssz().len()).sum();
            println!("ssz header size: {}", size);

            let size: usize = (0..100).map(|_| Block::random(100, 3).to_ssz().len()).sum();
            println!("ssz block size: {}", size);
        }
    }
}
