use super::{CodegenCtx, Instr};

impl CodegenCtx<'_, '_> {
    pub(super) fn place(&mut self, place: rustc_middle::mir::Place) {
        for (_, proj) in place.iter_projections().rev() {
            use rustc_middle::mir::ProjectionElem as PE;
            match proj {
                PE::Deref => todo!(),
                PE::Field(field, _) => self.instr(Instr::Field(field.as_u32())),
                PE::Index(_) => todo!(),
                PE::ConstantIndex { .. } => todo!(),
                PE::Subslice { .. } => todo!(),
                PE::Downcast(..) => todo!(),
                PE::OpaqueCast(..) => todo!(),
                PE::UnwrapUnsafeBinder(..) => todo!(),
            }
        }

        self.variables
            .get(&place.local)
            .copied()
            .map(|var| self.instr(Instr::Var(var)));
    }

    fn constant(&mut self, value: rustc_middle::mir::ConstValue, ty: rustc_middle::ty::Ty) {
        use rustc_const_eval::interpret::Scalar;
        use rustc_middle::mir::ConstValue;
        use rustc_middle::ty::TyKind;

        // TODO: Handle chars & bools
        match value {
            ConstValue::Scalar(Scalar::Int(value)) => {
                if ty.is_floating_point() {
                    self.instr(Instr::FConst(
                        match value.size().bytes() {
                            4 => f32::from_bits(value.to_u32()).into(),
                            8 => f64::from_bits(value.to_u64()) as _,
                            sz => {
                                panic!("invalid or unsupported floating point literal size: {sz}")
                            }
                        },
                        value.size().bits() as _,
                    ));
                } else if ty.is_signed() {
                    // TODO: Big values
                    let ivalue = value.to_int(value.size());
                    self.instr(Instr::IConst(
                        ivalue as _,
                        if ty.is_ptr_sized_integral() {
                            orco::types::IntegerSize::Size
                        } else {
                            orco::types::IntegerSize::Bits(value.size().bits() as _)
                        },
                    ));
                } else {
                    // TODO: Big values
                    let ivalue = value.to_uint(value.size());
                    self.instr(Instr::UConst(
                        ivalue as _,
                        if ty.is_ptr_sized_integral() {
                            orco::types::IntegerSize::Size
                        } else {
                            orco::types::IntegerSize::Bits(value.size().bits() as _)
                        },
                    ));
                }
            }
            ConstValue::Scalar(Scalar::Ptr(..)) => todo!(),
            ConstValue::ZeroSized => match ty.kind() {
                // TODO: We might need to do more
                // TODO: Generics
                TyKind::FnDef(func, generics) => {
                    let symbol = self.ir_body.use_symbol(
                        self.convert_path(*func),
                        self.convert_generic_args(generics.skip_binder()),
                    );
                    self.instr(Instr::Global(symbol));
                }
                TyKind::Adt(..) => {
                    self.convert_ty(ty).map(|ty| {
                        let var = self.ir_body.declare_var(ty, Some("zst".to_owned()));
                        self.instr(Instr::Var(var));
                    });
                }
                _ => panic!("Unknown zero-sized const {value:?}"),
            },
            ConstValue::Slice { .. } => todo!(),
            ConstValue::Indirect { .. } => todo!(),
        }
    }

    pub(super) fn op(&mut self, op: &rustc_middle::mir::Operand) {
        use rustc_middle::mir::{Const, Operand};
        match op {
            Operand::Copy(place) | Operand::Move(place) => {
                self.place(*place);
            }
            Operand::Constant(value) => match value.const_ {
                Const::Ty(..) => todo!(),
                Const::Unevaluated(uc, ..) => {
                    panic!("unevaluated const encountered ({uc:?})")
                }
                Const::Val(value, ty) => self.constant(value, ty),
            },
            Operand::RuntimeChecks(..) => todo!(),
        }
    }
}
