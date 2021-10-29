use criterion::{BenchmarkId, black_box, Criterion, criterion_group, criterion_main};

fn criterion_benchmark(c: &mut Criterion) {
  {
    use binary_serializer::v1::encoder::ToBytes;
    use binary_serializer::v1::decoder::FromBytes;

    let bytes = [0u64; 16384].to_bytes();

    c.bench_with_input(BenchmarkId::new("from_bytes-v1", bytes.len()), &bytes, |b, bytes| b.iter(|| {
      let bytes = bytes.clone();

      black_box(Vec::<u64>::from_bytes(bytes).unwrap());
    }));
  }

  {
    use binary_serializer::v2::encoder::ToBytes;
    use binary_serializer::v2::decoder::FromBytes;
    use binary_serializer::v2::common::ByteEndian;

    let bytes = [0u64; 16384].to_bytes();

    c.bench_with_input(BenchmarkId::new("from_bytes-v2", bytes.len()), &bytes, |b, bytes| b.iter(|| {
      black_box(Vec::<u64>::from_bytes(bytes, ByteEndian::Little).unwrap());
    }));
  }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);