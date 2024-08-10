use super::*;

/// Scopes & Symbol mapping
pub mod scopes;
pub use scopes::Scope;

/// Type variable ID, used for type inference with Hindley–Milner algorithm
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeVariableId(u64);

impl std::fmt::Display for TypeVariableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<type variable #{}>", self.0)
    }
}

/// Type inference information for a function
#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct TypeInference<'a> {
    /// Return type of a function
    pub return_type: Option<diagnostics::Spanned<ir::Type>>,
    /// Error reporter
    #[derivative(Debug = "ignore")]
    pub reporter: &'a mut dyn diagnostics::ErrorReporter,
    /// Interpreter context
    pub interpreter: Interpreter,
    /// Type table
    pub type_table: Vec<(Vec<TypeVariableId>, ir::Type)>,

    /// Root module
    pub root_module: std::pin::Pin<&'a ir::Module>,
    /// Current module
    pub current_module: ir::expression::symbol_reference::InternalPointer<ir::Module>,
    /// Current module path
    pub current_module_path: Path,

    /// Set this flag once a fatal error has been encountered
    pub abort_compilation: bool,

    pub(crate) scopes: Vec<Scope>,
    next_type_variable_id: TypeVariableId,
    next_variable_id: ir::expression::variable_declaration::VariableId,
}

impl<'a> TypeInference<'a> {
    /// Create a new [TypeInference]
    pub fn new(
        reporter: &'a mut dyn diagnostics::ErrorReporter,
        interpreter: Interpreter,
        root_module: &'a ir::Module,
    ) -> Self {
        let root_module = std::pin::Pin::new(root_module);
        Self {
            return_type: None,
            reporter,
            interpreter,
            type_table: Vec::new(),

            root_module,
            current_module: ir::expression::symbol_reference::InternalPointer::new(root_module),
            current_module_path: Path::new(),

            abort_compilation: false,

            scopes: Vec::new(),
            next_type_variable_id: TypeVariableId(0),
            next_variable_id: 0,
        }
    }

    /// Allocate a new type variable
    pub fn alloc_type_variable(&mut self, r#type: ir::Type) -> TypeVariableId {
        let id = self.next_type_variable_id;
        self.next_type_variable_id.0 += 1;
        self.type_table.push((vec![id], r#type));
        id
    }

    /// If the type is not complete, make it a type variable
    pub fn complete(&mut self, r#type: &mut ir::Type) {
        if !r#type.complete() {
            *r#type = ir::Type::TypeVariable(
                self.alloc_type_variable(std::mem::replace(r#type, ir::Type::Error)),
            );
        }
        match r#type {
            ir::Type::Pointer(r#type, _) => self.complete(r#type),
            ir::Type::FunctionPointer(args, return_type) => {
                for arg in args.iter_mut() {
                    self.complete(arg);
                }
                self.complete(return_type);
            }
            _ => (),
        }
    }

    /// Make two types equal
    pub fn equate(&mut self, lhs: &ir::Type, rhs: &ir::Type) {
        assert!(lhs.complete());
        assert!(rhs.complete());
        match (lhs, rhs) {
            (ir::Type::TypeVariable(lhs), ir::Type::TypeVariable(rhs)) => {
                let type_variables = self
                    .type_table
                    .iter_mut()
                    .enumerate()
                    .filter(|(_, (ids, _))| ids.contains(lhs) || ids.contains(rhs))
                    .collect::<Vec<_>>();

                pub(crate) type TwoTypeVariables<'a> =
                    [(usize, &'a mut (Vec<TypeVariableId>, ir::Type)); 2];
                let type_variables: Result<TwoTypeVariables, _> = type_variables.try_into();

                match type_variables {
                    Ok([(_, (type1_ids, type1)), (type2_index, (type2_ids, type2))]) => {
                        type1_ids.append(type2_ids);
                        type1.equate(type2);
                        self.type_table.remove(type2_index);
                    }
                    Err(_) => (),
                }
            }
            (ir::Type::TypeVariable(type_variable), r#type)
            | (r#type, ir::Type::TypeVariable(type_variable)) => {
                let (_, type_variable) = self
                    .type_table
                    .iter_mut()
                    .find(|(ids, _)| ids.contains(type_variable))
                    .expect("Invalid type variable!");
                type_variable.equate(r#type);
            }
            _ => (),
        }
    }

    /// Inline the type variable if the type is a type variable (One-layer type inline)
    pub fn inline(&self, r#type: &mut ir::Type) {
        if let ir::Type::TypeVariable(type_variable) = r#type {
            let (_, type_variable) = self
                .type_table
                .iter()
                .find(|(ids, _)| ids.contains(type_variable))
                .expect("Invalid type variable!");
            *r#type = type_variable.clone()
        }
    }

    /// Finish a type, replace all type variables with concrete types
    pub fn finish(&mut self, r#type: &mut ir::Type, what: &str, span: Option<&Span>) {
        self.inline(r#type);
        if r#type == &ir::Type::IntegerWildcard {
            *r#type = ir::Type::Int(std::num::NonZeroU16::new(4).unwrap());
        } else if r#type == &ir::Type::FloatWildcard {
            *r#type = ir::Type::Float(std::num::NonZeroU16::new(4).unwrap());
        }
        if !r#type.complete() {
            self.report(
                Report::build(ReportKind::Error)
                    .with_code("typechecking::type_not_inferred")
                    .with_message(format!("Could not infer type for {}", what))
                    .opt_label(span.cloned(), |label| {
                        label.with_message("Here").with_color(colors::Label)
                    })
                    .finish(),
            );
        }
    }

    /// Report an error
    pub fn report(&mut self, report: Report) {
        if report.kind == ReportKind::Error {
            self.abort_compilation = true;
        }
        self.reporter.report(report);
    }
}
