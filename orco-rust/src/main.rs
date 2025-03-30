pub mod hir;
pub use hir::Hir;

use orco::backend::Backend;

fn main() {
    let file = "orco-rust/samples/simple.rs";
    let mut ctx = hir::Context::default();
    let print_id = ctx.registry.declare_fn(
        "print".to_owned(),
        orco::Signature::new(vec![orco::Parameter::new(
            "value".to_owned(),
            orco::Type::Int(32),
        )]),
    );

    let mut hir = hir::Hir::new();
    hir::parse_file(&mut ctx, &mut hir, file, hir::Path::single("sample"));
    hir.resolve(&ctx);

    dbg!(&ctx.registry);

    let mut object = orco_cranelift::Object::new("x86_64-unknown-linux-gnu", ctx.registry);
    let mut print_sb = object.declare_function(print_id, "print");
    print_sb.external();
    print_sb.finish();

    for (id, function) in hir.functions.iter() {
        let function = function.read().unwrap();
        let mut sb = object.declare_function(*id, &function.path);
        sb.public();
        sb.finish();
    }

    for (id, function) in hir.functions.iter() {
        let function = function.read().unwrap();
        let body = &hir.bodies[function.body].read().unwrap();
        let mut builder = object.build_function(*id);
        let value = body.expression.build(builder.as_mut());
        builder.ret(value);
        builder.finish();
    }

    ctx.diag.emit();

    let object = object.finish();
    std::fs::write("foo.o", object.emit().unwrap()).unwrap();
}
