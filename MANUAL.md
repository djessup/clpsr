# clpsr Manual

## NAME

**clpsr** \- CIDR merge utility for IPv4

## SYNOPSIS

```
clpsr [--input <FILE>] [--tolerance <N>]
```

Reads CIDRs from standard input when `--input` is omitted. Each line should contain a single IPv4 CIDR.

## DESCRIPTION

`clpsr` normalizes, deduplicates, and merges IPv4 networks into the smallest possible set of non-overlapping prefixes. It removes subnets that are fully covered by larger ranges and merges adjacent networks with identical prefix lengths when they cleanly form their supernet. The tool performs **lossless aggregation only**; it never expands ranges or rounds prefixes.

## OPTIONS

- `-i`, `--input <FILE>`
  - Read CIDRs from the provided file instead of stdin.
- `-t`, `--tolerance <N>`
  - Maximum number of extra addresses allowed when merging CIDRs (default: 0). When set to N > 0, the algorithm may merge networks even if the resulting supernet covers addresses outside the original set, as long as the added address count ≤ N. Can be specified as an integer (e.g., `512`) or a bit mask size (e.g., `/22`). Bit mask sizes are converted to the equivalent number of addresses (e.g., `/22` = 1024 addresses, `/16` = 65536 addresses). See [Tolerance-based merging](#tolerance-based-merging) for details.
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
- **Tolerance-based merging:** When `--tolerance N` is specified with N > 0, the tool may merge networks that introduce extra addresses, as long as the added address count does not exceed N. Tolerance can be specified as an integer (e.g., `512`) or a bit mask size (e.g., `/22`). See [Tolerance-based merging](#tolerance-based-merging) for details.

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

Tolerance-based merging example:

```bash
# Without tolerance: networks remain separate
echo -e "10.0.0.0/24\n10.0.2.0/24" | clpsr
# 10.0.0.0/24
# 10.0.2.0/24

# With integer tolerance >= 512: can merge into /22 (adds 512 addresses)
echo -e "10.0.0.0/24\n10.0.2.0/24" | clpsr --tolerance 512
# 10.0.0.0/22

# Using bit mask format: /22 = 1024 addresses
echo -e "10.0.0.0/24\n10.0.2.0/24" | clpsr --tolerance /22
# 10.0.0.0/22
```

## TOLERANCE-BASED MERGING

By default (`--tolerance 0`), `clpsr` only performs lossless merging where the resulting supernet exactly represents the original networks. With `--tolerance N` where N > 0, the tool may merge networks that introduce extra addresses, as long as the added address count does not exceed N.

**Tolerance format:**

- **Integer format:** Specify the exact number of extra addresses allowed (e.g., `--tolerance 512`).
- **Bit mask format:** Specify a prefix length, which is converted to the equivalent number of addresses (e.g., `--tolerance /22` = 1024 addresses, `--tolerance /16` = 65536 addresses).

**How tolerance works:**

1. The algorithm evaluates potential merges by computing the minimal supernet that covers both networks.
2. It calculates the number of extra addresses: `supernet_addresses - (network1_addresses + network2_addresses - overlap)`.
3. If the extra address count ≤ tolerance, the merge is accepted.
4. Tolerance is applied per merge operation, not globally. Each merge is evaluated independently against the tolerance budget.
5. The algorithm prioritizes merges that minimize the total CIDR count while respecting the tolerance constraint.

**Edge cases and considerations:**

- **Overlapping networks:** When networks overlap, the overlap is correctly accounted for in the extra address calculation.
- **Exact merges preferred:** Adjacent networks that can merge exactly (0 extra addresses) are always merged, regardless of tolerance.
- **Iterative merging:** The algorithm continues merging until no further merges are possible, potentially using tolerance across multiple iterations.
- **Tolerance per merge:** Each merge operation is evaluated independently. If tolerance is 512 and a merge adds 512 addresses, it's accepted. Subsequent merges are also evaluated independently with the same tolerance budget.

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
