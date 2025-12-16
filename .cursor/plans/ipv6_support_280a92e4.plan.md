---
name: IPv6 Support
overview: Extend functionality to support IPv6 CIDRs with separate flag or auto-detection
todos:
  - id: add-ip-version-flags
    content: Add --ipv4/--ipv6 flags or --version enum to Args struct
    status: pending
  - id: create-ip-version-detection
    content: Create IP version detection function
    status: pending
  - id: create-generic-parser
    content: Create parse_ip_nets() using IpNet enum
    status: pending
  - id: create-generic-merger
    content: Create merge_ip_nets() that handles both versions separately
    status: pending
  - id: adapt-helpers-ipv6
    content: Adapt helper functions for IPv6 (use u128, handle 128-bit addresses)
    status: pending
  - id: update-main-for-ipnet
    content: Update main() to use generic functions
    status: pending
  - id: add-ipv6-tests
    content: Add comprehensive IPv6 unit and integration tests
    status: pending
---

# IPv6 Support Implementation Plan

## Overview

Add support for IPv6 CIDR blocks alongside IPv4, with `--ipv6` flag or auto-detection from input.

## Changes Required

### 1. Dependencies (`Cargo.toml`)

- `ipnet` crate already supports IPv6 via `Ipv6Net` type
- No additional dependencies needed

### 2. CLI Arguments (`src/main.rs`)

- Add `--ipv6` flag (boolean) to force IPv6 mode
- Add `--ipv4` flag (boolean) to force IPv4 mode
- Auto-detect IP version from first valid CIDR if neither flag specified
- Consider `--version` flag with values: `auto`, `ipv4`, `ipv6`

### 3. Generic Parsing (`src/lib.rs`)

- Create generic `parse_ip_nets<R: BufRead>(reader: R) -> Result<Vec<IpNet>, String>`
- Use `ipnet::IpNet` enum (covers both IPv4 and IPv6)
- Detect IP version from first valid CIDR
- Return error if mixed versions detected (unless tolerance specified)

### 4. Generic Merging (`src/lib.rs`)

- Create `merge_ip_nets(nets: Vec<IpNet>, tolerance: u64) -> Vec<IpNet>`
- Handle IPv4 and IPv6 separately (cannot merge across versions)
- Split input into IPv4 and IPv6 vectors
- Process each separately, then combine results
- Maintain sorting: IPv4 first, then IPv6

### 5. IPv6-Specific Logic (`src/lib.rs`)

- Adapt `try_merge_exact()` for IPv6 (128-bit addresses, prefix lengths 0-128)
- Adapt `find_covering_supernet()` for IPv6
- Adapt `network_address_count()` for IPv6: `2^(128 - prefix_len)`
- Use `u128` for IPv6 address calculations instead of `u32`

### 6. Tolerance Handling (`src/lib.rs`)

- IPv6 tolerance uses same semantics (extra addresses)
- Consider much larger address spaces (2^128 vs 2^32)
- May need `u128` for tolerance calculations in extreme cases

### 7. Output Formatting (`src/main.rs`)

- Output format functions handle `IpNet` enum
- Display IPv4 and IPv6 networks appropriately

### 8. Testing

- Add unit tests for IPv6 parsing in `src/lib.rs`
- Add unit tests for IPv6 merging (adjacent /64s, etc.)
- Add integration tests for `--ipv6` flag
- Test auto-detection behavior
- Test error handling for mixed versions

## Implementation Details

### IP Version Detection

```rust
fn detect_ip_version(cidr: &str) -> Option<IpVersion> {
    if cidr.parse::<Ipv4Net>().is_ok() {
        Some(IpVersion::V4)
    } else if cidr.parse::<Ipv6Net>().is_ok() {
        Some(IpVersion::V6)
    } else {
        None
    }
}
```

### Separate Processing

- Process IPv4 and IPv6 networks separately
- Cannot merge across IP versions
- Output maintains separation (IPv4 first, then IPv6)

## Files to Modify

- `src/main.rs`: Add IP version flags, handle IpNet enum
- `src/lib.rs`: Add generic parsing/merging functions, IPv6-specific helpers
- `tests/integration_test.rs`: Add IPv6 tests