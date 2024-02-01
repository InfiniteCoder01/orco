use gag::BufferRedirect;
use orco::{diagnostic::StandardStream, *};

fn assert_stderr(f: impl FnOnce(Codebase), target: &str) {
    let capture = BufferRedirect::stderr().unwrap();
    let codebase = Codebase::new(diagnostic::DiagnosticWriter::StandardStream(
        StandardStream::stderr(diagnostic::ColorChoice::Never),
        diagnostic::Config::default(),
    ));
    f(codebase);
    use std::io::Read;
    let mut stderr = String::new();
    capture.into_inner().read_to_string(&mut stderr).unwrap();
    assert_eq!(stderr, target);
}

#[test]
fn test_interner() {
    assert_stderr(
        |codebase| {
            use string_interner::symbol::Symbol as _;
            let foo1 = codebase.interned("foo1");
            let bar1 = codebase.interned("bar1");
            let baz1 = Symbol::try_from_usize(993).unwrap(); // An easteregg BTW.
            assert_eq!(codebase.resolve_symbol(foo1), "foo1");
            assert_eq!(codebase.resolve_symbol(bar1), "bar1");
            assert_eq!(codebase.resolve_symbol(baz1), "<error>");
        },
        "bug: Failed to resolve symbol\n = Symbol: 993\n\n",
    );
}

#[test]
fn test_parse_path() {
    let mut codebase = Codebase::default();
    assert_eq!(
        codebase.parse_path("foo::bar::baz"),
        vec![
            codebase.interned("foo"),
            codebase.interned("bar"),
            codebase.interned("baz")
        ]
    );
}
