# AGENTS.md

## 1. Project Overview & Purpose

### System Purpose & Constraints

`clpsr` merges IPv4 CIDR blocks into a minimal covering set. Reads from stdin or file, parses, deduplicates, merges adjacent networks. Processes input line-by-line for memory efficiency, outputs to stdout. Optional tolerance parameter allows merging networks that introduce extra addresses within a specified budget.

### Critical Invariants

- **Input parsing**: Empty lines ignored; invalid CIDRs return descriptive errors with line numbers
- **Merging algorithm**: Networks merge only when adjacent with identical prefix lengths and combined supernet exactly represents both subnets (tolerance=0). With tolerance > 0, networks may merge if extra address count ≤ tolerance
- **Output ordering**: Sorted by network address (as u32), then by prefix length
- **Coverage removal**: Subnets fully covered by supernets are removed
- **Iterative merging**: Continues until no further merges possible
- **Error propagation**: Parsing errors return `String`; I/O errors use `io::Error`
- **Pure functions**: Core logic in `lib.rs` has no side effects; all I/O isolated to `main.rs`
- **Tolerance semantics**: Applied per merge operation, not globally; exact merges (0 extra addresses) always preferred

### Key Dependencies

- **`ipnet`** (v2.9): `Ipv4Net` type for CIDR representation and parsing. Used throughout `lib.rs`.
- **`clap`** (v4.5 with derive): CLI argument parsing. Used only in `main.rs`.
- **`criterion`** (dev-dependency): Benchmarking framework in `benches/`.

### Architecture Overview

1. **CLI Entry Point** (`src/main.rs`): Parses `--input`, `--tolerance`; handles I/O; calls library functions
2. **Core Library** (`src/lib.rs`): `parse_ipv4_nets`, `merge_ipv4_nets`, helpers (`sort_and_dedup`, `remove_covered_nets`, `network_covers`, `try_merge_exact`, `try_merge_with_tolerance`, `find_covering_supernet`)
3. **Tests**: `src/lib.rs` mod tests (unit), `tests/integration_test.rs` (integration)
4. **Benchmarks**: `benches/parse_bench.rs`, `benches/merge_bench.rs`

## 2. Repository Structure & Navigation

### Repository Map

- **`src/`**: `main.rs` (CLI), `lib.rs` (core library + unit tests)
- **`tests/`**: `integration_test.rs` (end-to-end tests)
- **`benches/`**: `parse_bench.rs`, `merge_bench.rs` (performance benchmarks)
- **`target/`**: Build artifacts - **Do not modify**
- **`Cargo.toml`**: Dependencies, edition (2024)
- **`Cargo.lock`**: Auto-generated - **Do not modify**

### Golden Path Trace

1. `src/main.rs::main()` (line 22) - Entry
2. `src/main.rs::Args::parse()` (line 23) - Parse `--input`, `--tolerance`
3. `src/main.rs::main()` (lines 25-28) - Create `BufRead` from stdin/file
4. `src/lib.rs::parse_ipv4_nets()` (line 9) - Parse CIDRs, return `Vec<Ipv4Net>`
5. `src/lib.rs::merge_ipv4_nets()` (line 38) - Merge iteratively:
   - `sort_and_dedup()` (line 40)
   - `try_merge_with_tolerance()` (line 52)
   - `remove_covered_nets()` (line 65)
   - Repeat until no changes (lines 42-68)
6. `src/main.rs::main()` (lines 34-36) - Print results

### Task→Location Routing Table

| Task | Start in | Usually touch |
|------|----------|----------------|
| Modify CLI arguments/flags | `src/main.rs` | `src/main.rs` |
| Change parsing behavior | `src/lib.rs::parse_ipv4_nets` | `src/lib.rs`, tests |
| Modify merging algorithm | `src/lib.rs::merge_ipv4_nets` | `src/lib.rs`, helpers, tests |
| Add utility functions | `src/lib.rs` | `src/lib.rs`, tests if public |
| Add integration tests | `tests/integration_test.rs` | `tests/integration_test.rs` |
| Add benchmarks | `benches/` | `benches/*.rs` |
| Update dependencies | `Cargo.toml` | `Cargo.toml` |
| Add test cases | `src/lib.rs` mod tests | `src/lib.rs` |

