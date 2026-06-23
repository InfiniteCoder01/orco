use super::{CodegenCtx, oc};

impl<'tcx, B: orco::DeclarationBackend<'tcx>, CG: oc::BodyCodegen> CodegenCtx<'_, 'tcx, B, CG> {
    pub(super) fn place(&mut self, place: rustc_middle::mir::Place<'tcx>) -> Option<oc::Place> {
        let mut res = oc::Place::Variable(self.variables[&place.local]?);
        for (_, proj) in place.iter_projections() {
            use rustc_middle::mir::ProjectionElem as PE;
            match proj {
                PE::Deref => res = oc::Place::Deref(self.codegen.read(res)),
                PE::Field(field, _) => res = oc::Place::Field(Box::new(res), field.index()),
                PE::Index(_) => todo!(),
                PE::ConstantIndex { .. } => todo!(),
                PE::Subslice { .. } => todo!(),
                PE::Downcast(..) => todo!(),
                PE::OpaqueCast(..) => todo!(),
                PE::UnwrapUnsafeBinder(..) => todo!(),
            }
        }
        Some(res)
    }

    pub(super) fn op(&mut self, op: &rustc_middle::mir::Operand<'tcx>) -> Option<oc::Value> {
        use rustc_const_eval::interpret::Scalar;
        use rustc_middle::mir::{Const, ConstValue, Operand};
        Some(match op {
            Operand::Copy(place) | Operand::Move(place) => {
                let place = self.place(*place)?;
                self.codegen.read(place)
            }
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
                                self.codegen.fconst(
                                    match value.size().bytes() {
                                        4 => f32::from_bits(value.to_u32()).into(),
                                        8 => f64::from_bits(value.to_u64()) as _,
                                        sz => panic!(
                                            "invalid or unsupported floating point literal size: {sz}"
                                        ),
                                    },
                                    value.size().bits() as _,
                                )
                            } else if ty.is_signed() {
                                self.codegen.iconst(
                                    value.to_int(value.size()),
                                    if ty.is_ptr_sized_integral() {
                                        orco::types::IntegerSize::Size
                                    } else {
                                        orco::types::IntegerSize::Bits(value.size().bits() as _)
                                    },
                                )
                            } else {
                                self.codegen.uconst(
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
                        rustc_middle::ty::TyKind::FnDef(func, generics) => self
                            .codegen
                            .read(oc::Place::Global(self.generic_name(*func, generics))),
                        rustc_middle::ty::TyKind::Adt(..) => {
                            let var = self.codegen.declare_var(self.convert_ty(ty)?, Some("zst"));
                            self.codegen.read(var.into())
                        }
                        _ => panic!("Unknown zero-sized const {op:?}"),
                    },
                    ConstValue::Slice { .. } => todo!(),
                    ConstValue::Indirect { .. } => todo!(),
                }
            }
            Operand::RuntimeChecks(..) => todo!(),
        })
    }
}
