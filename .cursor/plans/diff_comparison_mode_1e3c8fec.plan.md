---
name: Diff Comparison Mode
overview: Add diff mode to compare two CIDR sets and show differences
todos:
  - id: refactor-to-subcommands
    content: Refactor CLI to use clap subcommands (merge, diff)
    status: pending
  - id: create-diff-result-struct
    content: Create DiffResult struct in src/lib.rs
    status: pending
  - id: implement-diff-logic
    content: Implement diff_cidr_sets() function
    status: pending
  - id: add-diff-subcommand
    content: Add diff subcommand with file arguments
    status: pending
  - id: add-diff-formatting
    content: Add diff output formatting in main()
    status: pending
  - id: add-diff-tests
    content: Add unit and integration tests for diff functionality
    status: pending
---

# Diff/Comparison Mode Implementation Plan

## Overview

Add `clpsr diff` subcommand to compare two CIDR sets and show added, removed, and changed networks.

## Changes Required

### 1. CLI Structure (`src/main.rs`)

- Convert to subcommand structure using `clap::Subcommand`
- Add `Diff` subcommand with two required positional arguments: `file1` and `file2`
- Structure: `clpsr diff <file1> <file2>`
- Consider optional flags: `--format`, `--unified` (like git diff)

### 2. Diff Logic (`src/lib.rs`)

- Create `diff_cidr_sets(a: &[Ipv4Net], b: &[Ipv4Net]) -> DiffResult`
- Compare two sets of networks
- Identify:
  - Networks only in set A (removed)
  - Networks only in set B (added)
  - Networks in both (unchanged)
- Use set operations: intersection, difference

### 3. Diff Result Structure (`src/lib.rs`)

- Create `DiffResult` struct:
  ```rust
  pub struct DiffResult {
      pub added: Vec<Ipv4Net>,
      pub removed: Vec<Ipv4Net>,
      pub unchanged: Vec<Ipv4Net>,
  }
  ```


### 4. Output Formatting (`src/main.rs`)

- Format diff output:
  - Option A: Simple list format
    ```
    Added:
    10.0.0.0/24
    
    Removed:
    192.168.1.0/24
    
    Unchanged:
    172.16.0.0/16
    ```

  - Option B: Unified diff format (like git diff)
  - Option C: JSON format (if format feature exists)

### 5. Normalization for Comparison (`src/lib.rs`)

- Both input sets should be normalized (sorted, deduplicated, merged)
- Use existing `merge_ipv4_nets()` to normalize each set
- Compare normalized sets for accurate diff

### 6. Subcommand Handling (`src/main.rs`)

- Refactor `main()` to handle subcommands
- Pattern:
  ```rust
  #[derive(Parser)]
  enum Cli {
      Merge(MergeArgs),
      Diff(DiffArgs),
  }
  ```


### 7. Testing

- Add unit tests for `diff_cidr_sets()` function
- Test with identical sets (no diff)
- Test with completely different sets
- Test with overlapping sets
- Add integration tests for `diff` subcommand

## Implementation Details

### Diff Algorithm

- Normalize both sets (sort, dedup, merge)
- Use set difference: `added = B - A`, `removed = A - B`
- Use set intersection: `unchanged = A ∩ B`
- Consider using `HashSet` for efficient lookups

### Backward Compatibility

- Default subcommand could be `merge` to maintain existing behavior
- Or: if no subcommand, assume merge operation

## Files to Modify

- `src/main.rs`: Add subcommand structure, implement diff command
- `src/lib.rs`: Add `diff_cidr_sets()` function and `DiffResult` struct
- `tests/integration_test.rs`: Add diff mode tests