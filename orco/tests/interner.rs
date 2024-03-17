mod test_utils;
use orco::*;
use test_utils::*;

#[test]
fn interner() {
    assert_stderr(
        |codebase| {
            let foo1 = codebase.interned("foo1");
            let bar1 = codebase.interned("bar1");
            assert_eq!(codebase.resolve_symbol(foo1), "foo1");
            assert_eq!(codebase.resolve_symbol(bar1), "bar1");
        },
        "",
    );
}

#[test]
#[should_panic]
fn interner_fail() {
    assert_stderr(
        |codebase| {
            use string_interner::symbol::Symbol as _;
            let baz1 = Symbol::try_from_usize(993).unwrap(); // An easteregg BTW.
            assert_eq!(codebase.resolve_symbol(baz1), "<error>");
        },
        "",
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

#[test]
fn multithreaded() {
    let test_symbols = ["foo", "bar", "baz"];
    assert_stderr(
        move |codebase| {
            let codebase = &codebase;
            let symbol_ids = std::sync::Mutex::new(std::collections::HashMap::new());
            std::thread::scope(|s| {
                for symbol in test_symbols {
                    let symbol_ids = &symbol_ids;
                    s.spawn(move || {
                        let id = codebase.interned(symbol);
                        assert_eq!(codebase.resolve_symbol(id), symbol);
                        symbol_ids.lock().unwrap().insert(id, symbol);
                    });
                }
            });
            std::thread::scope(|s| {
                for (&id, &symbol) in symbol_ids.lock().unwrap().iter() {
                    s.spawn(move || {
                        assert_eq!(codebase.resolve_symbol(id), symbol);
                    });
                }
            });
        },
        "",
    )
}
