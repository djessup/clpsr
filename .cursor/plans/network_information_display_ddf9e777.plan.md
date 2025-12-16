---
name: Network Information Display
overview: Add --info flag to display detailed network information for each CIDR
todos:
  - id: add-info-flag
    content: Add --info boolean flag to Args struct
    status: pending
  - id: create-network-info-struct
    content: Create NetworkInfo struct in src/lib.rs
    status: pending
  - id: implement-get-network-info
    content: Implement get_network_info() function
    status: pending
  - id: add-info-formatting
    content: Add formatted output for --info mode in main()
    status: pending
  - id: add-info-tests
    content: Add unit and integration tests for info display
    status: pending
---

# Network Information Display Implementation Plan

## Overview

Add `--info` flag to display first/last address, address count, and broadcast address for each network.

## Changes Required

### 1. CLI Arguments (`src/main.rs`)

- Add `--info` flag (boolean) to `Args` struct
- When enabled, display detailed information for each network

### 2. Network Information Struct (`src/lib.rs`)

- Create `NetworkInfo` struct (may reuse from Output Format Options feature)
- Fields: network address, prefix length, first address, last address, broadcast address, address count
- Use existing `network_address_count()` helper

### 3. Information Extraction (`src/lib.rs`)

- Create `get_network_info(net: &Ipv4Net) -> NetworkInfo`
- Extract: `net.network()` (first), `net.broadcast()` (last/broadcast), `net.prefix_len()`, address count
- Format addresses as strings

### 4. Output Formatting (`src/main.rs`)

- When `--info` flag is set, format output with detailed information
- Format options:
  - Table format with columns
  - Multi-line format per network
  - Consider alignment for readability

### 5. Output Format Options

- Option A: Table format (recommended)
  ```
  Network          Prefix  First Address    Last Address     Broadcast        Addresses
  10.0.0.0/24      24      10.0.0.0         10.0.0.255       10.0.0.255       256
  ```

- Option B: Multi-line format
  ```
  10.0.0.0/24:
    Prefix: 24
    First: 10.0.0.0
    Last: 10.0.0.255
    Broadcast: 10.0.0.255
    Addresses: 256
  ```


### 6. Testing

- Add unit tests for `get_network_info()` function
- Test with various prefix lengths (/32, /24, /16, /8, /0)
- Verify address calculations are correct
- Add integration test for `--info` flag

## Implementation Details

### NetworkInfo Struct

```rust
pub struct NetworkInfo {
    pub network: Strin
    
g,
    pub prefix: u8,
    pub first_address: String,
    pub last_address: String,
    pub broadcast_address: String,
    pub address_count: u64,
}
```

### Integration with Existing Code

- Can reuse `NetworkInfo` from Output Format Options if that feature is implemented
- Use `ipnet::Ipv4Net` methods: `network()`, `broadcast()`, `prefix_len()`

## Files to Modify

- `src/main.rs`: Add `--info` flag, format detailed output
- `src/lib.rs`: Add `get_network_info()` function and `NetworkInfo` struct
- `tests/integration_test.rs`: Add info mode tests