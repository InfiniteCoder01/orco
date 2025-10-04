use super::{CodegenCtx, oc};

impl<'tcx, 'a, CG: oc::Codegen<'a>> CodegenCtx<'tcx, CG> {
    pub(super) fn var(&self, place: rustc_middle::mir::Place) -> oc::Variable {
        self.variables[place.local.index()] // TODO: projection
    }

    pub(super) fn op(&self, op: &rustc_middle::mir::Operand) -> oc::Operand {
        use rustc_const_eval::interpret::Scalar;
        use rustc_middle::mir::{Const, ConstValue, Operand};
        match op {
            Operand::Copy(place) => oc::Operand::Variable(self.var(*place)),
            Operand::Move(place) => oc::Operand::Variable(self.var(*place)),
            Operand::Constant(value) => match value.const_ {
                Const::Ty(..) => todo!(),
                Const::Unevaluated(uc, ..) => {
                    panic!("unevaluated const encountered ({uc:?})")
                }
                Const::Val(value, ty) => match value {
                    ConstValue::Scalar(scalar) => match scalar {
                        Scalar::Int(value) => oc::Operand::UConst(value.to_bits(value.size())),
                        Scalar::Ptr(..) => todo!(),
                    },
                    ConstValue::ZeroSized => match ty.kind() {
                        // TODO: We might need to do more
                        rustc_middle::ty::TyKind::FnDef(func, ..) => {
                            // TODO: Generics
                            oc::Operand::Global(crate::declare::convert_path(self.tcx, *func))
                        }
                        _ => panic!("Unknown zero-sized const {op:?}"),
                    },
                    ConstValue::Slice { .. } => todo!(),
                    ConstValue::Indirect { .. } => todo!(),
                },
            },
        }
    }
}
