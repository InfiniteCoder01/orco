use super::*;

/// Constant value
#[derive(Derivative, Clone)]
#[derivative(Debug, PartialEq)]
pub enum Constant {
    /// Unsigned integer
    Integer {
        /// Value
        value: u128,
        /// Type of this literal
        r#type: Type,
        /// Metadata
        #[derivative(Debug = "ignore", PartialEq = "ignore")]
        metadata: Box<dyn IntegerMetadata>,
    },
    /// Floating point number
    Float {
        /// Value
        value: f64,
        /// Type of this literal
        r#type: Type,
        /// Metadata
        #[derivative(Debug = "ignore", PartialEq = "ignore")]
        metadata: Box<dyn FloatMetadata>,
    },
    /// C-Style String, bytes should end with '\0'
    CString(
        Vec<u8>,
        #[derivative(Debug = "ignore", PartialEq = "ignore")] Box<dyn CStringMetadata>,
    ),
}

impl Constant {
    /// Get the type of the constant value
    pub fn get_type(&self) -> Type {
        match self {
            Self::Integer { r#type, .. } => r#type.clone(),
            Self::Float { r#type, .. } => r#type.clone(),
            Self::CString(..) => Type::Pointer(Box::new(Type::Char), false),
        }
    }

    /// Infer types
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        match self {
            Self::Integer { r#type, .. } => type_inference.complete(r#type),
            Self::Float { r#type, .. } => type_inference.complete(r#type),
            Self::CString(..) => (),
        }
        self.get_type()
    }

    /// Finish and check types
    pub fn finish_and_check_types(
        &mut self,
        span: &Option<Span>,
        type_inference: &mut TypeInference,
    ) -> Type {
        match self {
            Self::Integer {
                r#type,
                value,
                metadata,
            } => {
                type_inference.finish(r#type, &metadata.name(), span.as_ref());
                let fits = match r#type {
                    Type::Unsigned(size) if size.get() == 16 => true,
                    Type::Unsigned(size) => *value < 1 << (size.get() * 8),
                    Type::Int(size) => *value < 1 << (size.get() * 8 - 1),
                    ref r#type if !r#type.complete() => true,
                    r#type => unimplemented!("{}", r#type),
                };
                if !fits {
                    type_inference.report(metadata.integer_literal_doesnt_fit(
                        *value,
                        r#type,
                        span.clone(),
                    ));
                }
            }
            Self::Float {
                r#type, metadata, ..
            } => {
                type_inference.finish(r#type, &metadata.name(), span.as_ref());
            }
            _ => (),
        }
        self.get_type()
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer { value, r#type, .. } => {
                write!(f, "{}", value)?;
                if r#type != &Type::Wildcard {
                    write!(f, "{}", r#type)?;
                }
                Ok(())
            }
            Self::Float { value, r#type, .. } => {
                write!(f, "{}", value)?;
                if r#type != &Type::Wildcard {
                    write!(f, "{}", r#type)?;
                }
                Ok(())
            }
            Self::CString(bytes, ..) => {
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

declare_metadata! {
    /// Frontend metadata for integer constant
    trait IntegerMetadata {
        /// Name provider for the constant
        fn name(&self) -> std::borrow::Cow<str> {
            std::borrow::Cow::Borrowed("integer constant")
        }

        /// Callback of integer literal doesn't fit error
        fn integer_literal_doesnt_fit(&self, value: u128, r#type: &Type, span: Option<Span>) -> Report {
            Report::build(ReportKind::Error)
                .with_code("typechecking::integer_literal_doesnt_fit")
                .with_message(format!("Integer literal '{value}' doesn't fit in the type '{type}'"))
                .opt_label(span, |label| label.with_message(format!("Integer literal '{value}' doesn't fit in the type '{type}'")).with_color(colors::Label))
                .finish()
        }
    }

    /// Frontend metadata for float constant
    trait FloatMetadata {
        /// Name provider for the constant
        fn name(&self) -> std::borrow::Cow<str> {
            std::borrow::Cow::Borrowed("float constant")
        }
    }

    /// Frontend metadata for C String
    trait CStringMetadata {}
}
