use crate::TyCtxt;

/// Convert a type from rust MIR to orco.
#[must_use]
pub fn convert<'a>(
    tcx: TyCtxt,
    backend: &impl orco::DeclarationBackend<'a>,
    ty: rustc_middle::ty::Ty,
    map: GenericMap,
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
        TyKind::Adt(def, generics) => orco::Type::Symbol(crate::names::generic_name(
            tcx,
            backend,
            def.did(),
            map,
            generics,
        )),
        TyKind::Foreign(..) => todo!(),
        TyKind::Str => orco::Type::Error,
        TyKind::Array(ty, _size) => {
            orco::Type::Array(Box::new(convert(tcx, backend, *ty, map)?), 42)
        } // TODO: Use size!
        TyKind::Pat(..) => todo!(),
        TyKind::Slice(..) => todo!(),
        TyKind::RawPtr(ty, mutability) => orco::Type::Ptr(
            Box::new(convert(tcx, backend, *ty, map).unwrap_or(orco::Type::Error)),
            mutability.is_mut(),
        ),
        TyKind::Ref(_, ty, mutability) => orco::Type::Ptr(
            Box::new(convert(tcx, backend, *ty, map).unwrap_or(orco::Type::Error)),
            mutability.is_mut(),
        ),
        TyKind::FnDef(..) => todo!(),
        TyKind::FnPtr(sig, _) => {
            let sig = sig.skip_binder();
            orco::Type::FnPtr {
                params: sig
                    .inputs()
                    .iter()
                    .flat_map(|ty| convert(tcx, backend, *ty, map))
                    .collect(),
                return_type: convert(tcx, backend, sig.output(), map).map(Box::new),
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
                .filter_map(|ty| convert(tcx, backend, ty, map).map(|ty| (None, ty)))
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

/// Resolve generics during type conversion
#[derive(Clone, Copy, Debug, Default)]
pub struct GenericMap<'a>(pub usize, pub &'a [orco::Type]);

impl<'a> GenericMap<'a> {
    #![allow(missing_docs)]
    pub fn new(generics: &rustc_middle::ty::Generics, args: &'a [orco::Type]) -> Self {
        let counts = generics.own_counts();
        let offset = generics.has_self as usize + counts.lifetimes;
        Self(offset, args)
    }

    /// Resolve [`rustc_middle::ty::ParamTy`] to [`orco::Type`]
    pub fn resolve(&self, param: rustc_middle::ty::ParamTy) -> orco::Type {
        if self.generic() {
            if param.index == 0 {
                self.1[0].clone()
            } else {
                self.1[param.index as usize - self.0].clone()
            }
        } else {
            orco::Type::Symbol(param.name.as_str().into())
        }
    }

    pub fn args(&self) -> &[orco::Type] {
        self.1
    }

    /// Whether this has any generics
    pub fn generic(&self) -> bool {
        self.0 != 0 || !self.1.is_empty()
    }
}

/// Decides, weather to wrap the items in a macro.
/// `name_key` is passed in separately for ability to
/// use names from trait declarations
pub fn wrap_generics<'a, 'tcx: 'a, B>(
    tcx: TyCtxt<'tcx>,
    backend: &B,
    key: rustc_hir::def_id::DefId,
    name_key: rustc_hir::def_id::DefId,
    macro_prefix: &str,
    callback: impl Fn(TyCtxt<'tcx>, &B, orco::Symbol, GenericMap) + Send + Sync + 'a,
) where
    B: orco::DeclarationBackend<'a>,
{
    let generics = tcx.generics_of(key);
    let name = crate::names::convert_path(tcx, name_key);
    if !generics.requires_monomorphization(tcx) {
        callback(tcx, backend, name.into(), GenericMap::new(generics, &[]));
    } else {
        let tcx = rustc_data_structures::sync::check_dyn_thread_safe()
            .expect(
                "You have to enable `-Z threads` (f.e. `-Z threads=sync`) to be able to use macros",
            )
            .derive(tcx);

        backend.macro_(
            format!("{macro_prefix}{name}").into(),
            move |backend, args| {
                let mut name = name.clone();
                for arg in args {
                    name.push('_');
                    name.push_str(&arg.hashable_name());
                }

                callback(*tcx, backend, name.into(), GenericMap::new(generics, args));
            },
            true,
        );
    }
}
