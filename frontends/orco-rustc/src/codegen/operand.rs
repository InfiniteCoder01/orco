use super::{CodegenCtx, oc};

impl<'tcx, 'a, CG: oc::BodyCodegen> CodegenCtx<'tcx, CG> {
    pub(super) fn place(&self, place: rustc_middle::mir::Place<'tcx>) -> oc::Place {
        let mut res = oc::Place::Variable(self.variables[place.local.index()]);
        for (place, proj) in place.iter_projections() {
            use rustc_middle::mir::ProjectionElem as PE;
            match proj {
                PE::Deref => res = oc::Place::Deref(Box::new(res)),
                PE::Field(field, _) => {
                    let ty = place.ty(&self.body.local_decls, self.tcx);
                    use rustc_middle::ty::TyKind as TK;
                    let field = match ty.ty.kind() {
                        TK::Adt(adt, _) => {
                            adt.variants()
                                [ty.variant_index.or(adt.variants().last_index()).unwrap()] // FIXME: this is a mess
                            .fields[field]
                                .name
                                .to_string()
                        }
                        TK::Tuple(_) => field.index().to_string(),
                        _ => panic!("Trying to access field of {ty:?}"),
                    };

                    res = oc::Place::Field(Box::new(res), field.into());
                }
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

    pub(super) fn op(&self, op: &rustc_middle::mir::Operand<'tcx>) -> oc::Operand {
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
                                        orco::IntegerSize::Size
                                    } else {
                                        orco::IntegerSize::Bits(value.size().bits() as _)
                                    },
                                )
                            } else {
                                oc::Operand::UConst(
                                    value.to_uint(value.size()),
                                    if ty.is_ptr_sized_integral() {
                                        orco::IntegerSize::Size
                                    } else {
                                        orco::IntegerSize::Bits(value.size().bits() as _)
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
                        rustc_middle::ty::TyKind::Adt(..) => oc::Operand::Unit, // TODO: Ain't working
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
