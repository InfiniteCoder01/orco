use crate::{Backend, SymbolKind};
use orco::codegen as oc;
use std::collections::HashMap;

mod intrinsics;
mod value;
use value::ValueInfo;

/// Implementation of [`orco::BodyCodegen`]
pub struct Codegen<'a, 'b: 'a> {
    /// Backend context that will recieve the symbol once codegen is done
    pub backend: &'a Backend<'b>,
    /// Symbol name
    pub name: orco::Symbol,

    /// Currently generated function body as a string
    body: String,
    /// Current indentation level
    indent: usize,

    /// A variable info list. Variables never get removed,
    /// this can be indexed using [`Variable::0`] directly
    variables: Vec<VariableInfo>,
    /// Number of arguments this function has
    arg_count: usize,

    /// Map of [`Value::0`] to value info. Entries get
    /// removed whenever values get used
    values: HashMap<usize, ValueInfo>,
    next_value_index: usize,
}

struct VariableInfo {
    name: String,
    ty: orco::Type,
}

impl<'a, 'b: 'a> Codegen<'a, 'b> {
    #[allow(missing_docs)]
    pub fn new(ctx: &'a Backend<'b>, name: orco::Symbol) -> Self {
        let mut this = Self {
            backend: ctx,
            name,

            body: "{\n".to_owned(),
            indent: 1,

            variables: Vec::new(),
            arg_count: 0,

            values: HashMap::new(),
            next_value_index: 0,
        };

        let symbol = ctx.get_symbol(this.name);
        let symbol = symbol.get();
        if let SymbolKind::Function(signature) = symbol {
            this.body = format!(
                "{} {{\n",
                crate::symbols::FmtFunction {
                    name: &crate::symname(name), // FIXME: Generics
                    signature: &signature,
                    name_all_args: true
                }
            );

            for (idx, (name, ty)) in signature.params.iter().enumerate() {
                let name = name.clone().unwrap_or_else(|| format!("arg{idx}"));
                this.variables.push(VariableInfo {
                    ty: ty.clone(),
                    name,
                });
            }
            this.arg_count = signature.params.len();
        } else {
            panic!("Trying to define a non-function symbol {symbol:#?}")
        }

        this
    }

    /// Adds indent to the body
    fn indent(&mut self) {
        for _ in 0..self.indent {
            self.body.push_str("  ");
        }
    }

    /// Add a line to the source code
    pub fn line(&mut self, args: std::fmt::Arguments<'_>) {
        self.indent();
        std::fmt::write(&mut self.body, args).unwrap();
        self.body.push('\n');
    }
}

impl oc::BodyCodegen for Codegen<'_, '_> {
    fn comment(&mut self, comment: &str) {
        for line in comment.split('\n') {
            self.line(format_args!("// {line}"));
        }
    }

    fn type_of(&self, id: usize) -> orco::Type {
        self.values[&id].ty.clone()
    }

    fn declare_var(&mut self, mut ty: orco::Type) -> oc::Variable {
        self.backend.intern_type(&mut ty, false);
        let id = self.variables.len();
        let name = format!("var{}", id);

        if !matches!(&ty, orco::Type::Struct { fields } if fields.is_empty()) {
            self.line(format_args!(
                "{};",
                crate::types::FmtType {
                    ty: &ty,
                    constant: false,
                    name: Some(&name),
                }
            ));
        }

        self.variables.push(VariableInfo { name, ty });
        oc::Variable(id)
    }

    fn arg_var(&self, idx: usize) -> oc::Variable {
        assert!(
            idx < self.arg_count,
            "trying to access argument #{idx}, but there are only {} arguments.",
            self.arg_count
        );
        oc::Variable(idx)
    }

    fn assign(&mut self, target: oc::Place, value: oc::Value) {
        let target = self.place(target).expression;
        let value = self.use_value(value).expression;
        self.line(format_args!("{target} = {value};"));
    }

    fn iconst(&mut self, value: i128, size: orco::types::IntegerSize) -> oc::Value {
        self.mk_value(ValueInfo::new(value.to_string(), orco::Type::Integer(size))) // TODO: Literal sizes
    }

    fn uconst(&mut self, value: u128, size: orco::types::IntegerSize) -> oc::Value {
        self.mk_value(ValueInfo::new(
            value.to_string(),
            orco::Type::Unsigned(size),
        )) // TODO: Literal sizes
    }

    fn fconst(&mut self, value: f64, size: u16) -> oc::Value {
        self.mk_value(ValueInfo::new(value.to_string(), orco::Type::Float(size))) // TODO: Literal sizes
    }

    fn bconst(&mut self, value: bool) -> oc::Value {
        self.mk_value(ValueInfo::new(value.to_string(), orco::Type::Bool))
    }

    fn read(&mut self, place: oc::Place) -> oc::Value {
        let place = self.place(place);
        self.mk_value(place)
    }

    fn reference(&mut self, place: oc::Place) -> oc::Value {
        let mut place = self.place(place);
        place.expression.insert(0, '&');
        self.mk_value(place)
    }

    fn call(&mut self, func: oc::Value, args: Vec<oc::Value>) -> Option<oc::Value> {
        let func = self.use_value(func);
        let ty = match func.ty {
            orco::Type::FnPtr {
                params,
                return_type,
            } => {
                assert_eq!(params.len(), args.len());
                return_type
            }
            ty => panic!("trying to call {ty:#?} (which is not a function)"),
        };

        let mut call = func.expression;
        call.push('(');
        for (idx, arg) in args.into_iter().enumerate() {
            let arg = self.use_value(arg);
            if idx > 0 {
                call.push_str(", ");
            }
            call.push_str(&arg.expression);
        }
        call.push(')');

        match ty {
            Some(rt) => Some(self.mk_value(ValueInfo::new(call, *rt))),
            None => {
                self.line(format_args!("{call};"));
                None
            }
        }
    }

    fn return_(&mut self, value: Option<oc::Value>) {
        if let Some(value) = value {
            let value = self.use_value(value).expression;
            self.line(format_args!("return {value};"));
        } else {
            self.line(format_args!("return;"));
        }
    }

    fn intrinsics(&mut self) -> impl oc::Intrinsics + '_ {
        self
    }

    fn acf(&mut self) -> impl oc::ACFCodegen + '_ {
        self
    }
}

impl oc::ACFCodegen for &mut Codegen<'_, '_> {
    fn alloc_label(&mut self) -> oc::Label {
        oc::Label(0)
    }

    fn label(&mut self, label: oc::Label) {
        let _ = label;
    }

    fn jump(&mut self, label: oc::Label) {
        let _ = label;
    }

    fn cjump(&mut self, condition: oc::Value, label: oc::Label) {
        let _ = (condition, label);
    }
}

impl std::ops::Drop for Codegen<'_, '_> {
    fn drop(&mut self) {
        self.body.push('}');
        self.backend.define(std::mem::take(&mut self.body));
    }
}
