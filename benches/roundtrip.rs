//! Benchmark ZeroProto roundtrip performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zeroproto::{MessageBuilder, MessageReader};

fn bench_scalar_roundtrip(c: &mut Criterion) {
    c.bench_function("scalar_roundtrip", |b| {
        b.iter(|| {
            let mut builder = MessageBuilder::new();
            builder.set_scalar(0, 42u32).unwrap();
            builder.set_scalar(1, 3.14f64).unwrap();
            builder.set_scalar(2, true).unwrap();
            let data = builder.finish();

            let reader = MessageReader::new(black_box(&data)).unwrap();
            let _: u32 = reader.get_scalar(0).unwrap();
            let _: f64 = reader.get_scalar(1).unwrap();
            let _: bool = reader.get_scalar(2).unwrap();
        });
    });
}

fn bench_string_roundtrip(c: &mut Criterion) {
    let test_string = "Hello, ZeroProto! This is a test string for benchmarking.";

    c.bench_function("string_roundtrip", |b| {
        b.iter(|| {
            let mut builder = MessageBuilder::new();
            builder.set_string(0, black_box(test_string)).unwrap();
            let data = builder.finish();

            let reader = MessageReader::new(black_box(&data)).unwrap();
            let _: &str = reader.get_string(0).unwrap();
        });
    });
}

fn bench_vector_roundtrip(c: &mut Criterion) {
    let test_vector: Vec<u64> = (0..1000).collect();

    c.bench_function("vector_roundtrip", |b| {
        b.iter(|| {
            let mut builder = MessageBuilder::new();
            builder.set_vector(0, black_box(&test_vector)).unwrap();
            let data = builder.finish();

            let reader = MessageReader::new(black_box(&data)).unwrap();
            let vector_reader = reader.get_vector::<u64>(0).unwrap();
            let _: Vec<u64> = vector_reader.collect().unwrap();
        });
    });
}

fn bench_nested_message_roundtrip(c: &mut Criterion) {
    c.bench_function("nested_message_roundtrip", |b| {
        b.iter(|| {
            // Create nested message
            let mut nested_builder = MessageBuilder::new();
            nested_builder.set_scalar(0, 123u64).unwrap();
            nested_builder.set_string(1, "nested").unwrap();
            let nested_data = nested_builder.finish();

            // Create outer message
            let mut outer_builder = MessageBuilder::new();
            outer_builder.set_scalar(0, 456u64).unwrap();
            outer_builder
                .set_message(1, black_box(&nested_data))
                .unwrap();
            let outer_data = outer_builder.finish();

            // Read back
            let outer_reader = MessageReader::new(black_box(&outer_data)).unwrap();
            let _: u64 = outer_reader.get_scalar(0).unwrap();

            let nested_reader = outer_reader.get_message(1).unwrap();
            let _: u64 = nested_reader.get_scalar(0).unwrap();
            let _: &str = nested_reader.get_string(1).unwrap();
        });
    });
}

fn bench_large_message(c: &mut Criterion) {
    c.bench_function("large_message", |b| {
        b.iter(|| {
            let mut builder = MessageBuilder::new();

            // Add many fields
            for i in 0..100 {
                builder.set_scalar(i, i as u32).unwrap();
            }

            let data = builder.finish();

            let reader = MessageReader::new(black_box(&data)).unwrap();
            for i in 0..100 {
                let _: u32 = reader.get_scalar(i).unwrap();
            }
        });
    });
}

fn bench_serialization_only(c: &mut Criterion) {
    c.bench_function("serialization_only", |b| {
        b.iter(|| {
            let mut builder = MessageBuilder::new();
            builder.set_scalar(0, 42u32).unwrap();
            builder.set_scalar(1, 3.14f64).unwrap();
            builder.set_string(2, "test string").unwrap();

            let _: Vec<u8> = builder.finish();
        });
    });
}

fn bench_deserialization_only(c: &mut Criterion) {
    // Pre-create the data
    let mut builder = MessageBuilder::new();
    builder.set_scalar(0, 42u32).unwrap();
    builder.set_scalar(1, 3.14f64).unwrap();
    builder.set_string(2, "test string").unwrap();
    let data = builder.finish();

    c.bench_function("deserialization_only", |b| {
        b.iter(|| {
            let reader = MessageReader::new(black_box(&data)).unwrap();
            let _: u32 = reader.get_scalar(0).unwrap();
            let _: f64 = reader.get_scalar(1).unwrap();
            let _: &str = reader.get_string(2).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_scalar_roundtrip,
    bench_string_roundtrip,
    bench_vector_roundtrip,
    bench_nested_message_roundtrip,
    bench_large_message,
    bench_serialization_only,
    bench_deserialization_only
);

criterion_main!(benches);
