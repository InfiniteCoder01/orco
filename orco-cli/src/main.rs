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
    // env_logger::init();
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
    let cli = Cli::parse();

    let unit: Result<orco_c::Unit, _> = if cli.path == std::path::Path::new("-") {
        let mut source = String::new();
        std::io::stdin().read_to_string(&mut source).unwrap();
        orco_c::parsel::parse_str(&source)
    } else {
        orco_c::parsel::parse_str(&std::fs::read_to_string(cli.path).unwrap())
    };
    let unit = match unit {
        Ok(unit) => unit,
        Err(err) => {
            println!("{}", err);
            panic!();
        }
    };

    let mut ctx = orco::TypeInferenceContext::new();
    let symbols = unit.build(&mut ctx);
    for (name, symbol) in symbols {
        println!(
            "const {} = {}{}",
            name,
            symbol,
            if !matches!(symbol, orco::Expression::Function(_)) {
                ";"
            } else {
                ""
            }
        );
    }

    // println!("{}", &unit as &dyn orco::Unit);
    // orco_cranelift::build(
    //     &unit
    //         .symbols
    //         .iter()
    //         .map(orco_c::Symbol::as_orco)
    //         .collect::<Vec<_>>(),
    // );
}
