---
name: Multiple Input Files
overview: Support merging CIDRs from multiple input files simultaneously
todos:
  - id: update-input-flag
    content: Change --input flag to accept multiple PathBuf values
    status: pending
  - id: create-multi-file-parser
    content: Create function to parse multiple input files
    status: pending
  - id: update-main-logic
    content: Update main() to handle multiple inputs and combine results
    status: pending
  - id: improve-error-context
    content: Add filename context to error messages
    status: pending
  - id: add-multi-file-tests
    content: Add integration tests for multiple input files
    status: pending
---

# Multiple Input Files Implementation Plan

## Overview

Add support for `--input` flag accepting multiple files, merging CIDRs from all sources together.

## Changes Required

### 1. CLI Arguments (`src/main.rs`)

- Change `--input` from `Option<PathBuf>` to `Vec<PathBuf>`
- Use `clap::ArgAction::Append` to allow multiple `--input` flags
- Maintain backward compatibility: single `--input` still works
- If no `--input` specified, read from stdin

### 2. Input Handling (`src/main.rs`)

- Create function to handle multiple inputs: `read_from_multiple_sources(inputs: &[PathBuf]) -> Result<Box<dyn BufRead>, io::Error>`
- Options:
  - Option A: Read all files into memory, concatenate
  - Option B: Use `Chain` to chain multiple readers
  - Option C: Parse each file separately, combine results
  - Recommend Option C for better error reporting (know which file failed)

### 3. Parsing Strategy (`src/main.rs`)

- Parse each input file separately using `parse_ipv4_nets()`
- Combine all parsed networks into single vector
- Preserve error context: include filename in error messages
- Handle empty files gracefully

### 4. Error Handling (`src/main.rs`)

- If one file fails, report error with filename
- Consider: fail fast vs. continue processing other files
  - Recommend fail fast for simplicity
  - Could add `--continue-on-error` flag later

### 5. Stdin Handling (`src/main.rs`)

- If no `--input` flags, read from stdin (existing behavior)
- If `--input` flags present, don't read from stdin
- Consider: allow mixing stdin and files with special marker (e.g., `-` for stdin)

### 6. Testing

- Add integration tests for multiple input files
- Test with 2-3 files containing overlapping networks
- Test error handling when one file is invalid
- Test empty files
- Test mixing valid and invalid files

## Implementation Details

### Function Signature

```rust
fn parse_multiple_inputs(inputs: &[PathBuf]) -> Result<Vec<Ipv4Net>, String> {
    let mut all_nets = Vec::new();
    for input in inputs {
        let reader = BufReader::new(File::open(input)?);
        let nets = parse_ipv4_nets(reader)
            .map_err(|e| format!("{}: {}", input.display(), e))?;
        all_nets.extend(nets);
    }
    Ok(all_nets)
}
```

### Stdin Handling

- Use `-` as special filename to represent stdin: `--input - --input file.txt`
- Or: if no `--input` flags, use stdin; otherwise ignore stdin

## Files to Modify

- `src/main.rs`: Change `--input` to accept multiple values, add multi-file parsing logic
- `tests/integration_test.rs`: Add tests for multiple input files