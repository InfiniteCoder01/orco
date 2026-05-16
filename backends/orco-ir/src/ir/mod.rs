mod statement;
pub use statement::Statement;

/// Info about one variable in a body
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable {
    /// Type of this variable
    pub ty: orco::Type,
    /// Wether this variable comes from function arguments
    pub arg: bool,
}

/// A function body
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct Body {
    /// All variables used in the body.
    /// Index this with [`orco::codegen::Variable::0`]
    pub variables: Vec<Variable>,
    /// See [Statement]
    pub statements: Vec<Statement>,
}

impl std::fmt::Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for (idx, var) in self.variables.iter().enumerate() {
            write!(f, "  let _{idx}: {}", var.ty)?;
            if var.arg {
                write!(f, " = <argument>")?;
            }
            writeln!(f, ";")?;
        }
        for (idx, statement) in self.statements.iter().enumerate() {
            if statement.is_expression() {
                writeln!(f, "  <{idx}> = {statement};")?;
                continue;
            }

            for line in statement.to_string().split('\n') {
                writeln!(f, "  {line}")?;
            }
        }
        write!(f, "}}")
    }
}
