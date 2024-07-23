use super::*;

// /// Scopes & Symbol mapping
// pub mod scopes;
// pub use scopes::Scope;

/// Type variable ID, used for type inference with Hindley–Milner algorithm
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeVariableId(u64);

impl std::fmt::Display for TypeVariableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<type variable #{}>", self.0)
    }
}

/// Type inference information for a function
pub struct TypeInference<'a> {
    /// Return type of a function
    pub return_type: Option<&'a diagnostics::Spanned<ir::Type>>,
    /// Error reporter
    pub reporter: &'a mut dyn diagnostics::ErrorReporter,
    /// Type table
    pub type_table: Vec<(Vec<TypeVariableId>, ir::Type)>,

    next_type_variable_id: TypeVariableId,

    /// Root module
    pub root_module: &'a ir::Module,
    /// Current module
    pub current_module: &'a ir::Module,
    /// Current module path
    pub current_module_path: Path,
    // scopes: Vec<Scope>,
}

impl<'a> TypeInference<'a> {
    /// Create a new [TypeInference]
    pub fn new(
        reporter: &'a mut dyn diagnostics::ErrorReporter,
        root_module: &'a ir::Module,
    ) -> Self {
        Self {
            return_type: None,
            reporter,
            type_table: Vec::new(),

            next_type_variable_id: TypeVariableId(0),

            root_module,
            current_module: root_module,
            current_module_path: Path::new(),
            // scopes: Vec::new(),
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
    pub fn complete(&mut self, r#type: ir::Type) -> ir::Type {
        if r#type.complete() {
            r#type
        } else {
            ir::Type::TypeVariable(self.alloc_type_variable(r#type))
        }
    }

    /// Make two types equal
    pub fn equate(&mut self, lhs: &ir::Type, rhs: &ir::Type) -> ir::Type {
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
                        type1.equate(type2.clone());
                        let r#type = ir::Type::TypeVariable(type1_ids[0]);
                        self.type_table.remove(type2_index);
                        r#type
                    }
                    Err(type_variables) => type_variables[0].1 .1.clone(),
                }
            }
            (ir::Type::TypeVariable(type_variable), r#type)
            | (r#type, ir::Type::TypeVariable(type_variable)) => {
                let (type_ids, type_variable) = self
                    .type_table
                    .iter_mut()
                    .find(|(ids, _)| ids.contains(type_variable))
                    .expect("Invalid type variable!");
                type_variable.equate(r#type.clone());
                ir::Type::TypeVariable(type_ids[0])
            }
            (lhs, rhs) => lhs.clone() | rhs.clone(),
        }
    }

    /// Inline the type variable if the type is a type variable (One-layer type inline)
    pub fn inline(&self, r#type: ir::Type) -> ir::Type {
        if let ir::Type::TypeVariable(type_variable) = r#type {
            let (_, type_variable) = self
                .type_table
                .iter()
                .find(|(ids, _)| ids.contains(&type_variable))
                .expect("Invalid type variable!");
            type_variable.clone()
        } else {
            r#type
        }
    }

    /// Finish a type, replace all type variables with concrete types
    pub fn finish(&mut self, r#type: &mut ir::Type, what: &str, span: diagnostics::Span) {
        *r#type = self.inline(r#type.clone());
        if r#type == &ir::Type::IntegerWildcard {
            *r#type = ir::Type::Int(std::num::NonZeroU16::new(4).unwrap());
        } else if r#type == &ir::Type::FloatWildcard {
            *r#type = ir::Type::Float(std::num::NonZeroU16::new(4).unwrap());
        }
        if !r#type.complete() {
            self.reporter.report_type_error(
                format!("Could not infer type for {}", what),
                span,
                vec![],
            );
        }
    }
}