### Do-Not-Open List

- **`target/`**: Build artifacts, never modify
- **`Cargo.lock`**: Auto-generated, only modify via `cargo update`

### Entry Point Index

- **CLI**: `src/main.rs::main()` - `cargo run -- --input <file> --tolerance <N>`
- **Library**: `src/lib.rs` - Public: `parse_ipv4_nets()`, `merge_ipv4_nets()`
- **Unit tests**: `src/lib.rs` mod tests - `cargo test --lib`
- **Integration tests**: `tests/integration_test.rs` - `cargo test --test integration_test`
- **Benchmarks**: `benches/*.rs` - `cargo bench`

### When to Use Retrieval vs Direct Inspection

- **Codebase retrieval**: Don't know which file contains function; searching for concept implementation; understanding structure
- **Direct file view**: Have exact path; need complete implementation; reading tests for specific function
- **Symbol search**: Finding all usages of function/type
- **Regex search**: Finding patterns (e.g., `#[test]`), error messages, string literals

### Component Boundaries

- **`main.rs`**: I/O, CLI parsing, orchestration. NO business logic.
- **`lib.rs`**: Parsing, merging logic. NO I/O or CLI parsing. Public API: 2 functions.
- **`tests/integration_test.rs`**: End-to-end CLI/library tests. NO unit tests.

### Minimal Working Sets

- **CLI changes**: `src/main.rs`, `Cargo.toml` (if adding deps)
- **Algorithm changes**: `src/lib.rs` (functions + tests)
- **Adding tests**: `src/lib.rs` mod tests (unit) or `tests/integration_test.rs` (integration)
- **Adding benchmarks**: `benches/*.rs`

### Saved Search Recipes

- Test functions: `#[test]`
- Public functions: `^pub fn`
- Test helpers: `^pub\(crate\) fn`
- Error handling: `map_err|Result<|Err\(`
- Network operations: `Ipv4Net|prefix_len|network\(\)`
- Tolerance code: `tolerance|try_merge_with_tolerance`

## 3. Conventions, Patterns & Standards

### Enforced Style & Lint Rules

- **Formatter**: `cargo fmt`
- **Linter**: `cargo clippy` (fix all warnings)
- **Type check**: `cargo check`
- **Build**: `cargo build` (use `--release` for optimized)
- **Tests**: `cargo test` (`--lib` for unit only, `--test integration_test` for integration)
- **Benchmarks**: `cargo bench`

### Naming Conventions

- **Functions**: `snake_case` with verbs (`parse_ipv4_nets`, `merge_ipv4_nets`, `remove_covered_nets`)
- **Helpers**: `snake_case`, action-prefixed (`try_merge_exact`, `sort_and_dedup`, `network_covers`)
- **Implementation**: `_impl` suffix for logic, wrappers handle `#[cfg]` attributes
- **Types**: Use `Ipv4Net` from `ipnet`. No custom types.
- **Files**: `snake_case.rs`
- **Tests**: Descriptive names matching what they test (`merges_adjacent_subnets`, `tolerance_allows_non_adjacent_merge`)
- **CLI args**: `kebab-case` in help, `snake_case` in struct fields

### Existing Utilities Index

- **`src/lib.rs::parse_ipv4_nets<R: BufRead>(reader: R) -> Result<Vec<Ipv4Net>, String>`**: Parse CIDRs from reader. Ignores empty lines. Errors include line numbers.
- **`src/lib.rs::merge_ipv4_nets(nets: Vec<Ipv4Net>, tolerance: u64) -> Vec<Ipv4Net>`**: Merge and normalize. Tolerance controls extra addresses (0 = lossless).
- **`ipnet::Ipv4Net`**: Core CIDR type. Use `parse()`, `addr()`, `prefix_len()`, `network()`, `broadcast()`.
- **`src/lib.rs::sort_and_dedup(nets: &mut Vec<Ipv4Net>)`** (pub(crate)): Sort by address/prefix, remove duplicates.
- **`src/lib.rs::remove_covered_nets(nets: Vec<Ipv4Net>) -> (Vec<Ipv4Net>, bool)`** (pub(crate)): Remove covered subnets, return compacted list and changed flag.
- **`src/lib.rs::network_covers(supernet: &Ipv4Net, subnet: &Ipv4Net) -> bool`** (pub(crate)): Check if supernet covers subnet.

