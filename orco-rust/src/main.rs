pub mod hir;
pub use hir::Hir;

pub mod diagnostic;
pub use diagnostic::DiagCtx;

pub mod backend;
use backend::cl::InstBuilder;

pub struct Context {
    pub diag: crate::DiagCtx,
    pub hir: Hir,
}

fn parse_file(ctx: &mut Context, filepath: impl AsRef<std::path::Path>, modpath: hir::Path) {
    let filepath = filepath.as_ref();
    let source = std::fs::read_to_string(filepath).unwrap();
    ctx.diag
        .set_source(miette::NamedSource::new(filepath.to_string_lossy(), source.clone()).into());
    let file = match syn::parse_str::<syn::File>(&source) {
        Ok(file) => file,
        Err(err) => {
            ctx.diag.syntax_error(err);
            return;
        }
    };

    for item in file.items {
        match item {
            syn::Item::Fn(r#fn) => {
                let path = modpath.clone().join(&r#fn.sig.ident);
                let body = ctx
                    .hir
                    .bodies
                    .insert(hir::Body::new(hir::Block::parse(&r#fn.block, &path).into()));
                ctx.hir.functions.insert(hir::Function {
                    path,
                    signature: r#fn.sig.into(),
                    body,
                });
            }
            _ => todo!(),
        }
    }
}

fn main() {
    let file = "orco-rust/samples/simple.rs";
    let mut ctx = Context {
        diag: DiagCtx::new(),
        hir: hir::Hir::new(),
    };
    parse_file(&mut ctx, file, hir::Path::empty()); // hir::Path::single("sample")
    ctx.hir.resolve();

    let mut object = backend::Object::new("x86_64-unknown-linux-gnu");
    for (handle, function) in ctx.hir.functions.iter_with_handles() {
        object.declare_function(handle, &function.path, &function.signature);
    }

    for (handle, function) in ctx.hir.functions.iter_with_handles() {
        let body = &ctx.hir.bodies[function.body];
        object.build_function(handle, |builder| {
            let value = body.expression.build(builder);
            builder.0.ins().return_(&value);
        });
    }

    ctx.diag.emit();

    let object = object.object.finish();
    std::fs::write("foo.o", object.emit().unwrap()).unwrap();
}
