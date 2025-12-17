use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::io::Cursor;

use clpsr::parse_ipv4_nets;

/// Generates test data with the specified number of CIDR blocks.
///
/// Creates a string containing `size` CIDR blocks, one per line, using the pattern
/// `10.0.{i % 256}.0/24` to cycle through valid third octet values.
///
/// # Arguments
///
/// * `size` - Number of CIDR blocks to generate
///
/// # Returns
///
/// A string containing the CIDR blocks, one per line
fn generate_test_data(size: usize) -> String {
    let mut data = String::new();
    for i in 0..size {
        data.push_str(&format!("10.0.{}.0/24\n", i % 256));
    }
    data
}

/// Benchmarks parsing 10 CIDR blocks.
fn bench_parse_small(c: &mut Criterion) {
    let input = generate_test_data(10);
    c.bench_function("parse_10_cidrs", |b| {
        b.iter(|| {
            let reader = Cursor::new(black_box(&input));
            parse_ipv4_nets(reader).unwrap()
        })
    });
}

/// Benchmarks parsing 100 CIDR blocks.
fn bench_parse_medium(c: &mut Criterion) {
    let input = generate_test_data(100);
    c.bench_function("parse_100_cidrs", |b| {
        b.iter(|| {
            let reader = Cursor::new(black_box(&input));
            parse_ipv4_nets(reader).unwrap()
        })
    });
}

/// Benchmarks parsing 1000 CIDR blocks.
fn bench_parse_large(c: &mut Criterion) {
    let input = generate_test_data(1000);
    c.bench_function("parse_1000_cidrs", |b| {
        b.iter(|| {
            let reader = Cursor::new(black_box(&input));
            parse_ipv4_nets(reader).unwrap()
        })
    });
}

/// Benchmarks parsing 10000 CIDR blocks.
fn bench_parse_very_large(c: &mut Criterion) {
    let input = generate_test_data(10000);
    c.bench_function("parse_10000_cidrs", |b| {
        b.iter(|| {
            let reader = Cursor::new(black_box(&input));
            parse_ipv4_nets(reader).unwrap()
        })
    });
}

/// Benchmarks parsing 100 CIDR blocks interspersed with empty lines.
///
/// Tests the performance impact of skipping empty lines during parsing.
fn bench_parse_with_empty_lines(c: &mut Criterion) {
    let mut input = String::new();
    for i in 0..100 {
        input.push_str(&format!("10.0.{}.0/24\n", i % 256));
        if i % 10 == 0 {
            input.push_str("\n  \n\t\n");
        }
    }
    c.bench_function("parse_100_cidrs_with_empty_lines", |b| {
        b.iter(|| {
            let reader = Cursor::new(black_box(&input));
            parse_ipv4_nets(reader).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_large,
    bench_parse_very_large,
    bench_parse_with_empty_lines
);
criterion_main!(benches);
