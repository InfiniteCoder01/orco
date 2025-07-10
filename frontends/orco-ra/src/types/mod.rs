use crate::{ob, ra};

fn dirty_get_kind<'a>(ty: &'a ra::hir::Type<'a>) -> &'a ra::ty::TyKind {
    pub struct TypeLayout {
        env: triomphe::Arc<ra::ty::traits::TraitEnvironment>,
        ty: ra::ty::Ty,
    }
    unsafe { std::mem::transmute::<_, &TypeLayout>(ty) }
        .ty
        .kind(ra::ty::Interner)
}

impl crate::RAFrontend {
    pub fn convert_type<TB: ob::TypeBackend>(
        &self,
        backend: &mut TB,
        ty: &ra::hir::Type,
    ) -> ob::Type {
        let kind = dirty_get_kind(ty);
        if ty.is_unit() {
            ob::Type::Symbol(backend.unit())
        } else if ty.is_scalar() {
            let scalar = match kind {
                ra::ty::TyKind::Scalar(scalar) => scalar,
                _ => unreachable!(),
            };
            use ra_ap_hir_ty::Scalar;
            use ra_ap_hir_ty::primitive::{IntTy, UintTy};
            ob::Type::Symbol(match scalar {
                Scalar::Bool => todo!(),
                Scalar::Char => todo!(),
                Scalar::Int(IntTy::Isize) => backend.size_type(true),
                Scalar::Int(ty) => backend.int(
                    match ty {
                        IntTy::Isize => unreachable!(),
                        IntTy::I8 => 8,
                        IntTy::I16 => 16,
                        IntTy::I32 => 32,
                        IntTy::I64 => 64,
                        IntTy::I128 => 128,
                    },
                    true,
                ),
                Scalar::Uint(UintTy::Usize) => backend.size_type(false),
                Scalar::Uint(ty) => backend.int(
                    match ty {
                        UintTy::Usize => unreachable!(),
                        UintTy::U8 => 8,
                        UintTy::U16 => 16,
                        UintTy::U32 => 32,
                        UintTy::U64 => 64,
                        UintTy::U128 => 128,
                    },
                    false,
                ),
                Scalar::Float(float_ty) => todo!(),
            })
        } else {
            panic!("unsupported rust type: {:#?}", ty)
        }
    }
}
