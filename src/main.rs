use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::process;

use clap::Parser;
use ipnet::Ipv4Net;

use clpsr::{merge_ipv4_nets, parse_ipv4_nets};

/// Parses a tolerance value from a string.
///
/// Accepts two formats:
/// - Integer: A decimal number representing the maximum number of extra addresses (e.g., `"512"`)
/// - Bit mask: A prefix length in CIDR notation (e.g., `"/16"`), which is converted to the
///   equivalent number of addresses using the formula `2^(32 - prefix_len)`
///
/// # Arguments
///
/// * `s` - String to parse as a tolerance value
///
/// # Returns
///
/// * `Ok(u64)` - The tolerance value as a number of addresses
/// * `Err(String)` - Error message if parsing fails
///
/// # Examples
///
/// ```
/// assert_eq!(parse_tolerance("512").unwrap(), 512);
/// assert_eq!(parse_tolerance("/16").unwrap(), 65536);  // 2^(32-16) = 65536
/// assert_eq!(parse_tolerance("/24").unwrap(), 256);   // 2^(32-24) = 256
/// ```
fn parse_tolerance(s: &str) -> Result<u64, String> {
    if let Some(prefix_len_str) = s.strip_prefix('/') {
        // Parse as bit mask size (e.g., "/16")
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

/// Command-line arguments for the CIDR merge utility.
#[derive(Parser, Debug)]
#[command(author, version, about = "CIDR merge utility", long_about = None)]
struct Args {
    /// Optional path to a file containing CIDRs (one per line).
    ///
    /// If omitted, CIDRs are read from standard input. Empty lines are ignored.
    /// Each non-empty line should contain a single IPv4 CIDR block in standard notation
    /// (e.g., `10.0.0.0/24`).
    #[arg(short, long)]
    input: Option<PathBuf>,
    /// Maximum number of extra addresses allowed when merging CIDRs.
    ///
    /// Defaults to `0`, which means only lossless (exact) merges are performed.
    /// When set to `N > 0`, the algorithm may merge networks even if the resulting
    /// supernet covers addresses outside the original set, as long as the added
    /// address count does not exceed `N`.
    ///
    /// Can be specified in two formats:
    /// - Integer: A decimal number (e.g., `512`)
    /// - Bit mask: A prefix length in CIDR notation (e.g., `"/16"`), which is converted
    ///   to the equivalent number of addresses (e.g., `"/16"` = 65536 addresses)
    ///
    /// # Examples
    ///
    /// - `--tolerance 512` - Allow up to 512 extra addresses
    /// - `--tolerance /16` - Allow up to 65536 extra addresses (equivalent to a /16 block)
    #[arg(short, long, default_value_t = 0, value_parser = parse_tolerance)]
    tolerance: u64,
    /// Validate that the input is already optimally merged. Exit code 1 if further merges are possible.
    #[arg(long)]
    check: bool,
    /// Show merge statistics (also available as `--verbose`).
    #[arg(long, alias = "verbose")]
    stats: bool,
}

fn normalize_for_check(mut nets: Vec<Ipv4Net>) -> Vec<Ipv4Net> {
    // Check mode must detect any change the merge step would perform, including dropping
    // duplicates. Sorting provides a stable ordering for comparison while preserving the
    // original multiplicity so that repeated CIDRs remain visible as a behavioral change.
    nets.sort_by(|a, b| {
        u32::from(a.addr())
            .cmp(&u32::from(b.addr()))
            .then(a.prefix_len().cmp(&b.prefix_len()))
    });
    nets
}

fn total_addresses(nets: &[Ipv4Net]) -> u128 {
    nets.iter()
        .map(|net| 1u128 << (32 - net.prefix_len()))
        .sum()
}

/// Main entry point for the CIDR merge utility.
///
/// Reads IPv4 CIDR blocks from a file or standard input, merges them into a minimal
/// covering set, and prints the results to standard output (one CIDR per line).
///
/// # Errors
///
/// Returns an error if:
/// - The input file cannot be opened or read
/// - Any line contains an invalid CIDR block
/// - Standard input cannot be read
///
/// # Exit Codes
///
/// - `0` - Success
/// - Non-zero - Error occurred
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let reader: Box<dyn BufRead> = match args.input {
        Some(path) => Box::new(BufReader::new(File::open(path)?)),
        None => Box::new(BufReader::new(io::stdin().lock())),
    };

    let nets =
        parse_ipv4_nets(reader).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let merged = merge_ipv4_nets(nets.clone(), args.tolerance);

    if args.stats {
        let normalized_input = merge_ipv4_nets(nets.clone(), 0);
        let input_total = total_addresses(&normalized_input);
        let merged_total = total_addresses(&merged);
        let reduction = if nets.is_empty() {
            0.0
        } else {
            ((nets.len() as f64 - merged.len() as f64) / nets.len() as f64) * 100.0
        };

        eprintln!("CIDR merge statistics:");
        eprintln!("  Input CIDRs: {}", nets.len());
        eprintln!("  Merged CIDRs: {}", merged.len());
        eprintln!("  Reduction: {:.2}%", reduction);
        eprintln!("  Total addresses (input): {input_total}");
        eprintln!("  Total addresses (merged): {merged_total}");
        let extra_addresses = merged_total.saturating_sub(input_total);
        if extra_addresses > 0 {
            eprintln!("  Extra addresses from tolerance: {extra_addresses}");
        }
    }

    if args.check {
        let normalized_input = normalize_for_check(nets);

        if normalized_input != merged {
            process::exit(1);
        }

        return Ok(());
    }

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
