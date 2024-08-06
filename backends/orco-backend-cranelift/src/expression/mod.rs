use super::*;
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::{FunctionBuilder, Variable};

/// Build constants
pub mod constant;

/// Reference symbols
pub mod symbol_reference;

/// Build code blocks
pub mod block;

/// Build operator-based expressions (unary, binary, assignment, etc.)
pub mod operator;

/// Build branching constructs
pub mod branching;

/// Build function calls
pub mod call;

impl crate::Object<'_> {
    /// Build an expression
    pub fn build_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &orco::ir::Expression,
    ) -> Option<Value> {
        use orco::ir::Expression;
        match expr {
            Expression::Function(_) => unimplemented!("Functions in runtime are not supported"),
            Expression::ExternFunction(_) => {
                unimplemented!("Extern functions in runtime are not supported")
            }
            Expression::Constant(value) => self.build_constant(builder, value),
            Expression::Symbol(symbol, ..) => self.build_symbol_reference(builder, symbol),
            Expression::BinaryExpression(expr) => self.build_binary_expression(builder, expr),
            Expression::UnaryExpression(expr) => self.build_unary_expression(builder, expr),
            Expression::Block(block) => self.build_block(builder, block),
            Expression::If(expr) => self.build_if_expression(builder, expr),
            Expression::Call(expr) => self.build_call_expression(builder, expr),
            Expression::Return(value) => {
                let ret = self.build_expression(builder, &value.0);
                builder.ins().return_(&ret.into_iter().collect::<Vec<_>>());
                None
            }
            Expression::VariableDeclaration(declaration) => {
                let variable = Variable::new(*declaration.id.try_lock().unwrap() as _);
                builder.declare_var(
                    variable,
                    self.convert_type(&declaration.r#type.try_lock().unwrap()),
                );
                if let Some(value) = &declaration.value {
                    let value = self.build_expression(builder, &value.try_lock().unwrap()).expect("Can't initialize a variable to a unit type, did you run type checking/inference?");
                    builder.def_var(variable, value);
                }
                None
            }
            Expression::Assignment(expr) => self.build_assignment_expression(builder, expr),
            Expression::Error(span) => panic!("IR contains errors at {:?}!", span),
        }
    }
}
