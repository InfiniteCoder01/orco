use std::collections::HashMap;

use orco::ir::Instr;

mod instructions;
mod intrinsics;
mod place_vars;

/// Context for code generation.
pub struct Context<'a> {
    /// Reference to a module to reference other symbols.
    pub module: &'a orco::Module,
    /// Body to get the code from.
    pub body: &'a orco::Body,
    /// Actual disambiguated C-compatible variable names.
    var_names: Vec<String>,
    /// Actual disambiguated C-compatible label names.
    label_names: Vec<String>,

    /// The instruction to place variable decl in front of.
    /// See [`place_vars::compute_placement`]
    var_places: HashMap<usize, Vec<orco::ir::VariableId>>,
}

/// Makes all names unique by appending suffixes.
fn disambiguate(names: &mut Vec<String>) {
    let mut idx = HashMap::<String, Option<u32>>::new();
    for name in &mut *names {
        if name.is_empty() {
            continue;
        }

        if let Some(count) = idx.get_mut(name) {
            *count = Some(0);
        } else {
            idx.insert(name.clone(), None);
        }
    }

    use std::fmt::Write;
    for name in names {
        if let Some(idx) = idx.get_mut(name).and_then(Option::as_mut) {
            write!(name, "{idx}").unwrap();
            *idx += 1;
        }
    }
}

impl<'a> Context<'a> {
    #[allow(missing_docs)]
    pub fn new(module: &'a orco::Module, body: &'a orco::Body) -> Self {
        // Var names
        let mut var_names = Vec::with_capacity(body.variables.len());
        for (idx, var) in body.variables.iter().enumerate() {
            if crate::ty::is_unit(&var.ty) {
                var_names.push(String::new());
                continue;
            }
            var_names.push(match &var.name {
                Some(name) => crate::cname(name.into()),
                None if var.arg => format!("arg{idx}"),
                None => "var".to_owned(),
            });
        }

        disambiguate(&mut var_names);

        // Label names
        let mut label_names = Vec::with_capacity(body.label_names.len());
        for label in body.label_names.iter() {
            label_names.push(match &label {
                Some(name) => crate::cname(name.into()),
                None => "label".to_owned(),
            });
        }

        disambiguate(&mut label_names);

        Self {
            module,
            body,
            var_names,
            label_names,

            var_places: place_vars::compute_placement(body),
        }
    }
}

impl std::fmt::Display for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self
            .body
            .variables
            .iter()
            .all(|var| crate::ty::is_unit(&var.ty))
            && self.body.instructions.is_empty()
        {
            return write!(f, "{{}}");
        }

        writeln!(f, "{{")?;
        let mut idx = 0;
        while idx < self.body.instructions.len() {
            let instr = self.body.instructions[idx];

            // Label
            if matches!(instr, orco::ir::Instr::AcfLabel(..)) {
                idx = self.instr(f, idx, 0)?;
                writeln!(f)?;
                continue;
            }

            // Variables
            idx = place_vars::instr(f, self, idx)?;
        }

        write!(f, "}}")
    }
}
