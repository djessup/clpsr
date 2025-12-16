# Implementation Roadmap

This document outlines the optimal order for implementing the 14 planned
features to minimize rework and maximize efficiency.

## Dependency Analysis

### Breaking Changes (Signature Changes)

- **Comments/Annotations Preservation**: Changes `parse_ipv4_nets()` return type
  and `merge_ipv4_nets()` to handle comments
- **IPv6 Support**: Changes core types from `Ipv4Net` to `IpNet` (major
  architectural change)
- **Maximum Prefix Length Constraint**: Adds parameter to `merge_ipv4_nets()`
- **Exclusion/Filtering**: Adds parameter to `merge_ipv4_nets()`

### Structural Changes (CLI Architecture)

- **Diff/Comparison Mode**: Requires subcommand structure
- **Split/Expand Operation**: Requires subcommand structure

### Shared Infrastructure

- **Output Format Options**: Creates `NetworkInfo` struct (reusable by Network
  Information Display)
- **Configuration File Support**: Provides defaults for other features

### Independent Features

- Statistics/Verbose Mode, Validation/Check Mode, Progress Indicator, Colorized
  Output

## Current State

**v1.0.0**: Current release - Basic CIDR merging functionality

- Core merge algorithm with tolerance support
- Single input file or stdin
- Plain text output

## Recommended Implementation Order

### v1.1.0: Foundation & Quick Wins

**Goal**: Add value without breaking changes, establish patterns

1. **Statistics/Verbose Mode** ⭐ START HERE

   - No signature changes
   - Uses existing functions
   - Provides immediate value
   - Establishes stderr output pattern

2. **Validation/Check Mode**

   - No signature changes
   - Uses existing merge logic
   - Useful for CI/CD
   - Establishes exit code patterns

3. **Multiple Input Files**
   - Changes input handling only
   - No core signature changes
   - Useful immediately
   - Establishes multi-source pattern

### v1.2.0: Output Infrastructure

**Goal**: Build reusable output infrastructure

4. **Output Format Options**

   - Creates `NetworkInfo` struct (reusable)
   - Adds serde dependency
   - Establishes format abstraction
   - **Benefit**: Network Information Display can reuse `NetworkInfo`

5. **Network Information Display**

   - Reuses `NetworkInfo` from #4
   - Adds `--info` flag
   - Minimal new code

6. **Colorized Output**
   - Builds on output formatting
   - Independent feature
   - Can be done anytime after output formats

### v1.3.0: Merge Logic Enhancements

**Goal**: Enhance merge behavior before major refactors

7. **Maximum Prefix Length Constraint**

   - Modifies `merge_ipv4_nets()` signature
   - Should be done before IPv6 (easier to retrofit IPv4-only)
   - Establishes parameter pattern for merge options

8. **Exclusion/Filtering**
   - Modifies `merge_ipv4_nets()` signature
   - Similar pattern to #7
   - Can combine with #7 in same refactor if desired

### v1.4.0: Infrastructure Features

**Goal**: Add infrastructure that benefits remaining features

9. **Configuration File Support**

   - Provides defaults for other features
   - Should be done before features that can use config
   - Establishes config loading pattern

10. **Progress Indicator**
    - Adds callbacks to parsing/merging
    - Should be done before major refactors
    - Easier to add now than after IPv6/comments

### v2.0.0: Comments/Annotations Preservation ⚠️ BREAKING CHANGE

**Goal**: Handle breaking changes, major version bump

11. **Comments/Annotations Preservation**
    - Changes parsing and merge signatures
    - Major refactor required
    - Do this before IPv6 (easier to add IPv6 to comment-aware code)
    - **Impact**: All downstream code needs updates
    - **Version**: Major version bump due to breaking API changes

### v3.0.0: IPv6 Support ⚠️ MAJOR ARCHITECTURAL CHANGE

**Goal**: Add IPv6 support, major architectural change

12. **IPv6 Support**
    - Changes core types (`Ipv4Net` → `IpNet`)
    - Requires updating all features
    - Should be done after comments (easier to add IPv6 to comment-aware code)
    - **Impact**: Everything needs IPv6 support
    - **Version**: Major version bump due to architectural changes

### v3.1.0: Subcommand Features

**Goal**: Add subcommands that require CLI restructuring

13. **Diff/Comparison Mode**

    - Requires subcommand structure
    - Should be done after core features are stable
    - Establishes subcommand pattern

14. **Split/Expand Operation**
    - Requires subcommand structure
    - Can reuse subcommand pattern from #13
    - Natural pairing with diff

## Version Summary

```
v1.0.0: Current Release ✅
└── Basic CIDR merging functionality

v1.1.0: Foundation (3 features)
├── Statistics/Verbose Mode
├── Validation/Check Mode
└── Multiple Input Files

v1.2.0: Output Infrastructure (3 features)
├── Output Format Options
├── Network Information Display
└── Colorized Output

v1.3.0: Merge Enhancements (2 features)
├── Maximum Prefix Length Constraint
└── Exclusion/Filtering

v1.4.0: Infrastructure (2 features)
├── Configuration File Support
└── Progress Indicator

v2.0.0: Breaking Changes (1 feature) ⚠️
└── Comments/Annotations Preservation

v3.0.0: IPv6 Support (1 feature) ⚠️
└── IPv6 Support

v3.1.0: Subcommands (2 features)
├── Diff/Comparison Mode
└── Split/Expand Operation
```

## Key Decision Points

### When to Do IPv6?

**Option A: Early (v1.2.0)**

- Pro: All features get IPv6 support from start
- Con: Every feature becomes more complex immediately
- Con: Slows down initial development

**Option B: Late (v3.0.0) - RECOMMENDED**

- Pro: Features can be implemented quickly for IPv4
- Pro: IPv6 can be added systematically later
- Con: Need to retrofit IPv6 support
- **Recommendation**: Do IPv6 late (v3.0.0), after core features are stable

### When to Do Comments?

**Option A: Early (v1.1.0)**

- Pro: All features handle comments from start
- Con: Slows down initial development
- Con: Makes every feature more complex

**Option B: Late (v2.0.0) - RECOMMENDED**

- Pro: Features can be implemented quickly
- Pro: Comments can be added systematically
- Con: Need to retrofit comment support
- **Recommendation**: Do comments late (v2.0.0), but before IPv6 (v3.0.0)

### Subcommand Structure

- **Decision**: Implement subcommands in v3.1.0
- **Rationale**: Core merge functionality should be stable first
- **Benefit**: Diff and Split can share subcommand infrastructure

## Risk Mitigation

### High-Risk Features

1. **IPv6 Support**: Test thoroughly, consider feature flag
2. **Comments Preservation**: May require significant refactoring
3. **Subcommand Structure**: Requires careful CLI design

### Low-Risk Features (Good Starting Points)

1. Statistics/Verbose Mode
2. Validation/Check Mode
3. Output Format Options
4. Colorized Output

## Alternative: Incremental Value Approach

If prioritizing quick value delivery, focus on these low-risk, high-value
features first:

1. Statistics/Verbose Mode
2. Validation/Check Mode
3. Output Format Options
4. Multiple Input Files
5. Network Information Display (reuses #3)
6. Colorized Output

All six features are low-risk and provide immediate value. Then proceed with
remaining features as needed.
