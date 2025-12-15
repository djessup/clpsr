use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::Cursor;
use clpsr::parse_ipv4_nets;

fn generate_test_data(size: usize) -> String {
    let mut data = String::new();
    for i in 0..size {
        data.push_str(&format!("10.0.{}.0/24\n", i % 256));
    }
    data
}

fn bench_parse_small(c: &mut Criterion) {
    let input = generate_test_data(10);
    c.bench_function("parse_10_cidrs", |b| {
        b.iter(|| {
            let reader = Cursor::new(black_box(&input));
            parse_ipv4_nets(reader).unwrap()
        })
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    let input = generate_test_data(100);
    c.bench_function("parse_100_cidrs", |b| {
        b.iter(|| {
            let reader = Cursor::new(black_box(&input));
            parse_ipv4_nets(reader).unwrap()
        })
    });
}

fn bench_parse_large(c: &mut Criterion) {
    let input = generate_test_data(1000);
    c.bench_function("parse_1000_cidrs", |b| {
        b.iter(|| {
            let reader = Cursor::new(black_box(&input));
            parse_ipv4_nets(reader).unwrap()
        })
    });
}

fn bench_parse_very_large(c: &mut Criterion) {
    let input = generate_test_data(10000);
    c.bench_function("parse_10000_cidrs", |b| {
        b.iter(|| {
            let reader = Cursor::new(black_box(&input));
            parse_ipv4_nets(reader).unwrap()
        })
    });
}

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

