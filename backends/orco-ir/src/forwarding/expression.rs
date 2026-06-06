use super::{ir, oc};

impl ir::Place {
    /// Convert this place into [`oc::Place`],
    /// while generating code for inner expressions using
    /// [`ir::Expression::codegen`]
    pub(super) fn codegen(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        map_variable: &impl Fn(oc::Variable) -> oc::Variable,
    ) -> oc::Place {
        match self {
            ir::Place::Variable(variable) => map_variable(*variable).into(),
            ir::Place::Global(symbol) => oc::Place::Global(*symbol),
            ir::Place::Deref(value) => oc::Place::Deref(value.codegen(codegen, map_variable)),
            ir::Place::Field(place, idx) => place.codegen(codegen, map_variable).field(*idx),
        }
    }
}

impl ir::Expression {
    /// Codegen this expression into another [`oc::BodyCodegen`],
    /// mapping all variables
    pub(super) fn codegen(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        map_variable: &impl Fn(oc::Variable) -> oc::Variable,
    ) -> oc::Value {
        match self {
            Self::IConst(value, size) => codegen.iconst(*value, *size),
            Self::UConst(value, size) => codegen.uconst(*value, *size),
            Self::FConst(value, size) => codegen.fconst(*value, *size),
            Self::BConst(value) => codegen.bconst(*value),
            Self::Read(place) => {
                let place = place.codegen(codegen, map_variable);
                codegen.read(place)
            }
            Self::Reference(place, mutable) => {
                let place = place.codegen(codegen, map_variable);
                codegen.reference(place, *mutable)
            }
            Self::Call(func, args) => {
                let func = func.codegen(codegen, map_variable);
                let args = args
                    .iter()
                    .map(|arg| arg.codegen(codegen, map_variable))
                    .collect();
                codegen
                    .call(func, args)
                    .unwrap_or_else(|| panic!("trying to use value from calling a void function"))
            }

            Self::Intrinsic(intrinsic) => {
                use crate::ir::Intrinsic as I;
                use oc::Intrinsics as IT;
                match intrinsic {
                    I::Add(a, b) => {
                        let a = a.codegen(codegen, map_variable);
                        let b = b.codegen(codegen, map_variable);
                        codegen.intrinsics().add(a, b)
                    }
                    I::Mul(a, b) => {
                        let a = a.codegen(codegen, map_variable);
                        let b = b.codegen(codegen, map_variable);
                        codegen.intrinsics().mul(a, b)
                    }
                    I::Eq(a, b) => {
                        let a = a.codegen(codegen, map_variable);
                        let b = b.codegen(codegen, map_variable);
                        codegen.intrinsics().eq(a, b)
                    }
                    I::Not(a) => {
                        let a = a.codegen(codegen, map_variable);
                        codegen.intrinsics().not(a)
                    }
                }
            }
        }
    }
}
