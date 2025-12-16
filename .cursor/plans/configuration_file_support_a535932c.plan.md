---
name: Configuration File Support
overview: Add support for configuration file to store default tolerance, exclusions, and other settings
todos:
  - id: add-toml-dependency
    content: Add toml crate to Cargo.toml dependencies
    status: pending
  - id: create-config-struct
    content: Create Config struct with serde derives
    status: pending
  - id: implement-config-loading
    content: Implement load_config() function with file search logic
    status: pending
  - id: add-config-flag
    content: Add --config flag to Args struct
    status: pending
  - id: implement-config-merging
    content: Implement config merging with CLI args in main()
    status: pending
  - id: add-config-validation
    content: Add validation for config values
    status: pending
  - id: add-config-tests
    content: Add unit and integration tests for config file support
    status: pending
---

# Configuration File Support Implementation Plan

## Overview

Add `--config` flag to load settings from TOML configuration file, storing default tolerance, exclusions, and other preferences.

## Changes Required

### 1. Dependencies (`Cargo.toml`)

- Add `toml` crate for TOML parsing: `toml = "0.8"`
- Or use `serde` with `toml` feature if serde already added

### 2. CLI Arguments (`src/main.rs`)

- Add `--config` flag accepting path to config file (default: `~/.clpsr.toml` or `./.clpsr.toml`)
- Config file settings override defaults but are overridden by CLI flags
- Priority: CLI flags > config file > defaults

### 3. Configuration Structure (`src/lib.rs` or new `src/config.rs`)

- Create `Config` struct with serde derives:
  ```rust
  #[derive(Deserialize, Debug)]
  pub struct Config {
      pub tolerance: Option<u64>,
      pub exclusions: Option<Vec<String>>,
      pub max_prefix: Option<u8>,
      pub format: Option<String>,
      pub preserve_comments: Option<bool>,
      // ... other settings
  }
  ```


### 4. Config File Format (`src/lib.rs`)

- TOML format example:
  ```toml
  tolerance = 512
  exclusions = ["10.0.0.0/24", "192.168.1.0/24"]
  max_prefix = 22
  format = "json"
  preserve_comments = true
  ```


### 5. Config Loading (`src/lib.rs` or `src/config.rs`)

- Create `load_config(path: Option<PathBuf>) -> Result<Config, String>`
- Search order:

  1. `--config` flag path (if provided)
  2. `./.clpsr.toml` (current directory)
  3. `~/.clpsr.toml` (home directory)
  4. Default config (all None)

- Return default config if file not found (not an error)

### 6. Config Merging (`src/main.rs`)

- Merge config file settings with CLI arguments
- CLI flags take precedence over config file
- Use `Option::or()` pattern: `args.tolerance.or(config.tolerance).unwrap_or(0)`

### 7. Config Validation (`src/lib.rs`)

- Validate config values (tolerance range, prefix range, etc.)
- Return descriptive errors for invalid config
- Validate exclusion CIDRs are valid

### 8. Testing

- Add unit tests for config loading and parsing
- Test config file precedence (CLI > config > default)
- Test missing config file (should use defaults)
- Test invalid config file (should return error)
- Add integration tests for `--config` flag

## Implementation Details

### Config File Location

- Default locations checked in order
- First existing file wins
- If none exist, use default config (no error)

### Settings Merging

```rust
let tolerance = args.tolerance
    .or(config.tolerance)
    .unwrap_or(0);
```

### Error Handling

- Invalid config file: return error with file path and reason
- Missing config file: use defaults (not an error)
- Invalid values in config: return validation error

## Files to Modify

- `src/main.rs`: Add `--config` flag, load and merge config
- `src/lib.rs` or `src/config.rs`: Add Config struct and loading logic
- `Cargo.toml`: Add toml dependency
- `tests/integration_test.rs`: Add config file tests