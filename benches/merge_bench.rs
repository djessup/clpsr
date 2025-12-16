use clpsr::merge_ipv4_nets;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ipnet::Ipv4Net;

/// Generates a vector of adjacent networks that can be merged.
///
/// Creates networks using the pattern `10.{third_octet}.{octet}.0/24` where
/// the third and fourth octets cycle through valid values.
///
/// # Arguments
///
/// * `size` - Number of networks to generate
///
/// # Returns
///
/// A vector of IPv4 networks
fn generate_adjacent_networks(size: usize) -> Vec<Ipv4Net> {
    let mut nets = Vec::new();
    for i in 0..size {
        let octet = (i % 256) as u8;
        let third_octet = ((i / 256) % 256) as u8;
        let net_str = format!("10.{}.{}.0/24", third_octet, octet);
        nets.push(net_str.parse().unwrap());
    }
    nets
}

/// Generates pairs of adjacent networks that can be merged.
///
/// Creates `size` pairs of adjacent /24 networks, where each pair consists of
/// two consecutive networks (e.g., `10.0.0.0/24` and `10.0.1.0/24`).
///
/// # Arguments
///
/// * `size` - Number of pairs to generate (results in `2 * size` networks)
///
/// # Returns
///
/// A vector of IPv4 networks containing mergeable pairs
fn generate_mergeable_networks(size: usize) -> Vec<Ipv4Net> {
    let mut nets = Vec::new();
    // Generate pairs of adjacent networks that can be merged
    for i in 0..size {
        let base = i * 2;
        let first = format!("10.0.{}.0/24", base % 256);
        let second = format!("10.0.{}.0/24", (base + 1) % 256);
        nets.push(first.parse().unwrap());
        nets.push(second.parse().unwrap());
    }
    nets
}

/// Generates networks that are far apart and cannot be merged.
///
/// Creates networks using the pattern `10.{i % 256}.0.0/24`, where each network
/// is in a different third octet, making them non-adjacent and non-mergeable.
///
/// # Arguments
///
/// * `size` - Number of networks to generate
///
/// # Returns
///
/// A vector of IPv4 networks that cannot be merged
fn generate_non_mergeable_networks(size: usize) -> Vec<Ipv4Net> {
    let mut nets = Vec::new();
    // Generate networks that are far apart and cannot be merged
    for i in 0..size {
        let net_str = format!("10.{}.0.0/24", i % 256);
        nets.push(net_str.parse().unwrap());
    }
    nets
}

/// Benchmarks merging 10 adjacent networks.
fn bench_merge_small_adjacent(c: &mut Criterion) {
    let nets = generate_adjacent_networks(10);
    c.bench_function("merge_10_adjacent_networks", |b| {
        b.iter(|| merge_ipv4_nets(black_box(nets.clone()), 0))
    });
}

/// Benchmarks merging 100 adjacent networks.
fn bench_merge_medium_adjacent(c: &mut Criterion) {
    let nets = generate_adjacent_networks(100);
    c.bench_function("merge_100_adjacent_networks", |b| {
        b.iter(|| merge_ipv4_nets(black_box(nets.clone()), 0))
    });
}

/// Benchmarks merging 1000 adjacent networks.
fn bench_merge_large_adjacent(c: &mut Criterion) {
    let nets = generate_adjacent_networks(1000);
    c.bench_function("merge_1000_adjacent_networks", |b| {
        b.iter(|| merge_ipv4_nets(black_box(nets.clone()), 0))
    });
}

/// Benchmarks merging 100 networks arranged in 50 mergeable pairs.
fn bench_merge_mergeable_pairs(c: &mut Criterion) {
    let nets = generate_mergeable_networks(50); // 50 pairs = 100 networks
    c.bench_function("merge_100_mergeable_pairs", |b| {
        b.iter(|| merge_ipv4_nets(black_box(nets.clone()), 0))
    });
}

/// Benchmarks merging 100 non-mergeable networks.
///
/// Tests the performance when no merges are possible, measuring the overhead
/// of the merge algorithm when it cannot reduce the network count.
fn bench_merge_non_mergeable(c: &mut Criterion) {
    let nets = generate_non_mergeable_networks(100);
    c.bench_function("merge_100_non_mergeable_networks", |b| {
        b.iter(|| merge_ipv4_nets(black_box(nets.clone()), 0))
    });
}

/// Benchmarks merging 100 networks with gaps using tolerance.
///
/// Generates networks with gaps between them that can only be merged when
/// tolerance is enabled (e.g., `10.0.0.0/24` and `10.0.2.0/24`).
fn bench_merge_with_tolerance(c: &mut Criterion) {
    let mut nets = Vec::new();
    // Generate networks with gaps that can be merged with tolerance
    for i in 0..50 {
        let base = i * 2;
        nets.push(format!("10.0.{}.0/24", base).parse().unwrap());
        nets.push(format!("10.0.{}.0/24", base + 2).parse().unwrap());
    }
    c.bench_function("merge_100_networks_with_tolerance", |b| {
        b.iter(|| merge_ipv4_nets(black_box(nets.clone()), 512))
    });
}

/// Benchmarks merging 110 networks containing covered subnets.
///
/// Generates a mix of /16 supernets and their /24 subnets to test the
/// performance of removing covered networks during the merge process.
fn bench_merge_with_covered_subnets(c: &mut Criterion) {
    let mut nets = Vec::new();
    // Generate a mix of supernets and subnets
    for i in 0..10 {
        nets.push(format!("10.{}.0.0/16", i).parse().unwrap());
        for j in 0..10 {
            nets.push(format!("10.{}.{}.0/24", i, j).parse().unwrap());
        }
    }
    c.bench_function("merge_110_networks_with_covered_subnets", |b| {
        b.iter(|| merge_ipv4_nets(black_box(nets.clone()), 0))
    });
}

/// Benchmarks merging 16 networks in a scenario requiring multiple iterations.
///
/// Tests performance when the merge algorithm needs to run multiple passes
/// to fully merge all adjacent networks (e.g., 16 consecutive /24s merging
/// into a single /20).
fn bench_merge_iterative_scenario(c: &mut Criterion) {
    // Scenario that requires multiple iterations
    let mut nets = Vec::new();
    for i in 0..16 {
        nets.push(format!("10.0.{}.0/24", i).parse().unwrap());
    }
    c.bench_function("merge_16_networks_iterative", |b| {
        b.iter(|| merge_ipv4_nets(black_box(nets.clone()), 0))
    });
}

/// Benchmarks merging an empty vector of networks.
///
/// Tests the overhead of the merge algorithm with no input.
fn bench_merge_empty(c: &mut Criterion) {
    let nets = Vec::new();
    c.bench_function("merge_empty", |b| {
        b.iter(|| merge_ipv4_nets(black_box(nets.clone()), 0))
    });
}

/// Benchmarks merging a single network.
///
/// Tests the overhead of the merge algorithm with minimal input.
fn bench_merge_single(c: &mut Criterion) {
    let nets = vec!["10.0.0.0/24".parse::<Ipv4Net>().unwrap()];
    c.bench_function("merge_single_network", |b| {
        b.iter(|| merge_ipv4_nets(black_box(nets.clone()), 0))
    });
}

criterion_group!(
    benches,
    bench_merge_empty,
    bench_merge_single,
    bench_merge_small_adjacent,
    bench_merge_medium_adjacent,
    bench_merge_large_adjacent,
    bench_merge_mergeable_pairs,
    bench_merge_non_mergeable,
    bench_merge_with_tolerance,
    bench_merge_with_covered_subnets,
    bench_merge_iterative_scenario
);
criterion_main!(benches);
