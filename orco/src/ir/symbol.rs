use std::sync::RwLock;

use super::*;

#[derive(Debug)]
/// A symbol
pub struct Symbol {
    /// Symbol name
    pub name: Name,
    /// Symbol type
    pub r#type: Spanned<Type>,
    /// Symbol value
    pub value: Expression,
    /// Evaluated
    pub evaluated: Option<Value>,
    /// Tried to evaluate before, but failed because of a compilation error?
    pub evaluation_failed: bool,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(name: Name, r#type: Spanned<Type>, value: Expression) -> Self {
        Self {
            name,
            r#type,
            value,
            evaluated: None,
            evaluation_failed: false,
        }
    }
}

/// Ensure that this symbol is type-checked and evaluated
pub fn ensure_evaluated(symbol: &RwLock<Symbol>, type_inference: &mut TypeInference) {
    let should_evaluate = {
        let symbol = symbol.try_read().unwrap();
        symbol.evaluated.is_none() && !symbol.evaluation_failed
    };
    if !should_evaluate {
        return;
    }
    let mut checked_symbol = symbol.try_write().unwrap();
    if checked_symbol.evaluated.is_none() {
        let abort_compilation = type_inference.abort_compilation;
        type_inference.abort_compilation = false;

        let r#type = checked_symbol.value.infer_types(type_inference);
        type_inference.complete(&mut checked_symbol.r#type);
        type_inference.equate(&r#type, &checked_symbol.r#type);

        let r#type = checked_symbol.value.finish_and_check_types(type_inference);
        {
            let symbol: &mut Symbol = &mut checked_symbol;
            type_inference.finish(
                &mut symbol.r#type,
                &format!("symbol '{}'", symbol.name),
                Some(&symbol.name),
            );
        }
        if type_inference.abort_compilation {
            checked_symbol.evaluation_failed = true;
            return;
        }

        type_inference.abort_compilation = abort_compilation;

        checked_symbol.evaluated = Some(type_inference.interpreter.evaluate(&checked_symbol.value));
        drop(checked_symbol);

        if r#type == Type::Function {
            let symbol = symbol.try_read().unwrap();
            let function = symbol
                .evaluated
                .as_ref()
                .unwrap()
                .as_ref::<expression::Function>();
            function.infer_and_check_types(type_inference);
        }
    }
}

/// Check if a symbol is getting evaluated recursively
pub fn check_for_recursion(symbol: &RwLock<Symbol>) -> bool {
    match symbol.try_read() {
        Ok(_) => false,
        Err(std::sync::TryLockError::WouldBlock) => true,
        Err(err) => panic!("{}", err),
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            r#type: self.r#type.clone(),
            value: self.value.clone(),
            evaluated: None,
            evaluation_failed: false,
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self.evaluated.as_ref() {
            Some(evaluated) if self.value.get_type() == Type::Function => {
                format!("{}", evaluated.as_ref::<expression::Function>())
            }
            _ => format!("{}", self.value),
        };
        write!(f, "comptime {}: {} = {};", self.name, self.r#type, value)
    }
}
