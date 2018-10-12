extern crate serde_bench;
#[macro_use]
extern crate criterion;
extern crate flatbuffers;
extern crate protobuf;

use criterion::{Criterion, Fun};
use serde_bench::Header;

fn bench(c: &mut Criterion) {
    let flatbuffers = Fun::new("flatbuffers", |b, h: &Header| b.iter(|| h.to_flatbuffers()));
    let protobuf = Fun::new("protobuf", |b, h: &Header| b.iter(|| h.to_protobuf()));
    let ssz = Fun::new("ssz", |b, h: &Header| b.iter(|| h.to_ssz()));
    let functions = vec![flatbuffers, protobuf, ssz];
    let header = Header::random();
    c.bench_functions("serialize", functions, header);
}

criterion_group!(benches, bench);
criterion_main!(benches);
