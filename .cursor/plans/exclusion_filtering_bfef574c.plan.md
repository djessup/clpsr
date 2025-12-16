---
name: Exclusion Filtering
overview: Add ability to exclude specific networks from merging operations
todos:
  - id: add-exclude-flag
    content: Add --exclude flag with multiple values support to Args struct
    status: pending
  - id: create-filter-excluded
    content: Create filter_excluded() function in src/lib.rs
    status: pending
  - id: update-merge-signature
    content: Update merge_ipv4_nets() to accept exclusions parameter
    status: pending
  - id: integrate-filtering
    content: Integrate exclusion filtering into merge workflow
    status: pending
  - id: add-exclusion-tests
    content: Add unit and integration tests for exclusion functionality
    status: pending
---

# Exclusion/Filtering Implementation Plan

## Overview

Add `--exclude` flag to exclude specific networks from merging, useful when certain ranges must remain separate.

## Changes Required

### 1. CLI Arguments (`src/main.rs`)

- Add `--exclude` flag accepting multiple values: `--exclude 10.0.0.0/24 --exclude 192.168.1.0/24`
- Use `clap::ArgAction::Append` to allow multiple exclusions
- Parse exclusions as `Vec<Ipv4Net>` (or `Vec<IpNet>` if IPv6 support exists)

### 2. Exclusion Logic (`src/lib.rs`)

- Create `filter_excluded(nets: Vec<Ipv4Net>, exclusions: &[Ipv4Net]) -> Vec<Ipv4Net>`
- Remove networks that are covered by any exclusion network
- Use existing `network_covers()` helper to check coverage
- Return filtered vector

### 3. Merging Integration (`src/lib.rs`)

- Modify `merge_ipv4_nets()` to accept optional exclusions parameter
- Before merging, filter out excluded networks
- After merging, ensure no merged network covers an excluded range
- If a merge would create a supernet covering an exclusion, reject that merge

### 4. Exclusion Validation (`src/lib.rs`)

- Validate exclusion networks are valid CIDRs
- Consider: should exclusions themselves be mergeable? (Probably not - they're exclusion rules)
- Handle case where exclusion covers multiple input networks

### 5. Pre-merge Filtering Strategy

- Option A: Remove excluded networks before merging (simpler)
- Option B: Prevent merges that would cover exclusions (more complex, preserves non-excluded parts)
- Recommend Option A for initial implementation

### 6. Testing

- Add unit tests for `filter_excluded()` function
- Test exclusion of single network
- Test exclusion of supernet covering multiple networks
- Test exclusion doesn't affect non-excluded networks
- Add integration tests for `--exclude` flag

## Implementation Details

### Function Signature

```rust
pub fn merge_ipv4_nets_with_exclusions(
    nets: Vec<Ipv4Net>,
    tolerance: u64,
    exclusions: &[Ipv4Net],
) -> Vec<Ipv4Net>
```

### Filtering Logic

- Iterate through input networks
- Skip any network covered by an exclusion
- Process remaining networks through normal merge logic

## Files to Modify

- `src/main.rs`: Add `--exclude` flag, parse exclusions, pass to merge function
- `src/lib.rs`: Add `filter_excluded()` and update `merge_ipv4_nets()` signature
- `tests/integration_test.rs`: Add exclusion tests