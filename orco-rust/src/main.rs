pub mod hir;
pub use hir::Hir;

pub mod backend;
use backend::cl::InstBuilder;

fn parse_file(hir: &mut Hir, filepath: impl AsRef<std::path::Path>, modpath: hir::Path) {
    let source = std::fs::read_to_string(filepath).unwrap();
    let file = syn::parse_str::<syn::File>(&source).unwrap();

    for item in file.items {
        match item {
            syn::Item::Fn(r#fn) => {
                let path = modpath.clone().join(&r#fn.sig.ident);
                let body = hir
                    .bodies
                    .insert(hir::Body::new(hir::Block::parse(&r#fn.block, &path).into()));
                hir.functions.insert(hir::Function {
                    path,
                    signature: r#fn.sig.into(),
                    body,
                });
            }
            _ => todo!(),
        }
    }
}

fn build_literal(
    builder: &mut backend::FunctionBuilder,
    literal: &hir::Literal,
) -> backend::cl::Value {
    match &literal.lit {
        syn::Lit::Str(lit_str) => todo!(),
        syn::Lit::ByteStr(lit_byte_str) => todo!(),
        syn::Lit::CStr(lit_cstr) => todo!(),
        syn::Lit::Byte(lit_byte) => todo!(),
        syn::Lit::Char(lit_char) => todo!(),
        syn::Lit::Int(int) => builder
            .0
            .ins()
            .iconst(backend::cl::types::I32, int.base10_parse::<i64>().unwrap()),
        syn::Lit::Float(lit_float) => todo!(),
        syn::Lit::Bool(lit_bool) => todo!(),
        syn::Lit::Verbatim(literal) => todo!(),
        _ => todo!(),
    }
}

fn build_expression(
    builder: &mut backend::FunctionBuilder,
    expr: &hir::Expression,
) -> Vec<backend::cl::Value> {
    match expr {
        hir::Expression::Literal(literal) => vec![build_literal(builder, literal)],
        hir::Expression::Block(block) => {
            for statement in &block.statements {
                build_expression(builder, statement);
            }
            block
                .tail
                .as_ref()
                .map_or_else(Vec::new, |expr| build_expression(builder, expr.as_ref()))
        }
    }
}

fn main() {
    let file = "samples/simple.rs";
    let mut hir = hir::Hir::new();
    parse_file(&mut hir, file, hir::Path::empty()); // hir::Path::single("sample")
    hir.resolve();

    let mut object = backend::Object::new("x86_64-unknown-linux-gnu");
    for (handle, function) in hir.functions.iter_with_handles() {
        object.declare_function(handle, &function.path, &function.signature);
    }

    for (handle, function) in hir.functions.iter_with_handles() {
        let body = &hir.bodies[function.body];
        object.build_function(handle, |builder| {
            let value = build_expression(builder, &body.expression);
            builder.0.ins().return_(&value);
        });
    }

    let object = object.object.finish();
    std::fs::write("foo.o", object.emit().unwrap()).unwrap();
}
