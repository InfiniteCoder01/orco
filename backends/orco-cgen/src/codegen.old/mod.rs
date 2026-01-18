use crate::{BackendContext, FmtType};
use orco::codegen as oc;

pub struct Body {
    statements: Vec<Statement>,
}

/// Code generation session of a single function
pub(super) struct Codegen<'a, B: BackendContext> {
    backend: &'a B,

    name: orco::Symbol,
    signature: crate::symbols::FunctionSignature,
    variables: Vec<VariableInfo>,

    code: String,
    indent: usize,
}

/// Info about a variable within [Codegen] session
struct VariableInfo {
    ty: orco::Type,
    /// Variable name, for debugging purpuses
    name: String,
}

impl<'a, B: BackendContext> Codegen<'a, B> {
    pub(super) fn new(
        backend: &'a B,
        name: orco::Symbol,
        signature: crate::symbols::FunctionSignature,
    ) -> Self {
        let mut variables = Vec::new();
        for (idx, (name, ty)) in signature.params.iter().enumerate() {
            let name = name
                .map(|sym| backend.backend().escape(sym, false)) // TODO: FIXME
                .unwrap_or_else(|| format!("_{idx}"));
            variables.push(VariableInfo {
                ty: ty.clone(),
                name,
            });
        }

        Self {
            backend,
            name,
            signature,
            variables,
            code: "{\n".to_owned(),
            indent: 4,
        }
    }

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
            oc::Place::Global(symbol) => self.backend.backend().escape(symbol, false), // TODO: FIXME
            oc::Place::Deref(place) => format!("(*{})", self.fmt_place(*place)),
            oc::Place::Field(place, field) => {
                format!(
                    "{}.{}",
                    self.fmt_place(*place),
                    self.backend.backend().escape(field, false) // TODO: FIXME
                )
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
                    let sym = match self.backend.backend().symbols.get_sync(&sym) {
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

impl<B: BackendContext> oc::BodyCodegen for Codegen<'_, B> {
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
            self.line(&format!(
                "{};",
                FmtType {
                    backend: self.backend.backend(),
                    macro_context: false, // TODO: FIXME
                    ty: &ty,
                    name: Some(&name)
                }
            ));
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
        if self.is_void(&self.signature.return_type) {
            self.line("return;");
            return;
        }
        self.line(&format!("return {op};", op = self.op(value)));
    }

    fn acf(&mut self) -> &mut impl oc::ACFCodegen {
        self
    }
}

impl<B: BackendContext> oc::ACFCodegen for Codegen<'_, B> {
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

impl<B: BackendContext> std::ops::Drop for Codegen<'_, B> {
    fn drop(&mut self) {
        let body = if self.code.is_empty() {
            None
        } else {
            self.code.push('}');
            Some(std::mem::take(&mut self.code))
        };

        self.backend.symbol(
            self.name,
            crate::SymbolKind::Function {
                signature: std::mem::replace(
                    &mut self.signature,
                    crate::symbols::FunctionSignature {
                        params: Vec::new(),
                        return_type: orco::Type::Error,
                    },
                ),
                body,
            },
        );
    }
}
