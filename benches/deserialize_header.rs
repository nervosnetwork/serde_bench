use criterion::{criterion_group, criterion_main, Criterion, Fun};
use serde_bench::Header;

fn bench(c: &mut Criterion) {
    let flatbuffers = Fun::new("flatbuffers", |b, data: &[Vec<u8>; 3]| {
        b.iter(|| Header::from_flatbuffers(&data[0]))
    });
    let protobuf = Fun::new("protobuf", |b, data: &[Vec<u8>; 3]| {
        b.iter(|| Header::from_protobuf(&data[1]))
    });
    let molecule = Fun::new("molecule", |b, data: &[Vec<u8>; 3]| {
        b.iter(|| Header::from_molecule(&data[2]))
    });
    let functions = vec![flatbuffers, protobuf, molecule];
    let header = Header::random();
    let data = [header.to_flatbuffers(), header.to_protobuf(), header.to_molecule()];
    c.bench_functions("deserialize_header", functions, data);
}

criterion_group!(benches, bench);
criterion_main!(benches);
