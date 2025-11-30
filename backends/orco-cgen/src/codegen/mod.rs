use crate::{Backend, FmtType};
use orco::codegen as oc;

/// Info about a variable within [Codegen] session
struct VariableInfo {
    ty: orco::Type,
    /// Variable name, for debugging purpuses
    name: Option<String>,
}

fn is_void(ty: &orco::Type) -> bool {
    match ty {
        orco::Type::Symbol(sym) => *sym == "void",
        _ => false,
    }
}

/// Code generation session of a single function
pub struct Codegen<'a> {
    backend: &'a Backend,
    code: String,
    indent: usize,
    variables: Vec<VariableInfo>,
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
            oc::Operand::Unit => "<unit operand>".to_owned(),
        }
    }

    fn is_void(&self, place: &oc::Place) -> bool {
        match place {
            oc::Place::Variable(var) => is_void(&self.var(*var).ty),
            // oc::Place::Deref(place) => todo!(),
            // oc::Place::Field(place, istr) => todo!(),
            _ => false,
        }
    }

    fn is_op_void(&self, op: &oc::Operand) -> bool {
        match op {
            oc::Operand::Place(place) => self.is_void(place),
            oc::Operand::Unit => true,
            _ => false,
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

    fn declare_var(&mut self, ty: orco::Type) -> oc::Variable {
        let var = oc::Variable(self.variables.len());
        self.variables.push(VariableInfo { ty, name: None });

        if self.var(var).ty == orco::Type::Symbol("void".into()) {
            return var;
        }

        self.line(&format!(
            "{ty} {name};",
            ty = FmtType(&self.var(var).ty),
            name = self.var_name(var)
        ));
        var
    }

    fn arg_var(&self, idx: usize) -> oc::Variable {
        oc::Variable(idx)
    }

    fn assign(&mut self, value: oc::Operand, destination: oc::Place) {
        if self.is_op_void(&value) {
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
        if self.is_void(&destination) {
            self.line(&format!("{function}({args});"));
        } else {
            self.line(&format!(
                "{dst} = {function}({args});",
                dst = self.fmt_place(destination),
            ));
        }
    }

    fn return_(&mut self, value: oc::Operand) {
        if self.is_op_void(&value) {
            self.line("return;");
        } else {
            self.line(&format!("return {op};", op = self.op(value)));
        }
    }
}

impl std::ops::Drop for Codegen<'_> {
    fn drop(&mut self) {
        if self.code.is_empty() {
            return;
        }

        self.code.push_str("}");
        self.backend.defs.push(std::mem::take(&mut self.code));
    }
}

/// Start codegen for a function
pub fn function<'a, 'b>(
    backend: &'a Backend,
    name: orco::Symbol,
    sig: &'b crate::symbols::FunctionSignature,
) -> Codegen<'a> {
    use std::fmt::Write;

    let mut codegen = Codegen {
        backend,
        code: format!(
            "{ret} {name}(",
            ret = FmtType(&sig.return_type),
            name = crate::escape(name)
        ),
        indent: 4,
        variables: Vec::new(),
    };

    for (idx, (name, ty)) in sig.params.iter().enumerate() {
        if idx > 0 {
            codegen.code.push_str(", ");
        }
        write!(
            codegen.code,
            "{ty} {name}",
            ty = FmtType(ty),
            name = name.map(crate::escape).unwrap_or_else(|| format!("_{idx}"))
        )
        .unwrap();
        codegen.variables.push(VariableInfo {
            ty: ty.clone(),
            name: name.map(crate::escape),
        });
    }

    codegen.code.push_str(") {\n");
    codegen
}
