use crate::TyCtxt;

/// Convert a type from rust MIR to orco.
#[must_use]
pub fn convert(tcx: TyCtxt, ty: rustc_middle::ty::Ty) -> Option<orco::Type> {
    use rustc_middle::ty::{FloatTy, IntTy, TyKind, UintTy};
    Some(match ty.kind() {
        TyKind::Bool => orco::Type::Bool,
        TyKind::Char => orco::Type::Char(true),
        TyKind::Int(sz) => orco::Type::Integer(match sz {
            IntTy::Isize => orco::types::IntegerSize::Size,
            IntTy::I8 => orco::types::IntegerSize::Bits(8),
            IntTy::I16 => orco::types::IntegerSize::Bits(16),
            IntTy::I32 => orco::types::IntegerSize::Bits(32),
            IntTy::I64 => orco::types::IntegerSize::Bits(64),
            IntTy::I128 => orco::types::IntegerSize::Bits(128),
        }),
        TyKind::Uint(sz) => orco::Type::Unsigned(match sz {
            UintTy::Usize => orco::types::IntegerSize::Size,
            UintTy::U8 => orco::types::IntegerSize::Bits(8),
            UintTy::U16 => orco::types::IntegerSize::Bits(16),
            UintTy::U32 => orco::types::IntegerSize::Bits(32),
            UintTy::U64 => orco::types::IntegerSize::Bits(64),
            UintTy::U128 => orco::types::IntegerSize::Bits(128),
        }),
        TyKind::Float(sz) => orco::Type::Float(match sz {
            FloatTy::F16 => 16,
            FloatTy::F32 => 32,
            FloatTy::F64 => 64,
            FloatTy::F128 => 128,
        }),
        TyKind::Adt(def, generics) => orco::Type::Symbol(
            crate::names::convert_path(tcx, def.did()).into(),
            convert_generic_args(tcx, generics),
        ),
        TyKind::Foreign(..) => todo!(),
        TyKind::Str => orco::Type::Error,
        TyKind::Array(ty, _size) => orco::Type::Array(Box::new(convert(tcx, *ty)?), 42), // TODO: Use size!
        TyKind::Pat(..) => todo!(),
        TyKind::Slice(..) => todo!(),
        TyKind::RawPtr(ty, mutability) => orco::Type::Ptr(
            Box::new(convert(tcx, *ty).unwrap_or(orco::Type::Error)),
            mutability.is_mut(),
        ),
        TyKind::Ref(_, ty, mutability) => orco::Type::Ptr(
            Box::new(convert(tcx, *ty).unwrap_or(orco::Type::Error)),
            mutability.is_mut(),
        ),
        TyKind::FnDef(..) => todo!(),
        TyKind::FnPtr(sig, _) => {
            let sig = sig.skip_binder();
            orco::Type::FnPtr {
                params: sig
                    .inputs()
                    .iter()
                    .flat_map(|ty| convert(tcx, *ty))
                    .collect(),
                return_type: convert(tcx, sig.output()).map(Box::new),
            }
        }
        TyKind::UnsafeBinder(..) => todo!(),
        TyKind::Dynamic(..) => todo!(),
        TyKind::Closure(..) => todo!(),
        TyKind::CoroutineClosure(..) => todo!(),
        TyKind::Coroutine(..) => todo!(),
        TyKind::CoroutineWitness(..) => todo!(),
        TyKind::Never => todo!(),
        TyKind::Tuple(v) if v.is_empty() => return None,
        TyKind::Tuple(v) => orco::Type::Struct {
            fields: v
                .iter()
                .filter_map(|ty| convert(tcx, ty).map(|ty| (None, ty)))
                .collect(),
        },
        TyKind::Alias(..) => todo!(),
        TyKind::Param(param) => orco::Type::Param(param.name.as_str().into()),
        TyKind::Bound(..) => todo!(),
        TyKind::Placeholder(..) => todo!(),
        TyKind::Infer(var) => panic!("inference variable {var} found in type"),
        TyKind::Error(..) => orco::Type::Error,
    })
}

/// Convert MIR generic argument into [`orco::Type`]
pub fn convert_generic_arg(tcx: TyCtxt, arg: rustc_middle::ty::GenericArg) -> Option<orco::Type> {
    use rustc_middle::ty::GenericArgKind as GAK;
    match arg.kind() {
        GAK::Lifetime(_) => None,
        GAK::Type(ty) => convert(tcx, ty),
        GAK::Const(value) => todo!("const generics: {value}"),
    }
}

/// Convert a list of generic args, see [`convert_generic_arg`]
pub fn convert_generic_args(tcx: TyCtxt, args: &rustc_middle::ty::GenericArgs) -> Vec<orco::Type> {
    args.iter()
        .flat_map(|arg| convert_generic_arg(tcx, arg))
        .collect()
}

pub fn convert_generic_params(tcx: TyCtxt, key: rustc_hir::def_id::DefId) -> Vec<orco::Type> {
    let generics = tcx.generics_of(key);
    let mut types = generics
        .parent
        .map_or_else(Default::default, |key| convert_generic_params(tcx, key));
    for param in &generics.own_params {
        types.push(orco::Type::Param(param.name.as_str().into()));
    }
    types
}
