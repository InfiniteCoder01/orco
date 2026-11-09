mod instr;
pub use instr::Instruction as Instr;

mod variable;
pub use variable::{VariableId, VariableInfo};

mod label;
pub use label::LabelId;

mod intrinsics;
pub use intrinsics::Intrinsic;

/// A function body.
/// See also [`FmtBody`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Body {
    /// A list of instructions, with values following inverse stack-based order.
    /// See [`Instr`]
    pub instructions: Vec<Instr>,

    /// All variables used in the body.
    /// Index this with [`VariableId::0`].
    pub variables: Vec<VariableInfo>,

    /// Debug names attached to labels.
    /// Index this with [`LabelId::0`].
    pub label_names: Vec<Option<String>>,
}

impl Body {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get type of a value generated at index.
    pub fn value_ty(&self, module: &crate::Module, idx: usize) -> crate::Type {
        use crate::Type;
        match &self.instructions[idx] {
            Instr::Unit => Type::Unit,
            Instr::IConst(_, size) => Type::Integer(*size),
            Instr::UConst(_, size) => Type::Unsigned(*size),
            Instr::FConst(_, size) => Type::Float(*size),
            Instr::BConst(_) => Type::Bool,
            Instr::CConst(_, utf8) => Type::Char(*utf8),

            Instr::Global(id, generics) => {
                let mut ty = module.symbols[&id].ty();
                ty.instantiate(&generics);
                ty
            }
            Instr::Var(id) => self.var(*id).ty.clone(),
            Instr::Field(field_idx) => {
                let ty = module.inline_ty(self.value_ty(module, idx + 1));
                let Type::Struct { mut fields } = ty else {
                    panic!(
                        "trying to access field #{field_idx} on a non-struct type {}",
                        module.display(&ty)
                    );
                };
                fields.swap_remove(*field_idx as _).1
            }
            Instr::Assign => Type::Error,

            Instr::AcfLabel(..) | Instr::AcfJump(..) | Instr::AcfCJump(..) => Type::Error,
            Instr::Call(..) => {
                let ty = module.inline_ty(self.value_ty(module, idx + 1));
                let Type::FnPtr { types } = ty else {
                    panic!(
                        "trying to call a non-function of type {}",
                        module.display(&ty)
                    );
                };
                types.into_iter().next().unwrap()
            }
            Instr::Intrinsic(intr) => intr
                .type_override()
                .unwrap_or_else(|| self.value_ty(module, idx + 1)),
            Instr::Return => Type::Error,

            Instr::Cast(ty) => ty.as_ref().clone(),
            Instr::Transmute(ty) => ty.as_ref().clone(),

            Instr::Error => Type::Error,
        }
    }

    /// Skip the instruction at `idx` and return the new index.
    pub fn skip_instr(&self, mut idx: usize) -> usize {
        let instr = &self.instructions[idx];
        idx += 1;
        for _ in 0..instr.arg_count() {
            idx = self.skip_instr(idx);
        }
        idx
    }
}

