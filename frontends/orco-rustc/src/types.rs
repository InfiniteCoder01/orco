use crate::TyCtxt;
use crate::names::convert_path;

pub trait GenericMap {
    fn resolve(&self, param: rustc_middle::ty::ParamTy) -> orco::Type;
}

impl GenericMap for () {
    fn resolve(&self, param: rustc_middle::ty::ParamTy) -> orco::Type {
        orco::Type::Symbol(param.name.as_str().into())
    }
}

/// Convert a type from rust MIR to orco.
#[must_use]
pub fn convert<'a>(
    backend: &impl orco::DeclarationBackend<'a>,
    tcx: TyCtxt,
    ty: rustc_middle::ty::Ty,
    map: &impl GenericMap,
) -> Option<orco::Type> {
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
        TyKind::Adt(def, generics) => {
            let mut name = convert_path(tcx, def.did());
            let args = generics
                .iter()
                .filter_map(|generic| {
                    let ty = generic.as_type()?;
                    convert(backend, tcx, ty, map)
                })
                .collect::<Vec<_>>();

            if !args.is_empty() {
                backend.invoke_macro(name.as_str().into(), &args);
                for arg in args {
                    name.push('_');
                    name.push_str(&arg.hashable_name());
                }
            }

            orco::Type::Symbol(name.into())
        }
        TyKind::Foreign(..) => todo!(),
        TyKind::Str => orco::Type::Error,
        TyKind::Array(ty, _size) => {
            orco::Type::Array(Box::new(convert(backend, tcx, *ty, map)?), 42)
        } // TODO: Use size!
        TyKind::Pat(..) => todo!(),
        TyKind::Slice(..) => todo!(),
        TyKind::RawPtr(ty, mutability) => orco::Type::Ptr(
            Box::new(convert(backend, tcx, *ty, map).unwrap_or(orco::Type::Error)),
            mutability.is_mut(),
        ),
        TyKind::Ref(_, ty, mutability) => orco::Type::Ptr(
            Box::new(convert(backend, tcx, *ty, map).unwrap_or(orco::Type::Error)),
            mutability.is_mut(),
        ),
        TyKind::FnDef(..) => todo!(),
        TyKind::FnPtr(sig, _) => {
            let sig = sig.skip_binder();
            orco::Type::FnPtr {
                params: sig
                    .inputs()
                    .iter()
                    .flat_map(|ty| convert(backend, tcx, *ty, map))
                    .collect(),
                return_type: convert(backend, tcx, sig.output(), map).map(Box::new),
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
                .filter_map(|ty| convert(backend, tcx, ty, map).map(|ty| (None, ty)))
                .collect(),
        },
        TyKind::Alias(..) => todo!(),
        TyKind::Param(param) => map.resolve(*param),
        TyKind::Bound(..) => todo!(),
        TyKind::Placeholder(..) => todo!(),
        TyKind::Infer(var) => panic!("inference variable {var} found in type"),
        TyKind::Error(..) => orco::Type::Error,
    })
}
