use super::*;
use ir::expression::VariableDeclaration;
use std::sync::{Arc, Mutex};

/// Scope
pub type Scope = std::collections::HashMap<Span, SymbolReference>;

/// Variable maker
#[derive(Debug)]
pub struct SymbolMapper {
    /// Allow use of functions before their declaration
    pub use_functions_before_declaration: bool,

    /// Global scope. All global functions should be put here
    pub global: Scope,
    scopes: Vec<Scope>,
    id_counter: ir::expression::variable_declaration::VariableID,
}

impl SymbolMapper {
    /// Create a new variable maker
    pub fn new() -> Self {
        Self {
            use_functions_before_declaration: true,
            global: Scope::new(),
            scopes: vec![],
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
        self.scopes.last().unwrap_or(&self.global)
    }

    /// Get the current scope
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap_or(&mut self.global)
    }

    /// Declare a variable in the current scope
    pub fn declare_variable(
        &mut self,
        mut declaration: diagnostics::Spanned<VariableDeclaration>,
    ) -> ir::expression::Variable {
        declaration.id = Mutex::new(self.id_counter);
        self.id_counter += 1;
        let name = declaration.name.clone();
        let reference = Arc::new(declaration);
        self.current_scope_mut().insert(name, reference.clone());
        reference
    }

    /// Get symbol by name
    pub fn get_symbol(&self, name: &Span) -> Option<SymbolReference> {
        for scope in self.scopes.iter().rev() {
            if let Some(reference) = scope.get(name) {
                return Some(reference.clone());
            }
        }
        self.global.get(name).cloned()
    }

    /// Get a variable expression from the current scope, or report and return an error
    pub fn access_symbol(
        &mut self,
        reporter: &mut (impl diagnostics::ErrorReporter + ?Sized),
        name: &Span,
        span: Span,
    ) -> ir::Expression {
        match self.get_symbol(name) {
            Some(reference) => ir::Expression::Symbol(Spanned::new(reference, span)),
            None => {
                reporter.report_type_error(
                    format!(
                        "Variable or symbol '{}' was not declared in this scope",
                        name
                    ),
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
