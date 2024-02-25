use super::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A function
pub struct Function {
    /// Body of the function
    pub body: Vec<Expression>,
}
