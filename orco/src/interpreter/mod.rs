use super::*;
use ir::Type;

/// Value - object with a type, returned from an interpreter
pub mod value;
pub use value::Value;

/// Operator expressions
pub mod operator;

/// Context for an interpreter
#[derive(Clone, Debug, Default)]
pub struct Interpreter {}

impl Interpreter {
    /// Evaluate an expression
    pub fn evaluate(&mut self, expr: &ir::Expression) -> Value {
        match expr {
            ir::Expression::Function(function) => Value::new(function.as_ref().clone()),
            ir::Expression::ExternFunction(function) => Value::new(function.clone()),
            ir::Expression::Module(module) => Value::new(module.inner.clone()),
            ir::Expression::Constant(constant) => Value::from_constant(constant.inner.clone()),
            ir::Expression::Symbol(_, _) => todo!(),
            ir::Expression::BinaryExpression(expr) => self.evaluate_binary(expr),
            ir::Expression::UnaryExpression(_) => todo!(),
            ir::Expression::Block(_) => todo!(),
            ir::Expression::If(_) => todo!(),
            ir::Expression::Call(_) => todo!(),
            ir::Expression::Return(_) => todo!(),
            ir::Expression::VariableDeclaration(_) => todo!(),
            ir::Expression::Assignment(_) => todo!(),
            ir::Expression::Error(_) => panic!("Interpreting IR, that contains errors!"),
        }
    }
}
