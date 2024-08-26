use std::sync::RwLock;

use super::*;

#[derive(Debug)]
/// A symbol
pub struct Symbol {
    /// Symbol name
    pub name: Name,
    /// Symbol path, gets completed at type-inferece stage
    pub path: Path,
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
            path: Path::new(),
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
    let mut symbol_locked = symbol.try_write().unwrap();
    symbol_locked.path = type_inference
        .current_module_path
        .extend(symbol_locked.name.clone());
    if symbol_locked.evaluated.is_none() {
        let abort_compilation = type_inference.abort_compilation;
        type_inference.abort_compilation = false;

        let r#type = symbol_locked.value.infer_types(type_inference);
        type_inference.complete(&mut symbol_locked.r#type);
        type_inference.equate(&r#type, &symbol_locked.r#type);

        symbol_locked.value.finish_and_check_types(type_inference);
        {
            let symbol: &mut Symbol = &mut symbol_locked;
            type_inference.finish(
                &mut symbol.r#type,
                &format!("symbol '{}'", symbol.name),
                Some(&symbol.name),
            );
        }
        if type_inference.abort_compilation {
            symbol_locked.evaluation_failed = true;
            return;
        }

        type_inference.abort_compilation = abort_compilation;
        let mut evaluated = type_inference.interpreter.evaluate(&symbol_locked.value);
        if symbol_locked.r#type.inner == Type::Module {
            evaluated.as_mut::<Module>().parent = Some(type_inference.current_module);
        }
        symbol_locked.evaluated = Some(evaluated);
        drop(symbol_locked);

        let symbol_locked = symbol.try_read().unwrap();
        match symbol_locked.r#type.inner {
            Type::Function => {
                let function = symbol_locked
                    .evaluated
                    .as_ref()
                    .unwrap()
                    .as_ref::<expression::Function>();
                function.infer_and_check_types(type_inference);
            }
            Type::Module => {
                let module = symbol_locked.evaluated.as_ref().unwrap().as_ref::<Module>();
                let module = std::pin::Pin::new(module);

                let current_module = std::mem::replace(
                    &mut type_inference.current_module,
                    expression::symbol_reference::InternalPointer::new(module),
                );
                type_inference
                    .current_module_path
                    .push(symbol_locked.name.clone());

                module.infer_and_check_types(type_inference);

                type_inference.current_module_path.0.pop();
                type_inference.current_module = current_module;
            }
            _ => (),
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
            path: self.path.clone(),
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
            Some(evaluated) if self.r#type.inner == Type::Function => {
                format!("{}", evaluated.as_ref::<expression::Function>())
            }
            Some(evaluated) if self.r#type.inner == Type::Module => {
                format!("{}", evaluated.as_ref::<Module>())
            }
            _ => format!("{}", self.value),
        };
        write!(f, "comptime {}: {} = {};", self.name, self.r#type, value)
    }
}
