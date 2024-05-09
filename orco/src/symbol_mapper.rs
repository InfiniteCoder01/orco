use super::*;
use ir::expression::VariableDeclaration;
use std::sync::{Arc, Mutex};

/// Scope
pub type Scope = std::collections::HashMap<Span, SymbolReference>;

/// Variable maker
#[derive(Debug)]
pub struct SymbolMapper {
    scopes: Vec<Scope>,
    id_counter: ir::expression::variable_declaration::VariableID,
}

impl SymbolMapper {
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
    ) -> symbol_reference::VariableReference {
        declaration.id = self.id_counter;
        self.id_counter += 1;
        let name = declaration.name.clone();
        let reference = Arc::new(declaration.map(Mutex::new));
        self.current_scope_mut()
            .insert(name, SymbolReference::Variable(reference.clone()));
        reference
    }

    /// Get a variable from the current scope
    pub fn get_variable(&self, name: &Span) -> Option<SymbolReference> {
        for scope in self.scopes.iter().rev() {
            if let Some(reference) = scope.get(name) {
                return Some(reference.clone());
            }
        }
        None
    }

    /// Get a variable expression from the current scope, or report and return an error
    pub fn access_variable(
        &mut self,
        reporter: &mut (impl diagnostics::ErrorReporter + ?Sized),
        name: &Span,
        span: Span,
    ) -> ir::Expression {
        match self.get_variable(name) {
            Some(reference) => ir::Expression::Symbol(Spanned::new(reference, span)),
            None => {
                reporter.report_type_error(
                    format!("Variable '{}' was not found in this scope", name),
                    span.clone(),
                    vec![],
                );
                ir::Expression::Error(span)
            }
        }
    }
}

impl Default for SymbolMapper {
    fn default() -> Self {
        Self::new()
    }
}
