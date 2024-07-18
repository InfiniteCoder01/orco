use super::*;

impl Flowchart {
    /// Add an expression to this flowchart
    /// Layer should be left as zero, unless the expression is connected to another node
    pub fn render_expression(&mut self, expression: &ir::Expression, layer: usize) {
        self.layers
            .resize_with(self.layers.len().max(layer + 1), Vec::new);

        use ir::Expression;
        let node = match expression {
            Expression::Constant(constant) => Node {
                label: constant.inner.to_string(),
                children: None,
            },
            Expression::Symbol(_, _) => todo!(),
            Expression::BinaryExpression(expr) => {
                let index = self.layers.get(layer + 1).map_or(0, Vec::len);
                self.render_expression(&expr.lhs, layer + 1);
                self.render_expression(&expr.rhs, layer + 1);
                Node {
                    label: expr.op.to_string(),
                    children: Some(index..=index + 1),
                }
            }
            Expression::UnaryExpression(_) => todo!(),
            Expression::Block(_) => todo!(),
            Expression::If(_) => todo!(),
            Expression::Call(_) => todo!(),
            Expression::Return(_) => todo!(),
            Expression::VariableDeclaration(_) => todo!(),
            Expression::Assignment(_) => todo!(),
            Expression::Error(_) => todo!(),
        };
        self.layers[layer].push(node);
    }
}
