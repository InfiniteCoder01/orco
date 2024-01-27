use super::*;
use codespan_reporting::files::Files;

pub use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
pub use codespan_reporting::term::{
    termcolor::{ColorChoice, StandardStream},
    Config,
};

impl<'a> Codebase<'a> {
    /// Report a diagnostic
    pub fn report(&self, diagnostic: Diagnostic<<Self as Files<'a>>::FileId>) {
        match &self.diagnostic_writer {
            DiagnosticWriter::StandardStream(writer, config) => {
                if let Err(err) =
                    codespan_reporting::term::emit(&mut writer.lock(), config, self, &diagnostic)
                {
                    eprintln!("Failed to render diagnostic: {err}");
                    eprintln!("Original Diagnostic:");

                    eprintln!(
                        "{}{}: {}",
                        match diagnostic.severity {
                            Severity::Bug => "BUG",
                            Severity::Error => "Error",
                            Severity::Warning => "Warning",
                            Severity::Note => "Note",
                            Severity::Help => "Help",
                        },
                        diagnostic
                            .code
                            .map_or_else(String::new, |code| format!(" [{code}]")),
                        diagnostic.message
                    );
                    for label in diagnostic.labels {
                        eprintln!("   {} at <Unknown>", label.message);
                    }
                    for note in diagnostic.notes {
                        eprintln!(" = {}", note);
                    }
                }
            }
        }
    }
}

/// The diagnostic writer
pub enum DiagnosticWriter {
    /// Render diagnostics to console using codespan
    StandardStream(StandardStream, Config),
}

impl Default for DiagnosticWriter {
    fn default() -> Self {
        Self::StandardStream(StandardStream::stderr(ColorChoice::Auto), Config::default())
    }
}

impl std::fmt::Debug for DiagnosticWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StandardStream(_, _) => f.debug_struct("StandardStream").finish(),
        }
    }
}