### Modify vs Extend vs Create Decision Rules

- **Do NOT modify**: `target/`, `Cargo.lock` (auto-generated)
- **Extend existing functions**: When adding similar functionality
- **Create private helpers**: In `lib.rs` for orthogonal logic (`try_merge_exact`, `network_covers`, `find_covering_supernet`)
- **Create public functions**: Only if distinct, reusable operations
- **Modify `main.rs`**: For CLI changes (flags, I/O handling)
- **Add tests**: In `src/lib.rs` mod tests (unit) or `tests/integration_test.rs` (integration)
- **Create modules**: Only if file >1000 lines or distinct concerns
- **Use `pub(crate)`**: For testable functions not in public API

### Anti-Patterns & Deprecated Approaches

❌ **DON'T**: Parse CIDR strings manually

```rust
let parts: Vec<&str> = cidr.split('/').collect();
```

✅ **DO**: Use `ipnet::Ipv4Net::parse()` or `parse_ipv4_nets()`

```rust
let net: Ipv4Net = cidr.parse()?;
```

❌ **DON'T**: Perform I/O in library functions (`lib.rs`)

```rust
pub fn process_file(path: &str) -> Vec<Ipv4Net> {
    let file = File::open(path)?; // Don't do this in lib.rs
}
```

✅ **DO**: Keep I/O in `main.rs`, pass `BufRead` to library

```rust
let reader = BufReader::new(File::open(path)?);
let nets = parse_ipv4_nets(reader)?;
```

❌ **DON'T**: Modify vectors in place without clear intent

```rust
fn merge(nets: &mut Vec<Ipv4Net>) { nets.sort(); }
```

✅ **DO**: Use descriptive names and clear ownership

```rust
fn sort_and_dedup(nets: &mut Vec<Ipv4Net>) { /* ... */ }
```

❌ **DON'T**: Hardcode tolerance

```rust
let merged = merge_ipv4_nets(nets, 512); // Magic number
```

✅ **DO**: Pass tolerance from CLI

```rust
let merged = merge_ipv4_nets(nets, args.tolerance);
```

### Architectural Principles

- **Separation**: I/O (`main.rs`) vs. logic (`lib.rs`)
- **Pure functions**: Core algorithms have no side effects
- **Error handling**: `Result<T, E>` with context (line numbers for parsing errors)
- **Ownership**: Prefer owned types (`Vec<Ipv4Net>`) for return values
- **Test visibility**: `pub(crate)` with `#[cfg(test)]` wrappers for test-only visibility

**Example violation**:

❌ **Violation**: Mixing I/O with parsing

```rust
// In lib.rs - WRONG
pub fn parse_file(path: &str) -> Result<Vec<Ipv4Net>, String> {
    let file = File::open(path)?; // I/O in library
}
```

✅ **Fix**: Separate I/O from parsing

```rust
// In lib.rs - CORRECT
pub fn parse_ipv4_nets<R: BufRead>(reader: R) -> Result<Vec<Ipv4Net>, String> {
    // Only parsing logic, no I/O
}

// In main.rs - CORRECT
let reader = BufReader::new(File::open(path)?);
let nets = parse_ipv4_nets(reader)?;
```

### Performance Patterns

- **Streaming input**: Use `BufRead` for line-by-line processing
- **Iterative merging**: Continues until fixed point
- **In-place sorting**: `sort_and_dedup` modifies vector in place
- **Early termination**: Empty input returns immediately
- **Tolerance evaluation**: Checked per merge operation, not accumulated

## 4. Quality Standards & Completeness Requirements

