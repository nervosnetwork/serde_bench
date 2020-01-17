use criterion::{criterion_group, criterion_main, Criterion, Fun};
use serde_bench::Header;

fn bench(c: &mut Criterion) {
    let flatbuffers = Fun::new("flatbuffers", |b, h: &Header| b.iter(|| h.to_flatbuffers()));
    let protobuf = Fun::new("protobuf", |b, h: &Header| b.iter(|| h.to_protobuf()));
    let functions = vec![flatbuffers, protobuf];
    let header = Header::random();
    c.bench_functions("serialize_header", functions, header);
}

criterion_group!(benches, bench);
criterion_main!(benches);
