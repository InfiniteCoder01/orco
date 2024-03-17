use orco::*;

pub fn assert_stderr(f: impl FnOnce(Codebase), target: &str) {
    use gag::BufferRedirect;
    use orco::diagnostic::StandardStream;

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
