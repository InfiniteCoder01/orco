use crate::{Backend, FmtType};
use orco::codegen as oc;

/// Info about a variable within [Codegen] session
struct VariableInfo {
    ty: orco::Type,
    /// Variable name, for debugging purpuses
    name: String,
}

/// Code generation session of a single function
pub struct Codegen<'a> {
    backend: &'a Backend,
    code: String,
    indent: usize,
    variables: Vec<VariableInfo>,
    ret_void: bool,
}

impl Codegen<'_> {
    fn line(&mut self, line: &str) {
        self.code.reserve(self.indent + line.len() + 1);
        self.code.extend(std::iter::repeat_n(' ', self.indent));
        self.code.push_str(line);
        self.code.push('\n');
    }

    fn var(&self, var: oc::Variable) -> &VariableInfo {
        &self.variables[var.0]
    }

    fn fmt_place(&self, place: oc::Place) -> String {
        match place {
            oc::Place::Variable(variable) => self.var(variable).name.clone(),
            oc::Place::Global(symbol) => crate::escape(symbol),
            oc::Place::Deref(place) => format!("(*{})", self.fmt_place(*place)),
            oc::Place::Field(place, field) => {
                format!("{}.{}", self.fmt_place(*place), crate::escape(field))
            }
        }
    }

    fn op(&self, op: oc::Operand) -> String {
        use oc::Operand as OP;
        use orco::IntegerSize as IS;
        match op {
            OP::Place(var) => self.fmt_place(var),
            OP::IConst(val, IS::Bits(bits)) => format!("INT{bits}_C({val})"),
            OP::IConst(val, IS::Size) => format!("(ssize_t){val}ll"),
            OP::UConst(val, IS::Bits(bits)) => format!("UINT{bits}_C({val})"),
            OP::UConst(val, IS::Size) => format!("(size_t){val}ull"),
            OP::FConst(val, size) => {
                let suffix = match size {
                    32 => "f",
                    64 => "d",
                    _ => "",
                };
                if val.fract() == 0.0 {
                    format!("{val:.01}{suffix}")
                } else {
                    format!("{val}{suffix}")
                }
            }
            oc::Operand::Unit => "(s_) {}".to_owned(),
        }
    }

    fn is_void(&self, ty: &orco::Type) -> bool {
        ty == &orco::Type::Symbol("void".into())
    }

    fn place_ty(&self, place: &oc::Place) -> orco::Type {
        match place {
            oc::Place::Variable(var) => self.var(*var).ty.clone(),
            oc::Place::Global(..) => todo!(),
            oc::Place::Deref(..) => todo!(),
            oc::Place::Field(place, field) => {
                let mut ty = self.place_ty(place);
                while let orco::Type::Symbol(sym) = ty {
                    let sym = match self.backend.symbols.get_sync(&sym) {
                        Some(sym) => sym,
                        None => return orco::Type::Error,
                    };
                    ty = match sym.get() {
                        crate::SymbolKind::Type(ty) => ty.clone(),
                        _ => return orco::Type::Error,
                    };
                }
                match ty {
                    orco::Type::Struct(fields) => fields
                        .iter()
                        .find_map(|(name, ty)| {
                            if field == name {
                                Some(ty.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or(orco::Type::Error),
                    _ => orco::Type::Error,
                }
            }
        }
    }
}

impl oc::BodyCodegen for Codegen<'_> {
    fn external(mut self)
    where
        Self: Sized,
    {
        self.code.clear();
    }

    fn declare_var(&mut self, mut ty: orco::Type) -> oc::Variable {
        self.backend.intern_type(&mut ty, false, true);
        let var = oc::Variable(self.variables.len());
        let name = format!("_{}", var.0);

        if !self.is_void(&ty) {
            self.line(&format!("{};", FmtType(&ty, Some(&name))));
        }

        self.variables.push(VariableInfo { ty, name });
        var
    }

    fn arg_var(&self, idx: usize) -> oc::Variable {
        oc::Variable(idx)
    }

    fn assign(&mut self, value: oc::Operand, destination: oc::Place) {
        if self.is_void(&self.place_ty(&destination)) {
            return;
        }

        self.line(&format!(
            "{name} = {op};",
            name = self.fmt_place(destination),
            op = self.op(value),
        ));
    }

    fn call(&mut self, function: oc::Operand, args: Vec<oc::Operand>, destination: oc::Place) {
        let function = self.op(function);
        let args = args
            .into_iter()
            .map(|arg| self.op(arg))
            .collect::<Vec<_>>()
            .join(", ");

        if self.is_void(&self.place_ty(&destination)) {
            self.line(&format!("{function}({args});"));
            return;
        }

        self.line(&format!(
            "{dst} = {function}({args});",
            dst = self.fmt_place(destination),
        ));
    }

    fn return_(&mut self, value: oc::Operand) {
        if self.ret_void {
            self.line("return;");
            return;
        }
        self.line(&format!("return {op};", op = self.op(value)));
    }
}

impl oc::ACFCodegen for Codegen<'_> {
    fn label(&mut self, label: oc::Label) {
        self.code.push_str(&format!("_{}:\n", label.0));
    }

    fn jump(&mut self, label: oc::Label) {
        self.line(&format!("goto _{};", label.0));
    }

    fn cjump(&mut self, lhs: oc::Operand, rhs: u128, equal: bool, label: oc::Label) {
        if rhs == 0 {
            self.line(&format!(
                "if ({op}{value}) goto _{label};",
                op = if equal { "!" } else { "" },
                value = self.op(lhs),
                label = label.0
            ));
            return;
        }

        self.line(&format!(
            "if ({lhs} {op} {rhs}) goto _{label};",
            lhs = self.op(lhs),
            op = if equal { "==" } else { "!=" },
            label = label.0
        ));
    }
}

impl std::ops::Drop for Codegen<'_> {
    fn drop(&mut self) {
        if self.code.is_empty() {
            return;
        }

        self.code.push('}');
        self.backend.defs.push(std::mem::take(&mut self.code));
    }
}

/// Start codegen for a function
pub fn function<'a>(
    backend: &'a Backend,
    name: orco::Symbol,
    sig: &crate::symbols::FunctionSignature,
) -> Codegen<'a> {
    use std::fmt::Write as _;

    let mut codegen = Codegen {
        backend,
        code: crate::escape(name),
        indent: 4,
        variables: Vec::new(),
        ret_void: sig.return_type == orco::Type::Symbol("void".into()),
    };

    codegen.code.push('(');
    for (idx, (name, ty)) in sig.params.iter().enumerate() {
        if idx > 0 {
            codegen.code.push_str(", ");
        }
        let name = name.map(crate::escape).unwrap_or_else(|| format!("_{idx}"));
        write!(codegen.code, "{}", FmtType(ty, Some(&name)),).unwrap();
        codegen.variables.push(VariableInfo {
            ty: ty.clone(),
            name,
        });
    }

    codegen.code.push(')');
    codegen.code = format!("{} {{\n", FmtType(&sig.return_type, Some(&codegen.code)));
    codegen
}
