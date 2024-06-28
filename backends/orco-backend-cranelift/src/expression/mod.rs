use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::{FunctionBuilder, Variable};

/// Build constants
pub mod constant;

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
            Expression::Constant(value) => self.build_constant(builder, value),
            Expression::Symbol(symbol, ..) => match &symbol.inner {
                orco::SymbolReference::Variable(variable) => {
                    Some(builder.use_var(Variable::new(*variable.id.lock().unwrap() as _)))
                }
                _ => {
                    panic!(
                        "Invalid symbol: {}. Did you run type checking/inference?",
                        symbol.inner
                    )
                }
            },
            Expression::BinaryExpression(expr) => self.build_binary_expression(builder, expr),
            Expression::UnaryExpression(expr) => self.build_unary_expression(builder, expr),
            Expression::Block(block) => self.build_block(builder, block),
            Expression::If(expr) => self.build_if_expression(builder, expr),
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
            Expression::Call(expr) => self.build_call_expression(builder, expr),
            Expression::Return(value) => {
                let ret = self.build_expression(builder, &value.0);
                builder.ins().return_(&ret.into_iter().collect::<Vec<_>>());
                None
            }
            Expression::VariableDeclaration(declaration) => {
                let variable = Variable::new(*declaration.id.lock().unwrap() as _);
                builder.declare_var(
                    variable,
                    self.convert_type(&declaration.r#type.lock().unwrap()),
                );
                if let Some(value) = &declaration.value {
                    let value = self.build_expression(builder, &value.lock().unwrap()).expect("Can't initialize a variable to a unit type, did you run type checking/inference?");
                    builder.def_var(variable, value);
                }
                None
            }
            Expression::Assignment(expr) => self.build_assignment_expression(builder, expr),
            Expression::Error(span) => panic!("IR contains errors at {:?}!", span),
        }
    }
}
