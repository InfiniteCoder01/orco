pub struct Call {
    /// Function to call
    pub function: crate::ArcLock<crate::expression::Function>,
    /// Args for the function
    pub args: Vec<crate::Expression>,
}

                .unwrap_or("<unnamed function>")
        )?;
        for (index, arg) in self.args.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            arg.fmt(f)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
