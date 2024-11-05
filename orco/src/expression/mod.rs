use super::*;

pub mod block;
pub use block::Block;
pub use block::SimpleBlock;
// pub mod control_flow;
// pub use control_flow::Return;
// pub use control_flow::SimpleReturn;
// pub mod constant;

pub enum Expression {
    Block(Box<dyn Block>),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block(block) => block.fmt(f),
        }
    }
}
