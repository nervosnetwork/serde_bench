extern crate serde_bench;
#[macro_use]
extern crate criterion;
extern crate flatbuffers;
extern crate protobuf;

use criterion::{Criterion, Fun};
use serde_bench::Header;

fn bench(c: &mut Criterion) {
    let flatbuffers = Fun::new("flatbuffers", |b, data: &[Vec<u8>; 2]| {
        b.iter(|| Header::from_flatbuffers(&data[0]))
    });
    let protobuf = Fun::new("protobuf", |b, data: &[Vec<u8>; 2]| {
        b.iter(|| Header::from_protobuf(&data[1]))
    });
    let functions = vec![flatbuffers, protobuf];
    let header = Header::random();
    let data = [header.to_flatbuffers(), header.to_protobuf()];
    c.bench_functions("deserialize", functions, data);
}

criterion_group!(benches, bench);
criterion_main!(benches);
