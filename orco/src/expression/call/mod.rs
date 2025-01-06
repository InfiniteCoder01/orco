use super::*;

/// Calls are everything that happens.
/// `break`, `return`, adding two numbers
/// together are all function calls
#[repr(transparent)]
pub struct Call(soa_rs::Slice<InternalExpression>);

impl Call {
    pub fn function(&self) -> FunctionIndex {
        FunctionIndex(*self.0.idx(1).value)
    }

    pub fn args(&self) -> &Expressions {
        unsafe { &*(&raw const *self.0.idx(2..) as *const _) }
    }

    pub fn args_mut(&mut self) -> &mut Expressions {
        unsafe { &mut *(&raw mut *self.0.idx_mut(2..) as *mut _) }
    }
}

// impl std::fmt::Display for Call {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}(",
//             self.function
//                 .read()
//                 .unwrap()
//                 .name()
//                 .unwrap_or("<unnamed function>")
//         )?;
//         for (index, arg) in self.args.iter().enumerate() {
//             if index > 0 {
//                 write!(f, ", ")?;
//             }
//             arg.fmt(f)?;
//         }
//         write!(f, ")")?;
//         Ok(())
//     }
// }
