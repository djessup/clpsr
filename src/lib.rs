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
/// their combined supernet exactly represents the two subnets.
pub fn merge_ipv4_nets(nets: Vec<Ipv4Net>) -> Vec<Ipv4Net> {
    let mut normalized = nets;
    sort_and_dedup(&mut normalized);

    let mut changed = true;
    while changed {
        changed = false;
        let mut merged: Vec<Ipv4Net> = Vec::new();
        let mut idx = 0;

        while idx < normalized.len() {
            if idx + 1 < normalized.len() {
                if let Some(supernet) = try_merge(&normalized[idx], &normalized[idx + 1]) {
                    merged.push(supernet);
                    changed = true;
                    idx += 2;
                    continue;
                }
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
        if let Some(last) = compacted.last() {
            if network_covers(last, &net) {
                continue;
            }
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

fn try_merge(a: &Ipv4Net, b: &Ipv4Net) -> Option<Ipv4Net> {
    if a.prefix_len() != b.prefix_len() || a.prefix_len() == 0 {
        return None;
    }

    let prefix = a.prefix_len();
    let block_size = 1u32 << (32 - prefix);
    let a_net = u32::from(a.addr());
    let b_net = u32::from(b.addr());

    if a_net % (block_size * 2) != 0 {
        return None;
    }

    if a_net + block_size != b_net {
        return None;
    }

    Ipv4Net::new(a.addr(), prefix - 1).ok()
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

        let merged = merge_ipv4_nets(nets);

        assert_eq!(merged, vec!["10.10.0.0/23".parse::<Ipv4Net>().unwrap()]);
    }

    #[test]
    fn retains_non_mergeable_ranges() {
        let nets = vec![
            "192.168.1.0/24".parse::<Ipv4Net>().unwrap(),
            "192.168.3.0/24".parse::<Ipv4Net>().unwrap(),
            "192.168.4.0/24".parse::<Ipv4Net>().unwrap(),
        ];

        let merged = merge_ipv4_nets(nets);

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

        let merged = merge_ipv4_nets(nets);

        assert_eq!(merged, vec!["10.0.0.0/22".parse::<Ipv4Net>().unwrap()]);
    }

    #[test]
    fn removes_covered_subnets() {
        let nets = vec![
            "10.0.0.0/23".parse::<Ipv4Net>().unwrap(),
            "10.0.0.0/24".parse::<Ipv4Net>().unwrap(),
            "10.0.1.0/24".parse::<Ipv4Net>().unwrap(),
        ];

        let merged = merge_ipv4_nets(nets);

        assert_eq!(merged, vec!["10.0.0.0/23".parse::<Ipv4Net>().unwrap()]);
    }
}
