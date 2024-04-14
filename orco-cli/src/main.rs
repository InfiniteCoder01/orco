use std::path::Path;

fn main() {
    env_logger::init();
    let krate = Path::new("orco-lang/samples/simple.orco");
    orco_lang::Crate::parse(krate);
    // let krate = Path::new("crates/orco-rust/samples/simple.rs");
    // let _krate = codebase.add(Box::new(orco_rust::Crate::parse(krate, &codebase)));
    // codebase.visit_items(|path, item| println!("{:?}: {:#?}", path, item));
}
