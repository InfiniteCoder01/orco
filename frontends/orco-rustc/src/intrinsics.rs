use orco::Type;
use orco::codegen::{BodyCodegen as _, Intrinsics as _};

fn integers(mut cb: impl FnMut(Type)) {
    use orco::types::IntegerSize as IS;
    for bits in [8, 16, 32, 64] {
        // TODO: i128
        cb(Type::Integer(IS::Bits(bits)));
        cb(Type::Unsigned(IS::Bits(bits)));
    }
    cb(Type::Integer(IS::Size));
    cb(Type::Unsigned(IS::Size));
}

fn tuple2(ty1: Type, ty2: Type) -> Type {
    Type::Struct {
        fields: vec![(None, ty1), (None, ty2)],
    }
}

/// Declares rust's intrinsics
pub fn declare<'a>(backend: &impl orco::DeclarationBackend<'a>) {
    let intrinsic = |name: String, params, rt| {
        backend.function(
            name.into(),
            params,
            Some(rt),
            orco::attrs::FunctionAttributes {
                inlining: orco::attrs::Inlining::Always,
                ..orco::attrs::FunctionAttributes::default()
            },
        )
    };

    integers(|ty| {
        intrinsic(
            format!("__MulWithOverflow#{}", ty.hashable_name()),
            vec![(None, ty.clone()), (None, ty.clone())],
            tuple2(ty.clone(), Type::Bool),
        );
        intrinsic(
            format!("__AddWithOverflow#{}", ty.hashable_name()),
            vec![(None, ty.clone()), (None, ty.clone())],
            tuple2(ty.clone(), Type::Bool),
        );
    });
}

/// Codegens rust's intrinsics
pub fn codegen(backend: &impl orco::CodegenBackend) {
    integers(|ty| {
        let mut cg = backend.function(format!("__MulWithOverflow#{}", ty.hashable_name()).into());
        let a = cg.read(orco::codegen::Variable(0).into());
        let b = cg.read(orco::codegen::Variable(1).into());
        let result = cg.declare_var(tuple2(ty.clone(), Type::Bool));
        let sum = cg.intrinsics().mul(a, b);
        cg.assign(result.place().field(0), sum);
        let cfalse = cg.bconst(false);
        cg.assign(result.place().field(1), cfalse);
        let result = cg.read(result.into());
        cg.return_(Some(result));
        let mut cg = backend.function(format!("__AddWithOverflow#{}", ty.hashable_name()).into());
        let a = cg.read(orco::codegen::Variable(0).into());
        let b = cg.read(orco::codegen::Variable(1).into());
        let result = cg.declare_var(tuple2(ty.clone(), Type::Bool));
        let sum = cg.intrinsics().add(a, b);
        cg.assign(result.place().field(0), sum);
        let cfalse = cg.bconst(false);
        cg.assign(result.place().field(1), cfalse);
        let result = cg.read(result.into());
        cg.return_(Some(result));
    });
}
