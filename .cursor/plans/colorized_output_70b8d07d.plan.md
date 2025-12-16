---
name: Colorized Output
overview: Add color support to highlight merged vs original networks and improve readability
todos:
  - id: add-color-dependencies
    content: Add colored and atty crates to Cargo.toml
    status: pending
  - id: add-color-mode-enum
    content: Create ColorMode enum with auto/always/never options
    status: pending
  - id: add-color-flag
    content: Add --color flag to Args struct
    status: pending
  - id: implement-color-detection
    content: Implement should_colorize() with TTY and NO_COLOR checks
    status: pending
  - id: add-color-formatting
    content: Add color formatting to network output in main()
    status: pending
  - id: add-color-tests
    content: Add integration tests for color functionality
    status: pending
---

# Colorized Output Implementation Plan

## Overview

Add `--color` flag to enable colorized output, highlighting merged vs original networks and improving readability.

## Changes Required

### 1. Dependencies (`Cargo.toml`)

- Add `termcolor` crate for cross-platform color support: `termcolor = "1.4"`
- Or use `colored` crate: `colored = "2.1"` (simpler API)
- Recommend `colored` for simplicity

### 2. CLI Arguments (`src/main.rs`)

- Add `--color` flag with options: `auto`, `always`, `never`
- Default: `auto` (detect if stdout is TTY)
- Use `clap::ValueEnum` for color mode enum

### 3. Color Detection (`src/main.rs`)

- Create `should_colorize(mode: ColorMode) -> bool`
- `auto`: check if stdout is TTY using `atty::is(atty::Stream::Stdout)`
- `always`: always return true
- `never`: always return false
- Add `atty` dependency if using auto detection

### 4. Color Scheme (`src/lib.rs` or `src/main.rs`)

- Define color scheme:
  - Merged networks: green (indicates optimization)
  - Original networks: default/white (if applicable)
  - Errors: red
  - Headers/labels: cyan or yellow
- Consider: highlight networks that were merged vs kept as-is

### 5. Output Formatting (`src/main.rs`)

- Create `format_colored(net: &Ipv4Net, was_merged: bool, colorize: bool) -> String`
- When colorize is true, wrap network string with color codes
- When false, return plain string
- Use `colored` crate: `format!("{}", net.to_string().green())`

### 6. Merge Tracking (`src/lib.rs`)

- Modify `merge_ipv4_nets()` to return additional metadata about which networks were merged
- Options:
  - Option A: Return `Vec<(Ipv4Net, bool)>` where bool indicates if merged
  - Option B: Return separate struct with networks and merge info
  - Option C: Don't track, just colorize all output (simpler)
  - Recommend Option C for initial implementation

### 7. Color Output Strategy

- Simple approach: colorize all output networks (green) when `--color` enabled
- Advanced: track merge status and colorize differently
- For diff mode: use red for removed, green for added, default for unchanged

### 8. Terminal Compatibility (`src/main.rs`)

- Check if output is TTY before colorizing
- Respect `NO_COLOR` environment variable (standard)
- Disable colors when piping to file (unless `--color always`)

### 9. Testing

- Add integration tests for `--color` flag
- Test color output goes to stdout (not stderr)
- Test `NO_COLOR` environment variable disables colors
- Test piping disables colors (unless `--color always`)
- Consider: test with actual color codes (may be difficult)

## Implementation Details

### Color Mode Enum

```rust
#[derive(ValueEnum, Clone, Debug)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}
```

### Color Detection

```rust
fn should_colorize(mode: ColorMode) -> bool {
    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            std::env::var("NO_COLOR").is_err() && 
            atty::is(atty::Stream::Stdout)
        }
    }
}
```

### Simple Colorization

- Use `colored` crate for simple string coloring
- Example: `format!("{}", net.to_string().green())`
- No need to track merge status for initial version

## Files to Modify

- `src/main.rs`: Add `--color` flag, implement color detection and formatting
- `Cargo.toml`: Add `colored` and optionally `atty` dependencies
- `tests/integration_test.rs`: Add color flag tests