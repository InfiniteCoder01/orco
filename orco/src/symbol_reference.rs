use super::*;
use ir::expression::variable_declaration::VariableDeclaration;
use ir::Type;
use std::sync::Arc;

/// Function reference
pub type FunctionReference = Arc<Spanned<ir::symbol::Function>>;
/// Extern function reference
pub type ExternFunctionReference = Arc<Spanned<ir::symbol::function::Signature>>;

/// A symbol (function, variable, etc.)
pub trait Symbol: std::fmt::Debug {
    /// Get type this symbol evaluates to when accessed
    fn get_type(&self) -> Type;
    /// Infer types for the symbol when referenced
    fn infer_types(&self, type_inference: &mut TypeInference) -> Type;
    /// Finish and check types for the symbol when referenced
    fn finish_and_check_types(&self, span: Span, type_inference: &mut TypeInference) -> Type;
    /// Display this symbol as if it was accessed
    fn display(&self, span: Span, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    /// Variable access if this symbol is a variable of any kind
    fn as_variable(&self) -> Option<&VariableDeclaration> {
        None
    }
    /// Get the function signature if this symbol is a function of any kind
    fn as_any_function(&self) -> Option<Spanned<&ir::symbol::function::Signature>> {
        None
    }
    /// Get the function if this symbol is a reference to a normal function
    fn as_function(&self) -> Option<&Spanned<ir::symbol::Function>> {
        None
    }
    /// Get the extern function if this symbol is a reference to an extern function
    fn as_extern_function(&self) -> Option<&Spanned<ir::symbol::function::Signature>> {
        None
    }
}

/// Symbol reference
pub type SymbolReference = Arc<dyn Symbol>;

impl std::fmt::Display for Spanned<SymbolReference> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display(self.span.clone(), f)
    }
}

impl Symbol for Spanned<VariableDeclaration> {
    fn get_type(&self) -> Type {
        self.r#type.inner.lock().unwrap().clone()
    }

    fn infer_types(&self, type_inference: &mut TypeInference) -> Type {
        let mut r#type = self.r#type.lock().unwrap();
        *r#type = type_inference.complete(r#type.clone());
        r#type.clone()
    }

    fn finish_and_check_types(&self, span: Span, type_inference: &mut TypeInference) -> Type {
        let mut r#type = self.r#type.lock().unwrap();
        type_inference.finish(&mut r#type, &format!("variable '{}'", self.name), span);
        r#type.clone()
    }

    fn display(&self, span: Span, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let show_id = std::env::var("ORCO_SHOW_VAR_ID").map_or(false, |show_id| show_id == "1");
        if show_id {
            write!(f, "{} (#{})", span, self.id.lock().unwrap())
        } else {
            write!(f, "{}", span)
        }
    }

    fn as_variable(&self) -> Option<&VariableDeclaration> {
        Some(self)
    }
}

impl Symbol for Spanned<ir::symbol::function::Signature> {
    fn get_type(&self) -> Type {
        self.inner.get_type()
    }

    fn infer_types(&self, _type_inference: &mut TypeInference) -> Type {
        self.inner.get_type()
    }

    fn finish_and_check_types(&self, _span: Span, _type_inference: &mut TypeInference) -> Type {
        self.inner.get_type()
    }

    fn display(&self, span: Span, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", span)
    }

    fn as_any_function(&self) -> Option<Spanned<&ir::symbol::function::Signature>> {
        Some(Spanned::new(&self.inner, self.span.clone()))
    }
}
