use super::*;
use codespan_reporting::files::{Error, Files};
use std::{path::Path, sync::Arc};

/// A unique identifier for a source file
pub type FileId = usize;

impl Codebase {
    /// Add a new source file to the codebase, providing the source code manually
    pub fn add_file(&self, path: impl AsRef<Path>, source: &str) -> <Self as Files<'_>>::FileId {
        let mut files = self.files.lock().unwrap();
        files.push(File::new(path, source));
        files.len() - 1
    }

    /// Add a new source file to the codebase, reading it from the file system
    pub fn read_file(&self, path: &Path) -> Result<<Self as Files<'_>>::FileId, std::io::Error> {
        Ok(self.add_file(path, &std::fs::read_to_string(path)?))
    }

    /// Get the source of a file, report a bug, if it doesn't exist
    pub fn get_file(&self, id: FileId) -> Option<Arc<str>> {
        match self.source(id) {
            Ok(source) => Some(source),
            Err(err) => {
                self.report(
                    Diagnostic::bug()
                        .with_message(err.to_string())
                        .with_notes(vec![format!("With file_id of {}", id)]),
                );
                None
            }
        }
    }
}

impl<'a> Files<'a> for Codebase {
    type FileId = FileId;
    type Name = <codebase::File as Files<'a>>::Name;
    type Source = <codebase::File as Files<'a>>::Source;

    fn name(&self, id: Self::FileId) -> Result<Self::Name, Error> {
        self.files
            .lock()
            .unwrap()
            .get(id)
            .ok_or(Error::FileMissing)?
            .name(())
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, Error> {
        self.files
            .lock()
            .unwrap()
            .get(id)
            .ok_or(Error::FileMissing)?
            .source(())
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        self.files
            .lock()
            .unwrap()
            .get(id)
            .ok_or(Error::FileMissing)?
            .line_index((), byte_index)
    }

    fn line_range(
        &'a self,
        id: Self::FileId,
        line_index: usize,
    ) -> Result<std::ops::Range<usize>, Error> {
        self.files
            .lock()
            .unwrap()
            .get(id)
            .ok_or(Error::FileMissing)?
            .line_range((), line_index)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A source file
pub struct File {
    path: Arc<FileName>,
    source: Arc<str>,
    line_starts: Vec<usize>,
}

impl File {
    /// Create a new source file.
    pub fn new(path: impl AsRef<Path>, source: &str) -> Self {
        let line_starts = codespan_reporting::files::line_starts(source).collect();
        Self {
            path: Arc::new(FileName(path.as_ref().to_path_buf())),
            source: source.into(),
            line_starts,
        }
    }

    /// Return the starting byte index of the line with the specified line index.
    /// Convenience method that already generates errors if necessary.
    fn line_start(&self, line_index: usize) -> Result<usize, Error> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => Ok(self
                .line_starts
                .get(line_index)
                .cloned()
                .expect("failed despite previous check")),
            Ordering::Equal => Ok(self.source.as_ref().len()),
            Ordering::Greater => Err(Error::LineTooLarge {
                given: line_index,
                max: self.line_starts.len() - 1,
            }),
        }
    }
}

impl std::fmt::Debug for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("File")
            .field("path", &self.path)
            .field("source", &self.source)
            .finish()
    }
}

impl Files<'_> for File {
    type FileId = ();
    type Name = Arc<FileName>;
    type Source = Arc<str>;

    fn name(&self, (): ()) -> Result<Self::Name, Error> {
        Ok(self.path.clone())
    }

    fn source(&self, (): ()) -> Result<Self::Source, Error> {
        Ok(self.source.clone())
    }

    fn line_index(&self, (): (), byte_index: usize) -> Result<usize, Error> {
        Ok(self
            .line_starts
            .binary_search(&byte_index)
            .unwrap_or_else(|next_line| next_line - 1))
    }

    fn line_range(&self, (): (), line_index: usize) -> Result<std::ops::Range<usize>, Error> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + 1)?;

        Ok(line_start..next_line_start)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A wrapped Path
pub struct FileName(pub std::path::PathBuf);

impl std::fmt::Display for FileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl std::ops::Deref for FileName {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
