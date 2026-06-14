mod expressions;
pub use expressions::{Expression, Place};

mod statements;
pub use statements::{AcfStatement, BcfStatement, Statement};

mod intrinsics;
pub use intrinsics::Intrinsic;

/// Info about one variable in a body
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable {
    /// Type of this variable
    pub ty: orco::Type,
    /// Wether this variable comes from function arguments
    pub arg: bool,
    /// Debug name
    pub name: Option<String>,
}

/// A function body
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct Body {
    /// All variables used in the body.
    /// Index this with [`orco::codegen::Variable::0`]
    pub variables: Vec<Variable>,
    /// Labels for ACF (see [`orco::codegen::AcfCodegen`]).
    /// [`orco::codegen::Label::0`] is an index into this vector,
    /// while values are indices into [`Self::statements`]
    pub labels: Vec<usize>,
    /// See [Statement]
    pub statements: Vec<Statement>,
}

impl Body {
    /// Shortcut to access [`Self::variables`]
    #[must_use]
    pub fn get_variable(&self, variable: orco::codegen::Variable) -> &Variable {
        self.variables
            .get(variable.0)
            .unwrap_or_else(|| panic!("invalid variable _{}", variable.0))
    }
}

impl std::fmt::Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for (idx, var) in self.variables.iter().enumerate() {
            write!(f, "  let _{idx}: {}", var.ty)?;
            if var.arg {
                write!(f, " = <argument>")?;
            }
            write!(f, ";")?;
            if let Some(name) = &var.name {
                write!(f, " // {name}")?;
            }
            writeln!(f)?;
        }

        let mut statement_idx_to_label = std::collections::HashMap::new();
        for (idx, label) in self.labels.iter().enumerate() {
            statement_idx_to_label.insert(label, idx);
        }

        let mut indent = 1;
        for (idx, statement) in self.statements.iter().enumerate() {
            if let Some(label) = statement_idx_to_label.get(&idx) {
                writeln!(f, "label{label}:")?;
            }

            if matches!(
                statement,
                Statement::Bcf(BcfStatement::Else | BcfStatement::End)
            ) {
                indent -= 1;
            }

            for line in statement.to_string().split('\n') {
                for _ in 0..indent {
                    write!(f, "  ")?;
                }
                writeln!(f, "{line}")?;
            }

            if matches!(
                statement,
                Statement::Bcf(BcfStatement::If(..) | BcfStatement::Else | BcfStatement::Loop)
            ) {
                indent += 1
            }
        }
        write!(f, "}}")
    }
}
