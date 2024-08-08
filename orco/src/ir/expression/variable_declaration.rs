use super::*;
use std::sync::Mutex;

/// Variable declaration statement
#[derive(Derivative)]
#[derivative(Debug)]
pub struct VariableDeclaration {
    /// Variable name
    pub name: Name,
    /// Variable ID, just a counting up number assigned automatically, when calling
    /// Useful for some backends
    pub id: Mutex<VariableId>,
    /// Is variable mutable?
    pub mutable: Spanned<bool>,
    /// Variable type
    pub r#type: Spanned<Mutex<Type>>,
    /// Initial value (optional (I wish it was nesessarry))
    pub value: Option<Mutex<Expression>>,
    /// Span of the declaration
    #[derivative(Debug = "ignore")]
    pub span: Option<Span>,
    /// Metadata
    #[derivative(Debug = "ignore")]
    pub metadata: Box<dyn VariableDeclarationMetadata>,
}

/// Variable ID, for more information see [`VariableDeclaration::id`]
pub type VariableId = u64;

impl VariableDeclaration {
    /// Create a new variable declaration
    pub fn new(
        name: Name,
        mutable: Spanned<bool>,
        r#type: Spanned<Type>,
        value: Option<Expression>,
        span: Option<Span>,
        metadata: impl VariableDeclarationMetadata + 'static,
    ) -> Self {
        Self {
            name,
            id: Mutex::new(0),
            mutable,
            r#type: r#type.map(Mutex::new),
            value: value.map(Mutex::new),
            span,
            metadata: Box::new(metadata),
        }
    }

    /// Infer types
    pub fn infer_types(self: std::pin::Pin<&Self>, type_inference: &mut TypeInference) -> Type {
        *self.id.try_lock().unwrap() = type_inference.new_variable_id();
        let mut r#type = self.r#type.inner.try_lock().unwrap();
        type_inference.complete(&mut r#type);
        if let Some(value) = &self.value {
            let value_type = value.try_lock().unwrap().infer_types(type_inference);
            type_inference.equate(&r#type, &value_type);
        }

        type_inference.current_scope_mut().insert(
            self.name.clone(),
            SymbolReference::Variable(symbol_reference::InternalPointer::new(self)),
        );
        Type::Unit
    }

    /// Finish and check types
    pub fn finish_and_check_types(&self, type_inference: &mut TypeInference) -> Type {
        let mut r#type = self.r#type.inner.try_lock().unwrap();
        type_inference.finish(
            &mut r#type,
            &format!("variable '{}'", self.name),
            Some(self.name.clone()),
        );
        if let Some(value) = &self.value {
            let mut value = value.try_lock().unwrap();
            let value_type = value.finish_and_check_types(type_inference);
            if !value_type.morphs(&r#type) {
                self.metadata.variable_declaration_type_mismatch(
                    type_inference,
                    VariableDeclarationTypeMismatch {
                        expected: r#type.clone(),
                        got: value_type,
                        src: value.span().as_ref().unwrap().named_source(),
                        expression_span: value.span().as_ref().unwrap().source_span(),
                        declaration_span: self.r#type.span.as_ref().unwrap().source_span(),
                    },
                );
            }
        }
        Type::Unit
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Incompatible types for variable declaration: expected '{expected}', got '{got}'")]
#[diagnostic(code(typechecking::call::variable_declaration_type_mismatch))]
/// Variable declaration type mismatch
pub struct VariableDeclarationTypeMismatch {
    /// Expected type
    pub expected: Type,
    /// Got type
    pub got: Type,

    #[source_code]
    /// File where the error occurred
    pub src: NamedSource<Src>,
    #[label("Here")]
    /// Span of the expression
    pub expression_span: SourceSpan,
    #[label("Expected because of this")]
    /// Span of the type in declaration
    pub declaration_span: SourceSpan,
}
impl Clone for VariableDeclaration {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            id: Mutex::new(*self.id.try_lock().unwrap()),
            mutable: self.mutable.clone(),
            r#type: Spanned::opt(
                Mutex::new(self.r#type.try_lock().unwrap().clone()),
                self.r#type.span.clone(),
            ),
            value: self
                .value
                .as_ref()
                .map(|value| Mutex::new(value.try_lock().unwrap().clone())),
            span: self.span.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl std::fmt::Display for VariableDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let show_id = std::env::var("ORCO_SHOW_VAR_ID").map_or(false, |show_id| show_id == "1");
        write!(f, "let ")?;
        if self.mutable.inner {
            write!(f, "mut ")?;
        }
        write!(f, "{}", self.name)?;
        if show_id {
            write!(f, " (#{})", self.id.try_lock().unwrap())?;
        }
        write!(f, ": {}", self.r#type.try_lock().unwrap())?;
        if let Some(value) = &self.value {
            write!(f, " = {}", value.try_lock().unwrap())?;
        }
        Ok(())
    }
}

declare_metadata! {
    /// Frontend metadata for variable declaration
    trait VariableDeclarationMetadata {
        Diagnostics:
        /// Variable declaration type mismatch error callback
        variable_declaration_type_mismatch(VariableDeclarationTypeMismatch) abort_compilation;
    }
}
