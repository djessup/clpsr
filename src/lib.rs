use std::io::BufRead;

use ipnet::Ipv4Net;

/// Parse IPv4 CIDRs from the provided buffered reader.
///
/// Empty lines are ignored. Invalid CIDRs return a descriptive error with the
/// offending line number.
pub fn parse_ipv4_nets<R: BufRead>(reader: R) -> Result<Vec<Ipv4Net>, String> {
    let mut nets = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let raw = line.map_err(|err| format!("Failed to read line {}: {err}", idx + 1))?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        match trimmed.parse::<Ipv4Net>() {
            Ok(net) => nets.push(net),
            Err(err) => return Err(format!("Line {}: {err}", idx + 1)),
        }
    }

    Ok(nets)
}

/// Normalize, deduplicate, and merge IPv4 CIDRs into a minimal covering set.
///
/// This function merges adjacent networks with identical prefix lengths when
/// their combined supernet exactly represents the two subnets. When `tolerance`
/// is greater than 0, it may also merge networks that introduce extra addresses
/// as long as the added address count does not exceed the tolerance.
///
/// # Arguments
///
/// * `nets` - Vector of IPv4 networks to merge
/// * `tolerance` - Maximum number of extra addresses allowed when merging (0 for lossless merging only)
pub fn merge_ipv4_nets(nets: Vec<Ipv4Net>, tolerance: u64) -> Vec<Ipv4Net> {
    let mut normalized = nets;
    sort_and_dedup(&mut normalized);

    let mut changed = true;
    while changed {
        changed = false;
        let mut merged: Vec<Ipv4Net> = Vec::new();
        let mut idx = 0;

        while idx < normalized.len() {
            // Try to merge with next network
            if idx + 1 < normalized.len()
                && let Some((supernet, _extra_addrs)) =
                    try_merge_with_tolerance(&normalized[idx], &normalized[idx + 1], tolerance)
            {
                merged.push(supernet);
                changed = true;
                idx += 2;
                continue;
            }

            merged.push(normalized[idx]);
            idx += 1;
        }

        sort_and_dedup(&mut merged);
        let (compacted, removed_subnets) = remove_covered_nets(merged);
        changed |= removed_subnets;
        normalized = compacted;
    }

    normalized
}

#[cfg(test)]
pub(crate) fn sort_and_dedup(nets: &mut Vec<Ipv4Net>) {
    nets.sort_by(|a, b| {
        u32::from(a.addr())
            .cmp(&u32::from(b.addr()))
            .then(a.prefix_len().cmp(&b.prefix_len()))
    });
    nets.dedup();
}

#[cfg(not(test))]
fn sort_and_dedup(nets: &mut Vec<Ipv4Net>) {
    nets.sort_by(|a, b| {
        u32::from(a.addr())
            .cmp(&u32::from(b.addr()))
            .then(a.prefix_len().cmp(&b.prefix_len()))
    });
    nets.dedup();
}

#[cfg(test)]
pub(crate) fn remove_covered_nets(nets: Vec<Ipv4Net>) -> (Vec<Ipv4Net>, bool) {
    remove_covered_nets_impl(nets)
}

#[cfg(not(test))]
fn remove_covered_nets(nets: Vec<Ipv4Net>) -> (Vec<Ipv4Net>, bool) {
    remove_covered_nets_impl(nets)
}

fn remove_covered_nets_impl(nets: Vec<Ipv4Net>) -> (Vec<Ipv4Net>, bool) {
    if nets.is_empty() {
        return (nets, false);
    }

    let mut compacted = Vec::with_capacity(nets.len());
    compacted.push(nets[0]);

    for net in nets.into_iter().skip(1) {
        if let Some(last) = compacted.last()
            && network_covers_impl(last, &net)
        {
            continue;
        }

        compacted.push(net);
    }

    let removed_any = compacted.len() < compacted.capacity();
    (compacted, removed_any)
}

#[cfg(test)]
pub(crate) fn network_covers(supernet: &Ipv4Net, subnet: &Ipv4Net) -> bool {
    network_covers_impl(supernet, subnet)
}

#[cfg(not(test))]
fn network_covers(supernet: &Ipv4Net, subnet: &Ipv4Net) -> bool {
    network_covers_impl(supernet, subnet)
}

