use super::*;

pub trait Block {
    fn statements<'a>(&'a self) -> DynIter<'a, &'a Expression>;
    fn statements_mut<'a>(&'a mut self) -> DynIter<'a, &'a mut Expression>;
}

impl std::fmt::Display for dyn Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for statement in self.statements() {
            writeln!(f, "{}", indent::indent_all_by(4, format!("{statement};")))?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

pub struct SimpleBlock {
    pub statements: Vec<Expression>,
}

impl SimpleBlock {
    pub fn new(statements: Vec<Expression>) -> Self {
        Self { statements }
    }
}

impl Block for SimpleBlock {
    fn statements<'a>(&'a self) -> DynIter<'a, &'a Expression> {
        Box::new(self.statements.iter())
    }

    fn statements_mut<'a>(&'a mut self) -> DynIter<'a, &'a mut Expression> {
        Box::new(self.statements.iter_mut())
    }
}

impl std::fmt::Display for SimpleBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn Block).fmt(f)
    }
}
