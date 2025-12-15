use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;

use clpsr::{merge_ipv4_nets, parse_ipv4_nets};

/// Parse tolerance value from string.
/// Accepts either an integer (e.g., "512") or a bit mask size (e.g., "/16").
/// Bit mask sizes are converted to the equivalent number of addresses.
fn parse_tolerance(s: &str) -> Result<u64, String> {
    if s.starts_with('/') {
        // Parse as bit mask size (e.g., "/16")
        let prefix_len_str = &s[1..];
        let prefix_len: u8 = prefix_len_str
            .parse()
            .map_err(|_| format!("Invalid prefix length: {prefix_len_str}"))?;

        if prefix_len > 32 {
            return Err(format!(
                "Prefix length must be between 0 and 32, got: {prefix_len}"
            ));
        }

        // Convert prefix length to address count: 2^(32 - prefix_len)
        let address_count = 1u64 << (32 - prefix_len);
        Ok(address_count)
    } else {
        // Parse as integer
        s.parse().map_err(|_| {
            format!("Invalid tolerance value: {s}. Expected an integer or bit mask size like /16")
        })
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "CIDR merge utility", long_about = None)]
struct Args {
    /// Optional path to a file containing CIDRs (one per line). If omitted, stdin is used.
    #[arg(short, long)]
    input: Option<PathBuf>,
    /// Maximum number of extra addresses allowed when merging CIDRs. Defaults to 0 (lossless merging only).
    /// When set to N > 0, the algorithm may merge networks even if the resulting supernet covers
    /// addresses outside the original set, as long as the added address count ≤ N.
    /// Can be specified as an integer (e.g., "512") or a bit mask size (e.g., "/16").
    /// Bit mask sizes are converted to the equivalent number of addresses (e.g., "/16" = 65536 addresses).
    #[arg(short, long, default_value_t = 0, value_parser = parse_tolerance)]
    tolerance: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let reader: Box<dyn BufRead> = match args.input {
        Some(path) => Box::new(BufReader::new(File::open(path)?)),
        None => Box::new(BufReader::new(io::stdin().lock())),
    };

    let nets =
        parse_ipv4_nets(reader).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let merged = merge_ipv4_nets(nets, args.tolerance);

    for net in merged {
        println!("{net}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_tolerance;

    #[test]
    fn test_parse_tolerance_integer() {
        assert_eq!(parse_tolerance("0").unwrap(), 0);
        assert_eq!(parse_tolerance("512").unwrap(), 512);
        assert_eq!(parse_tolerance("65536").unwrap(), 65536);
        assert_eq!(parse_tolerance("4294967296").unwrap(), 4294967296);
    }

    #[test]
    fn test_parse_tolerance_bit_mask() {
        // /32 = 2^(32-32) = 2^0 = 1 address
        assert_eq!(parse_tolerance("/32").unwrap(), 1);
        // /24 = 2^(32-24) = 2^8 = 256 addresses
        assert_eq!(parse_tolerance("/24").unwrap(), 256);
        // /16 = 2^(32-16) = 2^16 = 65536 addresses
        assert_eq!(parse_tolerance("/16").unwrap(), 65536);
        // /8 = 2^(32-8) = 2^24 = 16777216 addresses
        assert_eq!(parse_tolerance("/8").unwrap(), 16777216);
        // /0 = 2^(32-0) = 2^32 = 4294967296 addresses
        assert_eq!(parse_tolerance("/0").unwrap(), 4294967296);
    }

    #[test]
    fn test_parse_tolerance_invalid_prefix_length() {
        assert!(parse_tolerance("/33").is_err());
        assert!(parse_tolerance("/100").is_err());
        assert!(parse_tolerance("/-1").is_err());
    }

    #[test]
    fn test_parse_tolerance_invalid_format() {
        assert!(parse_tolerance("abc").is_err());
        assert!(parse_tolerance("").is_err());
        assert!(parse_tolerance("16/").is_err());
        assert!(parse_tolerance("//16").is_err());
    }

    #[test]
    fn test_parse_tolerance_edge_cases() {
        // Test that /16 equals 65536 (common use case)
        assert_eq!(parse_tolerance("/16").unwrap(), 65536);
        // Test that /22 equals 1024 (another common use case)
        assert_eq!(parse_tolerance("/22").unwrap(), 1024);
        // Test that /23 equals 512 (another common use case)
        assert_eq!(parse_tolerance("/23").unwrap(), 512);
    }
}
