//! C transpilation backend for orco.
//! Also used to generate C headers and
//! is generally the reference for other backends.
//! See [FmtModule].
// TODO: ABI
#![warn(missing_docs)]

/// Type formatting & other things.
pub mod ty;
use ty::FmtType;

/// Symbol formatting stuff.
pub mod symbols;

/// Code generation, used to generate function bodies.
pub mod codegen;
pub use codegen::Context;

/// Topologically sorts types in a module.
mod topsort;

/// Generate C code for one [`orco::Module`].
pub struct FmtModule<'a>(pub &'a orco::Module);

impl std::fmt::Display for FmtModule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FmtModule(module) = self;

        writeln!(f, "#include <stdint.h>")?;
        writeln!(f, "#include <stddef.h>")?;
        writeln!(f, "#include <stdbool.h>")?;
        writeln!(f)?;

        let mut any = false;
        for (id, alias) in &module.types {
            if matches!(alias.ty, orco::Type::Struct { .. }) {
                let name = ty_name(*id, alias);
                writeln!(f, "typedef struct {name} {name};")?;
                any = true;
            }
        }

        if any {
            writeln!(f)?;
            any = false;
        }

        topsort::visit(module, |id, alias| {
            any = true;
            writeln!(
                f,
                "typedef {};",
                FmtType {
                    module,
                    ty: &alias.ty,
                    constant: false,
                    name: Some(&ty_name(id, alias))
                }
            )
        })?;

        if any {
            writeln!(f)?;
            any = false;
        }

        for (id, symbol) in &module.symbols {
            any = true;
            let name = sym_name(*id, symbol);
            match symbol {
                orco::Symbol::Function(func) => {
                    writeln!(
                        f,
                        "{};",
                        symbols::FmtFunction {
                            module,
                            name: &name,
                            function: &func,
                            name_all_args: false,
                        }
                    )?;
                }
            }
        }

        if any {
            writeln!(f)?;
        }

        for (id, symbol) in &module.symbols {
            let orco::Symbol::Function(func) = symbol;
            let Some(body) = &func.body else {
                continue;
            };

            writeln!(
                f,
                "{} {}\n",
                symbols::FmtFunction {
                    module,
                    name: &sym_name(*id, symbol),
                    function: &func,
                    name_all_args: true,
                },
                codegen::Context::new(self.0, body)
            )?;
        }

        Ok(())
    }
}

/// Get the name of the symbol used in generated C code ("mangling")
pub fn cname(name: &str) -> String {
    // Take only the method name, not the path
    // FIXME: conflicts...
    let mut new_name = String::new();
    for split in name.split([',', '<', '>', '{', '}']) {
        let split = &split[split.rfind([':', '.']).map_or(0, |i| i + 1)..];
        if !split.is_empty() {
            match new_name.chars().last() {
                None | Some('_') => (),
                _ => new_name.push('_'),
            }
            new_name.push_str(split);
        }
    }

    let mut new_name = new_name.replace(|c: char| !c.is_ascii_alphanumeric(), "_");
    if new_name.chars().next().is_none_or(|c| c.is_ascii_digit()) {
        new_name.insert(0, '_');
    }

    new_name
}

/// Get name for a type.
pub fn ty_name(id: orco::ty::AliasId, alias: &orco::ty::Alias) -> String {
    match &alias.name {
        Some(name) => cname(name),
        None => format!("_ty{}", id.0),
    }
}

/// Get name for a symbol.
pub fn sym_name(id: orco::SymbolId, symbol: &orco::Symbol) -> String {
    match symbol.name() {
        Some(name) => cname(name),
        None => format!("_ty{}", id.0),
    }
}
