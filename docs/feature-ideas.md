# Feature Ideas for clpsr

This document contains potential features that could enhance `clpsr` while maintaining its focus as a small, focused CIDR merging utility.

## High-Value Features

### 1. Statistics/Verbose Mode
- Display before/after counts, reduction percentage, total addresses covered
- Useful for understanding merge effectiveness
- **Example**: `clpsr --stats` or `clpsr --verbose`

### 2. Output Format Options
- Support JSON output for programmatic use
- Support CSV for spreadsheet analysis
- Preserve original format but add metadata
- **Example**: `clpsr --format json`

### 3. IPv6 Support
- Extend functionality to IPv6 CIDRs (separate flag or auto-detect)
- **Example**: `clpsr --ipv6` or detect from input

### 4. Exclusion/Filtering
- Exclude specific networks from merging
- Useful when certain ranges must remain separate
- **Example**: `clpsr --exclude 10.0.0.0/24`

### 5. Validation/Check Mode
- Verify if input is already optimal (no merges possible)
- Exit code indicates if changes are needed
- **Example**: `clpsr --check` (exit 0 if optimal, 1 if not)

## Medium-Value Features

### 6. Network Information Display
- Show first/last address, address count, broadcast for each network
- Useful for debugging and verification
- **Example**: `clpsr --info`

### 7. Comments/Annotations Preservation
- Preserve comments from input (e.g., `10.0.0.0/24  # Production`)
- Maintain context during merging
- **Example**: `clpsr --preserve-comments`

### 8. Multiple Input Files
- Merge CIDRs from multiple sources
- **Example**: `clpsr --input file1.txt --input file2.txt`

### 9. Diff/Comparison Mode
- Compare two CIDR sets and show differences
- Useful for change tracking
- **Example**: `clpsr diff file1.txt file2.txt`

### 10. Maximum Prefix Length Constraint
- Prevent merging beyond a certain prefix length
- Useful for policy compliance
- **Example**: `clpsr --max-prefix 22`

## Lower Priority (Nice-to-Have)

### 11. Split/Expand Operation
- Reverse operation: expand networks into subnets
- **Example**: `clpsr split 10.0.0.0/22 --prefix 24`

### 12. Progress Indicator
- Show progress for large inputs
- Useful for very large files

### 13. Configuration File Support
- Store default tolerance, exclusions, etc.
- **Example**: `clpsr --config ~/.clpsr.toml`

### 14. Colorized Output
- Highlight merged vs. original networks
- Improve readability

## Recommendations

### Start With:
1. **Statistics/verbose mode** - Low complexity, high value
2. **Output format (JSON)** - Useful for automation
3. **Validation/check mode** - Useful for CI/CD

### Consider Later:
- **IPv6 support** - Significant architectural change
- **Exclusion/filtering** - Adds complexity but useful

### Avoid:
- Features that conflict with the "small, focused utility" philosophy
- Overly complex operations that belong in separate tools