impl crate::Display<'_, Body> {
    /// Debug-print an instruction at `idx` with it's arguments into `f`.
    pub fn debug_instr(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        mut idx: usize,
    ) -> Result<usize, std::fmt::Error> {
        let debug_args = move |mut idx, f: &mut std::fmt::Formatter<'_>, args| {
            write!(f, "(")?;
            for i in 0..args {
                if i > 0 {
                    write!(f, ", ")?;
                }
                idx = self.debug_instr(f, idx)?;
            }
            write!(f, ")")?;
            Ok(idx)
        };

        match &self.instructions[idx] {
            Instr::Unit => write!(f, "unit").map(|_| idx + 1),
            Instr::IConst(value, size) => write!(f, "{value}_i{size}").map(|_| idx + 1),
            Instr::UConst(value, size) => write!(f, "{value}_u{size}").map(|_| idx + 1),
            Instr::FConst(value, size) => write!(f, "{value}_f{size}").map(|_| idx + 1),
            Instr::BConst(value) => write!(f, "{value}").map(|_| idx + 1),
            Instr::CConst(value, utf8) => match char::from_u32(*value) {
                Some(chr) => write!(f, "{:?}", chr),
                None => write!(f, "{}({value})", self.project(&crate::Type::Char(*utf8))),
            }
            .map(|_| idx + 1),

            Instr::Global(id, generics) => write!(
                f,
                "{}{}",
                self.project(id),
                self.project(generics.as_slice())
            )
            .map(|_| idx + 1),
            Instr::Var(id) => write!(f, "{}", self.var_debug_name(*id)).map(|_| idx + 1),
            Instr::Assign => {
                idx = self.debug_instr(f, idx + 1)?;
                write!(f, " = ")?;
                self.debug_instr(f, idx)
            }
            Instr::Return => {
                write!(f, "return ")?;
                self.debug_instr(f, idx + 1)
            }

            Instr::Field(field_idx) => {
                let ty = self.module.inline_ty(self.value_ty(self.module, idx + 1));
                idx = self.debug_instr(f, idx + 1)?;
                let crate::Type::Struct { fields } = ty else {
                    panic!(
                        "trying to access field #{field_idx} on a non-struct type {}",
                        self.project(&ty)
                    );
                };

                match &fields[*field_idx as usize].0 {
                    Some(name) => write!(f, ".{name}")?,
                    None => write!(f, ".field_{field_idx}")?,
                }

                Ok(idx)
            }

            Instr::AcfLabel(label) => {
                write!(f, "{}:", self.label_debug_name(*label)).map(|_| idx + 1)
            }
            Instr::AcfJump(label) => {
                write!(f, "jump {}", self.label_debug_name(*label)).map(|_| idx + 1)
            }
            Instr::AcfCJump(label) => {
                write!(f, "if ")?;
                idx = self.debug_instr(f, idx + 1)?;
                write!(f, " jump {}", self.label_debug_name(*label))?;
                Ok(idx)
            }

            Instr::Call(args) => {
                idx = self.debug_instr(f, idx + 1)?;
                debug_args(idx, f, *args)
            }

            Instr::Intrinsic(intr) if intr.infix() => {
                idx += 1;
                write!(f, "(")?;
                for i in 0..intr.arg_count() {
                    if i > 0 {
                        write!(f, " {intr} ")?;
                    }
                    idx = self.debug_instr(f, idx)?;
                }
                write!(f, ")")?;
                Ok(idx)
            }

            Instr::Intrinsic(intr) => {
                write!(f, "{intr}")?;
                match intr.arg_count() {
                    0 => Ok(idx + 1),
                    1 => self.debug_instr(f, idx + 1),
                    args => debug_args(idx + 1, f, args),
                }
            }

            Instr::Cast(ty) => {
                write!(f, "cast<{}>", self.project(ty.as_ref()))?;
                debug_args(idx + 1, f, 1)
            }

            Instr::Transmute(ty) => {
                write!(f, "transmute<{}>", self.project(ty.as_ref()))?;
                debug_args(idx + 1, f, 1)
            }

            Instr::Error => write!(f, "error").map(|_| idx + 1),
        }
    }
}

impl std::fmt::Display for crate::Display<'_, Body> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.instructions.is_empty() {
            return write!(f, "{{}}");
        }

        writeln!(f, "{{")?;
        for (idx, var) in self.variables.iter().enumerate() {
            write!(
                f,
                "  let {}: {}",
                self.var_debug_name(VariableId(idx as _)),
                self.project(&var.ty)
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
                idx = self.debug_instr(f, idx)?;
                writeln!(f)?;
                continue;
            }

            write!(f, "  ")?;
            idx = self.debug_instr(f, idx)?;
            writeln!(f, ";")?;
        }

        write!(f, "}}")
    }
}
