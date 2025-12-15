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

fn sort_and_dedup(nets: &mut Vec<Ipv4Net>) {
    nets.sort_by(|a, b| {
        u32::from(a.addr())
            .cmp(&u32::from(b.addr()))
            .then(a.prefix_len().cmp(&b.prefix_len()))
    });
    nets.dedup();
}

fn remove_covered_nets(nets: Vec<Ipv4Net>) -> (Vec<Ipv4Net>, bool) {
    if nets.is_empty() {
        return (nets, false);
    }

    let mut compacted = Vec::with_capacity(nets.len());
    compacted.push(nets[0]);

    for net in nets.into_iter().skip(1) {
        if let Some(last) = compacted.last()
            && network_covers(last, &net)
        {
            continue;
        }

        compacted.push(net);
    }

    let removed_any = compacted.len() < compacted.capacity();
    (compacted, removed_any)
}

fn network_covers(supernet: &Ipv4Net, subnet: &Ipv4Net) -> bool {
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
fn try_merge_exact(a: &Ipv4Net, b: &Ipv4Net) -> Option<Ipv4Net> {
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
fn find_covering_supernet(a: &Ipv4Net, b: &Ipv4Net) -> Option<Ipv4Net> {
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
fn network_address_count(net: &Ipv4Net) -> u64 {
    1u64 << (32 - net.prefix_len())
}

/// Calculates the number of overlapping addresses between two networks.
fn network_overlap(a: &Ipv4Net, b: &Ipv4Net) -> u64 {
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
    use super::merge_ipv4_nets;
    use ipnet::Ipv4Net;

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
}
