use super::*;
use std::num::NonZeroU16;

/// Parse a type, error if there is no
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Spanned<ir::Type> {
    let start = parser.span().1.start;
    let mut r#type = if let Some(r#type) = parser.expect_ident("type") {
        if let Some(bytes) = numeric_type_size(&r#type, "i") {
            ir::Type::Int(bytes)
        } else if let Some(bytes) = numeric_type_size(&r#type, "u") {
            ir::Type::Unsigned(bytes)
        } else if let Some(bytes) = numeric_type_size(&r#type, "f") {
            ir::Type::Float(bytes)
        } else {
            match r#type.as_str() {
                "bool" => ir::Type::Bool,
                "char" => ir::Type::Char,
                _ => ir::Type::Custom(r#type.inner),
            }
        }
    } else {
        parser.expected_error("a type");
        ir::Type::Error
    };
    while parser.match_operator(Operator::Star) {
        r#type = ir::Type::Pointer(Box::new(r#type));
    }
    parser.wrap_span(r#type, start)
}

fn numeric_type_size(name: &str, prefix: &str) -> Option<NonZeroU16> {
    name.strip_prefix(prefix)
        .and_then(|bits| bits.parse::<u32>().ok())
        .and_then(|bits| (bits / 8).try_into().ok())
        .and_then(NonZeroU16::new)
}
