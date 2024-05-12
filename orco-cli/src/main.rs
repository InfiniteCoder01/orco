#![doc = include_str!("../README.md")]
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
    let mut reporter = orco::diagnostics::DefaultReporter;

    let krate = if cli.path == std::path::Path::new("-") {
        let mut source = String::new();
        std::io::stdin().read_to_string(&mut source).unwrap();
        orco_lang::Crate {
            root: orco_lang::parser::parse(&mut orco_lang::lexer::Parser::new(
                &orco_lang::lexer::Source(orco::Src::new(source, "<buffer>".into())),
                &mut reporter,
            )),
        }
    } else {
        orco_lang::Crate::parse(cli.path, &mut reporter)
    };
    krate.root.infer_and_check_types(&mut reporter);
    orco_backend_cranelift::build(&krate.root);
}