fn network_covers_impl(supernet: &Ipv4Net, subnet: &Ipv4Net) -> bool {
    if supernet.prefix_len() > subnet.prefix_len() {
        return false;
    }

    let super_start = u32::from(supernet.network());
    let super_end = u32::from(supernet.broadcast());

    let sub_start = u32::from(subnet.network());
    let sub_end = u32::from(subnet.broadcast());

    super_start <= sub_start && super_end >= sub_end
}

/// Attempts to merge two networks, returning the supernet and extra address count if successful.
///
/// Returns `Some((supernet, extra_addrs))` if the networks can be merged within tolerance,
/// where `extra_addrs` is the number of addresses in the supernet that weren't in the original networks.
/// Returns `None` if merging is not possible or would exceed tolerance.
fn try_merge_with_tolerance(a: &Ipv4Net, b: &Ipv4Net, tolerance: u64) -> Option<(Ipv4Net, u64)> {
    // First, try exact merge (lossless)
    if let Some(supernet) = try_merge_exact(a, b) {
        return Some((supernet, 0));
    }

    // If tolerance is 0, only exact merges are allowed
    if tolerance == 0 {
        return None;
    }

    // Find the minimal supernet that covers both networks
    let covering_supernet = find_covering_supernet(a, b)?;

    // Calculate addresses in original networks
    let a_addrs = network_address_count(a);
    let b_addrs = network_address_count(b);

    // Check for overlap - if networks overlap, we need to account for that
    let overlap = network_overlap(a, b);
    let original_total = a_addrs + b_addrs - overlap;

    // Calculate addresses in supernet
    let supernet_addrs = network_address_count(&covering_supernet);

    // Extra addresses = supernet addresses - original addresses
    let extra_addrs = supernet_addrs.saturating_sub(original_total);

    // Accept merge if within tolerance
    if extra_addrs <= tolerance {
        Some((covering_supernet, extra_addrs))
    } else {
        None
    }
}

/// Attempts an exact (lossless) merge of two networks.
/// Only succeeds if networks are adjacent with identical prefix lengths.
#[cfg(test)]
pub(crate) fn try_merge_exact(a: &Ipv4Net, b: &Ipv4Net) -> Option<Ipv4Net> {
    try_merge_exact_impl(a, b)
}

#[cfg(not(test))]
fn try_merge_exact(a: &Ipv4Net, b: &Ipv4Net) -> Option<Ipv4Net> {
    try_merge_exact_impl(a, b)
}

fn try_merge_exact_impl(a: &Ipv4Net, b: &Ipv4Net) -> Option<Ipv4Net> {
    if a.prefix_len() != b.prefix_len() || a.prefix_len() == 0 {
        return None;
    }

    let prefix = a.prefix_len();
    let block_size = 1u64 << (32 - prefix);
    let a_net = u32::from(a.addr()) as u64;
    let b_net = u32::from(b.addr()) as u64;

    if !a_net.is_multiple_of(block_size * 2) {
        return None;
    }

    if a_net + block_size != b_net {
        return None;
    }

    Ipv4Net::new(a.addr(), prefix - 1).ok()
}

/// Finds the minimal supernet that covers both networks.
/// Returns None if no such supernet exists (shouldn't happen for valid IPv4 networks).
#[cfg(test)]
pub(crate) fn find_covering_supernet(a: &Ipv4Net, b: &Ipv4Net) -> Option<Ipv4Net> {
    find_covering_supernet_impl(a, b)
}

#[cfg(not(test))]
fn find_covering_supernet(a: &Ipv4Net, b: &Ipv4Net) -> Option<Ipv4Net> {
    find_covering_supernet_impl(a, b)
}

