use crate::{ob, ra};
use ra::ty::Scalar;
use ra::ty::Ty;
use ra::ty::TyKind;

fn convert_scalar(pts: &impl ob::PrimitiveTypeSource, scalar: Scalar) -> ob::Type {
    use ra::ty::primitive::{FloatTy, IntTy, UintTy};
    match scalar {
        Scalar::Bool => pts.bool(),
        Scalar::Char => todo!(),
        Scalar::Int(IntTy::Isize) => pts.size_type(true),
        Scalar::Int(IntTy::I8) => pts.int(8, true),
        Scalar::Int(IntTy::I16) => pts.int(16, true),
        Scalar::Int(IntTy::I32) => pts.int(32, true),
        Scalar::Int(IntTy::I64) => pts.int(64, true),
        Scalar::Int(IntTy::I128) => pts.int(128, true),
        Scalar::Uint(UintTy::Usize) => pts.size_type(false),
        Scalar::Uint(UintTy::U8) => pts.int(8, false),
        Scalar::Uint(UintTy::U16) => pts.int(16, false),
        Scalar::Uint(UintTy::U32) => pts.int(32, false),
        Scalar::Uint(UintTy::U64) => pts.int(64, false),
        Scalar::Uint(UintTy::U128) => pts.int(128, false),
        Scalar::Float(FloatTy::F16) => pts.float(16),
        Scalar::Float(FloatTy::F32) => pts.float(32),
        Scalar::Float(FloatTy::F64) => pts.float(64),
        Scalar::Float(FloatTy::F128) => pts.float(128),
    }
}

pub fn convert(pts: &impl ob::PrimitiveTypeSource, ty: &Ty) -> ob::Type {
    match ty.kind(ra::ty::Interner) {
        TyKind::Adt(..) => todo!(),
        TyKind::AssociatedType(..) => todo!(),
        TyKind::Scalar(scalar) => convert_scalar(pts, *scalar),
        TyKind::Tuple(..) => todo!(),
        TyKind::Array(..) => todo!(),
        TyKind::Slice(..) => todo!(),
        TyKind::Raw(..) => todo!(),
        TyKind::Ref(..) => todo!(),
        TyKind::OpaqueType(..) => todo!(),
        TyKind::FnDef(..) => todo!(),
        TyKind::Str => todo!(),
        TyKind::Never => todo!(),
        TyKind::Closure(..) => todo!(),
        TyKind::Coroutine(..) => todo!(),
        TyKind::CoroutineWitness(..) => todo!(),
        TyKind::Foreign(..) => todo!(),
        TyKind::Error => todo!(),
        TyKind::Placeholder(..) => todo!(),
        TyKind::Dyn(..) => todo!(),
        TyKind::Alias(..) => todo!(),
        TyKind::Function(..) => todo!(),
        TyKind::BoundVar(..) => todo!(),
        TyKind::InferenceVar(..) => todo!(),
    }
}
