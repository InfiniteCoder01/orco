use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::Module;

pub mod block;
pub mod constant;

impl crate::Object<'_> {
    pub fn build_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expression: &orco::ir::Expression,
    ) -> Option<Value> {
        use orco::ir::Expression;
        match expression {
            orco::ir::expression::Expression::Constant(value) => {
                self.build_constant(builder, value)
            }
            orco::ir::expression::Expression::Variable(variable) => {
                let variable = variable.lock().unwrap();
                Some(builder.use_var(Variable::new(variable.id as _)))
            }
            orco::ir::expression::Expression::BinaryOp(lhs, op, rhs) => {
                let lhs = self.build_expression(builder, lhs)?;
                let rhs = self.build_expression(builder, rhs)?;
                use cranelift_codegen::ir::condcodes::IntCC;
                use orco::ir::expression::BinaryOp;
                match op {
                    BinaryOp::Add => Some(builder.ins().iadd(lhs, rhs)),
                    BinaryOp::Sub => Some(builder.ins().isub(lhs, rhs)),
                    BinaryOp::Mul => Some(builder.ins().imul(lhs, rhs)),
                    BinaryOp::Div => Some(builder.ins().sdiv(lhs, rhs)),
                    BinaryOp::Mod => Some(builder.ins().srem(lhs, rhs)),
                    BinaryOp::Eq => Some(builder.ins().icmp(IntCC::Equal, lhs, rhs)),
                    BinaryOp::Ne => Some(builder.ins().icmp(IntCC::NotEqual, lhs, rhs)),
                    BinaryOp::Lt => Some(builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs)),
                    BinaryOp::Le => {
                        Some(builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs))
                    }
                    BinaryOp::Gt => Some(builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs)),
                    BinaryOp::Ge => Some(builder.ins().icmp(
                        IntCC::SignedGreaterThanOrEqual,
                        lhs,
                        rhs,
                    )),
                }
            }
            Expression::UnaryOp(op, value) => {
                let value = self.build_expression(builder, value)?;
                use orco::ir::expression::UnaryOp;
                match op.inner {
                    UnaryOp::Neg => Some(builder.ins().ineg(value)),
                }
            }
            Expression::Block(block) => self.build_block(builder, block),
            Expression::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // TODO: If values
                let condition = self.build_expression(builder, condition).expect("Can't pass a unit type as an argument to an if statement, did you run type checking/inference?");
                let then_block = builder.create_block();
                let else_block = if else_branch.is_some() {
                    Some(builder.create_block())
                } else {
                    None
                };
                let merge_block = builder.create_block();

                builder.ins().brif(
                    condition,
                    then_block,
                    &[],
                    else_block.unwrap_or(merge_block),
                    &[],
                );

                builder.switch_to_block(then_block);
                builder.seal_block(then_block);
                self.build_expression(builder, then_branch);
                builder.ins().jump(merge_block, &[]);

                if let (Some(else_branch), Some(else_block)) = (else_branch, else_block) {
                    builder.switch_to_block(else_block);
                    self.build_expression(builder, else_branch);
                    builder.ins().jump(merge_block, &[]);
                    builder.seal_block(else_block);
                }

                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);
                None
            }
            // Expression::While {
            //     condition,
            //     body,
            //     ..
            // } => {
            //     let condition = self.build_expression(builder, condition).expect("Can't pass a unit type as an argument to an if statement, did you run type checking/inference?");
            //     let body_block = builder.create_block();
            //     let merge_block = builder.create_block();
            //
            //     builder.switch_to_block(body_block);
            //     builder.ins().brif(
            //         condition,
            //         body_block,
            //         &[],
            //         merge_block,
            //         &[],
            //     );
            //
            //     self.build_block(builder, then_branch);
            //     builder.ins().jump(body_block, &[]);
            //     builder.seal_block(body_block);
            //
            //     builder.switch_to_block(merge_block);
            //     builder.seal_block(merge_block);
            //     None
            // }
            orco::ir::expression::Expression::FunctionCall { name, args } => {
                let function = self.object.declare_func_in_func(
                    *self
                        .functions
                        .get(&name.inner)
                        .unwrap_or_else(|| panic!("Function {} is not defined", name.inner)),
                    builder.func,
                );
                let args = args.iter().map(|arg| self.build_expression(builder, arg).expect("Can't pass a unit type as an argument to a function, did you run type checking/inference?")).collect::<Vec<_>>();
                let instruction = builder.ins().call(function, &args);
                builder.inst_results(instruction).first().copied()
            }
            Expression::Return(value) => {
                let ret = self.build_expression(builder, value);
                builder.ins().return_(&ret.into_iter().collect::<Vec<_>>());
                None
            }
            Expression::VariableDeclaration(declaration) => {
                let declaration = declaration.lock().unwrap();
                let variable = Variable::new(declaration.id as _);
                builder.declare_var(variable, self.convert_type(&declaration.r#type));
                if let Some(value) = &declaration.value {
                    let value = self.build_expression(builder, value).expect("Can't initialize a variable to a unit type, did you run type checking/inference?");
                    builder.def_var(variable, value);
                }
                None
            }
            Expression::Assignment(target, value) => {
                let value = self.build_expression(builder, value)?;
                match target.as_ref() {
                    Expression::Variable(variable) => {
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
            Expression::Error(span) => panic!("IR contains errors at {:?}!", span),
        }
    }
}