### Definition of Done Checklist

- [ ] **Tests updated/added**: In `src/lib.rs` mod tests (unit) or `tests/integration_test.rs` (integration)
- [ ] **Formatter passes**: `cargo fmt` (no changes needed)
- [ ] **Linter passes**: `cargo clippy` (fix all warnings)
- [ ] **Type checks**: `cargo check` succeeds
- [ ] **All tests pass**: `cargo test` (unit + integration)
- [ ] **Error handling**: Fallible operations return `Result<T, E>` with descriptive errors
- [ ] **Documentation**: Public functions have doc comments (`///`) with purpose, params, returns, errors
- [ ] **Input validation**: Parsing functions validate input, return errors with context (line numbers)
- [ ] **No panics**: Avoid `unwrap()`/`expect()` in library code
- [ ] **Backward compatibility**: Public API changes maintain compatibility or document breaking changes

### How to Run Tests Quickly

```bash
cargo test                          # All tests
cargo test --lib                    # Unit tests only
cargo test --test integration_test  # Integration tests only
cargo test merges_adjacent          # Specific test pattern
cargo test -- --nocapture           # Show println! output
cargo test -- --test-threads=1      # Single-threaded (debugging)
cargo bench                         # Benchmarks
```

### Security & Compliance Guardrails

- **Input validation**: CIDR parsing validates via `ipnet::Ipv4Net::parse()`. Invalid input returns errors, never panics.
- **No secrets**: Tool doesn't handle secrets/auth. No special security beyond input validation.
- **Safe deserialization**: Uses `ipnet` library (returns `Result`).
- **Dependency constraints**: Pin in `Cargo.toml`. Review `Cargo.lock` for updates.
- **File I/O**: Standard library operations. No special permissions needed.

### Performance/Footprint Budgets

- **Memory**: Process line-by-line (`BufRead`) for large files. Avoid loading entire file.
- **Algorithm complexity**: Efficient for <1000 networks. Consider optimization for >10,000.
- **Pitfalls**:
  - ❌ `std::fs::read_to_string()` for large files
  - ✅ `BufRead` for streaming: `BufReader::new(File::open()?)`
  - ❌ Parsing CIDRs multiple times
  - ✅ Single parse pass, reuse `Ipv4Net` values
  - ❌ Unnecessary copies
  - ✅ In-place operations: `sort_and_dedup` modifies vector

**Detection**:

```bash
cargo build --release              # Optimized build
ls -lh target/release/clpsr         # Binary size
cargo bench                         # Performance regressions
```

### API/Schema Change Impact Checklist

When changing public API (`src/lib.rs` public functions) or CLI:

- [ ] **Update callers**: Check `src/main.rs` for usage
- [ ] **Update tests**: Match new signatures/behavior
- [ ] **Update documentation**: Doc comments (`///`) for changed functions
- [ ] **Backward compatibility**: Maintain or document breaking changes
- [ ] **Version bump**: Update `Cargo.toml` version if breaking
- [ ] **CLI help**: Update `clap` derive attributes if CLI changes

**Example**: Changing `merge_ipv4_nets` signature:
1. Update signature in `src/lib.rs`
2. Update call site in `src/main.rs` (line 32)
3. Update tests in `src/lib.rs` mod tests
4. Update integration tests in `tests/integration_test.rs`
5. Update doc comment
6. Run `cargo test`

### Observability Standards

- **Error messages**: Include context (line numbers for parsing errors)
- **No logging**: Uses stdout for output, stderr for errors (`println!`, `eprintln!`)
- **Exit codes**: `Ok(())` on success, error on failure

**Example error**:

```rust
// Good: Includes line number
Err(format!("Line {}: {}", idx + 1, err))

// Bad: Generic error
Err("Parse error".to_string())
```

### Required Artifacts

- **Public API docs**: Doc comments (`///`) on all public functions in `src/lib.rs`
- **CLI help**: Auto-generated by `clap` via `cargo run -- --help`
- **Tests**: Coverage in `src/lib.rs` mod tests (unit) and `tests/integration_test.rs` (integration)
