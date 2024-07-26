use super::*;

impl Interpreter {
    pub fn evaluate_binary(&mut self, expr: &ir::expression::BinaryExpression) -> Value {
        let lhs = self.evaluate(expr.lhs.as_ref());
        let rhs = self.evaluate(expr.rhs.as_ref());

        macro_rules! binary {
            ($op:tt) => {
                match expr.get_type() {
                    Type::Int(size) if size.get() == 1 => Value::new(*lhs.into::<i8>() $op *rhs.into::<i8>()),
                    Type::Int(size) if size.get() == 2 => Value::new(*lhs.into::<i16>() $op *rhs.into::<i16>()),
                    Type::Int(size) if size.get() == 4 => Value::new(*lhs.into::<i32>() $op *rhs.into::<i32>()),
                    Type::Int(size) if size.get() == 8 => Value::new(*lhs.into::<i64>() $op *rhs.into::<i64>()),
                    Type::Int(size) if size.get() == 16 => Value::new(*lhs.into::<i128>() $op *rhs.into::<i128>()),
                    Type::Unsigned(size) if size.get() == 1 => Value::new(*lhs.into::<u8>() $op *rhs.into::<u8>()),
                    Type::Unsigned(size) if size.get() == 2 => Value::new(*lhs.into::<u16>() $op *rhs.into::<u16>()),
                    Type::Unsigned(size) if size.get() == 4 => Value::new(*lhs.into::<u32>() $op *rhs.into::<u32>()),
                    Type::Unsigned(size) if size.get() == 8 => Value::new(*lhs.into::<u64>() $op *rhs.into::<u64>()),
                    Type::Unsigned(size) if size.get() == 16 => Value::new(*lhs.into::<u128>() $op *rhs.into::<u128>()),
                    invalid => panic!(
                        "Can't do binary operation '{}' with operands of type '{}'",
                        stringify!($op),
                        invalid
                    )
                }
            }
        }

        use ir::expression::BinaryOp;
        match expr.op {
            BinaryOp::Add => binary!(+),
            BinaryOp::Mul => binary!(*),
            invalid => todo!("Binary operator '{}'", invalid),
        }
    }
}
