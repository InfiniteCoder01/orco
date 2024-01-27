pub mod codebase;
pub mod diagnostic;
pub mod interner;
pub mod ir;

pub use codebase::{Codebase, Symbol};
pub use diagnostic::{Diagnostic, Label, Severity};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Input<'a> {
    File(&'a std::path::Path),
    String(&'a str),
}

pub trait Unit {
    fn parse(input: Input) -> Self
    where
        Self: Sized;

    fn build(&self, others: &std::collections::HashMap<String, Box<dyn Unit>>) -> ir::Module;
    fn get_item(
        &self,
        others: &std::collections::HashMap<String, Box<dyn Unit>>,
        path: &[Symbol],
    ) -> ir::Item;
}

impl Codebase<'_> {
    pub fn parse_path(&mut self, path: &str) -> Vec<Symbol> {
        path.split("::")
            .map(|segment| self.interner.get_or_intern(segment))
            .collect()
    }
}
