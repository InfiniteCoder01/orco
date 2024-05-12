use super::*;

impl crate::Object<'_> {
    /// Build an if expression
    pub fn build_if_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &orco::ir::expression::IfExpression,
    ) -> Option<Value> {
        // TODO: If values
        let condition = self.build_expression(builder, &expr.condition).expect("Can't pass a unit type as an argument to an if statement, did you run type checking/inference?");
        let then_block = builder.create_block();
        let else_block = if expr.else_branch.is_some() {
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
        self.build_expression(builder, &expr.then_branch);
        if expr.then_branch.get_type() != orco::ir::Type::Never {
            builder.ins().jump(merge_block, &[]);
        }

        if let (Some(else_branch), Some(else_block)) = (&expr.else_branch, else_block) {
            builder.switch_to_block(else_block);
            self.build_expression(builder, else_branch);
            builder.ins().jump(merge_block, &[]);
            builder.seal_block(else_block);
        }

        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);
        None
    }
}
