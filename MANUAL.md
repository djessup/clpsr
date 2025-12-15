# clpsr Manual

## NAME

**clpsr** \- CIDR merge utility for IPv4

## SYNOPSIS

```
clpsr [--input <FILE>]
```

Reads CIDRs from standard input when `--input` is omitted. Each line should contain a single IPv4 CIDR.

## DESCRIPTION

`clpsr` normalizes, deduplicates, and merges IPv4 networks into the smallest possible set of non-overlapping prefixes. It removes subnets that are fully covered by larger ranges and merges adjacent networks with identical prefix lengths when they cleanly form their supernet. The tool performs **lossless aggregation only**; it never expands ranges or rounds prefixes.

## OPTIONS

- `-i`, `--input <FILE>`
  - Read CIDRs from the provided file instead of stdin.
- `-h`, `--help`
  - Show a short usage summary and exit.
- `-V`, `--version`
  - Print version information and exit.

## INPUT FORMAT

- IPv4 CIDRs only (e.g., `10.0.0.0/24`).
- One CIDR per line. Leading and trailing whitespace is ignored.
- Empty lines are skipped.
- Invalid lines cause the program to exit with a descriptive error that includes the line number.

## MODES

- **Lossless merge (default):** Deduplicates, removes covered subnets, and merges adjacent CIDRs that form an exact supernet. No data is dropped beyond redundant or fully covered ranges.
- **Tolerance-based merging:** Not implemented. `clpsr` refuses to approximate or widen ranges; any tolerance-driven behavior must be performed upstream before invoking the tool.

## EXAMPLES

Merge adjacent prefixes from stdin:

```bash
echo -e "10.0.0.0/24\n10.0.1.0/24" | clpsr
# 10.0.0.0/23
```

Merge and deduplicate from a file:

```bash
cat > cidrs.txt <<'CIDR'
192.168.0.0/24
192.168.0.0/24
192.168.1.0/24
CIDR
clpsr --input cidrs.txt
# 192.168.0.0/23
```

Non-mergeable ranges remain separate:

```bash
echo -e "203.0.113.0/25\n203.0.113.128/26" | clpsr
# 203.0.113.0/25
# 203.0.113.128/26
```

Invalid input produces a line-specific error:

```bash
echo -e "10.0.0.0/24\nnot-a-cidr" | clpsr
# Line 2: invalid IPv4 address syntax
```

## EXIT CODES

- `0`: Success; merged CIDRs printed to stdout.
- `1`: Runtime error such as failing to read input or parse CIDRs. The error message is printed to stderr.
- `2`: Argument parsing error produced by `clap` (e.g., unknown flags); help text is printed to stderr.

## ERROR MESSAGES

- IO failures are reported with the OS error description.
- CIDR parsing errors include the offending line number for quick debugging.
- Argument errors are handled by `clap` and include a brief usage summary.

## TROUBLESHOOTING

- Confirm the input is IPv4-only; IPv6 is not supported.
- Ensure the file or stdin stream ends with a newline so the last line is read.
- If output seems unchanged, the ranges may not be mergeable under strict lossless rules.

## RELATED TOOLS

- `aggregate`, `cidrmerge`, and other network-aggregation utilities.
- `ipcalc` for general IP math and network inspection.