fn find_covering_supernet_impl(a: &Ipv4Net, b: &Ipv4Net) -> Option<Ipv4Net> {
    let a_start = u32::from(a.network());
    let a_end = u32::from(a.broadcast());
    let b_start = u32::from(b.network());
    let b_end = u32::from(b.broadcast());

    let min_start = a_start.min(b_start);
    let max_end = a_end.max(b_end);

    // Find the smallest prefix length (largest block) that can cover the range
    let range_size = (max_end - min_start + 1) as u64;

    // Calculate required prefix length: find largest n (smallest prefix length) where 2^(32-n) >= range_size
    // This is equivalent to: n = floor(32 - log2(range_size))
    // We iterate from largest to smallest to find the first (largest n) that works
    let mut prefix_len = 32;
    for n in (0..=32).rev() {
        let block_size = 1u64 << (32 - n);
        if block_size >= range_size {
            prefix_len = n;
            break;
        }
    }

    // Align the network address to the prefix boundary
    let block_size = 1u64 << (32 - prefix_len);
    let aligned_start = (min_start as u64 / block_size) * block_size;

    Ipv4Net::new(std::net::Ipv4Addr::from(aligned_start as u32), prefix_len).ok()
}

/// Returns the number of addresses in a network.
#[cfg(test)]
pub(crate) fn network_address_count(net: &Ipv4Net) -> u64 {
    1u64 << (32 - net.prefix_len())
}

#[cfg(not(test))]
fn network_address_count(net: &Ipv4Net) -> u64 {
    1u64 << (32 - net.prefix_len())
}

/// Calculates the number of overlapping addresses between two networks.
#[cfg(test)]
pub(crate) fn network_overlap(a: &Ipv4Net, b: &Ipv4Net) -> u64 {
    network_overlap_impl(a, b)
}

#[cfg(not(test))]
fn network_overlap(a: &Ipv4Net, b: &Ipv4Net) -> u64 {
    network_overlap_impl(a, b)
}

