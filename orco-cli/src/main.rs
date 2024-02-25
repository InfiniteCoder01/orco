use std::path::Path;

fn main() {
    let mut codebase = orco::Codebase::default();
    let krate = Path::new("crates/orco-lang/samples/simple.orco");
    orco_lang::Crate::parse(krate, &codebase);
    // let krate = Path::new("crates/orco-rust/samples/simple.rs");
    // let _krate = codebase.add(Box::new(orco_rust::Crate::parse(krate, &codebase)));
    // codebase.visit_items(|path, item| println!("{:?}: {:#?}", path, item));
}
