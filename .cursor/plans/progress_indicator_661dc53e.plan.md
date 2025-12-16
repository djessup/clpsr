---
name: Progress Indicator
overview: Add progress indicator for large input processing to show processing status
todos:
  - id: add-progress-flag
    content: Add --progress boolean flag to Args struct
    status: pending
  - id: add-progress-reporting
    content: Add progress reporting hooks to parse_ipv4_nets() and merge_ipv4_nets()
    status: pending
  - id: implement-progress-display
    content: Implement progress display in main() using stderr
    status: pending
  - id: add-progress-threshold
    content: Add threshold logic to avoid excessive progress updates
    status: pending
  - id: add-progress-tests
    content: Add integration tests for progress indicator
    status: pending
---

# Progress Indicator Implementation Plan

## Overview

Add progress indicator to show processing status for large inputs, useful for very large files.

## Changes Required

### 1. CLI Arguments (`src/main.rs`)

- Add `--progress` flag (boolean) to enable progress display
- Default: disabled (for backward compatibility and piping)
- When enabled, output progress to stderr (keep stdout clean)

### 2. Progress Tracking (`src/lib.rs`)

- Create `ProgressReporter` trait or struct for progress callbacks
- Modify `parse_ipv4_nets()` to accept optional progress callback
- Modify `merge_ipv4_nets()` to accept optional progress callback
- Report progress at key stages: parsing, sorting, merging iterations

### 3. Progress Display (`src/main.rs`)

- Use simple text-based progress (no external dependencies)
- Format options:
  - Option A: Percentage: `Parsing: 50% (5000/10000 lines)`
  - Option B: Spinner: `Parsing... [=====>    ] 50%`
  - Option C: Simple counter: `Processing line 5000/10000...`
  - Recommend Option C for simplicity

### 4. Progress Updates (`src/lib.rs`)

- In `parse_ipv4_nets()`: report every N lines (e.g., every 1000 lines)
- In `merge_ipv4_nets()`: report after each merge iteration
- Use `eprintln!()` for progress output (stderr)

### 5. Input Size Estimation (`src/main.rs`)

- For file input: get file size to estimate progress
- For stdin: cannot estimate size, use line count only
- Display: `Processing: 5000 lines (estimated 50% complete)`

### 6. Performance Considerations

- Progress updates should be infrequent to avoid performance impact
- Use counter threshold (update every N items) rather than every item
- Consider: only show progress for inputs above threshold (e.g., >1000 lines)

### 7. Testing

- Add integration tests for `--progress` flag
- Test progress output goes to stderr
- Test progress doesn't interfere with stdout output
- Test with small inputs (should still work)

## Implementation Details

### Progress Callback

```rust
pub trait ProgressReporter {
    fn report_parsing(&self, line_count: usize);
    fn report_merging(&self, iteration: usize, network_count: usize);
}
```

### Simple Implementation

- For initial version, use simple function pointer or closure
- Report to stderr directly from library functions
- Keep it simple: `eprintln!("Parsing: {} lines", count)`

### Progress Threshold

- Only show progress for operations taking >1 second (estimate)
- Or: always show if `--progress` flag set, but update infrequently

## Files to Modify

- `src/main.rs`: Add `--progress` flag, implement progress display
- `src/lib.rs`: Add progress reporting hooks to parsing and merging functions
- `tests/integration_test.rs`: Add progress flag tests