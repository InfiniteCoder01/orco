#[path = "test_utils.rs"]
mod test_utils;
use orco::*;
use test_utils::*;

#[test]
#[should_panic]
fn test_diagnostics() {
    assert_stderr(
        |codebase| {
            let file = codebase.add_file("test.rs", "ERROR");
            codebase.report(
                Diagnostic::new(Severity::Error)
                    .with_message("Example error")
                    .with_code("Example")
                    .with_labels(vec![Label::primary(file, 0..5).with_message("Here!")])
                    .with_notes(vec!["Note...".to_owned()]),
            )
        },
        "",
    );
}

#[test]
#[should_panic]
fn test_fallback_diagnostics() {
    assert_stderr(
        |codebase| {
            codebase.report(
                Diagnostic::new(Severity::Error)
                    .with_message("Example error")
                    .with_code("Example")
                    .with_labels(vec![Label::primary(0, 0..5).with_message("Here!")])
                    .with_notes(vec!["Note...".to_owned()]),
            );
        },
        "",
    );
}
