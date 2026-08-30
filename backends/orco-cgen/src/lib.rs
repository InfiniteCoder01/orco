//! C transpilation backend for orco.
//! Also used to generate C headers and
//! is generally the reference for other backends.
//! See [FmtModule].
// TODO: ABI
#![warn(missing_docs)]

/// Type formatting & other things.
pub mod types;
use types::FmtType;

/// Symbol formatting stuff.
pub mod symbols;

// /// Code generation, used to generate function bodies.
// pub mod codegen;
// pub use codegen::Codegen;

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
        for (name, alias) in module.types.pin().iter() {
            if matches!(alias.type_, orco::Type::Struct { .. }) {
                let name = cname(*name);
                writeln!(f, "typedef struct {name} {name};")?;
                any = true;
            }
        }

        if any {
            writeln!(f)?;
            any = false;
        }

        topsort::visit(module, |name, ty| {
            any = true;
            writeln!(
                f,
                "typedef {};",
                FmtType {
                    ty: &ty,
                    constant: false,
                    name: Some(&cname(name))
                }
            )
        })?;

        if any {
            writeln!(f)?;
            any = false;
        }

        for (name, function) in module.functions.pin().iter() {
            any = true;
            writeln!(
                f,
                "{};",
                symbols::FmtFunction {
                    name: &cname(*name),
                    function,
                    name_all_args: false,
                }
            )?;
        }

        if any {
            writeln!(f)?;
        }

        Ok(())
    }
}

/// Get the name of the symbol used in generated C code ("mangling")
pub fn cname(name: orco::Symbol) -> String {
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
