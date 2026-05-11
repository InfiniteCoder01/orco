/// Attributes for a function
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionAttributes {
    /// Inlining mode
    pub inlining: Inlining,
}

impl std::fmt::Display for FunctionAttributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inlining != Inlining::Auto {
            write!(f, "[inline({})] ", self.inlining)?;
        }

        Ok(())
    }
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

impl std::fmt::Display for Inlining {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Inlining::Never => write!(f, "never"),
            Inlining::Auto => write!(f, "auto"),
            Inlining::Hint => write!(f, "hint"),
            Inlining::Always => write!(f, "always"),
        }
    }
}
