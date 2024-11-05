use super::*;

pub trait Return {
    fn expression(&self) -> Expression;
}

impl std::fmt::Display for &dyn Return {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "return {}",
            indent::indent_all_by(4, self.expression().to_string().trim())
        )
    }
}

pub struct SimpleReturn<'a> {
    pub expression: Expression<'a>,
}

impl<'a> SimpleReturn<'a> {
    pub fn new(expression: Expression<'a>) -> Self {
        Self { expression }
    }
}

impl Return for SimpleReturn<'_> {
    fn expression(&self) -> Expression {
        self.expression
    }
}

impl AsExpression for SimpleReturn<'_> {
    fn as_expression(&self) -> Expression {
        Expression::Return(self)
    }
}

impl std::fmt::Display for SimpleReturn<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn Return).fmt(f)
    }
}
