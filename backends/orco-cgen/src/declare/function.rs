use super::Type;

/// Function signature using C [Type]s without a name
/// (see [`super::Declaration`] for name and generics).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionSignature {
    /// Parameter types with optional names
    pub params: Vec<(Type, Option<String>)>,
    /// Return type
    pub ret: Type,
}
