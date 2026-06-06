use super::{Expression, Place};
use orco::codegen as oc;

/// Basic statements
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Statement {
    /// See [`oc::BodyCodegen::comment`]
    Comment(String),
    /// See [`oc::BodyCodegen::assign`]
    Assign(Place, Expression),
    /// See [`oc::BodyCodegen::call`].
    /// For functions which don't return a value
    Call(Expression, Vec<Expression>),
    /// See [`oc::BodyCodegen::return`]
    Return(Option<Expression>),

    /// See [`oc::BodyCodegen::acf`]
    Acf(AcfStatement),
    /// See [`oc::BodyCodegen::bcf`]
    Bcf(BcfStatement),
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
            Self::Assign(target, value) => write!(f, "{target} = {value};")?,
            Self::Call(func, args) => {
                write!(f, "{func}(")?;
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")?;
            }
            Self::Return(value) => {
                write!(f, "return")?;
                if let Some(value) = value {
                    write!(f, " {value}")?;
                }
                write!(f, ";")?;
            }

            Self::Acf(acf) => write!(f, "{acf}")?,
            Self::Bcf(bcf) => write!(f, "{bcf}")?,
        }
        Ok(())
    }
}

/// Arbitrary control flow statements.
/// See [`oc::AcfCodegen`]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum AcfStatement {
    /// See [`oc::AcfCodegen::jump`]
    Jump(oc::Label),
    /// See [`oc::AcfCodegen::cjump`]
    Cjump(Expression, oc::Label),
}

impl std::fmt::Display for AcfStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Jump(label) => write!(f, "jump label{};", label.0),
            Self::Cjump(value, label) => write!(f, "jump label{} if {value};", label.0),
        }
    }
}

/// Block-like control flow statements (classic, flattened).
/// See [`oc::BcfCodegen`]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum BcfStatement {
    /// See [`oc::BcfCodegen::if_`]
    If(Expression),
    /// See [`oc::BcfCodegen::else_`]
    Else,
    /// See [`oc::BcfCodegen::end`]
    End,
    /// See [`oc::BcfCodegen::loop_`]
    Loop,
    /// See [`oc::BcfCodegen::break`]
    Break,
    /// See [`oc::BcfCodegen::continue`]
    Continue,
    /// See [`oc::BcfCodegen::cbreak`]
    Cbreak(Expression),
    /// See [`oc::BcfCodegen::ccontinue`]
    Ccontinue(Expression),
}

impl std::fmt::Display for BcfStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::If(condition) => write!(f, "if {condition} {{"),
            Self::Else => write!(f, "}} else {{"),
            Self::End => write!(f, "}}"),
            Self::Loop => write!(f, "loop {{"),
            Self::Break => write!(f, "break;"),
            Self::Continue => write!(f, "continue;"),
            Self::Cbreak(value) => write!(f, "break if {value};"),
            Self::Ccontinue(value) => write!(f, "continue if {value};"),
        }
    }
}
