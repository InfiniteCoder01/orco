mod instr;
pub use instr::Instruction;

/// Id of a variable (index into variables list).
/// It is known that all function arguments have sequential IDs, starting from index 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableId(pub u32);

impl std::fmt::Display for VariableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Info about one variable in a body.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable {
    /// Type of this variable.
    pub ty: crate::Type,
    /// Wether this variable comes from function arguments.
    pub arg: bool,
    /// Debug name.
    pub name: Option<String>,
}

/// A function body.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Body {
    /// All variables used in the body.
    /// Index this with [`orco::codegen::Variable::0`].
    pub variables: Vec<Variable>,
    /// A list of instructions, with values following stack-based order.
    pub instructions: Vec<Instruction>,
}

impl Body {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare_var(&mut self, ty: crate::Type) -> VariableId {
        let id = VariableId(self.variables.len() as _);
        self.variables.push(Variable {
            ty,
            arg: false,
            name: None,
        });
        id
    }

    pub fn var(&self, id: VariableId) -> &Variable {
        self.variables
            .get(id.0 as usize)
            .unwrap_or_else(|| panic!("invalid variable id {id}"))
    }

    pub fn var_mut(&mut self, id: VariableId) -> &mut Variable {
        self.variables
            .get_mut(id.0 as usize)
            .unwrap_or_else(|| panic!("invalid variable id {id}"))
    }

    pub fn var_debug_name(&self, id: VariableId) -> String {
        format!("{}{id}", self.var(id).name.as_deref().unwrap_or("_"))
    }

    pub fn debug_instr(
        &self,
        mut idx: usize,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<usize, std::fmt::Error> {
        match self.instructions[idx] {
            Instruction::Var(id) => write!(f, "{}", self.var_debug_name(id)).map(|_| idx + 1),
            Instruction::Assign => {
                idx = self.debug_instr(idx + 1, f)?;
                write!(f, " = ")?;
                self.debug_instr(idx, f)
            }
            Instruction::Return(has_value) => {
                write!(f, "return")?;
                if has_value {
                    write!(f, " ")?;
                    self.debug_instr(idx + 1, f)
                } else {
                    Ok(idx + 1)
                }
            }

            instr => {
                write!(f, "{instr}")?;
                idx += 1;
                let args = instr.arg_count();
                if args > 0 {
                    write!(f, "(")?;
                    for i in 0..args {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        idx = self.debug_instr(idx, f)?;
                    }
                    write!(f, ")")?;
                }
                Ok(idx)
            }
        }
    }
}

impl std::fmt::Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.variables.is_empty() && self.instructions.is_empty() {
            return write!(f, "{{}}");
        }

        writeln!(f, "{{")?;
        for (idx, var) in self.variables.iter().enumerate() {
            write!(
                f,
                "  let {}: {}",
                self.var_debug_name(VariableId(idx as _)),
                var.ty
            )?;
            if var.arg {
                write!(f, " = <argument>")?;
            }
            write!(f, ";")?;
            writeln!(f)?;
        }

        let mut idx = 0;
        while idx < self.instructions.len() {
            write!(f, "  ")?;
            idx = self.debug_instr(idx, f)?;
            writeln!(f, ";")?;
        }

        write!(f, "}}")
    }
}
