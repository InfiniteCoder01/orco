use super::*;
use ir::expression::{VariableDeclaration, VariableReference};
use std::sync::{Arc, Mutex};

/// Scope
pub type Scope = std::collections::HashMap<String, VariableReference>;

/// Variable maker
#[derive(Debug)]
pub struct VariableMapper {
    scopes: Vec<Scope>,
    id_counter: ir::expression::variable_declaration::VariableID,
}

impl VariableMapper {
    /// Create a new variable maker
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            id_counter: 0,
        }
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pop a scope from the stack
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Get the current scope
    pub fn current_scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    /// Get the current scope
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    /// Declare a variable in the current scope
    pub fn declare_variable(
        &mut self,
        mut declaration: diagnostics::Spanned<VariableDeclaration>,
    ) -> VariableReference {
        declaration.id = self.id_counter;
        self.id_counter += 1;
        let name = declaration.name.inner.clone();
        let reference = Arc::new(declaration.map(Mutex::new));
        self.current_scope_mut().insert(name, reference.clone());
        reference
    }

    /// Get a variable from the current scope
    pub fn get_variable(&self, name: &str) -> Option<VariableReference> {
        self.current_scope().get(name).cloned()
    }

    /// Get a variable expression from the current scope, or report and return an error
    pub fn access_variable(&mut self, name: &str, span: Span) -> ir::Expression {
        match self.get_variable(name) {
            Some(reference) => ir::Expression::Variable(diagnostics::Spanned {
                inner: reference,
                span,
            }),
            None => {
                todo!("report");
                // ir::Expression::Error(span),
            }
        }
    }
}

impl Default for VariableMapper {
    fn default() -> Self {
        Self::new()
    }
}
