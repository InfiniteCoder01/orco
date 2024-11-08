// #![doc = include_str!("../README.md")]
use clap::Parser;
use std::io::Read;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input file, use '-' for stdin
    path: std::path::PathBuf,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    let unit: orco_c::Unit = if cli.path == std::path::Path::new("-") {
        let mut source = String::new();
        std::io::stdin().read_to_string(&mut source).unwrap();
        orco_c::parsel::parse_str(&source).unwrap()
    } else {
        orco_c::parsel::parse_str(&std::fs::read_to_string(cli.path).unwrap()).unwrap()
    };

    orco_cranelift::build(&unit);
}
