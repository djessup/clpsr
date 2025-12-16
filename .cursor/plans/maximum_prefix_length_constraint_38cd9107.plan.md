---
name: Maximum Prefix Length Constraint
overview: Add --max-prefix flag to prevent merging beyond a specified prefix length
todos:
  - id: add-max-prefix-flag
    content: Add --max-prefix flag to Args struct with validation
    status: pending
  - id: create-constraint-check
    content: Create check_prefix_constraint() helper function
    status: pending
  - id: update-merge-functions
    content: Update try_merge functions to accept and check max_prefix constraint
    status: pending
  - id: update-merge-signature
    content: Update merge_ipv4_nets() to accept max_prefix parameter
    status: pending
  - id: add-constraint-tests
    content: Add unit and integration tests for prefix constraint
    status: pending
---

# Maximum Prefix Length Constraint Implementation Plan

## Overview

Add `--max-prefix` flag to prevent merging networks that would result in a prefix length smaller (larger network) than the specified maximum.

## Changes Required

### 1. CLI Arguments (`src/main.rs`)

- Add `--max-prefix` flag accepting u8 value (0-32 for IPv4, 0-128 for IPv6)
- Validate range based on IP version
- Default: no constraint (allow any prefix length)

### 2. Merge Constraint Logic (`src/lib.rs`)

- Modify `try_merge_exact()` and `try_merge_with_tolerance()` to accept optional `max_prefix: Option<u8>`
- Before accepting a merge, check if resulting supernet prefix length >= max_prefix
- If merge would violate constraint, reject the merge
- Example: `--max-prefix 22` prevents merging into /21 or larger networks

### 3. Constraint Checking (`src/lib.rs`)

- Create helper function `check_prefix_constraint(net: &Ipv4Net, max_prefix: Option<u8>) -> bool`
- Returns `true` if network prefix length >= max_prefix (or no constraint)
- Returns `false` if network violates constraint

### 4. Merge Function Updates (`src/lib.rs`)

- Update `merge_ipv4_nets()` signature to accept `max_prefix: Option<u8>`
- Pass constraint to `try_merge_with_tolerance()` calls
- Reject merges that would violate constraint

### 5. Constraint Semantics

- `--max-prefix 22` means: resulting networks must have prefix length >= 22
- This prevents creating networks larger than /22
- Useful for policy compliance (e.g., "no networks larger than /22")

### 6. Integration with Tolerance

- Constraint applies independently of tolerance
- Even if tolerance allows merge, constraint can reject it
- Check constraint after checking tolerance

### 7. Testing

- Add unit tests for prefix constraint checking
- Test constraint prevents merges that would violate it
- Test constraint allows merges that satisfy it
- Test with various prefix values
- Add integration tests for `--max-prefix` flag

## Implementation Details

### Constraint Check

```rust
fn check_prefix_constraint(net: &Ipv4Net, max_prefix: Option<u8>) -> bool {
    match max_prefix {
        Some(max) => net.prefix_len() >= max,
        None => true, // No constraint
    }
}
```

### Merge Rejection

- In `try_merge_with_tolerance()`, after finding supernet, check constraint
- If constraint violated, return `None` (merge rejected)

## Files to Modify

- `src/main.rs`: Add `--max-prefix` flag, pass to merge function
- `src/lib.rs`: Add constraint checking, update merge functions
- `tests/integration_test.rs`: Add max-prefix tests