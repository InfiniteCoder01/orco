mod test_utils;
use codespan_reporting::files::Files;
use orco::{codebase::FileName, *};
use test_utils::*;

#[test]
fn files() {
    let codebase = Codebase::default();
    let foo_id = codebase.add_file("foo.rs", "foo");
    let bar_id = codebase.add_file("bar.rs", "bar");
    assert_eq!(codebase.get_file(foo_id), Some("foo".into()));
    assert_eq!(codebase.get_file(bar_id), Some("bar".into()));
    assert_eq!(codebase.source(foo_id).unwrap(), "foo".into());
    assert_eq!(codebase.source(bar_id).unwrap(), "bar".into());
    assert_eq!(
        codebase.name(foo_id).unwrap(),
        std::sync::Arc::new(FileName("foo.rs".into()))
    );
    assert_eq!(
        codebase.name(bar_id).unwrap(),
        std::sync::Arc::new(FileName("bar.rs".into()))
    );
}

#[test]
fn non_existing() {
    let codebase = Codebase::default();
    codebase.add_file("foo.rs", "foo");
    codebase.add_file("bar.rs", "bar");
    let baz_id = 42;
    assert_eq!(codebase.get_file(baz_id), None);
    assert!(codebase.source(baz_id).is_err());
    assert!(codebase.name(baz_id).is_err());
}

#[test]
#[should_panic]
fn errors() {
    assert_stderr(
        |codebase| {
            codebase.add_file("foo.rs", "foo");
            codebase.add_file("bar.rs", "bar");
            let baz_id = 42;
            codebase.get_file(baz_id);
        },
        "",
    )
}

#[test]
fn multithreaded() {
    let test_files = [("foo.rs", "Foo"), ("bar.rs", "bar"), ("baz.rs", "baz")];
    assert_stderr(
        move |codebase| {
            let codebase = &codebase;
            let file_ids = std::sync::Mutex::new(std::collections::HashMap::new());
            std::thread::scope(|s| {
                for (name, content) in test_files {
                    let file_ids = &file_ids;
                    s.spawn(move || {
                        let id = codebase.add_file(name, content);
                        assert_eq!(codebase.get_file(id), Some(content.into()));
                        file_ids.lock().unwrap().insert(id, content);
                    });
                }
            });
            std::thread::scope(|s| {
                for (&id, &content) in file_ids.lock().unwrap().iter() {
                    s.spawn(move || {
                        assert_eq!(codebase.get_file(id), Some(content.into()));
                    });
                }
            });
        },
        "",
    )
}
