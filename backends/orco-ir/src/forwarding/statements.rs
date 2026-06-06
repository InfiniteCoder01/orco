use super::{ir, oc};

impl ir::Statement {
    /// Codegen this statement into another [`oc::BodyCodegen`],
    /// mapping all variables and labels (ACF)
    pub(super) fn codegen(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        map_variable: &impl Fn(oc::Variable) -> oc::Variable,
        map_label: impl Fn(oc::Label) -> oc::Label,
    ) {
        match self {
            Self::Comment(comment) => codegen.comment(comment),
            Self::Assign(place, value) => {
                let place = place.codegen(codegen, map_variable);
                let value = value.codegen(codegen, map_variable);
                codegen.assign(place, value)
            }
            Self::Call(func, args) => {
                let func = func.codegen(codegen, map_variable);
                let args = args
                    .iter()
                    .map(|arg| arg.codegen(codegen, map_variable))
                    .collect();
                if let Some(value) = codegen.call(func, args) {
                    codegen.mk_tmp(value);
                }
            }
            Self::Return(value) => {
                let value = value
                    .as_ref()
                    .map(|value| value.codegen(codegen, map_variable));
                codegen.return_(value)
            }

            Self::Acf(acf) => acf.codegen(codegen, map_variable, map_label),
            Self::Bcf(bcf) => bcf.codegen(codegen, map_variable),
        }
    }
}

impl ir::AcfStatement {
    /// Codegen this statement into another [`oc::BodyCodegen`],
    /// mapping all variables and labels (ACF)
    fn codegen(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        map_variable: &impl Fn(oc::Variable) -> oc::Variable,
        map_label: impl Fn(oc::Label) -> oc::Label,
    ) {
        use oc::AcfCodegen as _;
        match self {
            Self::Jump(label) => codegen.acf().jump(map_label(*label)),
            Self::Cjump(value, label) => {
                let value = value.codegen(codegen, map_variable);
                codegen.acf().cjump(value, map_label(*label))
            }
        }
    }
}

impl ir::BcfStatement {
    /// Codegen this statement into another [`oc::BodyCodegen`],
    /// mapping all variables and labels (ACF)
    fn codegen(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        map_variable: &impl Fn(oc::Variable) -> oc::Variable,
    ) {
        use oc::BcfCodegen as _;
        match self {
            Self::If(value) => {
                let value = value.codegen(codegen, map_variable);
                codegen.bcf().if_(value)
            }
            Self::Else => codegen.bcf().else_(),
            Self::End => codegen.bcf().end(),
            Self::Loop => codegen.bcf().loop_(),
            Self::Break => codegen.bcf().break_(),
            Self::Continue => codegen.bcf().continue_(),
            Self::Cbreak(value) => {
                let value = value.codegen(codegen, map_variable);
                codegen.bcf().cbreak(value)
            }
            Self::Ccontinue(value) => {
                let value = value.codegen(codegen, map_variable);
                codegen.bcf().ccontinue(value)
            }
        }
    }
}
