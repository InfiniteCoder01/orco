use super::*;

impl crate::Object<'_> {
    pub fn build_binary_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &orco::ir::expression::BinaryExpression,
    ) -> Option<Value> {
        let lhs = self.build_expression(builder, &expr.lhs)?;
        let rhs = self.build_expression(builder, &expr.rhs)?;
        use cranelift_codegen::ir::condcodes::IntCC;
        use orco::ir::expression::BinaryOp;
        match expr.op {
            BinaryOp::Add => Some(builder.ins().iadd(lhs, rhs)),
            BinaryOp::Sub => Some(builder.ins().isub(lhs, rhs)),
            BinaryOp::Mul => Some(builder.ins().imul(lhs, rhs)),
            BinaryOp::Div => Some(builder.ins().sdiv(lhs, rhs)),
            BinaryOp::Mod => Some(builder.ins().srem(lhs, rhs)),
            BinaryOp::Eq => Some(builder.ins().icmp(IntCC::Equal, lhs, rhs)),
            BinaryOp::Ne => Some(builder.ins().icmp(IntCC::NotEqual, lhs, rhs)),
            BinaryOp::Lt => Some(builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs)),
            BinaryOp::Le => Some(builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs)),
            BinaryOp::Gt => Some(builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs)),
            BinaryOp::Ge => Some(
                builder
                    .ins()
                    .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),
            ),
        }
    }

    pub fn build_unary_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &orco::ir::expression::UnaryExpression,
    ) -> Option<Value> {
        let value = self.build_expression(builder, &expr.expr)?;
        use orco::ir::expression::UnaryOp;
        match expr.op {
            UnaryOp::Neg => Some(builder.ins().ineg(value)),
        }
    }

    pub fn build_assignment_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &orco::ir::expression::AssignmentExpression,
    ) -> Option<Value> {
        let value = self.build_expression(builder, &expr.value)?;
        match expr.target.as_ref() {
            orco::ir::Expression::Symbol(orco::Spanned {
                inner: orco::SymbolReference::Variable(variable),
                ..
            }) => {
                let variable = variable.lock().unwrap();
                let variable = Variable::new(variable.id as _);
                builder.def_var(variable, value);
            }
            target => panic!(
                "Can't assign to '{}'! Did you run type checking/inference?",
                target
            ),
        }
        None
    }
}
