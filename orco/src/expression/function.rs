use crate::types::FunctionSignature;

pub enum FunctionBody {
    Block(Option<String>, Vec<crate::Expression>),
    External(String),
}

/// Function, defined in a very non-rusty way (suggest me an enum that works)
pub struct Function {
    /// Function signature
    pub signature: FunctionSignature,
    /// Function body
    pub body: FunctionBody,
}

impl Function {
    pub fn new(
        name: Option<String>,
        signature: FunctionSignature,
        body: Vec<crate::Expression>,
    ) -> Self {
        Self {
            signature,
            body: FunctionBody::Block(name, body),
        }
    }

    pub fn external(name: String, signature: FunctionSignature) -> Self {
        Self {
            signature,
            body: FunctionBody::External(name),
        }
    }

    pub fn name(&self) -> Option<&str> {
        match &self.body {
            FunctionBody::Block(name, _) => name.as_ref().map(String::as_str),
            FunctionBody::External(name) => Some(name),
        }
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.body {
            FunctionBody::Block(name, body) => {
                writeln!(
                    f,
                    "fn {}{} {{",
                    name.as_ref().map(String::as_str).unwrap_or_default(),
                    self.signature
                )?;
                for expression in body {
                    writeln!(f, "{}", indent::indent_all_by(4, format!("{expression};")))?;
                }
                write!(f, "}}")?;
                Ok(())
            }
            FunctionBody::External(name) => write!(f, "fn {}()", &name),
        }
    }
}
