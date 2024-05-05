use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::Module;

pub mod constant;

pub mod block;

pub mod operator;

pub mod branching;

impl crate::Object<'_> {
    pub fn build_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &orco::ir::Expression,
    ) -> Option<Value> {
        use orco::ir::Expression;
        match expr {
            Expression::Constant(value) => self.build_constant(builder, value),
            Expression::Variable(variable) => {
                let variable = variable.lock().unwrap();
                Some(builder.use_var(Variable::new(variable.id as _)))
            }
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
            orco::ir::expression::Expression::FunctionCall { name, args } => {
                let function = self.object.declare_func_in_func(
                    *self
                        .functions
                        .get(name)
                        .unwrap_or_else(|| panic!("Function {} is not defined", name)),
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
            Expression::Assignment(expr) => self.build_assignment_expression(builder, expr),
            Expression::Error(span) => panic!("IR contains errors at {:?}!", span),
        }
    }
}
