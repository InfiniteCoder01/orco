use super::*;

/// Constant value
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constant {
    /// Unsigned integer
    UnsignedInteger {
        /// Value
        value: u128,
        /// Type of this literal
        r#type: Type,
    },
    /// Signed integer
    SignedInteger {
        /// Value
        value: i128,
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
            Self::UnsignedInteger { r#type, .. } => r#type.clone(),
            Self::SignedInteger { r#type, .. } => r#type.clone(),
            Self::CString(_) => Type::Pointer(Box::new(Type::Char)),
        }
    }

    /// Infer types
    pub fn infer_types(&mut self, target_type: &Type, type_inference: &mut TypeInference) -> Type {
        match self {
            Self::UnsignedInteger { r#type, .. } | Self::SignedInteger { r#type, .. } => {
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
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        match self {
            Self::UnsignedInteger { r#type, .. } | Self::SignedInteger { r#type, .. } => {
                type_inference.finish(r#type)
            }
            _ => (),
        }
        self.get_type()
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsignedInteger { value, r#type } => {
                write!(f, "{}", value)?;
                if r#type != &Type::Wildcard {
                    write!(f, "{}", r#type)?;
                }
                Ok(())
            }
            Self::SignedInteger { value, r#type } => {
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
