extern crate serde_bench;
#[macro_use]
extern crate criterion;
extern crate flatbuffers;
extern crate protobuf;

use criterion::{Criterion, Fun};
use serde_bench::Block;

fn bench(c: &mut Criterion) {
    let flatbuffers = Fun::new("flatbuffers", |b, block: &Block| {
        b.iter(|| block.to_flatbuffers())
    });
    let protobuf = Fun::new("protobuf", |b, block: &Block| {
        b.iter(|| block.to_protobuf())
    });
    let ssz = Fun::new("ssz", |b, block: &Block| b.iter(|| block.to_ssz()));
    let functions = vec![flatbuffers, protobuf, ssz];
    let block = Block::random(100, 3);
    c.bench_functions("serialize_block", functions, block);
}

criterion_group!(benches, bench);
criterion_main!(benches);
