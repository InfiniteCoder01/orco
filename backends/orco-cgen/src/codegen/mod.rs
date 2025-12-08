use crate::{Backend, FmtType};
use orco::codegen as oc;

/// Info about a variable within [Codegen] session
struct VariableInfo {
    ty: orco::Type,
    /// Variable name, for debugging purpuses
    name: Option<String>,
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

    fn var_name(&self, var: oc::Variable) -> String {
        match &self.var(var).name {
            Some(name) => name.clone(),
            _ => format!("_{}", var.0),
        }
    }

    fn fmt_place(&self, place: oc::Place) -> String {
        match place {
            oc::Place::Variable(variable) => self.var_name(variable),
            oc::Place::Deref(place) => format!("(*{})", self.fmt_place(*place)),
            oc::Place::Field(place, field) => {
                format!("{}.{}", self.fmt_place(*place), crate::escape(field))
            }
        }
    }

    fn op(&self, op: oc::Operand) -> String {
        match op {
            oc::Operand::Global(symbol) => crate::escape(symbol),
            oc::Operand::Place(var) => self.fmt_place(var),
            oc::Operand::IConst(val, _size) => format!("{val}ll"),
            oc::Operand::UConst(val, _size) => format!("{val}ull"),
            oc::Operand::FConst(val, _size) => {
                if val.fract() == 0.0 {
                    format!("{val:.01}")
                } else {
                    val.to_string()
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

impl oc::BodyCodegen<'_> for Codegen<'_> {
    fn external(mut self)
    where
        Self: Sized,
    {
        self.code.clear();
    }

    fn comment(&mut self, comment: &str) {
        for line in comment.split('\n') {
            self.line(&format!("// {line}"));
        }
    }

    fn declare_var(&mut self, mut ty: orco::Type) -> oc::Variable {
        self.backend.intern_type(&mut ty, false, true);
        let var = oc::Variable(self.variables.len());
        self.variables.push(VariableInfo { ty, name: None });

        if self.is_void(&self.var(var).ty) {
            self.comment(&format!("void {};", self.var_name(var)));
            return var;
        }

        self.line(&format!(
            "{};",
            FmtType(&self.var(var).ty, Some(&self.var_name(var))),
        ));
        var
    }

    fn arg_var(&self, idx: usize) -> oc::Variable {
        oc::Variable(idx)
    }

    fn assign(&mut self, value: oc::Operand, destination: oc::Place) {
        if self.is_void(&self.place_ty(&destination)) {
            self.comment(&format!(
                "{name} = {op};",
                name = self.fmt_place(destination),
                op = self.op(value),
            ));
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
        write!(
            codegen.code,
            "{}",
            FmtType(
                ty,
                Some(&name.map(crate::escape).unwrap_or_else(|| format!("_{idx}")))
            ),
        )
        .unwrap();
        codegen.variables.push(VariableInfo {
            ty: ty.clone(),
            name: name.map(crate::escape),
        });
    }

    codegen.code.push(')');
    codegen.code = format!("{} {{\n", FmtType(&sig.return_type, Some(&codegen.code)));
    codegen
}
