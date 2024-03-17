mod test_utils;
use orco::*;
use test_utils::*;

#[test]
#[should_panic]
fn diagnostics() {
    assert_stderr(
        |codebase| {
            let file = codebase.add_file("test.rs", "ERROR");
            codebase.report(
                Diagnostic::error()
                    .with_message("Example error")
                    .with_code("Example")
                    .with_label(Label::primary(file, 0..5).with_message("Here!"))
                    .with_note("Note..."),
            )
        },
        "",
    );
}

#[test]
#[should_panic]
fn fallback() {
    assert_stderr(
        |codebase| {
            codebase.report(
                Diagnostic::error()
                    .with_message("Example error")
                    .with_code("Example")
                    .with_label(Label::primary(0, 0..5).with_message("Here!"))
                    .with_note("Note..."),
            );
        },
        "",
    );
}

#[test]
#[should_panic]
fn multithreaded() {
    assert_stderr(
        |codebase| {
            let file = codebase.add_file("test.rs", "ERROR");
            let codebase = &codebase;
            std::thread::scope(|s| {
                for i in 1..2 {
                    s.spawn(move || {
                        codebase.report(
                            Diagnostic::error()
                                .with_message(format!("Example error #{i}"))
                                .with_code("Example")
                                .with_label(Label::primary(file, 0..5).with_message("Here!"))
                                .with_note("Note..."),
                        );
                    });
                }
            });
        },
        "",
    );
}

#[test]
#[should_panic]
fn fallback_multithreaded() {
    assert_stderr(
        |codebase| {
            let codebase = &codebase;
            std::thread::scope(|s| {
                for i in 1..2 {
                    s.spawn(move || {
                        codebase.report(
                            Diagnostic::error()
                                .with_message(format!("Example error #{i}"))
                                .with_code("Example")
                                .with_label(Label::primary(i, 0..5).with_message("Here!"))
                                .with_note("Note..."),
                        );
                    });
                }
            });
        },
        "",
    );
}
