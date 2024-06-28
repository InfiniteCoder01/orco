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
            Self::CString(..) => Type::Pointer(Box::new(Type::Char)),
        }
    }

    /// Infer types
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        match self {
            Self::Integer { r#type, .. } => {
                *r#type = type_inference.complete(r#type.clone());
            }
            Self::Float { r#type, .. } => {
                *r#type = type_inference.complete(r#type.clone());
            }
            Self::CString(..) => (),
        }
        self.get_type()
    }

    /// Finish and check types
    pub fn finish_and_check_types(
        &mut self,
        span: Span,
        type_inference: &mut TypeInference,
    ) -> Type {
        match self {
            Self::Integer {
                r#type,
                value,
                metadata,
            } => {
                type_inference.finish(r#type, &metadata.name(), span.clone());
                let fits = match r#type {
                    Type::Unsigned(size) if size.get() == 16 => true,
                    Type::Unsigned(size) => *value < 1 << (size.get() * 8),
                    Type::Int(size) => *value < 1 << (size.get() * 8 - 1),
                    ref r#type if !r#type.complete() => true,
                    r#type => unimplemented!("{}", r#type),
                };
                if !fits {
                    metadata.integer_literal_doesnt_fit(
                        type_inference,
                        IntegerLiteralDoesntFit {
                            value: *value,
                            r#type: r#type.clone(),
                            src: span.named_source(),
                            span: span.source_span(),
                        },
                    )
                }
            }
            Self::Float {
                r#type, metadata, ..
            } => {
                type_inference.finish(r#type, &metadata.name(), span.clone());
            }
            _ => (),
        }
        self.get_type()
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Integer literal '{value}' doesn't fit in the type '{r#type}'")]
#[diagnostic(code(typechecking::constant::integer_literal_doesnt_fit))]
/// Integer literal doesn't fit
pub struct IntegerLiteralDoesntFit {
    /// Integer literal value
    pub value: u128,
    /// Type inferred for the integer literal
    pub r#type: Type,

    #[source_code]
    /// Source file where the error occurred
    pub src: NamedSource<Src>,
    #[label("Here")]
    /// Span of the integer literal
    pub span: SourceSpan,
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
        fn name(&self) -> std::borrow::Cow<'static, str> {
            std::borrow::Cow::Borrowed("integer constant")
        }

        Errors:
        /// Callback of integer literal doesn't fit error
        integer_literal_doesnt_fit(IntegerLiteralDoesntFit)
    }

    /// Frontend metadata for float constant
    trait FloatMetadata {
        /// Name provider for the constant
        fn name(&self) -> std::borrow::Cow<'static, str> {
            std::borrow::Cow::Borrowed("float constant")
        }
    }

    /// Frontend metadata for C String
    trait CStringMetadata {}
}
