use criterion::{criterion_group, criterion_main, Criterion, Fun};
use serde_bench::Block;

fn bench(c: &mut Criterion) {
    let flatbuffers = Fun::new("flatbuffers", |b, block: &Block| {
        b.iter(|| block.to_flatbuffers())
    });
    let protobuf = Fun::new("protobuf", |b, block: &Block| {
        b.iter(|| block.to_protobuf())
    });
    let molecule = Fun::new("molecule", |b, block: &Block| {
        b.iter(|| block.to_molecule())
    });
    let functions = vec![flatbuffers, protobuf, molecule];
    let block = Block::random(100, 3);
    c.bench_functions("serialize_block", functions, block);
}

criterion_group!(benches, bench);
criterion_main!(benches);
