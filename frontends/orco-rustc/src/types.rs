use crate::TyCtxt;
use crate::names::convert_path;

/// Convert a type from rust MIR to orco.
/// Pass your backend as the type source
pub fn convert(
    tcx: TyCtxt,
    pts: &impl orco::PrimitiveTypeSource,
    ty: rustc_middle::ty::Ty,
) -> orco::Type {
    use rustc_middle::ty::{FloatTy, IntTy, TyKind, UintTy};
    match ty.kind() {
        TyKind::Bool => pts.bool(),
        TyKind::Char => todo!(),
        TyKind::Int(IntTy::I8) => pts.int(8, true),
        TyKind::Int(IntTy::I16) => pts.int(16, true),
        TyKind::Int(IntTy::I32) => pts.int(32, true),
        TyKind::Int(IntTy::I64) => pts.int(64, true),
        TyKind::Int(IntTy::I128) => pts.int(128, true),
        TyKind::Int(IntTy::Isize) => pts.size_type(true),
        TyKind::Uint(UintTy::U8) => pts.int(8, false),
        TyKind::Uint(UintTy::U16) => pts.int(16, false),
        TyKind::Uint(UintTy::U32) => pts.int(32, false),
        TyKind::Uint(UintTy::U64) => pts.int(64, false),
        TyKind::Uint(UintTy::U128) => pts.int(128, false),
        TyKind::Uint(UintTy::Usize) => pts.size_type(false),
        TyKind::Float(FloatTy::F16) => pts.float(16),
        TyKind::Float(FloatTy::F32) => pts.float(32),
        TyKind::Float(FloatTy::F64) => pts.float(64),
        TyKind::Float(FloatTy::F128) => pts.float(128),
        TyKind::Adt(def, generics) => orco::Type::Symbol(convert_path(tcx, def.did())), // TODO: Generics
        TyKind::Foreign(..) => todo!(),
        TyKind::Str => todo!(),
        TyKind::Array(ty, size) => orco::Type::Array(Box::new(convert(tcx, pts, *ty)), 42), // TODO: Use size!
        TyKind::Pat(..) => todo!(),
        TyKind::Slice(..) => todo!(),
        TyKind::RawPtr(..) => todo!(),
        TyKind::Ref(..) => todo!(),
        TyKind::FnDef(..) => todo!(),
        TyKind::FnPtr(..) => todo!(),
        TyKind::UnsafeBinder(..) => todo!(),
        TyKind::Dynamic(..) => todo!(),
        TyKind::Closure(..) => todo!(),
        TyKind::CoroutineClosure(..) => todo!(),
        TyKind::Coroutine(..) => todo!(),
        TyKind::CoroutineWitness(..) => todo!(),
        TyKind::Never => todo!(),
        TyKind::Tuple(v) if v.is_empty() => pts.unit(),
        TyKind::Tuple(..) => todo!(),
        TyKind::Alias(..) => todo!(),
        TyKind::Param(param) => orco::Type::Symbol(param.name.as_str().into()), // TODO: Generics?
        TyKind::Bound(..) => todo!(),
        TyKind::Placeholder(..) => todo!(),
        TyKind::Infer(var) => panic!("inference variable {var} found in type"),
        TyKind::Error(..) => orco::Type::Error,
    }
}