fn network_overlap_impl(a: &Ipv4Net, b: &Ipv4Net) -> u64 {
    let a_start = u32::from(a.network());
    let a_end = u32::from(a.broadcast());
    let b_start = u32::from(b.network());
    let b_end = u32::from(b.broadcast());

    let overlap_start = a_start.max(b_start);
    let overlap_end = a_end.min(b_end);

    if overlap_start <= overlap_end {
        (overlap_end - overlap_start + 1) as u64
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use ipnet::Ipv4Net;

    // ========== parse_ipv4_nets tests ==========

    #[test]
    fn parse_ipv4_nets_parses_valid_cidrs() {
        let input = "10.0.0.0/24\n192.168.1.0/24\n172.16.0.0/16";
        let reader = Cursor::new(input);
        let result = parse_ipv4_nets(reader).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "10.0.0.0/24".parse::<Ipv4Net>().unwrap());
        assert_eq!(result[1], "192.168.1.0/24".parse::<Ipv4Net>().unwrap());
        assert_eq!(result[2], "172.16.0.0/16".parse::<Ipv4Net>().unwrap());
    }

    #[test]
    fn parse_ipv4_nets_ignores_empty_lines() {
        let input = "10.0.0.0/24\n\n192.168.1.0/24\n  \n\t\n172.16.0.0/16";
        let reader = Cursor::new(input);
        let result = parse_ipv4_nets(reader).unwrap();

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn parse_ipv4_nets_trims_whitespace() {
        let input = "  10.0.0.0/24  \n\t192.168.1.0/24\t";
        let reader = Cursor::new(input);
        let result = parse_ipv4_nets(reader).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "10.0.0.0/24".parse::<Ipv4Net>().unwrap());
        assert_eq!(result[1], "192.168.1.0/24".parse::<Ipv4Net>().unwrap());
    }

    #[test]
    fn parse_ipv4_nets_returns_error_for_invalid_cidr() {
        let input = "10.0.0.0/24\ninvalid\n192.168.1.0/24";
        let reader = Cursor::new(input);
        let result = parse_ipv4_nets(reader);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Line 2"));
    }

    #[test]
    fn parse_ipv4_nets_handles_empty_input() {
        let input = "";
        let reader = Cursor::new(input);
        let result = parse_ipv4_nets(reader).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn parse_ipv4_nets_handles_only_empty_lines() {
        let input = "\n\n  \n\t\n";
        let reader = Cursor::new(input);
        let result = parse_ipv4_nets(reader).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn parse_ipv4_nets_handles_malformed_ip() {
        let input = "10.0.0.0/24\n999.999.999.999/24\n192.168.1.0/24";
        let reader = Cursor::new(input);
        let result = parse_ipv4_nets(reader);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Line 2"));
    }

    #[test]
    fn parse_ipv4_nets_handles_invalid_prefix_length() {
        let input = "10.0.0.0/24\n192.168.1.0/33\n172.16.0.0/16";
        let reader = Cursor::new(input);
        let result = parse_ipv4_nets(reader);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Line 2"));
    }

    // ========== merge_ipv4_nets tests ==========

    #[test]
    fn merges_adjacent_subnets() {
        let nets = vec![
            "10.10.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.10.1.0/24".parse::<Ipv4Net>().unwrap(),
        ];

        let merged = merge_ipv4_nets(nets, 0);

        assert_eq!(merged, vec!["10.10.0.0/23".parse::<Ipv4Net>().unwrap()]);
    }

    #[test]
    fn retains_non_mergeable_ranges() {
        let nets = vec![
            "192.168.1.0/24".parse::<Ipv4Net>().unwrap(),
            "192.168.3.0/24".parse::<Ipv4Net>().unwrap(),
            "192.168.4.0/24".parse::<Ipv4Net>().unwrap(),
        ];

        let merged = merge_ipv4_nets(nets, 0);

        assert_eq!(
            merged,
            vec![
                "192.168.1.0/24".parse::<Ipv4Net>().unwrap(),
                "192.168.3.0/24".parse::<Ipv4Net>().unwrap(),
                "192.168.4.0/24".parse::<Ipv4Net>().unwrap(),
            ]
        );
    }

    #[test]
    fn deduplicates_and_merges_iteratively() {
        let nets = vec![
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.2.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.3.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
        ];

        let merged = merge_ipv4_nets(nets, 0);

        assert_eq!(merged, vec!["10.0.0.0/22".parse::<Ipv4Net>().unwrap()]);
    }

    #[test]
    fn removes_covered_subnets() {
        let nets = vec![
            "10.0.0.0/23".parse::<Ipv4Net>().unwrap(),
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(),
        ];

        let merged = merge_ipv4_nets(nets, 0);

        assert_eq!(merged, vec!["10.0.0.0/23".parse::<Ipv4Net>().unwrap()]);
    }

    #[test]
    fn merges_largest_adjacent_prefixes() {
        let nets = vec![
            "0.0.0.0/1".parse::<Ipv4Net>().unwrap(),
            "128.0.0.0/1".parse::<Ipv4Net>().unwrap(),
        ];

        let merged = merge_ipv4_nets(nets, 0);

        assert_eq!(merged, vec!["0.0.0.0/0".parse::<Ipv4Net>().unwrap()]);
    }

    #[test]
    fn tolerance_allows_non_adjacent_merge() {
        // Two /24 networks separated by one /24 gap
        // Without tolerance: cannot merge
        // With tolerance >= 256: can merge into /22 (covers 4 /24s, adds 2 /24s = 512 addresses)
        let nets = vec![
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.2.0/24".parse::<Ipv4Net>().unwrap(),
        ];

        // Without tolerance, should not merge
        let merged_no_tol = merge_ipv4_nets(nets.clone(), 0);
        assert_eq!(merged_no_tol.len(), 2);

        // With tolerance >= 512, should merge
        let merged_with_tol = merge_ipv4_nets(nets, 512);
        assert_eq!(merged_with_tol.len(), 1);
        assert_eq!(merged_with_tol[0].prefix_len(), 22);
    }

    #[test]
    fn tolerance_rejects_merge_exceeding_budget() {
        // Two /24 networks separated by one /24 gap
        // Merging into /22 adds 512 addresses
        let nets = vec![
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.2.0/24".parse::<Ipv4Net>().unwrap(),
        ];

        // With tolerance < 512, should not merge
        let merged = merge_ipv4_nets(nets, 256);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn tolerance_respects_exact_merge_preference() {
        // Adjacent networks should merge exactly (0 extra addresses) even with tolerance
        let nets = vec![
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(),
        ];

        let merged = merge_ipv4_nets(nets, 1000);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].prefix_len(), 23);
    }

    #[test]
    fn tolerance_handles_overlapping_networks() {
        // Networks that overlap should account for overlap correctly
        let nets = vec![
            "10.0.0.0/23".parse::<Ipv4Net>().unwrap(), // 10.0.0.0 - 10.0.1.255
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(), // 10.0.1.0 - 10.0.1.255 (overlaps)
        ];

        // Should remove covered subnet regardless of tolerance
        let merged = merge_ipv4_nets(nets, 0);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].prefix_len(), 23);
    }

    #[test]
    fn tolerance_accumulates_across_iterations() {
        // Test that tolerance is applied per merge, not globally
        // 10.0.0.0/24, 10.0.2.0/24, 10.0.4.0/24
        // First iteration: might merge 0-2 into /22 (adds 512), then 2-4 into /22 (adds 512)
        // With tolerance 512, only one merge should happen
        let nets = vec![
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.2.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.4.0/24".parse::<Ipv4Net>().unwrap(),
        ];

        // With tolerance 512, should merge one pair
        let merged = merge_ipv4_nets(nets, 512);
        // Should have 2 networks (one merged pair + one remaining)
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn merge_ipv4_nets_handles_empty_input() {
        let nets = vec![];
        let merged = merge_ipv4_nets(nets, 0);
        assert_eq!(merged.len(), 0);
    }

    #[test]
    fn merge_ipv4_nets_handles_single_network() {
        let nets = vec!["10.0.0.0/24".parse::<Ipv4Net>().unwrap()];
        let merged = merge_ipv4_nets(nets, 0);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], "10.0.0.0/24".parse::<Ipv4Net>().unwrap());
    }

    #[test]
    fn merge_ipv4_nets_handles_unsorted_input() {
        let nets = vec![
            "10.0.2.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(),
        ];
        let merged = merge_ipv4_nets(nets, 0);
        // 10.0.0.0/24 and 10.0.1.0/24 merge into 10.0.0.0/23
        // 10.0.2.0/24 remains separate (not adjacent to the /23)
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], "10.0.0.0/23".parse::<Ipv4Net>().unwrap());
        assert_eq!(merged[1], "10.0.2.0/24".parse::<Ipv4Net>().unwrap());
    }

    #[test]
    fn merge_ipv4_nets_handles_multiple_adjacent_groups() {
        let nets = vec![
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.4.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.5.0/24".parse::<Ipv4Net>().unwrap(),
        ];
        let merged = merge_ipv4_nets(nets, 0);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], "10.0.0.0/23".parse::<Ipv4Net>().unwrap());
        assert_eq!(merged[1], "10.0.4.0/23".parse::<Ipv4Net>().unwrap());
    }

    #[test]
    fn merge_ipv4_nets_handles_nested_subnets() {
        let nets = vec![
            "10.0.0.0/16".parse::<Ipv4Net>().unwrap(),
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.2.0/24".parse::<Ipv4Net>().unwrap(),
        ];
        let merged = merge_ipv4_nets(nets, 0);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], "10.0.0.0/16".parse::<Ipv4Net>().unwrap());
    }

    #[test]
    fn merge_ipv4_nets_handles_complex_merging_scenario() {
        // Test multiple iterations: merge adjacent, then merge the results
        let nets = vec![
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.2.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.3.0/24".parse::<Ipv4Net>().unwrap(),
        ];
        let merged = merge_ipv4_nets(nets, 0);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], "10.0.0.0/22".parse::<Ipv4Net>().unwrap());
    }

    #[test]
    fn merge_ipv4_nets_preserves_order_after_sorting() {
        let nets = vec![
            "192.168.1.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "172.16.0.0/24".parse::<Ipv4Net>().unwrap(),
        ];
        let merged = merge_ipv4_nets(nets, 0);
        assert_eq!(merged.len(), 3);
        // Should be sorted by network address
        assert!(u32::from(merged[0].addr()) < u32::from(merged[1].addr()));
        assert!(u32::from(merged[1].addr()) < u32::from(merged[2].addr()));
    }

    #[test]
    fn merge_ipv4_nets_handles_tolerance_edge_cases() {
        // Test tolerance = 0 (exact merge only)
        let nets = vec![
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.2.0/24".parse::<Ipv4Net>().unwrap(),
        ];
        let merged = merge_ipv4_nets(nets.clone(), 0);
        assert_eq!(merged.len(), 2);

        // Test tolerance = 511 (just below threshold)
        let merged = merge_ipv4_nets(nets.clone(), 511);
        assert_eq!(merged.len(), 2);

        // Test tolerance = 512 (at threshold)
        let merged = merge_ipv4_nets(nets.clone(), 512);
        assert_eq!(merged.len(), 1);

        // Test tolerance = u64::MAX (very large)
        let merged = merge_ipv4_nets(nets, u64::MAX);
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn merge_ipv4_nets_handles_very_small_networks() {
        let nets = vec![
            "10.0.0.0/32".parse::<Ipv4Net>().unwrap(),
            "10.0.0.1/32".parse::<Ipv4Net>().unwrap(),
        ];
        let merged = merge_ipv4_nets(nets, 0);
        // Two adjacent /32s can merge into a /31 which covers exactly 2 addresses
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], "10.0.0.0/31".parse::<Ipv4Net>().unwrap());
    }

    #[test]
    fn merge_ipv4_nets_handles_very_large_networks() {
        let nets = vec![
            "0.0.0.0/1".parse::<Ipv4Net>().unwrap(),
            "128.0.0.0/1".parse::<Ipv4Net>().unwrap(),
        ];
        let merged = merge_ipv4_nets(nets, 0);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], "0.0.0.0/0".parse::<Ipv4Net>().unwrap());
    }

    // ========== Helper function tests (using internal visibility) ==========

    #[test]
    fn test_try_merge_exact_adjacent_same_prefix() {
        let a = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        let b = "10.0.1.0/24".parse::<Ipv4Net>().unwrap();
        let result = try_merge_exact(&a, &b);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "10.0.0.0/23".parse::<Ipv4Net>().unwrap());
    }

    #[test]
    fn test_try_merge_exact_different_prefix_lengths() {
        let a = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        let b = "10.0.1.0/23".parse::<Ipv4Net>().unwrap();
        let result = try_merge_exact(&a, &b);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_merge_exact_non_adjacent() {
        let a = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        let b = "10.0.2.0/24".parse::<Ipv4Net>().unwrap();
        let result = try_merge_exact(&a, &b);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_merge_exact_prefix_zero() {
        let a = "10.0.0.0/0".parse::<Ipv4Net>().unwrap();
        let b = "10.0.1.0/0".parse::<Ipv4Net>().unwrap();
        let result = try_merge_exact(&a, &b);
        assert!(result.is_none());
    }

    #[test]
    fn test_network_covers_supernet_covers_subnet() {
        let supernet = "10.0.0.0/16".parse::<Ipv4Net>().unwrap();
        let subnet = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        assert!(network_covers(&supernet, &subnet));
    }

    #[test]
    fn test_network_covers_same_network() {
        let net = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        assert!(network_covers(&net, &net));
    }

    #[test]
    fn test_network_covers_subnet_does_not_cover_supernet() {
        let subnet = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        let supernet = "10.0.0.0/16".parse::<Ipv4Net>().unwrap();
        assert!(!network_covers(&subnet, &supernet));
    }

    #[test]
    fn test_network_covers_disjoint_networks() {
        let a = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        let b = "10.0.2.0/24".parse::<Ipv4Net>().unwrap();
        assert!(!network_covers(&a, &b));
        assert!(!network_covers(&b, &a));
    }

    #[test]
    fn test_network_covers_partial_overlap() {
        let a = "10.0.0.0/23".parse::<Ipv4Net>().unwrap();
        let b = "10.0.1.0/24".parse::<Ipv4Net>().unwrap();
        assert!(network_covers(&a, &b));
        assert!(!network_covers(&b, &a));
    }

    #[test]
    fn test_find_covering_supernet_adjacent_networks() {
        let a = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        let b = "10.0.2.0/24".parse::<Ipv4Net>().unwrap();
        let result = find_covering_supernet(&a, &b);
        assert!(result.is_some());
        // Should find /22 that covers both
        assert!(result.unwrap().prefix_len() <= 22);
    }

    #[test]
    fn test_find_covering_supernet_overlapping_networks() {
        let a = "10.0.0.0/23".parse::<Ipv4Net>().unwrap();
        let b = "10.0.1.0/24".parse::<Ipv4Net>().unwrap();
        let result = find_covering_supernet(&a, &b);
        assert!(result.is_some());
        // Should find /23 that covers both
        assert_eq!(result.unwrap().prefix_len(), 23);
    }

    #[test]
    fn test_find_covering_supernet_disjoint_networks() {
        let a = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        let b = "192.168.0.0/24".parse::<Ipv4Net>().unwrap();
        let result = find_covering_supernet(&a, &b);
        assert!(result.is_some());
        // Should find a very large supernet (likely /0)
        assert_eq!(result.unwrap().prefix_len(), 0);
    }

    #[test]
    fn test_network_address_count() {
        assert_eq!(network_address_count(&"10.0.0.0/32".parse::<Ipv4Net>().unwrap()), 1);
        assert_eq!(network_address_count(&"10.0.0.0/31".parse::<Ipv4Net>().unwrap()), 2);
        assert_eq!(network_address_count(&"10.0.0.0/24".parse::<Ipv4Net>().unwrap()), 256);
        assert_eq!(network_address_count(&"10.0.0.0/16".parse::<Ipv4Net>().unwrap()), 65536);
        assert_eq!(network_address_count(&"10.0.0.0/8".parse::<Ipv4Net>().unwrap()), 16777216);
        assert_eq!(network_address_count(&"0.0.0.0/0".parse::<Ipv4Net>().unwrap()), 4294967296);
    }

    #[test]
    fn test_network_overlap_no_overlap() {
        let a = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        let b = "10.0.2.0/24".parse::<Ipv4Net>().unwrap();
        assert_eq!(network_overlap(&a, &b), 0);
    }

    #[test]
    fn test_network_overlap_full_overlap() {
        let a = "10.0.0.0/16".parse::<Ipv4Net>().unwrap();
        let b = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        assert_eq!(network_overlap(&a, &b), 256);
    }

    #[test]
    fn test_network_overlap_partial_overlap() {
        let a = "10.0.0.0/23".parse::<Ipv4Net>().unwrap();
        let b = "10.0.1.0/24".parse::<Ipv4Net>().unwrap();
        assert_eq!(network_overlap(&a, &b), 256);
    }

    #[test]
    fn test_network_overlap_adjacent_no_overlap() {
        let a = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        let b = "10.0.1.0/24".parse::<Ipv4Net>().unwrap();
        assert_eq!(network_overlap(&a, &b), 0);
    }

    #[test]
    fn test_network_overlap_same_network() {
        let a = "10.0.0.0/24".parse::<Ipv4Net>().unwrap();
        assert_eq!(network_overlap(&a, &a), 256);
    }

    #[test]
    fn test_remove_covered_nets_empty() {
        let (result, changed) = remove_covered_nets(vec![]);
        assert_eq!(result.len(), 0);
        assert!(!changed);
    }

    #[test]
    fn test_remove_covered_nets_no_covered() {
        let nets = vec![
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.2.0/24".parse::<Ipv4Net>().unwrap(),
        ];
        let (result, changed) = remove_covered_nets(nets);
        assert_eq!(result.len(), 2);
        assert!(!changed);
    }

    #[test]
    fn test_remove_covered_nets_removes_covered() {
        let nets = vec![
            "10.0.0.0/16".parse::<Ipv4Net>().unwrap(),
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(),
        ];
        let (result, changed) = remove_covered_nets(nets);
        assert_eq!(result.len(), 1);
        assert!(changed);
        assert_eq!(result[0], "10.0.0.0/16".parse::<Ipv4Net>().unwrap());
    }

    #[test]
    fn test_sort_and_dedup_removes_duplicates() {
        let mut nets = vec![
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "192.168.1.0/24".parse::<Ipv4Net>().unwrap(),
        ];
        sort_and_dedup(&mut nets);
        assert_eq!(nets.len(), 2);
    }

    #[test]
    fn test_sort_and_dedup_sorts_by_address_then_prefix() {
        let mut nets = vec![
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.0.0/16".parse::<Ipv4Net>().unwrap(),
        ];
        sort_and_dedup(&mut nets);
        assert_eq!(nets[0], "10.0.0.0/16".parse::<Ipv4Net>().unwrap());
        assert_eq!(nets[1], "10.0.0.0/24".parse::<Ipv4Net>().unwrap());
        assert_eq!(nets[2], "10.0.1.0/24".parse::<Ipv4Net>().unwrap());
    }
}
