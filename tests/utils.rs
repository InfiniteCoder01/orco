#[path = "test_utils.rs"]
mod test_utils;
use orco::*;
use test_utils::*;

#[test]
fn interner() {
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
fn parse_path() {
    let codebase = Codebase::default();
    assert_eq!(
        codebase.parse_path("foo::bar::baz"),
        vec![
            codebase.interned("foo"),
            codebase.interned("bar"),
            codebase.interned("baz")
        ]
    );
}
