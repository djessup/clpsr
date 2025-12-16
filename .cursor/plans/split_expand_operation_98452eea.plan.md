---
name: Split Expand Operation
overview: Add split subcommand to expand networks into subnets (reverse of merge operation)
todos:
  - id: add-split-subcommand
    content: Add split subcommand to CLI with network and --prefix arguments
    status: pending
  - id: implement-split-network
    content: Implement split_network() function in src/lib.rs
    status: pending
  - id: implement-batch-split
    content: Implement split_networks() for multiple networks
    status: pending
  - id: add-split-validation
    content: Add validation for target prefix and network constraints
    status: pending
  - id: add-split-tests
    content: Add unit and integration tests for split functionality
    status: pending
---

# Split/Expand Operation Implementation Plan

## Overview

Add `clpsr split` subcommand to expand networks into smaller subnets, reverse of merge operation.

## Changes Required

### 1. CLI Structure (`src/main.rs`)

- Add `Split` subcommand to existing subcommand structure (or create if diff feature added it)
- Arguments:
  - Required: network to split (CIDR string or from input)
  - Required: `--prefix` target prefix length for subnets
  - Optional: `--input` file containing networks to split
- Structure: `clpsr split <network> --prefix <length>` or `clpsr split --input <file> --prefix <length>`

### 2. Split Logic (`src/lib.rs`)

- Create `split_network(net: &Ipv4Net, target_prefix: u8) -> Vec<Ipv4Net>`
- Validate: target_prefix must be > current prefix length
- Calculate number of subnets: `2^(target_prefix - current_prefix)`
- Generate all subnets by iterating through address space
- Return vector of subnets sorted by network address

### 3. Split Algorithm (`src/lib.rs`)

- For network `10.0.0.0/22` split to `/24`:
  - Current prefix: 22, target: 24
  - Difference: 24 - 22 = 2 bits
  - Number of subnets: 2^2 = 4
  - Generate: `10.0.0.0/24`, `10.0.1.0/24`, `10.0.2.0/24`, `10.0.3.0/24`

### 4. Batch Splitting (`src/lib.rs`)

- Create `split_networks(nets: Vec<Ipv4Net>, target_prefix: u8) -> Vec<Ipv4Net>`
- Split each network in input vector
- Combine all results, sort and deduplicate
- Handle networks already at or below target prefix (return as-is or skip)

### 5. Input Handling (`src/main.rs`)

- Support splitting single network from command line: `clpsr split 10.0.0.0/22 --prefix 24`
- Support splitting networks from file: `clpsr split --input file.txt --prefix 24`
- Support stdin: `clpsr split --prefix 24 < file.txt`

### 6. Validation (`src/lib.rs`)

- Validate target prefix is valid (0-32 for IPv4)
- Validate target prefix > current prefix (can't split to larger network)
- Return error if validation fails

### 7. Output Formatting (`src/main.rs`)

- Output split subnets one per line (same as merge output)
- Consider `--format` flag compatibility if format feature exists
- Maintain consistent output format with merge operation

### 8. Testing

- Add unit tests for `split_network()` function
- Test splitting /22 to /24 (4 subnets)
- Test splitting /24 to /26 (4 subnets)
- Test edge cases: /32 (can't split), /0 (very large split)
- Test batch splitting multiple networks
- Add integration tests for `split` subcommand

## Implementation Details

### Split Function

```rust
pub fn split_network(net: &Ipv4Net, target_prefix: u8) -> Result<Vec<Ipv4Net>, String> {
    if target_prefix <= net.prefix_len() {
        return Err(format!("Target prefix {} must be greater than current prefix {}", 
            target_prefix, net.prefix_len()));
    }
    
    let prefix_diff = target_prefix - net.prefix_len();
    let subnet_count = 1u64 << prefix_diff;
    let subnet_size = 1u64 << (32 - target_prefix);
    
    let mut subnets = Vec::new();
    let base_addr = u32::from(net.network());
    
    for i in 0..subnet_count {
        let subnet_addr = base_addr + (i * subnet_size) as u32;
        let subnet = Ipv4Net::new(Ipv4Addr::from(subnet_addr), target_prefix)?;
        subnets.push(subnet);
    }
    
    Ok(subnets)
}
```

## Files to Modify

- `src/main.rs`: Add split subcommand, handle split arguments
- `src/lib.rs`: Add `split_network()` and `split_networks()` functions
- `tests/integration_test.rs`: Add split subcommand tests