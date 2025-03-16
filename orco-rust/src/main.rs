pub mod hir;
pub use hir::Hir;

pub mod diagnostic;
pub use diagnostic::DiagCtx;

pub mod backend;
use backend::cl::InstBuilder;

fn main() {
    let file = "orco-rust/samples/simple.rs";
    let mut ctx = hir::Context {
        diag: DiagCtx::new(),
    };
    let mut hir = hir::Hir::new();
    hir::parse_file(&mut ctx, &mut hir, file, hir::Path::single("sample"));
    hir.resolve(&ctx);

    let mut object = backend::Object::new("x86_64-unknown-linux-gnu");
    for (handle, function) in hir.functions.iter_enumerated() {
        let function = function.read().unwrap();
        object.declare_function(handle, &function.path, &function.signature);
    }

    for (handle, function) in hir.functions.iter_enumerated() {
        let function = function.read().unwrap();
        let body = &hir.bodies[function.body].read().unwrap();
        object.build_function(handle, |builder| {
            let value = body.expression.build(builder);
            builder.0.ins().return_(&value);
        });
    }

    ctx.diag.emit();

    let object = object.object.finish();
    std::fs::write("foo.o", object.emit().unwrap()).unwrap();
}
