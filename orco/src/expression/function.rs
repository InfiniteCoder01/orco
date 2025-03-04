use crate::type_inference::intrinsics::Intrinsic;
use crate::types::FunctionSignature;

pub enum FunctionBody {
    Block(Vec<crate::Expression>),
    Intrinsic(Intrinsic),
}

/// Function, defined in a very non-rusty way (suggest me an enum that works)
pub struct Function {
    /// Function signature
    pub signature: FunctionSignature,
    /// Function name
    pub name: Option<String>,
    /// Function body
    pub body: FunctionBody,
}

impl Function {
    pub fn new(
        signature: FunctionSignature,
        name: Option<String>,
        body: Vec<crate::Expression>,
    ) -> Self {
        Self {
            signature,
            name,
            body: FunctionBody::Block(body),
        }
    }

    pub fn intrinsic(signature: FunctionSignature, intrinsic: Intrinsic) -> Self {
        Self {
            signature,
            name: None,
            body: FunctionBody::Intrinsic(intrinsic),
        }
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
                for expression in body {
                    writeln!(f, "{}", indent::indent_all_by(4, format!("{expression};")))?;
                }
                write!(f, "}}")?;
                Ok(())
            }
            FunctionBody::Intrinsic(intrinsic) => write!(f, " = {:?}", intrinsic),
        }
    }
}
