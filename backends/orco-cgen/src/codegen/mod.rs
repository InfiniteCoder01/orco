use crate::Backend;
use orco::codegen as oc;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

mod intrinsics;
mod value;
use value::ValueInfo;

/// Implementation of [`oc::BodyCodegen`]
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
    /// this can be indexed using [`oc::Variable::0`] directly
    variables: Vec<VariableInfo>,
    /// Used variable names, for disambiguation
    variable_names: HashSet<String>,
    /// Map of [`oc::Value::0`] to value info. Entries get
    /// removed whenever values get used
    values: HashMap<usize, ValueInfo>,
    next_value_id: usize,
    /// ID of the next label for ACF (see [`orco::codegen::AcfCodegen`]).
    next_label_id: usize,
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
            variable_names: HashSet::new(),
            values: HashMap::new(),
            next_value_id: 0,
            next_label_id: 0,
        };

        let signature = ctx
            .functions
            .get_sync(&this.name)
            .unwrap_or_else(|| panic!("trying to codegen an undeclared function {}", this.name));
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

    fn declare_var(&mut self, mut ty: orco::Type, name: Option<&str>) -> oc::Variable {
        self.backend.intern_type(&mut ty, false);
        let id = self.variables.len();
        let mut name =
            name.map_or_else(|| format!("var{}", id), |name| crate::symname(name.into())); // TODO: Not ideal
        if self.variable_names.contains(&name) {
            for disambiguator in 1.. {
                let disambiguated = format!("{name}{disambiguator}");
                if !self.variable_names.contains(&disambiguated) {
                    name = disambiguated;
                    break;
                }
            }
        }
        self.variable_names.insert(name.clone());

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

    fn reference(&mut self, place: oc::Place, mutable: bool) -> oc::Value {
        let mut place = self.place(place);
        place.expression.insert(0, '&');
        place.ty = orco::Type::Ptr(Box::new(place.ty), mutable);
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

    fn acf(&mut self) -> impl oc::AcfCodegen + '_ {
        self
    }

    fn bcf(&mut self) -> impl oc::BcfCodegen + '_ {
        self
    }
}

impl oc::AcfCodegen for &mut Codegen<'_, '_> {
    fn alloc_label(&mut self) -> oc::Label {
        self.next_label_id += 1;
        oc::Label(self.next_label_id - 1)
    }

    fn label(&mut self, label: oc::Label) {
        writeln!(&mut self.body, "label{}:", label.0).unwrap();
    }

    fn jump(&mut self, label: oc::Label) {
        self.line(format_args!("goto label{};", label.0));
    }

    fn cjump(&mut self, condition: oc::Value, label: oc::Label) {
        let condition = self.use_value(condition).expression;
        self.line(format_args!("if ({condition}) goto label{};", label.0));
    }
}

impl oc::BcfCodegen for &mut Codegen<'_, '_> {
    fn if_(&mut self, condition: oc::Value) {
        let condition = self.use_value(condition).expression;
        self.line(format_args!("if ({condition}) {{"));
        self.indent += 1;
    }

    fn else_(&mut self) {
        self.indent -= 1;
        self.line(format_args!("}} else {{"));
        self.indent += 1;
    }

    fn end(&mut self) {
        self.indent -= 1;
        self.line(format_args!("}}"));
    }

    fn loop_(&mut self) {
        self.line(format_args!("while (true) {{"));
        self.indent += 1;
    }

    fn break_(&mut self) {
        self.line(format_args!("break;"));
    }

    fn continue_(&mut self) {
        self.line(format_args!("continue;"));
    }

    // This is very unnecessary, but I think it looks cleaner :)
    fn cbreak(&mut self, condition: oc::Value) {
        let condition = self.use_value(condition).expression;
        self.line(format_args!("if ({condition}) break;"));
    }

    fn ccontinue(&mut self, condition: oc::Value) {
        let condition = self.use_value(condition).expression;
        self.line(format_args!("if ({condition}) continue;"));
    }
}

impl std::ops::Drop for Codegen<'_, '_> {
    fn drop(&mut self) {
        self.body.push('}');
        self.backend.define(std::mem::take(&mut self.body));
    }
}
