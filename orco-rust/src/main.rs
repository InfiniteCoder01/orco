pub mod hir;
pub use hir::Hir;

use orco::backend::Backend;

fn main() {
    let file = "orco-rust/samples/simple.rs";
    let mut ctx = hir::Context {
        diag: orco::diagnostic::DiagCtx::new(),
    };
    let mut hir = hir::Hir::new();
    hir::parse_file(&mut ctx, &mut hir, file, hir::Path::single("sample"));
    hir.resolve(&ctx);

    let mut object = orco_cranelift::Object::new("x86_64-unknown-linux-gnu");
    for (id, function) in hir.functions.iter_enumerated() {
        let function = function.read().unwrap();
        let sb = object.declare_function(orco::backend::FunctionId(id.into()), &function.path);
        sb.finish();
    }

    for (id, function) in hir.functions.iter_enumerated() {
        let function = function.read().unwrap();
        let body = &hir.bodies[function.body].read().unwrap();
        let mut builder = object.build_function(orco::backend::FunctionId(id.into()));
        let value = body.expression.build(builder.as_mut());
        builder.ret(value);
        builder.finish();
    }

    ctx.diag.emit();

    let object = object.finish();
    std::fs::write("foo.o", object.emit().unwrap()).unwrap();
}
