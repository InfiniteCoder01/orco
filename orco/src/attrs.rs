/// Attributes for a function
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionAttributes {
    /// Inlining mode
    pub inlining: Inlining,
}

/// Inlining mode, [`Inlining::Auto`] by default
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Inlining {
    /// Never inline the function
    Never,
    /// Automatically decide, whether to inline a function for optimization purpuses
    #[default]
    Auto,
    /// Suggest compiler to inline the function, but not force it
    Hint,
    /// Always try to inline the function
    Always,
}
