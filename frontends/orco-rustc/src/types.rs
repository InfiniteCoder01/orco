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
        TyKind::Int(sz) => pts.int(
            match sz {
                IntTy::Isize => orco::IntegerSize::Size,
                IntTy::I8 => orco::IntegerSize::Bits(8),
                IntTy::I16 => orco::IntegerSize::Bits(16),
                IntTy::I32 => orco::IntegerSize::Bits(32),
                IntTy::I64 => orco::IntegerSize::Bits(64),
                IntTy::I128 => orco::IntegerSize::Bits(128),
            },
            true,
        ),
        TyKind::Uint(sz) => pts.int(
            match sz {
                UintTy::Usize => orco::IntegerSize::Size,
                UintTy::U8 => orco::IntegerSize::Bits(8),
                UintTy::U16 => orco::IntegerSize::Bits(16),
                UintTy::U32 => orco::IntegerSize::Bits(32),
                UintTy::U64 => orco::IntegerSize::Bits(64),
                UintTy::U128 => orco::IntegerSize::Bits(128),
            },
            false,
        ),
        TyKind::Float(sz) => pts.float(match sz {
            FloatTy::F16 => 16,
            FloatTy::F32 => 32,
            FloatTy::F64 => 64,
            FloatTy::F128 => 128,
        }),
        TyKind::Adt(def, _generics) => orco::Type::Symbol(convert_path(tcx, def.did())), // TODO: Generics
        TyKind::Foreign(..) => todo!(),
        TyKind::Str => todo!(),
        TyKind::Array(ty, _size) => orco::Type::Array(Box::new(convert(tcx, pts, *ty)), 42), // TODO: Use size!
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
        TyKind::Tuple(v) => orco::Type::Struct(
            v.iter()
                .enumerate()
                .map(|(idx, ty)| (idx.to_string().into(), convert(tcx, pts, ty)))
                .collect(),
        ),
        TyKind::Alias(..) => todo!(),
        TyKind::Param(param) => orco::Type::Symbol(param.name.as_str().into()), // TODO: Generics?
        TyKind::Bound(..) => todo!(),
        TyKind::Placeholder(..) => todo!(),
        TyKind::Infer(var) => panic!("inference variable {var} found in type"),
        TyKind::Error(..) => orco::Type::Error,
    }
}
