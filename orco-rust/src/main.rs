pub mod hir;
pub use hir::Hir;

pub mod diagnostic;
pub use diagnostic::DiagCtx;

pub mod backend;
use backend::cl::InstBuilder;

pub struct Context {
    pub diag: crate::DiagCtx,
    pub hir: Hir,
    pub path: hir::Path,
}

fn parse_file(ctx: &mut Context, filepath: impl AsRef<std::path::Path>) {
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
                let back = if r#fn.sig.ident.to_string() == "main" {
                    Some(std::mem::replace(
                        &mut ctx.path,
                        hir::Path::single(&r#fn.sig.ident),
                    ))
                } else {
                    ctx.path.push(&r#fn.sig.ident);
                    None
                };

                let signature = hir::Signature::parse(ctx, r#fn.sig);
                let body = hir::Body::new(hir::Block::parse(ctx, &r#fn.block).into());
                let body = ctx.hir.bodies.push_get_id(body.into());
                ctx.hir.functions.push_get_id(
                    hir::Function {
                        path: ctx.path.clone(),
                        signature,
                        body,
                    }
                    .into(),
                );

                if let Some(back) = back {
                    ctx.path = back;
                } else {
                    ctx.path.pop();
                }
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
        path: hir::Path::single("sample"),
    };
    parse_file(&mut ctx, file);
    ctx.hir.resolve(&ctx);

    let mut object = backend::Object::new("x86_64-unknown-linux-gnu");
    for (handle, function) in ctx.hir.functions.iter_enumerated() {
        let function = function.read().unwrap();
        object.declare_function(handle, &function.path, &function.signature);
    }

    for (handle, function) in ctx.hir.functions.iter_enumerated() {
        let function = function.read().unwrap();
        let body = &ctx.hir.bodies[function.body].read().unwrap();
        object.build_function(handle, |builder| {
            let value = body.expression.build(builder);
            builder.0.ins().return_(&value);
        });
    }

    ctx.diag.emit();

    let object = object.object.finish();
    std::fs::write("foo.o", object.emit().unwrap()).unwrap();
}
