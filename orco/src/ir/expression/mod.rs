#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// An expression
pub enum Expression {
    /// Return a value
    Return(Box<Expression>),
    /// A unit value, called "void" in C langauge family. Can have only one variant
    Unit,
}
