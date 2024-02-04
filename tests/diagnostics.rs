#[path = "test_utils.rs"]
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
