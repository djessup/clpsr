# clpsr

`clpsr` is a tiny command-line utility that normalizes and merges IPv4 CIDR blocks into the minimal covering set. It deduplicates input, removes subnets covered by larger ranges, and merges adjacent networks when they cleanly form a supernet.

The project is intentionally small, providing a focused alternative to heavier network-address management tools when you only need quick CIDR aggregation.

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
clpsr [--input <FILE>]
```

`clpsr` reads IPv4 CIDRs (one per line) from stdin by default. Use `--input` to point to a file instead. Empty lines are ignored; invalid CIDRs emit an error that includes the line number.

### Flags and arguments

- `-i, --input <FILE>`: Optional path to a file containing CIDRs. When omitted, stdin is used.
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

Tolerance-based merging is not available—`clpsr` only performs lossless aggregation and will not round or approximate ranges.

### Sample input and output

Input:

```
10.10.0.0/24
10.10.1.0/24
10.10.3.0/24
```

Output:

```
10.10.0.0/23
10.10.3.0/24
```

## Troubleshooting

- Ensure all lines are valid IPv4 CIDRs; errors include the failing line number.
- Remove trailing spaces or tabs that might be part of a line.
- If you see no output, confirm the input contained at least one valid CIDR.
- Run `clpsr --help` for a concise description of available flags.

See [MANUAL.md](MANUAL.md) for a detailed guide.
