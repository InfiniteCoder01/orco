#![doc = include_str!("../README.md")]
use clap::{Parser, Subcommand};
use orco::diagnostics::ErrorReporter;
use std::io::Read;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Input file, use '-' for stdin
    path: std::path::PathBuf,
}

#[derive(Subcommand)]
pub enum Command {
    /// Build the file
    Build,
    #[cfg(feature = "visual")]
    /// Render IR to a png
    RenderIR,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let mut reporter = orco::diagnostics::DefaultReporter::default();

    let mut krate = if cli.path == std::path::Path::new("-") {
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
    krate.root.register();
    krate
        .root
        .infer_and_check_types(&mut reporter, &krate.root, &orco::Path::new());

    match cli.command {
        Command::Build => {
            if !reporter.has_errors() {
                orco_backend_cranelift::build(&krate.root);
            } else {
                std::process::exit(1);
            }
        }
        Command::RenderIR => {
            let mut flowchart = orco_visual::ir::Flowchart::default();
            let orco::SymbolReference::Function(ref main) =
                krate.root.symbol_map[&orco::Span::new("main")][0]
            else {
                panic!("'main' should be a function")
            };
            flowchart.render_expression(&main.body.lock().unwrap(), 0);
            if let Err(err) = flowchart
                .render()
                .save(orco_visual::ril::ImageFormat::Png, "ir.png")
            {
                log::error!("Failed to save an image: {}", err);
            }
        }
    }
}
