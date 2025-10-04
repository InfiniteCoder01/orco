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
            Operand::Constant(value) => {
                let (value, ty) = match value.const_ {
                    Const::Ty(..) => todo!(),
                    Const::Unevaluated(uc, ..) => {
                        panic!("unevaluated const encountered ({uc:?})")
                    }
                    Const::Val(value, ty) => (value, ty),
                };
                match value {
                    ConstValue::Scalar(scalar) => match scalar {
                        Scalar::Int(value) => {
                            if ty.is_floating_point() {
                                oc::Operand::FConst(
                                    match value.size().bytes() {
                                        4 => f32::from_bits(value.to_u32()) as _,
                                        8 => f64::from_bits(value.to_u64()) as _,
                                        sz => panic!(
                                            "invalid or unsupported floating point literal size: {sz}"
                                        ),
                                    },
                                    value.size().bits() as _,
                                )
                            } else if ty.is_signed() {
                                oc::Operand::IConst(
                                    value.to_int(value.size()),
                                    if ty.is_ptr_sized_integral() {
                                        oc::IntegerSize::Size
                                    } else {
                                        oc::IntegerSize::Bits(value.size().bits() as _)
                                    },
                                )
                            } else {
                                oc::Operand::UConst(
                                    value.to_uint(value.size()),
                                    if ty.is_ptr_sized_integral() {
                                        oc::IntegerSize::Size
                                    } else {
                                        oc::IntegerSize::Bits(value.size().bits() as _)
                                    },
                                )
                            }
                        }
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
                }
            }
        }
    }
}
