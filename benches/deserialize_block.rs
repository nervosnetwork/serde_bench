extern crate serde_bench;
#[macro_use]
extern crate criterion;
extern crate flatbuffers;
extern crate protobuf;

use criterion::{Criterion, Fun};
use serde_bench::Block;

fn bench(c: &mut Criterion) {
    let flatbuffers = Fun::new("flatbuffers", |b, data: &[Vec<u8>; 3]| {
        b.iter(|| Block::from_flatbuffers(&data[0]))
    });
    let protobuf = Fun::new("protobuf", |b, data: &[Vec<u8>; 3]| {
        b.iter(|| Block::from_protobuf(&data[1]))
    });
    let ssz = Fun::new("ssz", |b, data: &[Vec<u8>; 3]| {
        b.iter(|| Block::from_ssz(&data[2]))
    });
    let functions = vec![flatbuffers, protobuf, ssz];
    let block = Block::random(100, 3);
    let data = [block.to_flatbuffers(), block.to_protobuf(), block.to_ssz()];
    c.bench_functions("deserialize_block", functions, data);
}

criterion_group!(benches, bench);
criterion_main!(benches);
