use rustc_public::CrateDef as _;

/// Convert a type from rust MIR to orco.
#[must_use]
pub fn convert(ty: rustc_public::ty::Ty) -> orco::Type {
    use rustc_public::ty::{FloatTy, RigidTy, TyKind};
    let ty = match ty.kind() {
        TyKind::RigidTy(ty) => ty,
        TyKind::Alias(_, alias) => {
            eprintln!("HUH? Alias? {:?}", ty.kind());
            return orco::Type::Symbol(
                alias.def_id.name().into(),
                convert_generic_args(&alias.args),
            );
        }
        TyKind::Param(param) => return orco::Type::Param(param.name.as_str().into()),
        TyKind::Bound(_, _) => todo!(),
    };

    match ty {
        RigidTy::Bool => orco::Type::Bool,
        RigidTy::Char => orco::Type::Char(true),
        RigidTy::Int(sz) => orco::Type::Integer(match sz {
            rustc_public::ty::IntTy::Isize => orco::ty::IntegerSize::Size,
            sz => orco::ty::IntegerSize::Bits(sz.num_bytes() as u8 * 8),
        }),
        RigidTy::Uint(sz) => orco::Type::Unsigned(match sz {
            rustc_public::ty::UintTy::Usize => orco::ty::IntegerSize::Size,
            sz => orco::ty::IntegerSize::Bits(sz.num_bytes() as u8 * 8),
        }),
        RigidTy::Float(sz) => orco::Type::Float(match sz {
            FloatTy::F16 => 16,
            FloatTy::F32 => 32,
            FloatTy::F64 => 64,
            FloatTy::F128 => 128,
        }),
        RigidTy::Adt(def, args) => {
            orco::Type::Symbol(def.name().into(), convert_generic_args(&args))
        }
        RigidTy::Foreign(..) => todo!(),
        RigidTy::Str => orco::Type::Error,
        RigidTy::Array(ty, _size) => orco::Type::Array(Box::new(convert(ty)), 42), // TODO: Use size!
        RigidTy::Pat(..) => todo!(),
        RigidTy::Slice(..) => todo!(),
        RigidTy::RawPtr(ty, mutability) => orco::Type::Ptr(
            Box::new(convert(ty)),
            mutability == rustc_public::mir::Mutability::Mut,
        ),
        RigidTy::Ref(_, ty, mutability) => orco::Type::Ptr(
            Box::new(convert(ty)),
            mutability == rustc_public::mir::Mutability::Mut,
        ),
        RigidTy::FnDef(..) => todo!(),
        RigidTy::FnPtr(sig) => {
            let sig = sig.skip_binder();
            orco::Type::FnPtr {
                params: sig.inputs().iter().copied().map(convert).collect(),
                return_type: Box::new(convert(sig.output())),
            }
        }
        RigidTy::Dynamic(..) => todo!(),
        RigidTy::Closure(..) => todo!(),
        RigidTy::CoroutineClosure(..) => todo!(),
        RigidTy::Coroutine(..) => todo!(),
        RigidTy::CoroutineWitness(..) => todo!(),
        RigidTy::Never => todo!(),
        RigidTy::Tuple(v) if v.is_empty() => orco::Type::Unit,
        RigidTy::Tuple(v) => orco::Type::Struct {
            fields: v.iter().map(|ty| (None, convert(*ty))).collect(),
        },
    }
}

/// Convert MIR generic argument into [`orco::Type`]
fn convert_generic_arg(arg: &rustc_public::ty::GenericArgKind) -> Option<orco::Type> {
    use rustc_public::ty::GenericArgKind as GAK;
    match arg {
        GAK::Lifetime(_) => None,
        GAK::Type(ty) => Some(convert(*ty)),
        GAK::Const(value) => todo!("const generics: {value:?}"),
    }
}

/// Convert a list of generic args, see [`convert_generic_arg`]
pub fn convert_generic_args(args: &rustc_public::ty::GenericArgs) -> Vec<orco::Type> {
    args.0.iter().flat_map(convert_generic_arg).collect()
}

/// Get a list of generic param names
pub fn convert_generic_params(generics: &rustc_public::ty::Generics) -> Vec<orco::Symbol> {
    // TODO: parent?
    let mut types = Vec::new();
    for param in &generics.params {
        if param.kind == rustc_public::ty::GenericParamDefKind::Lifetime {
            continue;
        }
        types.push(param.name.as_str().into());
    }
    types
}
