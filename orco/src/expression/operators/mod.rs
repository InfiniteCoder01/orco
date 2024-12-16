use super::*;

pub trait Operator {
    //
}

#[debug_display]
impl std::fmt::Display for &dyn Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
