# clpsr

`clpsr` is a tiny command-line utility that normalizes and merges IPv4 CIDR
blocks into the minimal covering set. It deduplicates input, removes subnets
covered by larger ranges, and merges adjacent networks when they cleanly form a
supernet.

The project is intentionally small, providing a focused alternative to heavier
network-address management tools when you only need quick CIDR aggregation.

## Installation

Build from source with Cargo:

```bash
cargo install --path .
```

You can also run it directly without installing while developing:

```bash
cargo run -- < args >
```

## Usage synopsis

```bash
clpsr [--input <FILE>] [--tolerance <N>]
```

`clpsr` reads IPv4 CIDRs (one per line) from stdin by default. Use `--input` to
point to a file instead. Empty lines are ignored; invalid CIDRs emit an error
that includes the line number.

### Flags and arguments

- `-i, --input <FILE>`: Optional path to a file containing CIDRs. When omitted,
  stdin is used.
- `-t, --tolerance <N>`: Maximum number of extra addresses allowed when merging
  CIDRs (default: 0). When set to N > 0, the algorithm may merge networks even
  if the resulting supernet covers addresses outside the original set, as long
  as the added address count ≤ N. Can be specified as an integer (e.g., `512`) or
  a bit mask size (e.g., `/22`). Bit mask sizes are converted to the equivalent
  number of addresses (e.g., `/22` = 1024 addresses, `/16` = 65536 addresses).
  See [Tolerance-based merging](#tolerance-based-merging) for details.
- `-h, --help`: Show usage help.
- `-V, --version`: Show the current version.

## Quickstart examples

Lossless merging from stdin:

```bash
echo -e "10.0.0.0/24\n10.0.1.0/24" | clpsr
# 10.0.0.0/23
```

Merging from a file:

```bash
cat > cidrs.txt <<'CIDR'
192.168.1.0/24
192.168.1.0/24
192.168.2.0/24
CIDR
clpsr --input cidrs.txt
# 192.168.1.0/23
```

Input that cannot be merged remains untouched:

```bash
echo -e "203.0.113.0/25\n203.0.113.128/26" | clpsr
# 203.0.113.0/25
# 203.0.113.128/26
```

### Tolerance-based merging

By default (`--tolerance 0`), `clpsr` only performs lossless merging where the
resulting supernet exactly represents the original networks. With
`--tolerance N` where N > 0, the tool may merge networks that introduce extra
addresses, as long as the added address count does not exceed N.

**Example:** Merging non-adjacent networks

```bash
# Without tolerance: networks remain separate
echo -e "10.0.0.0/24\n10.0.2.0/24" | clpsr
# 10.0.0.0/24
# 10.0.2.0/24

# With tolerance >= 512: can merge into /22 (adds 512 addresses)
echo -e "10.0.0.0/24\n10.0.2.0/24" | clpsr --tolerance 512
# 10.0.0.0/22

# Using bit mask format: /22 = 1024 addresses (equivalent to --tolerance 1024)
echo -e "10.0.0.0/24\n10.0.2.0/24" | clpsr --tolerance /22
# 10.0.0.0/22
```

**How tolerance works:**

1. The algorithm evaluates potential merges by computing the minimal supernet
   that covers both networks.
2. It calculates the number of extra addresses:
   `supernet_addresses - (network1_addresses + network2_addresses - overlap)`.
3. If the extra address count ≤ tolerance, the merge is accepted.
4. Tolerance is applied per merge operation, not globally. Each merge is
   evaluated independently against the tolerance budget.
5. The algorithm prioritizes merges that minimize the total CIDR count while
   respecting the tolerance constraint.

**Edge cases and considerations:**

- **Overlapping networks**: When networks overlap, the overlap is correctly
  accounted for in the extra address calculation.
- **Exact merges preferred**: Adjacent networks that can merge exactly (0 extra
  addresses) are always merged, regardless of tolerance.
- **Iterative merging**: The algorithm continues merging until no further merges
  are possible, potentially using tolerance across multiple iterations.
- **Tolerance per merge**: Each merge operation is evaluated independently. If
  tolerance is 512 and a merge adds 512 addresses, it's accepted. Subsequent
  merges are also evaluated independently with the same tolerance budget.

### Sample input and output

**Lossless merging (default):**

Input:

```text
10.10.0.0/24
10.10.1.0/24
10.10.3.0/24
```

Output:

```text
10.10.0.0/23
10.10.3.0/24
```

**With tolerance:**

Input:

```text
10.10.0.0/24
10.10.2.0/24
10.10.3.0/24
```

Output (tolerance 0):

```text
10.10.0.0/24
10.10.2.0/24
10.10.3.0/24
```

Output (tolerance 512):

```text
10.10.0.0/22
```

## Development

### Running tests

The project uses [`cargo-nextest`](https://nexte.st/) for faster test execution and better output. Nextest provides:

- **Faster test execution**: Parallel test running with better resource utilization
- **Better output**: Clearer test results with better formatting and progress indicators
- **Test retries**: Automatic retry of flaky tests (configured in `nextest.toml`)
- **JUnit XML reports**: For CI/CD integration

```bash
# Install nextest (if not already installed)
cargo install cargo-nextest

# Run all tests with nextest
cargo nextest run

# Run only unit tests
cargo nextest run --lib

# Run only integration tests
cargo nextest run --test integration_test

# Run tests with specific profile
cargo nextest run --profile ci

# You can also use the standard cargo test command
cargo test
```

### Code coverage

The project uses `cargo-llvm-cov` for code coverage reporting. To generate coverage reports locally:

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report (LCOV format)
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Generate HTML coverage report
cargo llvm-cov --all-features --workspace --html --output-dir coverage
```

Coverage reports are automatically generated in CI and uploaded as artifacts. Coverage data is also sent to Codecov (if configured) for tracking coverage trends over time.

### Benchmarks

Run benchmarks with:

```bash
cargo bench
```

Benchmark results are stored in `target/criterion/` and include HTML reports.

## Troubleshooting

- Ensure all lines are valid IPv4 CIDRs; errors include the failing line number.
- Remove trailing spaces or tabs that might be part of a line.
- If you see no output, confirm the input contained at least one valid CIDR.
- Run `clpsr --help` for a concise description of available flags.

See [MANUAL.md](MANUAL.md) for a detailed guide.
