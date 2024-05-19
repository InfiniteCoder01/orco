use super::*;
use cranelift_module::Module;

impl crate::Object<'_> {
    /// Build a call expression
    pub fn build_call_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        call: &orco::ir::expression::CallExpression,
    ) -> Option<Value> {
        match call.expression.as_ref() {
            orco::ir::Expression::Symbol(symbol) => {
                let function = match &symbol.inner {
                    orco::SymbolReference::Function(function) => self.object.declare_func_in_func(
                        *self
                            .functions
                            .get(&function.signature.name)
                            .unwrap_or_else(|| panic!("Function {} is not defined", symbol.span)),
                        builder.func,
                    ),
                    orco::SymbolReference::ExternFunction(function) => self.object.declare_func_in_func(
                        *self
                            .functions
                            .get(&function.name)
                            .unwrap_or_else(|| panic!("Function {} is not defined", symbol.span)),
                        builder.func,
                    ),
                    _ => panic!("Can only call functions, not '{}'. Operator overloads should've been replaced by now, did you run type checking/inference?", call.expression),
                };
                let args = call
                    .args
                    .iter()
                    .map(|arg| self.build_expression(builder, arg)
                         .expect("Can't pass a unit type as an argument to a function, did you run type checking/inference?"))
                    .collect::<Vec<_>>();
                let instruction = builder.ins().call(function, &args);
                builder.inst_results(instruction).first().copied()
            }
            orco::ir::Expression::Error(_) => panic!("IR contains errors, did you run type checking/inference?"),
            _ => panic!("Can only call functions, not '{}'. Operator overloads should've been replaced by now, did you run type checking/inference?", call.expression),
        }
    }
}
