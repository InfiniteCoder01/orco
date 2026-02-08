use crate::symbols::FunctionSignature;
use crate::{BackendContext, SymbolKind};
use orco::codegen as oc;

/// Implementation of [`orco::BodyCodegen`]
pub struct Codegen<'a, B: BackendContext> {
    /// Backend context that will recieve the symbol once codegen is done
    pub ctx: &'a B,
    /// Symbol name
    pub name: orco::Symbol,

    body: String,
    indent: usize,
    variables: Vec<VariableInfo>,
}

struct VariableInfo {
    name: String,
    ty: orco::Type,
}

impl<'a, B: BackendContext> Codegen<'a, B> {
    #[allow(missing_docs)]
    pub fn new(ctx: &'a B, name: orco::Symbol) -> Self {
        let mut this = Self {
            ctx,
            name,

            body: "{\n".to_owned(),
            indent: 1,
            variables: Vec::new(),
        };

        let symbol = this.symbol();
        if let SymbolKind::Function(signature) = Self::skip_generics(symbol.get()) {
            this.body = format!(
                "{} {{\n",
                crate::symbols::FmtFunction {
                    macro_context: ctx.macro_context(),
                    name: &ctx.symname(name), // FIXME: Generics
                    signature,
                    name_all_args: true
                }
            );

            for (idx, (name, ty)) in signature.params.iter().enumerate() {
                let name = name.clone().unwrap_or_else(|| format!("_{idx}"));
                this.variables.push(VariableInfo {
                    ty: ty.clone(),
                    name,
                });
            }
        }

        this
    }

    fn symbol(&self) -> scc::hash_map::OccupiedEntry<'a, orco::Symbol, SymbolKind> {
        self.ctx
            .backend()
            .symbols
            .get_sync(&self.name)
            .unwrap_or_else(|| panic!("trying to codegen undeclared symbol {}", self.name))
    }

    fn skip_generics(mut symbol: &SymbolKind) -> &SymbolKind {
        while let SymbolKind::Generic { symbol: inner, .. } = symbol {
            symbol = inner;
        }
        symbol
    }

    fn indent(&mut self) {
        for _ in 0..self.indent {
            self.body.push_str("  ");
        }
    }

    /// Add a statement
    pub fn stmt(&mut self, statement: &str) {
        for line in statement.split('\n') {
            self.indent();
            self.body.push_str(line);
            self.body.push('\n');
        }
    }

    fn place(&mut self, place: oc::Place) {
        match place {
            oc::Place::Variable(var) => {
                self.body.push_str(&self.variables[var.0].name);
            }
            oc::Place::Global(symbol) => {
                self.body.push_str(&self.ctx.symname(symbol));
            }
            oc::Place::Deref(place) => {
                self.body.push_str("(*");
                self.place(*place);
                self.body.push(')');
            }
            oc::Place::Field(place, _field) => {
                self.place(*place);
                self.body.push('.');
                todo!()
            }
        }
    }

    fn op(&mut self, op: oc::Operand) {
        match op {
            oc::Operand::Place(place) => self.place(place),
            oc::Operand::IConst(value, _) => self.body.push_str(&value.to_string()), // TODO: Size
            oc::Operand::UConst(value, _) => self.body.push_str(&value.to_string()), // TODO: Size
            oc::Operand::FConst(value, _) => self.body.push_str(&value.to_string()), // TODO: Size
        }
    }

    fn place_ty(&self, place: &oc::Place) -> orco::Type {
        match place {
            oc::Place::Variable(variable) => self.variables[variable.0].ty.clone(),
            oc::Place::Global(symbol) => {
                let symbol = self.ctx.backend().symbols.get_sync(symbol);

                match symbol.as_ref().map(|symbol| symbol.get()) {
                    Some(SymbolKind::Function(signature)) => signature.ptr_type(),
                    Some(SymbolKind::Generic { .. }) => todo!(),
                    _ => orco::Type::Error,
                }
            }
            oc::Place::Deref(place) => match self.place_ty(place) {
                _ => orco::Type::Error,
            },
            oc::Place::Field(place, field) => match self.place_ty(place) {
                orco::Type::Struct { fields } => fields
                    .get(*field)
                    .map_or(orco::Type::Error, |(_, ty)| ty.clone()),
                _ => orco::Type::Error,
            },
        }
    }

    fn op_type(&self, op: &oc::Operand) -> orco::Type {
        match op {
            oc::Operand::Place(place) => self.place_ty(place),
            oc::Operand::IConst(_, size) => orco::Type::Integer(*size),
            oc::Operand::UConst(_, size) => orco::Type::Unsigned(*size),
            oc::Operand::FConst(_, size) => orco::Type::Float(*size),
        }
    }
}

impl<B: BackendContext> oc::BodyCodegen for Codegen<'_, B> {
    fn declare_var(&mut self, mut ty: orco::Type) -> oc::Variable {
        let id = self.variables.len();
        let name = format!("_{id}");
        self.ctx.intern_type(&mut ty, false, false);

        if !matches!(&ty, orco::Type::Struct { fields: fields } if fields.is_empty()) {
            self.stmt(&format!(
                "{};",
                crate::types::FmtType {
                    macro_context: false,
                    ty: &ty,
                    name: Some(&name),
                }
            ));
        }

        self.variables.push(VariableInfo { name, ty });
        oc::Variable(id)
    }

    fn arg_var(&self, idx: usize) -> oc::Variable {
        oc::Variable(idx)
    }

    fn assign(&mut self, value: oc::Operand, destination: oc::Place) {}

    fn call(&mut self, function: oc::Operand, args: Vec<oc::Operand>, destination: oc::Place) {
        self.indent();
        if match self.op_type(&function) {
            orco::Type::FnPtr { return_type, .. } => {
                *return_type != orco::Type::Symbol("void".into())
            }
            _ => true,
        } {
            self.place(destination);
            self.body.push_str(" = ");
        }
        self.op(function);
        self.body.push('(');
        for (idx, arg) in args.into_iter().enumerate() {
            if idx > 0 {
                self.body.push_str(", ");
            }
            self.op(arg);
        }
        self.body.push_str(");\n");
    }

    fn return_(&mut self, value: oc::Operand) {}

    fn acf(&mut self) -> &mut impl oc::ACFCodegen {
        self
    }
}

impl<B: BackendContext> oc::ACFCodegen for Codegen<'_, B> {
    fn label(&mut self, label: oc::Label) {}

    fn jump(&mut self, label: oc::Label) {}

    fn cjump(&mut self, lhs: oc::Operand, rhs: u128, equal: bool, label: oc::Label) {}
}

impl<B: BackendContext> std::ops::Drop for Codegen<'_, B> {
    fn drop(&mut self) {
        self.body.push('}');
        self.ctx.define(std::mem::take(&mut self.body));
    }
}
