use std::collections::HashMap;

use super::*;

/// Scope
pub type Scope = HashMap<PathSegment, SymbolReference>;

impl TypeInference<'_> {
    /// Start a new scope
    pub fn push_scope(&mut self) {
        println!("Push");
        self.scopes.push(Scope::new());
    }

    /// End the current scope
    pub fn pop_scope(&mut self) {
        println!("Pop");
        self.scopes.pop();
    }

    /// Get a mutable reference to the current scope
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes
            .last_mut()
            .expect("Current scope can only be a local scope")
    }

    /// Try to find a symbol in the local space, scope-aware
    pub fn get_symbol(&self, name: &PathSegment) -> Option<SymbolReference> {
        for scope in self.scopes.iter().rev() {
            if name.as_ref() == "k" {
                println!("New scope");
                for (var, val) in scope {
                    println!("Scope contains {}", var);
                }
            }
            if let Some(symbol) = scope.get(name) {
                return Some(symbol.clone());
            }
        }
        println!("Failed to resolve: {}", name);
        None
    }

    /// Try to resolve a symbol using the provided resolver
    pub fn resolve_symbol(&mut self, name: &Path) -> Option<SymbolReference> {
        (self.symbol_resolver)(self, name)
    }

    /// Get a new variable id (just a counter)
    pub fn new_variable_id(&mut self) -> ir::expression::variable_declaration::VariableId {
        let id = self.next_variable_id;
        self.next_variable_id += 1;
        id
    }
}
