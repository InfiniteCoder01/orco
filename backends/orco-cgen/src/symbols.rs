use crate::FmtType;
use orco::Function;

/// Formats function signature
pub struct FmtFunction<'a> {
    /// Function name.
    pub name: &'a str,
    /// Function itself.
    pub function: &'a Function,
    /// Wether to name all args (assign placeholder names)?
    pub name_all_args: bool,
}

impl std::fmt::Display for FmtFunction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FmtFunction {
            name,
            function,
            name_all_args,
        } = *self;

        use orco::attrs as oa;
        match function.attrs.inlining {
            oa::Inlining::Never => write!(f, "__attribute__ ((noinline)) ")?,
            oa::Inlining::Auto => (),
            oa::Inlining::Hint => write!(f, "inline ")?,
            oa::Inlining::Always => write!(f, "__attribute__ ((always_inline)) ")?,
        }

        let mut sig_noret = name.to_owned();

        use std::fmt::Write as _;
        write!(&mut sig_noret, "(")?;
        for (idx, (name, ty)) in function.params.iter().enumerate() {
            if idx > 0 {
                write!(sig_noret, ", ")?;
            }
            write!(
                sig_noret,
                "{}",
                FmtType {
                    ty,
                    constant: false,
                    name: match name {
                        Some(name) => Some(name.to_owned()),
                        None if name_all_args => Some(format!("arg{idx}")),
                        None => None,
                    }
                    .as_deref()
                }
            )?;
        }
        write!(sig_noret, ")")?;

        FmtType {
            ty: function
                .return_type
                .as_ref()
                .unwrap_or(&orco::Type::Symbol("void".into(), Vec::new())),
            constant: false,
            name: Some(&sig_noret),
        }
        .fmt(f)
    }
}
