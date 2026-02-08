use super::{CodegenCtx, oc};

impl<'tcx, CG: oc::BodyCodegen> CodegenCtx<'tcx, CG> {
    pub(super) fn place(&self, place: rustc_middle::mir::Place<'tcx>) -> oc::Place {
        let mut res = oc::Place::Variable(self.variables[place.local.index()]);
        for (_, proj) in place.iter_projections() {
            use rustc_middle::mir::ProjectionElem as PE;
            match proj {
                PE::Deref => res = oc::Place::Deref(Box::new(res)),
                PE::Field(field, _) => res = oc::Place::Field(Box::new(res), field.index()),
                PE::Index(_) => todo!(),
                PE::ConstantIndex { .. } => todo!(),
                PE::Subslice { .. } => todo!(),
                PE::Downcast(..) => todo!(),
                PE::OpaqueCast(..) => todo!(),
                PE::UnwrapUnsafeBinder(..) => todo!(),
            }
        }
        res
    }

    pub(super) fn op(&mut self, op: &rustc_middle::mir::Operand<'tcx>) -> oc::Operand {
        use rustc_const_eval::interpret::Scalar;
        use rustc_middle::mir::{Const, ConstValue, Operand};
        match op {
            Operand::Copy(place) => oc::Operand::Place(self.place(*place)),
            Operand::Move(place) => oc::Operand::Place(self.place(*place)),
            Operand::Constant(value) => {
                let (value, ty) = match value.const_ {
                    Const::Ty(..) => todo!(),
                    Const::Unevaluated(uc, ..) => {
                        panic!("unevaluated const encountered ({uc:?})")
                    }
                    Const::Val(value, ty) => (value, ty),
                };
                // TODO: Handle chars & bools
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
                                        orco::types::IntegerSize::Size
                                    } else {
                                        orco::types::IntegerSize::Bits(value.size().bits() as _)
                                    },
                                )
                            } else {
                                oc::Operand::UConst(
                                    value.to_uint(value.size()),
                                    if ty.is_ptr_sized_integral() {
                                        orco::types::IntegerSize::Size
                                    } else {
                                        orco::types::IntegerSize::Bits(value.size().bits() as _)
                                    },
                                )
                            }
                        }
                        Scalar::Ptr(..) => todo!(),
                    },
                    ConstValue::ZeroSized => match ty.kind() {
                        // TODO: We might need to do more
                        // TODO: Generics
                        rustc_middle::ty::TyKind::FnDef(func, ..) => oc::Operand::Place(
                            oc::Place::Global(crate::names::convert_path(self.tcx, *func).into()),
                        ),
                        rustc_middle::ty::TyKind::Adt(..) => {
                            let var = self
                                .codegen
                                .declare_var(crate::types::convert(self.tcx, ty));
                            oc::Operand::Place(oc::Place::Variable(var))
                        }
                        _ => panic!("Unknown zero-sized const {op:?}"),
                    },
                    ConstValue::Slice { .. } => todo!(),
                    ConstValue::Indirect { .. } => todo!(),
                }
            }
            Operand::RuntimeChecks(..) => todo!(),
        }
    }
}
