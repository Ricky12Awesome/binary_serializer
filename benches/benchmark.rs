use criterion::{BenchmarkId, black_box, Criterion, criterion_group, criterion_main};
use binary_serializer::common::ByteEndian;
use binary_serializer::decoder::FromBytes;
use binary_serializer::encoder::ToBytes;

fn criterion_benchmark(c: &mut Criterion) {
  let bytes = vec![0u64; 16384].as_slice().to_bytes(ByteEndian::Little);

  c.bench_with_input(BenchmarkId::new("from_bytes-v2", bytes.len()), &bytes, |b, bytes| b.iter(|| {
    black_box(Vec::<u64>::from_bytes(bytes, ByteEndian::Little).unwrap());
  }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);