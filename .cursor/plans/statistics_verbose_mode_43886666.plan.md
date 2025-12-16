---
name: Statistics Verbose Mode
overview: Add statistics/verbose mode to display before/after counts, reduction percentage, and total addresses covered
todos:
  - id: add-cli-flag
    content: Add --stats flag to Args struct in src/main.rs
    status: pending
  - id: create-statistics-struct
    content: Create Statistics struct in src/lib.rs
    status: pending
  - id: implement-calculate-statistics
    content: Implement calculate_statistics() function in src/lib.rs
    status: pending
  - id: add-statistics-output
    content: Add statistics output formatting in src/main.rs main() function
    status: pending
  - id: add-unit-tests
    content: Add unit tests for calculate_statistics() in src/lib.rs
    status: pending
  - id: add-integration-test
    content: Add integration test for --stats flag in tests/integration_test.rs
    status: pending
---

# Statistics/Verbose Mode Implementation Plan

## Overview

Add `--stats` or `--verbose` flag to display merge statistics including before/after counts, reduction percentage, and total addresses covered.

## Changes Required

### 1. CLI Arguments (`src/main.rs`)

- Add `--stats` or `--verbose` flag to `Args` struct
- Use `clap::ArgAction::SetTrue` for boolean flag
- Update help text to describe statistics output

### 2. Statistics Calculation (`src/lib.rs`)

- Create new public function `calculate_statistics(nets: &[Ipv4Net], merged: &[Ipv4Net]) -> Statistics`
- Return struct containing:
  - `input_count: usize`
  - `output_count: usize`
  - `reduction_percentage: f64`
  - `total_addresses_input: u64`
  - `total_addresses_output: u64`
- Use existing `network_address_count()` helper for address calculations
- Calculate reduction: `((input_count - output_count) as f64 / input_count as f64) * 100.0`

### 3. Output Formatting (`src/main.rs`)

- When `--stats` flag is set, print statistics before or after CIDR output
- Format statistics in human-readable format:
  ```
  Input:  N networks, M addresses
  Output: K networks, L addresses
  Reduction: X% (Y networks removed)
  ```

- Consider printing to stderr to keep stdout clean for piping

### 4. Testing

- Add unit tests for `calculate_statistics()` in `src/lib.rs` mod tests
- Test edge cases: empty input, single network, no reduction
- Add integration test in `tests/integration_test.rs` for CLI flag

## Implementation Details

### Statistics Struct

```rust
pub struct Statistics {
    pub input_count: usize,
    pub output_count: usize,
    pub reduction_percentage: f64,
    pub total_addresses_input: u64,
    pub total_addresses_output: u64,
}
```

### Integration Point

- Call `calculate_statistics()` after `merge_ipv4_nets()` in `main()`
- Conditionally print based on `args.stats` flag
- Maintain backward compatibility (no flag = no statistics)

## Files to Modify

- `src/main.rs`: Add CLI flag, call statistics function, format output
- `src/lib.rs`: Add `calculate_statistics()` function and `Statistics` struct
- `tests/integration_test.rs`: Add test for `--stats` flag