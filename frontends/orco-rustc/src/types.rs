use crate::TyCtxt;
use crate::names::convert_path;

/// Convert a type from rust MIR to orco.
/// Pass your backend as the type source
pub fn convert(tcx: TyCtxt, ty: rustc_middle::ty::Ty) -> orco::Type {
    use rustc_middle::ty::{FloatTy, IntTy, TyKind, UintTy};
    match ty.kind() {
        TyKind::Bool => orco::Type::Bool,
        TyKind::Char => todo!(),
        TyKind::Int(sz) => orco::Type::Integer(match sz {
            IntTy::Isize => orco::IntegerSize::Size,
            IntTy::I8 => orco::IntegerSize::Bits(8),
            IntTy::I16 => orco::IntegerSize::Bits(16),
            IntTy::I32 => orco::IntegerSize::Bits(32),
            IntTy::I64 => orco::IntegerSize::Bits(64),
            IntTy::I128 => orco::IntegerSize::Bits(128),
        }),
        TyKind::Uint(sz) => orco::Type::Unsigned(match sz {
            UintTy::Usize => orco::IntegerSize::Size,
            UintTy::U8 => orco::IntegerSize::Bits(8),
            UintTy::U16 => orco::IntegerSize::Bits(16),
            UintTy::U32 => orco::IntegerSize::Bits(32),
            UintTy::U64 => orco::IntegerSize::Bits(64),
            UintTy::U128 => orco::IntegerSize::Bits(128),
        }),
        TyKind::Float(sz) => orco::Type::Float(match sz {
            FloatTy::F16 => 16,
            FloatTy::F32 => 32,
            FloatTy::F64 => 64,
            FloatTy::F128 => 128,
        }),
        TyKind::Adt(def, generics) => orco::Type::Symbol(
            std::iter::once(convert_path(tcx, def.did()))
                .chain(
                    generics
                        .iter()
                        .flat_map(|generic| generic.as_term().map(|term| term.to_string())),
                )
                .collect::<Vec<_>>()
                .join("#")
                .into(),
        ),
        TyKind::Foreign(..) => todo!(),
        TyKind::Str => todo!(),
        TyKind::Array(ty, _size) => orco::Type::Array(Box::new(convert(tcx, *ty)), 42), // TODO: Use size!
        TyKind::Pat(..) => todo!(),
        TyKind::Slice(..) => todo!(),
        TyKind::RawPtr(..) => todo!(),
        TyKind::Ref(_, ty, mutability) => {
            orco::Type::Ptr(Box::new(convert(tcx, *ty)), mutability.is_mut())
        }
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
                .map(|(idx, ty)| {
                    let name = idx.to_string();
                    (
                        if name.chars().next().is_none_or(|c| c.is_ascii_digit()) {
                            None
                        } else {
                            Some(name)
                        },
                        convert(tcx, ty),
                    )
                })
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
