use super::*;

/// Constant value
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constant {
    /// Unsigned integer
    Integer {
        /// Value
        value: u128,
        /// Type of this literal
        r#type: Type,
    },
    /// C-Style String, bytes have to end with '\0'
    CString(Vec<u8>),
}

impl Constant {
    /// Get the type of the constant value
    pub fn get_type(&self) -> Type {
        match self {
            Self::Integer { r#type, .. } => r#type.clone(),
            Self::CString(_) => Type::Pointer(Box::new(Type::Char)),
        }
    }

    /// Infer types
    pub fn infer_types(&mut self, target_type: &Type, type_inference: &mut TypeInference) -> Type {
        match self {
            Self::Integer { r#type, .. } => {
                if !r#type.complete() {
                    *r#type = if target_type.complete() {
                        target_type.clone()
                    } else {
                        Type::TypeVariable(type_inference.alloc_type_variable(r#type.clone()))
                    };
                }
            }
            Self::CString(_) => (),
        }
        self.get_type()
    }

    /// Finish and check types
    pub fn finish_and_check_types(
        &mut self,
        span: Span,
        type_inference: &mut TypeInference,
    ) -> Type {
        #[allow(clippy::single_match)]
        match self {
            Self::Integer { r#type, value } => {
                type_inference.finish(r#type, "constant", span.clone());
                let fits = match r#type {
                    Type::Unsigned(size) if size.get() == 16 => true,
                    Type::Unsigned(size) => *value < 1 << (size.get() * 8),
                    Type::Int(size) => *value < 1 << (size.get() * 8 - 1),
                    r#type => unimplemented!("{}", r#type),
                };
                if !fits {
                    type_inference.reporter.report_type_error(
                        format!(
                            "Integer literal '{}' doesn't fit in the type '{}'",
                            value, r#type
                        ),
                        span,
                        vec![],
                    );
                }
            }
            _ => (),
        }
        self.get_type()
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer { value, r#type } => {
                write!(f, "{}", value)?;
                if r#type != &Type::Wildcard {
                    write!(f, "{}", r#type)?;
                }
                Ok(())
            }
            Self::CString(bytes) => {
                if let Ok(str) = std::str::from_utf8(bytes) {
                    write!(f, "c{:?}", str)
                } else {
                    write!(f, "c\"")?;
                    for byte in bytes {
                        write!(f, "\\x{:02x}", byte)?;
                    }
                    write!(f, "\"")?;
                    Ok(())
                }
            }
        }
    }
}
