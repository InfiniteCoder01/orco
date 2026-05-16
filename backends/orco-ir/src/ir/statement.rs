use orco::codegen as oc;

/// Basic instructions
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Statement {
    /// See [`orco::codegen::BodyCodegen::comment`]
    Comment(String),
    /// See [`orco::codegen::BodyCodegen::assign`]
    Assign(oc::Place, oc::Value),
    /// See [`orco::codegen::BodyCodegen::iconst`]
    IConst(i128, orco::types::IntegerSize),
    /// See [`orco::codegen::BodyCodegen::uconst`]
    UConst(u128, orco::types::IntegerSize),
    /// See [`orco::codegen::BodyCodegen::fconst`]
    FConst(f64, u16),
    /// See [`orco::codegen::BodyCodegen::fconst`]
    BConst(bool),
}

impl Statement {
    /// Weather this statement is an expression (it yields a value)
    pub fn is_expression(&self) -> bool {
        match self {
            Statement::Comment(..) => false,
            Statement::Assign(..) => false,
            Statement::IConst(..) => true,
            Statement::UConst(..) => true,
            Statement::FConst(..) => true,
            Statement::BConst(..) => true,
        }
    }
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Comment(comment) => {
                for (idx, line) in comment.split('\n').enumerate() {
                    if idx > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "// {line}")?;
                }
            }
            Self::Assign(target, value) => write!(f, "{target} = <{}>;", value.0)?,
            Self::IConst(value, size) => write!(f, "{value} as i{size}")?,
            Self::UConst(value, size) => write!(f, "{value} as u{size}")?,
            Self::FConst(value, size) => write!(f, "{value} as f{size}")?,
            Self::BConst(value) => write!(f, "{value}")?,
        }
        Ok(())
    }
}
