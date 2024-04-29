#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Diagnostics
pub mod diagnostics;
/// Intermediate Representation lives here
pub mod ir;
/// Source
pub mod source;

pub use source::*;

/// Type variable ID, used for type inference with Hindley–Milner algorithm
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeVariableID(u64);

impl std::fmt::Display for TypeVariableID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<type variable #{}>", self.0)
    }
}

/// Type inference information for a function
pub struct TypeInference<'a> {
    /// Root module
    pub root: &'a ir::Module,
    /// Return type of a function
    pub return_type: &'a diagnostics::Spanned<ir::Type>,
    /// Error reporter
    pub reporter: &'a mut dyn diagnostics::ErrorReporter,
    /// Type table
    pub type_table: Vec<(Vec<TypeVariableID>, ir::Type)>,
    /// Next type variable ID
    pub next_type_variable_id: u64,
}

impl<'a> TypeInference<'a> {
    /// Create a new [TypeInference]
    pub fn new(
        root: &'a ir::Module,
        return_type: &'a diagnostics::Spanned<ir::Type>,
        reporter: &'a mut dyn diagnostics::ErrorReporter,
    ) -> Self {
        Self {
            root,
            return_type,
            reporter,
            type_table: Vec::new(),
            next_type_variable_id: 0,
        }
    }

    /// Get function signature
    pub fn signature<'b>(&'b self, name: &str) -> Option<&'a ir::item::function::Signature> {
        self.root
            .items
            .get(name)
            .and_then(|item| item.function_signature())
    }

    /// Allocate a new type variable
    pub fn alloc_type_variable(&mut self, r#type: ir::Type) -> TypeVariableID {
        let id = TypeVariableID(self.next_type_variable_id);
        self.next_type_variable_id += 1;
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

                type TwoTypeVariables<'a> = [(usize, &'a mut (Vec<TypeVariableID>, ir::Type)); 2];
                let type_variables: Result<TwoTypeVariables, _> = type_variables.try_into();

                match type_variables {
                    Ok([(_, (type1_ids, type1)), (type2_index, (type2_ids, type2))]) => {
                        type1_ids.append(type2_ids);
                        *type1 |= type2.clone();
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
                *type_variable |= r#type.clone();
                ir::Type::TypeVariable(type_ids[0])
            }
            (lhs, rhs) => lhs.clone() | rhs.clone(),
        }
    }

    /// Finish a type, replace all type variables with concrete types
    pub fn finish(&mut self, r#type: &mut ir::Type, what: &str, span: diagnostics::Span) {
        if let ir::Type::TypeVariable(type_variable) = r#type {
            let (_, type_variable) = self
                .type_table
                .iter()
                .find(|(ids, _)| ids.contains(type_variable))
                .expect("Invalid type variable!");
            *r#type = type_variable.clone();
        }

        if !r#type.complete() {
            self.reporter.report_type_error(format!("Could not infer type for {}", what), span, None);
        }
    }
}
