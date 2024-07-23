use super::*;

/// Value - object with a type, returned from an interpreter
pub mod value;
pub use value::Value;

/// Context for an interpreter
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interpreter {}

impl Interpreter {
    /// Evaluate an expression
    pub fn evaluate(&self, expr: &ir::Expression) -> Value {
        match expr {
            ir::Expression::Function(function) => Value::from(function.as_ref().clone()),
            ir::Expression::Constant(constant) => Value::from_constant(constant.inner.clone()),
            ir::Expression::BinaryExpression(_) => todo!(),
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
