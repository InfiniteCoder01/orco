mod instr;
pub use instr::Instruction as Instr;

mod variable;
pub use variable::{VariableId, VariableInfo};

mod symbol_ref;
pub use symbol_ref::{SymbolId, SymbolUse};

mod label;
pub use label::LabelId;

mod intrinsics;
pub use intrinsics::Intrinsic;

/// A function body.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Body {
    /// All variables used in the body.
    /// Index this with [`VariableId::0`].
    pub variables: Vec<VariableInfo>,
    /// All symbols referenced in this body.
    pub symbols: Vec<SymbolUse>,
    /// Reverse map of all used symbols inside [`Self::symbols`]
    /// to their respecive [`SymbolId`] for quick interning.
    interned_symbols: std::collections::HashMap<(crate::Symbol, Vec<crate::Type>), SymbolId>,
    /// Debug names attached to labels.
    /// Index this with [`LabelId::0`].
    pub label_names: Vec<Option<String>>,
    /// A list of instructions, with values following inverse stack-based order.
    /// See [`Instr`]
    pub instructions: Vec<Instr>,
}

impl Body {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get type of a value generated at index.
    /// Requires module access for global symbols.
    pub fn value_ty(&self, idx: usize) -> crate::Type {
        use crate::Type;
        match self.instructions[idx] {
            Instr::IConst(_, size) => Type::Integer(size),
            Instr::UConst(_, size) => Type::Unsigned(size),
            Instr::FConst(_, size) => Type::Float(size),
            Instr::BConst(_) => Type::Bool,

            Instr::Global(id) => self.symbol(id).ty.clone(),
            Instr::Var(id) => self.var(id).ty.clone(),
            Instr::Field(field_idx) => {
                let ty = self.value_ty(idx + 1);
                let Type::Struct { mut fields } = ty else {
                    panic!("trying to access field #{field_idx} on a non-struct type {ty}");
                };
                fields.swap_remove(field_idx as _).1
            }
            Instr::Assign => Type::Error,

            Instr::AcfLabel(..) | Instr::AcfJump(..) | Instr::AcfCJump(..) => Type::Error,
            Instr::Call(..) => {
                let ty = self.value_ty(idx + 1);
                let Type::FnPtr { return_type, .. } = ty else {
                    panic!("trying to call a non-function of type {ty}");
                };
                return_type.map_or(Type::Error, |ty| *ty)
            }
            Instr::Intrinsic(intr) => intr
                .type_override()
                .unwrap_or_else(|| self.value_ty(idx + 1)),
            Instr::Return(..) => Type::Error,
            Instr::Error => Type::Error,
        }
    }

    /// Debug-print an instruction at `idx` with it's arguments into `f`
    pub fn debug_instr(
        &self,
        mut idx: usize,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<usize, std::fmt::Error> {
        let debug_args = move |mut idx, f: &mut std::fmt::Formatter<'_>, args| {
            write!(f, "(")?;
            for i in 0..args {
                if i > 0 {
                    write!(f, ", ")?;
                }
                idx = self.debug_instr(idx, f)?;
            }
            write!(f, ")")?;
            Ok(idx)
        };

        match self.instructions[idx] {
            Instr::Global(id) => write!(f, "{}", self.symbol(id)).map(|_| idx + 1),
            Instr::Var(id) => write!(f, "{}", self.var_debug_name(id)).map(|_| idx + 1),
            Instr::Assign => {
                idx = self.debug_instr(idx + 1, f)?;
                write!(f, " = ")?;
                self.debug_instr(idx, f)
            }
            Instr::Return(has_value) => {
                write!(f, "return")?;
                if has_value {
                    write!(f, " ")?;
                    self.debug_instr(idx + 1, f)
                } else {
                    Ok(idx + 1)
                }
            }

            Instr::Field(field_idx) => {
                let ty = self.value_ty(idx + 1);
                idx = self.debug_instr(idx + 1, f)?;
                let crate::Type::Struct { fields } = ty else {
                    panic!("trying to access field #{field_idx} on a non-struct type {ty}");
                };

                match &fields[field_idx as usize].0 {
                    Some(name) => write!(f, ".{name}")?,
                    None => write!(f, ".field_{field_idx}")?,
                }

                Ok(idx)
            }

            Instr::AcfLabel(label) => {
                write!(f, "{}:", self.label_debug_name(label)).map(|_| idx + 1)
            }
            Instr::AcfJump(label) => {
                write!(f, "jump {}", self.label_debug_name(label)).map(|_| idx + 1)
            }
            Instr::AcfCJump(label) => {
                write!(f, "if ")?;
                idx = self.debug_instr(idx + 1, f)?;
                write!(f, " jump {}", self.label_debug_name(label))?;
                Ok(idx)
            }

            Instr::Call(args) => {
                idx = self.debug_instr(idx + 1, f)?;
                debug_args(idx, f, args)
            }

            Instr::Intrinsic(intr) if intr.infix() => {
                write!(f, "(")?;
                for i in 0..intr.arg_count() {
                    if i > 0 {
                        write!(f, " {intr} ")?;
                    }
                    idx = self.debug_instr(idx, f)?;
                }
                write!(f, ")")?;
                Ok(idx)
            }

            instr => {
                write!(f, "{instr}")?;
                idx += 1;
                let args = instr.arg_count();
                if args > 0 {
                    idx = debug_args(idx, f, args)?;
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
            if matches!(self.instructions[idx], Instr::AcfLabel(..)) {
                idx = self.debug_instr(idx, f)?;
                writeln!(f)?;
                continue;
            }

            write!(f, "  ")?;
            idx = self.debug_instr(idx, f)?;
            writeln!(f, ";")?;
        }

        write!(f, "}}")
    }
}
