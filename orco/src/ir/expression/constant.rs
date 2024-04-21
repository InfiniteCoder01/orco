use super::*;

/// Constant value
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constant {
    /// Unsigned integer
    UnsignedInteger {
        /// Value
        value: u128,
        /// Size in bytes, None to infer
        size: Option<u16>,
    },
    /// Signed integer
    SignedInteger {
        /// Value
        value: i128,
        /// Size in bytes, None to infer
        size: Option<u16>,
    },
    /// C-Style String, bytes have to end with '\0'
    CString(Vec<u8>),
}

impl Constant {
    /// Infer types
    pub fn infer_types(&mut self, target_type: &Type) {
        match self {
            Self::UnsignedInteger { value, size } => {
                if size.is_none() {
                    match target_type {
                        Type::Int(target_size) => {
                            if let Ok(value) = (*value).try_into() {
                                *self = Self::SignedInteger {
                                    value,
                                    size: Some(target_size.get()),
                                }
                            }
                        }
                        Type::Unsigned(target_size) => *size = Some(target_size.get()),
                        _ => (),
                    }
                }
            }
            Self::SignedInteger { size, .. } => {
                if size.is_none() {
                    if let Type::Int(target_size) = target_type {
                        *size = Some(target_size.get())
                    }
                }
            }
            Self::CString(_) => (),
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsignedInteger { value, size } => {
                write!(f, "{}", value)?;
                if let Some(size) = size {
                    write!(f, "u{}", size * 8)?;
                }
                Ok(())
            }
            Self::SignedInteger { value, size } => {
                write!(f, "{}", value)?;
                if let Some(size) = size {
                    write!(f, "i{}", size * 8)?;
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
