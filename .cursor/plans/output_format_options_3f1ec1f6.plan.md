---
name: Output Format Options
overview: Add support for JSON and CSV output formats for programmatic use
todos:
  - id: add-format-enum
    content: Create OutputFormat enum with clap::ValueEnum in src/main.rs
    status: pending
  - id: add-format-flag
    content: Add --format flag to Args struct
    status: pending
  - id: create-network-info
    content: Create NetworkInfo struct in src/lib.rs
    status: pending
  - id: implement-formatters
    content: Implement format_output_plain/json/csv functions
    status: pending
  - id: update-main-output
    content: Update main() to use format-specific output
    status: pending
  - id: add-dependencies
    content: Add serde and serde_json to Cargo.toml
    status: pending
  - id: add-format-tests
    content: Add unit and integration tests for formats
    status: pending
---

# Output Format Options Implementation Plan

## Overview

Add `--format` flag to support JSON and CSV output formats in addition to the default plain text format.

## Changes Required

### 1. CLI Arguments (`src/main.rs`)

- Add `--format` flag to `Args` struct with enum values: `plain`, `json`, `csv`
- Use `clap::ValueEnum` derive for format enum
- Default to `plain` for backward compatibility

### 2. Output Formatting Functions (`src/lib.rs`)

- Create `format_output_plain(nets: &[Ipv4Net]) -> String`
- Create `format_output_json(nets: &[Ipv4Net]) -> String`
- Create `format_output_csv(nets: &[Ipv4Net]) -> String`
- Consider adding `serde` and `serde_json` dependencies for JSON serialization
- For JSON: array of objects with `network`, `prefix`, `first_address`, `last_address`, `address_count`
- For CSV: header row, then rows with `network,prefix,first_address,last_address,address_count`

### 3. Network Metadata (`src/lib.rs`)

- Create helper function `network_metadata(net: &Ipv4Net) -> NetworkInfo`
- Struct contains: network address, prefix length, first address, last address, address count
- Use existing `network_address_count()` helper

### 4. Main Function Updates (`src/main.rs`)

- Replace direct `println!` loop with format-specific output function
- Call appropriate formatter based on `args.format`
- Output to stdout (maintains piping compatibility)

### 5. Dependencies (`Cargo.toml`)

- Add `serde = { version = "1.0", features = ["derive"] }`
- Add `serde_json = "1.0"` (or use `ipnet`'s serde support if available)

### 6. Testing

- Add unit tests for each formatter in `src/lib.rs` mod tests
- Test JSON validity (parse output and verify structure)
- Test CSV format (verify headers and row count)
- Add integration tests in `tests/integration_test.rs` for each format

## Implementation Details

### Format Enum

```rust
#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Plain,
    Json,
    Csv,
}
```

### NetworkInfo Struct

```rust
#[derive(Serialize)]
pub struct NetworkInfo {
    pub network: String,
    pub prefix: u8,
    pub first_address: String,
    pub last_address: String,
    pub address_count: u64,
}
```

## Files to Modify

- `src/main.rs`: Add format flag, call formatters
- `src/lib.rs`: Add formatter functions and NetworkInfo struct
- `Cargo.toml`: Add serde dependencies
- `tests/integration_test.rs`: Add format tests