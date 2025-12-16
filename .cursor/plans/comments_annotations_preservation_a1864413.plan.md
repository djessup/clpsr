---
name: Comments Annotations Preservation
overview: Preserve comments and annotations from input files during parsing and output
todos:
  - id: add-preserve-comments-flag
    content: Add --preserve-comments flag to Args struct
    status: pending
  - id: create-comment-parsing
    content: Create parse_ipv4_nets_with_comments() function
    status: pending
  - id: create-network-with-comment
    content: Create NetworkWithComment struct or tuple type
    status: pending
  - id: update-merge-for-comments
    content: Update merge functions to preserve comments
    status: pending
  - id: add-comment-output
    content: Add comment formatting to output in main()
    status: pending
  - id: add-comment-tests
    content: Add unit and integration tests for comment preservation
    status: pending
---

# Comments/Annotations Preservation Implementation Plan

## Overview

Add `--preserve-comments` flag to maintain comments from input (e.g., `10.0.0.0/24  # Production`) through the merge process.

## Changes Required

### 1. CLI Arguments (`src/main.rs`)

- Add `--preserve-comments` flag (boolean) to `Args` struct
- When enabled, parse and preserve comments from input lines

### 2. Parsing with Comments (`src/lib.rs`)

- Create `parse_ipv4_nets_with_comments<R: BufRead>(reader: R) -> Result<(Vec<Ipv4Net>, Vec<Option<String>>), String>`
- Parse CIDR and optional comment (separated by `#` or `//`)
- Return tuple: (networks, comments) where comments vector aligns with networks
- Handle inline comments: `10.0.0.0/24  # Production`
- Handle end-of-line comments: `10.0.0.0/24  // Production`

### 3. Comment Association (`src/lib.rs`)

- Create `NetworkWithComment` struct or use tuple `(Ipv4Net, Option<String>)`
- Maintain comment association through merge process
- When networks merge, decide comment strategy:
  - Option A: Keep comment from first network
  - Option B: Combine comments: `# Production, Staging`
  - Option C: Keep most specific comment
  - Recommend Option A for simplicity

### 4. Merge Logic Updates (`src/lib.rs`)

- Modify merge functions to handle `Vec<(Ipv4Net, Option<String>)>` instead of `Vec<Ipv4Net>`
- Extract networks for merging, preserve comments
- After merge, associate comment with merged network
- Create wrapper functions to maintain backward compatibility

### 5. Output Formatting (`src/main.rs`)

- When `--preserve-comments` is set, output format: `{network}  # {comment}`
- Preserve original comment format (spacing, comment marker)
- Handle networks without comments

### 6. Comment Parsing Rules

- Comments start with `#` or `//`
- Comments can have leading/trailing whitespace
- Empty comments (`# `) should be preserved as empty string
- Comments after CIDR on same line

### 7. Testing

- Add unit tests for comment parsing
- Test comment preservation through merge
- Test comment handling when networks merge
- Test empty comments, missing comments
- Add integration tests for `--preserve-comments` flag

## Implementation Details

### Parsed Line Structure

```rust
struct ParsedLine {
    network: Ipv4Net,
    comment: Option<String>,
    original_line: String, // For preserving exact format
}
```

### Comment Merge Strategy

- When two networks merge, use comment from first network (by address order)
- If both have comments, concatenate: `"{comment1}, {comment2}"`
- Preserve comment spacing and markers

## Files to Modify

- `src/main.rs`: Add `--preserve-comments` flag, handle comment output
- `src/lib.rs`: Add comment parsing, update merge functions to handle comments
- `tests/integration_test.rs`: Add comment preservation tests