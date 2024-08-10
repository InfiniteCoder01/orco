use orco::ir::expression::SymbolReference;

use super::*;

impl crate::Object {
    /// Build a constant expression
    pub fn build_symbol_reference(
        &mut self,
        builder: &mut FunctionBuilder,
        symbol: &SymbolReference,
    ) -> Option<Value> {
        match &symbol {
            SymbolReference::Symbol(symbol) => {
                let symbol = symbol.try_read().unwrap();
                self.build_constant_value(
                    builder,
                    symbol
                        .evaluated
                        .as_ref()
                        .expect("Non-evaluated symbols in IR!"),
                    &symbol.value.get_type(),
                )
            }
            SymbolReference::Variable(variable) => {
                Some(builder.use_var(Variable::new(*variable.id.try_lock().unwrap() as _)))
            }
            _ => {
                panic!(
                    "Invalid/unsupported symbol: {}. Did you run type checking/inference?",
                    symbol
                )
            }
        }
    }
}
