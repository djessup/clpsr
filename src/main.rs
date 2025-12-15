use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;

use clpsr::{merge_ipv4_nets, parse_ipv4_nets};

#[derive(Parser, Debug)]
#[command(author, version, about = "CIDR merge utility", long_about = None)]
struct Args {
    /// Optional path to a file containing CIDRs (one per line). If omitted, stdin is used.
    #[arg(short, long)]
    input: Option<PathBuf>,
    /// Maximum number of extra addresses allowed when merging CIDRs. Defaults to 0 (lossless merging only).
    /// When set to N > 0, the algorithm may merge networks even if the resulting supernet covers
    /// addresses outside the original set, as long as the added address count ≤ N.
    #[arg(short, long, default_value_t = 0)]
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
