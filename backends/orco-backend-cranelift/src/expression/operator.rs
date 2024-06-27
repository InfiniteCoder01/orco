use super::*;

impl crate::Object<'_> {
    /// Build a binary expression
    pub fn build_binary_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &orco::ir::expression::BinaryExpression,
    ) -> Option<Value> {
        let lhs = self.build_expression(builder, &expr.lhs)?;
        let rhs = self.build_expression(builder, &expr.rhs)?;
        let r#type = expr.lhs.get_type() | expr.rhs.get_type();
        use orco::ir::expression::BinaryOp;
        if matches!(
            r#type,
            orco::ir::Type::Float(_) | orco::ir::Type::FloatWildcard
        ) {
            use cranelift_codegen::ir::condcodes::FloatCC;
            match expr.op {
                BinaryOp::Add => Some(builder.ins().fadd(lhs, rhs)),
                BinaryOp::Sub => Some(builder.ins().fsub(lhs, rhs)),
                BinaryOp::Mul => Some(builder.ins().fmul(lhs, rhs)),
                BinaryOp::Div => Some(builder.ins().fdiv(lhs, rhs)),
                BinaryOp::Mod => unimplemented!("fmod"),
                BinaryOp::Eq => Some(builder.ins().fcmp(FloatCC::Equal, lhs, rhs)),
                BinaryOp::Ne => Some(builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs)),
                BinaryOp::Lt => Some(builder.ins().fcmp(FloatCC::LessThan, lhs, rhs)),
                BinaryOp::Le => Some(builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs)),
                BinaryOp::Gt => Some(builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs)),
                BinaryOp::Ge => Some(builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs)),
            }
        } else {
            let unsigned = matches!(r#type, orco::ir::Type::Unsigned(_));
            use cranelift_codegen::ir::condcodes::IntCC;
            match expr.op {
                BinaryOp::Add => Some(builder.ins().iadd(lhs, rhs)),
                BinaryOp::Sub => Some(builder.ins().isub(lhs, rhs)),
                BinaryOp::Mul => Some(builder.ins().imul(lhs, rhs)),
                BinaryOp::Div => Some(builder.ins().sdiv(lhs, rhs)),
                BinaryOp::Mod => Some(builder.ins().srem(lhs, rhs)),
                BinaryOp::Eq => Some(builder.ins().icmp(IntCC::Equal, lhs, rhs)),
                BinaryOp::Ne => Some(builder.ins().icmp(IntCC::NotEqual, lhs, rhs)),
                BinaryOp::Lt if unsigned => {
                    Some(builder.ins().icmp(IntCC::UnsignedLessThan, lhs, rhs))
                }
                BinaryOp::Le if unsigned => {
                    Some(builder.ins().icmp(IntCC::UnsignedLessThanOrEqual, lhs, rhs))
                }
                BinaryOp::Gt if unsigned => {
                    Some(builder.ins().icmp(IntCC::UnsignedGreaterThan, lhs, rhs))
                }
                BinaryOp::Ge if unsigned => Some(builder.ins().icmp(
                    IntCC::UnsignedGreaterThanOrEqual,
                    lhs,
                    rhs,
                )),
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
    }

    /// Build a unary expression
    pub fn build_unary_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &orco::ir::expression::UnaryExpression,
    ) -> Option<Value> {
        let r#type = expr.get_type();
        let value = self.build_expression(builder, &expr.expr)?;
        use orco::ir::expression::UnaryOp;
        if matches!(r#type, orco::ir::Type::Float(_)) {
            match expr.op {
                UnaryOp::Neg => Some(builder.ins().fneg(value)),
            }
        } else {
            match expr.op {
                UnaryOp::Neg => Some(builder.ins().ineg(value)),
            }
        }
    }

    /// Build an assignment expression
    pub fn build_assignment_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &orco::ir::expression::AssignmentExpression,
    ) -> Option<Value> {
        let value = self.build_expression(builder, &expr.value)?;
        if let orco::ir::Expression::Symbol(symbol) = expr.target.as_ref() {
            if let orco::SymbolReference::Variable(variable) = &symbol.inner {
                let variable = Variable::new(*variable.id.lock().unwrap() as _);
                builder.def_var(variable, value);
            } else {
                panic!(
                    "Can't assign to '{}'! Did you run type checking/inference?",
                    symbol.inner
                )
            }
        } else {
            panic!(
                "Can't assign to '{}'! Did you run type checking/inference?",
                expr.target
            )
        }
        None
    }
}
