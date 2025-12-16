---
name: Validation Check Mode
overview: Add --check mode to verify if input is already optimal without making changes
todos:
  - id: add-check-flag
    content: Add --check boolean flag to Args struct
    status: pending
  - id: create-optimization-check
    content: Create is_optimally_merged() function in src/lib.rs
    status: pending
  - id: implement-exit-codes
    content: Implement exit code logic in main() for check mode
    status: pending
  - id: add-check-output
    content: Add status messages to stderr for check mode
    status: pending
  - id: add-check-tests
    content: Add unit and integration tests for check mode
    status: pending
---

# Validation/Check Mode Implementation Plan

## Overview

Add `--check` flag that verifies if input CIDRs are already optimally merged. Exit code 0 if optimal, 1 if merges are possible.

## Changes Required

### 1. CLI Arguments (`src/main.rs`)

- Add `--check` flag (boolean) to `Args` struct
- When `--check` is set, suppress normal output (or output to stderr only)
- Set exit code based on optimization status

### 2. Optimization Check (`src/lib.rs`)

- Create `is_optimally_merged(nets: &[Ipv4Net], tolerance: u64) -> bool`
- Function checks if any merges are possible without actually performing them
- Use existing merge logic but short-circuit on first possible merge
- Return `true` if no merges possible, `false` if merges exist

### 3. Efficient Check Implementation (`src/lib.rs`)

- Don't perform full merge - just detect if merge is possible
- Iterate through sorted networks checking adjacent pairs
- Use `try_merge_with_tolerance()` to check mergeability
- Early return on first mergeable pair found

### 4. Main Function Updates (`src/main.rs`)

- When `--check` flag is set:
  - Parse and merge networks as normal
  - Compare input count vs output count
  - If counts differ, exit with code 1 (not optimal)
  - If counts same, verify no merges possible, exit with code 0 (optimal)
- Output status message to stderr: "Input is optimal" or "Merges possible: N networks can be reduced to M"

### 5. Exit Code Semantics

- Exit code 0: Input is optimal (no merges possible)
- Exit code 1: Input is not optimal (merges possible)
- Exit code 2+: Error conditions (parsing errors, I/O errors)

### 6. Testing

- Add unit tests for `is_optimally_merged()` function
- Test with already-merged input (should return true)
- Test with mergeable input (should return false)
- Add integration tests for `--check` flag
- Test exit codes in integration tests

## Implementation Details

### Check Function

```rust
pub fn is_optimally_merged(nets: &[Ipv4Net], tolerance: u64) -> bool {
    lContinueet mut sorted = nets.to_vec();
    sort_and_dedup(&mut sorted);
    
    // Check for mergeable pairs
    for i in 0..sorted.len().saturating_sub(1) {
        if try_merge_with_tolerance(&sorted[i], &sorted[i + 1], tolerance).is_some() {
            return false;
        }
    }
    
    // Check for covered subnets
    let (compacted, changed) = remove_covered_nets(sorted);
    !changed && compacted.len() == nets.len()
}
```

## Files to Modify

- `src/main.rs`: Add `--check` flag, implement exit code logic
- `src/lib.rs`: Add `is_optimally_merged()` function
- `tests/integration_test.rs`: Add check mode tests with exit code verification