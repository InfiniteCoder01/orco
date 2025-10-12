use crate::Backend;
use orco::codegen as oc;

/// Info about a variable within [Codegen] session
struct VariableInfo {
    ty: crate::Type,
}

/// Hidden struct for code generation session of a single function
struct Codegen<'a> {
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
        format!("_{}", var.0)
    }

    fn op(&self, op: oc::Operand) -> String {
        match op {
            oc::Operand::Global(symbol) => crate::escape(symbol),
            oc::Operand::Variable(var) => self.var_name(var),
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

    fn is_void(&self, op: oc::Operand) -> bool {
        match op {
            oc::Operand::Variable(var) => self.var(var).ty.is_void(),
            oc::Operand::Unit => true,
            _ => false,
        }
    }
}

impl oc::Codegen<'_> for Codegen<'_> {
    fn comment(&mut self, comment: &str) {
        for line in comment.split('\n') {
            self.line(&format!("// {line}"));
        }
    }

    fn declare_var(&mut self, ty: &orco::Type) -> oc::Variable {
        let ty = self.backend.convert_type(ty);
        let var = oc::Variable(self.variables.len());
        self.variables.push(VariableInfo { ty });

        if self.var(var).ty.is_void() {
            return var;
        }

        self.line(&format!(
            "{ty} {name};",
            ty = &self.var(var).ty,
            name = self.var_name(var)
        ));
        var
    }

    fn arg_var(&self, idx: usize) -> oc::Variable {
        oc::Variable(idx)
    }

    fn assign(&mut self, value: oc::Operand, destination: oc::Variable) {
        if self.is_void(value) {
            self.comment(&format!(
                "{name} = {op};",
                name = self.var_name(destination),
                op = self.op(value),
            ));
        }
        self.line(&format!(
            "{name} = {op};",
            name = self.var_name(destination),
            op = self.op(value),
        ));
    }

    fn call(&mut self, function: oc::Operand, args: Vec<oc::Operand>, destination: oc::Variable) {
        let function = self.op(function);
        let args = args
            .into_iter()
            .map(|arg| self.op(arg))
            .collect::<Vec<_>>()
            .join(", ");
        if self.var(destination).ty.is_void() {
            self.line(&format!("{function}({args});"));
        } else {
            self.line(&format!(
                "{name} = {function}({args});",
                name = self.var_name(destination),
            ));
        }
    }

    fn return_(&mut self, value: oc::Operand) {
        if self.is_void(value) {
            self.line("return;");
        } else {
            self.line(&format!("return {op};", op = self.op(value)));
        }
    }
}

impl orco::DefinitionBackend for Backend {
    fn define_function(&self, name: orco::Symbol) -> impl oc::Codegen<'_> {
        use std::fmt::Write;
        let decl = self
            .decls
            .get_sync(&name)
            .unwrap_or_else(|| panic!("tried to define undeclared function '{name}'"));
        let crate::DeclarationKind::Function(sig) = &decl.kind else {
            panic!("'{name}' is not a function");
        };

        let mut codegen = Codegen {
            backend: self,
            code: format!("{ret} {name}(", ret = sig.ret, name = decl.name),
            indent: 4,
            variables: Vec::new(),
        };

        for (idx, (ty, _)) in sig.params.iter().enumerate() {
            if idx > 0 {
                codegen.code.push_str(", ");
            }
            write!(codegen.code, "{ty} _{idx}",).unwrap();
            codegen.variables.push(VariableInfo { ty: ty.clone() });
        }
        codegen.code.push_str(") {\n");
        codegen
    }
}

impl std::ops::Drop for Codegen<'_> {
    fn drop(&mut self) {
        self.code.push_str("}");
        self.backend.defs.push(std::mem::take(&mut self.code));
    }
}
